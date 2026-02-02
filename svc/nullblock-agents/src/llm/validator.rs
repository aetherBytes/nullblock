#![allow(dead_code)]

use crate::{
    config::ApiKeys,
    error::{AppError, AppResult},
    models::{LLMRequest, LLMResponse},
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::timeout;
use tracing::{error, info, warn};

use super::factory::LLMServiceFactory;

const VALIDATION_PROMPT: &str = "What is 2+2? Respond with only the number.";
const VALIDATION_TIMEOUT_SECS: u64 = 5;

pub struct ModelValidator {
    llm_factory: Arc<RwLock<LLMServiceFactory>>,
}

impl ModelValidator {
    pub fn new(llm_factory: Arc<RwLock<LLMServiceFactory>>) -> Self {
        Self { llm_factory }
    }

    pub async fn validate_model(&self, model_name: &str, _api_keys: &ApiKeys) -> AppResult<bool> {
        info!("🔍 Validating model: {}", model_name);

        let request = LLMRequest {
            prompt: VALIDATION_PROMPT.to_string(),
            system_prompt: None,
            messages: None,
            max_tokens: Some(10),
            temperature: Some(0.0),
            top_p: None,
            stop_sequences: None,
            tools: None,
            model_override: Some(model_name.to_string()),
            concise: true,
            max_chars: Some(10),
            reasoning: None,
        };

        let validation_timeout = Duration::from_secs(VALIDATION_TIMEOUT_SECS);
        let factory = self.llm_factory.read().await;

        let validation_result = timeout(validation_timeout, factory.generate(&request, None)).await;

        match validation_result {
            Ok(Ok(response)) => {
                if self.is_valid_response(&response) {
                    info!("✅ Model {} validation passed", model_name);
                    Ok(true)
                } else {
                    warn!(
                        "❌ Model {} returned invalid response: {}",
                        model_name, response.content
                    );
                    Ok(false)
                }
            }
            Ok(Err(e)) => {
                if self.is_recoverable_error(&e) {
                    warn!(
                        "⚠️ Model {} failed validation (recoverable): {}",
                        model_name, e
                    );
                    Ok(false)
                } else {
                    error!(
                        "❌ Model {} failed validation (unrecoverable): {}",
                        model_name, e
                    );
                    Err(e)
                }
            }
            Err(_) => {
                warn!(
                    "⏱️ Model {} validation timed out after {} seconds",
                    model_name, VALIDATION_TIMEOUT_SECS
                );
                Ok(false)
            }
        }
    }

    pub async fn validate_model_is_free(&self, model_name: &str) -> AppResult<bool> {
        let factory = self.llm_factory.read().await;
        let free_models = factory.get_free_models().await?;

        let is_free = free_models.iter().any(|model| {
            model
                .get("id")
                .and_then(|id| id.as_str())
                .map(|id| id == model_name)
                .unwrap_or(false)
        });

        if is_free {
            info!("💰 Model {} is free", model_name);
        } else {
            warn!("💸 Model {} is not free", model_name);
        }

        Ok(is_free)
    }

    fn is_valid_response(&self, response: &LLMResponse) -> bool {
        !response.content.trim().is_empty()
            && response.content.len() < 100
            && response.finish_reason != "error"
    }

    fn is_recoverable_error(&self, error: &AppError) -> bool {
        match error {
            AppError::ModelNotAvailable(_) => true,
            AppError::LLMRequestFailed(msg) => {
                msg.contains("502")
                    || msg.contains("503")
                    || msg.contains("Bad Gateway")
                    || msg.contains("Provider returned error")
                    || msg.contains("no longer available")
                    || msg.contains("rate limit")
            }
            AppError::TimeoutError(_) => true,
            AppError::RateLimitError(_) => true,
            AppError::NetworkError(_) => true,
            _ => false,
        }
    }
}

pub async fn sort_models_by_context_length(models: Vec<serde_json::Value>) -> Vec<String> {
    let mut model_with_context: Vec<(String, u64)> = models
        .into_iter()
        .filter_map(|model| {
            let id = model
                .get("id")
                .and_then(|id| id.as_str())
                .map(|s| s.to_string())?;

            let context = model
                .get("context_length")
                .and_then(|c| c.as_u64())
                .unwrap_or(0);

            Some((id, context))
        })
        .collect();

    model_with_context.sort_by(|a, b| b.1.cmp(&a.1));

    model_with_context.into_iter().map(|(id, _)| id).collect()
}
