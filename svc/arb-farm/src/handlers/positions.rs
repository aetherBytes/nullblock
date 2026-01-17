use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::error::AppError;
use crate::execution::{OpenPosition, PositionStatus, ExitReason, BaseCurrency};
use crate::server::AppState;

#[derive(Debug, Serialize)]
pub struct PositionsResponse {
    pub positions: Vec<OpenPosition>,
    pub stats: PositionStatsResponse,
}

#[derive(Debug, Serialize)]
pub struct PositionStatsResponse {
    pub total_positions_opened: u64,
    pub total_positions_closed: u64,
    pub active_positions: u32,
    pub total_realized_pnl: f64,
    pub total_unrealized_pnl: f64,
    pub stop_losses_triggered: u32,
    pub take_profits_triggered: u32,
    pub time_exits_triggered: u32,
}

pub async fn get_positions(
    State(state): State<AppState>,
) -> Result<Json<PositionsResponse>, AppError> {
    let positions = state.position_manager.get_open_positions().await;
    let stats = state.position_manager.get_stats().await;

    Ok(Json(PositionsResponse {
        positions,
        stats: PositionStatsResponse {
            total_positions_opened: stats.total_positions_opened,
            total_positions_closed: stats.total_positions_closed,
            active_positions: stats.active_positions,
            total_realized_pnl: stats.total_realized_pnl,
            total_unrealized_pnl: stats.total_unrealized_pnl,
            stop_losses_triggered: stats.stop_losses_triggered,
            take_profits_triggered: stats.take_profits_triggered,
            time_exits_triggered: stats.time_exits_triggered,
        },
    }))
}

pub async fn get_position(
    State(state): State<AppState>,
    Path(position_id): Path<Uuid>,
) -> Result<Json<OpenPosition>, AppError> {
    state
        .position_manager
        .get_position(position_id)
        .await
        .ok_or_else(|| AppError::NotFound(format!("Position {} not found", position_id)))
        .map(Json)
}

#[derive(Debug, Deserialize)]
pub struct ManualExitRequest {
    pub exit_percent: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct ExitResponse {
    pub success: bool,
    pub message: String,
    pub position_id: Uuid,
}

pub async fn close_position(
    State(state): State<AppState>,
    Path(position_id): Path<Uuid>,
    Json(request): Json<ManualExitRequest>,
) -> Result<Json<ExitResponse>, AppError> {
    let exit_percent = request.exit_percent.unwrap_or(100.0);

    match state
        .position_monitor
        .trigger_manual_exit(position_id, exit_percent, &state.dev_signer)
        .await
    {
        Ok(_) => {
            // Release capital if this was a full exit
            if exit_percent >= 100.0 {
                if let Some(released) = state.capital_manager.release_capital(position_id).await {
                    info!(
                        "💸 Released {} SOL capital for closed position {}",
                        released as f64 / 1_000_000_000.0,
                        position_id
                    );
                }
            }

            Ok(Json(ExitResponse {
                success: true,
                message: format!("Exit triggered for {}% of position", exit_percent),
                position_id,
            }))
        }
        Err(e) => Ok(Json(ExitResponse {
            success: false,
            message: format!("Exit failed: {}", e),
            position_id,
        })),
    }
}

#[derive(Debug, Serialize)]
pub struct EmergencyExitResponse {
    pub signals_triggered: usize,
    pub message: String,
}

pub async fn emergency_close_all(
    State(state): State<AppState>,
) -> Result<Json<EmergencyExitResponse>, AppError> {
    // Get all open positions before closing
    let positions = state.position_manager.get_open_positions().await;

    let signals = state.position_manager.emergency_close_all().await;
    let count = signals.len();

    // Release capital for all closed positions
    let mut capital_released: u64 = 0;
    for position in positions {
        if let Some(released) = state.capital_manager.release_capital(position.id).await {
            capital_released += released;
        }
    }

    if capital_released > 0 {
        info!(
            "💸 Emergency exit released {} SOL total capital",
            capital_released as f64 / 1_000_000_000.0
        );
    }

    Ok(Json(EmergencyExitResponse {
        signals_triggered: count,
        message: format!("Emergency exit triggered for {} positions, released {} SOL", count, capital_released as f64 / 1_000_000_000.0),
    }))
}

#[derive(Debug, Serialize)]
pub struct MonitorStatusResponse {
    pub monitoring_active: bool,
    pub price_check_interval_secs: u64,
    pub exit_slippage_bps: u16,
    pub active_positions: u32,
    pub pending_exit_signals: usize,
}

pub async fn get_monitor_status(
    State(state): State<AppState>,
) -> Result<Json<MonitorStatusResponse>, AppError> {
    let stats = state.position_monitor.get_position_stats().await;
    let signals = state.position_manager.get_pending_exit_signals().await;

    Ok(Json(MonitorStatusResponse {
        monitoring_active: true,
        price_check_interval_secs: 30,
        exit_slippage_bps: 100,
        active_positions: stats.active_positions,
        pending_exit_signals: signals.len(),
    }))
}

#[derive(Debug, Deserialize)]
pub struct StartMonitorRequest {
    pub price_check_interval_secs: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct StartMonitorResponse {
    pub success: bool,
    pub message: String,
}

pub async fn start_monitor(
    State(state): State<AppState>,
) -> Result<Json<StartMonitorResponse>, AppError> {
    state.start_position_monitor();

    Ok(Json(StartMonitorResponse {
        success: true,
        message: "Position monitor started".to_string(),
    }))
}

#[derive(Debug, Serialize)]
pub struct ExposureResponse {
    pub sol_exposure: f64,
    pub usdc_exposure: f64,
    pub usdt_exposure: f64,
    pub total_exposure_sol: f64,
}

pub async fn get_exposure(
    State(state): State<AppState>,
) -> Result<Json<ExposureResponse>, AppError> {
    let sol = state.position_manager.get_total_exposure_by_base(BaseCurrency::Sol).await;
    let usdc = state.position_manager.get_total_exposure_by_base(BaseCurrency::Usdc).await;
    let usdt = state.position_manager.get_total_exposure_by_base(BaseCurrency::Usdt).await;

    Ok(Json(ExposureResponse {
        sol_exposure: sol,
        usdc_exposure: usdc,
        usdt_exposure: usdt,
        total_exposure_sol: sol + usdc + usdt,
    }))
}
