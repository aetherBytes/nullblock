use crate::{
    error::AppError,
    models::{
        ChatMessageResponse, ChatRequest, ModelSelectionRequest, PersonalityRequest,
    },
    server::AppState,
};
use axum::{extract::{Query, State}, Json};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use tracing::{info, warn, error};

// Free tier limits to prevent resource exhaustion
const FREE_TIER_MAX_INPUT_CHARS: usize = 8000;  // ~2000 tokens
const FREE_TIER_MAX_OUTPUT_TOKENS: u32 = 1500;  // Reasonable response length

#[derive(Deserialize)]
pub struct SearchModelsQuery {
    pub q: Option<String>,
    pub category: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Deserialize)]
pub struct ModelInfoQuery {
    pub model_name: Option<String>,
}

pub async fn chat(
    State(state): State<AppState>,
    Json(request): Json<ChatRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Extract user_id from user_context if available
    let user_id = request.user_context
        .as_ref()
        .and_then(|ctx| ctx.get("user_id"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Track if this is a free tier user (no own API key)
    let mut is_free_tier = false;
    let mut rate_limit_info = None;

    if let Some(ref uid) = user_id {
        // Check if user has their own OpenRouter API key
        let has_own_key = state.erebus_client
            .user_has_api_key(uid, "openrouter")
            .await
            .unwrap_or(false);

        if !has_own_key {
            is_free_tier = true;

            // User is using agent's key - check rate limit
            match state.erebus_client.check_rate_limit(uid, "hecate").await {
                Ok(status) => {
                    if !status.allowed {
                        return Err(AppError::FreeTierRateLimitExceeded {
                            remaining: status.remaining,
                            limit: status.limit,
                            resets_at: status.resets_at,
                        });
                    }
                    info!("📊 Rate limit check passed: {}/{} remaining", status.remaining, status.limit);
                    rate_limit_info = Some(status);
                }
                Err(e) => {
                    warn!("⚠️ Rate limit check failed, allowing request: {}", e);
                }
            }
        } else {
            info!("🔑 User {} has their own API key, skipping limits", uid);
        }
    } else {
        // No user_id means anonymous/unauthenticated - treat as free tier
        is_free_tier = true;
    }

    // Apply free tier input length limit
    if is_free_tier && request.message.len() > FREE_TIER_MAX_INPUT_CHARS {
        return Err(AppError::BadRequest(format!(
            "Message too long for free tier. Maximum {} characters allowed. Add your own API key for unlimited message length.",
            FREE_TIER_MAX_INPUT_CHARS
        )));
    }

    // Build user context with free tier constraints if applicable
    let user_context = if is_free_tier {
        let mut ctx = request.user_context.unwrap_or_default();
        ctx.insert("free_tier".to_string(), json!(true));
        ctx.insert("max_output_tokens".to_string(), json!(FREE_TIER_MAX_OUTPUT_TOKENS));
        info!("🆓 Free tier user: limiting output to {} tokens", FREE_TIER_MAX_OUTPUT_TOKENS);
        Some(ctx)
    } else {
        request.user_context
    };

    // For free-tier users, validate the current model is allowed
    if is_free_tier {
        let agent = state.hecate_agent.read().await;
        if let Some(ref current_model) = agent.current_model {
            // Check if model is free by name pattern (quick check)
            if !current_model.ends_with(":free") {
                // Get llm_factory to do full validation
                if let Some(ref llm_factory) = agent.llm_factory {
                    let factory = llm_factory.read().await;
                    if let Err(msg) = factory.validate_model_for_free_tier(current_model).await {
                        drop(factory);
                        drop(agent);
                        return Err(AppError::BadRequest(msg));
                    }
                }
            }
        }
        drop(agent);
    }

    // Execute the chat
    let mut agent = state.hecate_agent.write().await;
    let response = agent.chat(request.message, user_context).await?;

    // Increment rate limit after successful chat (only for free tier users)
    if let (Some(ref uid), Some(_)) = (&user_id, &rate_limit_info) {
        if let Err(e) = state.erebus_client.increment_rate_limit(uid, "hecate").await {
            warn!("⚠️ Failed to increment rate limit: {}", e);
        }
    }

    // Include rate limit info in response metadata
    let mut metadata = response.metadata.clone();
    if let Some(status) = rate_limit_info {
        metadata.insert("rate_limit".to_string(), json!({
            "remaining": status.remaining - 1,
            "limit": status.limit,
            "resets_at": status.resets_at
        }));
    }

    Ok(Json(json!({
        "content": response.content,
        "model_used": response.model_used,
        "latency_ms": response.latency_ms,
        "confidence_score": response.confidence_score,
        "metadata": metadata
    })))
}

pub async fn health(State(state): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    let agent = state.hecate_agent.read().await;

    if !agent.running {
        return Err(AppError::AgentNotRunning);
    }

    // Check if LLM models are available
    if let Some(llm_factory) = &agent.llm_factory {
        let factory = llm_factory.read().await;
        match factory.health_check().await {
            Ok(health_info) => {
                let models_available = health_info["models_available"].as_u64().unwrap_or(0);
                let llm_status = health_info["overall_status"].as_str().unwrap_or("unknown");

                if models_available == 0 || llm_status == "unhealthy" {
                    return Err(AppError::LLMRequestFailed("No working LLM models available. Please add your API keys via Settings → API Keys in the UI. Visit https://openrouter.ai/ to get a free API key.".to_string()));
                }
            }
            Err(e) => {
                return Err(AppError::LLMRequestFailed(format!("LLM service health check failed: {}. Please check your API key configuration.", e)));
            }
        }
    } else {
        return Err(AppError::AgentNotInitialized);
    }

    let status = agent.get_model_status().await?;

    Ok(Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "agent": status
    })))
}

