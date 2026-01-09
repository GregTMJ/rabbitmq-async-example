use lapin::RecoveryConfig;
use lapin::{Connection, ConnectionProperties};
use log::info;
use sqlx::{Pool, Postgres};

use crate::database::get_connection_pool;
use crate::errors::CustomProjectErrors;
use crate::rmq::handlers::RmqConnection;

#[derive(Clone, Default)]
struct SqlConnection {
    url: String,
    max_pool_size: u8,
}

#[derive(Default)]
pub struct ConnectionBuilder {
    rmq_url: String,
    sql_connection: SqlConnection,
}

impl ConnectionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_rmq_url(
        mut self,
        url: String,
    ) -> Self {
        self.rmq_url = url;
        self
    }

    pub fn with_sql_pool(
        mut self,
        url: String,
        max_pool_size: u8,
    ) -> Self {
        self.sql_connection = SqlConnection { url, max_pool_size };
        self
    }

    async fn build_rmq_connection(&self) -> Result<Connection, CustomProjectErrors> {
        let rmq_url = &self.rmq_url;
        Connection::connect(
            rmq_url,
            ConnectionProperties::default()
                .with_experimental_recovery_config(RecoveryConfig::full()),
        )
        .await
        .map_err(|e| CustomProjectErrors::RMQConnectionError(e.to_string()))
    }

    async fn build_sql_connection_pool(
        &self
    ) -> Result<Pool<Postgres>, CustomProjectErrors> {
        let sql_connection = &self.sql_connection;
        get_connection_pool(&sql_connection.url, sql_connection.max_pool_size).await
    }

    pub async fn build(self) -> Result<RmqConnection, CustomProjectErrors> {
        info!("---- Opening Database Pool Connection ----");
        let sql_connection_pool = self.build_sql_connection_pool().await?;
        info!("---- Pool ready for usage ----");

        info!("---- Starting RMQ connection ----");
        let rmq_connection = self.build_rmq_connection().await?;
        info!("---- RMQ Connection established ----");

        info!("---- Opening channel ----");
        let channel = rmq_connection.create_channel().await.map_err(|msg| {
            CustomProjectErrors::RMQChannelCreationError(msg.to_string())
        })?;
        info!("---- RMQ channel ready to handle ----");

        Ok(RmqConnection::new(channel, sql_connection_pool))
    }
}
