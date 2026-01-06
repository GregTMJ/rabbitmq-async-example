use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::types::{Json, JsonValue, Uuid};
use std::str::FromStr;

use crate::prelude::{CustomProjectErrors, MappedError};

#[derive(Debug, Serialize, Deserialize, FromRow, Default)]
pub struct FailTable {
    id: i32,
    pub application_id: String,
    pub serhub_request_id: String,
    pub service_id: i32,
    pub system_id: i32,
    pub error_type: Option<String>,
    pub error_message: Option<String>,
    pub error_traceback: Option<String>,
    pub data: Option<Json<JsonValue>>,
    timestamptz_saved: DateTime<Utc>,
}

impl TryFrom<&MappedError> for FailTable {
    type Error = CustomProjectErrors;
    fn try_from(value: &MappedError) -> Result<Self, Self::Error> {
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
            service_id: value.service_id,
            system_id: value.system_id,
            error_type: value.error_type.to_owned(),
            error_message: value.error_message.to_owned(),
            error_traceback: value.error_traceback.to_owned(),
            data: Some(
                serde_json::json!(value.data.as_ref().unwrap_or_default()).into(),
            ),
            ..Default::default()
        })
    }
}
