use crate::{
    config::ApiKeys,
    error::{AppError, AppResult},
    log_model_info,
    models::{LLMRequest, LLMResponse, ModelConfig, ModelProvider},
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use super::{
    providers::{
        AnthropicProvider, GroqProvider, OllamaProvider, OpenAIProvider, OpenRouterProvider,
        Provider,
    },
    router::{ModelRouter, OptimizationGoal, TaskRequirements},
};

pub struct LLMServiceFactory {
    providers: HashMap<ModelProvider, Arc<dyn Provider>>,
    router: Arc<RwLock<ModelRouter>>,
    request_stats: Arc<RwLock<HashMap<String, usize>>>,
    cost_tracking: Arc<RwLock<HashMap<String, f64>>>,
}

impl LLMServiceFactory {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            router: Arc::new(RwLock::new(ModelRouter::new())),
            request_stats: Arc::new(RwLock::new(HashMap::new())),
            cost_tracking: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn initialize(&mut self, api_keys: &ApiKeys) -> AppResult<()> {
        info!("🧠 Initializing LLM Service Factory...");

        // Initialize providers based on available API keys
        let mut available_providers = Vec::new();
        let mut missing_providers = Vec::new();

        // OpenAI
        if let Some(api_key) = &api_keys.openai {
            self.providers.insert(
                ModelProvider::OpenAI,
                Arc::new(OpenAIProvider::new(api_key.clone())),
            );
            available_providers.push("openai");
        } else {
            missing_providers.push("openai");
        }

        // Anthropic
        if let Some(api_key) = &api_keys.anthropic {
            self.providers.insert(
                ModelProvider::Anthropic,
                Arc::new(AnthropicProvider::new(api_key.clone())),
            );
            available_providers.push("anthropic");
        } else {
            missing_providers.push("anthropic");
        }

        // Groq
        if let Some(api_key) = &api_keys.groq {
            self.providers.insert(
                ModelProvider::Groq,
                Arc::new(GroqProvider::new(api_key.clone())),
            );
            available_providers.push("groq");
        } else {
            missing_providers.push("groq");
        }

        // OpenRouter
        if let Some(api_key) = &api_keys.openrouter {
            self.providers.insert(
                ModelProvider::OpenRouter,
                Arc::new(OpenRouterProvider::new(api_key.clone())),
            );
            available_providers.push("openrouter");
            info!("OpenRouter is configured as the cloud model aggregator");
        } else {
            missing_providers.push("openrouter");
        }

        // Ollama (local, no API key needed)
        self.providers
            .insert(ModelProvider::Ollama, Arc::new(OllamaProvider::new(None)));
        available_providers.push("ollama");

        // Log provider status
        if !available_providers.is_empty() {
            info!("Available LLM providers: {}", available_providers.join(", "));
        }
        if !missing_providers.is_empty() {
            warn!(
                "Missing API keys for providers: {}",
                missing_providers.join(", ")
            );
        }

        // Test local model connectivity
        self.test_local_models().await;

        info!("✅ LLM Service Factory initialized");
        Ok(())
    }

    pub async fn generate(
        &self,
        request: &LLMRequest,
        requirements: Option<TaskRequirements>,
    ) -> AppResult<LLMResponse> {
        let requirements = requirements.unwrap_or_else(TaskRequirements::default);

        // Route request to optimal model
        let router = self.router.read().await;
        let routing_decision = router.route_request(&requirements).await?;
        drop(router);

        // Override model if specified in request
        let (selected_model, model_config) = if let Some(override_model) = &request.model_override {
            // For simplicity, create a basic config for override models
            let config = self.create_model_config_for_override(override_model);
            (override_model.clone(), config)
        } else {
            (routing_decision.selected_model, routing_decision.model_config)
        };

        info!("🧠 Using model: {} (confidence: {:.2})", selected_model, routing_decision.confidence);

        // Generate response with the selected model
        let response = self.generate_with_model(request, &model_config).await?;

        // Log model info
        log_model_info!(
            response.model_used,
            model_config.provider.as_str(),
            response.cost_estimate
        );

        // Update statistics
        self.update_stats(&selected_model, &response).await;

        Ok(response)
    }

    async fn generate_with_model(
        &self,
        request: &LLMRequest,
        config: &ModelConfig,
    ) -> AppResult<LLMResponse> {
        let provider = self
            .providers
            .get(&config.provider)
            .ok_or_else(|| AppError::ModelNotAvailable(format!("Provider {} not available", config.provider.as_str())))?;

        provider.generate(request, config).await
    }

    async fn test_local_models(&self) {
        // Test Ollama connectivity
        if let Some(provider) = self.providers.get(&ModelProvider::Ollama) {
            match provider.health_check().await {
                Ok(true) => {
                    info!("✅ Ollama is available (local model server)");
                    let mut router = self.router.write().await;
                    // Enable Ollama models in router
                    router.update_model_status("llama2".to_string(), true);
                }
                _ => {
                    warn!("⚠️ Ollama not accessible");
                    let mut router = self.router.write().await;
                    router.update_model_status("llama2".to_string(), false);
                }
            }
        }
    }

    pub async fn health_check(&self) -> AppResult<serde_json::Value> {
        let mut status = serde_json::json!({
            "overall_status": "healthy",
            "api_providers": {},
            "local_providers": {},
            "models_available": 0,
            "default_model": "x-ai/grok-4-fast:free",
            "issues": []
        });

        let mut api_providers = serde_json::Map::new();
        let mut local_providers = serde_json::Map::new();
        let mut available_models = 0;

        // Check each provider
        for (provider_type, provider) in &self.providers {
            let is_healthy = provider.health_check().await.unwrap_or(false);

            match provider_type {
                ModelProvider::Ollama => {
                    local_providers.insert("ollama".to_string(), serde_json::Value::Bool(is_healthy));
                    if is_healthy {
                        available_models += 1;
                    }
                }
                _ => {
                    api_providers.insert(provider_type.as_str().to_string(), serde_json::Value::Bool(is_healthy));
                    if is_healthy {
                        available_models += 1;
                    }
                }
            }
        }

        status["api_providers"] = serde_json::Value::Object(api_providers);
        status["local_providers"] = serde_json::Value::Object(local_providers);
        status["models_available"] = serde_json::Value::Number(available_models.into());

        // Determine overall health
        if available_models == 0 {
            status["overall_status"] = serde_json::Value::String("unhealthy".to_string());
            status["issues"]
                .as_array_mut()
                .unwrap()
                .push(serde_json::Value::String("No working models available".to_string()));
        }

        Ok(status)
    }

    pub async fn get_stats(&self) -> serde_json::Value {
        let request_stats = self.request_stats.read().await;
        let cost_tracking = self.cost_tracking.read().await;
        let router = self.router.read().await;

        serde_json::json!({
            "request_stats": *request_stats,
            "cost_tracking": *cost_tracking,
            "router_stats": router.get_usage_stats()
        })
    }

    async fn update_stats(&self, model_name: &str, response: &LLMResponse) {
        let mut request_stats = self.request_stats.write().await;
        let mut cost_tracking = self.cost_tracking.write().await;

        *request_stats.entry(model_name.to_string()).or_insert(0) += 1;
        *cost_tracking.entry(model_name.to_string()).or_insert(0.0) += response.cost_estimate;
    }

    fn create_model_config_for_override(&self, model_name: &str) -> ModelConfig {
        // Create a basic model config for override models (primarily for OpenRouter dynamic models)
        ModelConfig {
            name: model_name.to_string(),
            display_name: model_name.split('/').last().unwrap_or(model_name).to_string(),
            icon: "🤖".to_string(),
            provider: ModelProvider::OpenRouter, // Assume OpenRouter for dynamic models
            tier: crate::models::ModelTier::Standard,
            capabilities: vec![
                crate::models::ModelCapability::Conversation,
                crate::models::ModelCapability::Reasoning,
            ],
            metrics: crate::models::ModelMetrics {
                avg_latency_ms: 1000.0,
                tokens_per_second: 50.0,
                cost_per_1k_tokens: 0.001,
                context_window: 8000,
                max_output_tokens: 4096,
                quality_score: 0.75,
                reliability_score: 0.80,
            },
            api_endpoint: "https://openrouter.ai/api/v1/chat/completions".to_string(),
            api_key_env: Some("OPENROUTER_API_KEY".to_string()),
            description: format!("Dynamic model: {}", model_name),
            enabled: true,
            supports_reasoning: model_name.contains("deepseek-r1") || model_name.contains("reasoning"),
            is_popular: false,
            created: None,
        }
    }

    pub async fn quick_generate(&self, prompt: &str, concise: bool) -> AppResult<String> {
        let request = LLMRequest {
            prompt: prompt.to_string(),
            system_prompt: None,
            messages: None,
            max_tokens: if concise { Some(150) } else { None },
            temperature: Some(if concise { 0.5 } else { 0.8 }),
            top_p: None,
            stop_sequences: None,
            tools: None,
            model_override: None,
            concise,
            max_chars: if concise { Some(100) } else { None },
            reasoning: None,
        };

        let requirements = TaskRequirements {
            optimization_goal: if concise { OptimizationGoal::Speed } else { OptimizationGoal::Balanced },
            ..TaskRequirements::default()
        };

        let response = self.generate(&request, Some(requirements)).await?;
        Ok(response.content)
    }

    pub fn is_model_available(&self, model_name: &str, api_keys: &ApiKeys) -> bool {
        // Check if model is a known static model or a dynamic OpenRouter model
        if model_name == "x-ai/grok-4-fast:free" {
            // This is our default free model - available if we have OpenRouter key or if it's truly free
            api_keys.openrouter.is_some()
        } else if model_name.contains("/") || model_name.contains(":") {
            // Dynamic OpenRouter model
            api_keys.openrouter.is_some()
        } else {
            // For other models, we'd need to check against a model registry
            false
        }
    }

    pub fn get_model_availability_reason(&self, model_name: &str, api_keys: &ApiKeys) -> String {
        if !self.is_model_available(model_name, api_keys) {
            if model_name.contains("/") || model_name.contains(":") {
                if api_keys.openrouter.is_none() {
                    format!("Model '{}' requires OPENROUTER_API_KEY to be set.", model_name)
                } else {
                    format!("Model '{}' is temporarily unavailable.", model_name)
                }
            } else {
                format!("Unknown model '{}'.", model_name)
            }
        } else {
            format!("Model '{}' is available.", model_name)
        }
    }
}