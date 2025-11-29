use std::str::FromStr;

use chrono::Local;
use log::{info, warn};
use sqlx::{Pool, Postgres};
use uuid::Uuid as Uuidv4;

use crate::{
    database::models::{
        ApplicationRequests, ApplicationResponses, FailTable, Services,
    },
    errors::CustomProjectErrors,
    mapping::schemas::{MappedError, Request, ServiceResponse},
};

pub async fn get_service_info(
    service_id: &i32,
    connection: &Pool<Postgres>,
) -> Result<Services, CustomProjectErrors> {
    let query_result =
        sqlx::query_as::<_, Services>("SELECT * FROM services WHERE id = $1")
            .bind(service_id)
            .fetch_one(connection)
            .await;
    match query_result {
        Ok(row) => Ok(row),
        Err(msg) => Err(CustomProjectErrors::DatabaseOperationError(msg.to_string())),
    }
}

pub async fn save_client_request(
    request: &Request,
    connection: &Pool<Postgres>,
) -> Result<bool, CustomProjectErrors> {
    let request_data = ApplicationRequests::try_from(request)?;
    let request_query = sqlx::query(
        "INSERT INTO application_requests (application_id, serhub_request_id, system_id, service_id, application_data) 
            VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(request_data.application_id)
    .bind(request_data.serhub_request_id)
    .bind(request_data.system_id)
    .bind(request_data.service_id)
    .bind(request_data.application_data).execute(connection).await;
    match request_query {
        Ok(_) => {
            info!("Client request saved into database!");
            Ok(true)
        }
        Err(msg) => {
            warn!("Client request not saved into database: {msg}");
            Ok(false)
        }
    }
}

pub async fn save_service_response(
    service_response: &ServiceResponse,
    connection: &Pool<Postgres>,
) -> Result<bool, CustomProjectErrors> {
    let response_to_save = ApplicationResponses::try_from(service_response)?;
    let result_query = sqlx::query(
    "INSERT INTO application_responses 
    (application_id, serhub_request_id, system_id, service_id, is_cache, status, status_description, response, target)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)")
    .bind(response_to_save.application_id)
    .bind(response_to_save.serhub_request_id)
    .bind(response_to_save.system_id)
    .bind(response_to_save.service_id)
    .bind(response_to_save.is_cache)
    .bind(response_to_save.status)
    .bind(response_to_save.status_description)
    .bind(response_to_save.response)
    .bind(response_to_save.target).execute(connection).await;
    match result_query {
        Ok(_) => {
            info!("Response saved in database");
            Ok(true)
        }
        Err(msg) => {
            warn!("Response not saved in database with error: {msg}");
            Ok(false)
        }
    }
}

pub async fn save_to_fail_table(
    mapped_error: &MappedError,
    connection: &Pool<Postgres>,
) -> Result<bool, CustomProjectErrors> {
    let sql_mapped_error = FailTable::try_from(mapped_error)?;
    let result_query = sqlx::query(
        "INSERT INTO fail_table (application_id, serhub_request_id, system_id, service_id, error_type, error_message, error_traceback, data, created_at) 
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",)
        .bind(sql_mapped_error.application_id)
        .bind(sql_mapped_error.serhub_request_id)
        .bind(sql_mapped_error.system_id)
        .bind(sql_mapped_error.service_id)
        .bind(sql_mapped_error.error_type)
        .bind(sql_mapped_error.error_message)
        .bind(sql_mapped_error.error_traceback)
        .bind(sql_mapped_error.data)
        .bind(Local::now())
        .execute(connection).await;
    match result_query {
        Ok(_) => {
            info!("Fail data saved in database");
            Ok(true)
        }
        Err(msg) => {
            warn!("Fail data not saved in database with error: {msg}");
            Ok(false)
        }
    }
}

pub async fn check_application_response(
    serhub_request_id: &str,
    connection: &Pool<Postgres>,
) -> Result<bool, CustomProjectErrors> {
    let result_query: Result<bool, _> = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM application_responses WHERE serhub_request_id = $1)",
    )
    .bind(Uuidv4::from_str(serhub_request_id).map_err(|e| CustomProjectErrors::DatabaseOperationError(e.to_string()))?)
    .fetch_one(connection)
    .await;
    match result_query {
        Ok(result) => Ok(result),
        Err(msg) => {
            warn!("Failed to check existing query in database with error: {msg}");
            Ok(false)
        }
    }
}

pub async fn save_response_with_request(
    request: &Request,
    connection: &Pool<Postgres>,
) -> Result<bool, CustomProjectErrors> {
    let sql_request = ApplicationRequests::try_from(request)?;
    let result_query = sqlx::query(
        "INSERT INTO service_responses (application_id, serhub_request_id, system_id, service_id, is_cache)
        VALUES ($1, $2, $3, $4, $5)",
    ).bind(sql_request.application_id)
    .bind(sql_request.serhub_request_id)
    .bind(sql_request.system_id)
    .bind(sql_request.service_id)
    .bind(false)
    .execute(connection).await;
    match result_query {
        Ok(_) => {
            info!("Inserted a timeout response!");
            Ok(true)
        }
        Err(_) => {
            info!("timeout response is not inserted!");
            Ok(false)
        }
    }
}
