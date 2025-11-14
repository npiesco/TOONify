// WASM bindings for TOONify
// Exposes core conversion functions to JavaScript/TypeScript

use wasm_bindgen::prelude::*;

// Set up better panic messages in the browser console
pub fn set_panic_hook() {
    console_error_panic_hook::set_once();
}

// Use wee_alloc as the global allocator for smaller WASM binary size
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Initialize the WASM module
/// Call this before using any other functions
#[wasm_bindgen(start)]
pub fn init() {
    set_panic_hook();
}

/// Convert JSON string to TOON format
/// 
/// # Arguments
/// * `json_str` - A JSON string to convert
/// 
/// # Returns
/// * `Result<String, JsValue>` - The TOON formatted string or an error
/// 
/// # Example
/// ```javascript
/// import { json_to_toon } from './pkg/toonify';
/// 
/// const json = '{"users":[{"id":1,"name":"Alice"}]}';
/// const toon = json_to_toon(json);
/// console.log(toon);
/// // Output: users[1]{id,name}:\n1,Alice
/// ```
#[wasm_bindgen]
pub fn json_to_toon(json_str: &str) -> Result<String, JsValue> {
    crate::converter::json_to_toon(json_str)
        .map_err(|e| JsValue::from_str(&format!("Conversion error: {}", e)))
}

/// Convert TOON format string to JSON
/// 
/// # Arguments
/// * `toon_str` - A TOON formatted string to convert
/// 
/// # Returns
/// * `Result<String, JsValue>` - The JSON string or an error
/// 
/// # Example
/// ```javascript
/// import { toon_to_json } from './pkg/toonify';
/// 
/// const toon = 'users[1]{id,name}:\n1,Alice';
/// const json = toon_to_json(toon);
/// console.log(json);
/// // Output: {"users":[{"id":1,"name":"Alice"}]}
/// ```
#[wasm_bindgen]
pub fn toon_to_json(toon_str: &str) -> Result<String, JsValue> {
    crate::converter::toon_to_json(toon_str)
        .map_err(|e| JsValue::from_str(&format!("Conversion error: {}", e)))
}

/// Get the version of the TOONify WASM module
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_json_to_toon() {
        let json = r#"{"users":[{"id":1,"name":"Alice"}]}"#;
        let result = json_to_toon(json);
        assert!(result.is_ok());
        let toon = result.unwrap();
        assert!(toon.contains("users"));
        assert!(toon.contains("Alice"));
    }

    #[test]
    fn test_wasm_toon_to_json() {
        let toon = "users[1]{id,name}:\n1,Alice";
        let result = toon_to_json(toon);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("users"));
        assert!(json.contains("Alice"));
    }

    #[test]
    fn test_wasm_version() {
        let ver = version();
        assert!(!ver.is_empty());
    }
}

