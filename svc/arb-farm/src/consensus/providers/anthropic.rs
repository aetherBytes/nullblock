use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::error::{AppError, AppResult};

pub struct AnthropicClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl AnthropicClient {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.into(),
            base_url: "https://api.anthropic.com/v1".to_string(),
        }
    }

    pub async fn query(
        &self,
        prompt: &str,
        system_prompt: Option<&str>,
        max_tokens: u32,
    ) -> AppResult<AnthropicResponse> {
        let start = Instant::now();

        let request = MessageRequest {
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens,
            system: system_prompt.map(|s| s.to_string()),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
        };

        let response = self
            .client
            .post(format!("{}/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Anthropic request failed: {}", e)))?;

        let latency_ms = start.elapsed().as_millis() as u64;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AppError::ExternalApi(format!(
                "Anthropic API error ({}): {}",
                status, body
            )));
        }

        let message_response: MessageResponse = response
            .json()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse response: {}", e)))?;

        let content = message_response
            .content
            .first()
            .map(|c| c.text.clone())
            .unwrap_or_default();

        Ok(AnthropicResponse {
            content,
            latency_ms,
            model: message_response.model,
            input_tokens: message_response.usage.input_tokens,
            output_tokens: message_response.usage.output_tokens,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
struct MessageRequest {
    model: String,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    messages: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Clone, Deserialize)]
struct MessageResponse {
    content: Vec<ContentBlock>,
    model: String,
    usage: UsageInfo,
}

#[derive(Debug, Clone, Deserialize)]
struct ContentBlock {
    text: String,
}

#[derive(Debug, Clone, Deserialize)]
struct UsageInfo {
    input_tokens: u32,
    output_tokens: u32,
}

#[derive(Debug, Clone)]
pub struct AnthropicResponse {
    pub content: String,
    pub latency_ms: u64,
    pub model: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
}
