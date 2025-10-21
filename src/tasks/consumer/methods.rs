use lapin::{Channel, message::Delivery};
use log::{debug, info};
use sqlx::{Pool, Postgres};
use std::{rc::Rc, sync::Arc};
use validator::Validate;

use crate::{
    database::{
        functions::{
            check_application_response, get_service_info, save_client_request,
            save_response_with_request, save_service_response, save_to_fail_table,
        },
        models::services::Services,
    },
    errors::CustomProjectErrors,
    mapping::schemas::{
        IncomingServiceInfo, MappedError, RMQDeserializer, Request, ServiceInfo,
        ServiceResponse,
    },
    tasks::{
        consumer::utils::{
            get_request, send_delayed_message, send_publish_error_message,
            send_timeout_error_message, send_timeout_error_service,
        },
        producer::methods::{send_message_to_client, send_message_to_service},
    },
};

pub async fn on_client_message(
    msg: Delivery,
    connection: Arc<Pool<Postgres>>,
    channel: Arc<Channel>,
) -> Result<(), CustomProjectErrors> {
    info!("Got an incoming request!");
    let request = get_request(&channel, &msg.data, &msg.properties).await?;

    let database_service_info: Services =
        get_service_info(&request.application.service_id, &connection).await?;
    let base_service_info = IncomingServiceInfo::from(database_service_info);
    debug!("Got the following db info {base_service_info:?}");
    let service_info: ServiceInfo = ServiceInfo::from(base_service_info);
    let (reply_to, correlation_id) = (
        Rc::new(msg.properties.reply_to().clone().unwrap_or_default()),
        Rc::new(msg.properties.correlation_id().clone().unwrap_or_default()),
    );
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
    let _ = match send_message_to_service(
        &channel,
        &request,
        Rc::clone(&reply_to),
        Rc::clone(&correlation_id),
    )
    .await
    {
        Ok(val) => Ok::<(), CustomProjectErrors>(val),
        Err(err) => {
            info!("Got an error while publishing message!");
            send_publish_error_message(
                &request,
                &err.to_string(),
                &channel,
                &connection,
                Rc::clone(&reply_to),
                Rc::clone(&correlation_id),
            )
            .await?;
            return Err(err);
        }
    };
    send_delayed_message(
        &request,
        &channel,
        Rc::clone(&reply_to),
        Rc::clone(&correlation_id),
    )
    .await?;
    Ok(())
}

pub async fn on_service_message(
    msg: Delivery,
    connection: Arc<Pool<Postgres>>,
    channel: Arc<Channel>,
) -> Result<(), CustomProjectErrors> {
    info!("Incoming data for on_service_message");
    let service_response: ServiceResponse =
        RMQDeserializer::from_rabbitmq_json::<ServiceResponse>(&msg.data)?;
    let save_result = save_service_response(&service_response, &connection).await;
    if save_result {
        send_message_to_client(
            &channel,
            &service_response,
            Rc::new(msg.properties.reply_to().clone().unwrap_or_default()),
            Rc::new(msg.properties.correlation_id().clone().unwrap_or_default()),
        )
        .await?;
    }

    Ok(())
}

pub async fn on_fail_message(
    msg: Delivery,
    connection: Arc<Pool<Postgres>>,
    _channel: Arc<Channel>,
) -> Result<(), CustomProjectErrors> {
    info!("Incoming data for on_fail_message");
    let mapped_error: MappedError =
        RMQDeserializer::from_rabbitmq_json::<MappedError>(&msg.data)?;
    save_to_fail_table(&mapped_error, &connection).await;
    Ok(())
}

pub async fn on_timeout_message(
    msg: Delivery,
    connection: Arc<Pool<Postgres>>,
    channel: Arc<Channel>,
) -> Result<(), CustomProjectErrors> {
    info!("Incoming data for on_timeout_message");
    let request: Request = RMQDeserializer::from_rabbitmq_json::<Request>(&msg.data)?;

    let existing_application_response = check_application_response(
        &request.service_info.serhub_request_id,
        &connection,
    )
    .await;
    let insert_incoming_request =
        save_response_with_request(&request, &connection).await;

    if !existing_application_response && insert_incoming_request {
        send_timeout_error_message(&channel, &request, &msg.properties).await?;
        send_timeout_error_service(&channel, &request, &msg.properties).await?;
    };
    Ok(())
}
