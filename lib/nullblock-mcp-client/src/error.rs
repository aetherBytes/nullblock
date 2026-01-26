use thiserror::Error;

#[derive(Error, Debug)]
pub enum McpError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("JSON-RPC error {code}: {message}")]
    JsonRpcError { code: i64, message: String },

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Connection not initialized")]
    NotInitialized,

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Service not found: {0}")]
    ServiceNotFound(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Max iterations reached in agentic loop")]
    MaxIterationsReached,

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

pub type McpResult<T> = Result<T, McpError>;
