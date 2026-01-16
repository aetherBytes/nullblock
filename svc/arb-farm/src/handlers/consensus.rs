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
