use axum::{extract::State, http::StatusCode, response::Json};
use serde::Serialize;
use std::sync::Arc;

use crate::models::ContentTemplate;
use crate::repository::ContentRepository;

use super::generate::AppState;

pub async fn list_templates(
    State(state): State<AppState>,
) -> Result<Json<Vec<ContentTemplate>>, (StatusCode, Json<ErrorResponse>)> {
    let pool = Arc::new(state.db.pool().clone());
    let repo = ContentRepository::new(pool);

    let templates = repo.list_templates(true).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    Ok(Json(templates))
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}
