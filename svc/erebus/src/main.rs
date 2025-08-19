use axum::{
    response::Json,
    routing::get,
    Router,
};
use serde::Serialize;
use std::{net::SocketAddr, fs};
use tokio;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};
use tracing_appender::{rolling, non_blocking};

mod resources;
use resources::{WalletManager, McpHandler};
use resources::wallets::routes::create_wallet_routes;
use resources::mcp::routes::create_mcp_routes;

#[derive(Serialize)]
struct StatusResponse {
    status: String,
    service: String,
    version: String,
    subsystems: Vec<String>,
}

async fn health_check() -> Json<StatusResponse> {
    Json(StatusResponse {
        status: "healthy".to_string(),
        service: "erebus".to_string(),
        version: "0.1.0".to_string(),
        subsystems: vec![
            "wallets".to_string(),
            "mcp".to_string(),
            "sessions".to_string(),
        ],
    })
}

fn setup_logging() {
    // Create logs directory
    fs::create_dir_all("logs").expect("Failed to create logs directory");
    
    // Setup file appender with daily rotation
    let file_appender = rolling::daily("logs", "erebus.log");
    let (file_writer, _guard) = non_blocking(file_appender);
    
    // Setup error file appender
    let error_appender = rolling::daily("logs", "erebus-errors.log");
    let (error_writer, _error_guard) = non_blocking(error_appender);
    
    // Create layers
    let console_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(false)
        .with_line_number(false);
    
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(file_writer)
        .with_ansi(false)
        .with_target(true);
    
    let error_layer = tracing_subscriber::fmt::layer()
        .with_writer(error_writer)
        .with_ansi(false)
        .with_target(true);
    
    // Initialize subscriber
    tracing_subscriber::registry()
        .with(console_layer)
        .with(file_layer)
        .with(error_layer.with_filter(EnvFilter::new("error")))
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();
    
    // Don't drop the guards - they need to live for the duration of the program
    std::mem::forget(_guard);
    std::mem::forget(_error_guard);
}

#[tokio::main]
async fn main() {
    // Setup logging first
    setup_logging();
    
    info!("============================================================");
    info!("🔥 EREBUS WALLET SERVER STARTING");
    info!("📍 Version: 0.1.0");
    info!("🕐 Timestamp: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"));
    info!("============================================================");
    
    // Initialize subsystem managers
    info!("🏗️  Initializing subsystem managers...");
    let wallet_manager = WalletManager::new();
    let mcp_handler = McpHandler::new();
    info!("✅ Subsystem managers initialized");

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Create main router with organized subsystem routes
    let app = Router::new()
        // Core system endpoints
        .route("/health", get(health_check))
        // Wallet subsystem routes
        .merge(create_wallet_routes().with_state(wallet_manager))
        // MCP subsystem routes  
        .merge(create_mcp_routes().with_state(mcp_handler))
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("🚀 Erebus server listening on {}", addr);
    info!("┌─────────────────────────────────────────────┐");
    info!("│             EREBUS SUBSYSTEMS               │");
    info!("├─────────────────────────────────────────────┤");
    info!("│ 🏥 CORE:");
    info!("│   GET  /health - System health check");
    info!("│");
    info!("│ 👛 WALLET SUBSYSTEM:");
    info!("│   GET  /api/wallets - List supported wallets");
    info!("│   POST /api/wallets/detect - Detect available wallets");
    info!("│   POST /api/wallets/connect - Initiate wallet connection");
    info!("│   GET  /api/wallets/status - Get wallet status");
    info!("│   POST /api/wallets/challenge - Create auth challenge");
    info!("│   POST /api/wallets/verify - Verify wallet signature");
    info!("│   GET  /api/wallets/{{type}}/networks - Get networks");
    info!("│   POST /api/wallets/sessions/validate - Validate session");
    info!("│");
    info!("│ 🔗 MCP SUBSYSTEM:");
    info!("│   POST /mcp - Main MCP protocol endpoint");
    info!("│   POST /mcp/initialize - Initialize MCP server");
    info!("│   POST /mcp/resources - List available resources");
    info!("│   POST /mcp/tools - List available tools"); 
    info!("│   POST /mcp/prompts - List available prompts");
    info!("└─────────────────────────────────────────────┘");
    info!("🎯 Ready for agentic workflows and MCP integration");
    info!("📝 Logs: logs/erebus.log (main), logs/erebus-errors.log (errors)");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    
    // Add graceful shutdown handling
    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
        info!("============================================================");
        info!("🛑 EREBUS WALLET SERVER SHUTTING DOWN");
        info!("🕐 Timestamp: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"));
        info!("============================================================");
        info!("👋 Erebus server shutdown complete");
    };
    
    if let Err(e) = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await
    {
        error!("❌ Server error: {}", e);
    }
}
