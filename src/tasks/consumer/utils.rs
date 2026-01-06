use crate::{
    configs::PROJECT_CONFIG,
    database::functions::save_response_with_request,
    errors::CustomProjectErrors,
    mapping::schemas::{
        BaseRequest, ByPassRequest, RMQDeserializer, Request, ServiceResponse,
    },
    rmq::schemas::Exchange,
    tasks::producer::methods::{send_message, send_message_to_client},
};
use lapin::{
    Channel,
    options::ExchangeDeclareOptions,
    protocol::basic::AMQPProperties,
    types::{AMQPValue, FieldTable, ShortString},
};
use log::info;
use sqlx::{Pool, Postgres};
use std::rc::Rc;
use uuid::Uuid;
use validator::Validate;

use crate::mapping::schemas::MappedError;

pub async fn get_request(
    channel: &Channel,
    payload: &[u8],
    amq_properties: &AMQPProperties,
) -> Result<BaseRequest, CustomProjectErrors> {
    let status_description: Vec<String>;
    let error_message: String;

    let base_request = ByPassRequest::from_rabbitmq_json(payload)?;
    let request = BaseRequest::from_rabbitmq_json(payload);

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
        Rc::new(amq_properties.correlation_id().clone().unwrap_or_default()),
        Rc::new(amq_properties.reply_to().clone().unwrap_or_default()),
    )
    .await?;
    Err(CustomProjectErrors::ValidationError(
        "BaseRequest".to_string(),
        error_message,
    ))
}

pub async fn check_exchange_exists(
    channel: &Channel,
    exchange: &Exchange<'_>,
) -> Result<(), CustomProjectErrors> {
    match channel
        .exchange_declare(
            exchange.name,
            exchange.exchange_type.clone(),
            ExchangeDeclareOptions {
                passive: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await
    {
        Ok(_) => Ok(()),
        Err(err) => {
            info!("Exchange error {err}");
            channel
                .wait_for_recovery(err.clone())
                .await
                .map_err(|e| CustomProjectErrors::RMQChannelError(e.to_string()))?;
            Err(CustomProjectErrors::RMQPublishError(err.to_string()))
        }
    }
}

pub async fn send_timeout_error_message(
    channel: &Channel,
    request: &Request,
    amq_properties: &AMQPProperties,
) -> Result<(), CustomProjectErrors> {
    let error_response = MappedError::generate_error_response(
        request,
        "Service".to_string(),
        "ServiceTimeout".to_string(),
    );
    let fail_exchange = Exchange::new(
        &PROJECT_CONFIG.rmq_exchange,
        &PROJECT_CONFIG.rmq_exchange_type,
    );
    let properties = AMQPProperties::default()
        .with_correlation_id(
            amq_properties.correlation_id().clone().unwrap_or_default(),
        )
        .with_reply_to(amq_properties.reply_to().clone().unwrap_or_default());
    send_message(
        channel,
        error_response.to_json()?.as_bytes(),
        &fail_exchange,
        &PROJECT_CONFIG.rmq_fail_table_queue,
        properties,
    )
    .await?;
    Ok(())
}

pub async fn send_timeout_error_service(
    channel: &Channel,
    request: &Request,
    amq_properties: &AMQPProperties,
) -> Result<(), CustomProjectErrors> {
    let service_response = ServiceResponse::generate_response(
        request,
        Some(false),
        "ServiceTimeout".to_string(),
        vec!["service_timeout".to_string()],
    );
    let response_exchange = Exchange::new(
        &PROJECT_CONFIG.rmq_exchange,
        &PROJECT_CONFIG.rmq_exchange_type,
    );
    let properties = AMQPProperties::default()
        .with_correlation_id(
            amq_properties.correlation_id().clone().unwrap_or_default(),
        )
        .with_reply_to(amq_properties.reply_to().clone().unwrap_or_default());
    send_message(
        channel,
        service_response.to_json()?.as_bytes(),
        &response_exchange,
        &PROJECT_CONFIG.rmq_service_response_queue,
        properties,
    )
    .await?;
    Ok(())
}

pub async fn send_delayed_message(
    request: &Request,
    channel: &Channel,
    reply_to: Rc<ShortString>,
    correlation_id: Rc<ShortString>,
) -> Result<(), CustomProjectErrors> {
    let timeout_exchange: Exchange = Exchange::new(
        &PROJECT_CONFIG.rmq_delayed_exchange,
        &PROJECT_CONFIG.rmq_exchange_type,
    );
    let headers = {
        let mut temp_header = FieldTable::default();
        let timeout =
            serde_json::to_value(request.service_info.service_timeout * 1000).unwrap();
        temp_header.insert(
            ShortString::from("x-delay"),
            AMQPValue::try_from(&timeout, lapin::types::AMQPType::Float).unwrap(),
        );
        temp_header
    };
    let properties = AMQPProperties::default()
        .with_correlation_id(correlation_id.as_str().into())
        .with_reply_to(reply_to.as_str().into())
        .with_headers(headers);
    send_message(
        channel,
        request.to_json()?.as_bytes(),
        &timeout_exchange,
        &PROJECT_CONFIG.rmq_timeout_queue,
        properties,
    )
    .await?;
    Ok(())
}

pub async fn send_publish_error_message(
    request: &Request,
    error_message: &str,
    channel: &Channel,
    connection: &Pool<Postgres>,
    reply_to: Rc<ShortString>,
    correlation_id: Rc<ShortString>,
) -> Result<(), CustomProjectErrors> {
    let service_error_response = ServiceResponse::generate_response(
        request,
        Some(false),
        "RMQPublishError".to_string(),
        vec![error_message.to_string()],
    );
    let response_exchange = Exchange::new(
        &PROJECT_CONFIG.rmq_exchange,
        &PROJECT_CONFIG.rmq_exchange_type,
    );
    save_response_with_request(request, connection).await?;
    let properties = AMQPProperties::default()
        .with_correlation_id(correlation_id.as_str().into())
        .with_reply_to(reply_to.as_str().into());
    send_message(
        channel,
        service_error_response.to_json()?.as_bytes(),
        &response_exchange,
        &PROJECT_CONFIG.rmq_service_response_queue,
        properties,
    )
    .await?;
    Ok(())
}
