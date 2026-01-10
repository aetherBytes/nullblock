use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Database(ref e) => {
                tracing::error!("Database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
            AppError::NotFound(ref message) => (StatusCode::NOT_FOUND, message.clone()),
            AppError::BadRequest(ref message) => (StatusCode::BAD_REQUEST, message.clone()),
            AppError::Unauthorized(ref message) => (StatusCode::UNAUTHORIZED, message.clone()),
            AppError::Forbidden(ref message) => (StatusCode::FORBIDDEN, message.clone()),
            AppError::Conflict(ref message) => (StatusCode::CONFLICT, message.clone()),
            AppError::Internal(ref message) => {
                tracing::error!("Internal error: {}", message);
                (StatusCode::INTERNAL_SERVER_ERROR, message.clone())
            }
        };

        let body = Json(json!({
            "success": false,
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
