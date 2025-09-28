use crate::{
    error::{AppError, AppResult},
    models::{LLMRequest, LLMResponse, ModelConfig, ModelProvider},
};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Instant;

#[async_trait]
pub trait Provider: Send + Sync {
    async fn generate(&self, request: &LLMRequest, config: &ModelConfig) -> AppResult<LLMResponse>;
    fn provider_type(&self) -> ModelProvider;
    async fn health_check(&self) -> AppResult<bool>;
}

pub struct OpenAIProvider {
    client: Client,
    api_key: String,
}

impl OpenAIProvider {
    pub fn new(api_key: String) -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", api_key).parse().unwrap(),
        );

        let client = Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .unwrap();

        Self { client, api_key }
    }
}

#[async_trait]
impl Provider for OpenAIProvider {
    async fn generate(&self, request: &LLMRequest, config: &ModelConfig) -> AppResult<LLMResponse> {
        let start = Instant::now();

        let mut messages = Vec::new();
        
        // Add system prompt if provided
        if let Some(system_prompt) = &request.system_prompt {
            messages.push(json!({
                "role": "system",
                "content": system_prompt
            }));
        }

        // Add conversation history or just the prompt
        if let Some(history) = &request.messages {
            for msg in history {
                messages.push(json!(msg));
            }
        } else {
            messages.push(json!({
                "role": "user", 
                "content": request.prompt
            }));
        }

        let mut payload = json!({
            "model": config.name,
            "messages": messages,
            "max_tokens": request.max_tokens.unwrap_or(config.metrics.max_output_tokens),
            "temperature": request.temperature.unwrap_or(0.8)
        });

        if let Some(tools) = &request.tools {
            payload["tools"] = json!(tools);
            payload["tool_choice"] = json!("auto");
        }

        if let Some(stop_sequences) = &request.stop_sequences {
            payload["stop"] = json!(stop_sequences);
        }

        let response = self.client
            .post(&config.api_endpoint)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(AppError::LLMRequestFailed(format!(
                "OpenAI API error {}: {}", 
                status, 
                error_text
            )));
        }

        let data: Value = response.json().await?;
        let choice = &data["choices"][0];
        let usage = data.get("usage").cloned().unwrap_or_else(|| json!({}));

        let input_tokens = usage["prompt_tokens"].as_u64().unwrap_or(0) as u32;
        let output_tokens = usage["completion_tokens"].as_u64().unwrap_or(0) as u32;
        let total_tokens = input_tokens + output_tokens;

        let cost_estimate = (total_tokens as f64) * config.metrics.cost_per_1k_tokens / 1000.0;

        let mut usage_map = HashMap::new();
        usage_map.insert("prompt_tokens".to_string(), input_tokens);
        usage_map.insert("completion_tokens".to_string(), output_tokens);
        usage_map.insert("total_tokens".to_string(), total_tokens);

        let latency_ms = start.elapsed().as_millis() as f64;

        Ok(LLMResponse {
            content: choice["message"]["content"].as_str().unwrap_or("").to_string(),
            model_used: config.name.clone(),
            usage: usage_map,
            latency_ms,
            cost_estimate,
            finish_reason: choice["finish_reason"].as_str().unwrap_or("stop").to_string(),
            tool_calls: choice["message"]["tool_calls"].as_array().map(|calls| {
                calls.iter().cloned().collect()
            }),
            metadata: Some({
                let mut meta = HashMap::new();
                meta.insert("provider".to_string(), json!("openai"));
                meta.insert("model_config".to_string(), json!(config.name));
                meta
            }),
            reasoning: None,
            reasoning_details: None,
        })
    }

    fn provider_type(&self) -> ModelProvider {
        ModelProvider::OpenAI
    }

    async fn health_check(&self) -> AppResult<bool> {
        // Simple health check - attempt to list models
        let response = self.client
            .get("https://api.openai.com/v1/models")
            .send()
            .await?;
        Ok(response.status().is_success())
    }
}

pub struct AnthropicProvider {
    client: Client,
    api_key: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("x-api-key", api_key.parse().unwrap());
        headers.insert("anthropic-version", "2023-06-01".parse().unwrap());

        let client = Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .unwrap();

        Self { client, api_key }
    }
}

