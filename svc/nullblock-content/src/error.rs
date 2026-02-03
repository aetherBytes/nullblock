use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContentError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Repository error: {0}")]
    RepositoryError(String),

    #[error("Generation error: {0}")]
    GenerationError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl From<sqlx::Error> for ContentError {
    fn from(err: sqlx::Error) -> Self {
        ContentError::DatabaseError(err.to_string())
    }
}

impl From<serde_json::Error> for ContentError {
    fn from(err: serde_json::Error) -> Self {
        ContentError::SerializationError(err.to_string())
    }
}
