mod toon;
mod converter;

use axum::{
    routing::{post, get},
    Router,
    Json,
    http::StatusCode,
    response::IntoResponse,
};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use tonic::{transport::Server, Request, Response, Status};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::io::{self, Read, Write};
use std::fs;
use tracing_subscriber;
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder as GzEncoderWrite;

pub mod pb {
    tonic::include_proto!("converter");
}

use pb::converter_service_server::{ConverterService, ConverterServiceServer};
use pb::{ConvertRequest, ConvertResponse};

#[derive(Parser)]
#[command(name = "toonify")]
#[command(about = "TOONify - High-performance JSON ↔ TOON converter", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert between JSON and TOON formats
    Convert {
        /// Input file path (use '-' for stdin)
        input: String,
        
        /// Output file path (defaults to stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Compress TOON data with gzip
    Compress {
        /// Input file path (omit for stdin)
        #[arg(short, long)]
        input: Option<PathBuf>,
        
        /// Output file path (omit for stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Decompress gzip-compressed TOON data
    Decompress {
        /// Input file path (omit for stdin)
        #[arg(short, long)]
        input: Option<PathBuf>,
        
        /// Output file path (omit for stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Validate TOON data against a schema
    Validate {
        /// Schema file path (JSON format)
        #[arg(short, long)]
        schema: PathBuf,
        
        /// Input TOON file path (omit for stdin)
        #[arg(short, long)]
        input: Option<PathBuf>,
    },
    /// Start the API server (gRPC + REST)
    Serve,
}

#[derive(Clone)]
struct ConverterServiceImpl;

#[tonic::async_trait]
impl ConverterService for ConverterServiceImpl {
    async fn json_to_toon(
        &self,
        request: Request<ConvertRequest>,
    ) -> Result<Response<ConvertResponse>, Status> {
        let req = request.into_inner();
        
        match converter::json_to_toon(&req.data) {
            Ok(result) => Ok(Response::new(ConvertResponse {
                result,
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(ConvertResponse {
                result: String::new(),
                error: e,
            })),
        }
    }

    async fn toon_to_json(
        &self,
        request: Request<ConvertRequest>,
    ) -> Result<Response<ConvertResponse>, Status> {
        let req = request.into_inner();
        
        match converter::toon_to_json(&req.data) {
            Ok(result) => Ok(Response::new(ConvertResponse {
                result,
                error: String::new(),
            })),
            Err(e) => Ok(Response::new(ConvertResponse {
                result: String::new(),
                error: e,
            })),
        }
    }
}

#[derive(Deserialize)]
struct ConvertPayload {
    data: String,
}

#[derive(Serialize)]
struct ConvertResult {
    result: Option<String>,
    error: Option<String>,
}

async fn health_check() -> &'static str {
    "TOONify API - Blazing Fast!"
}

async fn json_to_toon_handler(
    Json(payload): Json<ConvertPayload>,
) -> impl IntoResponse {
    match converter::json_to_toon(&payload.data) {
        Ok(result) => (
            StatusCode::OK,
            Json(ConvertResult {
                result: Some(result),
                error: None,
            }),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ConvertResult {
                result: None,
                error: Some(e),
            }),
        ),
    }
}

async fn toon_to_json_handler(
    Json(payload): Json<ConvertPayload>,
) -> impl IntoResponse {
    match converter::toon_to_json(&payload.data) {
        Ok(result) => (
            StatusCode::OK,
            Json(ConvertResult {
                result: Some(result),
                error: None,
            }),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ConvertResult {
                result: None,
                error: Some(e),
            }),
        ),
    }
}

fn detect_format(content: &str) -> Result<&'static str, String> {
    let trimmed = content.trim();
    
    // Try to parse as JSON first
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        match serde_json::from_str::<serde_json::Value>(trimmed) {
            Ok(_) => return Ok("json"),
            Err(_) => {}
        }
    }
    
    // Check for TOON format patterns
    if trimmed.contains("{") && trimmed.contains("}:") {
        return Ok("toon");
    }
    
    // Default: try to parse as JSON
    match serde_json::from_str::<serde_json::Value>(trimmed) {
        Ok(_) => Ok("json"),
        Err(_) => Ok("toon"), // Assume TOON if JSON parse fails
    }
}