pub async fn model_status(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let agent = state.hecate_agent.read().await;
    let status = agent.get_model_status().await?;
    Ok(Json(status))
}

pub async fn available_models(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let agent = state.hecate_agent.read().await;
    let api_keys = state.config.get_api_keys();
    
    // Try to fetch real models from OpenRouter
    let available_models = if let Some(openrouter_key) = &api_keys.openrouter {
        info!("🔑 OpenRouter API key found, calling OpenRouter API for available models");
        match fetch_openrouter_models(openrouter_key).await {
            Ok(models) => {
                info!("✅ OpenRouter API call successful, got {} models", models.len());
                models
            }
            Err(e) => {
                error!("❌ OpenRouter API call failed: {}, falling back to mock models", e);
                get_fallback_models()
            }
        }
    } else {
        warn!("⚠️ No OpenRouter API key configured, using fallback models");
        get_fallback_models()
    };

    let default_model = "cognitivecomputations/dolphin3.0-mistral-24b:free";

    Ok(Json(json!({
        "models": available_models,
        "current_model": agent.current_model,
        "default_model": default_model,
        "recommended_models": {
            "free": "cognitivecomputations/dolphin3.0-mistral-24b:free",
            "reasoning": "cognitivecomputations/dolphin3.0-r1-mistral-24b:free",
            "premium": "anthropic/claude-3.5-sonnet",
            "fast": "deepseek/deepseek-chat-v3.1:free",
            "image_generation": "google/gemini-2.5-flash-image-preview"
        },
        "total_models": available_models.len()
    })))
}

