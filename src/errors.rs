use thiserror::Error;

#[derive(Debug, Default, Error)]
pub enum CustomProjectErrors {
    #[error("Error while connecting to RMQ: {0}")]
    RMQConnectionError(String),
    #[error("Channel on create error: {0}")]
    RMQChannelCreationError(String),
    #[error("Channel method error: {0}")]
    RMQChannelError(String),
    #[error("Message publish error: {0}")]
    RMQPublishError(String),
    #[error("{0}")]
    RMQAckError(String),
    #[error("Model {0} validation error: {1}")]
    ValidationError(String, String),
    #[error("{0}")]
    DatabaseConnectionError(String),
    #[error("{0}")]
    DatabaseOperationError(String),
    #[error("{0}")]
    DatabaseTypeValidationError(String),
    #[error("Rmq message deserializing error: {0}")]
    IncomingSerializingMessageError(String),
    #[error("Struct cannot be serialized: {0}")]
    SerializingStructError(String),
    #[error("Health Check error")]
    DatabaseHealthCheckError,
    #[error("Unknown error")]
    #[default]
    Unknown,
}