fn run_compress(input: Option<PathBuf>, output: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("[COMPRESS] Starting compression...");
    
    // Read input
    let input_data = if let Some(input_path) = input {
        eprintln!("[COMPRESS] Reading from file: {:?}", input_path);
        fs::read(&input_path)?
    } else {
        eprintln!("[COMPRESS] Reading from STDIN");
        let mut buffer = Vec::new();
        io::stdin().read_to_end(&mut buffer)?;
        buffer
    };
    
    eprintln!("[COMPRESS] Input size: {} bytes", input_data.len());
    
    // Compress using gzip
    let mut encoder = GzEncoderWrite::new(Vec::new(), Compression::default());
    encoder.write_all(&input_data)?;
    let compressed_data = encoder.finish()?;
    
    eprintln!("[COMPRESS] Compressed size: {} bytes", compressed_data.len());
    let ratio = (1.0 - (compressed_data.len() as f64 / input_data.len() as f64)) * 100.0;
    eprintln!("[COMPRESS] Compression ratio: {:.2}%", ratio);
    
    // Write output
    if let Some(output_path) = output {
        eprintln!("[COMPRESS] Writing to file: {:?}", output_path);
        fs::write(output_path, compressed_data)?;
        eprintln!("[COMPRESS] File written successfully");
    } else {
        eprintln!("[COMPRESS] Writing to STDOUT");
        io::stdout().write_all(&compressed_data)?;
        io::stdout().flush()?;
    }
    
    Ok(())
}

fn run_decompress(input: Option<PathBuf>, output: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("[DECOMPRESS] Starting decompression...");
    
    // Read input
    let compressed_data = if let Some(input_path) = input {
        eprintln!("[DECOMPRESS] Reading from file: {:?}", input_path);
        fs::read(&input_path)?
    } else {
        eprintln!("[DECOMPRESS] Reading from STDIN");
        let mut buffer = Vec::new();
        io::stdin().read_to_end(&mut buffer)?;
        buffer
    };
    
    eprintln!("[DECOMPRESS] Compressed size: {} bytes", compressed_data.len());
    
    // Decompress using gzip
    let mut decoder = GzDecoder::new(&compressed_data[..]);
    let mut decompressed_data = Vec::new();
    decoder.read_to_end(&mut decompressed_data)?;
    
    eprintln!("[DECOMPRESS] Decompressed size: {} bytes", decompressed_data.len());
    let ratio = (decompressed_data.len() as f64 / compressed_data.len() as f64 - 1.0) * 100.0;
    eprintln!("[DECOMPRESS] Expansion ratio: {:.2}%", ratio);
    
    // Write output
    if let Some(output_path) = output {
        eprintln!("[DECOMPRESS] Writing to file: {:?}", output_path);
        fs::write(output_path, decompressed_data)?;
        eprintln!("[DECOMPRESS] File written successfully");
    } else {
        eprintln!("[DECOMPRESS] Writing to STDOUT");
        io::stdout().write_all(&decompressed_data)?;
        io::stdout().flush()?;
    }
    
    Ok(())
}

fn run_validate(schema_path: PathBuf, input: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("[VALIDATE] Starting validation...");
    
    // Read schema
    eprintln!("[VALIDATE] Reading schema from: {:?}", schema_path);
    let schema_content = fs::read_to_string(&schema_path)?;
    let schema: serde_json::Value = serde_json::from_str(&schema_content)
        .map_err(|e| format!("Invalid schema JSON: {}", e))?;
    
    eprintln!("[VALIDATE] Schema loaded successfully");
    
    // Read TOON input
    let toon_data = if let Some(input_path) = input {
        eprintln!("[VALIDATE] Reading TOON from file: {:?}", input_path);
        fs::read_to_string(&input_path)?
    } else {
        eprintln!("[VALIDATE] Reading TOON from STDIN");
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    };
    
    eprintln!("[VALIDATE] TOON data size: {} bytes", toon_data.len());
    
    // Convert TOON to JSON for validation
    eprintln!("[VALIDATE] Parsing TOON data...");
    let json_value = converter::toon_to_json(&toon_data)
        .map_err(|e| format!("Failed to parse TOON: {}", e))?;
    
    let parsed_value: serde_json::Value = serde_json::from_str(&json_value)?;
    eprintln!("[VALIDATE] TOON parsed successfully");
    
    // Validate against schema
    eprintln!("[VALIDATE] Validating against schema...");
    validate_value(&parsed_value, &schema)?;
    
    eprintln!("[VALIDATE] ✓ Validation passed!");
    println!("✓ TOON data is valid according to schema");
    
    Ok(())
}

