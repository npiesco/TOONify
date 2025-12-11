mod toon;
mod converter;

#[cfg(feature = "job-queue")]
mod job_queue;

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
use glob::glob;
use notify::{Watcher, RecursiveMode, Event, event::{CreateKind, ModifyKind}, EventKind};
use std::sync::mpsc::channel;
use regex::Regex;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

#[cfg(feature = "cache")]
use moka::future::Cache as MokaCache;

#[cfg(feature = "persistent-cache")]
use sled::Db as SledDb;

#[cfg(feature = "rate-limit")]
use tower_governor::{
    governor::GovernorConfigBuilder, 
    GovernorLayer,
    key_extractor::GlobalKeyExtractor,
};

// Use pre-generated protobuf code (no need for protoc/cmake at build time)
mod proto;
pub use proto::generated as pb;

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
    /// Batch convert multiple files in a directory
    Batch {
        /// Input directory containing files to convert
        #[arg(short, long)]
        input_dir: PathBuf,
        
        /// Output directory for converted files
        #[arg(short, long)]
        output_dir: PathBuf,
        
        /// Source format (json or toon, auto-detect if omitted)
        #[arg(long)]
        from: Option<String>,
        
        /// Target format (json or toon, auto-detect if omitted)
        #[arg(long)]
        to: Option<String>,
        
        /// File pattern (e.g., "*.json", defaults to all files)
        #[arg(short, long)]
        pattern: Option<String>,
        
        /// Process subdirectories recursively
        #[arg(short, long)]
        recursive: bool,
        
        /// Enable parallel processing for faster batch conversions
        #[arg(long)]
        parallel: bool,
    },
    /// Watch directory and auto-convert files on change
    Watch {
        /// Input directory to watch
        #[arg(short, long)]
        input_dir: PathBuf,
        
        /// Output directory for converted files
        #[arg(short, long)]
        output_dir: PathBuf,
        
        /// Source format (json or toon, auto-detect if omitted)
        #[arg(long)]
        from: Option<String>,
        
        /// Target format (json or toon, auto-detect if omitted)
        #[arg(long)]
        to: Option<String>,
        
        /// File pattern (e.g., "*.json", defaults to all files)
        #[arg(short, long)]
        pattern: Option<String>,
    },
    /// Start the API server (gRPC + REST)
    Serve {
        /// Enable Moka cache with specified size (number of entries)
        #[arg(long)]
        cache_size: Option<u64>,
        
        /// TTL for cache entries in seconds (optional, no expiration if not set)
        #[arg(long)]
        cache_ttl: Option<u64>,
        
        /// Enable Sled persistent cache (path to database file, e.g., "./cache.db")
        #[arg(long)]
        persistent_cache: Option<String>,
        
        /// Enable job queue for distributed processing
        #[arg(long)]
        enable_job_queue: bool,
        
        /// Number of worker threads for job processing (default: 4)
        #[arg(long, default_value = "4")]
        workers: usize,
        
        /// Job queue backend ("memory" or redis URL like "redis://127.0.0.1:6379")
        #[arg(long)]
        job_queue_backend: Option<String>,
        
        /// Enable rate limiting (requests per window)
        #[arg(long)]
        rate_limit: Option<u32>,
        
        /// Rate limit window in seconds (default: 60)
        #[arg(long, default_value = "60")]
        rate_limit_window: u64,
    },
}

// Moka cache type for high-performance in-memory caching
#[cfg(feature = "cache")]
type MokaConversionCache = Arc<MokaCache<String, String>>;

#[cfg(feature = "cache")]
fn create_moka_cache(size: u64, ttl_seconds: Option<u64>) -> MokaConversionCache {
    let mut builder = MokaCache::builder()
        .max_capacity(size);
    
    if let Some(ttl) = ttl_seconds {
        builder = builder.time_to_live(std::time::Duration::from_secs(ttl));
    }
    
    Arc::new(builder.build())
}

// Sled database type for persistent caching
#[cfg(feature = "persistent-cache")]
type SledCacheDb = Arc<SledDb>;

// Cache state with Moka (hot) and Sled (persistent)
#[derive(Clone)]
struct CacheState {
    #[cfg(feature = "cache")]
    moka: Option<MokaConversionCache>,
    #[cfg(feature = "persistent-cache")]
    sled: Option<SledCacheDb>,
}

