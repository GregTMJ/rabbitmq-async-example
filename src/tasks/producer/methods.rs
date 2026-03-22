use crate::configs::PROJECT_CONFIG;
use crate::mapping::schemas::RMQDeserializer;
use crate::prelude::*;
use crate::tasks::consumer::utils::check_exchange_exists;
use lapin::Channel;
use lapin::options::BasicPublishOptions;
use lapin::protocol::basic::AMQPProperties;
use lapin::types::ShortString;
use log::{info, warn};

pub async fn send_message_to_service(
    channel: &Channel,
    request: &Request,
    reply_to: ShortString,
    correlation_id: ShortString,
) -> Result<(), CustomProjectErrors> {
    let service_info = &request.service_info;
    let expiration = {
        let timestamp_now = service_info.timestamp_received;
        let service_timeout = service_info.service_timeout as f64;
        let current_timestamp = chrono::Local::now().timestamp() as f64;
        ((timestamp_now + service_timeout - current_timestamp) * 1000.0).max(0.0)
    };
    if expiration == 0.0 {
        warn!(
            "Message won't be handled for ttl reasons for {}",
            service_info.serhub_request_id
        );
        return Ok(());
    }
    let amq_properties = AMQPProperties::default()
        .with_content_type("application/json".into())
        .with_correlation_id(correlation_id)
        .with_reply_to(reply_to)
        .with_expiration(expiration.to_string().into());

    let exchange =
        Exchange::new(&service_info.exchange, &PROJECT_CONFIG.rmq_exchange_type);
    check_exchange_exists(channel, &exchange).await?;

    match channel
        .basic_publish(
            &service_info.exchange,
            &service_info.routing_key,
            BasicPublishOptions::default(),
            request.to_json()?.as_bytes(),
            amq_properties,
        )
        .await
    {
        Ok(confirm) => match confirm.await {
            Ok(_) => {
                info!(
                    "Message sent to service with id: {}",
                    request.application.service_id
                );
                Ok(())
            }
            Err(msg) => Err(CustomProjectErrors::RMQPublishError(msg.to_string())),
        },
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
            service_response.to_json()?.as_bytes(),
            amq_properties,
        )
        .await
    {
        Ok(confirm) => match confirm.await {
            Ok(_) => info!("Message to client was sent!"),
            Err(msg) => {
                return Err(CustomProjectErrors::RMQPublishError(msg.to_string()));
            }
        },
        Err(msg) => return Err(CustomProjectErrors::RMQPublishError(msg.to_string())),
    }
    Ok(())
}

pub async fn send_message<'a>(
    channel: &Channel,
    payload: &'a [u8],
    exchange: &'a Exchange<'_>,
    routing_key: &'a str,
    properties: AMQPProperties,
) -> Result<(), CustomProjectErrors> {
    match channel
        .basic_publish(
            exchange.name,
            routing_key,
            BasicPublishOptions::default(),
            payload,
            properties,
        )
        .await
    {
        Ok(confirm) => match confirm.await {
            Ok(_) => {
                info!("message sent!");
                Ok(())
            }
            Err(msg) => Err(CustomProjectErrors::RMQPublishError(msg.to_string())),
        },
        Err(msg) => Err(CustomProjectErrors::RMQPublishError(msg.to_string())),
    }
}
