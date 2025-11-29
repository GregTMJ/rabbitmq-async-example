use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::types::{Json, JsonValue, Uuid};
use std::str::FromStr;

use crate::errors::CustomProjectErrors;
use crate::mapping::schemas::ServiceResponse;

#[derive(Debug, Serialize, Deserialize, FromRow, Default)]
pub struct ApplicationResponses {
    id: i32,
    pub application_id: String,
    pub serhub_request_id: String,
    pub service_id: i32,
    pub system_id: i32,
    pub is_cache: bool,
    pub status: String,
    pub status_description: Json<JsonValue>,
    pub response: Option<Json<JsonValue>>,
    pub target: Json<JsonValue>,
    timestamptz_saved: DateTime<Utc>,
}

impl TryFrom<&ServiceResponse> for ApplicationResponses {
    type Error = CustomProjectErrors;
    fn try_from(value: &ServiceResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            application_id: Uuid::from_str(&value.application_id)
                .map_err(|e| {
                    CustomProjectErrors::DatabaseTypeValidationError(e.to_string())
                })?
                .to_string(),
            serhub_request_id: Uuid::from_str(&value.serhub_request_id)
                .map_err(|e| {
                    CustomProjectErrors::DatabaseTypeValidationError(e.to_string())
                })?
                .to_string(),
            status_description: serde_json::json!(value.status_description).into(),
            target: serde_json::json!(value.target).into(),
            is_cache: value.is_cache,
            service_id: value.service_id,
            system_id: value.system_id,
            status: value.status.to_owned(),
            response: value.response.to_owned(),
            ..Default::default()
        })
    }
}
