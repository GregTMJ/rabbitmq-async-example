use crate::{
    errors::CustomProjectErrors,
    mapping::schemas::{Request, ServiceResponse},
    rmq::schemas::Exchange,
};
use lapin::Channel;
use lapin::options::BasicPublishOptions;
use lapin::protocol::basic::AMQPProperties;
use lapin::types::ShortString;
use log::info;

pub async fn send_message_to_service(
    channel: &Channel,
    request: &Request,
    reply_to: ShortString,
    correlation_id: ShortString,
) -> Result<(), CustomProjectErrors> {
    let service_info = &request.service_info;
    let expiration = {
        let timestamp_now = service_info.timestamp_received;
        let service_timeout = service_info.service_timeout as i64;
        (timestamp_now + service_timeout as f32 - chrono::Local::now().timestamp() as f32)
            * 1000_f32
    };
    let amq_properties = AMQPProperties::default()
        .with_content_type("application/json".into())
        .with_correlation_id(correlation_id)
        .with_reply_to(reply_to)
        .with_expiration(expiration.to_string().into());
    match channel
        .basic_publish(
            &service_info.exchange,
            &service_info.routing_key,
            BasicPublishOptions::default(),
            serde_json::to_string(request).unwrap().as_bytes(),
            amq_properties,
        )
        .await
    {
        Ok(_) => {
            info!(
                "Message sent to service with id: {}",
                request.application.service_id
            );
            Ok(())
        }
        Err(msg) => Err(CustomProjectErrors::RMQPublishError(msg.to_string())),
    }
}

pub async fn send_message_to_client(
    channel: &Channel,
    service_response: &ServiceResponse,
    reply_to: ShortString,
    correlation_id: ShortString,
) -> Result<(), CustomProjectErrors> {
    info!("Producing response to client");
    let expiration = 60 * 1000;
    let target_info = &service_response.target;
    let amq_properties = AMQPProperties::default()
        .with_content_type("application/json".into())
        .with_correlation_id(correlation_id)
        .with_reply_to(reply_to)
        .with_expiration(expiration.to_string().into())
        .with_app_id(ShortString::from(
            service_response.application_id.to_owned(),
        ));
    match channel
        .basic_publish(
            &target_info.exchange,
            &target_info.routing_key,
            BasicPublishOptions::default(),
            serde_json::to_string(service_response).unwrap().as_bytes(),
            amq_properties,
        )
        .await
    {
        Ok(_) => {
            info!("Message sent");
            Ok(())
        }
        Err(msg) => Err(CustomProjectErrors::RMQPublishError(msg.to_string())),
    }
}

pub async fn send_message<'a>(
    channel: &Channel,
    body: &'a [u8],
    exchange: &'a Exchange<'_>,
    expiration: Option<f32>,
    correlation_id: &'a ShortString,
    reply_to: &'a ShortString,
) -> Result<(), CustomProjectErrors> {
    let expiration = expiration.unwrap_or(0.0) * 1000.0;
    let amq_properties = AMQPProperties::default()
        .with_content_encoding("utf-8".into())
        .with_content_type("application/json".into())
        .with_correlation_id(correlation_id.clone())
        .with_reply_to(reply_to.clone())
        .with_expiration(expiration.to_string().into());
    match channel
        .basic_publish(
            exchange.name,
            exchange.routing_key,
            BasicPublishOptions::default(),
            body,
            amq_properties,
        )
        .await
    {
        Ok(_) => {
            info!("message sent");
            Ok(())
        }
        Err(msg) => Err(CustomProjectErrors::RMQPublishError(msg.to_string())),
    }
}
