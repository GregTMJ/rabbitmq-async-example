use std::str::FromStr;

use crate::configs::PROJECT_CONFIG;
use uuid::Uuid;
use validator::ValidationError;

pub fn validate_not_empty(value: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        return Err(ValidationError::new("Value not given"));
    }
    Ok(())
}

pub fn validate_uuid_value(value: &str) -> Result<(), ValidationError> {
    let check_uuid_valid = Uuid::from_str(value);
    if check_uuid_valid.is_err() {
        return Err(ValidationError::new("Incoming value isn't a Uuid"));
    }
    Ok(())
}

pub fn validate_incoming_service_id(service_id: i32) -> Result<(), ValidationError> {
    if !PROJECT_CONFIG
        .get_available_services()
        .contains(&service_id)
    {
        return Err(ValidationError::new("Service ID is not available"));
    }
    Ok(())
}

pub fn validate_incoming_system_id(system_id: i32) -> Result<(), ValidationError> {
    if !PROJECT_CONFIG.get_available_systems().contains(&system_id) {
        return Err(ValidationError::new("System ID is not available"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_not_empty_val() {
        let test_data = "test";

        let result = validate_not_empty(test_data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ());
    }

    #[test]
    fn validate_empty_val() {
        let test_data = String::new();
        let expected_error = ValidationError::new("Value not given");

        let result = validate_not_empty(&test_data);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), expected_error);
    }

    #[test]
    fn validate_uuid_value_ok() {
        let test_data = Uuid::new_v4().to_string();
        let result = validate_uuid_value(&test_data);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ());
    }

    #[test]
    fn validate_uuid_value_fail() {
        let test_data = String::from("Foo");
        let expected_error = ValidationError::new("Incoming value isn't a Uuid");

        let result = validate_uuid_value(&test_data);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), expected_error)
    }

    #[test]
    fn validate_incoming_service_id_ok() {
        let test_service_id = 1;

        let result = validate_incoming_service_id(test_service_id);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ());
    }

    #[test]
    fn validate_incoming_service_id_fail() {
        let test_service_id = 999;
        let expected_error = ValidationError::new("Service ID is not available");

        let result = validate_incoming_service_id(test_service_id);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), expected_error);
    }

    #[test]
    fn validate_incoming_system_id_ok() {
        let test_system_id = 1;

        let result = validate_incoming_system_id(test_system_id);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ());
    }

    #[test]
    fn validate_incoming_system_id_fail() {
        let test_system_id = 999;
        let expected_error = ValidationError::new("System ID is not available");

        let result = validate_incoming_system_id(test_system_id);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), expected_error);
    }
}
