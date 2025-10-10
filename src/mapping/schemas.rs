use chrono::prelude::*;
use derivative::Derivative;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value as AnyJsonValue;
use sqlx::types::{Json, Uuid};
use validator::Validate;

use crate::database::models::services::Services;
use crate::errors::CustomProjectErrors;
use crate::mapping::validators::{
    validate_incoming_service_id, validate_incoming_system_id, validate_not_empty,
};
use lapin::ExchangeKind;

#[derive(Debug)]
pub struct Exchange<'a> {
    pub name: &'a str,
    pub routing_key: &'a str,
    pub exchange_type: ExchangeKind,
}

impl<'a> Exchange<'a> {
    pub fn new(name: &'a str, routing_key: &'a str, exchange_type: &str) -> Self {
        let exchange_type = match exchange_type {
            "direct" => ExchangeKind::Direct,
            "fanout" => ExchangeKind::Fanout,
            _ => ExchangeKind::Topic,
        };
        Self {
            name,
            routing_key,
            exchange_type,
        }
    }
}

#[derive(Debug, Default)]
pub struct Queue<'a> {
    pub name: &'a str,
}

#[derive(Derivative, Serialize, Deserialize, Validate)]
#[derivative(Debug, Default)]
pub struct Application {
    pub application_id: String,
    #[validate(custom(function = "validate_incoming_service_id"))]
    pub service_id: i32,
    #[validate(custom(function = "validate_incoming_system_id"))]
    pub system_id: i32,
    #[derivative(Default(value = "false"))]
    pub multi_request: bool,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize, Debug, Derivative)]
#[derivative(Default)]
#[serde(default)]
pub struct IncomingServiceInfo {
    #[derivative(Default(value = "Local::now().timestamp() as f32"))]
    pub timestamp_received: f32,
    pub service_timeout: Option<u16>,
    #[derivative(Default(value = "Uuid::new_v4().to_string()"))]
    pub serhub_request_id: String,
    #[derivative(Default(value = "String::new()"))]
    pub cached_fields: String,
    pub cache_expiration: Option<String>,
    pub exchange: Option<String>,
    pub routing_key: Option<String>,
}

impl TryFrom<Services> for IncomingServiceInfo {
    type Error = CustomProjectErrors;
    fn try_from(value: Services) -> Result<Self, Self::Error> {
        Ok(Self {
            service_timeout: Some(value.timeout as u16),
            cache_expiration: value.cache_expiration,
            cached_fields: value.cache_fields,
            routing_key: Some(value.routing_key),
            exchange: Some(value.exchange),
            ..Default::default()
        })
    }
}

#[derive(Debug, Deserialize, Serialize, Validate, Derivative)]
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

impl TryFrom<IncomingServiceInfo> for ServiceInfo {
    type Error = CustomProjectErrors;
    fn try_from(value: IncomingServiceInfo) -> Result<Self, Self::Error> {
        Ok(Self {
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
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct RmqTarget {
    pub vhost: String,
    pub exchange: String,
    pub routing_key: String,
    pub queue: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BaseRequest {
    pub application: Application,
    pub person: AnyJsonValue,
    pub service_info: Option<IncomingServiceInfo>,
    pub target: RmqTarget,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub application: Application,
    pub person: AnyJsonValue,
    pub service_info: ServiceInfo,
    pub target: RmqTarget,
}

impl Request {
    pub fn new(base_request: BaseRequest, service_info: ServiceInfo) -> Self {
        Self {
            application: base_request.application,
            person: base_request.person,
            service_info,
            target: base_request.target,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Derivative)]
#[derivative(Default)]
#[serde(default)]
pub struct ServiceResponse {
    pub application_id: String,
    pub serhub_request_id: String,
    pub service_id: i32,
    pub system_id: i32,
    pub is_cache: bool,
    pub status: String,
    #[derivative(Default(value = "Vec::new()"))]
    pub status_description: Vec<String>,
    #[derivative(Default(value = "Local::now().to_string()"))]
    pub response_created_time: String,
    pub response: Option<Json<AnyJsonValue>>,
    pub target: RmqTarget,
}

impl TryFrom<&Request> for ServiceResponse {
    type Error = CustomProjectErrors;
    fn try_from(value: &Request) -> Result<Self, Self::Error> {
        Ok(Self {
            application_id: value.application.application_id.clone(),
            serhub_request_id: value.service_info.serhub_request_id.clone(),
            service_id: value.application.service_id,
            system_id: value.application.system_id,
            is_cache: false,
            status: "ServiceTimeout".to_owned(),
            status_description: vec!["service_timeout".to_owned()],
            target: value.target.clone(),
            ..Default::default()
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MappedError {
    pub application_id: String,
    pub serhub_request_id: String,
    pub service_id: i32,
    pub system_id: i32,
    pub error_type: Option<String>,
    pub error_message: Option<String>,
    pub error_traceback: Option<String>,
    pub data: Option<AnyJsonValue>,
}

impl TryFrom<&Request> for MappedError {
    type Error = CustomProjectErrors;
    fn try_from(value: &Request) -> Result<Self, Self::Error> {
        Ok(Self {
            application_id: value.application.application_id.clone(),
            serhub_request_id: value.service_info.serhub_request_id.clone(),
            service_id: value.application.service_id,
            system_id: value.application.service_id,
            error_type: Some("Service".to_owned()),
            error_message: Some("ServiceTimeout".to_owned()),
            error_traceback: None,
            data: None,
        })
    }
}

// Used to specify which structs can be deserialized from RabbitMQ messages.
pub trait RMQDeserializer: DeserializeOwned {
    fn from_rabbitmq_json<T>(value: Vec<u8>) -> Result<Self, CustomProjectErrors>
    where
        T: DeserializeOwned,
    {
        serde_json::from_slice(&value)
            .map_err(|e| CustomProjectErrors::IncomingSerializingMessageError(e.to_string()))
    }
}
impl RMQDeserializer for MappedError {}
impl RMQDeserializer for BaseRequest {}
impl RMQDeserializer for Request {}
impl RMQDeserializer for ServiceResponse {}
