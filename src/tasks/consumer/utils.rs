use crate::{
    configs::PROJECT_CONFIG,
    errors::CustomProjectErrors,
    mapping::schemas::{Exchange, Request, ServiceResponse},
    tasks::producer::methods::send_message,
};
use lapin::{Channel, protocol::basic::AMQPProperties, types::ShortString};

use crate::mapping::schemas::MappedError;

pub async fn send_timeout_error_message(
    channel: &Channel,
    request: &Request,
    amq_properties: &AMQPProperties,
) -> Result<(), CustomProjectErrors> {
    let error_response = MappedError::try_from(request)?;
    let fail_exchange = Exchange::new(
        &PROJECT_CONFIG.RMQ_EXCHANGE,
        &PROJECT_CONFIG.RMQ_FAIL_TABLE_QUEUE,
        &PROJECT_CONFIG.RMQ_EXCHANGE_TYPE,
    );
    send_message(
        channel,
        serde_json::to_string(&error_response).unwrap().as_bytes(),
        &fail_exchange,
        None,
        amq_properties
            .correlation_id()
            .as_ref()
            .unwrap_or(&ShortString::from(String::new())),
        amq_properties
            .reply_to()
            .as_ref()
            .unwrap_or(&ShortString::from(String::new())),
    )
    .await?;
    Ok(())
}

pub async fn send_timeout_error_service(
    channel: &Channel,
    request: &Request,
    amq_properties: &AMQPProperties,
) -> Result<(), CustomProjectErrors> {
    let service_response = ServiceResponse::try_from(request)?;
    let response_exchange = Exchange::new(
        &PROJECT_CONFIG.RMQ_EXCHANGE,
        &PROJECT_CONFIG.RMQ_SERVICE_RESPONSE_QUEUE,
        &PROJECT_CONFIG.RMQ_EXCHANGE_TYPE,
    );
    send_message(
        channel,
        serde_json::to_string(&service_response).unwrap().as_bytes(),
        &response_exchange,
        None,
        amq_properties
            .correlation_id()
            .as_ref()
            .unwrap_or(&ShortString::from(String::new())),
        amq_properties
            .reply_to()
            .as_ref()
            .unwrap_or(&ShortString::from(String::new())),
    )
    .await?;
    Ok(())
}
