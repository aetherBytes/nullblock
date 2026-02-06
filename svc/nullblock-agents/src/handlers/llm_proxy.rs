use crate::{
    error::AppError,
    models::LLMRequest,
    server::AppState,
};
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tracing::info;

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
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
    Json(request): Json<ChatCompletionRequest>,
) -> Result<Json<ChatCompletionResponse>, AppError> {
    info!(
        "LLM Proxy: chat completions request, model={:?}, messages={}",
        request.model,
        request.messages.len()
    );

    let messages_for_llm: Vec<HashMap<String, String>> = request
        .messages
        .iter()
        .map(|m| {
            let mut map = HashMap::new();
            map.insert("role".to_string(), m.role.clone());
            map.insert(
                "content".to_string(),
                m.content.clone().unwrap_or_default(),
            );
            map
        })
        .collect();

    let prompt = request
        .messages
        .last()
        .and_then(|m| m.content.clone())
        .unwrap_or_default();

    let system_prompt = request
        .messages
        .iter()
        .find(|m| m.role == "system")
        .and_then(|m| m.content.clone());

    let model_override = match request.model.as_deref() {
        Some("auto") | None => None,
        Some(m) => Some(m.to_string()),
    };

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
    let llm_factory = agent
        .llm_factory
        .as_ref()
        .ok_or(AppError::AgentNotInitialized)?
        .clone();
    drop(agent);

    let factory = llm_factory.read().await;
    let response = factory.generate(&llm_request, None).await?;

    let usage = Usage {
        prompt_tokens: *response.usage.get("prompt_tokens").unwrap_or(&0),
        completion_tokens: *response.usage.get("completion_tokens").unwrap_or(&0),
        total_tokens: *response.usage.get("total_tokens").unwrap_or(&0),
    };

    let message = ResponseMessage {
        role: "assistant".to_string(),
        content: if response.content.is_empty() {
            None
        } else {
            Some(response.content)
        },
        tool_calls: response.tool_calls,
    };

    let completion = ChatCompletionResponse {
        id: format!("chatcmpl-{}", uuid::Uuid::new_v4()),
        object: "chat.completion".to_string(),
        created: chrono::Utc::now().timestamp(),
        model: response.model_used,
        choices: vec![Choice {
            index: 0,
            message,
            finish_reason: response.finish_reason,
        }],
        usage,
    };

    info!("LLM Proxy: completion returned successfully");
    Ok(Json(completion))
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
