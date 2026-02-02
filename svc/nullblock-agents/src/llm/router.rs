#![allow(dead_code)]

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

impl TaskRequirements {
    pub fn for_image_generation() -> Self {
        Self {
            required_capabilities: vec![
                ModelCapability::ImageGeneration,
                ModelCapability::Creative,
            ],
            optimization_goal: OptimizationGoal::Quality,
            priority: Priority::High,
            task_type: "image_generation".to_string(),
            allow_local_models: false,
            preferred_providers: vec!["openrouter".to_string()],
            min_quality_score: Some(0.8),
            max_cost_per_1k_tokens: Some(5.0), // Increased for better image models
            min_context_window: Some(1000),
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

    pub async fn route_request(
        &self,
        requirements: &TaskRequirements,
    ) -> AppResult<RoutingDecision> {
        let available_models = self.get_available_models(requirements)?;

        if available_models.is_empty() {
            return Err(AppError::ModelNotAvailable(
                "No models available for the specified requirements".to_string(),
            ));
        }

        let (selected_model, confidence, reasoning) =
            self.select_optimal_model_with_confidence(&available_models, requirements)?;

        Ok(RoutingDecision {
            selected_model: selected_model.name.clone(),
            model_config: selected_model.clone(),
            confidence,
            reasoning,
            alternatives: available_models
                .into_iter()
                .map(|m| m.name)
                .take(3)
                .collect(),
            estimated_cost: selected_model.metrics.cost_per_1k_tokens,
            estimated_latency_ms: if selected_model.metrics.tokens_per_second > 0.0 {
                1000.0 / selected_model.metrics.tokens_per_second
            } else {
                1000.0
            },
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
            if self.is_model_available(&model.name) && self.meets_requirements(&model, requirements)
            {
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

    fn select_optimal_model(
        &self,
        available_models: &[ModelConfig],
        requirements: &TaskRequirements,
    ) -> AppResult<ModelConfig> {
        if available_models.is_empty() {
            return Err(AppError::ModelNotAvailable(
                "No available models".to_string(),
            ));
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

    fn select_optimal_model_with_confidence(
        &self,
        available_models: &[ModelConfig],
        requirements: &TaskRequirements,
    ) -> AppResult<(ModelConfig, f64, Vec<String>)> {
        if available_models.is_empty() {
            return Err(AppError::ModelNotAvailable(
                "No available models".to_string(),
            ));
        }

        let mut scored_models: Vec<(f64, &ModelConfig)> = available_models
            .iter()
            .map(|model| (self.calculate_model_score(model, requirements), model))
            .collect();

        // Sort by score (higher is better)
        scored_models.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        let best_model = scored_models[0].1.clone();
        let best_score = scored_models[0].0;

        // Calculate confidence based on:
        // 1. Score separation from next best model (higher separation = higher confidence)
        // 2. Number of alternatives (fewer alternatives = higher confidence in selection)
        // 3. Absolute score value (higher score = higher confidence)

        let mut confidence: f64 = 0.5; // Base confidence
        let mut reasoning = Vec::new();

        // Factor 1: Score separation (0.0 - 0.3 confidence)
        if scored_models.len() > 1 {
            let second_score = scored_models[1].0;
            let score_gap = (best_score - second_score) / best_score.max(1.0);

            if score_gap > 0.3 {
                confidence += 0.3;
                reasoning.push(format!(
                    "Strong lead over alternatives ({:.1}% better)",
                    score_gap * 100.0
                ));
            } else if score_gap > 0.15 {
                confidence += 0.2;
                reasoning.push(format!(
                    "Moderate lead over alternatives ({:.1}% better)",
                    score_gap * 100.0
                ));
            } else {
                confidence += 0.1;
                reasoning.push(format!(
                    "Close competition with alternatives ({:.1}% better)",
                    score_gap * 100.0
                ));
            }
        } else {
            confidence += 0.3;
            reasoning.push("Only available model matching requirements".to_string());
        }

        // Factor 2: Number of alternatives (0.0 - 0.1 confidence)
        if available_models.len() <= 3 {
            confidence += 0.1;
            reasoning.push(format!(
                "Limited alternatives ({} models)",
                available_models.len()
            ));
        } else {
            confidence += 0.05;
        }

        // Factor 3: Absolute score quality (0.0 - 0.1 confidence)
        if best_score > 80.0 {
            confidence += 0.1;
            reasoning.push("High quality score".to_string());
        } else if best_score > 60.0 {
            confidence += 0.05;
        }

        // Cap confidence at 1.0
        confidence = confidence.min(1.0);

        reasoning.insert(
            0,
            format!("Selected {} with score {:.1}", best_model.name, best_score),
        );

        Ok((best_model, confidence, reasoning))
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
            }
            OptimizationGoal::Quality => {
                score += model.metrics.quality_score * 30.0;
            }
            OptimizationGoal::Speed => {
                score += (model.metrics.tokens_per_second / 100.0).min(30.0);
            }
            OptimizationGoal::Balanced => {
                // Already included quality and reliability above
                if model.metrics.cost_per_1k_tokens == 0.0 {
                    score += 10.0;
                }
                score += (model.metrics.tokens_per_second / 200.0).min(10.0);
            }
            OptimizationGoal::Reliability => {
                score += model.metrics.reliability_score * 30.0;
            }
        }

        // Tier bonuses
        match model.tier {
            ModelTier::Premium => score += 10.0,
            ModelTier::Standard => score += 5.0,
            ModelTier::Fast => score += 3.0,
            ModelTier::Free => score += 1.0,
        }

        // Provider preferences
        if requirements
            .preferred_providers
            .contains(&model.provider.as_str().to_string())
        {
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
        Some(vec![
            "cognitivecomputations/dolphin3.0-mistral-24b:free".to_string(),
            "cognitivecomputations/dolphin3.0-r1-mistral-24b:free".to_string(),
            "deepseek/deepseek-chat-v3.1:free".to_string(),
            "nvidia/nemotron-nano-9b-v2:free".to_string(),
        ])
    }

    fn get_static_models(&self) -> Vec<ModelConfig> {
        vec![
            ModelConfig {
                name: "cognitivecomputations/dolphin3.0-mistral-24b:free".to_string(),
                display_name: "Dolphin 3.0 Mistral 24B Free".to_string(),
                icon: "🐬".to_string(),
                provider: ModelProvider::OpenRouter,
                tier: ModelTier::Free,
                capabilities: vec![
                    ModelCapability::Conversation,
                    ModelCapability::Reasoning,
                    ModelCapability::Creative,
                    ModelCapability::FunctionCalling,
                ],
                metrics: crate::models::ModelMetrics {
                    avg_latency_ms: 900.0,
                    tokens_per_second: 60.0,
                    cost_per_1k_tokens: 0.0,
                    context_window: 32000,
                    max_output_tokens: 8192,
                    quality_score: 0.88,
                    reliability_score: 0.92,
                },
                api_endpoint: "https://openrouter.ai/api/v1/chat/completions".to_string(),
                api_key_env: Some("OPENROUTER_API_KEY".to_string()),
                description: "Ultimate general purpose free model for coding, math, agentic, and function calling".to_string(),
                enabled: true,
                supports_reasoning: false,
                is_popular: true,
                created: None,
            },
            ModelConfig {
                name: "deepseek/deepseek-chat-v3.1:free".to_string(),
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
                    context_window: 163800,
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
            },
            ModelConfig {
                name: "google/gemini-2.5-flash-image-preview".to_string(),
                display_name: "Gemini 2.5 Flash Image".to_string(),
                icon: "🎨".to_string(),
                provider: ModelProvider::OpenRouter,
                tier: ModelTier::Premium,
                capabilities: vec![
                    ModelCapability::ImageGeneration,
                    ModelCapability::Creative,
                    ModelCapability::Conversation,
                    ModelCapability::Vision,
                ],
                metrics: crate::models::ModelMetrics {
                    avg_latency_ms: 3000.0,
                    tokens_per_second: 30.0,
                    cost_per_1k_tokens: 1.5,  // More accurate cost estimate
                    context_window: 1000000,
                    max_output_tokens: 8192,
                    quality_score: 0.95,
                    reliability_score: 0.90,
                },
                api_endpoint: "https://openrouter.ai/api/v1/chat/completions".to_string(),
                api_key_env: Some("OPENROUTER_API_KEY".to_string()),
                description: "Gemini 2.5 Flash Image - Advanced image generation with contextual understanding".to_string(),
                enabled: true,
                supports_reasoning: false,
                is_popular: true,
                created: None,
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_model(
        name: &str,
        provider: ModelProvider,
        quality: f64,
        reliability: f64,
        cost: f64,
    ) -> ModelConfig {
        ModelConfig {
            name: name.to_string(),
            display_name: name.to_string(),
            icon: "🤖".to_string(),
            provider,
            tier: ModelTier::Standard,
            capabilities: vec![ModelCapability::Conversation],
            metrics: crate::models::ModelMetrics {
                context_window: 4096,
                max_output_tokens: 2048,
                quality_score: quality,
                cost_per_1k_tokens: cost,
                tokens_per_second: 50.0,
                reliability_score: reliability,
                avg_latency_ms: 500.0,
            },
            api_endpoint: format!("https://test.com/{}", name),
            api_key_env: None,
            description: format!("Test model {}", name),
            enabled: true,
            supports_reasoning: false,
            is_popular: false,
            created: None,
        }
    }

    #[test]
    fn test_confidence_high_score_gap() {
        let router = ModelRouter::new();
        let models = vec![
            create_test_model("high-model", ModelProvider::OpenRouter, 0.9, 0.95, 0.01),
            create_test_model("low-model", ModelProvider::OpenRouter, 0.5, 0.6, 0.01),
        ];

        let requirements = TaskRequirements::default();
        let (_, confidence, reasoning) = router
            .select_optimal_model_with_confidence(&models, &requirements)
            .unwrap();

        assert!(
            confidence > 0.8,
            "High score gap should yield high confidence"
        );
        assert!(!reasoning.is_empty(), "Should have reasoning");
        assert!(
            reasoning.iter().any(|r| r.contains("Strong lead")),
            "Should mention strong lead in reasoning"
        );
    }

    #[test]
    fn test_confidence_close_competition() {
        let router = ModelRouter::new();
        let models = vec![
            create_test_model("model-a", ModelProvider::OpenRouter, 0.85, 0.9, 0.01),
            create_test_model("model-b", ModelProvider::OpenRouter, 0.83, 0.88, 0.01),
        ];

        let requirements = TaskRequirements::default();
        let (_, confidence, reasoning) = router
            .select_optimal_model_with_confidence(&models, &requirements)
            .unwrap();

        assert!(
            confidence < 0.85,
            "Close competition should yield moderate confidence"
        );
        assert!(
            reasoning
                .iter()
                .any(|r| r.contains("competition") || r.contains("Close")),
            "Should mention close competition"
        );
    }

    #[test]
    fn test_confidence_single_model() {
        let router = ModelRouter::new();
        let models = vec![create_test_model(
            "only-model",
            ModelProvider::OpenRouter,
            0.8,
            0.85,
            0.01,
        )];

        let requirements = TaskRequirements::default();
        let (_, confidence, reasoning) = router
            .select_optimal_model_with_confidence(&models, &requirements)
            .unwrap();

        assert!(
            confidence > 0.7,
            "Single available model should have high confidence"
        );
        assert!(
            reasoning.iter().any(|r| r.contains("Only available")),
            "Should mention being only available model"
        );
    }

    #[test]
    fn test_confidence_bounds() {
        let router = ModelRouter::new();

        let test_cases = vec![
            vec![create_test_model(
                "perfect",
                ModelProvider::OpenRouter,
                1.0,
                1.0,
                0.0,
            )],
            vec![
                create_test_model("good", ModelProvider::OpenRouter, 0.9, 0.9, 0.01),
                create_test_model("bad", ModelProvider::OpenRouter, 0.1, 0.1, 0.01),
            ],
            vec![
                create_test_model("a", ModelProvider::OpenRouter, 0.7, 0.7, 0.01),
                create_test_model("b", ModelProvider::OpenRouter, 0.69, 0.69, 0.01),
                create_test_model("c", ModelProvider::OpenRouter, 0.68, 0.68, 0.01),
            ],
        ];

        for models in test_cases {
            let requirements = TaskRequirements::default();
            let result = router.select_optimal_model_with_confidence(&models, &requirements);

            assert!(result.is_ok(), "Should successfully select model");
            let (_, confidence, _) = result.unwrap();

            assert!(
                confidence >= 0.5 && confidence <= 1.0,
                "Confidence should be in valid range [0.5, 1.0], got {}",
                confidence
            );
        }
    }

    #[test]
    fn test_reasoning_always_provided() {
        let router = ModelRouter::new();
        let models = vec![create_test_model(
            "test",
            ModelProvider::OpenRouter,
            0.8,
            0.8,
            0.01,
        )];

        let requirements = TaskRequirements::default();
        let (_, _, reasoning) = router
            .select_optimal_model_with_confidence(&models, &requirements)
            .unwrap();

        assert!(!reasoning.is_empty(), "Reasoning should always be provided");
        assert!(
            reasoning[0].contains("Selected"),
            "First reasoning should mention selection"
        );
    }
}
