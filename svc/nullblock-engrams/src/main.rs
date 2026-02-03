use std::env;
use std::net::SocketAddr;

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, warn};

mod config;
mod database;
mod error;
mod handlers;
mod models;
mod server;

use crate::config::Config;
use crate::handlers::{engrams, health};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment from .env.dev if available
    if let Err(e) = dotenv::from_filename(".env.dev") {
        warn!("⚠️ Could not load .env.dev file: {}", e);
        warn!("   Expected location: .env.dev in current directory");
    } else {
        println!("✅ Loaded configuration from .env.dev");
    }

    // Setup logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,nullblock_engrams=debug".into()),
        )
        .init();

    // Load configuration
    let config = Config::from_env()?;
    info!("🔧 Configuration loaded: {}", config.service_name);

    // Create the application state
    let state = server::AppState::new(config.clone()).await?;

    // Build the router
    let app = create_router(state);

    // Get port from config or environment
    let port = env::var("ENGRAMS_PORT")
        .unwrap_or_else(|_| "9004".to_string())
        .parse::<u16>()
        .unwrap_or(9004);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let engrams_base_url =
        env::var("ENGRAMS_SERVICE_URL").unwrap_or_else(|_| format!("http://localhost:{}", port));

    info!("🚀 NullBlock Engrams Service starting...");
    info!("📡 Server will bind to: {}", addr);
    info!("🏥 Health check: {}/health", engrams_base_url);
    info!("🧠 Engrams API: {}/engrams", engrams_base_url);

    // Start the server
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("✅ Server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

fn create_router(state: server::AppState) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health::health_check))
        // Engram CRUD endpoints
        .route("/engrams", post(engrams::create_engram))
        .route("/engrams", get(engrams::list_engrams))
        .route("/engrams/:id", get(engrams::get_engram))
        .route("/engrams/:id", put(engrams::update_engram))
        .route("/engrams/:id", delete(engrams::delete_engram))
        // Wallet-scoped endpoints
        .route(
            "/engrams/wallet/:wallet",
            get(engrams::get_engrams_by_wallet),
        )
        .route(
            "/engrams/wallet/:wallet/:key",
            get(engrams::get_engram_by_wallet_key),
        )
        // Search and operations
        .route("/engrams/search", post(engrams::search_engrams))
        .route("/engrams/:id/fork", post(engrams::fork_engram))
        .route("/engrams/:id/publish", post(engrams::publish_engram))
        // Add state
        .with_state(state)
        // Add middleware
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any),
        )
        .layer(TraceLayer::new_for_http())
}
