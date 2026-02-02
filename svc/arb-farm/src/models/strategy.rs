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
    #[serde(default)]
    pub last_tested_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub last_executed_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub test_results: Option<StrategyTestResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyTestResult {
    pub tested_at: DateTime<Utc>,
    pub simulated_profit_lamports: i64,
    pub risk_score: i32,
    pub passed: bool,
    pub notes: Option<String>,
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
    GraduationSnipe,
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
            StrategyType::GraduationSnipe => write!(f, "graduation_snipe"),
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
    #[serde(default = "default_auto_execute")]
    pub auto_execute_enabled: bool,
    #[serde(default = "default_require_consensus")]
    pub require_consensus: bool,
    #[serde(default = "default_require_confirmation")]
    pub require_confirmation: bool,
    #[serde(default = "default_staleness_hours")]
    pub staleness_threshold_hours: u32,
    #[serde(default)]
    pub stop_loss_percent: Option<f64>,
    #[serde(default)]
    pub take_profit_percent: Option<f64>,
    #[serde(default)]
    pub trailing_stop_percent: Option<f64>,
    #[serde(default)]
    pub time_limit_minutes: Option<u32>,
    #[serde(default = "default_base_currency")]
    pub base_currency: String,
    #[serde(default = "default_capital_allocation_percent")]
    pub max_capital_allocation_percent: f64,
    #[serde(default)]
    pub concurrent_positions: Option<u32>,
    #[serde(default)]
    pub momentum_adaptive_exits: bool,
    #[serde(default)]
    pub let_winners_run: bool,
}

fn default_base_currency() -> String {
    "sol".to_string()
}
fn default_capital_allocation_percent() -> f64 {
    25.0
}

fn default_auto_execute() -> bool {
    false
}
fn default_require_consensus() -> bool {
    false
}
fn default_require_confirmation() -> bool {
    true
}
fn default_staleness_hours() -> u32 {
    24
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
            auto_execute_enabled: false,
            require_consensus: false,
            require_confirmation: true,
            staleness_threshold_hours: 24,
            stop_loss_percent: Some(10.0), // DEFENSIVE: 10% tight stop
            take_profit_percent: Some(15.0), // DEFENSIVE: 15% TP (strong momentum extends)
            trailing_stop_percent: Some(8.0), // DEFENSIVE: 8% trailing stop
            time_limit_minutes: Some(5),   // DEFENSIVE: 5 min
            base_currency: "sol".to_string(),
            max_capital_allocation_percent: 25.0,
            concurrent_positions: Some(1),
            momentum_adaptive_exits: true, // Enable momentum tracking by default
            let_winners_run: true,         // Let profitable positions run
        }
    }
}

impl RiskParams {
    pub fn to_exit_config(&self) -> crate::execution::ExitConfig {
        use crate::execution::{
            AdaptivePartialTakeProfit, BaseCurrency, ExitMode, MomentumAdaptiveConfig,
        };

        let base = match self.base_currency.to_lowercase().as_str() {
            "usdc" => BaseCurrency::Usdc,
            "usdt" => BaseCurrency::Usdt,
            _ => BaseCurrency::Sol,
        };

        let exit_mode = if self.auto_execute_atomic {
            ExitMode::Atomic
        } else {
            ExitMode::Default
        };

        let momentum_adaptive = if self.momentum_adaptive_exits {
            Some(MomentumAdaptiveConfig::default())
        } else {
            None
        };

        let adaptive_partial_tp = if self.momentum_adaptive_exits {
            Some(AdaptivePartialTakeProfit::default())
        } else {
            None
        };

        crate::execution::ExitConfig {
            base_currency: base,
            exit_mode,
            stop_loss_percent: self.stop_loss_percent,
            take_profit_percent: self.take_profit_percent,
            trailing_stop_percent: self.trailing_stop_percent,
            time_limit_minutes: self.time_limit_minutes,
            partial_take_profit: None,
            custom_exit_instructions: None,
            momentum_adaptive,
            adaptive_partial_tp,
        }
    }

