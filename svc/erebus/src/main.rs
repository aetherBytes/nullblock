use axum::{
    response::Json,
    routing::get,
    Router,
};
use serde::Serialize;
use std::net::SocketAddr;
use tokio;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, error};
use tracing_subscriber::{EnvFilter, Layer};

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
    Json(StatusResponse {
        status: "running".to_string(),
        service: "erebus".to_string(),
        version: "0.1.0".to_string(),
        message: "💡 Erebus - Nullblock Wallet & MCP Server".to_string(),
    })
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
    
    // Create router with CORS
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    info!("🌐 Server starting on http://0.0.0.0:3000");
    info!("🏥 Health check: http://localhost:3000/health");
    info!("💡 Ready for agentic workflows and MCP integration");

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    info!("✅ Server listening on {}", addr);
    
    axum::serve(listener, app).await?;

    Ok(())
}