#[async_trait]
impl Provider for AnthropicProvider {
    async fn generate(&self, request: &LLMRequest, config: &ModelConfig) -> AppResult<LLMResponse> {
        let start = Instant::now();

        let mut messages = Vec::new();
        
        // Handle conversation history, filtering out system messages
        if let Some(history) = &request.messages {
            for msg in history {
                if let Some(role_value) = msg.get("role") {
                    if role_value.as_str() != "system" {
                        messages.push(json!(msg));
                    }
                }
            }
        } else {
            messages.push(json!({
                "role": "user",
                "content": request.prompt
            }));
        }

        let mut payload = json!({
            "model": config.name,
            "max_tokens": request.max_tokens.unwrap_or(config.metrics.max_output_tokens),
            "temperature": request.temperature.unwrap_or(0.8),
            "messages": messages
        });

        if let Some(system_prompt) = &request.system_prompt {
            payload["system"] = json!(system_prompt);
        }

        if let Some(stop_sequences) = &request.stop_sequences {
            payload["stop_sequences"] = json!(stop_sequences);
        }

        let response = self.client
            .post(&config.api_endpoint)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(AppError::LLMRequestFailed(format!(
                "Anthropic API error {}: {}", 
                status, 
                error_text
            )));
        }

        let data: Value = response.json().await?;
        
        let content = data["content"][0]["text"].as_str().unwrap_or("").to_string();
        let usage = data.get("usage").cloned().unwrap_or_else(|| json!({}));

        let input_tokens = usage["input_tokens"].as_u64().unwrap_or(0) as u32;
        let output_tokens = usage["output_tokens"].as_u64().unwrap_or(0) as u32;
        let total_tokens = input_tokens + output_tokens;

        let cost_estimate = (total_tokens as f64) * config.metrics.cost_per_1k_tokens / 1000.0;

        let mut usage_map = HashMap::new();
        usage_map.insert("prompt_tokens".to_string(), input_tokens);
        usage_map.insert("completion_tokens".to_string(), output_tokens);
        usage_map.insert("total_tokens".to_string(), total_tokens);

        let latency_ms = start.elapsed().as_millis() as f64;

        Ok(LLMResponse {
            content,
            model_used: config.name.clone(),
            usage: usage_map,
            latency_ms,
            cost_estimate,
            finish_reason: data.get("stop_reason").and_then(|v| v.as_str()).unwrap_or("stop").to_string(),
            tool_calls: None,
            metadata: Some({
                let mut meta = HashMap::new();
                meta.insert("provider".to_string(), json!("anthropic"));
                meta.insert("model_config".to_string(), json!(config.name));
                meta
            }),
            reasoning: None,
            reasoning_details: None,
        })
    }

    fn provider_type(&self) -> ModelProvider {
        ModelProvider::Anthropic
    }

    async fn health_check(&self) -> AppResult<bool> {
        // Anthropic doesn't have a simple health check endpoint, so we'll return true if we have an API key
        Ok(!self.api_key.is_empty())
    }
}

pub struct GroqProvider {
    client: Client,
}

impl GroqProvider {
    pub fn new(api_key: String) -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", api_key).parse().unwrap(),
        );

        let client = Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .unwrap();

        Self { client }
    }
}

#[async_trait]
impl Provider for GroqProvider {
    async fn generate(&self, request: &LLMRequest, config: &ModelConfig) -> AppResult<LLMResponse> {
        // Groq uses OpenAI-compatible API, so we can reuse the OpenAI implementation logic
        let start = Instant::now();

        let mut messages = Vec::new();
        
        if let Some(system_prompt) = &request.system_prompt {
            messages.push(json!({
                "role": "system",
                "content": system_prompt
            }));
        }

        if let Some(history) = &request.messages {
            for msg in history {
                messages.push(json!(msg));
            }
        } else {
            messages.push(json!({
                "role": "user", 
                "content": request.prompt
            }));
        }

        let payload = json!({
            "model": config.name,
            "messages": messages,
            "max_tokens": request.max_tokens.unwrap_or(config.metrics.max_output_tokens),
            "temperature": request.temperature.unwrap_or(0.8)
        });

        let response = self.client
            .post(&config.api_endpoint)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(AppError::LLMRequestFailed(format!(
                "Groq API error {}: {}", 
                status, 
                error_text
            )));
        }

