use serde_json::Value;
use crate::toon::{parse_toon, serialize_toon};

pub fn json_to_toon(json_str: &str) -> Result<String, String> {
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Invalid JSON: {}", e))?;
    
    serialize_toon(&value)
}

pub fn toon_to_json(toon_str: &str) -> Result<String, String> {
    let value = parse_toon(toon_str)?;
    
    serde_json::to_string_pretty(&value)
        .map_err(|e| format!("Failed to serialize JSON: {}", e))
}
