use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Services {
    pub id: i32,
    pub name: String,
    pub exchange: String,
    pub queue: String,
    pub routing_key: String,
    pub cache_fields: String,
    pub cache_expiration: Option<String>,
    pub timeout: i32,
}
