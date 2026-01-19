use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::consensus::{
    format_edge_context, get_default_models, ConsensusResult, ModelVote, AVAILABLE_MODELS,
};
use crate::error::AppResult;
use crate::server::AppState;

lazy_static::lazy_static! {
    static ref CONSENSUS_HISTORY: Arc<RwLock<HashMap<Uuid, ConsensusHistoryEntry>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusHistoryEntry {
    pub id: Uuid,
    pub edge_id: Uuid,
    pub result: ConsensusResult,
    pub edge_context: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ListConsensusQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub approved_only: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ConsensusListResponse {
    pub decisions: Vec<ConsensusHistoryEntry>,
    pub total: usize,
}

pub async fn list_consensus_history(
    State(_state): State<AppState>,
    Query(query): Query<ListConsensusQuery>,
) -> impl IntoResponse {
    let history = CONSENSUS_HISTORY.read().await;

    let mut decisions: Vec<ConsensusHistoryEntry> = history
        .values()
        .filter(|entry| {
            if let Some(approved_only) = query.approved_only {
                entry.result.approved == approved_only
            } else {
                true
            }
        })
        .cloned()
        .collect();

    decisions.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    let total = decisions.len();
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(50);

    let decisions: Vec<ConsensusHistoryEntry> = decisions
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();

    (
        StatusCode::OK,
        Json(ConsensusListResponse { decisions, total }),
    )
}

#[derive(Debug, Serialize)]
pub struct ConsensusDetailResponse {
    pub id: Uuid,
    pub edge_id: Uuid,
    pub approved: bool,
    pub agreement_score: f64,
    pub weighted_confidence: f64,
    pub reasoning_summary: String,
    pub model_votes: Vec<ModelVote>,
    pub edge_context: String,
    pub total_latency_ms: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn get_consensus_detail(
    State(_state): State<AppState>,
    Path(consensus_id): Path<Uuid>,
) -> impl IntoResponse {
    let history = CONSENSUS_HISTORY.read().await;

    match history.get(&consensus_id) {
        Some(entry) => (
            StatusCode::OK,
            Json(serde_json::json!(ConsensusDetailResponse {
                id: entry.id,
                edge_id: entry.edge_id,
                approved: entry.result.approved,
                agreement_score: entry.result.agreement_score,
                weighted_confidence: entry.result.weighted_confidence,
                reasoning_summary: entry.result.reasoning_summary.clone(),
                model_votes: entry.result.model_votes.clone(),
                edge_context: entry.edge_context.clone(),
                total_latency_ms: entry.result.total_latency_ms,
                created_at: entry.created_at,
            })),
        )
            .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Consensus decision not found",
                "consensus_id": consensus_id
            })),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct RequestConsensusRequest {
    pub edge_id: Option<Uuid>,
    pub edge_type: String,
    pub venue: String,
    pub token_pair: Vec<String>,
    pub estimated_profit_lamports: i64,
    pub risk_score: i32,
    pub route_data: serde_json::Value,
    pub models: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct RequestConsensusResponse {
    pub consensus_id: Uuid,
    pub edge_id: Uuid,
    pub approved: bool,
    pub agreement_score: f64,
    pub weighted_confidence: f64,
    pub reasoning_summary: String,
    pub model_votes: Vec<ModelVoteResponse>,
    pub total_latency_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct ModelVoteResponse {
    pub model: String,
    pub approved: bool,
    pub confidence: f64,
    pub reasoning: String,
    pub latency_ms: u64,
}

pub async fn request_consensus(
    State(state): State<AppState>,
    Json(request): Json<RequestConsensusRequest>,
) -> impl IntoResponse {
    if state.config.openrouter_api_key.is_none() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "error": "Consensus engine not configured - OpenRouter API key missing"
            })),
        )
            .into_response();
    }

    let edge_id = request.edge_id.unwrap_or_else(Uuid::new_v4);

    let edge_context = format_edge_context(
        &request.edge_type,
        &request.venue,
        &request.token_pair,
        request.estimated_profit_lamports,
        request.risk_score,
        &request.route_data,
    );

    let result = match state
        .consensus_engine
        .request_consensus(edge_id, &edge_context, request.models)
        .await
    {
        Ok(result) => result,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Consensus request failed: {}", e)
                })),
            )
                .into_response();
        }
    };

    let consensus_id = Uuid::new_v4();

    let event = state.consensus_engine.create_consensus_event(edge_id, &result);
    let _ = state.event_tx.send(event);

    {
        let mut history = CONSENSUS_HISTORY.write().await;
        history.insert(
            consensus_id,
            ConsensusHistoryEntry {
                id: consensus_id,
                edge_id,
                result: result.clone(),
                edge_context,
                created_at: chrono::Utc::now(),
            },
        );
    }

    let model_votes: Vec<ModelVoteResponse> = result
        .model_votes
        .iter()
        .map(|v| ModelVoteResponse {
            model: v.model.clone(),
            approved: v.approved,
            confidence: v.confidence,
            reasoning: v.reasoning.clone(),
            latency_ms: v.latency_ms,
        })
        .collect();

    (
        StatusCode::OK,
        Json(RequestConsensusResponse {
            consensus_id,
            edge_id,
            approved: result.approved,
            agreement_score: result.agreement_score,
            weighted_confidence: result.weighted_confidence,
            reasoning_summary: result.reasoning_summary,
            model_votes,
            total_latency_ms: result.total_latency_ms,
        }),
    )
        .into_response()
}

