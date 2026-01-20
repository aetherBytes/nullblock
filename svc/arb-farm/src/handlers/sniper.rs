use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::agents::{SnipePosition, SniperConfig, SniperStats};
use crate::error::AppResult;
use crate::server::AppState;

#[derive(Debug, Serialize)]
pub struct SniperStatsResponse {
    pub stats: SniperStats,
}

pub async fn get_sniper_stats(
    State(state): State<AppState>,
) -> AppResult<Json<SniperStatsResponse>> {
    let stats = state.graduation_sniper.get_stats().await;
    Ok(Json(SniperStatsResponse { stats }))
}

#[derive(Debug, Serialize)]
pub struct ListPositionsResponse {
    pub positions: Vec<SnipePosition>,
    pub total: usize,
}

pub async fn list_snipe_positions(
    State(state): State<AppState>,
) -> AppResult<Json<ListPositionsResponse>> {
    let positions = state.graduation_sniper.list_positions().await;
    let total = positions.len();
    Ok(Json(ListPositionsResponse { positions, total }))
}

#[derive(Debug, Deserialize)]
pub struct AddPositionRequest {
    pub mint: String,
    pub symbol: String,
    pub strategy_id: Option<String>,
    pub entry_tokens: u64,
    pub entry_price_sol: f64,
}

#[derive(Debug, Serialize)]
pub struct AddPositionResponse {
    pub success: bool,
    pub message: String,
    pub mint: String,
}

pub async fn add_snipe_position(
    State(state): State<AppState>,
    Json(request): Json<AddPositionRequest>,
) -> AppResult<Json<AddPositionResponse>> {
    let strategy_id = request.strategy_id
        .as_ref()
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or(Uuid::nil());

    state.graduation_sniper.add_position(
        &request.mint,
        &request.symbol,
        strategy_id,
        request.entry_tokens,
        request.entry_price_sol,
    ).await;

    Ok(Json(AddPositionResponse {
        success: true,
        message: format!(
            "Added snipe position for {} ({} tokens @ {} SOL)",
            request.symbol, request.entry_tokens, request.entry_price_sol
        ),
        mint: request.mint,
    }))
}

#[derive(Debug, Serialize)]
pub struct RemovePositionResponse {
    pub success: bool,
    pub message: String,
    pub mint: String,
    pub removed_position: Option<SnipePosition>,
}

pub async fn remove_snipe_position(
    State(state): State<AppState>,
    Path(mint): Path<String>,
) -> AppResult<Json<RemovePositionResponse>> {
    let removed = state.graduation_sniper.remove_position(&mint).await;

    Ok(Json(RemovePositionResponse {
        success: removed.is_some(),
        message: if removed.is_some() {
            format!("Removed snipe position for {}", mint)
        } else {
            format!("No snipe position found for {}", mint)
        },
        mint,
        removed_position: removed,
    }))
}

#[derive(Debug, Serialize)]
pub struct ManualSellResponse {
    pub success: bool,
    pub message: String,
    pub mint: String,
}

pub async fn manual_sell_position(
    State(state): State<AppState>,
    Path(mint): Path<String>,
) -> AppResult<Json<ManualSellResponse>> {
    match state.graduation_sniper.manual_sell(&mint).await {
        Ok(()) => Ok(Json(ManualSellResponse {
            success: true,
            message: format!("Manual sell initiated for {}", mint),
            mint,
        })),
        Err(e) => Ok(Json(ManualSellResponse {
            success: false,
            message: format!("Failed to sell {}: {}", mint, e),
            mint,
        })),
    }
}

#[derive(Debug, Serialize)]
pub struct SniperControlResponse {
    pub success: bool,
    pub message: String,
    pub is_running: bool,
}

pub async fn start_sniper(
    State(state): State<AppState>,
) -> AppResult<Json<SniperControlResponse>> {
    state.graduation_sniper.start().await;
    let stats = state.graduation_sniper.get_stats().await;

    Ok(Json(SniperControlResponse {
        success: true,
        message: "Graduation sniper started".to_string(),
        is_running: stats.is_running,
    }))
}

pub async fn stop_sniper(
    State(state): State<AppState>,
) -> AppResult<Json<SniperControlResponse>> {
    state.graduation_sniper.stop().await;
    let stats = state.graduation_sniper.get_stats().await;

    Ok(Json(SniperControlResponse {
        success: true,
        message: "Graduation sniper stopped".to_string(),
        is_running: stats.is_running,
    }))
}

#[derive(Debug, Serialize)]
pub struct SniperConfigResponse {
    pub config: SniperConfig,
}

pub async fn get_sniper_config(
    State(state): State<AppState>,
) -> AppResult<Json<SniperConfigResponse>> {
    let config = state.graduation_sniper.get_config().await;
    Ok(Json(SniperConfigResponse { config }))
}

#[derive(Debug, Deserialize)]
pub struct UpdateSniperConfigRequest {
    pub sell_delay_ms: Option<u64>,
    pub max_sell_retries: Option<u32>,
    pub slippage_bps: Option<u32>,
    pub max_concurrent_positions: Option<u32>,
    pub take_profit_percent: Option<f64>,
    pub stop_loss_percent: Option<f64>,
    pub auto_sell_on_graduation: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct UpdateSniperConfigResponse {
    pub success: bool,
    pub message: String,
    pub config: SniperConfig,
}

pub async fn update_sniper_config(
    State(state): State<AppState>,
    Json(request): Json<UpdateSniperConfigRequest>,
) -> AppResult<Json<UpdateSniperConfigResponse>> {
    let mut config = state.graduation_sniper.get_config().await;

    if let Some(v) = request.sell_delay_ms {
        config.sell_delay_ms = v;
    }
    if let Some(v) = request.max_sell_retries {
        config.max_sell_retries = v;
    }
    if let Some(v) = request.slippage_bps {
        config.slippage_bps = v;
    }
    if let Some(v) = request.max_concurrent_positions {
        config.max_concurrent_positions = v;
    }
    if let Some(v) = request.take_profit_percent {
        config.take_profit_percent = v;
    }
    if let Some(v) = request.stop_loss_percent {
        config.stop_loss_percent = v;
    }
    if let Some(v) = request.auto_sell_on_graduation {
        config.auto_sell_on_graduation = v;
    }

    state.graduation_sniper.update_config(config.clone()).await;

    Ok(Json(UpdateSniperConfigResponse {
        success: true,
        message: format!(
            "Sniper config updated: sell_delay={}ms, retries={}, slippage={}bps, max_positions={}, TP={:.1}%, SL={:.1}%, auto_sell={}",
            config.sell_delay_ms,
            config.max_sell_retries,
            config.slippage_bps,
            config.max_concurrent_positions,
            config.take_profit_percent,
            config.stop_loss_percent,
            config.auto_sell_on_graduation
        ),
        config,
    }))
}
