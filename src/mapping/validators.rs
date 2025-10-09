use std::env::{self};
use validator::ValidationError;

pub fn validate_not_empty(value: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        return Err(ValidationError::new("Value not given"));
    }
    Ok(())
}

pub fn validate_incoming_service_id(service_id: i32) -> Result<(), ValidationError> {
    let available_service_ids = env::var("AVAILABLE_SERVICES").unwrap_or_default();
    let service_ids_as_vec: Vec<i32> = available_service_ids
        .split(",")
        .filter(|val| !val.is_empty())
        .map(str::trim)
        .map(|val| val.parse::<i32>().unwrap())
        .collect();
    if !service_ids_as_vec.contains(&service_id) {
        return Err(ValidationError::new("Service ID is not available"));
    }
    Ok(())
}

pub fn validate_incoming_system_id(system_id: i32) -> Result<(), ValidationError> {
    let available_system_ids = env::var("AVAILABLE_USERS").unwrap_or_default();
    let system_ids_as_vec: Vec<i32> = available_system_ids
        .split(",")
        .filter(|val| !val.is_empty())
        .map(str::trim)
        .map(|val| val.parse::<i32>().unwrap())
        .collect();
    if !system_ids_as_vec.contains(&system_id) {
        return Err(ValidationError::new("System ID is not available"));
    }
    Ok(())
}
