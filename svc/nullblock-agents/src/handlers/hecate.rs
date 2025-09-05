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
use tracing::info;

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
    let mut agent = state.hecate_agent.write().await;
    let response = agent.chat(request.message, request.user_context).await?;

    Ok(Json(json!({
        "content": response.content,
        "model_used": response.model_used,
        "latency_ms": response.latency_ms,
        "confidence_score": response.confidence_score,
        "metadata": response.metadata
    })))
}

pub async fn health(State(state): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    let agent = state.hecate_agent.read().await;
    
    if !agent.running {
        return Err(AppError::AgentNotRunning);
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
    
    // Mock available models response
    let mut available_models = Vec::new();
    
    // Add default model if available
    let default_model = "deepseek/deepseek-chat-v3.1:free";
    if agent.is_model_available(default_model, &api_keys) {
        available_models.push(json!({
            "name": default_model,
            "display_name": "DeepSeek Chat v3.1 Free",
            "icon": "🤖",
            "provider": "openrouter",
            "available": true,
            "tier": "free",
            "context_length": 128000,
            "capabilities": ["conversation", "reasoning", "creative"],
            "cost_per_1k_tokens": 0.0,
            "supports_reasoning": true,
            "description": "Free DeepSeek model optimized for conversation",
            "is_popular": true
        }));
    }

    Ok(Json(json!({
        "models": available_models,
        "current_model": agent.current_model,
        "default_model": default_model,
        "recommended_models": {
            "free": "deepseek/deepseek-chat-v3.1:free",
            "reasoning": "deepseek/deepseek-r1",
            "premium": "anthropic/claude-3.5-sonnet"
        },
        "total_models": available_models.len()
    })))
}

pub async fn search_models(
    State(state): State<AppState>,
    Query(params): Query<SearchModelsQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let agent = state.hecate_agent.read().await;
    let api_keys = state.config.get_api_keys();
    
    // Mock search functionality
    let mut results = Vec::new();
    
    let default_model = "deepseek/deepseek-chat-v3.1:free";
    if agent.is_model_available(default_model, &api_keys) {
        let model_info = json!({
            "name": default_model,
            "display_name": "DeepSeek Chat v3.1 Free",
            "icon": "🤖",
            "provider": "openrouter",
            "available": true,
            "tier": "free",
            "context_length": 128000,
            "capabilities": ["conversation", "reasoning", "creative"],
            "cost_per_1k_tokens": 0.0,
            "supports_reasoning": true,
            "description": "Free DeepSeek model optimized for conversation",
            "is_popular": true,
            "categories": ["free"]
        });

        // Simple search filtering
        if let Some(query) = &params.q {
            if query.is_empty() || default_model.to_lowercase().contains(&query.to_lowercase()) 
                || "deepseek".contains(&query.to_lowercase()) {
                results.push(model_info);
            }
        } else {
            results.push(model_info);
        }
    }

    let limit = params.limit.unwrap_or(20);
    results.truncate(limit);

    Ok(Json(json!({
        "query": params.q.unwrap_or_default(),
        "results": results,
        "total_available": 1
    })))
}

pub async fn set_model(
    State(state): State<AppState>,
    Json(request): Json<ModelSelectionRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut agent = state.hecate_agent.write().await;
    let api_keys = state.config.get_api_keys();
    
    let old_model = agent.get_preferred_model();
    let success = agent.set_preferred_model(request.model_name.clone(), &api_keys).await;
    
    if success {
        Ok(Json(json!({
            "success": true,
            "model": request.model_name,
            "previous_model": old_model
        })))
    } else {
        let error_detail = agent.get_model_availability_reason(&request.model_name, &api_keys);
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
        agent.current_model.clone().unwrap_or_else(|| "deepseek/deepseek-chat-v3.1:free".to_string())
    });
    
    if !agent.is_model_available(&model_name, &api_keys) {
        return Err(AppError::ModelNotAvailable(format!("Model {} not found", model_name)));
    }
    
    // Mock model info
    let model_info = json!({
        "name": model_name,
        "display_name": "DeepSeek Chat v3.1 Free",
        "icon": "🤖",
        "provider": "openrouter",
        "description": "Free DeepSeek model optimized for conversation",
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