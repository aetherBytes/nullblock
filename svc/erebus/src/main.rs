use axum::{
    extract::{Path, Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Json, Response},
    routing::{delete, get, post, put},
    Router,
};
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info, warn};
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::{fmt, prelude::*, EnvFilter, Layer};

fn load_env() {
    // Try .env.dev first (development), then .env (production)
    if dotenvy::from_filename(".env.dev").is_ok() {
        println!("📁 Loaded environment from .env.dev");
    } else if dotenvy::dotenv().is_ok() {
        println!("📁 Loaded environment from .env");
    } else {
        println!("⚠️ No .env file found, using system environment variables");
    }
}

// Import our modules
mod auth;
mod crypto;
mod database;
mod resources;
mod user_references;
mod utils;
use resources::agents::routes::{
    agent_chat,
    agent_health,
    agent_status,
    cancel_task,
    // Task management routes
    create_task,
    create_task_from_template,
    delete_task,
    get_motivation_state,
    get_task,
    get_task_events,
    get_task_notifications,
    get_task_queues,
    get_task_stats,
    get_task_suggestions,
    get_task_templates,
    get_tasks,
    handle_notification_action,
    hecate_available_models,
    hecate_chat,
    hecate_clear,
    hecate_history,
    hecate_model_info,
    hecate_personality,
    hecate_search_models,
    hecate_set_model,
    hecate_status,
    hecate_tools,
    learn_from_task,
    mark_notification_read,
    pause_task,
    process_task,
    publish_task_event,
    // User management routes
    register_user,
    resume_task,
    retry_task,
    siren_chat,
    siren_set_model,
    start_task,
    update_motivation_state,
    update_task,
};
use resources::users::routes::{create_user_endpoint, get_user_endpoint, lookup_user_endpoint};
use resources::wallets::routes::create_wallet_routes;
use resources::{
    create_arb_routes, create_crossroads_routes, create_discovery_routes, create_engram_routes,
    create_mcp_routes, ExternalService, WalletManager,
};

#[derive(Serialize)]
struct StatusResponse {
    status: String,
    service: String,
    version: String,
    message: String,
}

#[derive(Clone)]
struct AppState {
    wallet_manager: WalletManager,
    external_service: Arc<ExternalService>,
    database: Arc<database::Database>,
}

async fn health_check(State(app_state): State<AppState>) -> Json<StatusResponse> {
    // Check database connection
    let database_status = match app_state.database.health_check().await {
        Ok(_) => "healthy",
        Err(_) => "unhealthy",
    };

    let overall_status = if database_status == "healthy" {
        "healthy"
    } else {
        "unhealthy"
    };
    let message = format!(
        "🎯 Ready for agentic workflows and MCP integration (DB: {})",
        database_status
    );

    Json(StatusResponse {
        status: overall_status.to_string(),
        service: "erebus".to_string(),
        version: "0.1.0".to_string(),
        message,
    })
}

async fn root() -> Json<StatusResponse> {
    let response = StatusResponse {
        status: "running".to_string(),
        service: "erebus".to_string(),
        version: "0.1.0".to_string(),
        message: "💡 Erebus - Nullblock Wallet & MCP Server".to_string(),
    };

    info!(
        "📤 Root endpoint response: {}",
        serde_json::to_string_pretty(&response).unwrap_or_default()
    );
    Json(response)
}

async fn proxy_task_sse(Path(task_id): Path<String>) -> Result<Response, StatusCode> {
    let protocols_url = std::env::var("PROTOCOLS_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:8001".to_string());

    let url = format!("{}/a2a/tasks/{}/sse", protocols_url, task_id);

    info!("🔌 Proxying SSE request to: {}", url);

    let client = reqwest::Client::new();
    match client.get(&url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                let stream = response.bytes_stream();
                Ok((
                    [(axum::http::header::CONTENT_TYPE, "text/event-stream")],
                    axum::body::Body::from_stream(stream),
                )
                    .into_response())
            } else {
                error!("❌ SSE proxy failed: {}", response.status());
                Err(StatusCode::BAD_GATEWAY)
            }
        }
        Err(e) => {
            error!("❌ SSE proxy connection failed: {}", e);
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}

