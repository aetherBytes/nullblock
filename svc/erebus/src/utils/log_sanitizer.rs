use regex::Regex;
use serde_json::{json, Value};
use std::collections::HashMap;

lazy_static::lazy_static! {
    static ref API_KEY_REGEX: Regex = Regex::new(r"(sk-[a-zA-Z0-9]{20,}|sk-proj-[a-zA-Z0-9]{20,}|api_key[=:]\s*[a-zA-Z0-9]{20,})").unwrap();
    static ref WALLET_ADDRESS_REGEX: Regex = Regex::new(r"0x[a-fA-F0-9]{40}").unwrap();
    static ref BEARER_TOKEN_REGEX: Regex = Regex::new(r"Bearer\s+[a-zA-Z0-9\-_.]+").unwrap();
    static ref CONNECTION_STRING_REGEX: Regex = Regex::new(r"(postgresql|mysql|mongodb)://[^@]+@[^\s]+").unwrap();
    static ref EMAIL_REGEX: Regex = Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap();
    static ref IPV4_PRIVATE_REGEX: Regex = Regex::new(r"\b(10\.\d{1,3}\.\d{1,3}\.\d{1,3}|172\.(1[6-9]|2\d|3[01])\.\d{1,3}\.\d{1,3}|192\.168\.\d{1,3}\.\d{1,3})\b").unwrap();
    static ref FILE_PATH_REGEX: Regex = Regex::new(r"/home/[a-zA-Z0-9_-]+/[^\s]+").unwrap();
}

#[derive(Debug, Clone)]
pub struct SanitizedLog {
    pub timestamp: String,
    pub level: String,
    pub source: String,
    pub message: String,
    pub category: LogCategory,
    pub metadata: HashMap<String, Value>,
    pub sanitized: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogCategory {
    AgentConversation,
    LlmApiCall,
    ErebusRouting,
    TaskLifecycle,
    HealthCheck,
    SystemError,
    DatabaseQuery,
    Unknown,
}

pub struct LogSanitizer;

impl LogSanitizer {
    pub fn sanitize_text(text: &str) -> String {
        let mut sanitized = text.to_string();

        sanitized = API_KEY_REGEX
            .replace_all(&sanitized, |caps: &regex::Captures| {
                let key = &caps[0];
                if key.len() > 8 {
                    format!("{}****{}", &key[..4], &key[key.len() - 4..])
                } else {
                    "****".to_string()
                }
            })
            .to_string();

        sanitized = WALLET_ADDRESS_REGEX
            .replace_all(&sanitized, |caps: &regex::Captures| {
                let addr = &caps[0];
                format!("0x****{}", &addr[addr.len() - 4..])
            })
            .to_string();

        sanitized = BEARER_TOKEN_REGEX
            .replace_all(&sanitized, "Bearer ****")
            .to_string();

        sanitized = CONNECTION_STRING_REGEX
            .replace_all(&sanitized, "$1://****@****")
            .to_string();

        sanitized = EMAIL_REGEX
            .replace_all(&sanitized, "****@****.***")
            .to_string();

        sanitized = IPV4_PRIVATE_REGEX
            .replace_all(&sanitized, "[INTERNAL_IP]")
            .to_string();

        sanitized = FILE_PATH_REGEX
            .replace_all(&sanitized, |caps: &regex::Captures| {
                let path = &caps[0];
                if let Some(idx) = path.rfind('/') {
                    format!("[PROJECT_ROOT]/{}", &path[idx + 1..])
                } else {
                    "[PROJECT_ROOT]".to_string()
                }
            })
            .to_string();

        sanitized
    }

    pub fn sanitize_json_value(value: &Value) -> Value {
        match value {
            Value::String(s) => Value::String(Self::sanitize_text(s)),
            Value::Object(map) => {
                let mut sanitized_map = serde_json::Map::new();
                for (key, val) in map {
                    let sanitized_key = key.to_lowercase();

                    if sanitized_key.contains("password")
                        || sanitized_key.contains("secret")
                        || sanitized_key.contains("private")
                        || sanitized_key.contains("token")
                        || sanitized_key.contains("key")
                        || sanitized_key.contains("authorization")
                    {
                        sanitized_map.insert(key.clone(), Value::String("****".to_string()));
                    } else {
                        sanitized_map.insert(key.clone(), Self::sanitize_json_value(val));
                    }
                }
                Value::Object(sanitized_map)
            }
            Value::Array(arr) => {
                Value::Array(arr.iter().map(|v| Self::sanitize_json_value(v)).collect())
            }
            _ => value.clone(),
        }
    }

