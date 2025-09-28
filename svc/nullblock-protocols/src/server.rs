use axum::{routing::get, Router};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing::info;

use crate::health::health_check;
use crate::protocols::a2a::routes::create_a2a_routes;

pub struct Server {
    app: Router,
}

impl Server {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let app = Router::new()
            .route("/health", get(health_check))
            .nest("/a2a", create_a2a_routes())
            .nest("/v1", create_a2a_routes()) // Also serve on /v1 for REST compatibility
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
        info!("🔗 A2A JSON-RPC endpoint: {}/a2a/jsonrpc", protocols_base_url);
        info!("🌐 A2A REST endpoints: {}/v1/*", protocols_base_url);
        info!("📄 Agent Card: {}/v1/card", protocols_base_url);
        info!("📨 Messages: POST {}/v1/messages, {}/v1/messages/stream", protocols_base_url, protocols_base_url);
        info!("📋 Tasks: GET {}/v1/tasks, {}/v1/tasks/:id, POST {}/v1/tasks/:id/cancel", protocols_base_url, protocols_base_url, protocols_base_url);

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, self.app).await?;

        Ok(())
    }
}