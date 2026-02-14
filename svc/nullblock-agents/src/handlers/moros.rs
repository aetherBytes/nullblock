use crate::{
    error::AppError,
    models::{ChatMessageResponse, ChatRequest, ModelSelectionRequest},
    server::AppState,
};
use axum::{extract::State, Json};
use serde_json::json;

pub async fn chat(
    State(state): State<AppState>,
    Json(request): Json<ChatRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_context = request.user_context.clone();

    let mut agent = state.moros_agent.write().await;
    let response = agent.chat(request.message, user_context.clone()).await?;

    let msg_count = agent.get_conversation_history().await.len();
    if msg_count > 0 && msg_count % 10 == 0 {
        if let Some(wallet) = user_context
            .as_ref()
            .and_then(|ctx| ctx.get("wallet_address"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
        {
            let agent_ref = state.moros_agent.clone();
            tokio::spawn(async move {
                let agent = agent_ref.read().await;
                agent.save_session_to_engrams(&wallet).await;
            });
        }
    }

    Ok(Json(json!({
        "content": response.content,
        "model_used": response.model_used,
        "latency_ms": response.latency_ms,
        "confidence_score": response.confidence_score,
        "metadata": response.metadata
    })))
}

pub async fn health(State(state): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    let agent = state.moros_agent.read().await;

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
    let agent = state.moros_agent.read().await;
    let status = agent.get_model_status().await?;
    Ok(Json(status))
}

pub async fn available_models(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let agent = state.moros_agent.read().await;

    Ok(Json(json!({
        "current_model": agent.current_model,
        "default_model": "openrouter/free",
        "message": "Use /hecate/available-models for full model catalog"
    })))
}

pub async fn set_model(
    State(state): State<AppState>,
    Json(request): Json<ModelSelectionRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let api_keys = state.api_keys.clone();
    let mut agent = state.moros_agent.write().await;
    let old_model = agent.get_preferred_model();
    let success = agent
        .set_preferred_model(request.model_name.clone(), &api_keys)
        .await;

    if success {
        Ok(Json(json!({
            "success": true,
            "model": request.model_name,
            "previous_model": old_model
        })))
    } else {
        Err(AppError::ModelNotAvailable(format!(
            "Model '{}' is not available",
            request.model_name
        )))
    }
}

pub async fn clear_conversation(
    State(state): State<AppState>,
    body: Option<Json<serde_json::Value>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let agent = state.moros_agent.read().await;
    let conversation_length = agent.get_conversation_history().await.len();

    if conversation_length >= 2 {
        if let Some(wallet) = body
            .as_ref()
            .and_then(|b| b.get("wallet_address"))
            .and_then(|v| v.as_str())
        {
            agent.save_session_to_engrams(wallet).await;
        }
    }
    drop(agent);

    let mut agent = state.moros_agent.write().await;
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
    let agent = state.moros_agent.read().await;
    let history = agent.get_conversation_history().await;

    let response_history: Vec<ChatMessageResponse> =
        history.into_iter().map(|msg| msg.into()).collect();

    Ok(Json(response_history))
}

pub async fn get_tools(State(state): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    let agent = state.moros_agent.read().await;
    let tools = agent.get_mcp_tools().await?;

    Ok(Json(json!({
        "status": "success",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "data": tools
    })))
}
