use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::models::{
    AlertSeverity, BlockedEntity, ThreatAlert, ThreatCategory, ThreatEntityType,
    ThreatScore, ThreatStats, WalletAnalysis, WatchedWallet, WhitelistedEntity,
};
use crate::server::AppState;
use crate::threat::ThreatDetector;

lazy_static::lazy_static! {
    static ref THREAT_DETECTOR: ThreatDetector = ThreatDetector::default();
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: u16,
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        let status = StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, Json(self)).into_response()
    }
}

fn json_error(status: StatusCode, message: &str) -> ErrorResponse {
    ErrorResponse {
        error: message.to_string(),
        code: status.as_u16(),
    }
}

#[derive(Debug, Deserialize)]
pub struct ThreatCheckQuery {
    pub force_refresh: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct BlockedQuery {
    pub category: Option<ThreatCategory>,
    pub entity_type: Option<ThreatEntityType>,
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct AlertsQuery {
    pub severity: Option<AlertSeverity>,
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ReportThreatRequest {
    pub entity_type: ThreatEntityType,
    pub address: String,
    pub category: ThreatCategory,
    pub reason: String,
    pub evidence_url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WatchWalletRequest {
    pub wallet_address: String,
    pub related_token_mint: Option<String>,
    pub watch_reason: Option<String>,
    pub alert_on_sell: Option<bool>,
    pub alert_on_transfer: Option<bool>,
    pub alert_threshold_sol: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhitelistRequest {
    pub entity_type: ThreatEntityType,
    pub address: String,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct ThreatCheckResponse {
    pub success: bool,
    pub score: ThreatScore,
}

#[derive(Debug, Serialize)]
pub struct WalletCheckResponse {
    pub success: bool,
    pub analysis: WalletAnalysis,
}

#[derive(Debug, Serialize)]
pub struct BlockResponse {
    pub success: bool,
    pub entity: BlockedEntity,
}

#[derive(Debug, Serialize)]
pub struct WhitelistResponse {
    pub success: bool,
    pub entity: WhitelistedEntity,
}

#[derive(Debug, Serialize)]
pub struct WatchResponse {
    pub success: bool,
    pub watched: WatchedWallet,
}

#[derive(Debug, Serialize)]
pub struct AlertResponse {
    pub success: bool,
    pub alert: ThreatAlert,
}

pub async fn check_token(
    Path(mint): Path<String>,
    Query(_query): Query<ThreatCheckQuery>,
    State(_config): State<AppState>,
) -> Result<Json<ThreatCheckResponse>, ErrorResponse> {
    match THREAT_DETECTOR.check_token(&mint).await {
        Ok(score) => Ok(Json(ThreatCheckResponse {
            success: true,
            score,
        })),
        Err(e) => Err(json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("Failed to check token: {}", e),
        )),
    }
}

pub async fn check_wallet(
    Path(address): Path<String>,
    State(_config): State<AppState>,
) -> Result<Json<WalletCheckResponse>, ErrorResponse> {
    match THREAT_DETECTOR.check_wallet(&address).await {
        Ok(analysis) => Ok(Json(WalletCheckResponse {
            success: true,
            analysis,
        })),
        Err(e) => Err(json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("Failed to check wallet: {}", e),
        )),
    }
}

pub async fn list_blocked(
    Query(query): Query<BlockedQuery>,
    State(_config): State<AppState>,
) -> Json<Vec<BlockedEntity>> {
    let limit = query.limit.unwrap_or(100);
    let entities = THREAT_DETECTOR.get_blocked(query.category, limit);
    Json(entities)
}

pub async fn report_threat(
    State(_config): State<AppState>,
    Json(request): Json<ReportThreatRequest>,
) -> Json<BlockResponse> {
    let entity = THREAT_DETECTOR.block_entity(
        request.entity_type,
        request.address,
        request.category,
        request.reason,
        "user_report".to_string(),
    );

    Json(BlockResponse {
        success: true,
        entity,
    })
}

pub async fn add_watch(
    State(_config): State<AppState>,
    Json(request): Json<WatchWalletRequest>,
) -> Json<WatchResponse> {
    let mut watched = WatchedWallet::new(
        request.wallet_address,
        request.watch_reason.unwrap_or_else(|| "User requested watch".to_string()),
    );

    watched.related_token_mint = request.related_token_mint;
    watched.alert_on_sell = request.alert_on_sell.unwrap_or(true);
    watched.alert_on_transfer = request.alert_on_transfer.unwrap_or(true);
    if let Some(threshold) = request.alert_threshold_sol {
        watched.alert_threshold_sol = Some(rust_decimal::Decimal::from_f64_retain(threshold).unwrap_or_default());
    }

    let watched = THREAT_DETECTOR.add_watched_wallet(watched);

    Json(WatchResponse {
        success: true,
        watched,
    })
}

pub async fn whitelist_entity(
    State(_config): State<AppState>,
    Json(request): Json<WhitelistRequest>,
) -> Json<WhitelistResponse> {
    let entity = THREAT_DETECTOR.whitelist_entity(
        request.entity_type,
        request.address,
        request.reason,
        "user".to_string(),
    );

    Json(WhitelistResponse {
        success: true,
        entity,
    })
}

pub async fn list_whitelisted(
    Query(query): Query<BlockedQuery>,
    State(_config): State<AppState>,
) -> Json<Vec<WhitelistedEntity>> {
    let limit = query.limit.unwrap_or(100);
    let entities = THREAT_DETECTOR.get_whitelisted(limit);
    Json(entities)
}

pub async fn list_watched(
    Query(query): Query<BlockedQuery>,
    State(_config): State<AppState>,
) -> Json<Vec<WatchedWallet>> {
    let limit = query.limit.unwrap_or(100);
    let wallets = THREAT_DETECTOR.get_watched(limit);
    Json(wallets)
}

pub async fn get_alerts(
    Query(query): Query<AlertsQuery>,
    State(_config): State<AppState>,
) -> Json<Vec<ThreatAlert>> {
    let limit = query.limit.unwrap_or(50);
    let alerts = THREAT_DETECTOR.get_alerts(query.severity, limit);
    Json(alerts)
}

pub async fn get_score_history(
    Path(mint): Path<String>,
    State(_config): State<AppState>,
) -> Json<Option<ThreatScore>> {
    let score = THREAT_DETECTOR.get_score_history(&mint);
    Json(score)
}

pub async fn get_stats(
    State(_config): State<AppState>,
) -> Json<ThreatStats> {
    let stats = THREAT_DETECTOR.get_stats();
    Json(stats)
}

pub async fn remove_from_blocklist(
    Path(address): Path<String>,
    State(_config): State<AppState>,
) -> Json<serde_json::Value> {
    let removed = THREAT_DETECTOR.remove_from_blocklist(&address);
    Json(serde_json::json!({
        "success": removed,
        "address": address
    }))
}

pub async fn remove_from_whitelist(
    Path(address): Path<String>,
    State(_config): State<AppState>,
) -> Json<serde_json::Value> {
    let removed = THREAT_DETECTOR.remove_from_whitelist(&address);
    Json(serde_json::json!({
        "success": removed,
        "address": address
    }))
}

pub async fn is_blocked(
    Path(address): Path<String>,
    State(_config): State<AppState>,
) -> Json<serde_json::Value> {
    let blocked = THREAT_DETECTOR.is_blocked(&address);
    Json(serde_json::json!({
        "address": address,
        "is_blocked": blocked
    }))
}

pub async fn is_whitelisted(
    Path(address): Path<String>,
    State(_config): State<AppState>,
) -> Json<serde_json::Value> {
    let whitelisted = THREAT_DETECTOR.is_whitelisted(&address);
    Json(serde_json::json!({
        "address": address,
        "is_whitelisted": whitelisted
    }))
}
