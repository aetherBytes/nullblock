use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::agents::{ScannerStatus, VenueStatus};
use crate::models::VenueType;
use crate::server::AppState;

#[derive(Debug, Serialize)]
pub struct ScannerStatusResponse {
    pub id: String,
    pub is_running: bool,
    pub scan_interval_ms: u64,
    pub stats: ScannerStatsResponse,
    pub venues: Vec<VenueStatusResponse>,
}

#[derive(Debug, Serialize)]
pub struct ScannerStatsResponse {
    pub total_scans: u64,
    pub total_signals_detected: u64,
    pub last_scan_at: Option<String>,
    pub healthy_venues: u32,
    pub total_venues: u32,
}

#[derive(Debug, Serialize)]
pub struct VenueStatusResponse {
    pub id: String,
    pub name: String,
    pub venue_type: String,
    pub is_healthy: bool,
}

#[derive(Debug, Serialize)]
pub struct SignalResponse {
    pub id: String,
    pub signal_type: String,
    pub venue_id: String,
    pub venue_type: String,
    pub token_mint: Option<String>,
    pub pool_address: Option<String>,
    pub estimated_profit_bps: i32,
    pub confidence: f64,
    pub significance: String,
    pub metadata: serde_json::Value,
    pub detected_at: String,
    pub expires_at: String,
}

#[derive(Debug, Deserialize)]
pub struct SignalQuery {
    pub venue_type: Option<String>,
    pub min_profit_bps: Option<i32>,
    pub min_confidence: Option<f64>,
    pub limit: Option<usize>,
}

impl From<ScannerStatus> for ScannerStatusResponse {
    fn from(status: ScannerStatus) -> Self {
        Self {
            id: status.id.to_string(),
            is_running: status.is_running,
            scan_interval_ms: status.scan_interval_ms,
            stats: ScannerStatsResponse {
                total_scans: status.stats.total_scans,
                total_signals_detected: status.stats.total_signals_detected,
                last_scan_at: status.stats.last_scan_at.map(|t| t.to_rfc3339()),
                healthy_venues: status.stats.healthy_venues,
                total_venues: status.stats.total_venues,
            },
            venues: status.venue_statuses.into_iter().map(VenueStatusResponse::from).collect(),
        }
    }
}

impl From<VenueStatus> for VenueStatusResponse {
    fn from(status: VenueStatus) -> Self {
        Self {
            id: status.id.to_string(),
            name: status.name,
            venue_type: format!("{:?}", status.venue_type),
            is_healthy: status.is_healthy,
        }
    }
}

pub async fn get_scanner_status(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let status = state.scanner.get_status().await;
    Json(ScannerStatusResponse::from(status))
}

pub async fn start_scanner(
    State(state): State<AppState>,
) -> impl IntoResponse {
    state.scanner.start().await;
    (StatusCode::OK, Json(serde_json::json!({
        "status": "started",
        "message": "Scanner started successfully"
    })))
}

pub async fn stop_scanner(
    State(state): State<AppState>,
) -> impl IntoResponse {
    state.scanner.stop().await;
    (StatusCode::OK, Json(serde_json::json!({
        "status": "stopped",
        "message": "Scanner stopped successfully"
    })))
}

pub async fn get_signals(
    State(state): State<AppState>,
    Query(query): Query<SignalQuery>,
) -> impl IntoResponse {
    let signals_result = if let Some(venue_type_str) = &query.venue_type {
        let venue_type = match venue_type_str.to_lowercase().as_str() {
            "dex" | "dex_amm" => VenueType::DexAmm,
            "curve" | "bonding_curve" => VenueType::BondingCurve,
            "lending" => VenueType::Lending,
            "orderbook" => VenueType::Orderbook,
            _ => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "error": format!("Unknown venue type: {}", venue_type_str)
            }))).into_response(),
        };
        state.scanner.get_signals_by_venue(venue_type).await
    } else if let Some(min_confidence) = query.min_confidence {
        state.scanner.get_high_confidence_signals(min_confidence).await
    } else {
        state.scanner.scan_once().await
    };

    match signals_result {
        Ok(mut signals) => {
            // Filter by min profit if specified
            if let Some(min_profit) = query.min_profit_bps {
                signals.retain(|s| s.estimated_profit_bps >= min_profit);
            }

            // Apply limit
            let limit = query.limit.unwrap_or(100);
            signals.truncate(limit);

            let responses: Vec<SignalResponse> = signals
                .into_iter()
                .map(|s| SignalResponse {
                    id: s.id.to_string(),
                    signal_type: format!("{:?}", s.signal_type),
                    venue_id: s.venue_id.to_string(),
                    venue_type: format!("{:?}", s.venue_type),
                    token_mint: s.token_mint,
                    pool_address: s.pool_address,
                    estimated_profit_bps: s.estimated_profit_bps,
                    confidence: s.confidence,
                    significance: format!("{:?}", s.significance),
                    metadata: s.metadata,
                    detected_at: s.detected_at.to_rfc3339(),
                    expires_at: s.expires_at.to_rfc3339(),
                })
                .collect();

            Json(serde_json::json!({
                "signals": responses,
                "count": responses.len()
            })).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": e.to_string()
        }))).into_response(),
    }
}