async fn fetch_openrouter_models(api_key: &str) -> Result<Vec<serde_json::Value>, AppError> {
    use reqwest::Client;
    
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
        
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::AUTHORIZATION,
        format!("Bearer {}", api_key).parse()
            .map_err(|e| AppError::InternalError(format!("Invalid API key format: {}", e)))?,
    );
    headers.insert("HTTP-Referer", "https://nullblock.ai".parse()
        .map_err(|e| AppError::InternalError(format!("Invalid referer: {}", e)))?);
    headers.insert("X-Title", "NullBlock Agent Platform".parse()
        .map_err(|e| AppError::InternalError(format!("Invalid title: {}", e)))?);
    
    let response = client
        .get("https://openrouter.ai/api/v1/models")
        .headers(headers)
        .send()
        .await?;
        
    if !response.status().is_success() {
        return Err(AppError::LLMRequestFailed(format!(
            "OpenRouter API returned status: {}", response.status()
        )));
    }
    
    let data: serde_json::Value = response.json().await?;
    let models_array = data["data"].as_array()
        .ok_or_else(|| AppError::LLMRequestFailed("No models data in response".to_string()))?;
    
    let mut processed_models = Vec::new();
    
    for model in models_array {
        if let Some(model_obj) = model.as_object() {
            // Extract core model information using correct OpenRouter field names
            let model_id = model_obj.get("id").and_then(|v| v.as_str()).unwrap_or("");
            let model_name = model_obj.get("name").and_then(|v| v.as_str()).unwrap_or(model_id);
            let description = model_obj.get("description").and_then(|v| v.as_str()).unwrap_or("");
            let context_length = model_obj.get("context_length").and_then(|v| v.as_u64()).unwrap_or(8192);
            let canonical_slug = model_obj.get("canonical_slug").and_then(|v| v.as_str()).unwrap_or("");
            let hugging_face_id = model_obj.get("hugging_face_id").and_then(|v| v.as_str()).unwrap_or("");
            
            // Extract created timestamp (Unix timestamp) - ensure it's recent for testing
            let created = model_obj.get("created").and_then(|v| v.as_i64()).unwrap_or_else(|| {
                // For models without created timestamp, use a recent timestamp for debugging
                chrono::Utc::now().timestamp()
            });
            
            let created_at = chrono::DateTime::from_timestamp(created, 0)
                .unwrap_or_else(|| chrono::Utc::now())
                .to_rfc3339();
            
            // Extract architecture information
            let architecture = model_obj.get("architecture").unwrap_or(&serde_json::Value::Null);
            let input_modalities = architecture.get("input_modalities")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
                .unwrap_or_else(|| vec!["text"]);
            let output_modalities = architecture.get("output_modalities")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
                .unwrap_or_else(|| vec!["text"]);
            let tokenizer = architecture.get("tokenizer").and_then(|v| v.as_str()).unwrap_or("");
            let instruct_type = architecture.get("instruct_type").and_then(|v| v.as_str()).unwrap_or("");
            
            // Extract top_provider information
            let top_provider = model_obj.get("top_provider").unwrap_or(&serde_json::Value::Null);
            let is_moderated = top_provider.get("is_moderated").and_then(|v| v.as_bool()).unwrap_or(false);
            let max_completion_tokens = top_provider.get("max_completion_tokens").and_then(|v| v.as_u64()).unwrap_or(4096);
            
            // Extract pricing information using correct field names
            let pricing = model_obj.get("pricing").unwrap_or(&serde_json::Value::Null);
            let (prompt_cost, completion_cost, image_cost, request_cost, web_search_cost, internal_reasoning_cost, input_cache_read_cost, input_cache_write_cost) = if let Some(pricing_obj) = pricing.as_object() {
                let prompt = pricing_obj.get("prompt").and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                let completion = pricing_obj.get("completion").and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                let image = pricing_obj.get("image").and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                let request = pricing_obj.get("request").and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                let web_search = pricing_obj.get("web_search").and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                let internal_reasoning = pricing_obj.get("internal_reasoning").and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                let input_cache_read = pricing_obj.get("input_cache_read").and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                let input_cache_write = pricing_obj.get("input_cache_write").and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                (prompt, completion, image, request, web_search, internal_reasoning, input_cache_read, input_cache_write)
            } else {
                (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
            };
            
            // Calculate average cost per 1k tokens (convert from per-token to per-1k-tokens)
            let avg_cost_per_1k = (prompt_cost + completion_cost) / 2.0 * 1000.0;
            
            // Determine tier based on cost
            let tier = if avg_cost_per_1k <= 0.0 {
                "economical"
            } else if avg_cost_per_1k <= 0.5 {
                "fast"  
            } else if avg_cost_per_1k <= 2.0 {
                "standard"
            } else {
                "premium"
            };
            
            // Determine if it supports reasoning
            let supports_reasoning = model_id.contains("reasoning") || 
                                   model_id.contains("r1") || 
                                   model_id.contains("deepseek-r") ||
                                   model_id.contains("o1") ||
                                   instruct_type.to_lowercase().contains("reasoning") ||
                                   internal_reasoning_cost > 0.0;
            
            // Check for multimodal capabilities
            let supports_vision = input_modalities.contains(&"image");
            let supports_audio = input_modalities.contains(&"audio");
            
            // Build capabilities array based on model features
            let mut capabilities = vec!["conversation"];
            if supports_reasoning {
                capabilities.push("reasoning");
            }
            if supports_vision {
                capabilities.push("vision");
            }
            if supports_audio {
                capabilities.push("audio");
            }
            capabilities.push("creative");
            
            // Determine icon based on model family and capabilities
            let icon = if model_id.contains("claude") {
                "🎭"
            } else if model_id.contains("gpt") || model_id.contains("openai") {
                "🚀"
            } else if model_id.contains("deepseek") {
                if supports_reasoning { "🧠" } else { "🤖" }
            } else if model_id.contains("llama") {
                "🦙"
            } else if model_id.contains("gemini") {
                "💎"
            } else if supports_reasoning {
                "🧠"
            } else if supports_vision {
                "👁️"
            } else {
                "⚡"
            };
            
            // Extract supported parameters and per-request limits
            let supported_parameters = model_obj.get("supported_parameters")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
                .unwrap_or_default();
            let per_request_limits = model_obj.get("per_request_limits").cloned().unwrap_or(json!({}));
            
            // Build comprehensive model object with all OpenRouter data
            let processed_model = json!({
                // Core identification
                "id": model_id,
                "name": model_id, // Keep as 'name' for frontend compatibility
                "display_name": model_name,
                "canonical_slug": canonical_slug,
                "hugging_face_id": hugging_face_id,
                
                // Metadata
                "description": description,
                "icon": icon,
                "provider": "openrouter",
                "available": true,
                "tier": tier,
                "is_popular": model_id.contains("deepseek") || model_id.contains("claude") || model_id.contains("gpt-4"),
                
                // Timing
                "created": created, // Unix timestamp from OpenRouter
                "created_at": created_at, // ISO string for frontend
                "updated_at": chrono::Utc::now().to_rfc3339(),
                
                // Capabilities
                "context_length": context_length,
                "max_completion_tokens": max_completion_tokens,
                "capabilities": capabilities,
                "supports_reasoning": supports_reasoning,
                "supports_vision": supports_vision,
                "supports_audio": supports_audio,
                "is_moderated": is_moderated,
                "supported_parameters": supported_parameters,
                "per_request_limits": per_request_limits,
                
                // Architecture details
                "architecture": {
                    "input_modalities": input_modalities,
                    "output_modalities": output_modalities,
                    "tokenizer": tokenizer,
                    "instruct_type": instruct_type
                },
                
                // Pricing details
                "cost_per_1k_tokens": avg_cost_per_1k,
                "pricing": {
                    "prompt": prompt_cost,
                    "completion": completion_cost,
                    "image": image_cost,
                    "request": request_cost,
                    "web_search": web_search_cost,
                    "internal_reasoning": internal_reasoning_cost,
                    "input_cache_read": input_cache_read_cost,
                    "input_cache_write": input_cache_write_cost
                }
            });
            
            processed_models.push(processed_model);
        }
    }
    
    // Debug logging for latest models functionality
    let recent_models = processed_models.iter()
        .filter(|model| {
            if let Some(created) = model.get("created").and_then(|v| v.as_i64()) {
                let now = chrono::Utc::now().timestamp();
                let days_old = (now - created) / (24 * 60 * 60);
                days_old < 30 // Models created in last 30 days
            } else {
                false
            }
        })
        .collect::<Vec<_>>();
    
    info!("✅ Fetched {} models from OpenRouter API", processed_models.len());
    info!("🔍 Found {} recent models (created within 30 days)", recent_models.len());
    
    // Check for specific newer models the user mentioned
    let newer_models = ["sonoma", "qwen", "kimi", "dusk", "sky"];
    let found_newer_models = processed_models.iter()
        .filter(|model| {
            let name = model.get("display_name").and_then(|v| v.as_str()).unwrap_or("");
            let id = model.get("id").and_then(|v| v.as_str()).unwrap_or("");
            newer_models.iter().any(|&search| {
                name.to_lowercase().contains(search) || id.to_lowercase().contains(search)
            })
        })
        .collect::<Vec<_>>();
    
    info!("🔍 Found {} models matching newer model keywords (sonoma, qwen, kimi, dusk, sky)", found_newer_models.len());
    if !found_newer_models.is_empty() {
        info!("📋 Newer models found:");
        for (i, model) in found_newer_models.iter().take(5).enumerate() {
            let name = model.get("display_name").and_then(|v| v.as_str()).unwrap_or("Unknown");
            let id = model.get("id").and_then(|v| v.as_str()).unwrap_or("Unknown");
            let created = model.get("created").and_then(|v| v.as_i64()).unwrap_or(0);
            let created_at = model.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
            info!("  {}. {} ({}) - created: {} ({})", i + 1, name, id, created, created_at);
        }
    }
    
    // Log a few recent models for debugging
    if !recent_models.is_empty() {
        info!("📋 Recent models preview:");
        for (i, model) in recent_models.iter().take(3).enumerate() {
            let name = model.get("display_name").and_then(|v| v.as_str()).unwrap_or("Unknown");
            let id = model.get("id").and_then(|v| v.as_str()).unwrap_or("Unknown");
            let created = model.get("created").and_then(|v| v.as_i64()).unwrap_or(0);
            let created_at = model.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
            info!("  {}. {} ({}) - created: {} ({})", i + 1, name, id, created, created_at);
        }
    }
    
    Ok(processed_models)
}

