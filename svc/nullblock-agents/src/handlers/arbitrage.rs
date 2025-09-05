use crate::{models::HealthResponse, server::AppState};
use axum::{extract::State, Json};
use chrono::Utc;
use serde_json::Value;

pub async fn arbitrage_health(State(_state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        service: "arbitrage".to_string(),
        version: "0.1.0".to_string(),
        timestamp: Utc::now().to_rfc3339(),
        components: None,
    })
}

pub async fn get_opportunities(State(_state): State<AppState>) -> Json<Value> {
    Json(serde_json::json!({
        "opportunities": [],
        "status": "no_opportunities",
        "timestamp": Utc::now().to_rfc3339()
    }))
}

pub async fn get_summary(State(_state): State<AppState>) -> Json<Value> {
    Json(serde_json::json!({
        "summary": {
            "total_opportunities": 0,
            "potential_profit": 0.0,
            "status": "monitoring"
        },
        "timestamp": Utc::now().to_rfc3339()
    }))
}

pub async fn execute(State(_state): State<AppState>, Json(_payload): Json<Value>) -> Json<Value> {
    Json(serde_json::json!({
        "result": "arbitrage_not_implemented",
        "message": "Arbitrage execution is not yet implemented",
        "timestamp": Utc::now().to_rfc3339()
    }))
}