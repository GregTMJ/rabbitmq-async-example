use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::types::{Json, JsonValue};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct FailTable {
    pub id: i32,
    pub application_id: String,
    pub serhub_request_id: String,
    pub service_id: i32,
    pub system_id: i32,
    pub error_type: Option<String>,
    pub error_message: Option<String>,
    pub error_traceback: Option<String>,
    pub data: Option<Json<JsonValue>>,
    pub timestamptz_saved: DateTime<Utc>,
}