        let data: Value = response.json().await?;
        let choice = &data["choices"][0];
        let usage = data.get("usage").cloned().unwrap_or_else(|| json!({}));

        let input_tokens = usage["prompt_tokens"].as_u64().unwrap_or(0) as u32;
        let output_tokens = usage["completion_tokens"].as_u64().unwrap_or(0) as u32;
        let total_tokens = input_tokens + output_tokens;

        let cost_estimate = (total_tokens as f64) * config.metrics.cost_per_1k_tokens / 1000.0;

        let mut usage_map = HashMap::new();
        usage_map.insert("prompt_tokens".to_string(), input_tokens);
        usage_map.insert("completion_tokens".to_string(), output_tokens);
        usage_map.insert("total_tokens".to_string(), total_tokens);

        let latency_ms = start.elapsed().as_millis() as f64;

        Ok(LLMResponse {
            content: choice["message"]["content"].as_str().unwrap_or("").to_string(),
            model_used: config.name.clone(),
            usage: usage_map,
            latency_ms,
            cost_estimate,
            finish_reason: choice["finish_reason"].as_str().unwrap_or("stop").to_string(),
            tool_calls: None,
            metadata: Some({
                let mut meta = HashMap::new();
                meta.insert("provider".to_string(), json!("groq"));
                meta.insert("model_config".to_string(), json!(config.name));
                meta
            }),
            reasoning: None,
            reasoning_details: None,
        })
    }

    fn provider_type(&self) -> ModelProvider {
        ModelProvider::Groq
    }

    async fn health_check(&self) -> AppResult<bool> {
        let response = self.client
            .get("https://api.groq.com/openai/v1/models")
            .send()
            .await?;
        Ok(response.status().is_success())
    }
}

pub struct OllamaProvider {
    client: Client,
    base_url: String,
}

impl OllamaProvider {
    pub fn new(base_url: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .unwrap();

        Self {
            client,
            base_url: base_url.unwrap_or_else(|| {
                std::env::var("OLLAMA_BASE_URL")
                    .unwrap_or_else(|_| "http://localhost:11434".to_string())
            }),
        }
    }
}

#[async_trait]
impl Provider for OllamaProvider {
    async fn generate(&self, request: &LLMRequest, config: &ModelConfig) -> AppResult<LLMResponse> {
        let start = Instant::now();

        let mut payload = json!({
            "model": config.name,
            "prompt": request.prompt,
            "stream": false,
            "options": {
                "temperature": request.temperature.unwrap_or(0.8),
                "num_predict": request.max_tokens.unwrap_or(config.metrics.max_output_tokens)
            }
        });

        if let Some(system_prompt) = &request.system_prompt {
            payload["system"] = json!(system_prompt);
        }

        let url = format!("{}/api/generate", self.base_url);
        let response = self.client
            .post(&url)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(AppError::LLMRequestFailed(format!(
                "Ollama API error {}: {}", 
                status, 
                error_text
            )));
        }

        let data: Value = response.json().await?;
        let content = data["response"].as_str().unwrap_or("").to_string();

        // Ollama doesn't provide detailed usage stats, so we estimate
        let estimated_tokens = (content.len() / 4) as u32; // Rough estimation

        let mut usage_map = HashMap::new();
        usage_map.insert("total_tokens".to_string(), estimated_tokens);

        let latency_ms = start.elapsed().as_millis() as f64;

        Ok(LLMResponse {
            content,
            model_used: config.name.clone(),
            usage: usage_map,
            latency_ms,
            cost_estimate: 0.0, // Local models are free
            finish_reason: "stop".to_string(),
            tool_calls: None,
            metadata: Some({
                let mut meta = HashMap::new();
                meta.insert("provider".to_string(), json!("ollama"));
                meta.insert("model_config".to_string(), json!(config.name));
                meta
            }),
            reasoning: None,
            reasoning_details: None,
        })
    }

    fn provider_type(&self) -> ModelProvider {
        ModelProvider::Ollama
    }

    async fn health_check(&self) -> AppResult<bool> {
        let url = format!("{}/api/tags", self.base_url);
        let response = self.client
            .get(&url)
            .send()
            .await?;
        Ok(response.status().is_success())
    }
}

