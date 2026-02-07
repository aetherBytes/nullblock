use crate::{
    error::AppError,
    models::{LLMRequest, ModelProvider},
    server::AppState,
};
use axum::{extract::Path, extract::State, http::HeaderMap, Json};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tracing::{info, warn};

#[derive(Debug, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: Option<String>,
    pub messages: Vec<ChatMessage>,
    #[serde(default)]
    pub max_tokens: Option<u32>,
    #[serde(default)]
    pub temperature: Option<f64>,
    #[serde(default)]
    pub top_p: Option<f64>,
    #[serde(default)]
    pub tools: Option<Vec<Value>>,
    #[serde(default)]
    pub stop: Option<Vec<String>>,
    #[serde(default)]
    pub stream: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

fn extract_text_content(content: &Option<Value>) -> String {
    match content {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Array(parts)) => parts
            .iter()
            .filter_map(|part| {
                if part.get("type").and_then(|t| t.as_str()) == Some("text") {
                    part.get("text").and_then(|t| t.as_str()).map(String::from)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}

#[derive(Debug, Serialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

#[derive(Debug, Serialize)]
pub struct Choice {
    pub index: u32,
    pub message: ResponseMessage,
    pub finish_reason: String,
}

#[derive(Debug, Serialize)]
pub struct ResponseMessage {
    pub role: String,
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<Value>>,
}

#[derive(Debug, Serialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Serialize)]
pub struct ModelListResponse {
    pub object: String,
    pub data: Vec<ModelObject>,
}

#[derive(Debug, Serialize)]
pub struct ModelObject {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub owned_by: String,
}

pub async fn handle_chat_completions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ChatCompletionRequest>,
) -> axum::response::Response {
    let agent_name = headers
        .get("x-agent-name")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("default");

    let stream = request.stream.unwrap_or(false);

    info!(
        "LLM Proxy: chat completions request, model={:?}, messages={}, agent={}, tools={}, stream={}",
        request.model,
        request.messages.len(),
        agent_name,
        request.tools.as_ref().map_or(0, |t| t.len()),
        stream
    );

    let messages_for_llm: Vec<HashMap<String, String>> = request
        .messages
        .iter()
        .map(|m| {
            let mut map = HashMap::new();
            map.insert("role".to_string(), m.role.clone());
            map.insert("content".to_string(), extract_text_content(&m.content));
            map
        })
        .collect();

    let prompt = request
        .messages
        .last()
        .map(|m| extract_text_content(&m.content))
        .unwrap_or_default();

    let system_prompt = request
        .messages
        .iter()
        .find(|m| m.role == "system")
        .map(|m| extract_text_content(&m.content))
        .filter(|s| !s.is_empty());

    let is_agent_key_path = state.agent_openrouter_keys.contains_key(agent_name);

    let agent_model_pref = if is_agent_key_path {
        let prefs = state.agent_model_preferences.read().await;
        prefs.get(agent_name).cloned()
    } else {
        None
    };

    let use_agent_key = is_agent_key_path
        && (agent_model_pref.is_some()
            || !matches!(request.model.as_deref(), Some("auto") | None));

    let model_override = match request.model.as_deref() {
        Some("auto") | None if use_agent_key => agent_model_pref.clone(),
        Some("auto") | None => None,
        Some(m) => Some(m.to_string()),
    };

    info!(
        "LLM Proxy: routing — agent={}, agent_key={}, preference={:?}, use_agent_key={}, model_override={:?}",
        agent_name, is_agent_key_path, agent_model_pref, use_agent_key, model_override
    );

    let llm_request = LLMRequest {
        prompt,
        system_prompt,
        messages: Some(messages_for_llm),
        max_tokens: request.max_tokens,
        temperature: request.temperature,
        top_p: request.top_p,
        stop_sequences: request.stop,
        tools: request.tools,
        model_override,
        concise: false,
        max_chars: None,
        reasoning: None,
    };

    let agent = state.hecate_agent.read().await;
    let llm_factory = match agent.llm_factory.as_ref() {
        Some(f) => f.clone(),
        None => {
            return error_response(
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                "agent_not_initialized",
                "Agent LLM factory not ready",
            );
        }
    };
    drop(agent);

    let factory = llm_factory.read().await;

    let response = if use_agent_key {
        let agent_key = state.agent_openrouter_keys.get(agent_name).unwrap();
        info!("LLM Proxy: using {} agent key with model_override={:?}", agent_name, llm_request.model_override);
        factory
            .generate_with_key(&llm_request, ModelProvider::OpenRouter, agent_key)
            .await
    } else {
        info!("LLM Proxy: using default hecate path (free model router)");
        factory.generate(&llm_request, None).await
    };

    let response = match response {
        Ok(r) => r,
        Err(e) => {
            warn!("LLM Proxy: generation failed for agent={}: {}", agent_name, e);
            return error_response(
                axum::http::StatusCode::BAD_GATEWAY,
                "llm_error",
                &format!("LLM generation failed: {}", e),
            );
        }
    };

    let usage = Usage {
        prompt_tokens: *response.usage.get("prompt_tokens").unwrap_or(&0),
        completion_tokens: *response.usage.get("completion_tokens").unwrap_or(&0),
        total_tokens: *response.usage.get("total_tokens").unwrap_or(&0),
    };

    let content = if response.content.is_empty() {
        None
    } else {
        Some(response.content)
    };

    let id = format!("chatcmpl-{}", uuid::Uuid::new_v4());
    let created = chrono::Utc::now().timestamp();
    let model_used = response.model_used;
    let finish_reason = response.finish_reason;
    let tool_calls = response.tool_calls;

    info!(
        "LLM Proxy: completion — agent={}, model={}, finish={}, stream={}",
        agent_name, model_used, finish_reason, stream
    );

    if stream {
        let mut sse_body = String::new();

        let role_chunk = serde_json::json!({
            "id": id,
            "object": "chat.completion.chunk",
            "created": created,
            "model": model_used,
            "choices": [{"index": 0, "delta": {"role": "assistant"}, "finish_reason": null}]
        });
        sse_body.push_str(&format!("data: {}\n\n", serde_json::to_string(&role_chunk).unwrap()));

        if let Some(ref text) = content {
            let content_chunk = serde_json::json!({
                "id": id,
                "object": "chat.completion.chunk",
                "created": created,
                "model": model_used,
                "choices": [{"index": 0, "delta": {"content": text}, "finish_reason": null}]
            });
            sse_body.push_str(&format!("data: {}\n\n", serde_json::to_string(&content_chunk).unwrap()));
        }

        if let Some(ref tc) = tool_calls {
            let tc_chunk = serde_json::json!({
                "id": id,
                "object": "chat.completion.chunk",
                "created": created,
                "model": model_used,
                "choices": [{"index": 0, "delta": {"tool_calls": tc}, "finish_reason": null}]
            });
            sse_body.push_str(&format!("data: {}\n\n", serde_json::to_string(&tc_chunk).unwrap()));
        }

        let final_chunk = serde_json::json!({
            "id": id,
            "object": "chat.completion.chunk",
            "created": created,
            "model": model_used,
            "choices": [{"index": 0, "delta": {}, "finish_reason": finish_reason}],
            "usage": {"prompt_tokens": usage.prompt_tokens, "completion_tokens": usage.completion_tokens, "total_tokens": usage.total_tokens}
        });
        sse_body.push_str(&format!("data: {}\n\n", serde_json::to_string(&final_chunk).unwrap()));
        sse_body.push_str("data: [DONE]\n\n");

        axum::response::Response::builder()
            .status(axum::http::StatusCode::OK)
            .header("content-type", "text/event-stream")
            .header("cache-control", "no-cache")
            .header("connection", "keep-alive")
            .body(axum::body::Body::from(sse_body))
            .unwrap()
    } else {
        let message = ResponseMessage {
            role: "assistant".to_string(),
            content,
            tool_calls,
        };

        let completion = ChatCompletionResponse {
            id,
            object: "chat.completion".to_string(),
            created,
            model: model_used,
            choices: vec![Choice {
                index: 0,
                message,
                finish_reason,
            }],
            usage,
        };

        axum::response::Response::builder()
            .status(axum::http::StatusCode::OK)
            .header("content-type", "application/json")
            .body(axum::body::Body::from(serde_json::to_string(&completion).unwrap()))
            .unwrap()
    }
}

fn error_response(status: axum::http::StatusCode, error: &str, message: &str) -> axum::response::Response {
    axum::response::Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(axum::body::Body::from(
            serde_json::json!({"error": error, "message": message}).to_string(),
        ))
        .unwrap()
}

pub async fn handle_list_models(
    State(state): State<AppState>,
) -> Result<Json<ModelListResponse>, AppError> {
    info!("LLM Proxy: list models request");

    let agent = state.hecate_agent.read().await;
    let llm_factory = agent
        .llm_factory
        .as_ref()
        .ok_or(AppError::AgentNotInitialized)?
        .clone();
    drop(agent);

    let factory = llm_factory.read().await;
    let free_models = factory.get_free_models().await.unwrap_or_default();

    let now = chrono::Utc::now().timestamp();

    let mut models: Vec<ModelObject> = vec![ModelObject {
        id: "auto".to_string(),
        object: "model".to_string(),
        created: now,
        owned_by: "nullblock".to_string(),
    }];

    for model in &free_models {
        if let Some(id) = model.get("id").and_then(|v| v.as_str()) {
            models.push(ModelObject {
                id: id.to_string(),
                object: "model".to_string(),
                created: model
                    .get("created")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(now),
                owned_by: "openrouter".to_string(),
            });
        }
    }

    Ok(Json(ModelListResponse {
        object: "list".to_string(),
        data: models,
    }))
}

#[derive(Debug, Deserialize)]
pub struct SetModelRequest {
    pub agent_name: String,
    pub query: String,
}

pub async fn handle_set_model_preference(
    State(state): State<AppState>,
    Json(request): Json<SetModelRequest>,
) -> Result<Json<Value>, AppError> {
    let query = request.query.to_lowercase();
    info!(
        "LLM Proxy: set model preference for agent={}, query={}",
        request.agent_name, request.query
    );

    let agent = state.hecate_agent.read().await;
    let llm_factory = agent
        .llm_factory
        .as_ref()
        .ok_or(AppError::AgentNotInitialized)?
        .clone();
    drop(agent);

    let factory = llm_factory.read().await;
    let models = factory.fetch_available_models().await.unwrap_or_default();

    let mut matches: Vec<(String, String, f64)> = Vec::new();
    for model in &models {
        let id = model.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let name = model.get("name").and_then(|v| v.as_str()).unwrap_or(id);
        let id_lower = id.to_lowercase();
        let name_lower = name.to_lowercase();

        if id_lower == query || name_lower == query {
            matches.push((id.to_string(), name.to_string(), 1.0));
        } else if id_lower.contains(&query) || name_lower.contains(&query) {
            let score = if id_lower.starts_with(&query) {
                0.9
            } else {
                0.7
            };
            matches.push((id.to_string(), name.to_string(), score));
        }
    }

    matches.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    if matches.is_empty() {
        return Ok(Json(serde_json::json!({
            "success": false,
            "error": format!("No models found matching '{}'. Try 'claude', 'gpt', 'deepseek', 'llama', or 'gemini'.", request.query)
        })));
    }

    let best = &matches[0];
    let previous = {
        let mut prefs = state.agent_model_preferences.write().await;
        let prev = prefs.get(&request.agent_name).cloned();
        prefs.insert(request.agent_name.clone(), best.0.clone());
        prev
    };

    let alternatives: Vec<Value> = matches[1..matches.len().min(6)]
        .iter()
        .map(|m| serde_json::json!({ "id": m.0, "name": m.1, "score": m.2 }))
        .collect();

    info!(
        "🔄 Model preference set for {}: {} (was {:?})",
        request.agent_name, best.0, previous
    );

    Ok(Json(serde_json::json!({
        "success": true,
        "agent_name": request.agent_name,
        "model": best.0,
        "model_name": best.1,
        "previous_model": previous,
        "alternatives": alternatives
    })))
}

pub async fn handle_get_model_preference(
    State(state): State<AppState>,
    Path(agent_name): Path<String>,
) -> Result<Json<Value>, AppError> {
    let prefs = state.agent_model_preferences.read().await;
    let model = prefs.get(&agent_name).cloned();
    Ok(Json(serde_json::json!({
        "agent_name": agent_name,
        "model": model,
    })))
}
