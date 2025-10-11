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
    pub application_data: Json<JsonValue>,
    pub timestamptz_saved: DateTime<Utc>,
}
