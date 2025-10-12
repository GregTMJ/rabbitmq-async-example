use std::str::FromStr;

use chrono::Local;
use log::{info, warn};
use sqlx::{Pool, Postgres};
use uuid::Uuid as Uuidv4;

use crate::{
    database::models::services::Services,
    errors::CustomProjectErrors,
    mapping::schemas::{MappedError, Request, ServiceResponse},
};

pub async fn get_service_info(
    service_id: &i32,
    connection: &Pool<Postgres>,
) -> Result<Services, CustomProjectErrors> {
    let query_result = sqlx::query_as::<_, Services>("SELECT * FROM services WHERE id = $1")
        .bind(service_id)
        .fetch_one(connection)
        .await;
    match query_result {
        Ok(row) => Ok(row),
        Err(msg) => Err(CustomProjectErrors::DatabaseOperationError(msg.to_string())),
    }
}

pub async fn save_client_request(request: &Request, connection: &Pool<Postgres>) -> bool {
    let client_data = serde_json::json!(&request);
    let request_query = sqlx::query(
        "INSERT INTO application_requests (application_id, serhub_request_id, system_id, service_id, application_data) 
            VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(Uuidv4::from_str(&request.application.application_id).unwrap())
    .bind(Uuidv4::from_str(&request.service_info.serhub_request_id).unwrap())
    .bind(request.application.system_id)
    .bind(request.application.service_id)
    .bind(client_data).execute(connection).await;
    match request_query {
        Ok(_) => {
            info!("Client request saved into database!");
            true
        }
        Err(msg) => {
            warn!("Client request not saved into database: {msg}");
            false
        }
    }
}

pub async fn save_service_response(
    service_response: &ServiceResponse,
    connection: &Pool<Postgres>,
) -> bool {
    let target = serde_json::json!(&service_response.target);
    let json_vector = serde_json::json!(&service_response.status_description);
    let result_query = sqlx::query(
    "INSERT INTO application_responses 
    (application_id, serhub_request_id, system_id, service_id, is_cache, status, status_description, response, target)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",)
    .bind(Uuidv4::from_str(&service_response.application_id).unwrap())
    .bind(Uuidv4::from_str(&service_response.serhub_request_id).unwrap())
    .bind(service_response.system_id)
    .bind(service_response.service_id)
    .bind(service_response.is_cache)
    .bind(&service_response.status)
    .bind(&json_vector)
    .bind(&service_response.response)
    .bind(&target).execute(connection).await;
    match result_query {
        Ok(_) => {
            info!("Response saved in database");
            true
        }
        Err(msg) => {
            warn!("Response not saved in database with error: {msg}");
            false
        }
    }
}

pub async fn save_to_fail_table(mapped_error: &MappedError, connection: &Pool<Postgres>) -> bool {
    let data_as_json = serde_json::json!(mapped_error.data);
    let result_query = sqlx::query(
        "INSERT INTO fail_table (application_id, serhub_request_id, system_id, service_id, error_type, error_message, error_traceback, data, created_at) 
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",)
        .bind(Uuidv4::from_str(&mapped_error.application_id).unwrap())
        .bind(Uuidv4::from_str(&mapped_error.serhub_request_id).unwrap())
        .bind(mapped_error.system_id)
        .bind(mapped_error.service_id)
        .bind(&mapped_error.error_type)
        .bind(&mapped_error.error_message)
        .bind(&mapped_error.error_traceback)
        .bind(&data_as_json)
        .bind(Local::now())
        .execute(connection).await;
    match result_query {
        Ok(_) => {
            info!("Fail data saved in database");
            true
        }
        Err(msg) => {
            warn!("Fail data not saved in database with error: {msg}");
            false
        }
    }
}

pub async fn check_application_response(
    serhub_request_id: &str,
    connection: &Pool<Postgres>,
) -> bool {
    let result_query: Result<bool, _> = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM application_responses WHERE serhub_request_id = $1)",
    )
    .bind(Uuidv4::from_str(serhub_request_id).unwrap())
    .fetch_one(connection)
    .await;
    match result_query {
        Ok(result) => result,
        Err(msg) => {
            warn!("Failed to check existing query in database with error: {msg}");
            false
        }
    }
}

pub async fn save_response_with_request(request: &Request, connection: &Pool<Postgres>) -> bool {
    let result_query = sqlx::query(
        "INSERT INTO service_responses (application_id, serhub_request_id, system_id, service_id, is_cache)
        VALUES ($1, $2, $3, $4, $5)",
    ).bind(Uuidv4::from_str(&request.application.application_id).unwrap())
    .bind(Uuidv4::from_str(&request.service_info.serhub_request_id).unwrap())
    .bind(request.application.system_id)
    .bind(request.application.service_id)
    .bind(false)
    .execute(connection).await;
    match result_query {
        Ok(_) => {
            info!("Inserted a timeout response!");
            true
        }
        Err(_) => {
            info!("timeout response is not inserted!");
            false
        }
    }
}
