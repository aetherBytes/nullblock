use axum::{
    response::Json,
    routing::get,
    Router,
};
use serde::Serialize;
use std::net::SocketAddr;
use tokio;
use tower_http::cors::{Any, CorsLayer};

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

#[tokio::main]
async fn main() {
    // Initialize subsystem managers
    let wallet_manager = WalletManager::new();
    let mcp_handler = McpHandler::new();

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
    println!("🔥 Erebus server listening on {}", addr);
    println!("┌─────────────────────────────────────────────┐");
    println!("│             EREBUS SUBSYSTEMS               │");
    println!("├─────────────────────────────────────────────┤");
    println!("│ 🏥 CORE:");
    println!("│   GET  /health - System health check");
    println!("│");
    println!("│ 👛 WALLET SUBSYSTEM:");
    println!("│   GET  /api/wallets - List supported wallets");
    println!("│   POST /api/wallets/detect - Detect available wallets");
    println!("│   POST /api/wallets/connect - Initiate wallet connection");
    println!("│   GET  /api/wallets/status - Get wallet status");
    println!("│   POST /api/wallets/challenge - Create auth challenge");
    println!("│   POST /api/wallets/verify - Verify wallet signature");
    println!("│   GET  /api/wallets/{{type}}/networks - Get networks");
    println!("│   POST /api/wallets/sessions/validate - Validate session");
    println!("│");
    println!("│ 🔗 MCP SUBSYSTEM:");
    println!("│   POST /mcp - Main MCP protocol endpoint");
    println!("│   POST /mcp/initialize - Initialize MCP server");
    println!("│   POST /mcp/resources - List available resources");
    println!("│   POST /mcp/tools - List available tools"); 
    println!("│   POST /mcp/prompts - List available prompts");
    println!("└─────────────────────────────────────────────┘");
    println!("💡 Ready for agentic workflows and MCP integration");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
