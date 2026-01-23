use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::database::PositionRepository;
use crate::error::{AppError, AppResult};

pub const SOL_MINT: &str = "So11111111111111111111111111111111111111112";
pub const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
pub const USDT_MINT: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";

#[allow(dead_code)]
pub const CURVE_ENTRY_FEE_BPS: u16 = 100;      // 1% entry fee on bonding curves
#[allow(dead_code)]
pub const CURVE_EXIT_FEE_BPS: u16 = 100;       // 1% exit fee on bonding curves
#[allow(dead_code)]
pub const MIN_EXIT_SLIPPAGE_BPS: u16 = 150;    // 1.5% minimum slippage tolerance
#[allow(dead_code)]
pub const MIN_NET_PROFIT_THRESHOLD_PERCENT: f64 = 4.0;  // Break-even threshold after all costs

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BaseCurrency {
    Sol,
    Usdc,
    Usdt,
}

impl BaseCurrency {
    pub fn mint(&self) -> &'static str {
        match self {
            BaseCurrency::Sol => SOL_MINT,
            BaseCurrency::Usdc => USDC_MINT,
            BaseCurrency::Usdt => USDT_MINT,
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            BaseCurrency::Sol => "SOL",
            BaseCurrency::Usdc => "USDC",
            BaseCurrency::Usdt => "USDT",
        }
    }

    pub fn decimals(&self) -> u8 {
        match self {
            BaseCurrency::Sol => 9,
            BaseCurrency::Usdc => 6,
            BaseCurrency::Usdt => 6,
        }
    }

    pub fn from_mint(mint: &str) -> Option<Self> {
        match mint {
            SOL_MINT => Some(BaseCurrency::Sol),
            USDC_MINT => Some(BaseCurrency::Usdc),
            USDT_MINT => Some(BaseCurrency::Usdt),
            _ => None,
        }
    }

    pub fn is_base_currency(mint: &str) -> bool {
        Self::from_mint(mint).is_some()
    }
}

