use std::fmt::Debug;
use std::sync::Arc;

use futures::StreamExt;
use lapin::Channel;
use lapin::message::Delivery;
use lapin::options::{
    BasicConsumeOptions, ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions,
};
use lapin::types::FieldTable;
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
        F: Fn(Delivery, Arc<Pool<Postgres>>, Arc<Channel>) -> Fut + Sync + Send,
        Fut: Future<Output = Result<(), CustomProjectErrors>>,
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
            .map_err(|err| CustomProjectErrors::RMQChannelError(err.to_string()))?;

        while let Some(delivery) = consumer.next().await {
            match delivery {
                Ok(msg) => {
                    let result = callback(
                        msg,
                        Arc::clone(&self.sql_connection_pool),
                        Arc::clone(&self.channel),
                    )
                    .await;
                    if result.is_err() {
                        error!(
                            "Got an error while working with {callback_name}: {result:?}"
                        );
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