// Combined app state for all handlers
#[derive(Clone)]
struct AppState {
    cache: CacheState,
    #[cfg(feature = "job-queue")]
    job_store: Option<job_queue::JobStore>,
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

// Job Queue HTTP Handlers
#[cfg(feature = "job-queue")]
#[derive(Deserialize)]
struct SubmitJobPayload {
    operation: String,
    data: String,
}

#[cfg(feature = "job-queue")]
#[derive(Serialize)]
struct SubmitJobResponse {
    job_id: String,
}

#[cfg(feature = "job-queue")]
async fn submit_job_handler(
    axum::extract::State(app_state): axum::extract::State<AppState>,
    Json(payload): Json<SubmitJobPayload>,
) -> impl IntoResponse {
    if let Some(job_store) = app_state.job_store {
        let job_id = job_queue::submit_job(job_store, payload.operation, payload.data);
        Json(SubmitJobResponse { job_id })
    } else {
        Json(SubmitJobResponse { job_id: "error:job_queue_disabled".to_string() })
    }
}

#[cfg(feature = "job-queue")]
#[derive(Serialize)]
struct JobStatusResponse {
    status: String,
    error: Option<String>,
}

#[cfg(feature = "job-queue")]
async fn get_job_status_handler(
    axum::extract::State(app_state): axum::extract::State<AppState>,
    axum::extract::Path(job_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some(job_store) = app_state.job_store {
        if let Some((status, error)) = job_queue::get_job_status(job_store, &job_id) {
            let status_str = match status {
                job_queue::JobStatus::Pending => "pending",
                job_queue::JobStatus::Processing => "processing",
                job_queue::JobStatus::Completed => "completed",
                job_queue::JobStatus::Failed => "failed",
            };
            Json(JobStatusResponse {
                status: status_str.to_string(),
                error,
            })
        } else {
            Json(JobStatusResponse {
                status: "not_found".to_string(),
                error: Some("Job not found".to_string()),
            })
        }
    } else {
        Json(JobStatusResponse {
            status: "error".to_string(),
            error: Some("Job queue disabled".to_string()),
        })
    }
}

#[cfg(feature = "job-queue")]
#[derive(Serialize)]
struct JobResultResponse {
    result: Option<String>,
}

#[cfg(feature = "job-queue")]
async fn get_job_result_handler(
    axum::extract::State(app_state): axum::extract::State<AppState>,
    axum::extract::Path(job_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    if let Some(job_store) = app_state.job_store {
        let result = job_queue::get_job_result(job_store, &job_id);
        Json(JobResultResponse { result })
    } else {
        Json(JobResultResponse { result: None })
    }
}

#[cfg(feature = "job-queue")]
#[derive(Serialize)]
struct ListJobsResponse {
    jobs: Vec<job_queue::Job>,
}

#[cfg(feature = "job-queue")]
async fn list_jobs_handler(
    axum::extract::State(app_state): axum::extract::State<AppState>,
) -> impl IntoResponse {
    if let Some(job_store) = app_state.job_store {
        let jobs = job_queue::list_jobs(job_store);
        Json(ListJobsResponse { jobs })
    } else {
        Json(ListJobsResponse { jobs: vec![] })
    }
}

async fn json_to_toon_handler(
    axum::extract::State(app_state): axum::extract::State<AppState>,
    Json(payload): Json<ConvertPayload>,
) -> impl IntoResponse {
    let cache_state = app_state.cache;
    let cache_key = format!("toonify:json_to_toon:{}", payload.data);
    
    // Try Moka cache first (hot, lock-free, < 100ns)
    #[cfg(feature = "cache")]
    if let Some(ref moka) = cache_state.moka {
        if let Some(cached_result) = moka.get(&cache_key).await {
            eprintln!("[CACHE] Moka hit for json-to-toon");
            return (
                StatusCode::OK,
                Json(ConvertResult {
                    result: Some(cached_result),
                    error: None,
                }),
            );
        }
    }
    
    // Try Sled persistent cache if enabled (cold, ~1ms)
    #[cfg(feature = "persistent-cache")]
    if let Some(ref sled) = cache_state.sled {
        if let Ok(Some(cached_bytes)) = sled.get(cache_key.as_bytes()) {
            if let Ok(cached_result) = String::from_utf8(cached_bytes.to_vec()) {
                eprintln!("[CACHE] Sled hit for json-to-toon");
                
                // Warm up Moka cache from Sled
                #[cfg(feature = "cache")]
                if let Some(ref moka) = cache_state.moka {
                    moka.insert(cache_key.clone(), cached_result.clone()).await;
                }
                
                return (
                    StatusCode::OK,
                    Json(ConvertResult {
                        result: Some(cached_result),
                        error: None,
                    }),
                );
            }
        }
    }
    
    // Cache miss - perform conversion
    match converter::json_to_toon(&payload.data) {
        Ok(result) => {
            // Store in both Moka and Sled (write-through)
            #[cfg(feature = "cache")]
            if let Some(ref moka) = cache_state.moka {
                moka.insert(cache_key.clone(), result.clone()).await;
            }
            
            #[cfg(feature = "persistent-cache")]
            if let Some(ref sled) = cache_state.sled {
                let _ = sled.insert(cache_key.as_bytes(), result.as_bytes());
            }
            
            (
            StatusCode::OK,
            Json(ConvertResult {
                result: Some(result),
                error: None,
            }),
            )
        }
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
    axum::extract::State(app_state): axum::extract::State<AppState>,
    Json(payload): Json<ConvertPayload>,
) -> impl IntoResponse {
    let cache_state = app_state.cache;
    let cache_key = format!("toonify:toon_to_json:{}", payload.data);
    
    // Try Moka cache first (hot, lock-free, < 100ns)
    #[cfg(feature = "cache")]
    if let Some(ref moka) = cache_state.moka {
        if let Some(cached_result) = moka.get(&cache_key).await {
            eprintln!("[CACHE] Moka hit for toon-to-json");
            return (
                StatusCode::OK,
                Json(ConvertResult {
                    result: Some(cached_result),
                    error: None,
                }),
            );
        }
    }
    
    // Try Sled persistent cache if enabled (cold, ~1ms)
    #[cfg(feature = "persistent-cache")]
    if let Some(ref sled) = cache_state.sled {
        if let Ok(Some(cached_bytes)) = sled.get(cache_key.as_bytes()) {
            if let Ok(cached_result) = String::from_utf8(cached_bytes.to_vec()) {
                eprintln!("[CACHE] Sled hit for toon-to-json");
                
                // Warm up Moka cache from Sled
                #[cfg(feature = "cache")]
                if let Some(ref moka) = cache_state.moka {
                    moka.insert(cache_key.clone(), cached_result.clone()).await;
                }
                
                return (
                    StatusCode::OK,
                    Json(ConvertResult {
                        result: Some(cached_result),
                        error: None,
                    }),
                );
            }
        }
    }
    
    // Cache miss - perform conversion
    match converter::toon_to_json(&payload.data) {
        Ok(result) => {
            // Store in both Moka and Sled (write-through)
            #[cfg(feature = "cache")]
            if let Some(ref moka) = cache_state.moka {
                moka.insert(cache_key.clone(), result.clone()).await;
            }
            
            #[cfg(feature = "persistent-cache")]
            if let Some(ref sled) = cache_state.sled {
                let _ = sled.insert(cache_key.as_bytes(), result.as_bytes());
            }
            
            (
            StatusCode::OK,
            Json(ConvertResult {
                result: Some(result),
                error: None,
            }),
            )
        }
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
            
            // Get advanced validation constraints
            let patterns = schema_obj.get("patterns")
                .and_then(|v| v.as_object());
            let ranges = schema_obj.get("ranges")
                .and_then(|v| v.as_object());
            let string_lengths = schema_obj.get("string_lengths")
                .and_then(|v| v.as_object());
            let enums = schema_obj.get("enums")
                .and_then(|v| v.as_object());
            let formats = schema_obj.get("formats")
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
                    
                    let field_value = &item_obj[field_name];
                    
                    // Check field type if specified
                    if let Some(types) = field_types {
                        if let Some(expected_type) = types.get(field_name).and_then(|v| v.as_str()) {
                            validate_field_type(name, idx, field_name, field_value, expected_type)?;
                        }
                    }
                    
                    // Check regex pattern if specified
                    if let Some(patterns_map) = patterns {
                        if let Some(pattern_str) = patterns_map.get(field_name).and_then(|v| v.as_str()) {
                            validate_pattern(name, idx, field_name, field_value, pattern_str)?;
                        }
                    }
                    
                    // Check number range if specified
                    if let Some(ranges_map) = ranges {
                        if let Some(range_obj) = ranges_map.get(field_name).and_then(|v| v.as_object()) {
                            validate_range(name, idx, field_name, field_value, range_obj)?;
                        }
                    }
                    
                    // Check string length if specified
                    if let Some(lengths_map) = string_lengths {
                        if let Some(length_obj) = lengths_map.get(field_name).and_then(|v| v.as_object()) {
                            validate_string_length(name, idx, field_name, field_value, length_obj)?;
                        }
                    }
                    
                    // Check enum values if specified
                    if let Some(enums_map) = enums {
                        if let Some(allowed_values) = enums_map.get(field_name).and_then(|v| v.as_array()) {
                            validate_enum(name, idx, field_name, field_value, allowed_values)?;
                        }
                    }
                    
                    // Check custom format if specified
                    if let Some(formats_map) = formats {
                        if let Some(format_type) = formats_map.get(field_name).and_then(|v| v.as_str()) {
                            validate_format(name, idx, field_name, field_value, format_type)?;
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

fn validate_pattern(entity: &str, idx: usize, field: &str, value: &serde_json::Value, pattern_str: &str) -> Result<(), Box<dyn std::error::Error>> {
    let string_value = value.as_str()
        .ok_or(format!("Item {} in '{}': field '{}' must be a string for pattern matching", idx, entity, field))?;
    
    eprintln!("[VALIDATE] Checking pattern for field '{}': value='{}', pattern='{}'", field, string_value, pattern_str);
    
    let regex = Regex::new(pattern_str)
        .map_err(|e| format!("Invalid regex pattern '{}': {}", pattern_str, e))?;
    
    if !regex.is_match(string_value) {
        return Err(format!(
            "Item {} in '{}': field '{}' value '{}' does not match pattern '{}'",
            idx, entity, field, string_value, pattern_str
        ).into());
    }
    
    eprintln!("[VALIDATE] Pattern match successful for '{}'", field);
    Ok(())
}

fn validate_range(entity: &str, idx: usize, field: &str, value: &serde_json::Value, range_obj: &serde_json::Map<String, serde_json::Value>) -> Result<(), Box<dyn std::error::Error>> {
    let num_value = value.as_f64()
        .ok_or(format!("Item {} in '{}': field '{}' must be a number for range validation", idx, entity, field))?;
    
    eprintln!("[VALIDATE] Checking range for field '{}': value={}", field, num_value);
    
    if let Some(min_val) = range_obj.get("min").and_then(|v| v.as_f64()) {
        eprintln!("[VALIDATE] Checking minimum: {} >= {}", num_value, min_val);
        if num_value < min_val {
            return Err(format!(
                "Item {} in '{}': field '{}' value {} is below minimum {}",
                idx, entity, field, num_value, min_val
            ).into());
        }
    }
    
    if let Some(max_val) = range_obj.get("max").and_then(|v| v.as_f64()) {
        eprintln!("[VALIDATE] Checking maximum: {} <= {}", num_value, max_val);
        if num_value > max_val {
            return Err(format!(
                "Item {} in '{}': field '{}' value {} exceeds maximum {}",
                idx, entity, field, num_value, max_val
            ).into());
        }
    }
    
    eprintln!("[VALIDATE] Range check successful for '{}'", field);
    Ok(())
}

fn validate_string_length(entity: &str, idx: usize, field: &str, value: &serde_json::Value, length_obj: &serde_json::Map<String, serde_json::Value>) -> Result<(), Box<dyn std::error::Error>> {
    let string_value = value.as_str()
        .ok_or(format!("Item {} in '{}': field '{}' must be a string for length validation", idx, entity, field))?;
    
    let length = string_value.len();
    eprintln!("[VALIDATE] Checking string length for field '{}': length={}", field, length);
    
    if let Some(min_len) = length_obj.get("min").and_then(|v| v.as_u64()) {
        eprintln!("[VALIDATE] Checking minimum length: {} >= {}", length, min_len);
        if (length as u64) < min_len {
            return Err(format!(
                "Item {} in '{}': field '{}' length {} is below minimum {}",
                idx, entity, field, length, min_len
            ).into());
        }
    }
    
    if let Some(max_len) = length_obj.get("max").and_then(|v| v.as_u64()) {
        eprintln!("[VALIDATE] Checking maximum length: {} <= {}", length, max_len);
        if (length as u64) > max_len {
            return Err(format!(
                "Item {} in '{}': field '{}' length {} exceeds maximum {}",
                idx, entity, field, length, max_len
            ).into());
        }
    }
    
    eprintln!("[VALIDATE] String length check successful for '{}'", field);
    Ok(())
}

fn validate_enum(entity: &str, idx: usize, field: &str, value: &serde_json::Value, allowed_values: &Vec<serde_json::Value>) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("[VALIDATE] Checking enum for field '{}': value={:?}", field, value);
    eprintln!("[VALIDATE] Allowed values: {:?}", allowed_values);
    
    let is_allowed = allowed_values.iter().any(|allowed| allowed == value);
    
    if !is_allowed {
        let allowed_strs: Vec<String> = allowed_values
            .iter()
            .filter_map(|v| v.as_str().map(|s| format!("'{}'", s)))
            .collect();
        
        return Err(format!(
            "Item {} in '{}': field '{}' value {:?} is not one of allowed values: [{}]",
            idx, entity, field, value, allowed_strs.join(", ")
        ).into());
    }
    
    eprintln!("[VALIDATE] Enum check successful for '{}'", field);
    Ok(())
}

fn validate_format(entity: &str, idx: usize, field: &str, value: &serde_json::Value, format_type: &str) -> Result<(), Box<dyn std::error::Error>> {
    let string_value = value.as_str()
        .ok_or(format!("Item {} in '{}': field '{}' must be a string for format validation", idx, entity, field))?;
    
    eprintln!("[VALIDATE] Checking format for field '{}': value='{}', format='{}'", field, string_value, format_type);
    
    let is_valid = match format_type {
        "email" => {
            // Basic email validation regex
            let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
            email_regex.is_match(string_value)
        }
        "url" => {
            // Basic URL validation
            let url_regex = Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap();
            url_regex.is_match(string_value)
        }
        "date" => {
            // ISO 8601 date format (YYYY-MM-DD)
            let date_regex = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();
            date_regex.is_match(string_value)
        }
        "uuid" => {
            // UUID format
            let uuid_regex = Regex::new(r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$").unwrap();
            uuid_regex.is_match(string_value)
        }
        _ => {
            return Err(format!("Unknown format type: {}", format_type).into());
        }
    };
    
    if !is_valid {
        return Err(format!(
            "Item {} in '{}': field '{}' value '{}' does not match format '{}'",
            idx, entity, field, string_value, format_type
        ).into());
    }
    
    eprintln!("[VALIDATE] Format check successful for '{}'", field);
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

fn run_batch(
    input_dir: PathBuf,
    output_dir: PathBuf,
    from: Option<String>,
    to: Option<String>,
    pattern: Option<String>,
    recursive: bool,
    parallel: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("[BATCH] Starting batch conversion...");
    eprintln!("[BATCH] Input directory: {:?}", input_dir);
    eprintln!("[BATCH] Output directory: {:?}", output_dir);
    eprintln!("[BATCH] Recursive: {}", recursive);
    eprintln!("[BATCH] Parallel: {}", parallel);
    
    if !input_dir.exists() {
        return Err(format!("Input directory does not exist: {:?}", input_dir).into());
    }
    
    // Create output directory if it doesn't exist
    fs::create_dir_all(&output_dir)?;
    eprintln!("[BATCH] Output directory created/verified");
    
    // Build glob pattern
    let glob_pattern = if let Some(pat) = pattern {
        eprintln!("[BATCH] Using pattern: {}", pat);
        if recursive {
            format!("{}/**/{}", input_dir.display(), pat)
        } else {
            format!("{}/{}", input_dir.display(), pat)
        }
    } else {
        eprintln!("[BATCH] Using default pattern (all files)");
        if recursive {
            format!("{}/**/*", input_dir.display())
        } else {
            format!("{}/*", input_dir.display())
        }
    };
    
    eprintln!("[BATCH] Glob pattern: {}", glob_pattern);
    
    // Find all matching files
    let mut files_to_process = Vec::new();
    for entry in glob(&glob_pattern)? {
        match entry {
            Ok(path) => {
                if path.is_file() {
                    files_to_process.push(path);
                }
            }
            Err(e) => eprintln!("[BATCH] Error reading path: {:?}", e),
        }
    }
    
    eprintln!("[BATCH] Found {} files to process", files_to_process.len());
    
    if files_to_process.is_empty() {
        eprintln!("[BATCH] No files found matching pattern");
        return Ok(());
    }
    
    // Use Arc<Mutex<>> for thread-safe counters in parallel mode
    let successful = Arc::new(Mutex::new(0));
    let failed = Arc::new(Mutex::new(0));
    
    // Process files either in parallel or sequentially
    if parallel {
        // Parallel processing with rayon
        files_to_process.par_iter().enumerate().for_each(|(idx, file_path)| {
            eprintln!("[BATCH] Processing file {}/{}: {:?}", idx + 1, files_to_process.len(), file_path);
            
            process_file(
                file_path,
                &input_dir,
                &output_dir,
                &from,
                &to,
                Arc::clone(&successful),
                Arc::clone(&failed),
            );
        });
    } else {
        // Sequential processing
        for (idx, file_path) in files_to_process.iter().enumerate() {
            eprintln!("[BATCH] Processing file {}/{}: {:?}", idx + 1, files_to_process.len(), file_path);
            
            process_file(
                file_path,
                &input_dir,
                &output_dir,
                &from,
                &to,
                Arc::clone(&successful),
                Arc::clone(&failed),
            );
        }
    }
    
    // Extract final counts from Arc<Mutex<>>
    let successful_count = *successful.lock().unwrap();
    let failed_count = *failed.lock().unwrap();
    
    eprintln!("\n[BATCH] ==================== SUMMARY ====================");
    eprintln!("[BATCH] Total files processed: {}", files_to_process.len());
    eprintln!("[BATCH] Successful: {}", successful_count);
    eprintln!("[BATCH] Failed: {}", failed_count);
    eprintln!("[BATCH] ===================================================\n");
    
    println!("Batch conversion completed successfully!");
    println!("Processed {} files ({} successful, {} failed)", files_to_process.len(), successful_count, failed_count);
    
    if failed_count > 0 {
        return Err(format!("{} files failed to convert", failed_count).into());
    }
    
    Ok(())
}

// Helper function to process a single file
fn process_file(
    file_path: &PathBuf,
    input_dir: &PathBuf,
    output_dir: &PathBuf,
    from: &Option<String>,
    to: &Option<String>,
    successful: Arc<Mutex<i32>>,
    failed: Arc<Mutex<i32>>,
) {
    // Read file
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[BATCH] Failed to read file: {}", e);
            *failed.lock().unwrap() += 1;
            return;
        }
    };
    
    // Detect format if not specified
    let source_format = if let Some(f) = from.as_ref() {
        f.as_str()
    } else {
        match detect_format(&content) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("[BATCH] Failed to detect format: {}", e);
                *failed.lock().unwrap() += 1;
                return;
            }
        }
    };
    
    eprintln!("[BATCH] Source format: {}", source_format);
    
    // Determine target format
    let target_format = if let Some(t) = to.as_ref() {
        t.as_str()
    } else {
        // Auto-detect target: if source is JSON, target is TOON, and vice versa
        if source_format == "json" {
            "toon"
        } else {
            "json"
        }
    };
    
    eprintln!("[BATCH] Target format: {}", target_format);
    
    // Convert
    let converted = match (source_format, target_format) {
        ("json", "toon") => converter::json_to_toon(&content),
        ("toon", "json") => converter::toon_to_json(&content),
        ("json", "json") | ("toon", "toon") => {
            eprintln!("[BATCH] Source and target formats are the same, copying file");
            Ok(content)
        }
        _ => {
            eprintln!("[BATCH] Unsupported format combination: {} -> {}", source_format, target_format);
            *failed.lock().unwrap() += 1;
            return;
        }
    };
    
    let converted_content = match converted {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[BATCH] Conversion failed: {}", e);
            *failed.lock().unwrap() += 1;
            return;
        }
    };
    
    // Determine output path
    let relative_path = file_path.strip_prefix(&input_dir)
        .unwrap_or(file_path);
    
    let mut output_path = output_dir.join(relative_path);
    
    // Change extension based on target format
    let new_extension = match target_format {
        "json" => "json",
        "toon" => "toon",
        _ => "txt",
    };
    output_path.set_extension(new_extension);
    
    eprintln!("[BATCH] Output path: {:?}", output_path);
    
    // Create parent directories if needed
    if let Some(parent) = output_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!("[BATCH] Failed to create parent directory: {}", e);
            *failed.lock().unwrap() += 1;
            return;
        }
    }
    
    // Write output
    match fs::write(&output_path, converted_content) {
        Ok(_) => {
            eprintln!("[BATCH] ✓ Successfully converted: {:?}", file_path);
            *successful.lock().unwrap() += 1;
        }
        Err(e) => {
            eprintln!("[BATCH] Failed to write output: {}", e);
            *failed.lock().unwrap() += 1;
        }
    }
}

fn run_watch(
    input_dir: PathBuf,
    output_dir: PathBuf,
    from: Option<String>,
    to: Option<String>,
    pattern: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("[WATCH] Starting watch mode...");
    eprintln!("[WATCH] Watching directory: {:?}", input_dir);
    eprintln!("[WATCH] Output directory: {:?}", output_dir);
    
    if !input_dir.exists() {
        return Err(format!("Input directory does not exist: {:?}", input_dir).into());
    }
    
    // Create output directory
    fs::create_dir_all(&output_dir)?;
    eprintln!("[WATCH] Output directory created/verified");
    
    // Canonicalize paths to handle symlinks like /tmp -> /private/tmp on macOS
    let input_dir = input_dir.canonicalize()?;
    let output_dir = output_dir.canonicalize()?;
    eprintln!("[WATCH] Canonical input: {:?}", input_dir);
    eprintln!("[WATCH] Canonical output: {:?}", output_dir);
    
    
    // Create channel for file system events
    let (tx, rx) = channel();
    
    // Create watcher
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        if let Ok(event) = res {
            let _ = tx.send(event);
        }
    })?;
    
