use axum::{routing::get, Router};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::info;

use crate::health::health_check;
use crate::protocols::a2a::routes::create_a2a_routes;
use crate::protocols::a2a::sse::KafkaSSEBridge;
use crate::protocols::mcp::routes::create_mcp_routes;

#[derive(Clone)]
pub struct AppState {
    pub http_client: reqwest::Client,
    pub agents_service_url: String,
    pub erebus_base_url: String,
    pub arbfarm_url: String,
    pub kafka_bridge: Option<Arc<KafkaSSEBridge>>,
}

pub struct Server {
    app: Router,
}

impl Server {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let agents_service_url = std::env::var("AGENTS_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:9003".to_string());
        let erebus_base_url = std::env::var("EREBUS_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:3000".to_string());
        let arbfarm_url = std::env::var("ARBFARM_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:9007".to_string());

        info!("🔗 Agents Service URL: {}", agents_service_url);
        info!("🔗 Erebus Base URL: {}", erebus_base_url);
        info!("🔗 ArbFarm Service URL: {}", arbfarm_url);

        let kafka_bridge = if let Ok(bootstrap_servers) = std::env::var("KAFKA_BOOTSTRAP_SERVERS") {
            match KafkaSSEBridge::new(&bootstrap_servers) {
                Ok(bridge) => {
                    bridge.start_forwarding().await;
                    info!("✅ Kafka SSE bridge initialized for task streaming");
                    Some(Arc::new(bridge))
                }
                Err(e) => {
                    tracing::warn!("⚠️ Failed to initialize Kafka SSE bridge: {}", e);
                    None
                }
            }
        } else {
            tracing::warn!("⚠️ KAFKA_BOOTSTRAP_SERVERS not set, SSE streaming disabled");
            None
        };

        let state = AppState {
            http_client: reqwest::Client::new(),
            agents_service_url,
            erebus_base_url,
            arbfarm_url,
            kafka_bridge,
        };

        let a2a_router = create_a2a_routes(state.clone());
        let v1_router = create_a2a_routes(state.clone());
        let mcp_router = create_mcp_routes(state.clone());

        let app = Router::new()
            .nest("/a2a", a2a_router)
            .nest("/v1", v1_router)
            .nest("/mcp", mcp_router)
            .route("/health", get(health_check))
            .layer(CorsLayer::permissive());

        Ok(Self { app })
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let port = std::env::var("PORT")
            .unwrap_or_else(|_| "8001".to_string())
            .parse::<u16>()
            .unwrap_or(8001);

        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        let protocols_base_url = std::env::var("PROTOCOLS_SERVICE_URL")
            .unwrap_or_else(|_| format!("http://localhost:{}", port));

        info!("🚀 NullBlock Protocols Server starting on {}", addr);
        info!("📋 Health endpoint: {}/health", protocols_base_url);
        info!(
            "🔗 A2A JSON-RPC endpoint: {}/a2a/jsonrpc",
            protocols_base_url
        );
        info!("🌐 A2A REST endpoints: {}/v1/*", protocols_base_url);
        info!("📄 Agent Card: {}/v1/card", protocols_base_url);
        info!(
            "📨 Messages: POST {}/v1/messages, {}/v1/messages/stream",
            protocols_base_url, protocols_base_url
        );
        info!(
            "📋 Tasks: GET {}/v1/tasks, {}/v1/tasks/:id, POST {}/v1/tasks/:id/cancel",
            protocols_base_url, protocols_base_url, protocols_base_url
        );
        info!(
            "🔌 MCP JSON-RPC endpoint: {}/mcp/jsonrpc",
            protocols_base_url
        );
        info!("🧠 MCP Protocol Version: 2025-11-25",);

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, self.app).await?;

        Ok(())
    }
}
