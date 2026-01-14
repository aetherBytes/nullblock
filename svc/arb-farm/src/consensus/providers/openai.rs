use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::error::{AppError, AppResult};

pub struct OpenAIClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl OpenAIClient {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.into(),
            base_url: "https://api.openai.com/v1".to_string(),
        }
    }

    pub async fn query(
        &self,
        prompt: &str,
        system_prompt: Option<&str>,
        model: &str,
        max_tokens: u32,
    ) -> AppResult<OpenAIResponse> {
        let start = Instant::now();

        let mut messages = Vec::new();

        if let Some(sys) = system_prompt {
            messages.push(ChatMessage {
                role: "system".to_string(),
                content: sys.to_string(),
            });
        }

        messages.push(ChatMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        });

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
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("OpenAI request failed: {}", e)))?;

        let latency_ms = start.elapsed().as_millis() as u64;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AppError::ExternalApi(format!(
                "OpenAI API error ({}): {}",
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

        Ok(OpenAIResponse {
            content,
            latency_ms,
            model: model.to_string(),
            total_tokens: chat_response.usage.map(|u| u.total_tokens).unwrap_or(0),
        })
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
pub struct OpenAIResponse {
    pub content: String,
    pub latency_ms: u64,
    pub model: String,
    pub total_tokens: u32,
}
