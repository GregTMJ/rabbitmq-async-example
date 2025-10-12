use std::fmt::Debug;
use std::sync::Arc;

use futures::StreamExt;
use lapin::ErrorKind;
use lapin::options::{
    BasicAckOptions, BasicConsumeOptions, ExchangeDeclareOptions, QueueBindOptions,
    QueueDeclareOptions,
};
use lapin::protocol::basic::AMQPProperties;
use lapin::types::FieldTable;
use lapin::{Channel, Connection, ConnectionProperties, Error as RmqError};
use log::{error, info};
use sqlx::{Pool, Postgres};

use crate::errors::CustomProjectErrors;
use crate::rmq::schemas::{Exchange, Queue};

#[derive(Debug)]
pub struct RmqConnection {
    pub channel: Arc<Channel>,
    pub sql_connection_pool: Arc<Pool<Postgres>>,
}

impl RmqConnection {
    pub async fn bind_consumer<'a>(
        &self,
        exchange: &'a Exchange<'a>,
        queue: &'a Queue<'a>,
    ) -> Result<(), CustomProjectErrors> {
        info!("Binding queue to channel");
        self.channel
            .exchange_declare(
                exchange.name,
                exchange.exchange_type.clone(),
                ExchangeDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await
            .map_err(|msg| CustomProjectErrors::RMQChannelError(msg.to_string()))?;
        self.channel
            .queue_declare(
                queue.name,
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await
            .map_err(|msg| CustomProjectErrors::RMQChannelError(msg.to_string()))?;
        self.channel
            .queue_bind(
                queue.name,
                exchange.name,
                queue.routing_key,
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(|msg| CustomProjectErrors::RMQChannelError(msg.to_string()))?;
        info!("Binding successful");
        Ok(())
    }

    pub async fn start_consumer<F, Fut>(
        &self,
        exchange: Exchange<'_>,
        queue: Queue<'_>,
        callback: F,
        callback_name: &str,
    ) -> Result<(), CustomProjectErrors>
    where
        F: Fn(Vec<u8>, AMQPProperties, Arc<Pool<Postgres>>, Arc<Channel>) -> Fut + Sync + Send,
        Fut: Future<Output = Result<(), CustomProjectErrors>> + Send,
    {
        self.bind_consumer(&exchange, &queue).await?;
        info!("Starting consuming {callback_name}");
        let mut consumer = self
            .channel
            .basic_consume(
                queue.name,
                callback_name,
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(|msg| CustomProjectErrors::RMQChannelError(msg.to_string()))?;

        let channel = Arc::clone(&self.channel);
        while let Some(delivery) = consumer.next().await {
            match delivery {
                Ok(msg) => {
                    channel
                        .basic_ack(msg.delivery_tag, BasicAckOptions::default())
                        .await
                        .map_err(|msg| CustomProjectErrors::RMQChannelError(msg.to_string()))?;
                    let result = callback(
                        msg.data,
                        msg.properties,
                        self.sql_connection_pool.clone(),
                        channel.clone(),
                    )
                    .await;
                    if result.is_err() {
                        error!("Got an error while working with {callback_name}: {result:?}");
                    }
                }
                Err(msg) => {
                    error!("Got an rmq error {msg}")
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct RmqConnectionBuilder {
    rmq_url: Option<String>,
    sql_connection_pool: Option<Pool<Postgres>>,
}

impl RmqConnectionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn rmq_url(mut self, url: String) -> Self {
        self.rmq_url = Some(url);
        self
    }

    pub fn sql_pool(mut self, pool: Pool<Postgres>) -> Self {
        self.sql_connection_pool = Some(pool);
        self
    }

    pub async fn build(self) -> Result<RmqConnection, CustomProjectErrors> {
        let rmq_url = self
            .rmq_url
            .ok_or(RmqError::from(ErrorKind::NoConfiguredExecutor))
            .map_err(|msg| CustomProjectErrors::RMQConnectionError(msg.to_string()))?;
        let sql_connection_pool = self
            .sql_connection_pool
            .ok_or(RmqError::from(ErrorKind::NoConfiguredExecutor))
            .map_err(|msg| CustomProjectErrors::DatabaseConnectionError(msg.to_string()))?;

        info!("Starting RMQ connection");
        let conn = Connection::connect(&rmq_url, ConnectionProperties::default())
            .await
            .map_err(|msg| CustomProjectErrors::RMQConnectionError(msg.to_string()))?;
        info!("RMQ Connection established");

        info!("Opening channel");
        let channel = conn
            .create_channel()
            .await
            .map_err(|msg| CustomProjectErrors::RMQChannelCreationError(msg.to_string()))?;
        info!("RMQ channel ready to handle");

        Ok(RmqConnection {
            channel: Arc::new(channel),
            sql_connection_pool: Arc::new(sql_connection_pool),
        })
    }
}