#[derive(Debug, Serialize)]
pub struct AvailableModelsResponse {
    pub models: Vec<ModelInfo>,
    pub default_models: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: String,
}

pub async fn list_available_models(State(_state): State<AppState>) -> impl IntoResponse {
    let models: Vec<ModelInfo> = AVAILABLE_MODELS
        .iter()
        .map(|(id, display_name, _weight)| {
            let parts: Vec<&str> = id.split('/').collect();
            let provider = if parts.len() >= 2 {
                parts[0].to_string()
            } else {
                "unknown".to_string()
            };

            ModelInfo {
                id: id.to_string(),
                name: display_name.to_string(),
                provider,
            }
        })
        .collect();

    (
        StatusCode::OK,
        Json(AvailableModelsResponse {
            models,
            default_models: get_default_models(),
        }),
    )
}

#[derive(Debug, Serialize)]
pub struct ConsensusStatsResponse {
    pub total_decisions: usize,
    pub approved_count: usize,
    pub rejected_count: usize,
    pub average_agreement: f64,
    pub average_confidence: f64,
    pub average_latency_ms: f64,
    pub decisions_last_24h: usize,
}

pub async fn get_consensus_stats(State(_state): State<AppState>) -> impl IntoResponse {
    let history = CONSENSUS_HISTORY.read().await;

    if history.is_empty() {
        return (
            StatusCode::OK,
            Json(ConsensusStatsResponse {
                total_decisions: 0,
                approved_count: 0,
                rejected_count: 0,
                average_agreement: 0.0,
                average_confidence: 0.0,
                average_latency_ms: 0.0,
                decisions_last_24h: 0,
            }),
        );
    }

    let total = history.len();
    let approved = history.values().filter(|e| e.result.approved).count();
    let rejected = total - approved;

    let avg_agreement: f64 =
        history.values().map(|e| e.result.agreement_score).sum::<f64>() / total as f64;

    let avg_confidence: f64 =
        history.values().map(|e| e.result.weighted_confidence).sum::<f64>() / total as f64;

    let avg_latency: f64 =
        history.values().map(|e| e.result.total_latency_ms as f64).sum::<f64>() / total as f64;

    let twenty_four_hours_ago = chrono::Utc::now() - chrono::Duration::hours(24);
    let decisions_24h = history
        .values()
        .filter(|e| e.created_at > twenty_four_hours_ago)
        .count();

    (
        StatusCode::OK,
        Json(ConsensusStatsResponse {
            total_decisions: total,
            approved_count: approved,
            rejected_count: rejected,
            average_agreement: avg_agreement,
            average_confidence: avg_confidence,
            average_latency_ms: avg_latency,
            decisions_last_24h: decisions_24h,
        }),
    )
}

