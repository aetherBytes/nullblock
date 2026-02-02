use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::error::AppResult;
use crate::helius::laserstream::LaserStreamStatus;
use crate::helius::priority_fee::PriorityFeeResponse;
use crate::helius::types::{HeliusConfig, HeliusStatus, SenderStats, TokenMetadata};
use crate::server::AppState;

#[derive(Debug, Serialize)]
pub struct HeliusStatusResponse {
    pub connected: bool,
    pub api_key_configured: bool,
    pub laserstream_enabled: bool,
    pub sender_enabled: bool,
    pub rpc_url: String,
    pub sender_url: String,
}

pub async fn get_helius_status(
    State(state): State<AppState>,
) -> AppResult<Json<HeliusStatusResponse>> {
    let config = state.helius_rpc_client.get_config().await;

    Ok(Json(HeliusStatusResponse {
        connected: state.helius_rpc_client.is_configured(),
        api_key_configured: state.config.helius_api_key.is_some(),
        laserstream_enabled: config.laserstream_enabled,
        sender_enabled: config.use_helius_sender,
        rpc_url: state.config.helius_api_url.clone(),
        sender_url: state.config.helius_sender_url.clone(),
    }))
}

#[derive(Debug, Serialize)]
pub struct LaserStreamStatusResponse {
    pub connected: bool,
    pub subscriptions: Vec<LaserStreamSubscription>,
    pub avg_latency_ms: f64,
    pub events_per_second: f64,
}

#[derive(Debug, Serialize)]
pub struct LaserStreamSubscription {
    pub id: String,
    pub subscription_type: String,
    pub address: Option<String>,
    pub events_received: u64,
}

pub async fn get_laserstream_status(
    State(state): State<AppState>,
) -> AppResult<Json<LaserStreamStatusResponse>> {
    let status = state.laserstream_client.get_status().await;
    let connected = status == LaserStreamStatus::Connected;
    let subscribed_accounts = state.laserstream_client.get_subscribed_accounts().await;

    let subscriptions: Vec<LaserStreamSubscription> = subscribed_accounts
        .into_iter()
        .enumerate()
        .map(|(i, addr)| LaserStreamSubscription {
            id: format!("sub_{}", i),
            subscription_type: "account".to_string(),
            address: Some(addr),
            events_received: 0,
        })
        .collect();

    Ok(Json(LaserStreamStatusResponse {
        connected,
        subscriptions,
        avg_latency_ms: 0.0,
        events_per_second: 0.0,
    }))
}

