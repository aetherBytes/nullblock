use axum::{extract::State, Json};
use serde_json::{json, Value};

use crate::server::AppState;

pub async fn health_check(State(state): State<AppState>) -> Json<Value> {
    // Test database connection
    let db_status = match sqlx::query("SELECT 1").fetch_one(&state.db_pool).await {
        Ok(_) => "healthy",
        Err(_) => "unhealthy",
    };

    Json(json!({
        "status": "healthy",
        "service": "nullblock-engrams",
        "version": "0.1.0",
        "database": db_status,
    }))
}
