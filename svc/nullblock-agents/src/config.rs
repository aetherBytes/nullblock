use serde::{Deserialize, Serialize};
use std::env;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Environment variable error: {0}")]
    EnvVar(#[from] env::VarError),
    #[error("Parse error: {0}")]
    Parse(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub service_name: String,
    pub version: String,
    pub server: ServerConfig,
    pub llm: LLMConfig,
    pub logging: LoggingConfig,
    pub features: FeatureConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
    pub request_timeout_ms: u64,
    pub max_request_body_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    pub default_model: String,
    pub request_timeout_ms: u64,
    pub max_tokens: u32,
    pub temperature: f64,
    pub cache_ttl_seconds: u64,
    pub max_cache_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String, // "json" or "pretty"
    pub file_enabled: bool,
    pub file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureConfig {
    pub arbitrage_enabled: bool,
    pub social_trading_enabled: bool,
    pub information_gathering_enabled: bool,
    pub metrics_enabled: bool,
    pub database_enabled: bool,
    pub redis_enabled: bool,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            service_name: env::var("SERVICE_NAME").unwrap_or_else(|_| "nullblock-agents".to_string()),
            version: env::var("SERVICE_VERSION").unwrap_or_else(|_| "0.1.0".to_string()),
            
            server: ServerConfig {
                host: env::var("AGENTS_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("AGENTS_PORT")
                    .unwrap_or_else(|_| "9001".to_string())
                    .parse()
                    .map_err(|e| ConfigError::Parse(format!("AGENTS_PORT: {}", e)))?,
                cors_origins: env::var("CORS_ORIGINS")
                    .unwrap_or_else(|_| "http://localhost:5173,http://localhost:3000".to_string())
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),
                request_timeout_ms: env::var("REQUEST_TIMEOUT_MS")
                    .unwrap_or_else(|_| "300000".to_string()) // 5 minutes for thinking models
                    .parse()
                    .map_err(|e| ConfigError::Parse(format!("REQUEST_TIMEOUT_MS: {}", e)))?,
                max_request_body_size: env::var("MAX_REQUEST_BODY_SIZE")
                    .unwrap_or_else(|_| "1048576".to_string()) // 1MB
                    .parse()
                    .map_err(|e| ConfigError::Parse(format!("MAX_REQUEST_BODY_SIZE: {}", e)))?,
            },

            llm: LLMConfig {
                default_model: env::var("DEFAULT_LLM_MODEL")
                    .unwrap_or_else(|_| "deepseek/deepseek-chat-v3.1:free".to_string()),
                request_timeout_ms: env::var("LLM_REQUEST_TIMEOUT_MS")
                    .unwrap_or_else(|_| "300000".to_string()) // 5 minutes for thinking models
                    .parse()
                    .map_err(|e| ConfigError::Parse(format!("LLM_REQUEST_TIMEOUT_MS: {}", e)))?,
                max_tokens: env::var("LLM_MAX_TOKENS")
                    .unwrap_or_else(|_| "1200".to_string())
                    .parse()
                    .map_err(|e| ConfigError::Parse(format!("LLM_MAX_TOKENS: {}", e)))?,
                temperature: env::var("LLM_TEMPERATURE")
                    .unwrap_or_else(|_| "0.8".to_string())
                    .parse()
                    .map_err(|e| ConfigError::Parse(format!("LLM_TEMPERATURE: {}", e)))?,
                cache_ttl_seconds: env::var("LLM_CACHE_TTL_SECONDS")
                    .unwrap_or_else(|_| "300".to_string())
                    .parse()
                    .map_err(|e| ConfigError::Parse(format!("LLM_CACHE_TTL_SECONDS: {}", e)))?,
                max_cache_size: env::var("LLM_MAX_CACHE_SIZE")
                    .unwrap_or_else(|_| "1000".to_string())
                    .parse()
                    .map_err(|e| ConfigError::Parse(format!("LLM_MAX_CACHE_SIZE: {}", e)))?,
            },

            logging: LoggingConfig {
                level: env::var("LOG_LEVEL").unwrap_or_else(|_| "INFO".to_string()),
                format: env::var("LOG_FORMAT").unwrap_or_else(|_| "pretty".to_string()),
                file_enabled: env::var("LOG_FILE_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .map_err(|e| ConfigError::Parse(format!("LOG_FILE_ENABLED: {}", e)))?,
                file_path: env::var("LOG_FILE_PATH")
                    .unwrap_or_else(|_| "logs/nullblock-agents.log".to_string()),
            },

            features: FeatureConfig {
                arbitrage_enabled: env::var("FEATURE_ARBITRAGE")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .map_err(|e| ConfigError::Parse(format!("FEATURE_ARBITRAGE: {}", e)))?,
                social_trading_enabled: env::var("FEATURE_SOCIAL_TRADING")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .map_err(|e| ConfigError::Parse(format!("FEATURE_SOCIAL_TRADING: {}", e)))?,
                information_gathering_enabled: env::var("FEATURE_INFORMATION_GATHERING")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .map_err(|e| ConfigError::Parse(format!("FEATURE_INFORMATION_GATHERING: {}", e)))?,
                metrics_enabled: env::var("FEATURE_METRICS")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .map_err(|e| ConfigError::Parse(format!("FEATURE_METRICS: {}", e)))?,
                database_enabled: env::var("FEATURE_DATABASE")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .map_err(|e| ConfigError::Parse(format!("FEATURE_DATABASE: {}", e)))?,
                redis_enabled: env::var("FEATURE_REDIS")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .map_err(|e| ConfigError::Parse(format!("FEATURE_REDIS: {}", e)))?,
            },
        })
    }

    pub fn get_api_keys(&self) -> ApiKeys {
        ApiKeys {
            openai: Self::get_valid_api_key("OPENAI_API_KEY"),
            anthropic: Self::get_valid_api_key("ANTHROPIC_API_KEY"),
            groq: Self::get_valid_api_key("GROQ_API_KEY"),
            huggingface: Self::get_valid_api_key("HUGGINGFACE_API_KEY"),
            openrouter: Self::get_valid_api_key("OPENROUTER_API_KEY"),
        }
    }

    fn get_valid_api_key(env_var: &str) -> Option<String> {
        if let Ok(key) = env::var(env_var) {
            // Check for placeholder values that indicate the key isn't configured
            let placeholder_patterns = [
                "your-",
                "replace-",
                "enter-",
                "add-",
                "insert-",
                "api-key-here",
                "key-here",
                "token-here",
                "secret-here",
            ];

            let key_lower = key.to_lowercase();

            // Check if key is obviously a placeholder
            if placeholder_patterns.iter().any(|pattern| key_lower.contains(pattern)) {
                None
            } else if key.len() < 10 {
                // API keys are typically longer than 10 characters
                None
            } else {
                Some(key)
            }
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct ApiKeys {
    pub openai: Option<String>,
    pub anthropic: Option<String>,
    pub groq: Option<String>,
    pub huggingface: Option<String>,
    pub openrouter: Option<String>,
}

impl ApiKeys {
    pub fn available_providers(&self) -> Vec<&'static str> {
        let mut providers = Vec::new();
        
        if self.openai.is_some() {
            providers.push("openai");
        }
        if self.anthropic.is_some() {
            providers.push("anthropic");
        }
        if self.groq.is_some() {
            providers.push("groq");
        }
        if self.huggingface.is_some() {
            providers.push("huggingface");
        }
        if self.openrouter.is_some() {
            providers.push("openrouter");
        }
        
        // Local providers are always "available" (though might not be running)
        providers.push("ollama");
        
        providers
    }

    pub fn has_api_providers(&self) -> bool {
        self.openai.is_some() 
            || self.anthropic.is_some() 
            || self.groq.is_some() 
            || self.huggingface.is_some() 
            || self.openrouter.is_some()
    }
}