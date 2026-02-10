#![allow(dead_code)]

use crate::{
    error::{AppError, AppResult},
    models::{LLMRequest, LLMResponse, ModelConfig, ModelProvider},
};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Instant;
use tracing::{info, warn};

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

        let response = self
            .client
            .post(&config.api_endpoint)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(AppError::LLMRequestFailed(format!(
                "OpenAI API error {}: {}",
                status, error_text
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
            content: choice["message"]["content"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            model_used: config.name.clone(),
            usage: usage_map,
            latency_ms,
            cost_estimate,
            finish_reason: choice["finish_reason"]
                .as_str()
                .unwrap_or("stop")
                .to_string(),
            confidence_score: 1.0,
            tool_calls: choice["message"]["tool_calls"]
                .as_array()
                .map(|calls| calls.iter().cloned().collect()),
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
        let response = self
            .client
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
        headers.insert("anthropic-version", "2024-04-04".parse().unwrap());

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
                    if role_value.as_str() != Some("system") {
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

        if let Some(tools) = &request.tools {
            let anthropic_tools: Vec<Value> = tools
                .iter()
                .filter_map(|tool| {
                    let func = tool.get("function")?;
                    Some(json!({
                        "name": func["name"],
                        "description": func["description"],
                        "input_schema": func["parameters"]
                    }))
                })
                .collect();
            if !anthropic_tools.is_empty() {
                payload["tools"] = json!(anthropic_tools);
            }
        }

        if let Some(stop_sequences) = &request.stop_sequences {
            payload["stop_sequences"] = json!(stop_sequences);
        }

        let response = self
            .client
            .post(&config.api_endpoint)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(AppError::LLMRequestFailed(format!(
                "Anthropic API error {}: {}",
                status, error_text
            )));
        }

        let data: Value = response.json().await?;

        let content = data["content"]
            .as_array()
            .and_then(|blocks| {
                blocks
                    .iter()
                    .find(|b| b["type"] == "text")
                    .and_then(|b| b["text"].as_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_default();

        let tool_calls: Option<Vec<Value>> =
            data["content"]
                .as_array()
                .map(|blocks| {
                    blocks.iter().filter(|b| b["type"] == "tool_use").map(|b| {
                json!({
                    "id": b["id"],
                    "type": "function",
                    "function": {
                        "name": b["name"],
                        "arguments": serde_json::to_string(&b["input"]).unwrap_or_default()
                    }
                })
            }).collect()
                })
                .and_then(|v: Vec<Value>| if v.is_empty() { None } else { Some(v) });

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
            finish_reason: data
                .get("stop_reason")
                .and_then(|v| v.as_str())
                .unwrap_or("stop")
                .to_string(),
            confidence_score: 1.0,
            tool_calls,
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

        let response = self
            .client
            .post(&config.api_endpoint)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(AppError::LLMRequestFailed(format!(
                "Groq API error {}: {}",
                status, error_text
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
            content: choice["message"]["content"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            model_used: config.name.clone(),
            usage: usage_map,
            latency_ms,
            cost_estimate,
            finish_reason: choice["finish_reason"]
                .as_str()
                .unwrap_or("stop")
                .to_string(),
            confidence_score: 1.0,
            tool_calls: choice["message"]["tool_calls"]
                .as_array()
                .map(|calls| calls.iter().cloned().collect()),
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
        let response = self
            .client
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
        let response = self.client.post(&url).json(&payload).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(AppError::LLMRequestFailed(format!(
                "Ollama API error {}: {}",
                status, error_text
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
            confidence_score: 1.0,
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
        let response = self.client.get(&url).send().await?;
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

        if let Some(tools) = &request.tools {
            payload["tools"] = json!(tools);
            payload["tool_choice"] = json!("auto");
        }

        // Add modalities for Gemini image generation models
        if config.name.contains("gemini-2.5-flash-image") {
            info!("🎨 Adding image+text modalities for Gemini image generation");
            payload["modalities"] = json!(["image", "text"]);

            // Log token usage for debugging
            let approx_input_tokens = messages
                .iter()
                .map(|msg| {
                    msg.get("content")
                        .and_then(|c| c.as_str())
                        .unwrap_or("")
                        .len()
                        / 4
                })
                .sum::<usize>();
            info!(
                "📊 Approximate input tokens for image request: {}",
                approx_input_tokens
            );
        }

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

        let response = self
            .client
            .post(&config.api_endpoint)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;

            // Try to parse error as JSON to extract structured error details
            if let Ok(error_json) = serde_json::from_str::<Value>(&error_text) {
                // Check for rate limit errors (429)
                if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                    // Check if this is an upstream rate limit (from the model provider)
                    if let Some(error_obj) = error_json.get("error") {
                        if let Some(metadata_raw) = error_obj
                            .get("metadata")
                            .and_then(|m| m.get("raw"))
                            .and_then(|r| r.as_str())
                        {
                            if metadata_raw.contains("rate-limited upstream") {
                                warn!(
                                    "⚠️ Model {} is temporarily rate-limited at the provider level",
                                    config.name
                                );
                                return Err(AppError::ModelNotAvailable(format!(
                                    "Model '{}' is temporarily rate-limited at the provider level. Please try a different model or wait a moment.",
                                    config.name
                                )));
                            }
                        }
                    }

                    // Generic rate limit - could be OpenRouter or provider level
                    warn!("⚠️ Rate limit hit for model {}", config.name);
                    return Err(AppError::LLMRequestFailed(format!(
                        "Rate limit exceeded for model '{}'. Please wait a moment before retrying.",
                        config.name
                    )));
                }

                // Check for model not found errors
                if status == reqwest::StatusCode::NOT_FOUND {
                    // Check for "model not found" in metadata.raw
                    if let Some(metadata_raw) = error_json
                        .get("error")
                        .and_then(|e| e.get("metadata"))
                        .and_then(|m| m.get("raw"))
                        .and_then(|r| r.as_str())
                    {
                        if metadata_raw.contains("model not found") {
                            warn!("⚠️ Model {} not found in OpenRouter", config.name);
                            return Err(AppError::ModelNotAvailable(format!(
                                "Model '{}' is no longer available on OpenRouter",
                                config.name
                            )));
                        }
                    }

                    // Legacy check for "No endpoints found"
                    if error_text.contains("No endpoints found") {
                        warn!(
                            "⚠️ Model {} has no endpoints, suggesting fallback",
                            config.name
                        );
                        return Err(AppError::ModelNotAvailable(format!(
                            "Model '{}' is no longer available on OpenRouter",
                            config.name
                        )));
                    }
                }
            }

            // Generic error handling for other non-success responses
            return Err(AppError::LLMRequestFailed(format!(
                "OpenRouter API error {}: {}",
                status, error_text
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
        let reasoning = choice["message"]["reasoning"]
            .as_str()
            .map(|s| s.to_string());
        let reasoning_details = choice["message"]["reasoning_details"].as_array().cloned();

        // Handle image generation responses differently
        let content = if config.name.contains("gemini-2.5-flash-image")
            || config.name.contains("dall-e")
            || config.name.contains("stable-diffusion")
        {
            // For Gemini 2.5 Flash Image, check for inline_data with base64 images
            if config.name.contains("gemini-2.5-flash-image") {
                info!("🎨 Processing Gemini image generation response");

                // First check for images in the message.images field (OpenRouter format)
                if let Some(images) = choice["message"].get("images").and_then(|i| i.as_array()) {
                    info!("✅ Found {} image(s) in message.images array", images.len());
                    let mut result = String::new();

                    // Add text content if present
                    if let Some(text_content) = choice["message"]["content"].as_str() {
                        if !text_content.is_empty() {
                            info!("📝 Also found text content: {} chars", text_content.len());
                            result.push_str(text_content);
                            result.push_str("\n\n");
                        }
                    }

                    // Extract images from the images array
                    for (i, image) in images.iter().enumerate() {
                        if let Some(image_url_obj) = image.get("image_url") {
                            if let Some(url) = image_url_obj["url"].as_str() {
                                info!(
                                    "🖼️ Image {}: Found image URL ({}... bytes)",
                                    i,
                                    if url.len() > 100 { 100 } else { url.len() }
                                );
                                result.push_str(&format!("![Generated Image]({})\n\n", url));
                            }
                        }
                    }

                    result.trim().to_string()
                }
                // Fallback to checking content field
                else {
                    let message_content = &choice["message"]["content"];

                    info!(
                        "📊 Message content type: {}",
                        if message_content.is_array() {
                            "array"
                        } else if message_content.is_string() {
                            "string"
                        } else if message_content.is_object() {
                            "object"
                        } else {
                            "other"
                        }
                    );

                    // Path 1: Check if content is an array with parts (native Gemini format)
                    if let Some(parts) = message_content.as_array() {
                        info!("✅ Found parts array with {} elements", parts.len());
                        let mut result = String::new();
                        let mut found_images = 0;

                        for (i, part) in parts.iter().enumerate() {
                            info!(
                                "🔍 Part {}: keys = {:?}",
                                i,
                                part.as_object().map(|o| o.keys().collect::<Vec<_>>())
                            );

                            if let Some(text) = part["text"].as_str() {
                                info!("📝 Found text part: {} chars", text.len());
                                result.push_str(text);
                                result.push_str("\n\n");
                            }

                            // Check for inline_data (base64 images)
                            if let Some(inline_data) = part.get("inline_data") {
                                if let Some(data) = inline_data["data"].as_str() {
                                    found_images += 1;
                                    let mime_type =
                                        inline_data["mime_type"].as_str().unwrap_or("image/png");
                                    let data_preview =
                                        if data.len() > 50 { &data[..50] } else { data };
                                    info!("🖼️ Found inline_data image: mime={}, size={} bytes, preview={}", mime_type, data.len(), data_preview);
                                    let image_url = format!("data:{};base64,{}", mime_type, data);
                                    result.push_str(&format!(
                                        "![Generated Image]({})\n\n",
                                        image_url
                                    ));
                                }
                            }
                        }

                        if found_images > 0 {
                            info!(
                                "✅ Successfully extracted {} image(s) from response",
                                found_images
                            );
                        } else {
                            warn!("⚠️ No images found in parts array");
                        }

                        result.trim().to_string()
                    }
                    // Path 2: Check if content is a string (OpenRouter normalized format)
                    else if let Some(content_str) = message_content.as_str() {
                        info!("📄 Content is string, length: {} chars", content_str.len());

                        // Check if string contains base64 data URI
                        if content_str.contains("data:image/") && content_str.contains("base64,") {
                            info!("🖼️ Found base64 image data in string content");
                            content_str.to_string()
                        } else {
                            warn!("⚠️ String content doesn't contain image data");
                            content_str.to_string()
                        }
                    }
                    // Path 3: Check alternate locations
                    else {
                        warn!("⚠️ Unexpected content format, checking alternate locations");

                        // Try checking message.parts directly
                        if let Some(parts) =
                            choice["message"].get("parts").and_then(|p| p.as_array())
                        {
                            info!("🔍 Found message.parts array with {} elements", parts.len());
                            let mut result = String::new();

                            for part in parts {
                                if let Some(text) = part["text"].as_str() {
                                    result.push_str(text);
                                    result.push_str("\n\n");
                                }
                                if let Some(inline_data) = part.get("inline_data") {
                                    if let Some(data) = inline_data["data"].as_str() {
                                        let mime_type = inline_data["mime_type"]
                                            .as_str()
                                            .unwrap_or("image/png");
                                        info!("🖼️ Found image in message.parts: {}", mime_type);
                                        let image_url =
                                            format!("data:{};base64,{}", mime_type, data);
                                        result.push_str(&format!(
                                            "![Generated Image]({})\n\n",
                                            image_url
                                        ));
                                    }
                                }
                            }

                            result.trim().to_string()
                        } else {
                            // Log full response structure for debugging
                            warn!("❌ Could not find images in any expected location");
                            warn!(
                                "🔍 Full choice structure: {}",
                                serde_json::to_string_pretty(&choice)
                                    .unwrap_or_else(|_| "Could not serialize".to_string())
                            );
                            message_content.as_str().unwrap_or("").to_string()
                        }
                    }
                }
            } else {
                // For DALL-E and Stable Diffusion, look for image URLs
                let message_content = choice["message"]["content"].as_str().unwrap_or("");

                if message_content.contains("http")
                    && (message_content.contains(".jpg")
                        || message_content.contains(".png")
                        || message_content.contains(".jpeg")
                        || message_content.contains(".webp"))
                {
                    let image_urls: Vec<&str> = message_content
                        .split_whitespace()
                        .filter(|word| {
                            word.starts_with("http")
                                && (word.contains(".jpg")
                                    || word.contains(".png")
                                    || word.contains(".jpeg")
                                    || word.contains(".webp"))
                        })
                        .collect();

                    if !image_urls.is_empty() {
                        image_urls
                            .iter()
                            .map(|url| format!("![Generated Image]({})", url))
                            .collect::<Vec<String>>()
                            .join("\n\n")
                    } else {
                        message_content.to_string()
                    }
                } else {
                    message_content.to_string()
                }
            }
        } else {
            choice["message"]["content"]
                .as_str()
                .unwrap_or("")
                .to_string()
        };

        Ok(LLMResponse {
            content,
            model_used: config.name.clone(),
            usage: usage_map,
            latency_ms,
            cost_estimate,
            finish_reason: choice["finish_reason"]
                .as_str()
                .unwrap_or("stop")
                .to_string(),
            confidence_score: 1.0,
            tool_calls: choice["message"]["tool_calls"]
                .as_array()
                .map(|calls| calls.iter().cloned().collect()),
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
        let response = self
            .client
            .get("https://openrouter.ai/api/v1/models")
            .send()
            .await?;
        Ok(response.status().is_success())
    }
}
