use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info};

use crate::{
    models::{ErrorResponse, ChatRequest},
    server::AppState,
};

#[derive(Debug, Deserialize)]
pub struct GenerateContentRequest {
    pub content_type: String,
    pub context: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTwitterPostRequest {
    pub content: String,
    pub media_urls: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct MarketingContentResponse {
    pub success: bool,
    pub data: Option<crate::agents::siren_marketing::MarketingContent>,
    pub error: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Serialize)]
pub struct TwitterPostResponse {
    pub success: bool,
    pub data: Option<crate::agents::siren_marketing::TwitterPostResult>,
    pub error: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Serialize)]
pub struct ProjectAnalysisResponse {
    pub success: bool,
    pub data: Option<crate::agents::siren_marketing::ProjectAnalysis>,
    pub error: Option<String>,
    pub timestamp: String,
}

pub async fn generate_content(
    State(state): State<AppState>,
    Json(request): Json<GenerateContentRequest>,
) -> Result<Json<MarketingContentResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("📝 Generating marketing content: {}", request.content_type);

    let mut marketing_agent = state.marketing_agent.write().await;
    
    match marketing_agent.generate_content(request.content_type, request.context).await {
        Ok(content) => {
            info!("✅ Content generated successfully");
            Ok(Json(MarketingContentResponse {
                success: true,
                data: Some(content),
                error: None,
                timestamp: chrono::Utc::now().to_rfc3339(),
            }))
        }
        Err(e) => {
            error!("❌ Failed to generate content: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(
                    "content_generation_failed".to_string(),
                    format!("Failed to generate content: {}", e),
                )),
            ))
        }
    }
}

pub async fn create_twitter_post(
    State(state): State<AppState>,
    Json(request): Json<CreateTwitterPostRequest>,
) -> Result<Json<TwitterPostResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("📱 Creating Twitter post");

    let mut marketing_agent = state.marketing_agent.write().await;
    
    match marketing_agent.create_twitter_post(request.content, request.media_urls).await {
        Ok(result) => {
            info!("✅ Twitter post created successfully");
            Ok(Json(TwitterPostResponse {
                success: true,
                data: Some(result),
                error: None,
                timestamp: chrono::Utc::now().to_rfc3339(),
            }))
        }
        Err(e) => {
            error!("❌ Failed to create Twitter post: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(
                    "twitter_post_failed".to_string(),
                    format!("Failed to create Twitter post: {}", e),
                )),
            ))
        }
    }
}

pub async fn analyze_project_progress(
    State(state): State<AppState>,
) -> Result<Json<ProjectAnalysisResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("🔍 Analyzing project progress for marketing opportunities");

    let mut marketing_agent = state.marketing_agent.write().await;
    
    match marketing_agent.analyze_project_progress().await {
        Ok(analysis) => {
            info!("✅ Project analysis completed");
            Ok(Json(ProjectAnalysisResponse {
                success: true,
                data: Some(analysis),
                error: None,
                timestamp: chrono::Utc::now().to_rfc3339(),
            }))
        }
        Err(e) => {
            error!("❌ Failed to analyze project progress: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(
                    "project_analysis_failed".to_string(),
                    format!("Failed to analyze project progress: {}", e),
                )),
            ))
        }
    }
}

pub async fn get_siren_health(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    info!("🏥 Checking Siren agent health");

    let marketing_agent = state.marketing_agent.read().await;
    
    let health_status = if marketing_agent.running {
        "healthy"
    } else {
        "unhealthy"
    };

    let health_data = serde_json::json!({
        "status": health_status,
        "service": "siren_agent",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "current_model": marketing_agent.current_model.clone(),
        "preferred_model": marketing_agent.preferred_model.clone(),
        "components": {
            "llm_factory": if marketing_agent.llm_factory.is_some() { "ready" } else { "not_initialized" },
            "twitter_integration": if marketing_agent.twitter_api_key.is_some() { "configured" } else { "not_configured" },
            "content_themes": marketing_agent.content_themes.len(),
            "agent_id": marketing_agent.agent_id,
        }
    });

    Ok(Json(health_data))
}

pub async fn get_content_themes(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    info!("🎨 Getting available content themes");

    let marketing_agent = state.marketing_agent.read().await;
    
    let themes: Vec<serde_json::Value> = marketing_agent.content_themes
        .iter()
        .map(|(key, theme)| {
            serde_json::json!({
                "id": key,
                "name": theme.name,
                "description": theme.description,
                "hashtags": theme.hashtags,
                "tone": theme.tone,
                "target_audience": theme.target_audience,
                "content_templates": theme.content_templates
            })
        })
        .collect();

    let response = serde_json::json!({
        "success": true,
        "data": themes,
        "total": themes.len(),
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    Ok(Json(response))
}

pub async fn chat(
    State(state): State<AppState>,
    Json(request): Json<ChatRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    info!("🎭 Marketing agent chat request received");

    let mut marketing_agent = state.marketing_agent.write().await;

    match marketing_agent.chat(request.message, request.user_context).await {
        Ok(response) => {
            info!("✅ Marketing chat response generated: {} chars", response.content.len());

            // Extract latency_ms and confidence_score from metadata
            let latency_ms = response.metadata
                .as_ref()
                .and_then(|meta| meta.get("latency_ms"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            let confidence_score = response.metadata
                .as_ref()
                .and_then(|meta| meta.get("confidence_score"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.85);

            Ok(Json(serde_json::json!({
                "content": response.content,
                "model_used": response.model_used,
                "latency_ms": latency_ms,
                "confidence_score": confidence_score,
                "metadata": response.metadata
            })))
        }
        Err(e) => {
            error!("❌ Marketing chat failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(
                    "marketing_chat_failed".to_string(),
                    format!("Marketing chat failed: {}", e),
                )),
            ))
        }
    }
}

pub async fn set_model(
    State(state): State<AppState>,
    Json(request): Json<crate::models::ModelSelectionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    info!("🎯 Siren model selection request: {}", request.model_name);

    let mut marketing_agent = state.marketing_agent.write().await;
    let api_keys = state.config.get_api_keys();

    let success = marketing_agent.set_preferred_model(request.model_name.clone(), &api_keys).await;

    if success {
        info!("✅ Siren model successfully set to: {}", request.model_name);
        Ok(Json(serde_json::json!({
            "success": true,
            "model": request.model_name,
            "message": "Model successfully updated"
        })))
    } else {
        error!("❌ Failed to set Siren model to: {}", request.model_name);
        Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "model_not_available".to_string(),
                format!("Model {} is not available", request.model_name),
            )),
        ))
    }
}




