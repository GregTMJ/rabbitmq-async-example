use std::str::FromStr;

use log::{info, warn};
use sqlx::{Pool, Postgres};
use uuid::Uuid as Uuidv4;

use crate::{database::models::services::Services, mapping::schemas::ServiceResponse};

pub async fn get_service_info(
    service_id: &i32,
    connection: &Pool<Postgres>,
) -> Result<Services, String> {
    let query_result = sqlx::query_as::<_, Services>("SELECT * FROM services WHERE id = $1")
        .bind(service_id)
        .fetch_one(connection)
        .await;
    match query_result {
        Ok(row) => Ok(row),
        Err(msg) => Err(msg.to_string()),
    }
}

pub async fn save_service_response(
    service_response: &ServiceResponse,
    connection: &Pool<Postgres>,
) -> Result<bool, String> {
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
            Ok(true)
        }
        Err(msg) => {
            warn!("Response not saved in database with error: {msg}");
            Ok(false)
        }
    }
}
