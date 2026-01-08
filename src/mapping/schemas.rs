use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::types::Json;
use uuid::Uuid;
use validator::Validate;

use crate::database::models::services::Services;
use crate::errors::CustomProjectErrors;
use crate::mapping::validators::{
    validate_incoming_service_id, validate_incoming_system_id, validate_not_empty,
    validate_uuid_value,
};
use rmq_macros::RMQDeserializer;

use serde::de::DeserializeOwned;

// Used to specify which structs can be deserialized from RabbitMQ messages.
pub trait RMQDeserializer: DeserializeOwned + Serialize {
    fn from_rabbitmq_json(value: &[u8]) -> Result<Self, CustomProjectErrors> {
        serde_json::from_slice(value).map_err(|e| {
            CustomProjectErrors::IncomingSerializingMessageError(e.to_string())
        })
    }

    fn to_json(&self) -> Result<String, CustomProjectErrors> {
        serde_json::to_string(&self)
            .map_err(|e| CustomProjectErrors::SerializingStructError(e.to_string()))
    }
}

#[derive(Serialize, Deserialize, Validate, Debug, Default, RMQDeserializer)]
pub struct Application {
    #[validate(custom(function = "validate_uuid_value"))]
    pub application_id: String,
    #[validate(custom(function = "validate_incoming_service_id"))]
    pub service_id: i32,
    #[validate(custom(function = "validate_incoming_system_id"))]
    pub system_id: i32,
    pub multi_request: bool,
}

#[derive(Debug, Serialize, Deserialize, RMQDeserializer)]
pub struct BaseService {
    pub id: u32,
    pub name: String,
    pub exchange: String,
    pub queue: String,
    pub routing_key: String,
    pub cache_fields: String,
    pub cache_expiration: String,
    pub timeout: String,
}

#[derive(Serialize, Deserialize, Debug, Default, RMQDeserializer)]
#[serde(default)]
pub struct IncomingServiceInfo {
    pub timestamp_received: f32,
    pub service_timeout: Option<u16>,
    pub serhub_request_id: String,
    pub cached_fields: String,
    pub cache_expiration: Option<String>,
    pub exchange: Option<String>,
    pub routing_key: Option<String>,
}

impl TryFrom<&Services> for IncomingServiceInfo {
    type Error = CustomProjectErrors;
    fn try_from(value: &Services) -> Result<Self, CustomProjectErrors> {
        Ok(Self {
            timestamp_received: Local::now().timestamp() as f32,
            service_timeout: Some(value.timeout as u16),
            serhub_request_id: Uuid::new_v4().to_string(),
            cached_fields: value.cache_fields.clone(),
            cache_expiration: value.cache_expiration.clone(),
            exchange: Some(value.exchange.clone()),
            routing_key: Some(value.routing_key.clone()),
        })
    }
}

#[derive(Debug, Deserialize, Serialize, Validate, RMQDeserializer)]
pub struct ServiceInfo {
    pub timestamp_received: f32,
    pub service_timeout: u16,
    pub serhub_request_id: String,
    pub cache_fields: Vec<String>,
    pub cache_expiration: Option<String>,
    #[validate(custom(function = "validate_not_empty"))]
    pub exchange: String,
    #[validate(custom(function = "validate_not_empty"))]
    pub routing_key: String,
}

