mod toon;
mod converter;

use axum::{
    routing::{post, get},
    Router,
    Json,
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use tonic::{transport::Server, Request, Response, Status};
use std::net::SocketAddr;
use tracing_subscriber;

pub mod pb {
    tonic::include_proto!("converter");
}

use pb::converter_service_server::{ConverterService, ConverterServiceServer};
use pb::{ConvertRequest, ConvertResponse};

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
    "TOON Converter API - Blazing Fast!"
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

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let grpc_addr: SocketAddr = "0.0.0.0:50051".parse().unwrap();
    let http_addr: SocketAddr = "0.0.0.0:5000".parse().unwrap();

    let grpc_service = ConverterServiceServer::new(ConverterServiceImpl);

    tokio::spawn(async move {
        println!("[gRPC] Server listening on {}", grpc_addr);
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

    let listener = tokio::net::TcpListener::bind(&http_addr)
        .await
        .expect("Failed to bind");

    println!("[HTTP] REST API listening on {}", http_addr);
    println!("Endpoints:");
    println!("   GET  /            - Health check");
    println!("   POST /json-to-toon - Convert JSON to TOON");
    println!("   POST /toon-to-json - Convert TOON to JSON");

    axum::serve(listener, app)
        .await
        .expect("HTTP server failed");
}
