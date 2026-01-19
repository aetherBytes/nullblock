use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

pub const DEV_WALLET: &str = "YOUR_DEV_WALLET_PUBKEY";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    pub enabled: bool,
    pub models: Vec<ConsensusModelConfig>,
    pub min_consensus_threshold: f64,
    pub auto_apply_recommendations: bool,
    pub review_interval_hours: u32,
    pub max_tokens_per_request: u32,
    pub timeout_ms: u64,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            models: get_standard_models(),
            min_consensus_threshold: 0.6,
            auto_apply_recommendations: false,
            review_interval_hours: 24,
            max_tokens_per_request: 2048,
            timeout_ms: 30000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusModelConfig {
    pub model_id: String,
    pub display_name: String,
    pub provider: String,
    pub weight: f64,
    pub enabled: bool,
    pub max_tokens: u32,
}

impl ConsensusModelConfig {
    pub fn new(model_id: impl Into<String>, weight: f64) -> Self {
        let id = model_id.into();
        let parts: Vec<&str> = id.split('/').collect();
        let provider = if parts.len() >= 2 {
            parts[0].to_string()
        } else {
            "unknown".to_string()
        };
        let display_name = if parts.len() >= 2 {
            parts[1].replace('-', " ").to_string()
        } else {
            id.clone()
        };

        Self {
            model_id: id,
            display_name,
            provider,
            weight,
            enabled: true,
            max_tokens: 2048,
        }
    }

    pub fn with_display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = name.into();
        self
    }
}

pub fn get_standard_models() -> Vec<ConsensusModelConfig> {
    vec![
        ConsensusModelConfig::new("anthropic/claude-3.5-sonnet", 1.5)
            .with_display_name("Claude 3.5 Sonnet"),
        ConsensusModelConfig::new("openai/gpt-4-turbo", 1.0)
            .with_display_name("GPT-4 Turbo"),
        ConsensusModelConfig::new("meta-llama/llama-3.1-70b-instruct", 0.8)
            .with_display_name("Llama 3.1 70B"),
    ]
}

pub fn get_dev_wallet_models() -> Vec<ConsensusModelConfig> {
    vec![
        ConsensusModelConfig::new("anthropic/claude-3-opus", 2.0)
            .with_display_name("Claude 3 Opus"),
        ConsensusModelConfig::new("openai/gpt-4-turbo", 1.5)
            .with_display_name("GPT-4 Turbo"),
        ConsensusModelConfig::new("anthropic/claude-3.5-sonnet", 1.5)
            .with_display_name("Claude 3.5 Sonnet"),
        ConsensusModelConfig::new("meta-llama/llama-3.1-405b-instruct", 1.0)
            .with_display_name("Llama 3.1 405B"),
    ]
}

pub fn is_dev_wallet(wallet: &str) -> bool {
    wallet == DEV_WALLET || std::env::var("ARBFARM_DEV_MODE").is_ok()
}

pub fn get_models_for_wallet(wallet: &str) -> Vec<ConsensusModelConfig> {
    if is_dev_wallet(wallet) {
        get_dev_wallet_models()
    } else {
        get_standard_models()
    }
}

pub fn get_enabled_model_ids(config: &ConsensusConfig) -> Vec<String> {
    config
        .models
        .iter()
        .filter(|m| m.enabled)
        .map(|m| m.model_id.clone())
        .collect()
}

#[derive(Debug, Clone)]
pub struct ConsensusConfigManager {
    config: Arc<RwLock<ConsensusConfig>>,
    wallet_address: String,
}

impl ConsensusConfigManager {
    pub fn new(wallet_address: impl Into<String>) -> Self {
        let wallet = wallet_address.into();
        let mut config = ConsensusConfig::default();
        config.models = get_models_for_wallet(&wallet);

        Self {
            config: Arc::new(RwLock::new(config)),
            wallet_address: wallet,
        }
    }

    pub async fn get_config(&self) -> ConsensusConfig {
        self.config.read().await.clone()
    }

    pub async fn update_config(&self, new_config: ConsensusConfig) {
        let mut config = self.config.write().await;
        *config = new_config;
    }

    pub async fn set_enabled(&self, enabled: bool) {
        let mut config = self.config.write().await;
        config.enabled = enabled;
    }

    pub async fn add_model(&self, model: ConsensusModelConfig) {
        let mut config = self.config.write().await;
        if !config.models.iter().any(|m| m.model_id == model.model_id) {
            config.models.push(model);
        }
    }

    pub async fn remove_model(&self, model_id: &str) {
        let mut config = self.config.write().await;
        config.models.retain(|m| m.model_id != model_id);
    }

    pub async fn toggle_model(&self, model_id: &str, enabled: bool) {
        let mut config = self.config.write().await;
        if let Some(model) = config.models.iter_mut().find(|m| m.model_id == model_id) {
            model.enabled = enabled;
        }
    }

    pub async fn set_model_weight(&self, model_id: &str, weight: f64) {
        let mut config = self.config.write().await;
        if let Some(model) = config.models.iter_mut().find(|m| m.model_id == model_id) {
            model.weight = weight;
        }
    }

    pub async fn set_review_interval(&self, hours: u32) {
        let mut config = self.config.write().await;
        config.review_interval_hours = hours;
    }

    pub async fn set_min_threshold(&self, threshold: f64) {
        let mut config = self.config.write().await;
        config.min_consensus_threshold = threshold.clamp(0.0, 1.0);
    }

    pub async fn set_auto_apply(&self, auto_apply: bool) {
        let mut config = self.config.write().await;
        config.auto_apply_recommendations = auto_apply;
    }

    pub fn is_dev_wallet(&self) -> bool {
        is_dev_wallet(&self.wallet_address)
    }

    pub async fn reset_to_defaults(&self) {
        let models = get_models_for_wallet(&self.wallet_address);
        let mut config = self.config.write().await;
        *config = ConsensusConfig::default();
        config.models = models;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConsensusConfigRequest {
    pub enabled: Option<bool>,
    pub models: Option<Vec<ConsensusModelConfig>>,
    pub min_consensus_threshold: Option<f64>,
    pub auto_apply_recommendations: Option<bool>,
    pub review_interval_hours: Option<u32>,
}

pub const AVAILABLE_MODELS: &[(&str, &str, f64)] = &[
    ("anthropic/claude-3-opus", "Claude 3 Opus", 2.0),
    ("anthropic/claude-3.5-sonnet", "Claude 3.5 Sonnet", 1.5),
    ("anthropic/claude-3-haiku", "Claude 3 Haiku", 0.6),
    ("openai/gpt-4-turbo", "GPT-4 Turbo", 1.0),
    ("openai/gpt-4o", "GPT-4o", 1.2),
    ("openai/gpt-4o-mini", "GPT-4o Mini", 0.5),
    ("meta-llama/llama-3.1-405b-instruct", "Llama 3.1 405B", 1.0),
    ("meta-llama/llama-3.1-70b-instruct", "Llama 3.1 70B", 0.8),
    ("meta-llama/llama-3.1-8b-instruct", "Llama 3.1 8B", 0.4),
    ("google/gemini-pro-1.5", "Gemini Pro 1.5", 0.9),
    ("mistralai/mistral-large", "Mistral Large", 0.7),
    ("mistralai/mixtral-8x22b-instruct", "Mixtral 8x22B", 0.6),
];

pub fn get_all_available_models() -> Vec<ConsensusModelConfig> {
    AVAILABLE_MODELS
        .iter()
        .map(|(id, name, weight)| {
            ConsensusModelConfig::new(*id, *weight).with_display_name(*name)
        })
        .collect()
}
