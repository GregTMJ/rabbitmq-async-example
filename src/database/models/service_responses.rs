use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ApplicationResponses {
    pub id: i32,
    pub application_id: String,
    pub serhub_request_id: String,
    pub service_id: i32,
    pub system_id: i32,
    pub data: String,
    pub data_hash: Option<String>,
    pub is_cache: bool,
    pub timestamptz_saved: DateTime<Utc>,
}
