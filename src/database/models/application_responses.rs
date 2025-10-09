use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::types::{Json, JsonValue};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ApplicationResponses {
    pub id: i32,
    pub application_id: String,
    pub serhub_request_id: String,
    pub service_id: i32,
    pub system_id: i32,
    pub is_cache: bool,
    pub status: String,
    pub status_description: Json<JsonValue>,
    pub response: Json<JsonValue>,
    pub target: Json<JsonValue>,
    pub timestamptz_saved: DateTime<Utc>,
}
