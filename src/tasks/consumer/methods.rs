use lapin::{
    Channel,
    protocol::basic::AMQPProperties,
    types::{AMQPValue, FieldTable, ShortString},
};
use log::{debug, info};
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use validator::Validate;

use crate::{
    configs::PROJECT_CONFIG,
    database::{
        functions::{
            check_application_response, get_service_info, save_client_request,
            save_response_with_request, save_service_response, save_to_fail_table,
        },
        models::services::Services,
    },
    errors::CustomProjectErrors,
    mapping::schemas::{
        IncomingServiceInfo, MappedError, RMQDeserializer, Request, ServiceInfo, ServiceResponse,
    },
    rmq::schemas::Exchange,
    tasks::{
        consumer::utils::{get_request, send_timeout_error_message, send_timeout_error_service},
        producer::methods::{send_message, send_message_to_client, send_message_to_service},
    },
};

pub async fn on_client_message(
    payload: Vec<u8>,
    amq_properties: AMQPProperties,
    connection: Arc<Pool<Postgres>>,
    channel: Arc<Channel>,
) -> Result<(), CustomProjectErrors> {
    let request = get_request(&channel, &payload, &amq_properties).await?;

    let database_service_info: Services =
        get_service_info(&request.application.service_id, &connection).await?;
    let base_service_info = IncomingServiceInfo::from(database_service_info);
    debug!("Got the following db info {base_service_info:?}");
    let service_info: ServiceInfo = ServiceInfo::from(base_service_info);
    match service_info.validate() {
        Ok(val) => val,
        Err(msg) => {
            return Err(CustomProjectErrors::ValidationError(
                "ServiceInfo".to_owned(),
                msg.to_string(),
            ));
        }
    }
    let request = Request::new(request, service_info);
    save_client_request(&request, &connection).await;
    debug!("request to service body before sent: {request:?}");
    send_message_to_service(
        &channel,
        &request,
        &connection,
        amq_properties.reply_to().clone().unwrap_or_default(),
        amq_properties.correlation_id().clone().unwrap_or_default(),
    )
    .await?;
    let timeout_exchange: Exchange = Exchange::new(
        &PROJECT_CONFIG.RMQ_DELAYED_EXCHANGE,
        &PROJECT_CONFIG.RMQ_EXCHANGE_TYPE,
    );
    let headers = {
        let mut temp_header = FieldTable::default();
        let timeout = serde_json::to_value(request.service_info.service_timeout * 1000).unwrap();
        temp_header.insert(
            ShortString::from("x-delay"),
            AMQPValue::try_from(&timeout, lapin::types::AMQPType::Float).unwrap(),
        );
        temp_header
    };
    send_message(
        &channel,
        serde_json::to_string(&request).unwrap().as_bytes(),
        &timeout_exchange,
        &PROJECT_CONFIG.RMQ_TIMEOUT_QUEUE,
        None,
        amq_properties.correlation_id().clone().unwrap_or_default(),
        amq_properties.reply_to().clone().unwrap_or_default(),
        headers,
    )
    .await?;
    Ok(())
}

pub async fn on_service_message(
    payload: Vec<u8>,
    amq_properties: AMQPProperties,
    connection: Arc<Pool<Postgres>>,
    channel: Arc<Channel>,
) -> Result<(), CustomProjectErrors> {
    info!("Incoming data for on_service_message");
    let service_response: ServiceResponse =
        RMQDeserializer::from_rabbitmq_json::<ServiceResponse>(payload)?;
    let save_result = save_service_response(&service_response, &connection).await;
    if save_result {
        send_message_to_client(
            &channel,
            &service_response,
            amq_properties.reply_to().clone().unwrap_or_default(),
            amq_properties.correlation_id().clone().unwrap_or_default(),
        )
        .await?;
    }

    Ok(())
}

pub async fn on_fail_message(
    payload: Vec<u8>,
    _amq_properties: AMQPProperties,
    connection: Arc<Pool<Postgres>>,
    _channel: Arc<Channel>,
) -> Result<(), CustomProjectErrors> {
    info!("Incoming data for on_fail_message");
    let mapped_error: MappedError = RMQDeserializer::from_rabbitmq_json::<MappedError>(payload)?;
    save_to_fail_table(&mapped_error, &connection).await;
    Ok(())
}

pub async fn on_timeout_message(
    payload: Vec<u8>,
    amq_properties: AMQPProperties,
    connection: Arc<Pool<Postgres>>,
    channel: Arc<Channel>,
) -> Result<(), CustomProjectErrors> {
    info!("Incoming data for on_timeout_message");
    let request: Request = RMQDeserializer::from_rabbitmq_json::<Request>(payload)?;

    let existing_application_response =
        check_application_response(&request.service_info.serhub_request_id, &connection).await;
    let insert_incoming_request = save_response_with_request(&request, &connection).await;

    if !existing_application_response && insert_incoming_request {
        send_timeout_error_message(&channel, &request, &amq_properties).await?;
        send_timeout_error_service(&channel, &request, &amq_properties).await?;
    };
    Ok(())
}
