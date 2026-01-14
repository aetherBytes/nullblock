use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Strategy {
    pub id: Uuid,
    pub wallet_address: String,
    pub name: String,
    pub strategy_type: String,
    pub venue_types: Vec<String>,
    pub execution_mode: String,
    pub risk_params: RiskParams,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StrategyType {
    DexArb,
    CurveArb,
    Liquidation,
    JitLiquidity,
    Backrun,
    CopyTrade,
    Custom,
}

impl std::fmt::Display for StrategyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StrategyType::DexArb => write!(f, "dex_arb"),
            StrategyType::CurveArb => write!(f, "curve_arb"),
            StrategyType::Liquidation => write!(f, "liquidation"),
            StrategyType::JitLiquidity => write!(f, "jit_liquidity"),
            StrategyType::Backrun => write!(f, "backrun"),
            StrategyType::CopyTrade => write!(f, "copy_trade"),
            StrategyType::Custom => write!(f, "custom"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskParams {
    pub max_position_sol: f64,
    pub daily_loss_limit_sol: f64,
    pub min_profit_bps: u16,
    pub max_slippage_bps: u16,
    pub max_risk_score: i32,
    pub require_simulation: bool,
    pub auto_execute_atomic: bool,
}

impl Default for RiskParams {
    fn default() -> Self {
        Self {
            max_position_sol: 1.0,
            daily_loss_limit_sol: 0.5,
            min_profit_bps: 50,
            max_slippage_bps: 100,
            max_risk_score: 50,
            require_simulation: true,
            auto_execute_atomic: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStrategyRequest {
    pub wallet_address: String,
    pub name: String,
    pub strategy_type: String,
    pub venue_types: Vec<String>,
    pub execution_mode: String,
    pub risk_params: RiskParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStrategyRequest {
    pub name: Option<String>,
    pub venue_types: Option<Vec<String>>,
    pub execution_mode: Option<String>,
    pub risk_params: Option<RiskParams>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyStats {
    pub strategy_id: Uuid,
    pub total_trades: i64,
    pub winning_trades: i64,
    pub losing_trades: i64,
    pub win_rate: f64,
    pub total_profit_lamports: i64,
    pub total_loss_lamports: i64,
    pub net_pnl_lamports: i64,
    pub avg_profit_bps: f64,
    pub best_trade_lamports: i64,
    pub worst_trade_lamports: i64,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}
