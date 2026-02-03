use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub timestamp: DateTime<Utc>,
    pub version: String,
    pub components: HealthComponents,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthComponents {
    pub a2a_protocol: String,
    pub mcp_protocol: String,
    pub server: String,
}

pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        service: "nullblock-protocols".to_string(),
        timestamp: Utc::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        components: HealthComponents {
            a2a_protocol: "ready".to_string(),
            mcp_protocol: "ready".to_string(),
            server: "running".to_string(),
        },
    })
}
