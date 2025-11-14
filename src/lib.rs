mod toon;
pub mod converter;

// WASM bindings (only compiled for wasm32 target)
#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(not(target_arch = "wasm32"))]
use converter::{json_to_toon as json_to_toon_internal, toon_to_json as toon_to_json_internal};

// UniFFI bindings (not for WASM)
#[cfg(not(target_arch = "wasm32"))]
mod uniffi_bindings {
    use super::*;
    
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
    #[uniffi::export]
    pub fn json_to_toon(json_data: String) -> Result<String, ToonError> {
        json_to_toon_internal(&json_data).map_err(ToonError::from)
    }

    /// Convert TOON format to JSON string
    #[uniffi::export]
    pub fn toon_to_json(toon_data: String) -> Result<String, ToonError> {
        toon_to_json_internal(&toon_data).map_err(ToonError::from)
    }
}

// Re-export for non-WASM targets
#[cfg(not(target_arch = "wasm32"))]
pub use uniffi_bindings::*;

// Generate the UniFFI scaffolding (0.29+ uses proc-macros) - only for non-WASM
#[cfg(not(target_arch = "wasm32"))]
uniffi::setup_scaffolding!();

