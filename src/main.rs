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
