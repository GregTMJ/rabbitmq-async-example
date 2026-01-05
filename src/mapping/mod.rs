pub mod schemas;
pub mod validators;

use crate::errors::CustomProjectErrors;
use crate::mapping::schemas::{
    BaseRequest, ByPassRequest, MappedError, Request, ServiceResponse,
};
use serde::Serialize;
use serde::de::DeserializeOwned;

// Used to specify which structs can be deserialized from RabbitMQ messages.
pub trait RMQDeserializer: DeserializeOwned {
    fn from_rabbitmq_json<T: DeserializeOwned>(
        value: &[u8]
    ) -> Result<Self, CustomProjectErrors> {
        serde_json::from_slice(value).map_err(|e| {
            CustomProjectErrors::IncomingSerializingMessageError(e.to_string())
        })
    }

    fn to_json<T: DeserializeOwned>(&self) -> Result<String, CustomProjectErrors>
    where
        Self: Serialize,
    {
        serde_json::to_string(&self)
            .map_err(|e| CustomProjectErrors::SerializingStructError(e.to_string()))
    }
}

macro_rules! impl_rmq_deserializer {
    ($($type:ty), + $(,)?) => {
        $(impl RMQDeserializer for $type {}) +
    };
}

impl_rmq_deserializer! {
    BaseRequest,
    ByPassRequest,
    MappedError,
    Request,
    ServiceResponse,
}