lazy_static::lazy_static! {
    static ref CONSENSUS_CONFIG: Arc<RwLock<crate::consensus::ConsensusConfig>> =
        Arc::new(RwLock::new(crate::consensus::ConsensusConfig::default()));
    static ref CONVERSATION_HISTORY: Arc<RwLock<Vec<crate::engrams::ConversationLog>>> =
        Arc::new(RwLock::new(Vec::new()));
}

#[derive(Debug, Serialize)]
pub struct ConsensusConfigResponse {
    pub config: crate::consensus::ConsensusConfig,
    pub is_dev_wallet: bool,
    pub available_models: Vec<crate::consensus::ConsensusModelConfig>,
}

pub async fn get_consensus_config(State(state): State<AppState>) -> impl IntoResponse {
    let config = CONSENSUS_CONFIG.read().await.clone();
    let wallet = state.config.wallet_address.clone().unwrap_or_default();
    let is_dev = crate::consensus::is_dev_wallet(&wallet);
    let available = crate::consensus::get_all_available_models();

    (
        StatusCode::OK,
        Json(ConsensusConfigResponse {
            config,
            is_dev_wallet: is_dev,
            available_models: available,
        }),
    )
}

pub async fn update_consensus_config(
    State(state): State<AppState>,
    Json(request): Json<crate::consensus::UpdateConsensusConfigRequest>,
) -> impl IntoResponse {
    let mut config = CONSENSUS_CONFIG.write().await;

    if let Some(enabled) = request.enabled {
        config.enabled = enabled;
    }
    if let Some(models) = request.models {
        config.models = models;
    }
    if let Some(threshold) = request.min_consensus_threshold {
        config.min_consensus_threshold = threshold.clamp(0.0, 1.0);
    }
    if let Some(auto_apply) = request.auto_apply_recommendations {
        config.auto_apply_recommendations = auto_apply;
    }
    if let Some(interval) = request.review_interval_hours {
        config.review_interval_hours = interval;
    }

    let wallet = state.config.wallet_address.clone().unwrap_or_default();
    let is_dev = crate::consensus::is_dev_wallet(&wallet);

    (
        StatusCode::OK,
        Json(ConsensusConfigResponse {
            config: config.clone(),
            is_dev_wallet: is_dev,
            available_models: crate::consensus::get_all_available_models(),
        }),
    )
}

pub async fn reset_consensus_config(State(state): State<AppState>) -> impl IntoResponse {
    let wallet = state.config.wallet_address.clone().unwrap_or_default();
    let models = crate::consensus::get_models_for_wallet(&wallet);

    let mut config = CONSENSUS_CONFIG.write().await;
    *config = crate::consensus::ConsensusConfig::default();
    config.models = models;

    let is_dev = crate::consensus::is_dev_wallet(&wallet);

    (
        StatusCode::OK,
        Json(ConsensusConfigResponse {
            config: config.clone(),
            is_dev_wallet: is_dev,
            available_models: crate::consensus::get_all_available_models(),
        }),
    )
}

