use axum::{
    extract::Request,
    response::Json,
    routing::{get, post},
    Router,
    middleware::{self, Next},
    http::StatusCode,
};
use serde::Serialize;
use std::net::SocketAddr;
use tokio;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, error, warn};
use tracing_subscriber::{EnvFilter, Layer};

// Import our modules
mod resources;
use resources::agents::routes::{
    agent_health, hecate_chat, hecate_status, agent_chat, agent_status,
    hecate_personality, hecate_clear, hecate_history
};

#[derive(Serialize)]
struct StatusResponse {
    status: String,
    service: String,
    version: String,
    message: String,
}

async fn health_check() -> Json<StatusResponse> {
    Json(StatusResponse {
        status: "healthy".to_string(),
        service: "erebus".to_string(),
        version: "0.1.0".to_string(),
        message: "🎯 Ready for agentic workflows and MCP integration".to_string(),
    })
}

async fn root() -> Json<StatusResponse> {
    let response = StatusResponse {
        status: "running".to_string(),
        service: "erebus".to_string(),
        version: "0.1.0".to_string(),
        message: "💡 Erebus - Nullblock Wallet & MCP Server".to_string(),
    };
    
    info!("📤 Root endpoint response: {}", serde_json::to_string_pretty(&response).unwrap_or_default());
    Json(response)
}

/// Logging middleware for all requests
async fn logging_middleware(request: Request, next: Next) -> Result<axum::response::Response, StatusCode> {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let headers = request.headers().clone();
    
    // Extract request body for logging (for POST requests)
    let (parts, body) = request.into_parts();
    let body_bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("❌ Failed to read request body: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    
    // Log incoming request
    info!("📥 Incoming request: {} {}", method, uri);
    info!("📋 Request headers: {:#?}", headers);
    
    if !body_bytes.is_empty() {
        match serde_json::from_slice::<serde_json::Value>(&body_bytes) {
            Ok(json) => {
                info!("📝 Request body (JSON): {}", serde_json::to_string_pretty(&json).unwrap_or_default());
            }
            Err(_) => {
                // Not JSON, log as string if it's valid UTF-8
                match String::from_utf8(body_bytes.to_vec()) {
                    Ok(text) => info!("📝 Request body (Text): {}", text),
                    Err(_) => info!("📝 Request body: {} bytes (binary)", body_bytes.len()),
                }
            }
        }
    }
    
    // Rebuild request
    let request = Request::from_parts(parts, axum::body::Body::from(body_bytes));
    
    // Process request
    let start_time = std::time::Instant::now();
    let response = next.run(request).await;
    let duration = start_time.elapsed();
    
    // Log response
    info!("📤 Response: {} {} -> {} ({:.2}ms)", 
          method, uri, response.status(), duration.as_millis());
    
    Ok(response)
}

fn setup_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_filter(filter))
        .init();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logging();

    info!("🚀 Starting Erebus server...");
    info!("📍 Version: 0.1.0");
    info!("🕐 Timestamp: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"));
    info!("============================================================");
    
    // Create router with CORS and agent routes
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        // Agent routing endpoints
        .route("/api/agents/health", get(agent_health))
        .route("/api/agents/hecate/chat", post(hecate_chat))
        .route("/api/agents/hecate/status", get(hecate_status))
        .route("/api/agents/hecate/personality", post(hecate_personality))
        .route("/api/agents/hecate/clear", post(hecate_clear))
        .route("/api/agents/hecate/history", get(hecate_history))
        .route("/api/agents/:agent_name/chat", post(agent_chat))
        .route("/api/agents/:agent_name/status", get(agent_status))
        // Add logging middleware
        .layer(middleware::from_fn(logging_middleware))
        // Add CORS layer
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    info!("🌐 Server starting on http://0.0.0.0:3000");
    info!("🏥 Health check: http://localhost:3000/health");
    info!("🤖 Agent routing: http://localhost:3000/api/agents/health");
    info!("💬 Hecate chat: http://localhost:3000/api/agents/hecate/chat");
    info!("📊 Hecate status: http://localhost:3000/api/agents/hecate/status");
    info!("⚙️ Hecate personality: http://localhost:3000/api/agents/hecate/personality");
    info!("🧹 Hecate clear: http://localhost:3000/api/agents/hecate/clear");
    info!("📜 Hecate history: http://localhost:3000/api/agents/hecate/history");
    info!("💡 Ready for agentic workflows and MCP integration");

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    info!("✅ Server listening on {}", addr);
    
    axum::serve(listener, app).await?;

    Ok(())
}