    pub fn for_flashloan() -> Self {
        Self {
            max_position_sol: 10.0,
            daily_loss_limit_sol: 1.0,
            min_profit_bps: 10,
            max_slippage_bps: 50,
            max_risk_score: 30,
            require_simulation: true,
            auto_execute_atomic: true,
            auto_execute_enabled: true,
            require_consensus: false,
            require_confirmation: false,
            staleness_threshold_hours: 1,
            stop_loss_percent: None,
            take_profit_percent: None,
            trailing_stop_percent: None,
            time_limit_minutes: None,
            base_currency: "sol".to_string(),
            max_capital_allocation_percent: 50.0,
            concurrent_positions: Some(3),
            momentum_adaptive_exits: false,
            let_winners_run: false,
        }
    }

    pub fn for_copy_trade() -> Self {
        Self {
            max_position_sol: 0.5,
            daily_loss_limit_sol: 1.0,
            min_profit_bps: 0,
            max_slippage_bps: 150,
            max_risk_score: 60,
            require_simulation: true,
            auto_execute_atomic: false,
            auto_execute_enabled: false,
            require_consensus: false,
            require_confirmation: true,
            staleness_threshold_hours: 24,
            stop_loss_percent: Some(15.0),
            take_profit_percent: Some(50.0),
            trailing_stop_percent: Some(10.0),
            time_limit_minutes: Some(120),
            base_currency: "sol".to_string(),
            max_capital_allocation_percent: 20.0,
            concurrent_positions: Some(2),
            momentum_adaptive_exits: false,
            let_winners_run: false,
        }
    }

    pub fn conservative() -> Self {
        Self {
            max_position_sol: 0.1,
            daily_loss_limit_sol: 0.25,
            min_profit_bps: 100,
            max_slippage_bps: 100,
            max_risk_score: 40,
            require_simulation: true,
            auto_execute_atomic: false,
            auto_execute_enabled: false,
            require_consensus: false,
            require_confirmation: true,
            staleness_threshold_hours: 12,
            stop_loss_percent: Some(5.0),
            take_profit_percent: Some(15.0),
            trailing_stop_percent: Some(3.0),
            time_limit_minutes: Some(30),
            base_currency: "sol".to_string(),
            max_capital_allocation_percent: 10.0,
            concurrent_positions: Some(1),
            momentum_adaptive_exits: false,
            let_winners_run: false,
        }
    }

    pub fn moderate() -> Self {
        Self {
            max_position_sol: 0.5,
            daily_loss_limit_sol: 1.0,
            min_profit_bps: 50,
            max_slippage_bps: 150,
            max_risk_score: 60,
            require_simulation: true,
            auto_execute_atomic: false,
            auto_execute_enabled: false,
            require_consensus: false,
            require_confirmation: true,
            staleness_threshold_hours: 24,
            stop_loss_percent: Some(10.0),
            take_profit_percent: Some(30.0),
            trailing_stop_percent: Some(7.0),
            time_limit_minutes: Some(60),
            base_currency: "sol".to_string(),
            max_capital_allocation_percent: 20.0,
            concurrent_positions: Some(2),
            momentum_adaptive_exits: false,
            let_winners_run: false,
        }
    }

    pub fn aggressive() -> Self {
        Self {
            max_position_sol: 2.0,
            daily_loss_limit_sol: 5.0,
            min_profit_bps: 25,
            max_slippage_bps: 300,
            max_risk_score: 80,
            require_simulation: true,
            auto_execute_atomic: false,
            auto_execute_enabled: false,
            require_consensus: false,
            require_confirmation: true,
            staleness_threshold_hours: 48,
            stop_loss_percent: Some(10.0),    // DEFENSIVE
            take_profit_percent: Some(15.0),  // DEFENSIVE
            trailing_stop_percent: Some(8.0), // DEFENSIVE
            time_limit_minutes: Some(5),      // DEFENSIVE
            base_currency: "sol".to_string(),
            max_capital_allocation_percent: 40.0,
            concurrent_positions: Some(5),
            momentum_adaptive_exits: true, // Enable for aggressive strategy
            let_winners_run: true,
        }
    }