async fn proxy_message_sse() -> Result<Response, StatusCode> {
    let protocols_url = std::env::var("PROTOCOLS_SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:8001".to_string());

    let url = format!("{}/a2a/messages/sse", protocols_url);

    info!("🔌 Proxying message SSE request to: {}", url);

    let client = reqwest::Client::new();
    match client.get(&url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                let stream = response.bytes_stream();
                Ok((
                    [(axum::http::header::CONTENT_TYPE, "text/event-stream")],
                    axum::body::Body::from_stream(stream),
                )
                    .into_response())
            } else {
                error!("❌ Message SSE proxy failed: {}", response.status());
                Err(StatusCode::BAD_GATEWAY)
            }
        }
        Err(e) => {
            error!("❌ Message SSE proxy connection failed: {}", e);
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}

/// Comprehensive logging middleware for all requests and responses
async fn logging_middleware(
    request: Request,
    next: Next,
) -> Result<axum::response::Response, StatusCode> {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let headers = request.headers().clone();
    let user_agent = headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");
    let remote_addr = headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    // Generate unique request ID for tracing
    let request_id = uuid::Uuid::new_v4().to_string();

    // Extract request body for logging
    let (parts, body) = request.into_parts();
    let body_bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("❌ [{}] Failed to read request body: {}", request_id, e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    // Log structured request information
    info!("🔄 [{}] REQUEST START", request_id);
    info!(
        "📥 [{}] {} {} from {}",
        request_id, method, uri, remote_addr
    );
    info!("🌐 [{}] User-Agent: {}", request_id, user_agent);
    info!("📋 [{}] Headers: {:#?}", request_id, headers);

    // Log request body with size limit for security (sanitize API keys)
    if !body_bytes.is_empty() {
        let body_size = body_bytes.len();
        if body_size > 10_000 {
            info!(
                "📝 [{}] Request body: {} bytes (truncated for security)",
                request_id, body_size
            );
        } else {
            match serde_json::from_slice::<serde_json::Value>(&body_bytes) {
                Ok(mut json) => {
                    // Sanitize sensitive fields before logging
                    if let Some(obj) = json.as_object_mut() {
                        if obj.contains_key("api_key") {
                            obj.insert(
                                "api_key".to_string(),
                                serde_json::Value::String("***REDACTED***".to_string()),
                            );
                        }
                        if obj.contains_key("encrypted_key") {
                            obj.insert(
                                "encrypted_key".to_string(),
                                serde_json::Value::String("***REDACTED***".to_string()),
                            );
                        }
                    }
                    info!(
                        "📝 [{}] Request body (JSON): {}",
                        request_id,
                        serde_json::to_string_pretty(&json).unwrap_or_default()
                    );
                }
                Err(_) => {
                    match String::from_utf8(body_bytes.to_vec()) {
                        Ok(text) => {
                            if text.contains("api_key") || text.contains("encrypted_key") {
                                info!("📝 [{}] Request body: ***CONTAINS SENSITIVE DATA - REDACTED***", request_id);
                            } else {
                                info!("📝 [{}] Request body (Text): {}", request_id, text);
                            }
                        }
                        Err(_) => info!(
                            "📝 [{}] Request body: {} bytes (binary)",
                            request_id, body_size
                        ),
                    }
                }
            }
        }
    } else {
        info!("📝 [{}] Request body: empty", request_id);
    }

    // Rebuild request
    let request = Request::from_parts(parts, axum::body::Body::from(body_bytes));

    // Process request and capture timing
    let start_time = std::time::Instant::now();
    let response = next.run(request).await;
    let duration = start_time.elapsed();

    // Capture response details
    let status = response.status();
    let response_headers = response.headers().clone();

    // Log response information
    info!(
        "📤 [{}] RESPONSE: {} {} -> {} ({:.2}ms)",
        request_id,
        method,
        uri,
        status,
        duration.as_millis()
    );
    info!(
        "📋 [{}] Response headers: {:#?}",
        request_id, response_headers
    );

    let is_streaming = response_headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|ct| ct.contains("text/event-stream"))
        .unwrap_or(false);

    if is_streaming {
        info!(
            "✅ [{}] STREAMING RESPONSE (SSE) - passing through",
            request_id
        );
        return Ok(response);
    }

    let (parts, body) = response.into_parts();
    let response_body = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(bytes) => {
            if !bytes.is_empty() {
                let body_size = bytes.len();
                if body_size > 10_000 {
                    info!(
                        "📄 [{}] Response body: {} bytes (truncated for log size)",
                        request_id, body_size
                    );
                } else {
                    match serde_json::from_slice::<serde_json::Value>(&bytes) {
                        Ok(mut json) => {
                            if let Some(obj) = json.as_object_mut() {
                                if let Some(data) = obj.get_mut("data") {
                                    if let Some(arr) = data.as_array_mut() {
                                        for item in arr.iter_mut() {
                                            if let Some(item_obj) = item.as_object_mut() {
                                                if item_obj.contains_key("api_key") {
                                                    item_obj.insert(
                                                        "api_key".to_string(),
                                                        serde_json::Value::String(
                                                            "***REDACTED***".to_string(),
                                                        ),
                                                    );
                                                }
                                            }
                                        }
                                    } else if let Some(data_obj) = data.as_object_mut() {
                                        if data_obj.contains_key("api_key") {
                                            data_obj.insert(
                                                "api_key".to_string(),
                                                serde_json::Value::String(
                                                    "***REDACTED***".to_string(),
                                                ),
                                            );
                                        }
                                    }
                                }
                            }
                            info!(
                                "📄 [{}] Response body (JSON): {}",
                                request_id,
                                serde_json::to_string_pretty(&json).unwrap_or_default()
                            );
                        }
                        Err(_) => match String::from_utf8(bytes.to_vec()) {
                            Ok(text) => {
                                if text.contains("api_key") {
                                    info!("📄 [{}] Response body: ***CONTAINS SENSITIVE DATA - REDACTED***", request_id);
                                } else {
                                    info!("📄 [{}] Response body (Text): {}", request_id, text);
                                }
                            }
                            Err(_) => info!(
                                "📄 [{}] Response body: {} bytes (binary)",
                                request_id, body_size
                            ),
                        },
                    }
                }
            } else {
                info!("📄 [{}] Response body: empty", request_id);
            }
            bytes
        }
        Err(e) => {
            error!("❌ [{}] Failed to read response body: {}", request_id, e);
            axum::body::Bytes::new()
        }
    };

    if status.is_success() {
        info!("✅ [{}] REQUEST COMPLETED SUCCESSFULLY", request_id);
    } else if status.is_client_error() {
        warn!("⚠️ [{}] CLIENT ERROR: {}", request_id, status);
    } else if status.is_server_error() {
        error!("💥 [{}] SERVER ERROR: {}", request_id, status);
    }

    let response =
        axum::response::Response::from_parts(parts, axum::body::Body::from(response_body));

    Ok(response)
}