    // Watch only the input directory (not output)
    watcher.watch(&input_dir, RecursiveMode::Recursive)?;
    
    eprintln!("[WATCH] Monitoring for file changes... (Press Ctrl+C to stop)");
    eprintln!("[WATCH] Input: {:?}", input_dir);
    eprintln!("[WATCH] Output: {:?}", output_dir);
    println!("Watch mode active. Monitoring {:?} for changes.", input_dir);
    
    // Process file system events
    loop {
        match rx.recv() {
            Ok(event) => {
                eprintln!("[WATCH] Event: {:?}", event.kind);
                
                // Handle create and modify events
                let should_process = matches!(
                    event.kind,
                    EventKind::Create(CreateKind::File) | 
                    EventKind::Modify(ModifyKind::Data(_)) |
                    EventKind::Modify(ModifyKind::Any)
                );
                
                if should_process {
                    for file_path in event.paths {
                        // Skip non-files or files in output directory
                        if !file_path.is_file() || file_path.starts_with(&output_dir) {
                            continue;
                        }
                        
                        // Check pattern match
                        let matches = if let Some(ref pat) = pattern {
                            if let Some(filename) = file_path.file_name().and_then(|n| n.to_str()) {
                                if pat.contains('*') {
                                    let pat_parts: Vec<&str> = pat.split('*').collect();
                                    if pat_parts.len() == 2 {
                                        filename.starts_with(pat_parts[0]) && filename.ends_with(pat_parts[1])
                                    } else {
                                        false
                                    }
                                } else {
                                    filename == pat
                                }
                            } else {
                                false
                            }
                        } else {
                            true
                        };
                        
                        if !matches {
                            continue;
                        }
                        
                        eprintln!("[WATCH] File changed: {:?}", file_path);
                        
                        // Convert file
                        match (|| -> Result<(), Box<dyn std::error::Error>> {
                            eprintln!("[WATCH] Processing: {:?}", file_path);
                            
                            let content = fs::read_to_string(&file_path)?;
                            
                            let source_format = if let Some(f) = from.as_ref() {
                                f.as_str()
                            } else {
                                detect_format(&content)?
                            };
                            
                            let target_format = if let Some(t) = to.as_ref() {
                                t.as_str()
                            } else {
                                if source_format == "json" { "toon" } else { "json" }
                            };
                            
                            eprintln!("[WATCH] Format: {} -> {}", source_format, target_format);
                            
                            let converted = match (source_format, target_format) {
                                ("json", "toon") => converter::json_to_toon(&content),
                                ("toon", "json") => converter::toon_to_json(&content),
                                ("json", "json") | ("toon", "toon") => Ok(content),
                                _ => return Err(format!("Unsupported conversion: {} -> {}", source_format, target_format).into()),
                            }?;
                            
                            let relative_path = file_path.strip_prefix(&input_dir).unwrap_or(&file_path);
                            let mut output_path = output_dir.join(relative_path);
                            let new_extension = if target_format == "json" { "json" } else { "toon" };
                            output_path.set_extension(new_extension);
                            
                            if let Some(parent) = output_path.parent() {
                                fs::create_dir_all(parent)?;
                            }
                            
                            fs::write(&output_path, converted)?;
                            eprintln!("[WATCH] ✓ Converted: {:?} -> {:?}", file_path, output_path);
                            
                            Ok(())
                        })() {
                            Ok(_) => {},
                            Err(e) => eprintln!("[WATCH] Error converting {:?}: {}", file_path, e),
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("[WATCH] Watch error: {}", e);
                break;
            }
        }
    }
    
    Ok(())
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
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
        Some(Commands::Batch { input_dir, output_dir, from, to, pattern, recursive, parallel }) => {
            // CLI mode - batch convert files
            run_batch(input_dir, output_dir, from, to, pattern, recursive, parallel)?;
            Ok(())
        }
        Some(Commands::Watch { input_dir, output_dir, from, to, pattern }) => {
            // CLI mode - watch directory for changes
            run_watch(input_dir, output_dir, from, to, pattern)?;
            Ok(())
        }
        Some(Commands::Serve { cache_size, cache_ttl, persistent_cache, enable_job_queue, workers, job_queue_backend, rate_limit, rate_limit_window }) => {
            // Server mode
    tracing_subscriber::fmt::init();

            let grpc_addr: SocketAddr = "0.0.0.0:50051".parse()?;
            let http_addr: SocketAddr = "0.0.0.0:5000".parse()?;
            
            // Create Moka cache if requested
            #[cfg(feature = "cache")]
            let moka_cache = if let Some(size) = cache_size {
                if let Some(ttl) = cache_ttl {
                    eprintln!("[CACHE] Moka enabled: {} entries with {}s TTL", size, ttl);
                } else {
                    eprintln!("[CACHE] Moka enabled: {} entries (no TTL)", size);
                }
                Some(create_moka_cache(size, cache_ttl))
            } else {
                None
            };
            
            #[cfg(not(feature = "cache"))]
            let moka_cache: Option<()> = None;
            
            // Create Sled persistent cache if requested
            #[cfg(feature = "persistent-cache")]
            let sled_cache = if let Some(path) = persistent_cache {
                eprintln!("[CACHE] Sled persistent cache enabled: {}", path);
                match sled::open(&path) {
                    Ok(db) => Some(Arc::new(db)),
                    Err(e) => {
                        eprintln!("[ERROR] Failed to open Sled database: {}", e);
                        None
                    }
                }
            } else {
                None
            };
            
            #[cfg(not(feature = "persistent-cache"))]
            let sled_cache: Option<()> = None;
            
            // Log cache status
            if moka_cache.is_none() && sled_cache.is_none() {
                eprintln!("[CACHE] Disabled (no cache configured)");
            }

    let grpc_service = ConverterServiceServer::new(ConverterServiceImpl);

    tokio::spawn(async move {
                eprintln!("[gRPC] Server listening on {}", grpc_addr);
        Server::builder()
            .add_service(grpc_service)
            .serve(grpc_addr)
            .await
            .expect("gRPC server failed");
    });

    let cache_state = CacheState {
        #[cfg(feature = "cache")]
        moka: moka_cache,
        #[cfg(feature = "persistent-cache")]
        sled: sled_cache,
    };
    
    // Initialize job queue if enabled
    #[cfg(feature = "job-queue")]
    let job_store = if enable_job_queue {
        eprintln!("[JOB QUEUE] Enabled with {} workers", workers);
        let store = if let Some(backend) = job_queue_backend {
            if backend.starts_with("redis://") {
                eprintln!("[JOB QUEUE] Using Redis backend: {}", backend);
                // For now, use memory store. Redis implementation would go here.
                job_queue::create_job_store()
            } else {
                eprintln!("[JOB QUEUE] Using in-memory backend");
                job_queue::create_job_store()
            }
        } else {
            eprintln!("[JOB QUEUE] Using in-memory backend");
            job_queue::create_job_store()
        };
        
        // Start worker threads
        job_queue::start_workers(Arc::clone(&store), workers);
        Some(store)
    } else {
        None
    };
    
    // Create combined app state
    #[cfg(feature = "job-queue")]
    let app_state = AppState {
        cache: cache_state,
        job_store,
    };
    
    #[cfg(not(feature = "job-queue"))]
    let app_state = AppState {
        cache: cache_state,
    };
    
    let mut app = Router::new()
        .route("/", get(health_check))
        .route("/json-to-toon", post(json_to_toon_handler))
        .route("/toon-to-json", post(toon_to_json_handler));

    // Add job queue routes if enabled
    #[cfg(feature = "job-queue")]
    if enable_job_queue {
        app = app
            .route("/jobs/submit", post(submit_job_handler))
            .route("/jobs/{job_id}/status", get(get_job_status_handler))
            .route("/jobs/{job_id}/result", get(get_job_result_handler))
            .route("/jobs", get(list_jobs_handler));
    }
    
    let mut app = app.with_state(app_state);
    
    // Add rate limiting if enabled
    #[cfg(feature = "rate-limit")]
    if let Some(limit) = rate_limit {
        eprintln!("[RATE LIMIT] Enabled: {} requests per {} seconds (global rate limiting)", limit, rate_limit_window);
        
        // Use governor's period() method which takes Duration directly
        // For N requests per W seconds, we allow N requests total in a W-second window
        let period = std::time::Duration::from_secs(rate_limit_window) / limit;
        
        // Note: burst_size needs to be limit+1 due to how governor's GCRA algorithm works
        // With burst_size(N), only N-1 rapid requests succeed, so we use N+1 to allow N requests
        let governor_conf = Arc::new(
            GovernorConfigBuilder::default()
                .period(period)                      // Time to replenish 1 token
                .burst_size(limit + 1)               // burst_size + 1 to allow 'limit' rapid requests
                .key_extractor(GlobalKeyExtractor)   // Use global rate limiting (not per-IP)
                .finish()
                .unwrap()
        );
        
        app = app.layer(GovernorLayer::new(governor_conf));
    }
    
    let app = app;
            
            // Bind with custom socket options for better concurrency
            let socket = tokio::net::TcpSocket::new_v4()?;
            socket.set_reuseaddr(true)?;
            socket.bind(http_addr)?;
            let listener = socket.listen(1024)?; // Backlog of 1024 connections
            
            eprintln!("[HTTP] REST API listening on {}", http_addr);
            eprintln!("Endpoints:");
            eprintln!("   GET  /            - Health check");
            eprintln!("   POST /json-to-toon - Convert JSON to TOON");
            eprintln!("   POST /toon-to-json - Convert TOON to JSON");

    axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    tokio::signal::ctrl_c().await.ok();
                })
                .await?;
            Ok(())
        }
        None => {
            // Default to serve mode without cache
            tracing_subscriber::fmt::init();
            
            let grpc_addr: SocketAddr = "0.0.0.0:50051".parse()?;
            let http_addr: SocketAddr = "0.0.0.0:5000".parse()?;
            
            eprintln!("[CACHE] Disabled");
            
            let cache_state = CacheState {
                #[cfg(feature = "cache")]
                moka: None,
                #[cfg(feature = "persistent-cache")]
                sled: None,
            };
            
            #[cfg(feature = "job-queue")]
            let app_state = AppState {
                cache: cache_state,
                job_store: None,
            };
            
            #[cfg(not(feature = "job-queue"))]
            let app_state = AppState {
                cache: cache_state,
            };
            
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
                .route("/toon-to-json", post(toon_to_json_handler))
                .with_state(app_state);
            
            // Bind with custom socket options for better concurrency
            let socket = tokio::net::TcpSocket::new_v4()?;
            socket.set_reuseaddr(true)?;
            socket.bind(http_addr)?;
            let listener = socket.listen(1024)?; // Backlog of 1024 connections
            
            eprintln!("[HTTP] REST API listening on {}", http_addr);
            eprintln!("Endpoints:");
            eprintln!("   GET  /            - Health check");
            eprintln!("   POST /json-to-toon - Convert JSON to TOON");
            eprintln!("   POST /toon-to-json - Convert TOON to JSON");

    axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    tokio::signal::ctrl_c().await.ok();
                })
                .await?;
            Ok(())
        }
    }
}
