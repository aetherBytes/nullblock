use std::env;
use std::net::SocketAddr;

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, warn};

mod agents;
mod config;
mod database;
mod engrams;
mod error;
mod handlers;
mod kafka;
mod llm;
mod logging;
mod mcp;
mod models;
mod server;
mod services;
mod utils;

use crate::config::Config;
use crate::handlers::{health, hecate, llm_proxy, moros, siren_marketing, tasks, user_references};
use crate::logging::setup_logging;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment from .env.dev if available
    if let Err(e) = dotenv::from_filename(".env.dev") {
        warn!("⚠️ Could not load .env.dev file: {}", e);
        warn!("   Expected location: .env.dev in current directory");
        warn!("   Without .env.dev, OpenRouter API key won't be loaded!");
        warn!("   You may experience rate limiting with anonymous access.");
    } else {
        println!("✅ Loaded configuration from .env.dev");
    }

    // Setup logging
    setup_logging()?;

    // Load configuration
    let config = Config::from_env()?;
    info!("🔧 Configuration loaded: {}", config.service_name);

    // Create the application state
    let state = server::AppState::new(config.clone()).await?;

    // Build the router
    let app = create_router(state);

    // Get port from config or environment
    let port = env::var("AGENTS_PORT")
        .unwrap_or_else(|_| config.server.port.to_string())
        .parse::<u16>()
        .unwrap_or(9001);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let agents_base_url =
        env::var("AGENTS_SERVICE_URL").unwrap_or_else(|_| format!("http://localhost:{}", port));

    info!("🚀 NullBlock Agents Rust Service starting...");
    info!("📡 Server will bind to: {}", addr);
    info!("🏥 Health check: {}/health", agents_base_url);
    info!("🤖 Hecate agent: {}/hecate", agents_base_url);
    info!("🌑 Moros agent: {}/moros", agents_base_url);
    info!("📱 Siren agent: {}/siren", agents_base_url);
    info!("📚 API docs: {}/docs (future)", agents_base_url);

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
        // Hecate agent endpoints
        .route("/hecate/chat", post(hecate::chat))
        .route("/hecate/health", get(hecate::health))
        .route("/hecate/model-status", get(hecate::model_status))
        .route("/hecate/available-models", get(hecate::available_models))
        .route("/hecate/search-models", get(hecate::search_models))
        .route("/hecate/set-model", post(hecate::set_model))
        .route("/hecate/refresh-models", post(hecate::refresh_models))
        .route("/hecate/reset-models", post(hecate::reset_models))
        .route("/hecate/personality", post(hecate::set_personality))
        .route("/hecate/clear", post(hecate::clear_conversation))
        .route("/hecate/history", get(hecate::get_history))
        .route("/hecate/model-info", get(hecate::get_model_info))
        .route("/hecate/tools", get(hecate::get_tools))
        // Hecate session endpoints
        .route("/hecate/sessions", get(hecate::list_sessions))
        .route("/hecate/sessions/new", post(hecate::create_session))
        .route("/hecate/sessions/:session_id", get(hecate::get_session))
        .route(
            "/hecate/sessions/:session_id",
            delete(hecate::delete_session),
        )
        .route(
            "/hecate/sessions/:session_id/resume",
            post(hecate::resume_session),
        )
        // Moros agent endpoints
        .route("/moros/chat", post(moros::chat))
        .route("/moros/health", get(moros::health))
        .route("/moros/model-status", get(moros::model_status))
        .route("/moros/available-models", get(moros::available_models))
        .route("/moros/set-model", post(moros::set_model))
        .route("/moros/clear", post(moros::clear_conversation))
        .route("/moros/history", get(moros::get_history))
        .route("/moros/tools", get(moros::get_tools))
        // Task management endpoints
        .route("/tasks", post(tasks::create_task_handler))
        .route("/tasks", get(tasks::get_tasks_handler))
        .route("/tasks/:task_id", get(tasks::get_task))
        .route("/tasks/:task_id", put(tasks::update_task))
        .route("/tasks/:task_id", delete(tasks::delete_task))
        .route("/tasks/:task_id/start", post(tasks::start_task))
        .route("/tasks/:task_id/pause", post(tasks::pause_task))
        .route("/tasks/:task_id/resume", post(tasks::resume_task))
        .route("/tasks/:task_id/cancel", post(tasks::cancel_task))
        .route("/tasks/:task_id/retry", post(tasks::retry_task))
        .route("/tasks/:task_id/process", post(tasks::process_task))
        // Siren Marketing agent endpoints
        .route("/siren/chat", post(siren_marketing::chat))
        .route(
            "/siren/generate-content",
            post(siren_marketing::generate_content),
        )
        .route(
            "/siren/create-post",
            post(siren_marketing::create_twitter_post),
        )
        .route(
            "/siren/analyze-project",
            get(siren_marketing::analyze_project_progress),
        )
        .route("/siren/health", get(siren_marketing::get_siren_health))
        .route("/siren/model-status", get(siren_marketing::model_status))
        .route("/siren/themes", get(siren_marketing::get_content_themes))
        .route("/siren/set-model", post(siren_marketing::set_model))
        // OpenAI-compatible LLM proxy endpoints
        .route("/v1/chat/completions", post(llm_proxy::handle_chat_completions))
        .route("/v1/models", get(llm_proxy::handle_list_models))
        .route("/v1/set-model", post(llm_proxy::handle_set_model_preference))
        .route("/v1/model-preference/:agent_name", get(llm_proxy::handle_get_model_preference))
        // MCP JSON-RPC endpoint
        .route("/mcp/jsonrpc", post(mcp::jsonrpc::handle_jsonrpc))
        // User reference endpoints
        .route(
            "/user-references",
            post(user_references::create_user_reference),
        )
        .route(
            "/user-references",
            get(user_references::list_user_references),
        )
        .route(
            "/user-references/:wallet_address/:chain",
            get(user_references::get_user_reference),
        )
        .route(
            "/user-references/sync",
            post(user_references::sync_user_reference),
        )
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