    pub fn sanitize_metadata(metadata: &HashMap<String, Value>) -> HashMap<String, Value> {
        let mut sanitized = HashMap::new();

        for (key, value) in metadata {
            let sanitized_key = key.to_lowercase();

            if sanitized_key.contains("password")
                || sanitized_key.contains("secret")
                || sanitized_key.contains("private_key")
                || sanitized_key == "authorization"
                || sanitized_key == "x-api-key"
            {
                sanitized.insert(key.clone(), Value::String("****".to_string()));
            } else {
                sanitized.insert(key.clone(), Self::sanitize_json_value(value));
            }
        }

        sanitized
    }

    pub fn mask_user_id(user_id: &str) -> String {
        if user_id.len() > 8 {
            format!("{}****", &user_id[..8])
        } else {
            "****".to_string()
        }
    }

    pub fn categorize_log(source: &str, message: &str) -> LogCategory {
        let source_lower = source.to_lowercase();
        let message_lower = message.to_lowercase();

        if source_lower.contains("hecate") || source_lower.contains("siren") {
            if message_lower.contains("conversation") || message_lower.contains("chat") {
                return LogCategory::AgentConversation;
            }
        }

        if message_lower.contains("openrouter")
            || message_lower.contains("llm")
            || message_lower.contains("model")
            || source_lower.contains("providers")
        {
            return LogCategory::LlmApiCall;
        }

        if source_lower.contains("erebus") || message_lower.contains("proxy") || message_lower.contains("routing") {
            return LogCategory::ErebusRouting;
        }

        if message_lower.contains("task") && (message_lower.contains("created") || message_lower.contains("completed") || message_lower.contains("failed")) {
            return LogCategory::TaskLifecycle;
        }

        if message_lower.contains("health") || message_lower.contains("status") {
            return LogCategory::HealthCheck;
        }

        if message_lower.contains("error") || message_lower.contains("failed") {
            return LogCategory::SystemError;
        }

        if source_lower.contains("database") || source_lower.contains("sqlx") || source_lower.contains("postgres") {
            return LogCategory::DatabaseQuery;
        }

        LogCategory::Unknown
    }

    pub fn create_sanitized_log(
        timestamp: String,
        level: String,
        source: String,
        message: String,
        metadata: Option<HashMap<String, Value>>,
    ) -> SanitizedLog {
        let sanitized_message = Self::sanitize_text(&message);
        let sanitized_source = Self::sanitize_text(&source);
        let category = Self::categorize_log(&source, &message);

        let sanitized_metadata = metadata
            .map(|m| Self::sanitize_metadata(&m))
            .unwrap_or_default();

        SanitizedLog {
            timestamp,
            level,
            source: sanitized_source,
            message: sanitized_message,
            category,
            metadata: sanitized_metadata,
            sanitized: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_api_key() {
        let text = "Using API key: sk-proj-abcdefghijklmnopqrstuvwxyz";
        let sanitized = LogSanitizer::sanitize_text(text);
        assert!(!sanitized.contains("abcdefghijklmnopqrstuvwxyz"));
        assert!(sanitized.contains("sk-p****"));
    }

    #[test]
    fn test_sanitize_wallet_address() {
        let text = "Wallet: 0x1234567890abcdef1234567890abcdef12345678";
        let sanitized = LogSanitizer::sanitize_text(text);
        assert!(sanitized.contains("0x****5678"));
        assert!(!sanitized.contains("1234567890abcdef"));
    }

    #[test]
    fn test_sanitize_bearer_token() {
        let text = "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
        let sanitized = LogSanitizer::sanitize_text(text);
        assert!(sanitized.contains("Bearer ****"));
        assert!(!sanitized.contains("eyJhbGci"));
    }

    #[test]
    fn test_sanitize_connection_string() {
        let text = "postgresql://user:password@localhost:5432/db";
        let sanitized = LogSanitizer::sanitize_text(text);
        assert!(sanitized.contains("postgresql://****@****"));
        assert!(!sanitized.contains("user:password"));
    }

    #[test]
    fn test_categorize_log() {
        assert!(matches!(
            LogSanitizer::categorize_log("hecate.rs", "conversation started"),
            LogCategory::AgentConversation
        ));

        assert!(matches!(
            LogSanitizer::categorize_log("providers.rs", "OpenRouter API call"),
            LogCategory::LlmApiCall
        ));

        assert!(matches!(
            LogSanitizer::categorize_log("tasks.rs", "Task created successfully"),
            LogCategory::TaskLifecycle
        ));
    }
}
