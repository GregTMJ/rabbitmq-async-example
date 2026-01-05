use crate::mapping::RMQDeserializer;
use crate::mapping::schemas::Request;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::types::{Json, JsonValue, Uuid};

use crate::errors::CustomProjectErrors;

#[derive(Debug, Serialize, Deserialize, FromRow, Default)]
pub struct ApplicationRequests {
    id: i32,
    pub application_id: String,
    pub serhub_request_id: String,
    pub service_id: i32,
    pub system_id: i32,
    pub application_data: Json<JsonValue>,
    timestamptz_saved: DateTime<Utc>,
}

impl TryFrom<&Request> for ApplicationRequests {
    type Error = CustomProjectErrors;

    fn try_from(value: &Request) -> Result<Self, Self::Error> {
        Ok(Self {
            application_id: Uuid::from_str(&value.application.application_id)
                .map_err(|e| {
                    CustomProjectErrors::DatabaseTypeValidationError(e.to_string())
                })?
                .to_string(),
            serhub_request_id: Uuid::from_str(&value.service_info.serhub_request_id)
                .map_err(|e| {
                    CustomProjectErrors::DatabaseTypeValidationError(e.to_string())
                })?
                .to_string(),
            service_id: value.application.service_id,
            system_id: value.application.system_id,
            application_data: Json::decode_from_string(
                &value.to_json::<JsonValue>().map_err(|e| {
                    CustomProjectErrors::SerializingStructError(e.to_string())
                })?,
            )
            .map_err(|e| {
                CustomProjectErrors::DatabaseTypeValidationError(e.to_string())
            })?,
            ..Default::default()
        })
    }
}
