use axum::{
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio;

#[derive(Serialize)]
struct StatusResponse {
    status: String,
    service: String,
    version: String,
}

#[derive(Deserialize)]
struct McpRequest {
    method: String,
    params: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct McpResponse {
    result: Option<serde_json::Value>,
    error: Option<String>,
}

async fn health_check() -> Json<StatusResponse> {
    Json(StatusResponse {
        status: "healthy".to_string(),
        service: "erebus".to_string(),
        version: "0.1.0".to_string(),
    })
}

async fn mcp_handler(Json(request): Json<McpRequest>) -> Json<McpResponse> {
    match request.method.as_str() {
        "ping" => Json(McpResponse {
            result: Some(serde_json::json!({"message": "pong"})),
            error: None,
        }),
        _ => Json(McpResponse {
            result: None,
            error: Some("Method not implemented".to_string()),
        }),
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/mcp", post(mcp_handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Erebus server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
