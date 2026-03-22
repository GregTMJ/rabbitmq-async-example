use std::fmt::Debug;
use std::sync::Arc;

use futures::StreamExt;
use lapin::Channel;
use lapin::Connection as AMQPConnection;
use lapin::message::Delivery;
use lapin::options::{
    BasicAckOptions, BasicConsumeOptions, BasicNackOptions, BasicQosOptions,
    ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions,
};
use lapin::types::FieldTable;
use log::{error, info};
use sqlx::{Pool, Postgres};

use crate::prelude::{CustomProjectErrors, Exchange, Queue};

#[derive(Debug)]
pub struct RmqConnection {
    connection: Arc<AMQPConnection>,
    pub sql_connection_pool: Arc<Pool<Postgres>>,
}

impl RmqConnection {
    pub fn new(
        connection: AMQPConnection,
        sql_connection_pool: Pool<Postgres>,
    ) -> Self {
        Self {
            connection: Arc::new(connection),
            sql_connection_pool: Arc::new(sql_connection_pool),
        }
    }

    async fn bind_consumer<'a>(
        &self,
        channel: &Channel,
        exchange: &'a Exchange<'a>,
        queue: &'a Queue<'a>,
    ) -> Result<(), CustomProjectErrors> {
        info!("Binding queue {} to channel", queue.name);
        channel
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
        channel
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
        channel
            .queue_bind(
                queue.name,
                exchange.name,
                queue.routing_key,
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(|msg| CustomProjectErrors::RMQChannelError(msg.to_string()))?;
        info!("Binding queue {} successful", queue.name);
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
        let channel =
            self.connection.create_channel().await.map_err(|e| {
                CustomProjectErrors::RMQChannelCreationError(e.to_string())
            })?;

        channel
            .basic_qos(10, BasicQosOptions::default())
            .await
            .map_err(|e| CustomProjectErrors::RMQChannelError(e.to_string()))?;

        self.bind_consumer(&channel, &exchange, &queue).await?;
        info!("Starting consuming {callback_name}");
        let mut consumer = channel
            .basic_consume(
                queue.name,
                callback_name,
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(|err| CustomProjectErrors::RMQChannelError(err.to_string()))?;

        let channel = Arc::new(channel);

        while let Some(delivery) = consumer.next().await {
            match delivery {
                Ok(msg) => {
                    let delivery_tag = msg.delivery_tag;
                    let result = callback(
                        msg,
                        Arc::clone(&self.sql_connection_pool),
                        Arc::clone(&channel),
                    )
                    .await;
                    if let Err(ref e) = result {
                        error!("Error in {callback_name}: {e}");
                        if let Err(nack_err) = channel
                            .basic_nack(delivery_tag, BasicNackOptions::default())
                            .await
                        {
                            error!(
                                "Failed to nack delivery {delivery_tag}: {nack_err}"
                            );
                        }
                    } else if let Err(ack_err) = channel
                        .basic_ack(delivery_tag, BasicAckOptions::default())
                        .await
                    {
                        error!("Failed to ack delivery {delivery_tag}: {ack_err}");
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