#[derive(Debug, Deserialize)]
pub struct ListConversationsQuery {
    pub limit: Option<usize>,
    pub topic: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ConversationListResponse {
    pub conversations: Vec<crate::engrams::ConversationLog>,
    pub total: usize,
}

pub async fn list_conversations(
    State(state): State<AppState>,
    Query(query): Query<ListConversationsQuery>,
) -> impl IntoResponse {
    let wallet = state.config.wallet_address.clone().unwrap_or_else(|| "default".to_string());

    match state.engrams_client.get_conversations(&wallet, query.limit.map(|l| l as i64)).await {
        Ok(mut conversations) => {
            if let Some(topic) = query.topic {
                conversations.retain(|c| {
                    let topic_str = serde_json::to_string(&c.topic)
                        .unwrap_or_default()
                        .trim_matches('"')
                        .to_string();
                    topic_str.to_lowercase().contains(&topic.to_lowercase())
                });
            }

            let total = conversations.len();

            (
                StatusCode::OK,
                Json(ConversationListResponse {
                    conversations,
                    total,
                }),
            ).into_response()
        }
        Err(e) => {
            let in_memory = CONVERSATION_HISTORY.read().await.clone();
            let total = in_memory.len();

            tracing::warn!("Failed to fetch conversations from engrams, using in-memory: {}", e);

            (
                StatusCode::OK,
                Json(ConversationListResponse {
                    conversations: in_memory,
                    total,
                }),
            ).into_response()
        }
    }
}

pub async fn get_conversation_detail(
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
) -> impl IntoResponse {
    let wallet = state.config.wallet_address.clone().unwrap_or_else(|| "default".to_string());

    match state.engrams_client.get_conversations(&wallet, Some(100)).await {
        Ok(conversations) => {
            if let Some(conv) = conversations.into_iter().find(|c| c.session_id == session_id) {
                (StatusCode::OK, Json(serde_json::json!(conv))).into_response()
            } else {
                (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({
                        "error": "Conversation not found",
                        "session_id": session_id
                    })),
                ).into_response()
            }
        }
        Err(e) => {
            let in_memory = CONVERSATION_HISTORY.read().await;
            if let Some(conv) = in_memory.iter().find(|c| c.session_id == session_id) {
                (StatusCode::OK, Json(serde_json::json!(conv))).into_response()
            } else {
                (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({
                        "error": format!("Conversation not found: {}", e),
                        "session_id": session_id
                    })),
                ).into_response()
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListRecommendationsQuery {
    pub status: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct RecommendationListResponse {
    pub recommendations: Vec<crate::engrams::Recommendation>,
    pub total: usize,
}

pub async fn list_recommendations(
    State(state): State<AppState>,
    Query(query): Query<ListRecommendationsQuery>,
) -> impl IntoResponse {
    let wallet = state.config.wallet_address.clone().unwrap_or_else(|| "default".to_string());

    let status_filter = query.status.as_ref().and_then(|s| {
        match s.to_lowercase().as_str() {
            "pending" => Some(crate::engrams::RecommendationStatus::Pending),
            "acknowledged" => Some(crate::engrams::RecommendationStatus::Acknowledged),
            "applied" => Some(crate::engrams::RecommendationStatus::Applied),
            "rejected" => Some(crate::engrams::RecommendationStatus::Rejected),
            _ => None,
        }
    });

    match state.engrams_client.get_recommendations(
        &wallet,
        status_filter.as_ref(),
        query.limit.map(|l| l as i64),
    ).await {
        Ok(recommendations) => {
            let total = recommendations.len();
            (
                StatusCode::OK,
                Json(RecommendationListResponse {
                    recommendations,
                    total,
                }),
            )
        }
        Err(e) => {
            tracing::error!("Failed to fetch recommendations: {}", e);
            (
                StatusCode::OK,
                Json(RecommendationListResponse {
                    recommendations: vec![],
                    total: 0,
                }),
            )
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateRecommendationStatusRequest {
    pub status: String,
}

pub async fn update_recommendation_status(
    State(state): State<AppState>,
    Path(recommendation_id): Path<Uuid>,
    Json(request): Json<UpdateRecommendationStatusRequest>,
) -> impl IntoResponse {
    let wallet = state.config.wallet_address.clone().unwrap_or_else(|| "default".to_string());

    let new_status = match request.status.to_lowercase().as_str() {
        "pending" => crate::engrams::RecommendationStatus::Pending,
        "acknowledged" => crate::engrams::RecommendationStatus::Acknowledged,
        "applied" => crate::engrams::RecommendationStatus::Applied,
        "rejected" => crate::engrams::RecommendationStatus::Rejected,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Invalid status. Must be: pending, acknowledged, applied, or rejected"
                })),
            ).into_response();
        }
    };

    match state.engrams_client.update_recommendation_status(&wallet, &recommendation_id, new_status).await {
        Ok(engram) => {
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "engram_id": engram.id,
                    "recommendation_id": recommendation_id
                })),
            ).into_response()
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to update recommendation: {}", e)
                })),
            ).into_response()
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListEngramsQuery {
    pub engram_type: Option<String>,
    pub tags: Option<String>,
    pub limit: Option<i64>,
    pub query: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EngramListResponse {
    pub engrams: Vec<crate::engrams::Engram>,
    pub total: usize,
}

pub async fn list_engrams(
    State(state): State<AppState>,
    Query(query): Query<ListEngramsQuery>,
) -> impl IntoResponse {
    let wallet = state.config.wallet_address.clone().unwrap_or_else(|| "default".to_string());

    let tags = query.tags.map(|t| {
        t.split(',')
            .map(|s| s.trim().to_string())
            .collect::<Vec<_>>()
    });

    let search = crate::engrams::SearchRequest {
        wallet_address: Some(wallet),
        engram_type: query.engram_type,
        query: query.query,
        tags,
        limit: query.limit.or(Some(50)),
        offset: None,
    };

    match state.engrams_client.search_engrams(search).await {
        Ok(engrams) => {
            let total = engrams.len();
            (
                StatusCode::OK,
                Json(EngramListResponse { engrams, total }),
            )
        }
        Err(e) => {
            tracing::error!("Failed to fetch engrams: {}", e);
            (
                StatusCode::OK,
                Json(EngramListResponse {
                    engrams: vec![],
                    total: 0,
                }),
            )
        }
    }
}

pub async fn get_engram_detail(
    State(state): State<AppState>,
    Path(engram_key): Path<String>,
) -> impl IntoResponse {
    let wallet = state.config.wallet_address.clone().unwrap_or_else(|| "default".to_string());

    match state.engrams_client.get_engram_by_wallet_key(&wallet, &engram_key).await {
        Ok(Some(engram)) => {
            (StatusCode::OK, Json(serde_json::json!(engram))).into_response()
        }
        Ok(None) => {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "Engram not found",
                    "key": engram_key
                })),
            ).into_response()
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to fetch engram: {}", e)
                })),
            ).into_response()
        }
    }
}

