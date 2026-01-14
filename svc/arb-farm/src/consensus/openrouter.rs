use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::error::{AppError, AppResult};

pub struct OpenRouterClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl OpenRouterClient {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.into(),
            base_url: "https://openrouter.ai/api/v1".to_string(),
        }
    }

    pub async fn query_model(
        &self,
        model: &str,
        prompt: &str,
        system_prompt: Option<&str>,
        max_tokens: u32,
    ) -> AppResult<ModelResponse> {
        let start = Instant::now();

        let messages = if let Some(sys) = system_prompt {
            vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: sys.to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                },
            ]
        } else {
            vec![ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }]
        };

        let request = ChatRequest {
            model: model.to_string(),
            messages,
            max_tokens: Some(max_tokens),
            temperature: Some(0.3),
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("HTTP-Referer", "https://nullblock.io")
            .header("X-Title", "ArbFarm Consensus")
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("OpenRouter request failed: {}", e)))?;

        let latency_ms = start.elapsed().as_millis() as u64;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AppError::ExternalApi(format!(
                "OpenRouter API error ({}): {}",
                status, body
            )));
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse response: {}", e)))?;

        let content = chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(ModelResponse {
            model: model.to_string(),
            content,
            latency_ms,
            tokens_used: chat_response.usage.map(|u| u.total_tokens).unwrap_or(0),
        })
    }

    pub async fn query_structured<T: for<'de> Deserialize<'de>>(
        &self,
        model: &str,
        prompt: &str,
        system_prompt: Option<&str>,
    ) -> AppResult<(T, u64)> {
        let response = self
            .query_model(model, prompt, system_prompt, 2048)
            .await?;

        let json_start = response.content.find('{');
        let json_end = response.content.rfind('}');

        let json_str = match (json_start, json_end) {
            (Some(start), Some(end)) => &response.content[start..=end],
            _ => {
                return Err(AppError::ExternalApi(
                    "No JSON found in model response".to_string(),
                ))
            }
        };

        let parsed: T = serde_json::from_str(json_str)
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse JSON response: {}", e)))?;

        Ok((parsed, response.latency_ms))
    }
}

#[derive(Debug, Clone, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
    usage: Option<Usage>,
}

#[derive(Debug, Clone, Deserialize)]
struct Choice {
    message: ChatMessage,
}

#[derive(Debug, Clone, Deserialize)]
struct Usage {
    total_tokens: u32,
}

#[derive(Debug, Clone)]
pub struct ModelResponse {
    pub model: String,
    pub content: String,
    pub latency_ms: u64,
    pub tokens_used: u32,
}

pub const AVAILABLE_MODELS: &[(&str, &str, f64)] = &[
    ("anthropic/claude-3.5-sonnet", "Claude 3.5 Sonnet", 1.5),
    ("openai/gpt-4-turbo", "GPT-4 Turbo", 1.0),
    ("meta-llama/llama-3.1-70b-instruct", "Llama 3.1 70B", 0.8),
    ("google/gemini-pro-1.5", "Gemini Pro 1.5", 0.9),
    ("mistralai/mistral-large", "Mistral Large", 0.7),
];

pub fn get_default_models() -> Vec<String> {
    vec![
        "anthropic/claude-3.5-sonnet".to_string(),
        "openai/gpt-4-turbo".to_string(),
        "meta-llama/llama-3.1-70b-instruct".to_string(),
    ]
}

pub fn get_model_weight(model: &str) -> f64 {
    AVAILABLE_MODELS
        .iter()
        .find(|(id, _, _)| *id == model)
        .map(|(_, _, weight)| *weight)
        .unwrap_or(1.0)
}
