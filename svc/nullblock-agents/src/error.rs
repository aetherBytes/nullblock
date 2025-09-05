use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Agent not initialized")]
    AgentNotInitialized,
    
    #[error("Agent not running")]
    AgentNotRunning,
    
    #[error("Model not available: {0}")]
    ModelNotAvailable(String),
    
    #[error("LLM request failed: {0}")]
    LLMRequestFailed(String),
    
    #[error("Invalid model configuration: {0}")]
    InvalidModelConfig(String),
    
    #[error("Conversation error: {0}")]
    ConversationError(String),
    
    #[error("Arbitrage service error: {0}")]
    ArbitrageError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Timeout error: {0}")]
    TimeoutError(String),
    
    #[error("Authentication error: {0}")]
    AuthError(String),
    
    #[error("Rate limit exceeded: {0}")]
    RateLimitError(String),
    
    #[error("Internal server error: {0}")]
    InternalError(String),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::AgentNotInitialized 
            | AppError::AgentNotRunning => StatusCode::SERVICE_UNAVAILABLE,
            
            AppError::ModelNotAvailable(_) 
            | AppError::InvalidModelConfig(_) 
            | AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            
            AppError::AuthError(_) => StatusCode::UNAUTHORIZED,
            
            AppError::RateLimitError(_) => StatusCode::TOO_MANY_REQUESTS,
            
            AppError::TimeoutError(_) => StatusCode::REQUEST_TIMEOUT,
            
            AppError::LLMRequestFailed(_) 
            | AppError::ConversationError(_) 
            | AppError::ArbitrageError(_) 
            | AppError::ConfigError(_) 
            | AppError::NetworkError(_) 
            | AppError::SerializationError(_) 
            | AppError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn error_type(&self) -> &'static str {
        match self {
            AppError::AgentNotInitialized => "agent_not_initialized",
            AppError::AgentNotRunning => "agent_not_running",
            AppError::ModelNotAvailable(_) => "model_not_available",
            AppError::LLMRequestFailed(_) => "llm_request_failed",
            AppError::InvalidModelConfig(_) => "invalid_model_config",
            AppError::ConversationError(_) => "conversation_error",
            AppError::ArbitrageError(_) => "arbitrage_error",
            AppError::ConfigError(_) => "config_error",
            AppError::NetworkError(_) => "network_error",
            AppError::SerializationError(_) => "serialization_error",
            AppError::TimeoutError(_) => "timeout_error",
            AppError::AuthError(_) => "auth_error",
            AppError::RateLimitError(_) => "rate_limit_error",
            AppError::InternalError(_) => "internal_error",
            AppError::BadRequest(_) => "bad_request",
        }
    }

    pub fn user_friendly_message(&self) -> String {
        match self {
            AppError::AgentNotInitialized | AppError::AgentNotRunning => {
                "🤖 The agent service is starting up. Please try again in a moment.".to_string()
            }
            AppError::ModelNotAvailable(model) => {
                format!("🧠 The model '{}' is not currently available. Please select a different model or check your API keys.", model)
            }
            AppError::LLMRequestFailed(details) => {
                if details.to_lowercase().contains("api key") {
                    "🔑 Authentication issue with the AI service. Please check your API keys.".to_string()
                } else if details.to_lowercase().contains("timeout") {
                    "⏰ The AI service is taking too long to respond. Please try again in a moment.".to_string()
                } else if details.to_lowercase().contains("rate limit") {
                    "🚦 Too many requests. Please wait a moment before trying again.".to_string()
                } else {
                    "🌐 There was an issue communicating with the AI service. Please try again.".to_string()
                }
            }
            AppError::NetworkError(_) => {
                "🌐 Network connectivity issue. Please check your internet connection and try again.".to_string()
            }
            AppError::TimeoutError(_) => {
                "⏰ Request timed out. The service might be under heavy load. Please try again.".to_string()
            }
            AppError::AuthError(_) => {
                "🔑 Authentication failed. Please check your API keys and try again.".to_string()
            }
            AppError::RateLimitError(_) => {
                "🚦 Rate limit exceeded. Please wait a moment before making another request.".to_string()
            }
            _ => {
                format!("❌ {}", self.to_string())
            }
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status_code = self.status_code();
        let error_response = json!({
            "error": self.error_type(),
            "message": self.user_friendly_message(),
            "details": self.to_string(),
            "timestamp": Utc::now().to_rfc3339(),
            "status_code": status_code.as_u16()
        });

        // Log the error
        tracing::error!(
            error = %self,
            status_code = %status_code,
            error_type = self.error_type(),
            "Request failed with error"
        );

        (status_code, Json(error_response)).into_response()
    }
}

// Conversion from common error types
impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            AppError::TimeoutError(err.to_string())
        } else if err.is_connect() {
            AppError::NetworkError(err.to_string())
        } else {
            AppError::LLMRequestFailed(err.to_string())
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::SerializationError(err.to_string())
    }
}

impl From<crate::config::ConfigError> for AppError {
    fn from(err: crate::config::ConfigError) -> Self {
        AppError::ConfigError(err.to_string())
    }
}

impl From<tokio::time::error::Elapsed> for AppError {
    fn from(err: tokio::time::error::Elapsed) -> Self {
        AppError::TimeoutError(err.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;