#[derive(Debug, Deserialize)]
pub struct PriorityFeeQuery {
    pub account_keys: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PriorityFeeApiResponse {
    pub min: u64,
    pub low: u64,
    pub medium: u64,
    pub high: u64,
    pub very_high: u64,
    pub unsafe_max: u64,
    pub recommended: u64,
}

pub async fn get_priority_fees(
    State(state): State<AppState>,
    Query(query): Query<PriorityFeeQuery>,
) -> AppResult<Json<PriorityFeeApiResponse>> {
    let account_keys: Option<Vec<String>> = query
        .account_keys
        .map(|s| s.split(',').map(|k| k.trim().to_string()).collect());

    let fees = state
        .priority_fee_monitor
        .get_priority_fee_estimate(None, account_keys.as_deref())
        .await?;

    Ok(Json(PriorityFeeApiResponse {
        min: fees.min,
        low: fees.low,
        medium: fees.medium,
        high: fees.high,
        very_high: fees.very_high,
        unsafe_max: fees.unsafe_max,
        recommended: fees.recommended,
    }))
}

pub async fn get_cached_priority_fees(
    State(state): State<AppState>,
) -> AppResult<Json<Option<PriorityFeeApiResponse>>> {
    let cached = state.priority_fee_monitor.get_cached_fees().await;

    Ok(Json(cached.map(|fees| PriorityFeeApiResponse {
        min: fees.min,
        low: fees.low,
        medium: fees.medium,
        high: fees.high,
        very_high: fees.very_high,
        unsafe_max: fees.unsafe_max,
        recommended: fees.recommended,
    })))
}

#[derive(Debug, Serialize)]
pub struct SenderStatsResponse {
    pub total_sent: u64,
    pub total_confirmed: u64,
    pub total_failed: u64,
    pub success_rate: f64,
    pub avg_landing_ms: f64,
}

pub async fn get_sender_stats(
    State(state): State<AppState>,
) -> AppResult<Json<SenderStatsResponse>> {
    let stats = state.helius_sender.get_stats().await;

    Ok(Json(SenderStatsResponse {
        total_sent: stats.total_sent,
        total_confirmed: stats.total_confirmed,
        total_failed: stats.total_failed,
        success_rate: stats.success_rate,
        avg_landing_ms: stats.avg_landing_ms,
    }))
}

#[derive(Debug, Serialize)]
pub struct PingResponse {
    pub latency_ms: u64,
}

pub async fn ping_sender(State(state): State<AppState>) -> AppResult<Json<PingResponse>> {
    let latency = state.helius_sender.ping().await?;

    Ok(Json(PingResponse {
        latency_ms: latency,
    }))
}

#[derive(Debug, Deserialize)]
pub struct DasLookupRequest {
    pub mint: String,
}

pub async fn das_lookup(
    State(state): State<AppState>,
    Json(request): Json<DasLookupRequest>,
) -> AppResult<Json<TokenMetadata>> {
    let metadata = state.helius_das.get_asset(&request.mint).await?;
    Ok(Json(metadata))
}

#[derive(Debug, Deserialize)]
pub struct DasOwnerQuery {
    pub owner: String,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct DasAssetsResponse {
    pub assets: Vec<TokenMetadata>,
    pub total: usize,
}

pub async fn das_assets_by_owner(
    State(state): State<AppState>,
    Query(query): Query<DasOwnerQuery>,
) -> AppResult<Json<DasAssetsResponse>> {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);

    let assets = state
        .helius_das
        .get_assets_by_owner(&query.owner, page, limit)
        .await?;

    let total = assets.len();

    Ok(Json(DasAssetsResponse { assets, total }))
}

pub async fn get_helius_config(State(state): State<AppState>) -> AppResult<Json<HeliusConfig>> {
    let config = state.helius_rpc_client.get_config().await;
    Ok(Json(config))
}

#[derive(Debug, Deserialize)]
pub struct UpdateHeliusConfigRequest {
    pub laserstream_enabled: Option<bool>,
    pub default_priority_level: Option<String>,
    pub use_helius_sender: Option<bool>,
}

pub async fn update_helius_config(
    State(state): State<AppState>,
    Json(request): Json<UpdateHeliusConfigRequest>,
) -> AppResult<Json<HeliusConfig>> {
    let mut current = state.helius_rpc_client.get_config().await;

    if let Some(enabled) = request.laserstream_enabled {
        current.laserstream_enabled = enabled;
    }
    if let Some(level) = request.default_priority_level {
        current.default_priority_level = level;
    }
    if let Some(enabled) = request.use_helius_sender {
        current.use_helius_sender = enabled;
    }

    state.helius_rpc_client.update_config(current.clone()).await;

    Ok(Json(current))
}

#[derive(Debug, Deserialize)]
pub struct SendTransactionRequest {
    pub signed_transaction_base64: String,
    #[serde(default)]
    pub skip_preflight: bool,
}

#[derive(Debug, Serialize)]
pub struct SendTransactionResponse {
    pub success: bool,
    pub signature: Option<String>,
    pub error: Option<String>,
}

pub async fn send_transaction(
    State(state): State<AppState>,
    Json(request): Json<SendTransactionRequest>,
) -> AppResult<Json<SendTransactionResponse>> {
    match state
        .helius_sender
        .send_transaction(&request.signed_transaction_base64, request.skip_preflight)
        .await
    {
        Ok(signature) => Ok(Json(SendTransactionResponse {
            success: true,
            signature: Some(signature),
            error: None,
        })),
        Err(e) => Ok(Json(SendTransactionResponse {
            success: false,
            signature: None,
            error: Some(e.to_string()),
        })),
    }
}
