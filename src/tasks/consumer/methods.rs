use lapin::{Channel, protocol::basic::AMQPProperties, types::ShortString};
use log::{debug, info};
use serde_json::from_slice;
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use validator::Validate;

use crate::{
    database::{
        functions::{get_service_info, save_service_response},
        models::services::Services,
    },
    mapping::schemas::{BaseRequest, IncomingServiceInfo, Request, ServiceInfo, ServiceResponse},
    tasks::producer::methods::{send_message, send_message_to_client},
};

pub async fn on_client_message(
    incoming_message: Vec<u8>,
    amq_properties: AMQPProperties,
    connection: Arc<Pool<Postgres>>,
    channel: Arc<Channel>,
) -> Result<(), String> {
    let message = String::from_utf8(incoming_message.clone()).unwrap();
    debug!("Got the following message {message}");
    let incoming_request: BaseRequest =
        from_slice(&incoming_message).map_err(|e| format!("Couldn't struct with error {e}"))?;
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
    send_message(
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
    let incoming_response = ServiceResponse::try_from(incoming_message)?;
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