fn setup_logging() -> tracing_appender::non_blocking::WorkerGuard {
    let log_level = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    let filter = EnvFilter::new(&log_level);

    // Create logs directory if it doesn't exist
    std::fs::create_dir_all("logs").expect("Failed to create logs directory");

    // Set up file appender with daily rotation
    let file_appender = rolling::daily("logs", "erebus.log");
    let (non_blocking, guard) = non_blocking(file_appender);

    // Create file layer
    let file_layer = fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_filter(EnvFilter::new(&log_level));

    // Create console layer
    let console_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_ansi(true)
        .with_filter(filter);

    tracing_subscriber::registry()
        .with(file_layer)
        .with(console_layer)
        .init();

    guard
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env.dev or .env first
    load_env();

    let _guard = setup_logging();

    info!("🚀 Starting Erebus server...");
    info!("📍 Version: 0.1.0");
    info!(
        "🕐 Timestamp: {}",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
    );
    info!("============================================================");

    // Initialize database connection
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://postgres:REDACTED_DB_PASS@localhost:5440/erebus".to_string()
    });

    info!("🗄️ Initializing database connection...");
    let database = match database::Database::new(&database_url).await {
        Ok(db) => {
            info!("✅ Database connection established successfully");
            Arc::new(db)
        }
        Err(e) => {
            error!("❌ Failed to connect to database: {}", e);
            return Err(Box::new(e) as Box<dyn std::error::Error>);
        }
    };

    // Test database connection
    if let Err(e) = database.health_check().await {
        error!("❌ Database health check failed: {}", e);
        return Err(Box::new(e) as Box<dyn std::error::Error>);
    }
    info!("✅ Database health check passed");

    // Initialize encryption service
    let encryption_key = std::env::var("ENCRYPTION_MASTER_KEY")
        .expect("ENCRYPTION_MASTER_KEY must be set in environment");

    info!("🔐 Initializing encryption service...");
    let encryption_service = match crypto::EncryptionService::new(&encryption_key) {
        Ok(service) => {
            info!("✅ Encryption service initialized successfully");
            Arc::new(service)
        }
        Err(e) => {
            error!("❌ Failed to initialize encryption service: {}", e);
            return Err(format!("Encryption initialization failed: {}", e).into());
        }
    };

    // Create API key service
    let api_key_service = Arc::new(resources::api_keys::ApiKeyService::new(
        Arc::new(database.pool().clone()),
        encryption_service,
    ));
    info!("✅ API key service initialized");

    // Create wallet manager
    let wallet_manager = WalletManager::new();

    // Create external service
    let external_service = Arc::new(ExternalService::new());

    // Create app state
    let app_state = AppState {
        wallet_manager,
        external_service,
        database,
    };

    // Create router with CORS, agent routes, wallet routes, and crossroads routes
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        // Agent routing endpoints
        .route("/api/agents/health", get(agent_health))
        .route("/api/agents/hecate/chat", post(hecate_chat))
        .route("/api/agents/siren/chat", post(siren_chat))
        .route("/api/agents/siren/set-model", post(siren_set_model))
        .route("/api/agents/hecate/status", get(hecate_status))
        .route("/api/agents/hecate/personality", post(hecate_personality))
        .route("/api/agents/hecate/clear", post(hecate_clear))
        .route("/api/agents/hecate/history", get(hecate_history))
        .route(
            "/api/agents/hecate/available-models",
            get(hecate_available_models),
        )
        .route("/api/agents/hecate/set-model", post(hecate_set_model))
        .route("/api/agents/hecate/model-info", get(hecate_model_info))
        .route(
            "/api/agents/hecate/search-models",
            get(hecate_search_models),
        )
        .route("/api/agents/hecate/tools", get(hecate_tools))
        .route("/api/agents/:agent_name/chat", post(agent_chat))
        .route("/api/agents/:agent_name/status", get(agent_status))
        // Task management endpoints
        .route("/api/agents/tasks", post(create_task))
        .route("/api/agents/tasks", get(get_tasks))
        .route("/api/agents/tasks/:task_id", get(get_task))
        .route("/api/agents/tasks/:task_id", put(update_task))
        .route("/api/agents/tasks/:task_id", delete(delete_task))
        .route("/api/agents/tasks/:task_id/start", post(start_task))
        .route("/api/agents/tasks/:task_id/pause", post(pause_task))
        .route("/api/agents/tasks/:task_id/resume", post(resume_task))
        .route("/api/agents/tasks/:task_id/cancel", post(cancel_task))
        .route("/api/agents/tasks/:task_id/retry", post(retry_task))
        .route("/api/agents/tasks/queues", get(get_task_queues))
        .route("/api/agents/tasks/templates", get(get_task_templates))
        .route(
            "/api/agents/tasks/from-template",
            post(create_task_from_template),
        )
        .route("/api/agents/tasks/stats", get(get_task_stats))
        .route(
            "/api/agents/tasks/notifications",
            get(get_task_notifications),
        )
        .route(
            "/api/agents/tasks/notifications/:notification_id/read",
            post(mark_notification_read),
        )
        .route(
            "/api/agents/tasks/notifications/:notification_id/action",
            post(handle_notification_action),
        )
        .route("/api/agents/tasks/events", get(get_task_events))
        .route("/api/agents/tasks/events", post(publish_task_event))
        .route("/api/agents/tasks/motivation", get(get_motivation_state))
        .route("/api/agents/tasks/motivation", put(update_motivation_state))
        .route("/api/agents/tasks/suggestions", post(get_task_suggestions))
        .route("/api/agents/tasks/:task_id/learn", post(learn_from_task))
        .route("/api/agents/tasks/:task_id/process", post(process_task))
        // A2A SSE streaming endpoints
        .route("/a2a/tasks/:task_id/sse", get(proxy_task_sse))
        .route("/a2a/messages/sse", get(proxy_message_sse))
        // User management endpoints
        .route("/api/users/register", post(create_user_endpoint))
        .route("/api/users/lookup", post(lookup_user_endpoint))
        .route("/api/users/:user_id", get(get_user_endpoint))
        // Legacy agent user registration (deprecated - use /api/users/register)
        .route("/api/agents/users/register", post(register_user))
        // Logging endpoints
        .route(
            "/api/logs/recent",
            get(resources::logs::routes::get_recent_logs),
        )
        .route(
            "/api/logs/stream",
            get(resources::logs::routes::stream_logs),
        )
        // Merge wallet routes
        .merge(create_wallet_routes())
        // Merge crossroads routes
        .merge(create_crossroads_routes(&app_state.external_service))
        // Merge engram routes
        .merge(create_engram_routes())
        // Merge MCP proxy routes
        .merge(create_mcp_routes())
        // Merge API key routes
        .merge(resources::api_keys::create_api_key_routes(
            api_key_service.clone(),
        ))
        // Merge ArbFarm routes
        .merge(create_arb_routes())
        // Merge Discovery routes (federated MCP/agent/protocol discovery)
        .merge(create_discovery_routes())
        .with_state(app_state.clone())
        // Add logging middleware
        .layer(middleware::from_fn(logging_middleware))
        // Add CORS layer
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    // Get configurable base URL for this service
    let erebus_base_url =
        std::env::var("EREBUS_BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

    info!("🌐 Server starting on http://0.0.0.0:3000");
    info!("🏥 Health check: {}/health", erebus_base_url);
    info!("🤖 Agent routing: {}/api/agents/health", erebus_base_url);
    info!("💬 Hecate chat: {}/api/agents/hecate/chat", erebus_base_url);
    info!("🎭 Siren chat: {}/api/agents/siren/chat", erebus_base_url);
    info!(
        "📊 Hecate status: {}/api/agents/hecate/status",
        erebus_base_url
    );
    info!(
        "⚙️ Hecate personality: {}/api/agents/hecate/personality",
        erebus_base_url
    );
    info!(
        "🧹 Hecate clear: {}/api/agents/hecate/clear",
        erebus_base_url
    );
    info!(
        "📜 Hecate history: {}/api/agents/hecate/history",
        erebus_base_url
    );
    info!(
        "🧠 Hecate available models: {}/api/agents/hecate/available-models",
        erebus_base_url
    );
    info!(
        "🎯 Hecate set model: {}/api/agents/hecate/set-model",
        erebus_base_url
    );
    info!(
        "📋 Hecate model info: {}/api/agents/hecate/model-info",
        erebus_base_url
    );
    info!(
        "🔍 Hecate search models: {}/api/agents/hecate/search-models",
        erebus_base_url
    );
    info!("📋 Task management: {}/api/agents/tasks", erebus_base_url);
    info!(
        "⚡ Task events: {}/api/agents/tasks/events",
        erebus_base_url
    );
    info!(
        "🧠 Task motivation: {}/api/agents/tasks/motivation",
        erebus_base_url
    );
    info!(
        "💡 Task suggestions: {}/api/agents/tasks/suggestions",
        erebus_base_url
    );
    info!("👛 Wallet endpoints: {}/api/wallets", erebus_base_url);
    info!(
        "🔍 Wallet detection: {}/api/wallets/detect",
        erebus_base_url
    );
    info!(
        "🔐 Wallet challenge: {}/api/wallets/challenge",
        erebus_base_url
    );
    info!("✅ Wallet verify: {}/api/wallets/verify", erebus_base_url);
    info!(
        "🛣️  Crossroads marketplace: {}/api/marketplace",
        erebus_base_url
    );
    info!(
        "🔍 Discovery service: {}/api/discovery (federated)",
        erebus_base_url
    );
    info!(
        "🔧 Discovery tools: {}/api/discovery/tools",
        erebus_base_url
    );
    info!("🔥 Discovery hot: {}/api/discovery/hot", erebus_base_url);
    info!("⚙️  Admin panel: {}/api/admin", erebus_base_url);
    info!(
        "🏥 Crossroads health: {}/api/crossroads/health",
        erebus_base_url
    );
    info!(
        "🔐 API key management: {}/api/users/:user_id/api-keys",
        erebus_base_url
    );
    info!(
        "🔒 Internal API keys: {}/internal/users/:user_id/api-keys/decrypted",
        erebus_base_url
    );
    info!("🧠 Engram service: {}/api/engrams", erebus_base_url);
    info!("🏥 Engram health: {}/api/engrams/health", erebus_base_url);
    info!("🔌 MCP JSON-RPC: {}/mcp/jsonrpc", erebus_base_url);
    info!("🛠️ MCP tools: {}/mcp/tools", erebus_base_url);
    info!("📚 MCP resources: {}/mcp/resources", erebus_base_url);
    info!("💬 MCP prompts: {}/mcp/prompts", erebus_base_url);
    info!("🏥 MCP health: {}/mcp/health", erebus_base_url);
    info!("🎯 ArbFarm API: {}/api/arb", erebus_base_url);
    info!(
        "📊 ArbFarm Scanner: {}/api/arb/scanner/status",
        erebus_base_url
    );
    info!("💹 ArbFarm Edges: {}/api/arb/edges", erebus_base_url);
    info!(
        "🔍 ArbFarm Threats: {}/api/arb/threat/check/:mint",
        erebus_base_url
    );
    info!("💡 Ready for agentic workflows, marketplace operations, engrams, MCP, and service discovery");

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    info!("✅ Server listening on {}", addr);

    axum::serve(listener, app)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    Ok(())
}