fn validate_value(value: &serde_json::Value, schema: &serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("[VALIDATE] Validating data structure...");
    
    let schema_obj = schema.as_object()
        .ok_or("Schema must be a JSON object")?;
    
    let value_obj = value.as_object()
        .ok_or("TOON data must represent an object")?;
    
    // Validate each entity in schema
    for (entity_name, entity_schema) in schema_obj {
        eprintln!("[VALIDATE] Validating entity: {}", entity_name);
        
        if !value_obj.contains_key(entity_name) {
            return Err(format!("Missing entity '{}' in TOON data", entity_name).into());
        }
        
        let entity_value = &value_obj[entity_name];
        validate_entity(entity_name, entity_value, entity_schema)?;
    }
    
    eprintln!("[VALIDATE] All entities validated successfully");
    Ok(())
}

fn validate_entity(name: &str, value: &serde_json::Value, schema: &serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
    let schema_obj = schema.as_object()
        .ok_or(format!("Schema for '{}' must be an object", name))?;
    
    // Check type
    let entity_type = schema_obj.get("type")
        .and_then(|v| v.as_str())
        .ok_or(format!("Schema for '{}' must have 'type' field", name))?;
    
    eprintln!("[VALIDATE] Entity '{}' type: {}", name, entity_type);
    
    match entity_type {
        "array" => {
            let array = value.as_array()
                .ok_or(format!("Entity '{}' must be an array", name))?;
            
            eprintln!("[VALIDATE] Array '{}' has {} items", name, array.len());
            
            // Check min/max items
            if let Some(min_items) = schema_obj.get("min_items").and_then(|v| v.as_u64()) {
                eprintln!("[VALIDATE] Checking min_items: {} (actual: {})", min_items, array.len());
                if (array.len() as u64) < min_items {
                    return Err(format!(
                        "Entity '{}' has {} items but minimum is {}",
                        name, array.len(), min_items
                    ).into());
                }
            }
            
            if let Some(max_items) = schema_obj.get("max_items").and_then(|v| v.as_u64()) {
                eprintln!("[VALIDATE] Checking max_items: {} (actual: {})", max_items, array.len());
                if (array.len() as u64) > max_items {
                    return Err(format!(
                        "Entity '{}' has {} items but maximum is {}",
                        name, array.len(), max_items
                    ).into());
                }
            }
            
            // Get required fields
            let required_fields = schema_obj.get("fields")
                .and_then(|v| v.as_array())
                .ok_or(format!("Schema for '{}' must have 'fields' array", name))?;
            
            let required_field_names: Vec<String> = required_fields
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
            
            eprintln!("[VALIDATE] Required fields: {:?}", required_field_names);
            
            // Get field types if specified
            let field_types = schema_obj.get("field_types")
                .and_then(|v| v.as_object());
            
            // Validate each item in array
            for (idx, item) in array.iter().enumerate() {
                eprintln!("[VALIDATE] Validating item {}", idx);
                
                let item_obj = item.as_object()
                    .ok_or(format!("Item {} in '{}' must be an object", idx, name))?;
                
                // Check all required fields are present
                for field_name in &required_field_names {
                    if !item_obj.contains_key(field_name) {
                        return Err(format!(
                            "Item {} in '{}' is missing required field '{}'",
                            idx, name, field_name
                        ).into());
                    }
                    
                    // Check field type if specified
                    if let Some(types) = field_types {
                        if let Some(expected_type) = types.get(field_name).and_then(|v| v.as_str()) {
                            let field_value = &item_obj[field_name];
                            validate_field_type(name, idx, field_name, field_value, expected_type)?;
                        }
                    }
                }
            }
            
            eprintln!("[VALIDATE] Array '{}' validated successfully", name);
        }
        _ => {
            return Err(format!("Unsupported entity type: {}", entity_type).into());
        }
    }
    
    Ok(())
}

