use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::server::AppState;

pub async fn get_autonomous_executor_stats(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let stats = state.autonomous_executor.get_stats().await;

    (StatusCode::OK, Json(serde_json::json!({
        "executions_attempted": stats.executions_attempted,
        "executions_succeeded": stats.executions_succeeded,
        "executions_failed": stats.executions_failed,
        "total_sol_deployed": stats.total_sol_deployed,
        "is_running": stats.is_running,
    })))
}

pub async fn list_autonomous_executions(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let executions = state.autonomous_executor.list_executions().await;

    (StatusCode::OK, Json(serde_json::json!({
        "executions": executions,
        "count": executions.len(),
    })))
}

pub async fn start_autonomous_executor(
    State(state): State<AppState>,
) -> impl IntoResponse {
    state.autonomous_executor.start().await;

    (StatusCode::OK, Json(serde_json::json!({
        "success": true,
        "message": "Autonomous executor started",
    })))
}

pub async fn stop_autonomous_executor(
    State(state): State<AppState>,
) -> impl IntoResponse {
    state.autonomous_executor.stop().await;

    (StatusCode::OK, Json(serde_json::json!({
        "success": true,
        "message": "Autonomous executor stopped",
    })))
}
