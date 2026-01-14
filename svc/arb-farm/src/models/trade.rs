use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Trade {
    pub id: Uuid,
    pub edge_id: Option<Uuid>,
    pub strategy_id: Option<Uuid>,
    pub tx_signature: Option<String>,
    pub bundle_id: Option<String>,
    pub entry_price: Option<Decimal>,
    pub exit_price: Option<Decimal>,
    pub profit_lamports: Option<i64>,
    pub gas_cost_lamports: Option<i64>,
    pub slippage_bps: Option<i32>,
    pub executed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeStats {
    pub total_trades: i64,
    pub winning_trades: i64,
    pub losing_trades: i64,
    pub win_rate: f64,
    pub total_profit_lamports: i64,
    pub total_loss_lamports: i64,
    pub net_pnl_lamports: i64,
    pub total_gas_lamports: i64,
    pub avg_profit_per_trade: f64,
    pub best_trade_lamports: i64,
    pub worst_trade_lamports: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeFilter {
    pub edge_id: Option<Uuid>,
    pub strategy_id: Option<Uuid>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub min_profit: Option<i64>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl Default for TradeFilter {
    fn default() -> Self {
        Self {
            edge_id: None,
            strategy_id: None,
            from_date: None,
            to_date: None,
            min_profit: None,
            limit: Some(50),
            offset: Some(0),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StatsPeriod {
    Day,
    Week,
    Month,
    All,
}

impl Default for StatsPeriod {
    fn default() -> Self {
        Self::Week
    }
}
