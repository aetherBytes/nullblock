use axum::{extract::{Path, State}, http::StatusCode, response::Json};
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::ContentMetrics;
use crate::repository::ContentRepository;

use super::generate::AppState;

pub async fn get_metrics(
    State(state): State<AppState>,
    Path(content_id): Path<Uuid>,
) -> Result<Json<ContentMetrics>, (StatusCode, Json<ErrorResponse>)> {
    let pool = Arc::new(state.db.pool().clone());
    let repo = ContentRepository::new(pool);

    let metrics = repo
        .get_metrics(content_id)
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
                    error: "Metrics not found".to_string(),
                }),
            )
        })?;

    Ok(Json(metrics))
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}