pub struct OpenRouterProvider {
    client: Client,
}

impl OpenRouterProvider {
    pub fn new(api_key: String) -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", api_key).parse().unwrap(),
        );
        headers.insert("HTTP-Referer", "https://nullblock.ai".parse().unwrap());
        headers.insert("X-Title", "NullBlock Agent Platform".parse().unwrap());

        let client = Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .unwrap();

        Self { client }
    }
}

#[async_trait]
impl Provider for OpenRouterProvider {
    async fn generate(&self, request: &LLMRequest, config: &ModelConfig) -> AppResult<LLMResponse> {
        let start = Instant::now();

        let mut messages = Vec::new();
        
        if let Some(system_prompt) = &request.system_prompt {
            messages.push(json!({
                "role": "system",
                "content": system_prompt
            }));
        }

        if let Some(history) = &request.messages {
            for msg in history {
                messages.push(json!(msg));
            }
        } else {
            messages.push(json!({
                "role": "user", 
                "content": request.prompt
            }));
        }

        let mut payload = json!({
            "model": config.name,
            "messages": messages,
            "max_tokens": request.max_tokens.unwrap_or(config.metrics.max_output_tokens),
            "temperature": request.temperature.unwrap_or(0.8)
        });

        // Add reasoning configuration if supported and requested
        if let Some(reasoning) = &request.reasoning {
            if config.supports_reasoning && reasoning.enabled {
                let mut reasoning_payload = json!({
                    "enabled": true
                });
                
                if let Some(effort) = &reasoning.effort {
                    reasoning_payload["effort"] = json!(effort);
                } else if let Some(max_tokens) = reasoning.max_tokens {
                    reasoning_payload["max_tokens"] = json!(max_tokens);
                }
                
                if reasoning.exclude {
                    reasoning_payload["exclude"] = json!(true);
                }
                
                payload["reasoning"] = reasoning_payload;
            }
        }

        let response = self.client
            .post(&config.api_endpoint)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(AppError::LLMRequestFailed(format!(
                "OpenRouter API error {}: {}", 
                status, 
                error_text
            )));
        }

        let data: Value = response.json().await?;
        let choice = &data["choices"][0];
        let usage = data.get("usage").cloned().unwrap_or_else(|| json!({}));

        let input_tokens = usage["prompt_tokens"].as_u64().unwrap_or(0) as u32;
        let output_tokens = usage["completion_tokens"].as_u64().unwrap_or(0) as u32;
        let total_tokens = input_tokens + output_tokens;

        let cost_estimate = (total_tokens as f64) * config.metrics.cost_per_1k_tokens / 1000.0;

        let mut usage_map = HashMap::new();
        usage_map.insert("prompt_tokens".to_string(), input_tokens);
        usage_map.insert("completion_tokens".to_string(), output_tokens);
        usage_map.insert("total_tokens".to_string(), total_tokens);

        let latency_ms = start.elapsed().as_millis() as f64;

        // Extract reasoning information if available
        let reasoning = choice["message"]["reasoning"].as_str().map(|s| s.to_string());
        let reasoning_details = choice["message"]["reasoning_details"].as_array().cloned();

        Ok(LLMResponse {
            content: choice["message"]["content"].as_str().unwrap_or("").to_string(),
            model_used: config.name.clone(),
            usage: usage_map,
            latency_ms,
            cost_estimate,
            finish_reason: choice["finish_reason"].as_str().unwrap_or("stop").to_string(),
            tool_calls: choice["message"]["tool_calls"].as_array().map(|calls| {
                calls.iter().cloned().collect()
            }),
            metadata: Some({
                let mut meta = HashMap::new();
                meta.insert("provider".to_string(), json!("openrouter"));
                meta.insert("model_config".to_string(), json!(config.name));
                meta
            }),
            reasoning,
            reasoning_details,
        })
    }

    fn provider_type(&self) -> ModelProvider {
        ModelProvider::OpenRouter
    }

    async fn health_check(&self) -> AppResult<bool> {
        let response = self.client
            .get("https://openrouter.ai/api/v1/models")
            .send()
            .await?;
        Ok(response.status().is_success())
    }
}