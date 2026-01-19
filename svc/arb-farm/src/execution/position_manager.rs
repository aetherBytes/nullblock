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
}

impl Default for ExitConfig {
    fn default() -> Self {
        Self {
            base_currency: BaseCurrency::Sol,
            exit_mode: ExitMode::Default,
            stop_loss_percent: Some(10.0),
            take_profit_percent: Some(25.0),
            trailing_stop_percent: None,
            time_limit_minutes: Some(60),
            partial_take_profit: None,
            custom_exit_instructions: None,
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
        }
    }

    pub fn requires_monitoring(&self) -> bool {
        matches!(self.exit_mode, ExitMode::Default | ExitMode::Custom)
    }

    pub fn is_atomic(&self) -> bool {
        matches!(self.exit_mode, ExitMode::Atomic)
    }

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
        }
    }

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
        }
    }

    /// Exit config optimized for bonding curve (pump.fun/moonshot) trading
    /// OPTIMIZED for faster profit capture:
    /// - 10% stop loss limits drawdown
    /// - 25% take profit is achievable target (coins often hit 5-8% then reverse)
    /// - 5% trailing stop locks in gains quickly
    /// - 30 min time limit forces exit before curve goes cold
    /// - Tighter momentum thresholds protect profitable positions
    pub fn for_curve_bonding() -> Self {
        Self {
            base_currency: BaseCurrency::Sol,
            exit_mode: ExitMode::Default,
            stop_loss_percent: Some(10.0),      // -10% stop loss
            take_profit_percent: Some(25.0),    // +25% take profit (achievable)
            trailing_stop_percent: Some(5.0),   // 5% trailing stop (tight)
            time_limit_minutes: Some(30),       // 30 min max hold
            partial_take_profit: Some(PartialTakeProfit {
                first_target_percent: 10.0,     // Sell 50% at +10%
                first_exit_percent: 50.0,
                second_target_percent: 25.0,    // Sell remaining at +25%
                second_exit_percent: 100.0,
            }),
            custom_exit_instructions: None,
        }
    }

    /// Conservative curve config - tighter stops, faster exits
    pub fn for_curve_bonding_conservative() -> Self {
        Self {
            base_currency: BaseCurrency::Sol,
            exit_mode: ExitMode::Default,
            stop_loss_percent: Some(10.0),      // -10% stop loss
            take_profit_percent: Some(30.0),    // +30% take profit
            trailing_stop_percent: Some(10.0),  // 10% trailing stop
            time_limit_minutes: Some(30),       // 30 min max hold
            partial_take_profit: None,          // No partial - exit all at once
            custom_exit_instructions: None,
        }
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
        // Give positions more time in first 5 minutes (curves are volatile at launch)
        // Require stronger decay signal before exiting early
        let (velocity_threshold, score_threshold, decay_count_min) = if hold_time_mins < 5 {
            (-2.0, -40.0, 5u32)  // Stricter thresholds in first 5 min
        } else {
            (-0.5, -20.0, 3u32) // Standard thresholds after 5 min
        };

        // Exit if momentum has been consistently declining
        if self.momentum_decay_count >= decay_count_min {
            return true;
        }

        // Exit if velocity turned negative while we expected positive movement
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

pub struct PositionManager {
    positions: Arc<RwLock<HashMap<Uuid, OpenPosition>>>,
    positions_by_edge: Arc<RwLock<HashMap<Uuid, Uuid>>>,
    positions_by_token: Arc<RwLock<HashMap<String, Vec<Uuid>>>>,
    exit_signals: Arc<RwLock<Vec<ExitSignal>>>,
    priority_exits: Arc<RwLock<Vec<Uuid>>>,
    stats: Arc<RwLock<PositionManagerStats>>,
    position_repo: Option<Arc<PositionRepository>>,
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
            priority_exits: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(PositionManagerStats::default())),
            position_repo: None,
        }
    }

    pub fn with_repository(position_repo: Arc<PositionRepository>) -> Self {
        Self {
            positions: Arc::new(RwLock::new(HashMap::new())),
            positions_by_edge: Arc::new(RwLock::new(HashMap::new())),
            positions_by_token: Arc::new(RwLock::new(HashMap::new())),
            exit_signals: Arc::new(RwLock::new(Vec::new())),
            priority_exits: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(PositionManagerStats::default())),
            position_repo: Some(position_repo),
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
    ) -> AppResult<OpenPosition> {
        let position_id = Uuid::new_v4();
        let now = Utc::now();

        let mut initial_momentum = MomentumData::default();
        initial_momentum.price_history.push(PricePoint {
            price: entry_price,
            timestamp: now,
        });

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
            "📈 Position opened: {} | {} @ {} | Entry: {} {} | Exit config: SL {}% / TP {}%",
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

        // Check partial take profit BEFORE full take profit
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

        // Tighter exit thresholds when in profit - protect gains aggressively
        if position.unrealized_pnl_percent > 5.0 {
            // Exit if velocity turns negative while profitable (momentum reversal)
            if position.momentum.velocity < 0.0 && position.momentum.momentum_decay_count >= 2 {
                tracing::info!(
                    position_id = %position_id,
                    pnl_pct = position.unrealized_pnl_percent,
                    velocity = position.momentum.velocity,
                    decay_count = position.momentum.momentum_decay_count,
                    "📉 Profitable position losing momentum - exiting to protect gains"
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

            // Exit if dropped 3% from peak while profitable
            if pnl_drop_from_peak > 3.0 {
                tracing::info!(
                    position_id = %position_id,
                    current_pnl = position.unrealized_pnl_percent,
                    peak_pnl = peak_pnl_percent,
                    drop = pnl_drop_from_peak,
                    "📉 Dropped 3%+ from peak profit - exiting to protect gains"
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
        positions
            .values()
            .filter(|p| p.status == PositionStatus::Open)
            .cloned()
            .collect()
    }

    pub async fn has_open_position_for_mint(&self, mint: &str) -> bool {
        let positions = self.positions.read().await;
        positions
            .values()
            .any(|p| p.status == PositionStatus::Open && p.token_mint == mint)
    }

    pub async fn get_open_position_for_mint(&self, mint: &str) -> Option<OpenPosition> {
        let positions = self.positions.read().await;
        positions
            .values()
            .find(|p| p.status == PositionStatus::Open && p.token_mint == mint)
            .cloned()
    }

    pub async fn get_pending_exit_signals(&self) -> Vec<ExitSignal> {
        let signals = self.exit_signals.read().await;
        signals.clone()
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
        if !priority_exits.contains(&position_id) {
            priority_exits.push(position_id);
            info!(
                "🔥 Position {} added to HIGH PRIORITY exit queue (queue size: {})",
                position_id,
                priority_exits.len()
            );
        }
    }

    pub async fn drain_priority_exits(&self) -> Vec<Uuid> {
        let mut priority_exits = self.priority_exits.write().await;
        let exits: Vec<Uuid> = priority_exits.drain(..).collect();
        if !exits.is_empty() {
            info!("🔥 Draining {} positions from HIGH PRIORITY exit queue", exits.len());
        }
        exits
    }

    pub async fn has_priority_exits(&self) -> bool {
        let priority_exits = self.priority_exits.read().await;
        !priority_exits.is_empty()
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
}

impl PositionManager {
    pub async fn reconcile_wallet_tokens(
        &self,
        wallet_tokens: &[WalletTokenHolding],
    ) -> ReconciliationResult {
        let positions = self.positions.read().await;
        let tracked_mints: std::collections::HashSet<_> = positions
            .values()
            .filter(|p| p.status == PositionStatus::Open)
            .map(|p| p.token_mint.clone())
            .collect();

        let mut discovered_tokens = Vec::new();
        let mut orphaned_positions = Vec::new();

        for holding in wallet_tokens {
            if holding.mint == SOL_MINT || holding.mint == USDC_MINT || holding.mint == USDT_MINT {
                continue;
            }

            // Lower threshold to capture tokens with fractional balances
            // 0.0001 filters dust while allowing small positions
            if holding.balance < 0.0001 {
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
            if position.status == PositionStatus::Open {
                let wallet_has_token = wallet_tokens
                    .iter()
                    .any(|t| t.mint == position.token_mint && t.balance >= 0.0001);

                if !wallet_has_token {
                    warn!(
                        "⚠️ Position {} ({}) has no corresponding wallet balance - may have been sold externally",
                        position.id,
                        position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8])
                    );
                    orphaned_positions.push(position.id);
                }
            }
        }

        let result = ReconciliationResult {
            tracked_positions: tracked_mints.len(),
            discovered_tokens: discovered_tokens.clone(),
            orphaned_positions: orphaned_positions.clone(),
        };

        if !discovered_tokens.is_empty() || !orphaned_positions.is_empty() {
            info!(
                "📊 Wallet reconciliation complete: {} tracked, {} discovered, {} orphaned",
                result.tracked_positions,
                discovered_tokens.len(),
                orphaned_positions.len()
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
        const MAX_DISCOVERED_ENTRY_SOL: f64 = 0.1;
        const DEFAULT_DISCOVERED_ENTRY_SOL: f64 = 0.02;

        let mut reactivated = position.clone();
        reactivated.status = PositionStatus::Open;
        reactivated.entry_token_amount = new_balance;
        reactivated.entry_price = new_price;
        reactivated.current_price = new_price;
        reactivated.high_water_mark = new_price;
        reactivated.unrealized_pnl = 0.0;
        reactivated.unrealized_pnl_percent = 0.0;
        reactivated.exit_config = new_exit_config;

        // Cap entry amount for reactivated positions if it looks inflated
        if reactivated.entry_amount_base > MAX_DISCOVERED_ENTRY_SOL {
            info!(
                "   📉 Capping inflated entry {:.4} SOL to {:.4} SOL for reactivated position",
                reactivated.entry_amount_base, DEFAULT_DISCOVERED_ENTRY_SOL
            );
            reactivated.entry_amount_base = DEFAULT_DISCOVERED_ENTRY_SOL;
        }

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
            )
            .await
            .unwrap();

        let signals = manager.update_price("TokenMint123", 0.00130).await;

        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].reason, ExitReason::TakeProfit);
    }
}
