use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::models::{
    ApproveRequest, ExecutionToggleRequest, HecateRecommendation,
    RejectRequest, UpdateExecutionConfigRequest,
};
use crate::server::AppState;

pub async fn list_approvals(State(state): State<AppState>) -> impl IntoResponse {
    let approvals = state.approval_manager.list_all().await;
    (StatusCode::OK, Json(serde_json::json!({
        "approvals": approvals,
        "total": approvals.len()
    })))
}

pub async fn list_pending_approvals(State(state): State<AppState>) -> impl IntoResponse {
    let approvals = state.approval_manager.list_pending().await;
    (StatusCode::OK, Json(serde_json::json!({
        "approvals": approvals,
        "total": approvals.len()
    })))
}

pub async fn get_approval(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.approval_manager.get_approval(id).await {
        Some(approval) => (StatusCode::OK, Json(approval)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Approval not found"})),
        )
            .into_response(),
    }
}

pub async fn approve_approval(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<ApproveRequest>,
) -> impl IntoResponse {
    match state.approval_manager.approve(id, request.notes).await {
        Ok(approval) => (StatusCode::OK, Json(approval)).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn reject_approval(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<RejectRequest>,
) -> impl IntoResponse {
    match state.approval_manager.reject(id, request.reason).await {
        Ok(approval) => (StatusCode::OK, Json(approval)).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn get_execution_config(State(state): State<AppState>) -> impl IntoResponse {
    let config = state.approval_manager.get_config().await;
    (StatusCode::OK, Json(config))
}

pub async fn update_execution_config(
    State(state): State<AppState>,
    Json(request): Json<UpdateExecutionConfigRequest>,
) -> impl IntoResponse {
    let config = state.approval_manager.update_config(request).await;
    (StatusCode::OK, Json(config))
}

pub async fn toggle_execution(
    State(state): State<AppState>,
    Json(request): Json<ExecutionToggleRequest>,
) -> impl IntoResponse {
    let config = state.approval_manager.toggle_execution(request.enabled).await;
    (StatusCode::OK, Json(serde_json::json!({
        "enabled": config.auto_execution_enabled,
        "message": if config.auto_execution_enabled {
            "Auto-execution enabled"
        } else {
            "Auto-execution disabled"
        }
    })))
}

pub async fn add_hecate_recommendation(
    State(state): State<AppState>,
    Json(recommendation): Json<HecateRecommendation>,
) -> impl IntoResponse {
    match state.approval_manager.add_hecate_recommendation(recommendation).await {
        Ok(approval) => (StatusCode::OK, Json(approval)).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn cleanup_expired(State(state): State<AppState>) -> impl IntoResponse {
    let expired = state.approval_manager.cleanup_expired().await;
    (StatusCode::OK, Json(serde_json::json!({
        "expired_count": expired.len(),
        "expired_ids": expired
    })))
}
