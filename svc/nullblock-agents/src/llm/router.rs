use crate::{
    error::{AppError, AppResult},
    models::{ModelCapability, ModelConfig, ModelProvider, ModelTier},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptimizationGoal {
    Cost,
    Quality,
    Speed,
    Balanced,
    Reliability,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct TaskRequirements {
    pub required_capabilities: Vec<ModelCapability>,
    pub optimization_goal: OptimizationGoal,
    pub priority: Priority,
    pub task_type: String,
    pub allow_local_models: bool,
    pub preferred_providers: Vec<String>,
    pub min_quality_score: Option<f64>,
    pub max_cost_per_1k_tokens: Option<f64>,
    pub min_context_window: Option<u32>,
}

impl Default for TaskRequirements {
    fn default() -> Self {
        Self {
            required_capabilities: vec![ModelCapability::Conversation],
            optimization_goal: OptimizationGoal::Balanced,
            priority: Priority::Medium,
            task_type: "general".to_string(),
            allow_local_models: true,
            preferred_providers: vec!["openrouter".to_string()],
            min_quality_score: None,
            max_cost_per_1k_tokens: None,
            min_context_window: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RoutingDecision {
    pub selected_model: String,
    pub model_config: ModelConfig,
    pub confidence: f64,
    pub reasoning: Vec<String>,
    pub alternatives: Vec<String>,
    pub estimated_cost: f64,
    pub estimated_latency_ms: f64,
    pub fallback_models: Vec<String>,
}

pub struct ModelRouter {
    pub model_status: HashMap<String, bool>,
    pub usage_stats: HashMap<String, usize>,
}

impl ModelRouter {
    pub fn new() -> Self {
        Self {
            model_status: HashMap::new(),
            usage_stats: HashMap::new(),
        }
    }

    pub async fn route_request(&self, requirements: &TaskRequirements) -> AppResult<RoutingDecision> {
        let available_models = self.get_available_models(requirements)?;
        
        if available_models.is_empty() {
            return Err(AppError::ModelNotAvailable(
                "No models available for the specified requirements".to_string()
            ));
        }

        let selected_model = self.select_optimal_model(&available_models, requirements)?;
        
        Ok(RoutingDecision {
            selected_model: selected_model.name.clone(),
            model_config: selected_model,
            confidence: 0.9, // TODO: Calculate based on actual matching criteria
            reasoning: vec!["Model selected based on requirements".to_string()],
            alternatives: available_models.into_iter()
                .map(|m| m.name)
                .take(3)
                .collect(),
            estimated_cost: 0.001, // TODO: Calculate based on model metrics
            estimated_latency_ms: 1000.0, // TODO: Calculate based on model metrics
            fallback_models: self.get_fallback_models().unwrap_or_default(),
        })
    }

    pub fn update_model_status(&mut self, model_name: String, available: bool) {
        self.model_status.insert(model_name, available);
    }

    pub fn get_usage_stats(&self) -> HashMap<String, usize> {
        self.usage_stats.clone()
    }

    fn get_available_models(&self, requirements: &TaskRequirements) -> AppResult<Vec<ModelConfig>> {
        let mut available_models = Vec::new();
        
        // Get static models (this would come from a models registry in a real implementation)
        let static_models = self.get_static_models();
        
        for model in static_models {
            if self.is_model_available(&model.name) && self.meets_requirements(&model, requirements) {
                available_models.push(model);
            }
        }

        Ok(available_models)
    }

    fn is_model_available(&self, model_name: &str) -> bool {
        self.model_status.get(model_name).copied().unwrap_or(true)
    }

    fn meets_requirements(&self, model: &ModelConfig, requirements: &TaskRequirements) -> bool {
        // Check if model has required capabilities
        for required_cap in &requirements.required_capabilities {
            if !model.capabilities.contains(required_cap) {
                return false;
            }
        }

        // Check quality score
        if let Some(min_quality) = requirements.min_quality_score {
            if model.metrics.quality_score < min_quality {
                return false;
            }
        }

        // Check cost limit
        if let Some(max_cost) = requirements.max_cost_per_1k_tokens {
            if model.metrics.cost_per_1k_tokens > max_cost {
                return false;
            }
        }

        // Check context window
        if let Some(min_context) = requirements.min_context_window {
            if model.metrics.context_window < min_context {
                return false;
            }
        }

        // Check if local models are allowed
        if !requirements.allow_local_models && model.provider == ModelProvider::Ollama {
            return false;
        }

        true
    }

    fn select_optimal_model(&self, available_models: &[ModelConfig], requirements: &TaskRequirements) -> AppResult<ModelConfig> {
        if available_models.is_empty() {
            return Err(AppError::ModelNotAvailable("No available models".to_string()));
        }

        let mut scored_models: Vec<(f64, &ModelConfig)> = available_models
            .iter()
            .map(|model| (self.calculate_model_score(model, requirements), model))
            .collect();

        // Sort by score (higher is better)
        scored_models.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        // Return the best model
        Ok(scored_models[0].1.clone())
    }

    fn calculate_model_score(&self, model: &ModelConfig, requirements: &TaskRequirements) -> f64 {
        let mut score = 0.0;

        // Base score from quality and reliability
        score += model.metrics.quality_score * 40.0;
        score += model.metrics.reliability_score * 30.0;

        // Optimization goal adjustments
        match requirements.optimization_goal {
            OptimizationGoal::Cost => {
                // Lower cost is better (invert the cost score)
                if model.metrics.cost_per_1k_tokens == 0.0 {
                    score += 30.0; // Free models get max cost score
                } else {
                    score += (1.0 / model.metrics.cost_per_1k_tokens.max(0.001)) * 30.0;
                }
            },
            OptimizationGoal::Quality => {
                score += model.metrics.quality_score * 30.0;
            },
            OptimizationGoal::Speed => {
                score += (model.metrics.tokens_per_second / 100.0).min(30.0);
            },
            OptimizationGoal::Balanced => {
                // Already included quality and reliability above
                if model.metrics.cost_per_1k_tokens == 0.0 {
                    score += 10.0;
                }
                score += (model.metrics.tokens_per_second / 200.0).min(10.0);
            },
            OptimizationGoal::Reliability => {
                score += model.metrics.reliability_score * 30.0;
            },
        }

        // Tier bonuses
        match model.tier {
            ModelTier::Premium => score += 10.0,
            ModelTier::Standard => score += 5.0,
            ModelTier::Fast => score += 3.0,
            ModelTier::Free => score += 1.0,
        }

        // Provider preferences
        if requirements.preferred_providers.contains(&model.provider.as_str().to_string()) {
            score += 15.0;
        }

        // Popular model bonus
        if model.is_popular {
            score += 5.0;
        }

        // Usage frequency penalty (to encourage model diversity)
        let usage_count = self.usage_stats.get(&model.name).copied().unwrap_or(0);
        score -= (usage_count as f64 * 0.1).min(10.0);

        score.max(0.0)
    }

    fn get_fallback_models(&self) -> Option<Vec<String>> {
        // Return some reliable fallback models
        Some(vec![
            "x-ai/grok-4-fast:free".to_string(),
            "gpt-3.5-turbo".to_string(),
        ])
    }

    fn get_static_models(&self) -> Vec<ModelConfig> {
        // This would normally come from a configuration file or database
        // For now, return the default Hecate model
        vec![ModelConfig {
            name: "x-ai/grok-4-fast:free".to_string(),
            display_name: "DeepSeek Chat v3.1 Free".to_string(),
            icon: "🤖".to_string(),
            provider: ModelProvider::OpenRouter,
            tier: ModelTier::Free,
            capabilities: vec![
                ModelCapability::Conversation,
                ModelCapability::Reasoning,
                ModelCapability::Creative,
            ],
            metrics: crate::models::ModelMetrics {
                avg_latency_ms: 1000.0,
                tokens_per_second: 50.0,
                cost_per_1k_tokens: 0.0,
                context_window: 128000,
                max_output_tokens: 8192,
                quality_score: 0.85,
                reliability_score: 0.90,
            },
            api_endpoint: "https://openrouter.ai/api/v1/chat/completions".to_string(),
            api_key_env: Some("OPENROUTER_API_KEY".to_string()),
            description: "Free DeepSeek model optimized for conversation".to_string(),
            enabled: true,
            supports_reasoning: true,
            is_popular: true,
            created: None,
        }]
    }
}