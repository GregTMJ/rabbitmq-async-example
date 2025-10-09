use sqlx::Error as SqlError;
use sqlx::postgres::{PgConnection, PgPoolOptions};
use sqlx::{Connection, Pool, Postgres};

pub mod functions;
pub mod models;

pub async fn check_connection(database_url: &str) -> Result<(), SqlError> {
    PgConnection::connect(database_url).await?;
    Ok(())
}

pub async fn get_connection_pool(
    database_url: &str,
    max_connection: u8,
) -> Result<Pool<Postgres>, SqlError> {
    PgPoolOptions::new()
        .max_connections(max_connection as u32)
        .connect(database_url)
        .await
}