impl From<IncomingServiceInfo> for ServiceInfo {
    fn from(value: IncomingServiceInfo) -> Self {
        Self {
            timestamp_received: value.timestamp_received,
            service_timeout: value.service_timeout.unwrap_or_default(),
            serhub_request_id: value.serhub_request_id,
            cache_fields: value
                .cached_fields
                .split(",")
                .filter(|s| !s.is_empty())
                .map(str::trim)
                .map(String::from)
                .collect(),
            cache_expiration: value.cache_expiration,
            exchange: value.exchange.unwrap_or_default(),
            routing_key: value.routing_key.unwrap_or_default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct RmqTarget {
    pub vhost: String,
    pub exchange: String,
    pub routing_key: String,
    pub queue: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, RMQDeserializer)]
pub struct ByPassRequest {
    pub application: Application,
    pub target: RmqTarget,
}

#[derive(Debug, Serialize, Deserialize, RMQDeserializer)]
pub struct BaseRequest {
    pub application: Application,
    pub person: JsonValue,
    pub service_info: Option<IncomingServiceInfo>,
    pub target: RmqTarget,
}

#[derive(Debug, Serialize, Deserialize, RMQDeserializer)]
pub struct Request {
    pub application: Application,
    pub person: JsonValue,
    pub service_info: ServiceInfo,
    pub target: RmqTarget,
}

impl Request {
    pub fn new(
        base_request: BaseRequest,
        service_info: ServiceInfo,
    ) -> Self {
        Self {
            application: base_request.application,
            person: base_request.person,
            service_info,
            target: base_request.target,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, RMQDeserializer)]
#[serde(default)]
pub struct ServiceResponse {
    pub application_id: String,
    pub serhub_request_id: String,
    pub service_id: i32,
    pub system_id: i32,
    pub is_cache: bool,
    pub status: String,
    pub status_description: Vec<String>,
    pub response_created_time: String,
    pub response: Option<Json<JsonValue>>,
    pub target: RmqTarget,
}

impl Default for ServiceResponse {
    fn default() -> Self {
        Self {
            application_id: String::new(),
            serhub_request_id: String::new(),
            service_id: 0,
            system_id: 0,
            is_cache: false,
            status: String::new(),
            status_description: Vec::new(),
            response_created_time: Local::now().to_string(),
            response: None,
            target: RmqTarget::default(),
        }
    }
}

impl ServiceResponse {
    pub fn generate_response(
        value: &Request,
        is_cache: Option<bool>,
        status: String,
        status_description: Vec<String>,
    ) -> Self {
        Self {
            application_id: value.application.application_id.clone(),
            serhub_request_id: value.service_info.serhub_request_id.clone(),
            service_id: value.application.service_id,
            system_id: value.application.system_id,
            is_cache: is_cache.unwrap_or_default(),
            status,
            status_description,
            target: value.target.clone(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Serialize, Deserialize, RMQDeserializer)]
pub struct MappedError {
    pub application_id: String,
    pub serhub_request_id: String,
    pub service_id: i32,
    pub system_id: i32,
    pub error_type: Option<String>,
    pub error_message: Option<String>,
    pub error_traceback: Option<String>,
    pub data: Option<JsonValue>,
}

impl MappedError {
    pub fn generate_error_response(
        value: &Request,
        error_message: String,
        error_type: String,
    ) -> Self {
        Self {
            application_id: value.application.application_id.clone(),
            serhub_request_id: value.service_info.serhub_request_id.clone(),
            service_id: value.application.service_id,
            system_id: value.application.system_id,
            error_type: Some(error_type),
            error_message: Some(error_message),
            error_traceback: None,
            data: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use sqlx::types::Uuid;
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_application_construct() {
        let application = Application {
            application_id: Uuid::new_v4().to_string(),
            service_id: 1,
            system_id: 1,
            multi_request: false,
        };

        assert!(application.validate().is_ok());
        assert!(Uuid::from_str(&application.application_id).is_ok());
    }

    #[test]
    fn test_application_non_valid_construct() {
        let application = Application {
            application_id: "Foo".to_string(),
            service_id: 999,
            system_id: 999,
            multi_request: false,
        };

        let result = application.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_construct_service_info() {
        let mock_name: String = String::from("foo");
        let mock_service = Services {
            id: 1,
            name: mock_name.clone(),
            exchange: format!("{mock_name}.exchange").to_string(),
            queue: format!("{mock_name}.queue").to_string(),
            routing_key: format!("{mock_name}.routing_key").to_string(),
            cache_fields: String::new(),
            cache_expiration: Some("1d".to_string()),
            timeout: 15,
        };
        let result = IncomingServiceInfo::try_from(&mock_service).unwrap();

        assert_eq!(
            result.exchange,
            Some(format!("{mock_name}.exchange").to_string())
        );
        assert_eq!(
            result.routing_key,
            Some(format!("{mock_name}.routing_key").to_string())
        );

        let service_info = ServiceInfo::from(result);

        assert_eq!(
            service_info.exchange,
            format!("{mock_name}.exchange").to_string()
        );
        assert!(service_info.cache_fields.is_empty());

        let validation_result = service_info.validate();
        assert!(validation_result.is_ok())
    }

    #[test]
    fn test_invalid_construct_service_info() {
        let mock_name: String = String::from("foo");
        let mock_service = Services {
            id: 1,
            name: mock_name.clone(),
            exchange: format!("{mock_name}.exchange").to_string(),
            queue: format!("{mock_name}.queue").to_string(),
            routing_key: String::new(),
            cache_fields: String::new(),
            cache_expiration: Some("1d".to_string()),
            timeout: 15,
        };
        let result = IncomingServiceInfo::try_from(&mock_service).unwrap();
        let result = ServiceInfo::from(result).validate();

        assert!(result.is_err())
    }
}
