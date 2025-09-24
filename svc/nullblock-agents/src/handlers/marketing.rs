use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info};

use crate::{
    models::ErrorResponse,
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
    pub data: Option<crate::agents::marketing::MarketingContent>,
    pub error: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Serialize)]
pub struct TwitterPostResponse {
    pub success: bool,
    pub data: Option<crate::agents::marketing::TwitterPostResult>,
    pub error: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Serialize)]
pub struct ProjectAnalysisResponse {
    pub success: bool,
    pub data: Option<crate::agents::marketing::ProjectAnalysis>,
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

pub async fn get_marketing_health(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    info!("🏥 Checking marketing agent health");

    let marketing_agent = state.marketing_agent.read().await;
    
    let health_status = if marketing_agent.running {
        "healthy"
    } else {
        "unhealthy"
    };

    let health_data = serde_json::json!({
        "status": health_status,
        "service": "marketing_agent",
        "timestamp": chrono::Utc::now().to_rfc3339(),
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

