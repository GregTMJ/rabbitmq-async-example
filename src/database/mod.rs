use sqlx::postgres::{PgConnection, PgPoolOptions};
use sqlx::{Connection, Pool, Postgres};

use crate::CustomProjectErrors;

pub mod functions;
pub mod models;

pub async fn check_connection(database_url: &str) -> Result<(), CustomProjectErrors> {
    PgConnection::connect(database_url)
        .await
        .map_err(|_| CustomProjectErrors::DatabaseHealthCheckError)?;
    Ok(())
}

pub async fn get_connection_pool(
    database_url: &str,
    max_connection: u8,
) -> Result<Pool<Postgres>, CustomProjectErrors> {
    PgPoolOptions::new()
        .max_connections(max_connection as u32)
        .connect(database_url)
        .await
        .map_err(|e| CustomProjectErrors::DatabaseConnectionError(e.to_string()))
}