    /// Get risk profile by name
    pub fn from_profile(profile: &str) -> Self {
        match profile.to_lowercase().as_str() {
            "conservative" | "low" => Self::conservative(),
            "aggressive" | "high" => Self::aggressive(),
            "flashloan" | "atomic" => Self::for_flashloan(),
            "copy_trade" | "copy" => Self::for_copy_trade(),
            _ => Self::moderate(), // Default to moderate
        }
    }
}

impl Strategy {
    pub fn is_stale(&self) -> bool {
        let threshold_hours = self.risk_params.staleness_threshold_hours as i64;
        match self.last_tested_at {
            Some(tested_at) => {
                let hours_since_test = (Utc::now() - tested_at).num_hours();
                hours_since_test > threshold_hours
            }
            None => true,
        }
    }

    pub fn requires_confirmation(&self) -> bool {
        self.risk_params.require_confirmation || self.is_stale()
    }

    pub fn can_auto_execute(&self) -> bool {
        self.risk_params.auto_execute_enabled
            && !self.is_stale()
            && !self.risk_params.require_confirmation
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CurveStrategyMode {
    GraduationArbitrage,
    FastSnipe,
    ScalpOnCurve,
}

impl Default for CurveStrategyMode {
    fn default() -> Self {
        Self::GraduationArbitrage
    }
}

impl std::fmt::Display for CurveStrategyMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GraduationArbitrage => write!(f, "graduation_arbitrage"),
            Self::FastSnipe => write!(f, "fast_snipe"),
            Self::ScalpOnCurve => write!(f, "scalp_on_curve"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurveStrategyParams {
    #[serde(default)]
    pub mode: CurveStrategyMode,
    #[serde(default = "default_min_graduation_progress")]
    pub min_graduation_progress: f64,
    #[serde(default = "default_max_graduation_progress")]
    pub max_graduation_progress: f64,
    #[serde(default)]
    pub min_volume_24h_sol: f64,
    #[serde(default = "default_max_holder_concentration")]
    pub max_holder_concentration: f64,
    #[serde(default = "default_min_holder_count")]
    pub min_holder_count: u32,
    #[serde(default = "default_entry_sol_amount")]
    pub entry_sol_amount: f64,
    #[serde(default = "default_exit_on_graduation")]
    pub exit_on_graduation: bool,
    #[serde(default = "default_graduation_sell_delay_ms")]
    pub graduation_sell_delay_ms: u64,
    #[serde(default)]
    pub venue_filter: Option<Vec<String>>,
    #[serde(default)]
    pub min_score: Option<u32>,
    #[serde(default)]
    pub post_graduation_momentum_sell: bool,
    #[serde(default = "default_post_graduation_momentum_delay_ms")]
    pub post_graduation_momentum_delay_ms: u64,
    #[serde(default = "default_min_progress_velocity")]
    pub min_progress_velocity: f64,
    #[serde(default = "default_min_price_velocity")]
    pub min_price_velocity: f64,
    #[serde(default = "default_max_recent_pump_percent")]
    pub max_recent_pump_percent: f64,
}

fn default_min_graduation_progress() -> f64 {
    85.0
} // Narrowed entry window (was 70%)
fn default_max_graduation_progress() -> f64 {
    98.0
}
fn default_max_holder_concentration() -> f64 {
    30.0
} // Tighter to reduce dumps (was 50%)
fn default_min_holder_count() -> u32 {
    100
} // More distributed (was 50)
fn default_entry_sol_amount() -> f64 {
    0.1
}
fn default_exit_on_graduation() -> bool {
    true
}
fn default_graduation_sell_delay_ms() -> u64 {
    50
} // Faster exit (was 500ms)
fn default_post_graduation_momentum_delay_ms() -> u64 {
    250
} // Faster (was 1000ms)
fn default_min_progress_velocity() -> f64 {
    0.5
} // % per minute acceleration required
fn default_min_price_velocity() -> f64 {
    1.0
} // % per minute price increase required
fn default_max_recent_pump_percent() -> f64 {
    15.0
} // Skip if pumped >15% in last 2 min

impl Default for CurveStrategyParams {
    fn default() -> Self {
        Self {
            mode: CurveStrategyMode::GraduationArbitrage,
            min_graduation_progress: 85.0, // Narrowed entry window (was 70%)
            max_graduation_progress: 98.0,
            min_volume_24h_sol: 10.0,
            max_holder_concentration: 30.0, // Tighter to reduce dumps (was 50%)
            min_holder_count: 100,          // More distributed (was 50)
            entry_sol_amount: 0.1,
            exit_on_graduation: true,
            graduation_sell_delay_ms: 50, // Faster exit (was 500ms)
            venue_filter: None,
            min_score: None,
            post_graduation_momentum_sell: false,
            post_graduation_momentum_delay_ms: default_post_graduation_momentum_delay_ms(),
            min_progress_velocity: default_min_progress_velocity(),
            min_price_velocity: default_min_price_velocity(),
            max_recent_pump_percent: default_max_recent_pump_percent(),
        }
    }
}

impl CurveStrategyParams {
    pub fn fast_snipe() -> Self {
        Self {
            mode: CurveStrategyMode::FastSnipe,
            min_graduation_progress: 95.0,
            max_graduation_progress: 99.5,
            min_volume_24h_sol: 50.0,
            max_holder_concentration: 25.0, // Tighter for fast snipes (was 40%)
            min_holder_count: 100,
            entry_sol_amount: 0.5,
            exit_on_graduation: true,
            graduation_sell_delay_ms: 50, // Faster (was 100ms)
            venue_filter: None,
            min_score: Some(70),
            post_graduation_momentum_sell: true,
            post_graduation_momentum_delay_ms: 250, // Faster (was 500ms)
            min_progress_velocity: 1.0,             // Higher velocity required for snipes
            min_price_velocity: 2.0,                // Strong upward momentum
            max_recent_pump_percent: 10.0,          // Stricter FOMO filter
        }
    }

    pub fn scalp_on_curve() -> Self {
        Self {
            mode: CurveStrategyMode::ScalpOnCurve,
            min_graduation_progress: 50.0,
            max_graduation_progress: 85.0,
            min_volume_24h_sol: 20.0,
            max_holder_concentration: 40.0, // Tighter (was 60%)
            min_holder_count: 50,           // More distributed (was 30)
            entry_sol_amount: 0.2,
            exit_on_graduation: false,
            graduation_sell_delay_ms: 0,
            venue_filter: None,
            min_score: None,
            post_graduation_momentum_sell: false,
            post_graduation_momentum_delay_ms: 0,
            min_progress_velocity: 0.0, // No velocity requirement for scalping
            min_price_velocity: 0.0,    // No velocity requirement
            max_recent_pump_percent: 20.0, // More permissive for scalps
        }
    }

    pub fn matches_candidate(
        &self,
        progress: f64,
        volume_sol: f64,
        holder_concentration: f64,
        holder_count: u32,
    ) -> bool {
        progress >= self.min_graduation_progress
            && progress <= self.max_graduation_progress
            && volume_sol >= self.min_volume_24h_sol
            && holder_concentration <= self.max_holder_concentration
            && holder_count >= self.min_holder_count
    }

    pub fn matches_candidate_with_velocity(
        &self,
        progress: f64,
        volume_sol: f64,
        holder_concentration: f64,
        holder_count: u32,
        progress_velocity: f64,
        price_velocity: f64,
        recent_pump_percent: f64,
    ) -> bool {
        self.matches_candidate(progress, volume_sol, holder_concentration, holder_count)
            && progress_velocity >= self.min_progress_velocity
            && price_velocity >= self.min_price_velocity
            && recent_pump_percent <= self.max_recent_pump_percent
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurveStrategyStats {
    pub strategy_id: Uuid,
    pub total_entries: u32,
    pub successful_exits: u32,
    pub graduations_caught: u32,
    pub total_pnl_sol: f64,
    pub win_rate: f64,
    pub avg_hold_time_seconds: u64,
    pub best_trade_sol: f64,
    pub worst_trade_sol: f64,
}
