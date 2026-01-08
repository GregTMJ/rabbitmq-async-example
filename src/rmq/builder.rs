use std::fmt::Debug;

use lapin::{Connection, ConnectionProperties, Error as RmqError};
use lapin::{ErrorKind, RecoveryConfig};
use log::info;
use sqlx::{Pool, Postgres};

use crate::errors::CustomProjectErrors;
use crate::rmq::handlers::RmqConnection;

#[derive(Debug, Default)]
pub struct RmqConnectionBuilder {
    rmq_url: Option<String>,
    sql_connection_pool: Option<Pool<Postgres>>,
}

impl RmqConnectionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_rmq_url(
        mut self,
        url: String,
    ) -> Self {
        self.rmq_url = Some(url);
        self
    }

    pub fn with_sql_pool(
        mut self,
        pool: Pool<Postgres>,
    ) -> Self {
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
            .map_err(|msg| {
                CustomProjectErrors::DatabaseConnectionError(msg.to_string())
            })?;

        info!("---- Starting RMQ connection ----");
        let conn = Connection::connect(
            &rmq_url,
            ConnectionProperties::default()
                .with_experimental_recovery_config(RecoveryConfig::full()),
        )
        .await
        .map_err(|msg| CustomProjectErrors::RMQConnectionError(msg.to_string()))?;
        info!("---- RMQ Connection established ----");

        info!("---- Opening channel ----");
        let channel = conn.create_channel().await.map_err(|msg| {
            CustomProjectErrors::RMQChannelCreationError(msg.to_string())
        })?;
        info!("---- RMQ channel ready to handle ----");

        Ok(RmqConnection::new(channel, sql_connection_pool))
    }
}
