use axum::{extract::State, http::StatusCode, response::Json};
use chrono::Utc;
use serde::Serialize;
use std::sync::Arc;

use crate::database::Database;
use crate::events::{ContentEvent, EventPublisher};
use crate::generator::engine::ContentGenerator;
use crate::models::{CreateContentRequest, GenerateContentResponse};
use crate::repository::ContentRepository;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub generator: Arc<ContentGenerator>,
    pub event_publisher: Arc<dyn EventPublisher>,
}

pub async fn generate_content(
    State(state): State<AppState>,
    Json(payload): Json<CreateContentRequest>,
) -> Result<Json<GenerateContentResponse>, (StatusCode, Json<ErrorResponse>)> {
    let response = state
        .generator
        .generate(&payload.theme, payload.include_image)
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    let pool = Arc::new(state.db.pool().clone());
    let repo = ContentRepository::new(pool);

    let tags_vec: Vec<String> = response.tags.clone();
    let metadata = serde_json::json!({
        "generated_at": response.created_at,
        "theme": response.theme,
    });

    let content = repo
        .create_content(
            &response.theme,
            &response.text,
            &tags_vec,
            response.image_prompt.as_deref(),
            &metadata,
        )
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    let event = ContentEvent::Generated {
        content_id: content.id,
        theme: content.theme.clone(),
        status: format!("{:?}", content.status).to_lowercase(),
        metadata: metadata.clone(),
        timestamp: Utc::now(),
    };

    if let Err(e) = state.event_publisher.publish(event).await {
        tracing::warn!("Failed to publish content.generated event: {}", e);
    }

    Ok(Json(GenerateContentResponse {
        id: content.id,
        theme: content.theme,
        text: content.text,
        tags: content.tags,
        image_prompt: content.image_prompt,
        status: content.status,
        created_at: content.created_at,
    }))
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}
