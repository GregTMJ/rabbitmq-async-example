use lapin::{Channel, protocol::basic::AMQPProperties, types::ShortString};
use log::{debug, info};
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use validator::Validate;

use crate::{
    configs::PROJECT_CONFIG,
    database::{
        functions::{
            check_application_response, get_service_info, save_response_with_request,
            save_service_response, save_to_fail_table,
        },
        models::services::Services,
    },
    mapping::from_json_slice,
    mapping::schemas::{
        BaseRequest, Exchange, IncomingServiceInfo, MappedError, Request, ServiceInfo,
        ServiceResponse,
    },
    tasks::producer::methods::{send_message, send_message_to_client, send_message_to_service},
};

pub async fn on_client_message(
    incoming_message: Vec<u8>,
    amq_properties: AMQPProperties,
    connection: Arc<Pool<Postgres>>,
    channel: Arc<Channel>,
) -> Result<(), String> {
    let incoming_request: BaseRequest = from_json_slice(incoming_message)?;
    match incoming_request.application.validate() {
        Ok(val) => val,
        Err(msg) => return Err(msg.to_string()),
    };
    let database_service_info: Services =
        get_service_info(&incoming_request.application.service_id, &connection).await?;
    let base_service_info = IncomingServiceInfo::try_from(database_service_info)?;
    debug!("Got the following db info {base_service_info:?}");
    let service_info: ServiceInfo = ServiceInfo::try_from(base_service_info)?;
    match service_info.validate() {
        Ok(val) => val,
        Err(msg) => return Err(msg.to_string()),
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
) -> Result<(), String> {
    info!("Incoming data for on_service_message");
    let incoming_response: ServiceResponse = from_json_slice(incoming_message)?;
    save_service_response(&incoming_response, &connection).await?;
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
    Ok(())
}

pub async fn on_fail_message(
    incoming_message: Vec<u8>,
    _amq_properties: AMQPProperties,
    connection: Arc<Pool<Postgres>>,
    _channel: Arc<Channel>,
) -> Result<(), String> {
    info!("Incoming data for on_fail_message");
    let incoming_error: MappedError = from_json_slice(incoming_message)?;
    save_to_fail_table(&incoming_error, &connection).await?;
    Ok(())
}

pub async fn on_timeout_message(
    incoming_message: Vec<u8>,
    amq_properties: AMQPProperties,
    connection: Arc<Pool<Postgres>>,
    channel: Arc<Channel>,
) -> Result<(), String> {
    info!("Incoming data for on_timeout_message");
    let incoming_request: Request = from_json_slice(incoming_message)?;

    let existing_application_response = check_application_response(
        &incoming_request.service_info.serhub_request_id,
        &connection,
    )
    .await?;
    let insert_incoming_request =
        save_response_with_request(&incoming_request, &connection).await?;

    if !existing_application_response && insert_incoming_request {
        let application_id = incoming_request.application.application_id;
        let serhub_request_id = incoming_request.service_info.serhub_request_id;

        let error_response = MappedError {
            application_id: application_id.clone(),
            serhub_request_id: serhub_request_id.clone(),
            service_id: incoming_request.application.service_id,
            system_id: incoming_request.application.service_id,
            error_type: Some("Service".to_owned()),
            error_message: Some("ServiceTimeout".to_owned()),
            error_traceback: None,
            data: None,
        };
        let service_response = ServiceResponse {
            serhub_request_id: application_id.to_owned(),
            application_id: serhub_request_id.to_owned(),
            system_id: incoming_request.application.system_id,
            service_id: incoming_request.application.service_id,
            is_cache: false,
            status: "ServiceTimeout".to_owned(),
            status_description: vec!["service_timeout".to_owned()],
            response: None,
            target: incoming_request.target,
            ..Default::default()
        };

        let response_exchange = Exchange::new(
            &PROJECT_CONFIG.RMQ_EXCHANGE,
            &PROJECT_CONFIG.RMQ_SERVICE_RESPONSE_QUEUE,
            "direct",
        );
        let fail_exchange = Exchange::new(
            &PROJECT_CONFIG.RMQ_EXCHANGE,
            &PROJECT_CONFIG.RMQ_FAIL_TABLE_QUEUE,
            "direct",
        );

        send_message(
            &channel,
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
        send_message(
            &channel,
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
    };
    Ok(())
}