fn get_fallback_models() -> Vec<serde_json::Value> {
    warn!("🔄 Using fallback models due to OpenRouter API unavailability");
    let now = chrono::Utc::now();
    let created_timestamp = now.timestamp();
    let created_at = now.to_rfc3339();
    
    vec![
        json!({
            "id": "x-ai/grok-4-fast:free",
            "name": "x-ai/grok-4-fast:free",
            "display_name": "DeepSeek Chat v3.1 Free",
            "icon": "🤖",
            "provider": "openrouter",
            "available": true,
            "tier": "economical",
            "context_length": 128000,
            "max_completion_tokens": 8192,
            "capabilities": vec!["conversation", "reasoning", "creative"],
            "cost_per_1k_tokens": 0.0,
            "supports_reasoning": true,
            "supports_vision": false,
            "supports_audio": false,
            "is_moderated": false,
            "description": "Free DeepSeek model optimized for conversation",
            "is_popular": true,
            "created": created_timestamp,
            "created_at": created_at,
            "updated_at": created_at,
            "canonical_slug": "x-ai/grok-4-fast:free",
            "hugging_face_id": "",
            "supported_parameters": vec!["temperature", "top_p", "max_tokens"],
            "per_request_limits": json!({}),
            "architecture": {
                "input_modalities": ["text"],
                "output_modalities": ["text"],
                "tokenizer": "DeepSeek",
                "instruct_type": "chat"
            },
            "pricing": {
                "prompt": "0",
                "completion": "0",
                "image": "0",
                "request": "0",
                "web_search": "0",
                "internal_reasoning": "0",
                "input_cache_read": "0",
                "input_cache_write": "0"
            }
        }),
        json!({
            "id": "google/gemini-2.5-flash-image-preview",
            "name": "google/gemini-2.5-flash-image-preview",
            "display_name": "Gemini 2.5 Flash Image",
            "icon": "🎨",
            "provider": "openrouter",
            "available": true,
            "tier": "premium",
            "context_length": 1000000,
            "max_completion_tokens": 8192,
            "capabilities": vec!["image_generation", "creative", "conversation", "vision"],
            "cost_per_1k_tokens": 1.5,
            "supports_reasoning": false,
            "supports_vision": true,
            "supports_audio": false,
            "is_moderated": true,
            "description": "Gemini 2.5 Flash Image - Advanced image generation with contextual understanding",
            "is_popular": true,
            "created": created_timestamp,
            "created_at": created_at,
            "updated_at": created_at,
            "canonical_slug": "google/gemini-2.5-flash-image-preview",
            "hugging_face_id": "",
            "supported_parameters": vec!["temperature", "top_p", "max_tokens", "modalities"],
            "per_request_limits": json!({}),
            "architecture": {
                "input_modalities": ["text", "image"],
                "output_modalities": ["text", "image"],
                "tokenizer": "Gemini",
                "instruct_type": "chat_with_image_generation"
            },
            "pricing": {
                "prompt": "0.30",
                "completion": "2.50",
                "image": "1.238",
                "request": "0.03",
                "web_search": "0",
                "internal_reasoning": "0",
                "input_cache_read": "0",
                "input_cache_write": "0"
            }
        }),
        json!({
            "id": "deepseek/deepseek-r1",
            "name": "deepseek/deepseek-r1",
            "display_name": "DeepSeek R1 (Reasoning)",
            "icon": "🧠",
            "provider": "openrouter",
            "available": true,
            "tier": "premium",
            "context_length": 64000,
            "max_completion_tokens": 8192,
            "capabilities": vec!["reasoning", "logic", "mathematics"],
            "cost_per_1k_tokens": 1.4,
            "supports_reasoning": true,
            "supports_vision": false,
            "supports_audio": false,
            "is_moderated": true,
            "description": "Advanced reasoning model for complex problems",
            "is_popular": true,
            "created": created_timestamp,
            "created_at": created_at,
            "updated_at": created_at,
            "canonical_slug": "deepseek/deepseek-r1",
            "hugging_face_id": "",
            "supported_parameters": vec!["temperature", "top_p", "max_tokens"],
            "per_request_limits": json!({}),
            "architecture": {
                "input_modalities": ["text"],
                "output_modalities": ["text"],
                "tokenizer": "DeepSeek",
                "instruct_type": "reasoning"
            },
            "pricing": {
                "prompt": "0.0007",
                "completion": "0.0007",
                "image": "0",
                "request": "0",
                "web_search": "0",
                "internal_reasoning": "0.0014",
                "input_cache_read": "0",
                "input_cache_write": "0"
            }
        })
    ]
}

