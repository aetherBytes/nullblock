use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ApiKeyProvider {
    #[serde(rename = "openai")]
    OpenAI,
    #[serde(rename = "anthropic")]
    Anthropic,
    #[serde(rename = "groq")]
    Groq,
    #[serde(rename = "openrouter")]
    OpenRouter,
    #[serde(rename = "huggingface")]
    HuggingFace,
    #[serde(rename = "ollama")]
    Ollama,
}

impl sqlx::Type<sqlx::Postgres> for ApiKeyProvider {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for ApiKeyProvider {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> sqlx::encode::IsNull {
        <String as sqlx::Encode<sqlx::Postgres>>::encode(self.as_str().to_string(), buf)
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for ApiKeyProvider {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        Self::from_str(&s).map_err(|e| e.into())
    }
}

impl ApiKeyProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OpenAI => "openai",
            Self::Anthropic => "anthropic",
            Self::Groq => "groq",
            Self::OpenRouter => "openrouter",
            Self::HuggingFace => "huggingface",
            Self::Ollama => "ollama",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "openai" => Ok(Self::OpenAI),
            "anthropic" => Ok(Self::Anthropic),
            "groq" => Ok(Self::Groq),
            "openrouter" => Ok(Self::OpenRouter),
            "huggingface" => Ok(Self::HuggingFace),
            "ollama" => Ok(Self::Ollama),
            _ => Err(format!("Invalid provider: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApiKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: ApiKeyProvider,
    pub encrypted_key: Vec<u8>,
    pub encryption_iv: Vec<u8>,
    pub encryption_tag: Vec<u8>,
    pub key_prefix: Option<String>,
    pub key_suffix: Option<String>,
    pub key_name: Option<String>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub usage_count: i64,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ApiKeyResponse {
    pub id: String,
    pub user_id: String,
    pub provider: String,
    pub key_prefix: Option<String>,
    pub key_suffix: Option<String>,
    pub key_name: Option<String>,
    pub last_used_at: Option<String>,
    pub usage_count: i64,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<ApiKey> for ApiKeyResponse {
    fn from(key: ApiKey) -> Self {
        Self {
            id: key.id.to_string(),
            user_id: key.user_id.to_string(),
            provider: key.provider.as_str().to_string(),
            key_prefix: key.key_prefix,
            key_suffix: key.key_suffix,
            key_name: key.key_name,
            last_used_at: key.last_used_at.map(|dt| dt.to_rfc3339()),
            usage_count: key.usage_count,
            is_active: key.is_active,
            created_at: key.created_at.to_rfc3339(),
            updated_at: key.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    pub provider: String,
    pub api_key: String,
    pub key_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateApiKeyRequest {
    pub api_key: Option<String>,
    pub key_name: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ApiKeyListResponse {
    pub success: bool,
    pub data: Option<Vec<ApiKeyResponse>>,
    pub total: usize,
    pub error: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Serialize)]
pub struct ApiKeySingleResponse {
    pub success: bool,
    pub data: Option<ApiKeyResponse>,
    pub error: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Serialize)]
pub struct DecryptedApiKey {
    pub provider: String,
    pub api_key: String,
}

#[derive(Debug, Serialize)]
pub struct DecryptedApiKeysResponse {
    pub success: bool,
    pub data: Option<Vec<DecryptedApiKey>>,
    pub error: Option<String>,
    pub timestamp: String,
}