fn validate_field_type(entity: &str, idx: usize, field: &str, value: &serde_json::Value, expected_type: &str) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("[VALIDATE] Checking field '{}' type: expected {}, got {:?}", field, expected_type, value);
    
    let matches = match expected_type {
        "string" => value.is_string(),
        "number" => value.is_number(),
        "boolean" => value.is_boolean(),
        "null" => value.is_null(),
        _ => return Err(format!("Unknown type: {}", expected_type).into()),
    };
    
    if !matches {
        return Err(format!(
            "Item {} in '{}': field '{}' has wrong type (expected {}, got {})",
            idx,
            entity,
            field,
            expected_type,
            if value.is_string() { "string" }
            else if value.is_number() { "number" }
            else if value.is_boolean() { "boolean" }
            else if value.is_null() { "null" }
            else { "unknown" }
        ).into());
    }
    
    Ok(())
}

fn run_convert(input: String, output: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("[CLI] Reading input...");
    
    // Read input
    let input_content = if input == "-" {
        eprintln!("[CLI] Reading from STDIN");
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        eprintln!("[CLI] Reading from file: {}", input);
        fs::read_to_string(&input)?
    };
    
    eprintln!("[CLI] Input size: {} bytes", input_content.len());
    
    // Detect format
    let format = detect_format(&input_content)?;
    eprintln!("[CLI] Detected format: {}", format);
    
    // Convert
    let output_content = match format {
        "json" => {
            eprintln!("[CLI] Converting JSON → TOON");
            converter::json_to_toon(&input_content)
                .map_err(|e| format!("Conversion failed: {}", e))?
        }
        "toon" => {
            eprintln!("[CLI] Converting TOON → JSON");
            converter::toon_to_json(&input_content)
                .map_err(|e| format!("Conversion failed: {}", e))?
        }
        _ => return Err("Unknown format".into()),
    };
    
    eprintln!("[CLI] Conversion successful");
    eprintln!("[CLI] Output size: {} bytes", output_content.len());
    
    // Write output
    if let Some(output_path) = output {
        eprintln!("[CLI] Writing to file: {:?}", output_path);
        fs::write(output_path, output_content)?;
        eprintln!("[CLI] File written successfully");
    } else {
        eprintln!("[CLI] Writing to STDOUT");
        io::stdout().write_all(output_content.as_bytes())?;
        io::stdout().flush()?;
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    match cli.command {
        Some(Commands::Convert { input, output }) => {
            // CLI mode - convert file
            run_convert(input, output)?;
            Ok(())
        }
        Some(Commands::Compress { input, output }) => {
            // CLI mode - compress data
            run_compress(input, output)?;
            Ok(())
        }
        Some(Commands::Decompress { input, output }) => {
            // CLI mode - decompress data
            run_decompress(input, output)?;
            Ok(())
        }
        Some(Commands::Validate { schema, input }) => {
            // CLI mode - validate TOON against schema
            run_validate(schema, input)?;
            Ok(())
        }
        Some(Commands::Serve) | None => {
            // Server mode (default)
            tracing_subscriber::fmt::init();
            
            let grpc_addr: SocketAddr = "0.0.0.0:50051".parse()?;
            let http_addr: SocketAddr = "0.0.0.0:5000".parse()?;
            
            let grpc_service = ConverterServiceServer::new(ConverterServiceImpl);
            
            tokio::spawn(async move {
                eprintln!("[gRPC] Server listening on {}", grpc_addr);
                Server::builder()
                    .add_service(grpc_service)
                    .serve(grpc_addr)
                    .await
                    .expect("gRPC server failed");
            });
            
            let app = Router::new()
                .route("/", get(health_check))
                .route("/json-to-toon", post(json_to_toon_handler))
                .route("/toon-to-json", post(toon_to_json_handler));
            
            let listener = tokio::net::TcpListener::bind(&http_addr).await?;
            
            eprintln!("[HTTP] REST API listening on {}", http_addr);
            eprintln!("Endpoints:");
            eprintln!("   GET  /            - Health check");
            eprintln!("   POST /json-to-toon - Convert JSON to TOON");
            eprintln!("   POST /toon-to-json - Convert TOON to JSON");
            
            axum::serve(listener, app).await?;
            Ok(())
        }
    }
}