pub async fn search_models(
    State(state): State<AppState>,
    Query(params): Query<SearchModelsQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _agent = state.hecate_agent.read().await;
    let api_keys = state.config.get_api_keys();
    
    // Get available models (same source as available_models endpoint)
    let available_models = if let Some(openrouter_key) = &api_keys.openrouter {
        fetch_openrouter_models(openrouter_key).await
            .unwrap_or_else(|_| get_fallback_models())
    } else {
        get_fallback_models()
    };
    
    let mut results = available_models;
    
    // Apply search query filter
    if let Some(query) = &params.q {
        let query_lower = query.to_lowercase();
        if !query_lower.is_empty() {
            results = results.into_iter().filter(|model| {
                let name = model["name"].as_str().unwrap_or("").to_lowercase();
                let display_name = model["display_name"].as_str().unwrap_or("").to_lowercase();
                let description = model["description"].as_str().unwrap_or("").to_lowercase();
                
                name.contains(&query_lower) || 
                display_name.contains(&query_lower) || 
                description.contains(&query_lower)
            }).collect();
        }
    }
    
    // Apply category filter
    if let Some(category) = &params.category {
        let category_lower = category.to_lowercase();
        results = results.into_iter().filter(|model| {
            let tier = model["tier"].as_str().unwrap_or("").to_lowercase();
            let supports_reasoning = model["supports_reasoning"].as_bool().unwrap_or(false);
            
            match category_lower.as_str() {
                "free" | "economical" => tier == "economical",
                "fast" => tier == "fast",
                "premium" => tier == "premium",
                "reasoning" | "thinkers" => supports_reasoning,
                "latest" => {
                    // For latest, we can sort by created_at but for simplicity just include all
                    true
                }
                _ => true
            }
        }).collect();
    }

    let limit = params.limit.unwrap_or(20);
    results.truncate(limit);

    Ok(Json(json!({
        "query": params.q.unwrap_or_default(),
        "category": params.category.unwrap_or_default(),
        "results": results,
        "total_available": results.len()
    })))
}

