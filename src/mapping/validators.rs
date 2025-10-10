use crate::configs::PROJECT_CONFIG;
use validator::ValidationError;

pub fn validate_not_empty(value: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        return Err(ValidationError::new("Value not given"));
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
