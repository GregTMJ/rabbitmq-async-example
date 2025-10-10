pub mod schemas;
pub mod validators;

use crate::mapping::schemas::RMQDeserializer;

pub fn from_json_slice<T>(value: Vec<u8>) -> Result<T, String>
where
    T: RMQDeserializer,
{
    serde_json::from_slice(&value)
        .map_err(|e| format!("Got an error while trying to deserialize: {e}"))
}
