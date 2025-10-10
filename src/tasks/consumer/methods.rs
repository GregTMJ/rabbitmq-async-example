use lapin::{Channel, protocol::basic::AMQPProperties, types::ShortString};
use log::{debug, info};
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use validator::Validate;

use crate::{
    database::{
        functions::{
            check_application_response, get_service_info, save_response_with_request,
            save_service_response, save_to_fail_table,
        },
        models::services::Services,
    },
    errors::CustomProjectErrors,
    mapping::schemas::{
        BaseRequest, IncomingServiceInfo, MappedError, RMQDeserializer, Request, ServiceInfo,
        ServiceResponse,
    },
    tasks::{
        consumer::utils::{send_timeout_error_message, send_timeout_error_service},
        producer::methods::{send_message_to_client, send_message_to_service},
    },
};

pub async fn on_client_message(
    incoming_message: Vec<u8>,
    amq_properties: AMQPProperties,
    connection: Arc<Pool<Postgres>>,
    channel: Arc<Channel>,
) -> Result<(), CustomProjectErrors> {
    let incoming_request: BaseRequest =
        RMQDeserializer::from_rabbitmq_json::<BaseRequest>(incoming_message)?;
    match incoming_request.application.validate() {
        Ok(val) => val,
        Err(msg) => {
            return Err(CustomProjectErrors::ValidationError(
                "BaseRequest".to_string(),
                msg.to_string(),
            ));
        }
    };
    let database_service_info: Services =
        get_service_info(&incoming_request.application.service_id, &connection).await?;
    let base_service_info = IncomingServiceInfo::try_from(database_service_info)?;
    debug!("Got the following db info {base_service_info:?}");
    let service_info: ServiceInfo = ServiceInfo::try_from(base_service_info)?;
    match service_info.validate() {
        Ok(val) => val,
        Err(msg) => {
            return Err(CustomProjectErrors::ValidationError(
                "ServiceInfo".to_owned(),
                msg.to_string(),
            ));
        }
    }
    let request = Request::new(incoming_request, service_info);
    debug!("request to service body before sent: {request:?}");
    send_message_to_service(
        &channel,
        &request,
        amq_properties
            .reply_to()
            .as_ref()
            .unwrap_or(&ShortString::from(String::new())),
        amq_properties
            .correlation_id()
            .as_ref()
            .unwrap_or(&ShortString::from(String::new())),
    )
    .await?;
    Ok(())
}

pub async fn on_service_message(
    incoming_message: Vec<u8>,
    amq_properties: AMQPProperties,
    connection: Arc<Pool<Postgres>>,
    channel: Arc<Channel>,
) -> Result<(), CustomProjectErrors> {
    info!("Incoming data for on_service_message");
    let incoming_response: ServiceResponse =
        RMQDeserializer::from_rabbitmq_json::<ServiceResponse>(incoming_message)?;
    let save_result = save_service_response(&incoming_response, &connection).await;
    if save_result {
        send_message_to_client(
            &channel,
            &incoming_response,
            amq_properties
                .reply_to()
                .as_ref()
                .unwrap_or(&ShortString::from(String::new())),
            amq_properties
                .correlation_id()
                .as_ref()
                .unwrap_or(&ShortString::from(String::new())),
        )
        .await?;
    }

    Ok(())
}

pub async fn on_fail_message(
    incoming_message: Vec<u8>,
    _amq_properties: AMQPProperties,
    connection: Arc<Pool<Postgres>>,
    _channel: Arc<Channel>,
) -> Result<(), CustomProjectErrors> {
    info!("Incoming data for on_fail_message");
    let incoming_error: MappedError =
        RMQDeserializer::from_rabbitmq_json::<MappedError>(incoming_message)?;
    save_to_fail_table(&incoming_error, &connection).await;
    Ok(())
}

pub async fn on_timeout_message(
    incoming_message: Vec<u8>,
    amq_properties: AMQPProperties,
    connection: Arc<Pool<Postgres>>,
    channel: Arc<Channel>,
) -> Result<(), CustomProjectErrors> {
    info!("Incoming data for on_timeout_message");
    let incoming_request: Request =
        RMQDeserializer::from_rabbitmq_json::<Request>(incoming_message)?;

    let existing_application_response = check_application_response(
        &incoming_request.service_info.serhub_request_id,
        &connection,
    )
    .await;
    let insert_incoming_request = save_response_with_request(&incoming_request, &connection).await;

    if !existing_application_response && insert_incoming_request {
        send_timeout_error_message(&channel, &incoming_request, &amq_properties).await?;
        send_timeout_error_service(&channel, &incoming_request, &amq_properties).await?;
    };
    Ok(())
}
