use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum ProtocolError {
    InvalidRequest(String),
    TaskNotFound(String),
    AuthenticationRequired,
    InternalError(String),
}

impl IntoResponse for ProtocolError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ProtocolError::InvalidRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ProtocolError::TaskNotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ProtocolError::AuthenticationRequired => (
                StatusCode::UNAUTHORIZED,
                "Authentication required".to_string(),
            ),
            ProtocolError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolError::InvalidRequest(msg) => write!(f, "Invalid request: {}", msg),
            ProtocolError::TaskNotFound(msg) => write!(f, "Task not found: {}", msg),
            ProtocolError::AuthenticationRequired => write!(f, "Authentication required"),
            ProtocolError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for ProtocolError {}