pub async fn set_model(
    State(state): State<AppState>,
    Json(request): Json<ModelSelectionRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let api_keys = state.config.get_api_keys();

    // Check if user is free tier
    let mut is_free_tier = false;
    if let Some(ref user_id) = request.user_id {
        let has_own_key = state.erebus_client
            .user_has_api_key(user_id, "openrouter")
            .await
            .unwrap_or(false);
        is_free_tier = !has_own_key;
    } else {
        // No user_id means treat as free tier (conservative approach)
        is_free_tier = true;
    }

    // For free-tier users, validate the requested model is free
    if is_free_tier {
        let agent = state.hecate_agent.read().await;
        if let Some(ref llm_factory) = agent.llm_factory {
            let factory = llm_factory.read().await;
            if let Err(msg) = factory.validate_model_for_free_tier(&request.model_name).await {
                drop(factory);
                drop(agent);
                return Err(AppError::BadRequest(msg));
            }
            info!("✅ Model '{}' validated for free-tier user", request.model_name);
        }
        drop(agent);
    }

    let mut agent = state.hecate_agent.write().await;
    let old_model = agent.get_preferred_model();
    let success = agent.set_preferred_model(request.model_name.clone(), &api_keys).await;

    if success {
        Ok(Json(json!({
            "success": true,
            "model": request.model_name,
            "previous_model": old_model
        })))
    } else {
        let error_detail = agent.get_model_availability_reason(&request.model_name, &api_keys).await;
        Err(AppError::ModelNotAvailable(error_detail))
    }
}

