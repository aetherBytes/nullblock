use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

use super::config::ConsensusModelConfig;

const OPENROUTER_MODELS_URL: &str = "https://openrouter.ai/api/v1/models";

#[derive(Debug, Clone, Deserialize)]
pub struct OpenRouterModel {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub context_length: Option<u64>,
    pub pricing: Option<ModelPricing>,
    pub top_provider: Option<TopProvider>,
    #[serde(default)]
    pub architecture: Option<Architecture>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelPricing {
    pub prompt: Option<String>,
    pub completion: Option<String>,
    pub image: Option<String>,
    pub request: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TopProvider {
    pub context_length: Option<u64>,
    pub max_completion_tokens: Option<u64>,
    pub is_moderated: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Architecture {
    pub modality: Option<String>,
    pub tokenizer: Option<String>,
    pub instruct_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenRouterModelsResponse {
    pub data: Vec<OpenRouterModel>,
}

lazy_static::lazy_static! {
    static ref DISCOVERED_MODELS: Arc<RwLock<Vec<ConsensusModelConfig>>> =
        Arc::new(RwLock::new(Vec::new()));
}

pub const TOP_REASONING_MODELS: &[&str] = &[
    "anthropic/claude-sonnet-4",
    "anthropic/claude-3.5-sonnet",
    "anthropic/claude-3-opus",
    "openai/gpt-4-turbo",
    "openai/gpt-4o",
    "openai/o1-preview",
    "openai/o1-mini",
    "google/gemini-pro-1.5",
    "google/gemini-2.0-flash-thinking-exp",
    "x-ai/grok-2",
    "x-ai/grok-beta",
    "meta-llama/llama-3.1-405b-instruct",
    "meta-llama/llama-3.3-70b-instruct",
    "deepseek/deepseek-chat",
    "deepseek/deepseek-r1",
    "qwen/qwen-2.5-72b-instruct",
    "mistralai/mistral-large",
];

pub fn get_reasoning_model_weight(model_id: &str) -> f64 {
    match model_id {
        s if s.contains("claude-sonnet-4") => 2.5,
        s if s.contains("claude-3-opus") => 2.0,
        s if s.contains("claude-3.5-sonnet") => 1.8,
        s if s.contains("gpt-4-turbo") || s.contains("gpt-4o") => 1.5,
        s if s.contains("o1-preview") || s.contains("o1-mini") => 2.0,
        s if s.contains("gemini-pro") || s.contains("gemini-2.0") => 1.3,
        s if s.contains("grok") => 1.4,
        s if s.contains("llama-3.1-405b") || s.contains("llama-3.3-70b") => 1.2,
        s if s.contains("deepseek-r1") => 1.8,
        s if s.contains("deepseek-chat") => 1.0,
        s if s.contains("qwen") => 1.0,
        s if s.contains("mistral-large") => 0.9,
        _ => 0.8,
    }
}

pub async fn fetch_available_models(api_key: &str) -> Result<Vec<OpenRouterModel>, String> {
    let client = Client::new();

    let response = client
        .get(OPENROUTER_MODELS_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("HTTP-Referer", "https://nullblock.io")
        .header("X-Title", "ArbFarm Consensus Model Discovery")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch models: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("OpenRouter API error ({}): {}", status, body));
    }

    let models_response: OpenRouterModelsResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse models response: {}", e))?;

    Ok(models_response.data)
}

pub async fn discover_best_reasoning_models(api_key: &str) -> Vec<ConsensusModelConfig> {
    info!("Discovering best reasoning models from OpenRouter...");

    match fetch_available_models(api_key).await {
        Ok(all_models) => {
            let mut best_models: Vec<ConsensusModelConfig> = Vec::new();

            for preferred_id in TOP_REASONING_MODELS {
                if let Some(model) = all_models.iter().find(|m| m.id == *preferred_id) {
                    let weight = get_reasoning_model_weight(&model.id);
                    let config = ConsensusModelConfig {
                        model_id: model.id.clone(),
                        display_name: model.name.clone(),
                        provider: model.id.split('/').next().unwrap_or("unknown").to_string(),
                        weight,
                        enabled: true,
                        max_tokens: model.top_provider
                            .as_ref()
                            .and_then(|p| p.max_completion_tokens)
                            .unwrap_or(4096) as u32,
                    };
                    best_models.push(config);
                }
            }

            if best_models.is_empty() {
                warn!("No preferred reasoning models found, using available models");
                best_models = all_models
                    .iter()
                    .filter(|m| {
                        m.id.contains("claude") ||
                        m.id.contains("gpt-4") ||
                        m.id.contains("gemini") ||
                        m.id.contains("llama")
                    })
                    .take(5)
                    .map(|m| ConsensusModelConfig {
                        model_id: m.id.clone(),
                        display_name: m.name.clone(),
                        provider: m.id.split('/').next().unwrap_or("unknown").to_string(),
                        weight: get_reasoning_model_weight(&m.id),
                        enabled: true,
                        max_tokens: 4096,
                    })
                    .collect();
            }

            best_models.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap_or(std::cmp::Ordering::Equal));

            let top_models: Vec<ConsensusModelConfig> = best_models.into_iter().take(5).collect();

            info!("Discovered {} best reasoning models:", top_models.len());
            for model in &top_models {
                info!("  - {} (weight: {:.1})", model.display_name, model.weight);
            }

            let mut discovered = DISCOVERED_MODELS.write().await;
            *discovered = top_models.clone();

            top_models
        }
        Err(e) => {
            error!("Failed to discover models from OpenRouter: {}", e);
            warn!("Using fallback model list");
            get_fallback_models()
        }
    }
}

pub fn get_fallback_models() -> Vec<ConsensusModelConfig> {
    vec![
        ConsensusModelConfig {
            model_id: "anthropic/claude-3.5-sonnet".to_string(),
            display_name: "Claude 3.5 Sonnet".to_string(),
            provider: "anthropic".to_string(),
            weight: 1.8,
            enabled: true,
            max_tokens: 4096,
        },
        ConsensusModelConfig {
            model_id: "openai/gpt-4-turbo".to_string(),
            display_name: "GPT-4 Turbo".to_string(),
            provider: "openai".to_string(),
            weight: 1.5,
            enabled: true,
            max_tokens: 4096,
        },
        ConsensusModelConfig {
            model_id: "meta-llama/llama-3.1-70b-instruct".to_string(),
            display_name: "Llama 3.1 70B".to_string(),
            provider: "meta-llama".to_string(),
            weight: 1.0,
            enabled: true,
            max_tokens: 4096,
        },
    ]
}

pub async fn get_discovered_models() -> Vec<ConsensusModelConfig> {
    let models = DISCOVERED_MODELS.read().await;
    if models.is_empty() {
        get_fallback_models()
    } else {
        models.clone()
    }
}

pub async fn refresh_models(api_key: &str) -> Vec<ConsensusModelConfig> {
    discover_best_reasoning_models(api_key).await
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDiscoveryStatus {
    pub models_discovered: usize,
    pub last_refresh: Option<chrono::DateTime<chrono::Utc>>,
    pub source: String,
    pub top_models: Vec<String>,
}

pub async fn get_discovery_status() -> ModelDiscoveryStatus {
    let models = DISCOVERED_MODELS.read().await;
    ModelDiscoveryStatus {
        models_discovered: models.len(),
        last_refresh: Some(chrono::Utc::now()),
        source: if models.is_empty() { "fallback".to_string() } else { "openrouter".to_string() },
        top_models: models.iter().map(|m| m.display_name.clone()).collect(),
    }
}
