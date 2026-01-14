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
    Database(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("External API error: {0}")]
    ExternalApi(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Event bus error: {0}")]
    EventBus(String),

    #[error("Execution error: {0}")]
    Execution(String),

    #[error("Threat detected: {0}")]
    ThreatDetected(String),

    #[error("Consensus failed: {0}")]
    ConsensusFailed(String),

    #[error("Rate limited: {0}")]
    RateLimited(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("HTTP request error: {0}")]
    HttpRequest(String),

    #[error("Timeout: {0}")]
    Timeout(String),
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::HttpRequest(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Serialization(err.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            AppError::Database(msg) => {
                tracing::error!("Database error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, msg.clone())
            }
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::ExternalApi(msg) => {
                tracing::error!("External API error: {}", msg);
                (StatusCode::BAD_GATEWAY, msg.clone())
            }
            AppError::Validation(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg.clone()),
            AppError::Serialization(msg) => {
                tracing::error!("Serialization error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, msg.clone())
            }
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, msg.clone())
            }
            AppError::EventBus(msg) => {
                tracing::error!("Event bus error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, msg.clone())
            }
            AppError::Execution(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            AppError::ThreatDetected(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            AppError::ConsensusFailed(msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::RateLimited(msg) => (StatusCode::TOO_MANY_REQUESTS, msg.clone()),
            AppError::Configuration(msg) => {
                tracing::error!("Configuration error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, msg.clone())
            }
            AppError::HttpRequest(msg) => {
                tracing::error!("HTTP request error: {}", msg);
                (StatusCode::BAD_GATEWAY, msg.clone())
            }
            AppError::Timeout(msg) => {
                tracing::error!("Timeout: {}", msg);
                (StatusCode::GATEWAY_TIMEOUT, msg.clone())
            }
        };

        let body = Json(json!({
            "error": error_message,
            "code": status.as_u16()
        }));

        (status, body).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