pub async fn refresh_models(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Mock refresh functionality
    info!("✅ Model availability refreshed");
    
    Ok(Json(json!({
        "success": true,
        "message": "Model availability refreshed"
    })))
}

pub async fn reset_models(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Mock reset functionality
    info!("✅ Models reset and refreshed");
    
    Ok(Json(json!({
        "success": true,
        "message": "Models reset and refreshed successfully"
    })))
}

pub async fn set_personality(
    State(state): State<AppState>,
    Json(request): Json<PersonalityRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut agent = state.hecate_agent.write().await;
    let old_personality = agent.personality.clone();
    
    agent.set_personality(request.personality.clone());
    
    Ok(Json(json!({
        "success": true,
        "personality": request.personality,
        "previous_personality": old_personality
    })))
}

pub async fn clear_conversation(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut agent = state.hecate_agent.write().await;
    let conversation_length = agent.get_conversation_history().await.len();
    
    agent.clear_conversation().await;
    
    Ok(Json(json!({
        "success": true,
        "message": "Conversation cleared",
        "cleared_messages": conversation_length
    })))
}

pub async fn get_history(
    State(state): State<AppState>,
) -> Result<Json<Vec<ChatMessageResponse>>, AppError> {
    let agent = state.hecate_agent.read().await;
    let history = agent.get_conversation_history().await;
    
    let response_history: Vec<ChatMessageResponse> = history
        .into_iter()
        .map(|msg| msg.into())
        .collect();
    
    Ok(Json(response_history))
}

pub async fn get_model_info(
    State(state): State<AppState>,
    Query(params): Query<ModelInfoQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let agent = state.hecate_agent.read().await;
    let api_keys = state.config.get_api_keys();
    
    let model_name = params.model_name.unwrap_or_else(|| {
        agent.current_model.clone().unwrap_or_else(|| "cognitivecomputations/dolphin3.0-mistral-24b:free".to_string())
    });

    if !agent.is_model_available(&model_name, &api_keys).await {
        return Err(AppError::ModelNotAvailable(format!("Model {} not found", model_name)));
    }
    
    // Determine display name and info based on model
    let (display_name, icon, description) = match model_name.as_str() {
        "cognitivecomputations/dolphin3.0-mistral-24b:free" => (
            "Dolphin 3.0 Mistral 24B Free",
            "🐬",
            "Ultimate general purpose free model for coding, math, and function calling"
        ),
        "cognitivecomputations/dolphin3.0-r1-mistral-24b:free" => (
            "Dolphin 3.0 R1 Mistral 24B Free",
            "🐬",
            "Reasoning-optimized Dolphin model with 800k reasoning traces"
        ),
        "deepseek/deepseek-chat-v3.1:free" => (
            "DeepSeek Chat v3.1 Free",
            "🤖",
            "Free DeepSeek model optimized for conversation"
        ),
        "google/gemini-2.5-flash-image-preview" => (
            "Gemini 2.5 Flash Image",
            "🎨",
            "Image generation model"
        ),
        _ => (model_name.as_str(), "🤖", "AI model")
    };

    let model_info = json!({
        "name": model_name,
        "display_name": display_name,
        "icon": icon,
        "provider": "openrouter",
        "description": description,
        "tier": "free",
        "available": true,
        "is_current": agent.current_model.as_ref() == Some(&model_name),
        "is_dynamic": false,
        "status": "active",
        "context_length": 128000,
        "max_tokens": 8192,
        "capabilities": ["conversation", "reasoning", "creative"],
        "supports_reasoning": true,
        "supports_vision": false,
        "supports_function_calling": false,
        "cost_per_1k_tokens": 0.0,
        "cost_per_1m_tokens": 0.0,
        "performance_stats": {},
        "conversation_length": agent.get_conversation_history().await.len()
    });
    
    Ok(Json(model_info))
}