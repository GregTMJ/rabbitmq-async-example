use crate::{
    configs::PROJECT_CONFIG,
    errors::CustomProjectErrors,
    mapping::schemas::{BaseRequest, ByPassRequest, RMQDeserializer, Request, ServiceResponse},
    rmq::schemas::Exchange,
    tasks::producer::methods::{send_message, send_message_to_client},
};
use lapin::{Channel, protocol::basic::AMQPProperties, types::ShortString};
use uuid::Uuid;
use validator::Validate;

use crate::mapping::schemas::MappedError;

pub async fn get_request(
    channel: &Channel,
    payload: &[u8],
    amq_properties: &AMQPProperties,
) -> Result<BaseRequest, CustomProjectErrors> {
    let base_request: ByPassRequest =
        RMQDeserializer::from_rabbitmq_json::<ByPassRequest>(payload.to_vec())?;
    let request: Result<BaseRequest, CustomProjectErrors> =
        RMQDeserializer::from_rabbitmq_json::<BaseRequest>(payload.to_vec());
    let status_description: Vec<String>;
    let error_message: String;
    match request {
        Ok(body) => match body.application.validate() {
            Ok(_) => return Ok(body),
            Err(err) => {
                error_message = err.to_string();
                status_description = vec![error_message.clone()]
            }
        },
        Err(err) => {
            error_message = err.to_string();
            status_description = vec![error_message.clone()]
        }
    }
    let service_response = ServiceResponse {
        application_id: base_request.application.application_id,
        serhub_request_id: Uuid::new_v4().to_string(),
        service_id: base_request.application.service_id,
        system_id: base_request.application.system_id,
        is_cache: false,
        status: "RequestValidationError".to_string(),
        status_description,
        target: base_request.target,
        ..Default::default()
    };
    send_message_to_client(
        channel,
        &service_response,
        amq_properties
            .correlation_id()
            .clone()
            .unwrap_or(ShortString::from(String::new())),
        amq_properties
            .reply_to()
            .clone()
            .unwrap_or(ShortString::from(String::new())),
    )
    .await?;
    Err(CustomProjectErrors::ValidationError(
        "BaseRequest".to_string(),
        error_message,
    ))
}

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
            .clone()
            .unwrap_or(ShortString::from(String::new())),
        amq_properties
            .reply_to()
            .clone()
            .unwrap_or(ShortString::from(String::new())),
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
            .clone()
            .unwrap_or(ShortString::from(String::new())),
        amq_properties
            .reply_to()
            .clone()
            .unwrap_or(ShortString::from(String::new())),
    )
    .await?;
    Ok(())
}
