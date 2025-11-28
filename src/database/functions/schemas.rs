use crate::{
    errors::CustomProjectErrors,
    mapping::schemas::{MappedError, RMQDeserializer, ServiceResponse},
};
use serde_json::Value as AnyJsonValue;
use sqlx::types::Json;
use std::str::FromStr;
use uuid::Uuid as Uuidv4;

use crate::mapping::schemas::Request;

pub struct SqlRequest {
    pub application_id: Uuidv4,
    pub serhub_request_id: Uuidv4,
    pub service_id: i32,
    pub system_id: i32,
    pub request_data: String,
}

impl TryFrom<&Request> for SqlRequest {
    type Error = CustomProjectErrors;
    fn try_from(value: &Request) -> Result<Self, Self::Error> {
        Ok(Self {
            application_id: Uuidv4::from_str(&value.application.application_id)
                .map_err(|e| {
                    CustomProjectErrors::DatabaseTypeValidationError(e.to_string())
                })?,
            serhub_request_id: Uuidv4::from_str(&value.service_info.serhub_request_id)
                .map_err(|e| {
                    CustomProjectErrors::DatabaseTypeValidationError(e.to_string())
                })?,
            service_id: value.application.service_id,
            system_id: value.application.system_id,
            request_data: value.to_json::<String>().map_err(|e| {
                CustomProjectErrors::SerializingStructError(e.to_string())
            })?,
        })
    }
}

pub struct ServiceSqlResponse {
    pub application_id: Uuidv4,
    pub serhub_request_id: Uuidv4,
    pub service_id: i32,
    pub system_id: i32,
    pub is_cache: bool,
    pub status: String,
    pub status_description: AnyJsonValue,
    pub response: Option<Json<AnyJsonValue>>,
    pub target: AnyJsonValue,
}

impl TryFrom<&ServiceResponse> for ServiceSqlResponse {
    type Error = CustomProjectErrors;
    fn try_from(value: &ServiceResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            application_id: Uuidv4::from_str(&value.application_id).map_err(|e| {
                CustomProjectErrors::DatabaseTypeValidationError(e.to_string())
            })?,
            serhub_request_id: Uuidv4::from_str(&value.serhub_request_id).map_err(
                |e| CustomProjectErrors::DatabaseTypeValidationError(e.to_string()),
            )?,
            status_description: serde_json::json!(value.status_description),
            target: serde_json::json!(value.target),
            is_cache: value.is_cache,
            service_id: value.service_id,
            system_id: value.system_id,
            status: value.status.to_owned(),
            response: value.response.to_owned(),
        })
    }
}

pub struct SqlMappedError {
    pub application_id: Uuidv4,
    pub serhub_request_id: Uuidv4,
    pub service_id: i32,
    pub system_id: i32,
    pub error_type: Option<String>,
    pub error_message: Option<String>,
    pub error_traceback: Option<String>,
    pub data: AnyJsonValue,
}

impl TryFrom<&MappedError> for SqlMappedError {
    type Error = CustomProjectErrors;
    fn try_from(value: &MappedError) -> Result<Self, Self::Error> {
        Ok(Self {
            application_id: Uuidv4::from_str(&value.application_id).map_err(|e| {
                CustomProjectErrors::DatabaseTypeValidationError(e.to_string())
            })?,
            serhub_request_id: Uuidv4::from_str(&value.serhub_request_id).map_err(
                |e| CustomProjectErrors::DatabaseTypeValidationError(e.to_string()),
            )?,
            service_id: value.service_id,
            system_id: value.system_id,
            error_type: value.error_type.to_owned(),
            error_message: value.error_message.to_owned(),
            error_traceback: value.error_traceback.to_owned(),
            data: serde_json::json!(value.data.as_ref().unwrap_or_default()),
        })
    }
}
