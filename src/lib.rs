mod toon;
mod converter;

use converter::{json_to_toon as json_to_toon_internal, toon_to_json as toon_to_json_internal};

/// Error type for TOON conversion operations
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum ToonError {
    #[error("Conversion error: {message}")]
    ConversionError { message: String },
}

impl From<String> for ToonError {
    fn from(message: String) -> Self {
        ToonError::ConversionError { message }
    }
}

/// Convert JSON string to TOON format
/// 
/// # Arguments
/// * `json_data` - A JSON string to convert
/// 
/// # Returns
/// * `Ok(String)` - The converted TOON format
/// * `Err(ToonError)` - Error if conversion fails
#[uniffi::export]
pub fn json_to_toon(json_data: String) -> Result<String, ToonError> {
    json_to_toon_internal(&json_data).map_err(ToonError::from)
}

/// Convert TOON format to JSON string
/// 
/// # Arguments
/// * `toon_data` - A TOON format string to convert
/// 
/// # Returns
/// * `Ok(String)` - The converted JSON string
/// * `Err(ToonError)` - Error if conversion fails
#[uniffi::export]
pub fn toon_to_json(toon_data: String) -> Result<String, ToonError> {
    toon_to_json_internal(&toon_data).map_err(ToonError::from)
}

// Generate the UniFFI scaffolding (0.29+ uses proc-macros)
uniffi::setup_scaffolding!();

