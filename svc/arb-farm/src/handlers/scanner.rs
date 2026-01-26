use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::agents::{ScannerStatus, VenueStatus};
use crate::database::repositories::edges::CreateEdgeRecord;
use crate::events::AtomicityLevel;
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

#[derive(Debug, Serialize)]
pub struct ProcessSignalsResponse {
    pub signals_detected: usize,
    pub edges_created: usize,
    pub edges_rejected: usize,
    pub created_edge_ids: Vec<String>,
    pub rejection_reasons: Vec<String>,
}

pub async fn process_signals(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let signals_result = state.scanner.scan_once().await;

    match signals_result {
        Ok(signals) => {
            let signals_detected = signals.len();
            let mut edges_created = 0;
            let mut edges_rejected = 0;
            let mut created_edge_ids = Vec::new();
            let mut rejection_reasons = Vec::new();

            for signal in signals {
                match state.strategy_engine.match_signal(&signal).await {
                    Some(result) => {
                        if result.approved {
                            if let Some(edge) = result.created_edge {
                                let create_record = CreateEdgeRecord {
                                    strategy_id: Some(result.strategy_id),
                                    edge_type: edge.edge_type.clone(),
                                    execution_mode: edge.execution_mode.clone(),
                                    atomicity: edge.atomicity,
                                    simulated_profit_guaranteed: edge.simulated_profit_guaranteed,
                                    estimated_profit_lamports: edge.estimated_profit_lamports,
                                    risk_score: edge.risk_score,
                                    route_data: edge.route_data,
                                    expires_at: edge.expires_at,
                                };

                                match state.edge_repo.create(create_record).await {
                                    Ok(record) => {
                                        edges_created += 1;
                                        created_edge_ids.push(record.id.to_string());
                                        tracing::info!(
                                            edge_id = %record.id,
                                            signal_type = %edge.edge_type,
                                            "Edge created from signal"
                                        );

                                        // Capture edge as engram for learning/sharing
                                        if state.engrams_client.is_configured() {
                                            let wallet = state.config.wallet_address.clone()
                                                .unwrap_or_else(|| "default".to_string());
                                            let token_mint = signal.token_mint.as_deref();
                                            let _ = state.engrams_client.save_edge(
                                                &wallet,
                                                &record.id.to_string(),
                                                &record.edge_type,
                                                token_mint,
                                                record.estimated_profit_lamports.unwrap_or(0),
                                                record.risk_score.unwrap_or(50),
                                                &signal.metadata,
                                            ).await;
                                        }
                                    }
                                    Err(e) => {
                                        edges_rejected += 1;
                                        rejection_reasons.push(format!("DB error: {}", e));
                                        tracing::error!(error = %e, "Failed to create edge in DB");
                                    }
                                }
                            }
                        } else {
                            edges_rejected += 1;
                            if let Some(reason) = result.reason {
                                rejection_reasons.push(reason);
                            }
                        }
                    }
                    None => {
                        // No matching strategy found - not an error, just no match
                    }
                }
            }

            Json(ProcessSignalsResponse {
                signals_detected,
                edges_created,
                edges_rejected,
                created_edge_ids,
                rejection_reasons,
            }).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": e.to_string()
        }))).into_response(),
    }
}

// ============================================================================
// Behavioral Strategies
// ============================================================================

#[derive(Debug, Serialize)]
pub struct BehavioralStrategyResponse {
    pub name: String,
    pub strategy_type: String,
    pub is_active: bool,
    pub supported_venues: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct BehavioralStrategiesListResponse {
    pub strategies: Vec<BehavioralStrategyResponse>,
    pub total: usize,
    pub active_count: usize,
}

pub async fn list_behavioral_strategies(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let registry = state.scanner.get_strategy_registry();
    let strategies = registry.list().await;
    let active_count = registry.active_count().await;

    let strategy_list: Vec<BehavioralStrategyResponse> = strategies
        .iter()
        .map(|s| BehavioralStrategyResponse {
            name: s.name().to_string(),
            strategy_type: s.strategy_type().to_string(),
            is_active: s.is_active(),
            supported_venues: s.supported_venues().iter().map(|v| format!("{:?}", v)).collect(),
        })
        .collect();

    let total = strategy_list.len();

    Json(BehavioralStrategiesListResponse {
        strategies: strategy_list,
        total,
        active_count,
    })
}

pub async fn get_behavioral_strategy(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let registry = state.scanner.get_strategy_registry();
    let strategies = registry.list().await;

    if let Some(strategy) = strategies.iter().find(|s| s.name() == name) {
        Json(serde_json::json!({
            "name": strategy.name(),
            "strategy_type": strategy.strategy_type(),
            "is_active": strategy.is_active(),
            "supported_venues": strategy.supported_venues().iter().map(|v| format!("{:?}", v)).collect::<Vec<_>>(),
        })).into_response()
    } else {
        (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": format!("Behavioral strategy '{}' not found", name)
        }))).into_response()
    }
}

#[derive(Debug, Deserialize)]
pub struct ToggleBehavioralStrategyRequest {
    pub active: bool,
}

pub async fn toggle_behavioral_strategy(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(body): Json<ToggleBehavioralStrategyRequest>,
) -> impl IntoResponse {
    let registry = state.scanner.get_strategy_registry();

    if registry.toggle(&name, body.active).await {
        tracing::info!(
            strategy = %name,
            active = body.active,
            "Toggled behavioral strategy"
        );
        Json(serde_json::json!({
            "success": true,
            "name": name,
            "is_active": body.active,
            "message": format!("Behavioral strategy '{}' is now {}", name, if body.active { "active" } else { "inactive" })
        })).into_response()
    } else {
        (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": format!("Behavioral strategy '{}' not found", name)
        }))).into_response()
    }
}

pub async fn toggle_all_behavioral_strategies(
    State(state): State<AppState>,
    Json(body): Json<ToggleBehavioralStrategyRequest>,
) -> impl IntoResponse {
    let registry = state.scanner.get_strategy_registry();
    let strategies = registry.list().await;

    let mut toggled = 0;
    for strategy in &strategies {
        if registry.toggle(strategy.name(), body.active).await {
            toggled += 1;
        }
    }

    tracing::info!(
        count = toggled,
        active = body.active,
        "Toggled all behavioral strategies"
    );

    Json(serde_json::json!({
        "success": true,
        "toggled_count": toggled,
        "is_active": body.active,
        "message": format!("Toggled {} behavioral strategies to {}", toggled, if body.active { "active" } else { "inactive" })
    }))
}
