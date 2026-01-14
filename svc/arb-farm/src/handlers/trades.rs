use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppResult;
use crate::server::AppState;

#[derive(Debug, Deserialize)]
pub struct ListTradesQuery {
    pub strategy_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct TradeResponse {
    pub id: Uuid,
    pub edge_id: Uuid,
    pub strategy_id: Uuid,
    pub tx_signature: Option<String>,
    pub bundle_id: Option<String>,
    pub profit_lamports: Option<i64>,
    pub gas_cost_lamports: Option<i64>,
    pub slippage_bps: Option<i32>,
    pub executed_at: String,
}

#[derive(Debug, Serialize)]
pub struct ListTradesResponse {
    pub trades: Vec<TradeResponse>,
    pub total: usize,
}

pub async fn list_trades(
    State(state): State<AppState>,
    Query(query): Query<ListTradesQuery>,
) -> AppResult<Json<ListTradesResponse>> {
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let records = if let Some(strategy_id) = query.strategy_id {
        state
            .trade_repo
            .list_by_strategy(strategy_id, limit, offset)
            .await?
    } else {
        state.trade_repo.list(limit, offset).await?
    };

    let trades: Vec<TradeResponse> = records
        .iter()
        .map(|r| TradeResponse {
            id: r.id,
            edge_id: r.edge_id,
            strategy_id: r.strategy_id,
            tx_signature: r.tx_signature.clone(),
            bundle_id: r.bundle_id.clone(),
            profit_lamports: r.profit_lamports,
            gas_cost_lamports: r.gas_cost_lamports,
            slippage_bps: r.slippage_bps,
            executed_at: r.executed_at.to_rfc3339(),
        })
        .collect();

    let total = trades.len();

    Ok(Json(ListTradesResponse { trades, total }))
}

pub async fn get_trade(
    State(state): State<AppState>,
    Path(trade_id): Path<Uuid>,
) -> AppResult<Json<TradeResponse>> {
    let record = state
        .trade_repo
        .get_by_id(trade_id)
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound(format!("Trade {} not found", trade_id)))?;

    Ok(Json(TradeResponse {
        id: record.id,
        edge_id: record.edge_id,
        strategy_id: record.strategy_id,
        tx_signature: record.tx_signature,
        bundle_id: record.bundle_id,
        profit_lamports: record.profit_lamports,
        gas_cost_lamports: record.gas_cost_lamports,
        slippage_bps: record.slippage_bps,
        executed_at: record.executed_at.to_rfc3339(),
    }))
}

#[derive(Debug, Deserialize)]
pub struct TradeStatsQuery {
    pub period_days: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct TradeStatsResponse {
    pub total_trades: i64,
    pub winning_trades: i64,
    pub losing_trades: i64,
    pub win_rate: f64,
    pub total_profit_sol: f64,
    pub total_loss_sol: f64,
    pub net_pnl_sol: f64,
    pub total_gas_cost_sol: f64,
    pub avg_profit_sol: f64,
    pub largest_win_sol: f64,
    pub largest_loss_sol: f64,
}

pub async fn get_trade_stats(
    State(state): State<AppState>,
    Query(query): Query<TradeStatsQuery>,
) -> AppResult<Json<TradeStatsResponse>> {
    let stats = state.trade_repo.get_stats(query.period_days).await?;

    Ok(Json(TradeStatsResponse {
        total_trades: stats.total_trades,
        winning_trades: stats.winning_trades,
        losing_trades: stats.losing_trades,
        win_rate: stats.win_rate,
        total_profit_sol: stats.total_profit_lamports as f64 / 1e9,
        total_loss_sol: stats.total_loss_lamports as f64 / 1e9,
        net_pnl_sol: stats.net_pnl_lamports as f64 / 1e9,
        total_gas_cost_sol: stats.total_gas_cost_lamports as f64 / 1e9,
        avg_profit_sol: stats.avg_profit_lamports / 1e9,
        largest_win_sol: stats.largest_win_lamports as f64 / 1e9,
        largest_loss_sol: stats.largest_loss_lamports as f64 / 1e9,
    }))
}

#[derive(Debug, Deserialize)]
pub struct DailyStatsQuery {
    pub days: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct DailyStatsResponse {
    pub daily_stats: Vec<DailyStat>,
}

#[derive(Debug, Serialize)]
pub struct DailyStat {
    pub date: String,
    pub trade_count: i64,
    pub wins: i64,
    pub losses: i64,
    pub net_pnl_sol: f64,
    pub gas_cost_sol: f64,
}

pub async fn get_daily_stats(
    State(state): State<AppState>,
    Query(query): Query<DailyStatsQuery>,
) -> AppResult<Json<DailyStatsResponse>> {
    let days = query.days.unwrap_or(7);
    let stats = state.trade_repo.get_daily_stats(days).await?;

    let daily_stats: Vec<DailyStat> = stats
        .iter()
        .map(|s| DailyStat {
            date: s.date.to_string(),
            trade_count: s.trade_count,
            wins: s.wins,
            losses: s.losses,
            net_pnl_sol: s.net_pnl_lamports as f64 / 1e9,
            gas_cost_sol: s.gas_cost_lamports as f64 / 1e9,
        })
        .collect();

    Ok(Json(DailyStatsResponse { daily_stats }))
}
