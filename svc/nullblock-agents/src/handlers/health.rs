use crate::{models::HealthResponse, server::AppState};
use axum::{extract::State, Json};
use chrono::Utc;

pub async fn health_check(State(_state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        service: "nullblock-agents".to_string(),
        version: "0.1.0".to_string(),
        timestamp: Utc::now().to_rfc3339(),
        components: None,
    })
}