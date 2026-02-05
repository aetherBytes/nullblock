use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::models::{ContentQueue, ContentQueueResponse};
use crate::repository::ContentRepository;

use super::generate::AppState;

#[derive(Deserialize)]
pub struct QueueQuery {
    pub status: Option<String>,
    pub theme: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_queue(
    State(state): State<AppState>,
    Query(query): Query<QueueQuery>,
) -> Result<Json<ContentQueueResponse>, (StatusCode, Json<ErrorResponse>)> {
    let pool = Arc::new(state.db.pool().clone());
    let repo = ContentRepository::new(pool);
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let items = if let Some(status) = query.status.as_deref() {
        if status == "pending" {
            repo.list_pending(limit, offset).await
        } else if status == "posted" {
            repo.list_posted(limit, offset).await
        } else {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Invalid status: {}", status),
                }),
            ));
        }
    } else if let Some(theme) = query.theme.as_deref() {
        repo.list_by_theme(theme, limit, offset).await
    } else {
        repo.list_pending(limit, offset).await
    }
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    Ok(Json(ContentQueueResponse {
        total: items.len() as i64,
        items,
    }))
}

pub async fn get_content(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ContentQueue>, (StatusCode, Json<ErrorResponse>)> {
    let pool = Arc::new(state.db.pool().clone());
    let repo = ContentRepository::new(pool);

    let content = repo
        .get_content(id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Content not found".to_string(),
                }),
            )
        })?;

    Ok(Json(content))
}

pub async fn update_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateStatusRequest>,
) -> Result<Json<ContentQueue>, (StatusCode, Json<ErrorResponse>)> {
    let pool = Arc::new(state.db.pool().clone());
    let repo = ContentRepository::new(pool);

    let content = repo
        .update_status(id, &payload.status)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    Ok(Json(content))
}

pub async fn delete_content(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let pool = Arc::new(state.db.pool().clone());
    let repo = ContentRepository::new(pool);

    repo.delete_content(id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct UpdateStatusRequest {
    pub status: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}