impl Default for BaseCurrency {
    fn default() -> Self {
        BaseCurrency::Sol
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExitMode {
    Default,
    Atomic,
    Custom,
    Hold,
}

impl Default for ExitMode {
    fn default() -> Self {
        ExitMode::Default
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitConfig {
    pub base_currency: BaseCurrency,
    #[serde(default)]
    pub exit_mode: ExitMode,
    pub stop_loss_percent: Option<f64>,
    pub take_profit_percent: Option<f64>,
    pub trailing_stop_percent: Option<f64>,
    pub time_limit_minutes: Option<u32>,
    pub partial_take_profit: Option<PartialTakeProfit>,
    #[serde(default)]
    pub custom_exit_instructions: Option<String>,
    #[serde(default)]
    pub momentum_adaptive: Option<MomentumAdaptiveConfig>,
    #[serde(default)]
    pub adaptive_partial_tp: Option<AdaptivePartialTakeProfit>,
}

impl Default for ExitConfig {
    fn default() -> Self {
        // Use unified curve bonding config as default for consistency
        Self {
            base_currency: BaseCurrency::Sol,
            exit_mode: ExitMode::Default,
            stop_loss_percent: Some(30.0),       // 30% stop loss - allow curve volatility
            take_profit_percent: Some(100.0),    // 100% (2x) - tiered exit starts here
            trailing_stop_percent: Some(20.0),   // 20% trailing stop for moon bag
            time_limit_minutes: Some(15),        // 15 min - let winners run
            partial_take_profit: None,
            custom_exit_instructions: None,
            momentum_adaptive: None,
            adaptive_partial_tp: None,
        }
    }
}

impl ExitConfig {
    pub fn atomic() -> Self {
        Self {
            base_currency: BaseCurrency::Sol,
            exit_mode: ExitMode::Atomic,
            stop_loss_percent: None,
            take_profit_percent: None,
            trailing_stop_percent: None,
            time_limit_minutes: None,
            partial_take_profit: None,
            custom_exit_instructions: None,
            momentum_adaptive: None,
            adaptive_partial_tp: None,
        }
    }

    pub fn hold() -> Self {
        Self {
            base_currency: BaseCurrency::Sol,
            exit_mode: ExitMode::Hold,
            stop_loss_percent: None,
            take_profit_percent: None,
            trailing_stop_percent: None,
            time_limit_minutes: None,
            partial_take_profit: None,
            custom_exit_instructions: None,
            momentum_adaptive: None,
            adaptive_partial_tp: None,
        }
    }

    pub fn requires_monitoring(&self) -> bool {
        matches!(self.exit_mode, ExitMode::Default | ExitMode::Custom)
    }

    pub fn is_atomic(&self) -> bool {
        matches!(self.exit_mode, ExitMode::Atomic)
    }

    pub fn has_valid_exit_strategy(&self) -> bool {
        match self.exit_mode {
            ExitMode::Atomic => true,
            ExitMode::Hold => true,
            ExitMode::Default | ExitMode::Custom => {
                self.stop_loss_percent.is_some() ||
                self.take_profit_percent.is_some() ||
                self.time_limit_minutes.is_some() ||
                self.trailing_stop_percent.is_some()
            }
        }
    }

    pub fn ensure_minimum_exit_strategy(&mut self) {
        if !self.has_valid_exit_strategy() {
            self.stop_loss_percent = Some(15.0);
            self.take_profit_percent = Some(50.0);
            self.time_limit_minutes = Some(5);
        }
    }

    #[deprecated(since = "0.1.0", note = "Use for_curve_bonding() for live tokens or for_dead_token() for dead tokens")]
    pub fn for_discovered_token() -> Self {
        Self {
            base_currency: BaseCurrency::Sol,
            exit_mode: ExitMode::Default,
            stop_loss_percent: Some(30.0),
            take_profit_percent: Some(50.0),
            trailing_stop_percent: Some(15.0),
            time_limit_minutes: Some(120),
            partial_take_profit: Some(PartialTakeProfit {
                first_target_percent: 30.0,
                first_exit_percent: 50.0,
                second_target_percent: 50.0,
                second_exit_percent: 100.0,
            }),
            custom_exit_instructions: Some("Auto-created for discovered wallet token".to_string()),
            momentum_adaptive: None,
            adaptive_partial_tp: None,
        }
    }

    /// Exit config for dead/stale tokens that need immediate salvage sell
    /// Uses 0% stop loss to trigger immediate exit and high slippage tolerance
    pub fn for_dead_token() -> Self {
        Self {
            base_currency: BaseCurrency::Sol,
            exit_mode: ExitMode::Default,
            stop_loss_percent: Some(100.0), // Already underwater - just sell
            take_profit_percent: Some(0.1), // Any gain is a win
            trailing_stop_percent: None,
            time_limit_minutes: Some(1), // Immediate exit
            partial_take_profit: None,
            custom_exit_instructions: Some("DEAD TOKEN - salvage sell".to_string()),
            momentum_adaptive: None,
            adaptive_partial_tp: None,
        }
    }

    #[deprecated(since = "0.1.0", note = "Use for_curve_bonding() for live tokens or for_dead_token() for dead tokens")]
    pub fn for_discovered_with_metrics(volume_24h_sol: f64, holder_count: u32) -> Self {
        let (sl, tp, trailing, time_limit) = if volume_24h_sol > 100.0 && holder_count > 100 {
            (20.0, 40.0, Some(10.0), 360u32) // Extended from 180 to 360 min for high-metrics tokens
        } else if volume_24h_sol > 10.0 {
            (25.0, 50.0, Some(15.0), 180u32) // Extended from 120 to 180 min
        } else {
            (30.0, 75.0, Some(20.0), 90u32)  // Extended from 60 to 90 min
        };

        Self {
            base_currency: BaseCurrency::Sol,
            exit_mode: ExitMode::Default,
            stop_loss_percent: Some(sl),
            take_profit_percent: Some(tp),
            trailing_stop_percent: trailing,
            time_limit_minutes: Some(time_limit),
            partial_take_profit: if tp > 40.0 {
                Some(PartialTakeProfit {
                    first_target_percent: tp * 0.6,
                    first_exit_percent: 50.0,
                    second_target_percent: tp,
                    second_exit_percent: 100.0,
                })
            } else {
                None
            },
            custom_exit_instructions: Some(format!(
                "Auto-created: vol={:.1} SOL, holders={}",
                volume_24h_sol, holder_count
            )),
            momentum_adaptive: None,
            adaptive_partial_tp: None,
        }
    }

    /// Exit config optimized for bonding curve (pump.fun/moonshot) trading
    /// TIERED EXIT STRATEGY (2026 best practices):
    /// - Phase 1: At 2x (100% gain) - sell 50% to recover initial capital
    /// - Phase 2: At 150% (pre-migration) - sell 25% to lock in profits
    /// - Phase 3: Trailing stop on remaining 25% "moon bag"
    /// - Stop loss at 30% to handle curve volatility
    /// - Time limit extended to allow winners to run
    /// - Momentum tracking enabled to adapt exits dynamically
    pub fn for_curve_bonding() -> Self {
        Self {
            base_currency: BaseCurrency::Sol,
            exit_mode: ExitMode::Default,
            stop_loss_percent: Some(30.0),      // -30% stop loss (allow curve volatility)
            take_profit_percent: Some(100.0),   // +100% (2x) - first major TP target
            trailing_stop_percent: Some(20.0),  // 20% trailing stop for moon bag
            time_limit_minutes: Some(15),       // 15 min - extended to let winners run
            partial_take_profit: Some(PartialTakeProfit {
                first_target_percent: 100.0,    // 2x - sell 50% to recover capital
                first_exit_percent: 50.0,
                second_target_percent: 150.0,   // Pre-migration - sell 25%
                second_exit_percent: 25.0,
            }),
            custom_exit_instructions: None,
            momentum_adaptive: Some(MomentumAdaptiveConfig::default()),  // Enable momentum tracking
            adaptive_partial_tp: Some(AdaptivePartialTakeProfit::default()),
        }
    }

    /// Conservative curve config - tighter stops, faster exits
    pub fn for_curve_bonding_conservative() -> Self {
        Self {
            base_currency: BaseCurrency::Sol,
            exit_mode: ExitMode::Default,
            stop_loss_percent: Some(15.0),      // -15% stop loss
            take_profit_percent: Some(8.0),     // +8% take profit (conservative)
            trailing_stop_percent: Some(6.0),   // 6% trailing stop
            time_limit_minutes: Some(5),        // 5 min max hold - conservative is faster
            partial_take_profit: None,          // No partial - exit all at once
            custom_exit_instructions: None,
            momentum_adaptive: None,
            adaptive_partial_tp: None,
        }
    }

    /// Exit config with momentum-adaptive exits for curve bonding
    /// DEPRECATED: Now just delegates to for_curve_bonding() for unified config
    /// All curve positions use the same tiered exit strategy with momentum tracking
    pub fn for_curve_bonding_momentum_adaptive() -> Self {
        // Single source of truth - all curve trades use the same config
        Self::for_curve_bonding()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialTakeProfit {
    pub first_target_percent: f64,
    pub first_exit_percent: f64,
    pub second_target_percent: f64,
    pub second_exit_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricePoint {
    pub price: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MomentumData {
    pub price_history: Vec<PricePoint>,
    pub velocity: f64,            // Price change per minute (percent)
    pub momentum_score: f64,      // -100 to +100, current momentum strength
    pub predicted_time_to_tp_mins: Option<f32>,  // Estimated mins to reach TP
    pub momentum_decay_count: u32, // Consecutive updates with declining momentum
    #[serde(default)]
    pub consecutive_negative_readings: u32,  // For reversal confirmation
    #[serde(default)]
    pub peak_velocity: f64,       // Track maximum velocity achieved
}

impl MomentumData {
    const MAX_HISTORY_SIZE: usize = 30;  // Keep last 30 price points
    const MIN_HISTORY_FOR_PREDICTION: usize = 3;
    const DECAY_THRESHOLD: u32 = 5;  // Exit after 5 consecutive momentum declines

    pub fn update(&mut self, price: f64, entry_price: f64, take_profit_pct: Option<f64>) {
        let now = Utc::now();

        // Track previous velocity for decay detection
        let prev_velocity = self.velocity;

        // Add new price point
        self.price_history.push(PricePoint { price, timestamp: now });

        // Trim old history
        if self.price_history.len() > Self::MAX_HISTORY_SIZE {
            self.price_history.remove(0);
        }

        // Need at least 2 points to calculate velocity
        if self.price_history.len() < 2 {
            return;
        }

        // Calculate velocity (% change per minute) using recent history
        let (velocity, momentum) = self.calculate_velocity_and_momentum();
        self.velocity = velocity;
        self.momentum_score = momentum;

        // Track peak velocity
        if self.velocity > self.peak_velocity {
            self.peak_velocity = self.velocity;
        }

        // Track consecutive negative readings for reversal confirmation
        if self.velocity < 0.0 && self.momentum_score < 0.0 {
            self.consecutive_negative_readings += 1;
        } else if self.velocity > 0.0 {
            self.consecutive_negative_readings = 0;
        }

        // Predict time to target
        if let Some(tp) = take_profit_pct {
            self.predicted_time_to_tp_mins = self.predict_time_to_target(entry_price, price, tp);
        }

        // Track momentum decay (velocity dropping while still positive but slowing)
        if self.velocity < prev_velocity && self.velocity > 0.0 {
            self.momentum_decay_count += 1;
        } else if self.velocity >= prev_velocity {
            self.momentum_decay_count = 0;
        }
    }

    pub fn classify_strength(&self, config: &MomentumAdaptiveConfig) -> MomentumStrength {
        // Priority order: Reversing > Strong > Weak > Normal

        // Check for reversal (most critical)
        if self.velocity < config.reversal_velocity_threshold
            && self.momentum_score < config.reversal_momentum_score
            && self.consecutive_negative_readings >= config.reversal_confirmation_count
        {
            return MomentumStrength::Reversing;
        }

        // Check for strong momentum
        if self.velocity > config.strong_velocity_threshold
            && self.momentum_score > config.strong_momentum_score
        {
            return MomentumStrength::Strong;
        }

        // Check for weak momentum
        if self.velocity < config.weak_velocity_threshold
            || self.momentum_score < config.weak_momentum_score
        {
            return MomentumStrength::Weak;
        }

        MomentumStrength::Normal
    }

    pub fn calculate_adaptive_exit_percent(&self, base_percent: f64, config: &MomentumAdaptiveConfig) -> f64 {
        let strength = self.classify_strength(config);
        let multiplier = match strength {
            MomentumStrength::Strong => config.strong_exit_multiplier,
            MomentumStrength::Normal => config.normal_exit_multiplier,
            MomentumStrength::Weak => config.weak_exit_multiplier,
            MomentumStrength::Reversing => config.reversing_exit_multiplier,
        };

        (base_percent * multiplier).min(100.0)
    }

    pub fn calculate_adaptive_target(&self, base_target: f64, config: &MomentumAdaptiveConfig) -> f64 {
        let strength = self.classify_strength(config);
        match strength {
            MomentumStrength::Strong => {
                base_target * (1.0 + config.strong_target_extension_percent / 100.0)
            }
            MomentumStrength::Weak => {
                base_target * (1.0 - config.weak_target_reduction_percent / 100.0)
            }
            _ => base_target,
        }
    }

    pub fn should_exit_on_reversal(&self, config: &MomentumAdaptiveConfig, current_pnl_percent: f64) -> bool {
        if !config.reversal_immediate_exit {
            return false;
        }

        // Only exit on reversal if profitable above minimum threshold
        if current_pnl_percent < config.min_profit_for_momentum_exit {
            return false;
        }

        // Require reversal confirmation
        self.velocity < config.reversal_velocity_threshold
            && self.momentum_score < config.reversal_momentum_score
            && self.consecutive_negative_readings >= config.reversal_confirmation_count
    }

    fn calculate_velocity_and_momentum(&self) -> (f64, f64) {
        if self.price_history.len() < 2 {
            return (0.0, 0.0);
        }

        // Use last 5 points for short-term velocity, or all if less
        let window = self.price_history.len().min(5);
        let recent = &self.price_history[self.price_history.len() - window..];

        let first = &recent[0];
        let last = &recent[recent.len() - 1];

        let time_diff_mins = (last.timestamp - first.timestamp).num_seconds() as f64 / 60.0;
        if time_diff_mins < 0.1 {
            return (0.0, 0.0);
        }

        // Velocity = % change per minute
        let price_change_pct = ((last.price - first.price) / first.price) * 100.0;
        let velocity = price_change_pct / time_diff_mins;

        // Momentum score based on acceleration (change in velocity)
        // Compare recent velocity to older velocity
        let momentum = if self.price_history.len() >= 6 {
            let mid = self.price_history.len() / 2;
            let early = &self.price_history[0..mid];
            let early_first = &early[0];
            let early_last = &early[early.len() - 1];
            let early_time = (early_last.timestamp - early_first.timestamp).num_seconds() as f64 / 60.0;

            if early_time > 0.1 {
                let early_change = ((early_last.price - early_first.price) / early_first.price) * 100.0;
                let early_velocity = early_change / early_time;

                // Momentum = current velocity - previous velocity, scaled
                let accel = velocity - early_velocity;
                (accel * 10.0).clamp(-100.0, 100.0)
            } else {
                0.0
            }
        } else {
            (velocity * 5.0).clamp(-100.0, 100.0)  // Simple momentum from velocity
        };

        (velocity, momentum)
    }

    fn predict_time_to_target(&self, entry_price: f64, current_price: f64, take_profit_pct: f64) -> Option<f32> {
        if self.price_history.len() < Self::MIN_HISTORY_FOR_PREDICTION {
            return None;
        }

        // Current P&L %
        let current_pnl_pct = ((current_price - entry_price) / entry_price) * 100.0;

        // Distance to target
        let distance_pct = take_profit_pct - current_pnl_pct;

        // If we're already past target, time is 0
        if distance_pct <= 0.0 {
            return Some(0.0);
        }

        // If velocity is zero or negative, can't reach target
        if self.velocity <= 0.0 {
            return None;  // Can't predict - not moving toward target
        }

        // Time = distance / velocity
        let time_mins = distance_pct / self.velocity;

        // Cap at reasonable max (can't predict more than 2 hours out)
        if time_mins > 120.0 {
            return None;
        }

        Some(time_mins as f32)
    }

    pub fn should_exit_momentum_decay(&self, hold_time_mins: i64) -> bool {
        // TUNED: Less sensitive thresholds to avoid premature exits
        // Curve volatility is high - normal 10-30% swings shouldn't trigger exits
        let (velocity_threshold, score_threshold, decay_count_min) = if hold_time_mins < 5 {
            (-5.0, -60.0, 8u32)  // Very strict in first 5 min - allow high volatility (was -3, -50, 6)
        } else if hold_time_mins < 10 {
            (-3.0, -40.0, 6u32)  // Moderate strictness for 5-10 min hold
        } else {
            (-2.5, -35.0, 5u32)  // Standard after 10 min (was -2, -30, 5)
        };

        // Exit if momentum has been consistently declining
        if self.momentum_decay_count >= decay_count_min {
            return true;
        }

        // Exit if velocity turned strongly negative (confirmed downturn)
        if self.velocity < velocity_threshold && self.momentum_score < score_threshold {
            return true;
        }

        false
    }

    pub fn should_exit_predicted_time_exceeded(&self, hold_time_mins: i64, time_limit_mins: Option<u32>) -> bool {
        // Check if we've exceeded predicted time by a significant margin
        if let Some(predicted) = self.predicted_time_to_tp_mins {
            // If we've held 2x the predicted time without reaching target, exit
            let predicted_mins = predicted as i64;
            if predicted_mins > 0 && hold_time_mins > predicted_mins * 2 {
                return true;
            }
        }

        // If no velocity and holding for a while, momentum has stalled
        if self.velocity.abs() < 0.1 && hold_time_mins > 10 && self.price_history.len() >= 5 {
            return true;
        }

        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenPosition {
    pub id: Uuid,
    pub edge_id: Uuid,
    pub strategy_id: Uuid,
    pub token_mint: String,
    pub token_symbol: Option<String>,
    pub entry_amount_base: f64,
    pub entry_token_amount: f64,
    pub entry_price: f64,
    pub entry_time: DateTime<Utc>,
    pub entry_tx_signature: Option<String>,
    pub current_price: f64,
    pub current_value_base: f64,
    pub unrealized_pnl: f64,
    pub unrealized_pnl_percent: f64,
    pub high_water_mark: f64,
    pub exit_config: ExitConfig,
    pub partial_exits: Vec<PartialExit>,
    pub status: PositionStatus,
    #[serde(default)]
    pub momentum: MomentumData,
    #[serde(default)]
    pub remaining_amount_base: f64,
    #[serde(default)]
    pub remaining_token_amount: f64,
    #[serde(default)]
    pub venue: Option<String>,
    #[serde(default)]
    pub signal_source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialExit {
    pub exit_time: DateTime<Utc>,
    pub exit_percent: f64,
    pub exit_price: f64,
    pub profit_base: f64,
    pub tx_signature: Option<String>,
    pub reason: String,  // e.g., "PartialTakeProfit1", "PartialTakeProfit2"
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PositionStatus {
    Open,
    PendingExit,
    PartiallyExited,
    Closed,
    Failed,
    Orphaned,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExitReason {
    StopLoss,
    TakeProfit,
    TrailingStop,
    TimeLimit,
    Manual,
    PartialTakeProfit,
    Emergency,
    MomentumDecay,  // Momentum dropped below threshold, target unlikely to be reached
    MomentumAdaptivePartial,  // Partial exit scaled by momentum strength
    MomentumReversal,  // Immediate exit due to momentum reversal while profitable
    ExtendedTakeProfit,  // Extended target hit due to strong momentum
    Salvage,  // Dead token salvage sell with maximum slippage tolerance
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MomentumStrength {
    Strong,    // velocity > threshold AND momentum_score > threshold
    Normal,    // between strong and weak
    Weak,      // velocity < threshold OR momentum_score < threshold
    Reversing, // velocity < 0 AND momentum_score < threshold AND confirmations met
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MomentumAdaptiveConfig {
    pub strong_velocity_threshold: f64,      // %/min (default: 2.0)
    pub weak_velocity_threshold: f64,        // %/min (default: 0.5)
    pub reversal_velocity_threshold: f64,    // %/min (default: 0.0)

    pub strong_momentum_score: f64,          // default: 30.0
    pub weak_momentum_score: f64,            // default: 10.0
    pub reversal_momentum_score: f64,        // default: -20.0

    pub strong_exit_multiplier: f64,         // Sell less (default: 0.5)
    pub normal_exit_multiplier: f64,         // Sell standard (default: 1.0)
    pub weak_exit_multiplier: f64,           // Sell more (default: 1.5)
    pub reversing_exit_multiplier: f64,      // Full exit (default: 2.0)

    pub strong_target_extension_percent: f64,  // Extend target (default: 50.0)
    pub weak_target_reduction_percent: f64,    // Reduce target (default: 40.0)

    pub reversal_confirmation_count: u32,    // default: 2
    pub reversal_immediate_exit: bool,       // default: true
    pub min_profit_for_momentum_exit: f64,   // 3% minimum profit before momentum logic applies
}

impl Default for MomentumAdaptiveConfig {
    fn default() -> Self {
        Self {
            strong_velocity_threshold: 2.0,
            weak_velocity_threshold: 0.3,        // Was 0.5 - less sensitive to slow periods
            reversal_velocity_threshold: -0.5,   // Was 0.0 - require actual negative movement

            strong_momentum_score: 30.0,
            weak_momentum_score: 5.0,            // Was 10.0 - less sensitive to weak momentum
            reversal_momentum_score: -30.0,      // Was -20.0 - require stronger reversal signal

            strong_exit_multiplier: 0.5,
            normal_exit_multiplier: 1.0,
            weak_exit_multiplier: 1.3,           // Was 1.5 - sell less on weak momentum
            reversing_exit_multiplier: 1.5,      // Was 2.0 - don't panic sell on reversal

            strong_target_extension_percent: 50.0,
            weak_target_reduction_percent: 30.0, // Was 40.0 - reduce target less aggressively

            reversal_confirmation_count: 4,      // Was 2 - require more confirmations before reversal exit
            reversal_immediate_exit: true,
            min_profit_for_momentum_exit: 5.0,   // Was 3.0 - only apply momentum exits above break-even
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptivePartialTakeProfit {
    pub first_target_percent: f64,    // default: 10.0
    pub first_exit_percent: f64,      // default: 50.0
    pub second_target_percent: f64,   // default: 25.0
    pub second_exit_percent: f64,     // default: 50.0
    pub third_target_percent: f64,    // Extended target (default: 50.0)
    pub third_exit_percent: f64,      // default: 100.0 (remaining)
    pub enable_extended_targets: bool, // default: true
}

impl Default for AdaptivePartialTakeProfit {
    fn default() -> Self {
        Self {
            first_target_percent: 100.0,  // 2x = recover initial capital
            first_exit_percent: 50.0,     // Sell 50% to lock in breakeven
            second_target_percent: 150.0, // Pre-migration level (near curve completion)
            second_exit_percent: 25.0,    // Sell 25% more, keep 25% moon bag
            third_target_percent: 300.0,  // Extended moon target (3x+)
            third_exit_percent: 100.0,    // Exit remaining on extended target or trailing stop
            enable_extended_targets: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitSignal {
    pub position_id: Uuid,
    pub reason: ExitReason,
    pub exit_percent: f64,
    pub current_price: f64,
    pub triggered_at: DateTime<Utc>,
    pub urgency: ExitUrgency,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExitUrgency {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct PriorityExitEntry {
    pub position_id: Uuid,
    pub failed_attempts: u32,
    pub next_retry_at: DateTime<Utc>,
    pub is_rate_limited: bool,
}

impl PriorityExitEntry {
    pub fn new(position_id: Uuid) -> Self {
        Self {
            position_id,
            failed_attempts: 0,
            next_retry_at: Utc::now(),
            is_rate_limited: false,
        }
    }

    pub fn backoff_and_retry(&mut self, is_rate_limited: bool) {
        self.failed_attempts += 1;
        self.is_rate_limited = is_rate_limited;

        let base_delay_secs = if is_rate_limited {
            10 // Start with 10 second delay for rate limits
        } else {
            3 // 3 seconds for other failures
        };

        // Exponential backoff: 10s, 20s, 40s, 80s (max 2 min) for rate limits
        // Or: 3s, 6s, 12s, 24s (max 30s) for other failures
        let delay_secs = base_delay_secs * (1 << self.failed_attempts.min(3));
        let max_delay = if is_rate_limited { 120 } else { 30 };
        let actual_delay = delay_secs.min(max_delay);

        self.next_retry_at = Utc::now() + chrono::Duration::seconds(actual_delay as i64);
    }

    pub fn is_ready_for_retry(&self) -> bool {
        Utc::now() >= self.next_retry_at
    }

    pub fn should_give_up(&self) -> bool {
        // Give up after 10 attempts (about 10+ minutes of trying)
        self.failed_attempts >= 10
    }
}

pub struct PositionManager {
    positions: Arc<RwLock<HashMap<Uuid, OpenPosition>>>,
    positions_by_edge: Arc<RwLock<HashMap<Uuid, Uuid>>>,
    positions_by_token: Arc<RwLock<HashMap<String, Vec<Uuid>>>>,
    exit_signals: Arc<RwLock<Vec<ExitSignal>>>,
    priority_exits: Arc<RwLock<HashMap<Uuid, PriorityExitEntry>>>,
    stats: Arc<RwLock<PositionManagerStats>>,
    position_repo: Option<Arc<PositionRepository>>,
    pending_exit_retry_index: Arc<RwLock<usize>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PositionManagerStats {
    pub total_positions_opened: u64,
    pub total_positions_closed: u64,
    pub active_positions: u32,
    pub total_realized_pnl: f64,
    pub total_unrealized_pnl: f64,
    pub stop_losses_triggered: u32,
    pub take_profits_triggered: u32,
    pub time_exits_triggered: u32,
}

impl PositionManager {
    pub fn new() -> Self {
        Self {
            positions: Arc::new(RwLock::new(HashMap::new())),
            positions_by_edge: Arc::new(RwLock::new(HashMap::new())),
            positions_by_token: Arc::new(RwLock::new(HashMap::new())),
            exit_signals: Arc::new(RwLock::new(Vec::new())),
            priority_exits: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(PositionManagerStats::default())),
            position_repo: None,
            pending_exit_retry_index: Arc::new(RwLock::new(0)),
        }
    }

    pub fn with_repository(position_repo: Arc<PositionRepository>) -> Self {
        Self {
            positions: Arc::new(RwLock::new(HashMap::new())),
            positions_by_edge: Arc::new(RwLock::new(HashMap::new())),
            positions_by_token: Arc::new(RwLock::new(HashMap::new())),
            exit_signals: Arc::new(RwLock::new(Vec::new())),
            priority_exits: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(PositionManagerStats::default())),
            position_repo: Some(position_repo),
            pending_exit_retry_index: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn load_positions_from_db(&self) -> AppResult<usize> {
        let repo = match &self.position_repo {
            Some(r) => r,
            None => {
                warn!("No position repository configured - positions will not persist");
                return Ok(0);
            }
        };

        let db_positions = repo.get_open_positions().await?;
        let count = db_positions.len();

        if count > 0 {
            info!("📂 Loading {} positions from database...", count);
        }

        let mut positions = self.positions.write().await;
        let mut by_edge = self.positions_by_edge.write().await;
        let mut by_token = self.positions_by_token.write().await;
        let mut stats = self.stats.write().await;

        for position in db_positions {
            info!(
                "  ↳ Restored: {} | {} | Entry: {:.6} SOL @ {:.12}",
                &position.token_mint[..12],
                position.token_symbol.as_deref().unwrap_or("?"),
                position.entry_amount_base,
                position.entry_price
            );

            by_edge.insert(position.edge_id, position.id);
            by_token
                .entry(position.token_mint.clone())
                .or_insert_with(Vec::new)
                .push(position.id);
            debug!(
                mint = %position.token_mint,
                position_id = %position.id,
                exit_mode = ?position.exit_config.exit_mode,
                "   Indexed position in by_token map"
            );
            positions.insert(position.id, position);

            stats.total_positions_opened += 1;
            stats.active_positions += 1;
        }

        if count > 0 {
            info!("✅ Restored {} open positions from database", count);
        }

        Ok(count)
    }

    pub async fn open_position(
        &self,
        edge_id: Uuid,
        strategy_id: Uuid,
        token_mint: String,
        token_symbol: Option<String>,
        entry_amount_base: f64,
        entry_token_amount: f64,
        entry_price: f64,
        exit_config: ExitConfig,
        entry_tx_signature: Option<String>,
        venue: Option<String>,
        signal_source: Option<String>,
    ) -> AppResult<OpenPosition> {
        // Check for existing open position to prevent duplicates (race condition with reconciler)
        {
            let by_token = self.positions_by_token.read().await;
            if let Some(position_ids) = by_token.get(&token_mint) {
                let positions = self.positions.read().await;
                for pos_id in position_ids {
                    if let Some(pos) = positions.get(pos_id) {
                        if matches!(pos.status, PositionStatus::Open | PositionStatus::PendingExit | PositionStatus::PartiallyExited) {
                            info!(
                                "⏭️ Position already exists for {} (position {}) - skipping duplicate creation",
                                &token_mint[..12.min(token_mint.len())],
                                pos_id
                            );
                            return Err(crate::error::AppError::Internal(format!(
                                "Position already exists for mint {}",
                                token_mint
                            )));
                        }
                    }
                }
            }
        }

        let position_id = Uuid::new_v4();
        let now = Utc::now();

        let mut initial_momentum = MomentumData::default();
        initial_momentum.price_history.push(PricePoint {
            price: entry_price,
            timestamp: now,
        });

        let is_snipe = signal_source.as_deref() == Some("graduation_sniper");
        let snipe_emoji = if is_snipe { "🔫 " } else { "" };

        let position = OpenPosition {
            id: position_id,
            edge_id,
            strategy_id,
            token_mint: token_mint.clone(),
            token_symbol,
            entry_amount_base,
            entry_token_amount,
            entry_price,
            entry_time: now,
            entry_tx_signature,
            current_price: entry_price,
            current_value_base: entry_amount_base,
            unrealized_pnl: 0.0,
            unrealized_pnl_percent: 0.0,
            high_water_mark: entry_price,
            exit_config,
            partial_exits: Vec::new(),
            status: PositionStatus::Open,
            momentum: initial_momentum,
            remaining_amount_base: entry_amount_base,
            remaining_token_amount: entry_token_amount,
            venue,
            signal_source,
        };

        {
            let mut positions = self.positions.write().await;
            positions.insert(position_id, position.clone());
        }

        {
            let mut by_edge = self.positions_by_edge.write().await;
            by_edge.insert(edge_id, position_id);
        }

        {
            let mut by_token = self.positions_by_token.write().await;
            by_token
                .entry(token_mint.clone())
                .or_insert_with(Vec::new)
                .push(position_id);
        }

        {
            let mut stats = self.stats.write().await;
            stats.total_positions_opened += 1;
            stats.active_positions += 1;
        }

        if let Some(repo) = &self.position_repo {
            if let Err(e) = repo.save_position(&position).await {
                warn!("Failed to persist position to database: {}", e);
            } else {
                debug!("Position {} persisted to database", position_id);
            }
        }

        info!(
            "{}📈 Position opened: {} | {} @ {} | Entry: {} {} | Exit config: SL {}% / TP {}%",
            snipe_emoji,
            position_id,
            position.token_symbol.as_deref().unwrap_or(&token_mint[..8]),
            entry_price,
            entry_amount_base,
            position.exit_config.base_currency.symbol(),
            position.exit_config.stop_loss_percent.unwrap_or(0.0),
            position.exit_config.take_profit_percent.unwrap_or(0.0),
        );

        Ok(position)
    }

    pub async fn update_price(&self, token_mint: &str, current_price: f64) -> Vec<ExitSignal> {
        let mut signals = Vec::new();
        let mut positions_to_persist: Vec<(Uuid, f64, f64, f64, f64)> = Vec::new();

        let position_ids = {
            let by_token = self.positions_by_token.read().await;
            let ids = by_token.get(token_mint).cloned().unwrap_or_default();
            if ids.is_empty() {
                debug!(
                    mint = %token_mint,
                    by_token_count = by_token.len(),
                    "No positions found for mint in positions_by_token"
                );
            }
            ids
        };

        for position_id in position_ids {
            if let Some(signal) = self.check_exit_conditions(position_id, current_price).await {
                signals.push(signal);
            }
            // Gather position data for database persistence
            if let Some(pos) = self.get_position(position_id).await {
                if matches!(pos.status, PositionStatus::Open | PositionStatus::PartiallyExited) {
                    positions_to_persist.push((
                        position_id,
                        pos.current_price,
                        pos.unrealized_pnl,
                        pos.unrealized_pnl_percent,
                        pos.high_water_mark,
                    ));
                }
            }
        }

        // Persist price updates to database for open positions
        if let Some(repo) = &self.position_repo {
            for (id, price, pnl, pnl_pct, hwm) in positions_to_persist {
                if let Err(e) = repo.update_price(id, price, pnl, pnl_pct, hwm).await {
                    debug!("Failed to persist price update for {}: {}", id, e);
                }
            }
        }

        if !signals.is_empty() {
            let mut exit_signals = self.exit_signals.write().await;
            exit_signals.extend(signals.clone());
        }

        signals
    }

    async fn check_exit_conditions(
        &self,
        position_id: Uuid,
        current_price: f64,
    ) -> Option<ExitSignal> {
        let mut positions = self.positions.write().await;
        let position = positions.get_mut(&position_id)?;

        if position.status != PositionStatus::Open {
            return None;
        }

        if !position.exit_config.requires_monitoring() {
            debug!(
                position_id = %position_id,
                exit_mode = ?position.exit_config.exit_mode,
                "Skipping position monitoring (mode doesn't require it)"
            );
            return None;
        }

        position.current_price = current_price;

        // Calculate P&L percent first (price-based, avoids large token count issues)
        position.unrealized_pnl_percent = if position.entry_price > 0.0 {
            ((current_price - position.entry_price) / position.entry_price) * 100.0
        } else {
            0.0
        };

        // Calculate unrealized P&L based on REMAINING position size (not original entry)
        // This fixes the bug where P&L was calculated on full entry after partial exits
        let effective_base = if position.remaining_amount_base > 0.0 {
            position.remaining_amount_base
        } else {
            position.entry_amount_base
        };
        position.unrealized_pnl = effective_base * (position.unrealized_pnl_percent / 100.0);

        // Current value is remaining amount plus unrealized P&L
        position.current_value_base = effective_base + position.unrealized_pnl;

        if current_price > position.high_water_mark {
            position.high_water_mark = current_price;
        }

        // Update momentum data for time-to-target predictions
        let take_profit_pct = position.exit_config.take_profit_percent;
        position.momentum.update(current_price, position.entry_price, take_profit_pct);

        debug!(
            position_id = %position_id,
            price_points = position.momentum.price_history.len(),
            velocity = position.momentum.velocity,
            momentum_score = position.momentum.momentum_score,
            "📊 Momentum updated"
        );

        let config = &position.exit_config;
        let now = Utc::now();
        let hold_time_mins = (now - position.entry_time).num_minutes();

        if let Some(stop_loss) = config.stop_loss_percent {
            if position.unrealized_pnl_percent <= -stop_loss {
                position.status = PositionStatus::PendingExit;
                return Some(ExitSignal {
                    position_id,
                    reason: ExitReason::StopLoss,
                    exit_percent: 100.0,
                    current_price,
                    triggered_at: now,
                    urgency: ExitUrgency::Critical,
                });
            }
        }

        // ==================== MOMENTUM-ADAPTIVE EXIT LOGIC ====================
        // Check momentum reversal (Critical urgency, full exit to protect profits)
        if let Some(ref momentum_config) = config.momentum_adaptive {
            if position.momentum.should_exit_on_reversal(momentum_config, position.unrealized_pnl_percent) {
                let strength = position.momentum.classify_strength(momentum_config);
                tracing::info!(
                    position_id = %position_id,
                    pnl_pct = position.unrealized_pnl_percent,
                    velocity = position.momentum.velocity,
                    momentum_score = position.momentum.momentum_score,
                    strength = ?strength,
                    "🚨 MOMENTUM REVERSAL - immediate exit to protect profits"
                );
                position.status = PositionStatus::PendingExit;
                return Some(ExitSignal {
                    position_id,
                    reason: ExitReason::MomentumReversal,
                    exit_percent: 100.0,
                    current_price,
                    triggered_at: now,
                    urgency: ExitUrgency::Critical,
                });
            }
        }

        // Check momentum-adaptive partial take profits (takes priority over standard partial TP)
        if let (Some(ref momentum_config), Some(ref adaptive_tp)) = (&config.momentum_adaptive, &config.adaptive_partial_tp) {
            let strength = position.momentum.classify_strength(momentum_config);
            let already_did_first = position.partial_exits.iter()
                .any(|e| e.reason.contains("PartialTakeProfit1") || e.reason.contains("MomentumAdaptive1"));
            let already_did_second = position.partial_exits.iter()
                .any(|e| e.reason.contains("PartialTakeProfit2") || e.reason.contains("MomentumAdaptive2"));
            let already_did_third = position.partial_exits.iter()
                .any(|e| e.reason.contains("PartialTakeProfit3") || e.reason.contains("ExtendedTP"));

            // Calculate adaptive targets and exit percentages based on momentum strength
            let first_target = position.momentum.calculate_adaptive_target(
                adaptive_tp.first_target_percent, momentum_config);
            let first_exit_pct = position.momentum.calculate_adaptive_exit_percent(
                adaptive_tp.first_exit_percent, momentum_config);

            // First adaptive partial: momentum-adjusted target and size
            if !already_did_first && position.unrealized_pnl_percent >= first_target {
                tracing::info!(
                    position_id = %position_id,
                    pnl_pct = position.unrealized_pnl_percent,
                    target_pct = first_target,
                    base_target = adaptive_tp.first_target_percent,
                    exit_pct = first_exit_pct,
                    strength = ?strength,
                    "🎯 Momentum-adaptive partial #1 (strength={:?})",
                    strength
                );
                return Some(ExitSignal {
                    position_id,
                    reason: ExitReason::MomentumAdaptivePartial,
                    exit_percent: first_exit_pct,
                    current_price,
                    triggered_at: now,
                    urgency: ExitUrgency::Medium,
                });
            }

            // Second adaptive partial
            let second_target = position.momentum.calculate_adaptive_target(
                adaptive_tp.second_target_percent, momentum_config);
            let second_exit_pct = position.momentum.calculate_adaptive_exit_percent(
                adaptive_tp.second_exit_percent, momentum_config);

            if already_did_first && !already_did_second && position.unrealized_pnl_percent >= second_target {
                tracing::info!(
                    position_id = %position_id,
                    pnl_pct = position.unrealized_pnl_percent,
                    target_pct = second_target,
                    exit_pct = second_exit_pct,
                    strength = ?strength,
                    "🎯 Momentum-adaptive partial #2 (strength={:?})",
                    strength
                );
                return Some(ExitSignal {
                    position_id,
                    reason: ExitReason::MomentumAdaptivePartial,
                    exit_percent: second_exit_pct,
                    current_price,
                    triggered_at: now,
                    urgency: ExitUrgency::Medium,
                });
            }

            // Extended third target (only for strong momentum)
            if adaptive_tp.enable_extended_targets
                && already_did_first
                && already_did_second
                && !already_did_third
                && strength == MomentumStrength::Strong
            {
                let third_target = position.momentum.calculate_adaptive_target(
                    adaptive_tp.third_target_percent, momentum_config);

                if position.unrealized_pnl_percent >= third_target {
                    tracing::info!(
                        position_id = %position_id,
                        pnl_pct = position.unrealized_pnl_percent,
                        target_pct = third_target,
                        strength = ?strength,
                        "🚀 EXTENDED take profit hit (strong momentum rode to higher target)"
                    );
                    position.status = PositionStatus::PendingExit;
                    return Some(ExitSignal {
                        position_id,
                        reason: ExitReason::ExtendedTakeProfit,
                        exit_percent: adaptive_tp.third_exit_percent,
                        current_price,
                        triggered_at: now,
                        urgency: ExitUrgency::High,
                    });
                }
            }
        }
        // ==================== END MOMENTUM-ADAPTIVE LOGIC ====================

        // Check partial take profit BEFORE full take profit (standard, non-adaptive)
        if let Some(ref partial_tp) = config.partial_take_profit {
            let already_did_first_partial = position.partial_exits.iter()
                .any(|e| e.reason == "PartialTakeProfit1");
            let already_did_second_partial = position.partial_exits.iter()
                .any(|e| e.reason == "PartialTakeProfit2");

            // First partial: sell first_exit_percent at first_target_percent
            if !already_did_first_partial && position.unrealized_pnl_percent >= partial_tp.first_target_percent {
                tracing::info!(
                    position_id = %position_id,
                    pnl_pct = position.unrealized_pnl_percent,
                    target_pct = partial_tp.first_target_percent,
                    exit_pct = partial_tp.first_exit_percent,
                    "🎯 Partial take profit #1 triggered"
                );
                return Some(ExitSignal {
                    position_id,
                    reason: ExitReason::PartialTakeProfit,
                    exit_percent: partial_tp.first_exit_percent,
                    current_price,
                    triggered_at: now,
                    urgency: ExitUrgency::Medium,
                });
            }

            // Second partial: sell remaining at second_target_percent (if not at full TP yet)
            if already_did_first_partial && !already_did_second_partial
                && position.unrealized_pnl_percent >= partial_tp.second_target_percent
            {
                tracing::info!(
                    position_id = %position_id,
                    pnl_pct = position.unrealized_pnl_percent,
                    target_pct = partial_tp.second_target_percent,
                    "🎯 Partial take profit #2 triggered"
                );
                return Some(ExitSignal {
                    position_id,
                    reason: ExitReason::PartialTakeProfit,
                    exit_percent: partial_tp.second_exit_percent,
                    current_price,
                    triggered_at: now,
                    urgency: ExitUrgency::Medium,
                });
            }
        }

        if let Some(take_profit) = config.take_profit_percent {
            if position.unrealized_pnl_percent >= take_profit {
                position.status = PositionStatus::PendingExit;
                return Some(ExitSignal {
                    position_id,
                    reason: ExitReason::TakeProfit,
                    exit_percent: 100.0,
                    current_price,
                    triggered_at: now,
                    urgency: ExitUrgency::High,
                });
            }
        }

        if let Some(trailing_stop) = config.trailing_stop_percent {
            let drawdown_from_high =
                ((position.high_water_mark - current_price) / position.high_water_mark) * 100.0;
            if drawdown_from_high >= trailing_stop && position.unrealized_pnl_percent > 0.0 {
                position.status = PositionStatus::PendingExit;
                return Some(ExitSignal {
                    position_id,
                    reason: ExitReason::TrailingStop,
                    exit_percent: 100.0,
                    current_price,
                    triggered_at: now,
                    urgency: ExitUrgency::High,
                });
            }
        }

        // Tighter exit thresholds when in significant profit - protect gains (only at 10%+ to avoid false exits)
        if position.unrealized_pnl_percent > 10.0 {
            // Exit if velocity turns strongly negative while profitable (requires stronger reversal confirmation)
            // Require: velocity < -0.5 (actual decline), decay_count >= 3 (sustained), momentum_score < -10 (confirmed negative)
            if position.momentum.velocity < -0.5
               && position.momentum.momentum_decay_count >= 3
               && position.momentum.momentum_score < -10.0
            {
                tracing::info!(
                    position_id = %position_id,
                    pnl_pct = position.unrealized_pnl_percent,
                    velocity = position.momentum.velocity,
                    decay_count = position.momentum.momentum_decay_count,
                    momentum_score = position.momentum.momentum_score,
                    "📉 Profitable position confirmed reversal - exiting to protect gains"
                );
                position.status = PositionStatus::PendingExit;
                return Some(ExitSignal {
                    position_id,
                    reason: ExitReason::MomentumDecay,
                    exit_percent: 100.0,
                    current_price,
                    triggered_at: now,
                    urgency: ExitUrgency::High,
                });
            }

            // Calculate peak PnL from high water mark
            let peak_pnl_percent = ((position.high_water_mark - position.entry_price) / position.entry_price) * 100.0;
            let pnl_drop_from_peak = peak_pnl_percent - position.unrealized_pnl_percent;

            // Exit if dropped 6% from peak while still profitable (was 3%, allow more volatility)
            // Only trigger if we're still above 4% profit (above break-even after costs)
            if pnl_drop_from_peak > 6.0 && position.unrealized_pnl_percent > 4.0 {
                tracing::info!(
                    position_id = %position_id,
                    current_pnl = position.unrealized_pnl_percent,
                    peak_pnl = peak_pnl_percent,
                    drop = pnl_drop_from_peak,
                    "📉 Dropped 6%+ from peak profit - exiting to protect gains"
                );
                position.status = PositionStatus::PendingExit;
                return Some(ExitSignal {
                    position_id,
                    reason: ExitReason::TrailingStop,
                    exit_percent: 100.0,
                    current_price,
                    triggered_at: now,
                    urgency: ExitUrgency::High,
                });
            }
        }

        // NEW: Exit if momentum is slowing AND we're profitable after fees
        // This captures profits on positions where TP won't be reached but we're still in the green
        // Fees: ~1% entry + ~1% exit + ~1-2% slippage = ~4% break-even
        let profitable_after_fees = position.unrealized_pnl_percent > 5.0;
        let momentum_slowing = position.momentum.velocity < 0.5  // Not strongly positive
            && (position.momentum.momentum_decay_count >= 2     // Some decay observed
                || position.momentum.velocity.abs() < 0.3       // Velocity stalled
                || position.momentum.consecutive_negative_readings >= 2); // Recent negatives

        if profitable_after_fees && momentum_slowing && hold_time_mins >= 3 {
            tracing::info!(
                position_id = %position_id,
                pnl_pct = position.unrealized_pnl_percent,
                velocity = position.momentum.velocity,
                decay_count = position.momentum.momentum_decay_count,
                consecutive_neg = position.momentum.consecutive_negative_readings,
                hold_mins = hold_time_mins,
                "💰 Momentum slowing while profitable after fees - securing gains"
            );
            position.status = PositionStatus::PendingExit;
            return Some(ExitSignal {
                position_id,
                reason: ExitReason::MomentumDecay,
                exit_percent: 100.0,
                current_price,
                triggered_at: now,
                urgency: ExitUrgency::High,
            });
        }

        // Check momentum decay - exit if momentum has stalled or reversed
        // Only check if we're not already profitable (don't exit winners early)
        if position.unrealized_pnl_percent < 10.0 {
            if position.momentum.should_exit_momentum_decay(hold_time_mins) {
                tracing::info!(
                    position_id = %position_id,
                    velocity = position.momentum.velocity,
                    momentum_score = position.momentum.momentum_score,
                    decay_count = position.momentum.momentum_decay_count,
                    hold_mins = hold_time_mins,
                    "📉 Momentum decay detected - exiting position"
                );
                position.status = PositionStatus::PendingExit;
                return Some(ExitSignal {
                    position_id,
                    reason: ExitReason::MomentumDecay,
                    exit_percent: 100.0,
                    current_price,
                    triggered_at: now,
                    urgency: ExitUrgency::Medium,
                });
            }

            // Check if predicted time to target has been significantly exceeded
            if position.momentum.should_exit_predicted_time_exceeded(hold_time_mins, config.time_limit_minutes) {
                if let Some(predicted) = position.momentum.predicted_time_to_tp_mins {
                    tracing::info!(
                        position_id = %position_id,
                        predicted_mins = predicted,
                        actual_hold_mins = hold_time_mins,
                        velocity = position.momentum.velocity,
                        "⏰ Predicted time exceeded - momentum stalled"
                    );
                }
                position.status = PositionStatus::PendingExit;
                return Some(ExitSignal {
                    position_id,
                    reason: ExitReason::MomentumDecay,
                    exit_percent: 100.0,
                    current_price,
                    triggered_at: now,
                    urgency: ExitUrgency::Low,
                });
            }
        }

        if let Some(time_limit) = config.time_limit_minutes {
            let minutes_elapsed = (now - position.entry_time).num_minutes();
            if minutes_elapsed >= time_limit as i64 {
                position.status = PositionStatus::PendingExit;
                return Some(ExitSignal {
                    position_id,
                    reason: ExitReason::TimeLimit,
                    exit_percent: 100.0,
                    current_price,
                    triggered_at: now,
                    urgency: ExitUrgency::Medium,
                });
            }
        }

        None
    }

    pub async fn get_position(&self, position_id: Uuid) -> Option<OpenPosition> {
        let positions = self.positions.read().await;
        positions.get(&position_id).cloned()
    }

    pub async fn get_position_by_edge(&self, edge_id: Uuid) -> Option<OpenPosition> {
        let by_edge = self.positions_by_edge.read().await;
        let position_id = by_edge.get(&edge_id)?;
        let positions = self.positions.read().await;
        positions.get(position_id).cloned()
    }

    pub async fn get_open_positions(&self) -> Vec<OpenPosition> {
        let positions = self.positions.read().await;
        let all_count = positions.len();
        info!("🔍 get_open_positions called: {} positions in memory", all_count);
        let result: Vec<OpenPosition> = positions
            .values()
            .filter(|p| p.status == PositionStatus::Open)
            .cloned()
            .collect();

        if result.is_empty() && all_count > 0 {
            // Log status distribution for debugging
            let status_counts: std::collections::HashMap<&str, usize> = positions
                .values()
                .map(|p| match p.status {
                    PositionStatus::Open => "open",
                    PositionStatus::PendingExit => "pending_exit",
                    PositionStatus::PartiallyExited => "partially_exited",
                    PositionStatus::Closed => "closed",
                    PositionStatus::Failed => "failed",
                    PositionStatus::Orphaned => "orphaned",
                })
                .fold(std::collections::HashMap::new(), |mut acc, s| {
                    *acc.entry(s).or_insert(0) += 1;
                    acc
                });
            info!(
                "📊 get_open_positions: {} total in memory, 0 with status=Open. Status distribution: {:?}",
                all_count, status_counts
            );
        }

        result
    }

    pub async fn has_open_position_for_mint(&self, mint: &str) -> bool {
        let positions = self.positions.read().await;
        positions
            .values()
            .any(|p| matches!(p.status, PositionStatus::Open | PositionStatus::PendingExit | PositionStatus::PartiallyExited) && p.token_mint == mint)
    }

    pub async fn get_open_position_for_mint(&self, mint: &str) -> Option<OpenPosition> {
        let positions = self.positions.read().await;
        positions
            .values()
            .find(|p| matches!(p.status, PositionStatus::Open | PositionStatus::PendingExit | PositionStatus::PartiallyExited) && p.token_mint == mint)
            .cloned()
    }

    pub async fn get_pending_exit_signals(&self) -> Vec<ExitSignal> {
        let signals = self.exit_signals.read().await;
        signals.clone()
    }

    pub async fn get_pending_exit_positions(&self) -> Vec<OpenPosition> {
        let positions = self.positions.read().await;
        positions
            .values()
            .filter(|p| p.status == PositionStatus::PendingExit)
            .cloned()
            .collect()
    }

    pub async fn get_and_increment_retry_index(&self) -> usize {
        let mut index = self.pending_exit_retry_index.write().await;
        let current = *index;
        *index = current.wrapping_add(1);
        current
    }

    pub async fn clear_exit_signal(&self, position_id: Uuid) {
        let mut signals = self.exit_signals.write().await;
        signals.retain(|s| s.position_id != position_id);
    }

    pub async fn update_position_exit_config(
        &self,
        position_id: Uuid,
        new_config: ExitConfig,
    ) -> AppResult<OpenPosition> {
        let mut positions = self.positions.write().await;
        let position = positions
            .get_mut(&position_id)
            .ok_or_else(|| AppError::NotFound(format!("Position {} not found", position_id)))?;

        if position.status != PositionStatus::Open {
            return Err(AppError::BadRequest(format!(
                "Cannot update exit config for position {} with status {:?}",
                position_id, position.status
            )));
        }

        let old_config = position.exit_config.clone();
        position.exit_config = new_config;

        tracing::info!(
            position_id = %position_id,
            mint = %position.token_mint[..8.min(position.token_mint.len())],
            old_sl = ?old_config.stop_loss_percent,
            new_sl = ?position.exit_config.stop_loss_percent,
            old_tp = ?old_config.take_profit_percent,
            new_tp = ?position.exit_config.take_profit_percent,
            old_trailing = ?old_config.trailing_stop_percent,
            new_trailing = ?position.exit_config.trailing_stop_percent,
            "📝 Updated position exit config"
        );

        Ok(position.clone())
    }

    pub async fn close_position(
        &self,
        position_id: Uuid,
        exit_price: f64,
        realized_pnl: f64,
        exit_reason: &str,
        tx_signature: Option<String>,
    ) -> AppResult<OpenPosition> {
        let mut positions = self.positions.write().await;
        let position = positions
            .get_mut(&position_id)
            .ok_or_else(|| AppError::NotFound(format!("Position {} not found", position_id)))?;

        position.status = PositionStatus::Closed;
        position.current_price = exit_price;
        position.unrealized_pnl = 0.0;

        let closed_position = position.clone();

        {
            let mut stats = self.stats.write().await;
            stats.total_positions_closed += 1;
            stats.active_positions = stats.active_positions.saturating_sub(1);
            stats.total_realized_pnl += realized_pnl;

            match exit_reason {
                "StopLoss" => stats.stop_losses_triggered += 1,
                "TakeProfit" => stats.take_profits_triggered += 1,
                "TimeLimit" => stats.time_exits_triggered += 1,
                _ => {}
            }
        }

        if let Some(repo) = &self.position_repo {
            if let Err(e) = repo
                .close_position(
                    position_id,
                    exit_price,
                    realized_pnl,
                    exit_reason,
                    tx_signature.as_deref(),
                )
                .await
            {
                warn!("Failed to persist position close to database: {}", e);
            } else {
                debug!("Position {} close persisted to database", position_id);
            }
        }

        self.clear_exit_signal(position_id).await;

        info!(
            "📉 Position closed: {} | {} | Exit: {} | P&L: {:.4} {} | Reason: {}",
            position_id,
            closed_position
                .token_symbol
                .as_deref()
                .unwrap_or(&closed_position.token_mint[..8]),
            exit_price,
            realized_pnl,
            closed_position.exit_config.base_currency.symbol(),
            exit_reason,
        );

        Ok(closed_position)
    }

    pub async fn reset_position_status(&self, position_id: Uuid) -> AppResult<()> {
        let mut positions = self.positions.write().await;
        let position = positions
            .get_mut(&position_id)
            .ok_or_else(|| AppError::NotFound(format!("Position {} not found", position_id)))?;

        if position.status == PositionStatus::PendingExit {
            position.status = PositionStatus::Open;
            info!(
                "🔄 Position {} reset from PendingExit to Open for retry",
                position_id
            );
        }

        self.clear_exit_signal(position_id).await;

        Ok(())
    }

    pub async fn queue_priority_exit(&self, position_id: Uuid) {
        let mut priority_exits = self.priority_exits.write().await;
        if !priority_exits.contains_key(&position_id) {
            priority_exits.insert(position_id, PriorityExitEntry::new(position_id));
            info!(
                "🔥 Position {} added to HIGH PRIORITY exit queue (queue size: {})",
                position_id,
                priority_exits.len()
            );
        }
    }

    pub async fn record_priority_exit_failure(&self, position_id: Uuid, is_rate_limited: bool) {
        let mut priority_exits = self.priority_exits.write().await;

        if let Some(entry) = priority_exits.get_mut(&position_id) {
            entry.backoff_and_retry(is_rate_limited);

            if entry.should_give_up() {
                warn!(
                    "🔥❌ Position {} removed from priority queue after {} failed attempts",
                    position_id, entry.failed_attempts
                );
                priority_exits.remove(&position_id);
            } else {
                let delay_secs = (entry.next_retry_at - Utc::now()).num_seconds();
                info!(
                    "🔥⏳ Position {} backoff: attempt {}, retry in {}s{}",
                    position_id,
                    entry.failed_attempts,
                    delay_secs,
                    if is_rate_limited { " (rate limited)" } else { "" }
                );
            }
        } else {
            // Position wasn't in queue, add it with failure count
            let mut entry = PriorityExitEntry::new(position_id);
            entry.backoff_and_retry(is_rate_limited);
            priority_exits.insert(position_id, entry);
            info!(
                "🔥 Position {} added to HIGH PRIORITY exit queue with backoff (queue size: {})",
                position_id,
                priority_exits.len()
            );
        }
    }

    pub async fn drain_priority_exits(&self) -> Vec<Uuid> {
        let mut priority_exits = self.priority_exits.write().await;

        // Only return positions that are ready for retry
        let ready_exits: Vec<Uuid> = priority_exits
            .iter()
            .filter(|(_, entry)| entry.is_ready_for_retry())
            .map(|(id, _)| *id)
            .collect();

        // Remove the ready ones from the map (they'll be re-added if they fail)
        for id in &ready_exits {
            priority_exits.remove(id);
        }

        if !ready_exits.is_empty() {
            let pending_count = priority_exits.len();
            info!(
                "🔥 Draining {} positions from HIGH PRIORITY exit queue ({} still backing off)",
                ready_exits.len(),
                pending_count
            );
        }

        ready_exits
    }

    pub async fn has_priority_exits(&self) -> bool {
        let priority_exits = self.priority_exits.read().await;
        priority_exits.values().any(|e| e.is_ready_for_retry())
    }

    pub async fn priority_queue_status(&self) -> (usize, usize) {
        let priority_exits = self.priority_exits.read().await;
        let ready = priority_exits.values().filter(|e| e.is_ready_for_retry()).count();
        let backing_off = priority_exits.len() - ready;
        (ready, backing_off)
    }

    pub async fn get_stats(&self) -> PositionManagerStats {
        let stats = self.stats.read().await;
        let positions = self.positions.read().await;

        let mut current_stats = stats.clone();

        // Dynamically count all stats from actual position state to stay in sync
        current_stats.active_positions = positions
            .values()
            .filter(|p| p.status == PositionStatus::Open || p.status == PositionStatus::PendingExit)
            .count() as u32;

        current_stats.total_positions_opened = positions.len() as u64;

        current_stats.total_positions_closed = positions
            .values()
            .filter(|p| p.status == PositionStatus::Closed)
            .count() as u64;

        current_stats.total_unrealized_pnl = positions
            .values()
            .filter(|p| p.status == PositionStatus::Open)
            .map(|p| p.unrealized_pnl)
            .sum();

        current_stats
    }

    pub async fn get_total_exposure_by_base(&self, base: BaseCurrency) -> f64 {
        let positions = self.positions.read().await;
        positions
            .values()
            .filter(|p| p.status == PositionStatus::Open && p.exit_config.base_currency == base)
            .map(|p| p.current_value_base)
            .sum()
    }

    pub async fn record_partial_exit(
        &self,
        position_id: Uuid,
        exit_percent: f64,
        exit_price: f64,
        profit_sol: f64,
        tx_signature: Option<String>,
        reason: &str,
    ) -> AppResult<OpenPosition> {
        let mut positions = self.positions.write().await;
        let position = positions
            .get_mut(&position_id)
            .ok_or_else(|| AppError::NotFound(format!("Position {} not found", position_id)))?;

        let exited_amount_base = position.remaining_amount_base * (exit_percent / 100.0);
        let exited_tokens = position.remaining_token_amount * (exit_percent / 100.0);

        position.remaining_amount_base -= exited_amount_base;
        position.remaining_token_amount -= exited_tokens;

        if position.remaining_amount_base < 0.0 {
            position.remaining_amount_base = 0.0;
        }
        if position.remaining_token_amount < 0.0 {
            position.remaining_token_amount = 0.0;
        }

        let partial_exit = PartialExit {
            exit_time: Utc::now(),
            exit_percent,
            exit_price,
            profit_base: profit_sol,
            tx_signature,
            reason: reason.to_string(),
        };
        position.partial_exits.push(partial_exit);

        if position.status == PositionStatus::Open {
            position.status = PositionStatus::PartiallyExited;
        }

        info!(
            "📊 Partial exit recorded: {} | {}% @ {} | Remaining: {:.6} SOL / {:.0} tokens | Reason: {}",
            position_id,
            exit_percent,
            exit_price,
            position.remaining_amount_base,
            position.remaining_token_amount,
            reason
        );

        if let Some(repo) = &self.position_repo {
            if let Err(e) = repo.save_position(position).await {
                warn!("Failed to persist partial exit to database: {}", e);
            }
        }

        Ok(position.clone())
    }

    pub async fn emergency_close_all(&self) -> Vec<ExitSignal> {
        let mut signals = Vec::new();
        let positions = self.positions.read().await;

        for position in positions.values() {
            if position.status == PositionStatus::Open {
                signals.push(ExitSignal {
                    position_id: position.id,
                    reason: ExitReason::Emergency,
                    exit_percent: 100.0,
                    current_price: position.current_price,
                    triggered_at: Utc::now(),
                    urgency: ExitUrgency::Critical,
                });
            }
        }

        if !signals.is_empty() {
            let mut exit_signals = self.exit_signals.write().await;
            exit_signals.extend(signals.clone());
            warn!(
                "🚨 Emergency close triggered for {} positions",
                signals.len()
            );
        }

        signals
    }
}

impl Default for PositionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for PositionManager {
    fn clone(&self) -> Self {
        Self {
            positions: self.positions.clone(),
            positions_by_edge: self.positions_by_edge.clone(),
            positions_by_token: self.positions_by_token.clone(),
            exit_signals: self.exit_signals.clone(),
            priority_exits: self.priority_exits.clone(),
            stats: self.stats.clone(),
            position_repo: self.position_repo.clone(),
            pending_exit_retry_index: self.pending_exit_retry_index.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct WalletTokenHolding {
    pub mint: String,
    pub symbol: Option<String>,
    pub balance: f64,
    pub decimals: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReconciliationResult {
    pub tracked_positions: usize,
    pub discovered_tokens: Vec<WalletTokenHolding>,
    pub orphaned_positions: Vec<Uuid>,
    pub exit_strategies_fixed: Vec<Uuid>,
}

impl PositionManager {
    pub async fn reconcile_wallet_tokens(
        &self,
        wallet_tokens: &[WalletTokenHolding],
    ) -> ReconciliationResult {
        let mut discovered_tokens = Vec::new();
        let mut orphaned_positions = Vec::new();
        let mut positions_needing_exit_fix = Vec::new();

        // First pass: read-only analysis
        {
            let positions = self.positions.read().await;
            let tracked_mints: std::collections::HashSet<_> = positions
                .values()
                .filter(|p| matches!(p.status, PositionStatus::Open | PositionStatus::PendingExit | PositionStatus::PartiallyExited))
                .map(|p| p.token_mint.clone())
                .collect();

            for holding in wallet_tokens {
                if holding.mint == SOL_MINT || holding.mint == USDC_MINT || holding.mint == USDT_MINT {
                    continue;
                }

                // Check if this is a dust token (< 0.0001 balance)
                // These are too small to economically sell - tx fee exceeds value
                if holding.balance < 0.0001 {
                    if holding.balance > 0.0 && !tracked_mints.contains(&holding.mint) {
                        warn!(
                            "🧹 Dust token in wallet: {} ({}) - {:.9} tokens (too small to sell economically)",
                            holding.symbol.as_deref().unwrap_or("Unknown"),
                            &holding.mint[..8],
                            holding.balance
                        );
                    }
                    continue;
                }

                if !tracked_mints.contains(&holding.mint) {
                    info!(
                        "🔍 Discovered untracked token in wallet: {} ({}) - {} tokens",
                        holding.symbol.as_deref().unwrap_or("Unknown"),
                        &holding.mint[..8],
                        holding.balance
                    );
                    discovered_tokens.push(holding.clone());
                }
            }

            for position in positions.values() {
                if matches!(position.status, PositionStatus::Open | PositionStatus::PendingExit | PositionStatus::PartiallyExited) {
                    let wallet_has_token = wallet_tokens
                        .iter()
                        .any(|t| t.mint == position.token_mint && t.balance >= 0.0001);

                    if !wallet_has_token {
                        warn!(
                            "⚠️ Position {} ({}) [status: {:?}] has no corresponding wallet balance - may have been sold externally",
                            position.id,
                            position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8]),
                            position.status
                        );
                        orphaned_positions.push(position.id);
                    }

                    // Check for missing exit strategy
                    if !position.exit_config.has_valid_exit_strategy() {
                        warn!(
                            "⚠️ Position {} ({}) has no valid exit strategy - will assign default",
                            position.id,
                            position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8]),
                        );
                        positions_needing_exit_fix.push(position.id);
                    }
                }
            }
        }

        // Second pass: fix positions with missing exit strategies
        let mut exit_strategies_fixed = Vec::new();
        if !positions_needing_exit_fix.is_empty() {
            let mut positions = self.positions.write().await;
            for position_id in positions_needing_exit_fix {
                if let Some(position) = positions.get_mut(&position_id) {
                    position.exit_config.ensure_minimum_exit_strategy();
                    exit_strategies_fixed.push(position_id);
                    info!(
                        "🛡️ Fixed exit strategy for position {} ({}) - assigned SL {}% / TP {}%",
                        position_id,
                        position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8]),
                        position.exit_config.stop_loss_percent.unwrap_or(0.0),
                        position.exit_config.take_profit_percent.unwrap_or(0.0),
                    );

                    // Persist the fix to database
                    if let Some(repo) = &self.position_repo {
                        if let Err(e) = repo.save_position(position).await {
                            warn!("Failed to persist exit strategy fix for position {}: {}", position_id, e);
                        }
                    }
                }
            }
        }

        let tracked_count = {
            let positions = self.positions.read().await;
            positions.values()
                .filter(|p| matches!(p.status, PositionStatus::Open | PositionStatus::PendingExit | PositionStatus::PartiallyExited))
                .count()
        };

        let result = ReconciliationResult {
            tracked_positions: tracked_count,
            discovered_tokens: discovered_tokens.clone(),
            orphaned_positions: orphaned_positions.clone(),
            exit_strategies_fixed: exit_strategies_fixed.clone(),
        };

        if !discovered_tokens.is_empty() || !orphaned_positions.is_empty() || !exit_strategies_fixed.is_empty() {
            info!(
                "📊 Wallet reconciliation complete: {} tracked, {} discovered, {} orphaned, {} exit strategies fixed",
                result.tracked_positions,
                discovered_tokens.len(),
                orphaned_positions.len(),
                exit_strategies_fixed.len()
            );
        }

        result
    }

    pub async fn create_discovered_position(
        &self,
        holding: &WalletTokenHolding,
        estimated_entry_price: f64,
        estimated_entry_sol: f64,
    ) -> AppResult<OpenPosition> {
        let position = self
            .open_position(
                Uuid::new_v4(),
                Uuid::nil(),
                holding.mint.clone(),
                holding.symbol.clone(),
                estimated_entry_sol,
                holding.balance,
                estimated_entry_price,
                ExitConfig::default(),
                Some("discovered".to_string()),
                None,
                Some("wallet_discovery".to_string()),
            )
            .await?;

        info!(
            "📈 Created discovered position for {} ({}) - {} tokens @ estimated {} SOL",
            holding.symbol.as_deref().unwrap_or("Unknown"),
            &holding.mint[..8],
            holding.balance,
            estimated_entry_price
        );

        Ok(position)
    }

    pub async fn create_discovered_position_with_config(
        &self,
        holding: &WalletTokenHolding,
        estimated_entry_price: f64,
        estimated_entry_sol: f64,
        exit_config: ExitConfig,
    ) -> AppResult<OpenPosition> {
        // Check for existing position to prevent duplicates (race condition with autonomous executor)
        {
            let by_token = self.positions_by_token.read().await;
            if let Some(position_ids) = by_token.get(&holding.mint) {
                let positions = self.positions.read().await;
                for pos_id in position_ids {
                    if let Some(pos) = positions.get(pos_id) {
                        if matches!(pos.status, PositionStatus::Open | PositionStatus::PendingExit | PositionStatus::PartiallyExited) {
                            info!(
                                "⏭️ Skipping discovered position for {} - already tracked as position {}",
                                &holding.mint[..12],
                                pos_id
                            );
                            return Err(crate::error::AppError::Internal(format!(
                                "Position already exists for mint {}",
                                holding.mint
                            )));
                        }
                    }
                }
            }
        }

        // Also check database for recent positions to avoid race conditions
        if let Some(repo) = &self.position_repo {
            if let Ok(Some(existing)) = repo.get_open_by_mint(&holding.mint).await {
                info!(
                    "⏭️ Skipping discovered position for {} - found in database as position {}",
                    &holding.mint[..12],
                    existing.id
                );
                return Err(crate::error::AppError::Internal(format!(
                    "Position already exists in database for mint {}",
                    holding.mint
                )));
            }
        }

        let position = self
            .open_position(
                Uuid::new_v4(),
                Uuid::nil(),
                holding.mint.clone(),
                holding.symbol.clone(),
                estimated_entry_sol,
                holding.balance,
                estimated_entry_price,
                exit_config,
                Some("discovered_with_strategy".to_string()),
                None,
                Some("wallet_discovery".to_string()),
            )
            .await?;

        info!(
            "📈 Created discovered position with strategy for {} ({}) - {} tokens @ {:.12} SOL/token",
            holding.symbol.as_deref().unwrap_or("Unknown"),
            &holding.mint[..8],
            holding.balance,
            estimated_entry_price
        );

        Ok(position)
    }

    pub async fn mark_position_orphaned(&self, position_id: Uuid) -> AppResult<()> {
        // Check if position exists and needs status transition (read-only first)
        let should_decrement = {
            let positions = self.positions.read().await;
            positions.get(&position_id)
                .map(|p| p.status == PositionStatus::Open)
                .unwrap_or(false)
        };

        // Decrement stats if transitioning from Open (before acquiring positions write lock)
        if should_decrement {
            let mut stats = self.stats.write().await;
            if stats.active_positions > 0 {
                stats.active_positions -= 1;
                debug!("📊 Stats decremented: active_positions now {}", stats.active_positions);
            }
        }

        // Now update the position status
        let mut positions = self.positions.write().await;
        if let Some(position) = positions.get_mut(&position_id) {
            position.status = PositionStatus::Orphaned;
            info!(
                "📤 Marked position {} as orphaned (wallet balance missing)",
                position_id
            );

            if let Some(repo) = &self.position_repo {
                if let Err(e) = repo.update_status(position_id, "orphaned").await {
                    warn!("Failed to persist orphaned status to database: {}", e);
                }
            }
        }
        Ok(())
    }

    pub async fn reactivate_orphaned_position(
        &self,
        position: OpenPosition,
        new_balance: f64,
        new_price: f64,
        new_exit_config: ExitConfig,
    ) -> AppResult<OpenPosition> {
        let mut reactivated = position.clone();
        reactivated.status = PositionStatus::Open;
        reactivated.entry_token_amount = new_balance;
        reactivated.entry_price = new_price;
        reactivated.current_price = new_price;
        reactivated.high_water_mark = new_price;
        reactivated.unrealized_pnl = 0.0;
        reactivated.unrealized_pnl_percent = 0.0;
        reactivated.exit_config = new_exit_config;
        // Note: entry_amount_base is preserved from the original position creation

        {
            let mut positions = self.positions.write().await;
            positions.insert(reactivated.id, reactivated.clone());
        }

        {
            let mut by_edge = self.positions_by_edge.write().await;
            by_edge.insert(reactivated.edge_id, reactivated.id);
        }

        {
            let mut by_token = self.positions_by_token.write().await;
            by_token
                .entry(reactivated.token_mint.clone())
                .or_insert_with(Vec::new)
                .push(reactivated.id);
        }

        {
            let mut stats = self.stats.write().await;
            stats.active_positions += 1;
        }

        if let Some(repo) = &self.position_repo {
            if let Err(e) = repo.reactivate_position(reactivated.id, new_balance, new_price).await {
                warn!("Failed to persist reactivated position to database: {}", e);
            }
            if let Err(e) = repo.save_position(&reactivated).await {
                warn!("Failed to update reactivated position exit config: {}", e);
            }
        }

        info!(
            "♻️ Reactivated orphaned position {} for {} | Balance: {} | Price: {:.12} | Exit: SL {}%/TP {}%",
            reactivated.id,
            reactivated.token_symbol.as_deref().unwrap_or(&reactivated.token_mint[..8]),
            new_balance,
            new_price,
            reactivated.exit_config.stop_loss_percent.unwrap_or(0.0),
            reactivated.exit_config.take_profit_percent.unwrap_or(0.0),
        );

        Ok(reactivated)
    }

    pub async fn get_orphaned_position_by_mint(&self, mint: &str) -> Option<OpenPosition> {
        if let Some(repo) = &self.position_repo {
            match repo.get_orphaned_position_by_mint(mint).await {
                Ok(pos) => pos,
                Err(e) => {
                    warn!("Failed to query orphaned position: {}", e);
                    None
                }
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_currency_mints() {
        assert_eq!(
            BaseCurrency::Sol.mint(),
            "So11111111111111111111111111111111111111112"
        );
        assert_eq!(
            BaseCurrency::Usdc.mint(),
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
        );
        assert!(BaseCurrency::is_base_currency(USDC_MINT));
        assert!(!BaseCurrency::is_base_currency("random_mint"));
    }

    #[tokio::test]
    async fn test_position_lifecycle() {
        let manager = PositionManager::new();

        let position = manager
            .open_position(
                Uuid::new_v4(),
                Uuid::new_v4(),
                "TokenMint123".to_string(),
                Some("TEST".to_string()),
                1.0,
                1000.0,
                0.001,
                ExitConfig::default(),
                None,
                None,
                None,
            )
            .await
            .unwrap();

        assert_eq!(position.status, PositionStatus::Open);

        let stats = manager.get_stats().await;
        assert_eq!(stats.active_positions, 1);
    }

    #[tokio::test]
    async fn test_stop_loss_trigger() {
        let manager = PositionManager::new();

        let _position = manager
            .open_position(
                Uuid::new_v4(),
                Uuid::new_v4(),
                "TokenMint123".to_string(),
                Some("TEST".to_string()),
                1.0,
                1000.0,
                0.001,
                ExitConfig {
                    stop_loss_percent: Some(10.0),
                    ..Default::default()
                },
                None,
                None,
                None,
            )
            .await
            .unwrap();

        let signals = manager.update_price("TokenMint123", 0.00085).await;

        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].reason, ExitReason::StopLoss);
    }

    #[tokio::test]
    async fn test_take_profit_trigger() {
        let manager = PositionManager::new();

        let _position = manager
            .open_position(
                Uuid::new_v4(),
                Uuid::new_v4(),
                "TokenMint123".to_string(),
                Some("TEST".to_string()),
                1.0,
                1000.0,
                0.001,
                ExitConfig {
                    take_profit_percent: Some(25.0),
                    ..Default::default()
                },
                None,
                None,
                None,
            )
            .await
            .unwrap();

        let signals = manager.update_price("TokenMint123", 0.00130).await;

        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].reason, ExitReason::TakeProfit);
    }

    #[test]
    fn test_exit_config_has_valid_exit_strategy_atomic() {
        let config = ExitConfig {
            exit_mode: ExitMode::Atomic,
            ..Default::default()
        };
        assert!(config.has_valid_exit_strategy());
    }

    #[test]
    fn test_exit_config_has_valid_exit_strategy_hold() {
        let config = ExitConfig {
            exit_mode: ExitMode::Hold,
            ..Default::default()
        };
        assert!(config.has_valid_exit_strategy());
    }

    #[test]
    fn test_exit_config_has_valid_exit_strategy_default_with_stop_loss() {
        let config = ExitConfig {
            exit_mode: ExitMode::Default,
            stop_loss_percent: Some(15.0),
            ..Default::default()
        };
        assert!(config.has_valid_exit_strategy());
    }

    #[test]
    fn test_exit_config_has_valid_exit_strategy_default_with_take_profit() {
        let config = ExitConfig {
            exit_mode: ExitMode::Default,
            take_profit_percent: Some(50.0),
            ..Default::default()
        };
        assert!(config.has_valid_exit_strategy());
    }

    #[test]
    fn test_exit_config_has_valid_exit_strategy_default_with_time_limit() {
        let config = ExitConfig {
            exit_mode: ExitMode::Default,
            time_limit_minutes: Some(120),
            ..Default::default()
        };
        assert!(config.has_valid_exit_strategy());
    }

    #[test]
    fn test_exit_config_has_valid_exit_strategy_default_with_trailing_stop() {
        let config = ExitConfig {
            exit_mode: ExitMode::Default,
            trailing_stop_percent: Some(10.0),
            ..Default::default()
        };
        assert!(config.has_valid_exit_strategy());
    }

    #[test]
    fn test_exit_config_invalid_when_default_mode_with_no_params() {
        let config = ExitConfig {
            exit_mode: ExitMode::Default,
            stop_loss_percent: None,
            take_profit_percent: None,
            time_limit_minutes: None,
            trailing_stop_percent: None,
            ..Default::default()
        };
        assert!(!config.has_valid_exit_strategy());
    }

    #[test]
    fn test_exit_config_invalid_when_custom_mode_with_no_params() {
        let config = ExitConfig {
            exit_mode: ExitMode::Custom,
            stop_loss_percent: None,
            take_profit_percent: None,
            time_limit_minutes: None,
            trailing_stop_percent: None,
            ..Default::default()
        };
        assert!(!config.has_valid_exit_strategy());
    }

    #[test]
    fn test_ensure_minimum_exit_strategy_adds_defaults() {
        let mut config = ExitConfig {
            exit_mode: ExitMode::Default,
            stop_loss_percent: None,
            take_profit_percent: None,
            time_limit_minutes: None,
            trailing_stop_percent: None,
            ..Default::default()
        };
        assert!(!config.has_valid_exit_strategy());

        config.ensure_minimum_exit_strategy();

        assert!(config.has_valid_exit_strategy());
        assert_eq!(config.stop_loss_percent, Some(15.0));
        assert_eq!(config.take_profit_percent, Some(50.0));
        assert_eq!(config.time_limit_minutes, Some(5));
    }

    #[test]
    fn test_ensure_minimum_exit_strategy_preserves_existing() {
        let mut config = ExitConfig {
            exit_mode: ExitMode::Default,
            stop_loss_percent: Some(20.0),
            take_profit_percent: None,
            time_limit_minutes: None,
            trailing_stop_percent: None,
            ..Default::default()
        };
        assert!(config.has_valid_exit_strategy());

        config.ensure_minimum_exit_strategy();

        assert_eq!(config.stop_loss_percent, Some(20.0));
        assert_eq!(config.take_profit_percent, None);
        assert_eq!(config.time_limit_minutes, None);
    }
}