pub async fn get_learning_summary(State(state): State<AppState>) -> impl IntoResponse {
    let wallet = state.config.wallet_address.clone().unwrap_or_else(|| "default".to_string());

    let recommendations = state.engrams_client.get_recommendations(&wallet, None, Some(100)).await.unwrap_or_default();
    let conversations = state.engrams_client.get_conversations(&wallet, Some(20)).await.unwrap_or_default();

    let pending_recommendations = recommendations.iter()
        .filter(|r| r.status == crate::engrams::RecommendationStatus::Pending)
        .count();

    let applied_recommendations = recommendations.iter()
        .filter(|r| r.status == crate::engrams::RecommendationStatus::Applied)
        .count();

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "total_recommendations": recommendations.len(),
            "pending_recommendations": pending_recommendations,
            "applied_recommendations": applied_recommendations,
            "total_conversations": conversations.len(),
            "recent_conversations": conversations.into_iter().take(5).collect::<Vec<_>>(),
            "recent_recommendations": recommendations.into_iter().take(5).collect::<Vec<_>>(),
        })),
    )
}

pub fn save_conversation_to_memory(conversation: crate::engrams::ConversationLog) {
    tokio::spawn(async move {
        let mut history = CONVERSATION_HISTORY.write().await;
        history.push(conversation);
        if history.len() > 100 {
            history.remove(0);
        }
    });
}

pub async fn get_model_discovery_status(State(_state): State<AppState>) -> impl IntoResponse {
    let status = crate::consensus::get_discovery_status().await;
    (StatusCode::OK, Json(status))
}

pub async fn refresh_models(State(state): State<AppState>) -> impl IntoResponse {
    if let Some(ref api_key) = state.config.openrouter_api_key {
        let models = crate::consensus::refresh_models(api_key).await;

        let mut config = CONSENSUS_CONFIG.write().await;
        config.models = models.clone();

        (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "models_discovered": models.len(),
                "models": models,
            })),
        ).into_response()
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "error": "OpenRouter API key not configured"
            })),
        ).into_response()
    }
}

pub async fn get_discovered_models(State(_state): State<AppState>) -> impl IntoResponse {
    let models = crate::consensus::get_discovered_models().await;
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "models": models,
            "count": models.len(),
        })),
    )
}
