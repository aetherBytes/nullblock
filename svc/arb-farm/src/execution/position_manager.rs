use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialTakeProfit {
    pub first_target_percent: f64,
    pub first_exit_percent: f64,
    pub second_target_percent: f64,
    pub second_exit_percent: f64,
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
    pub current_price: f64,
    pub current_value_base: f64,
    pub unrealized_pnl: f64,
    pub unrealized_pnl_percent: f64,
    pub high_water_mark: f64,
    pub exit_config: ExitConfig,
    pub partial_exits: Vec<PartialExit>,
    pub status: PositionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialExit {
    pub exit_time: DateTime<Utc>,
    pub exit_percent: f64,
    pub exit_price: f64,
    pub profit_base: f64,
    pub tx_signature: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PositionStatus {
    Open,
    PendingExit,
    PartiallyExited,
    Closed,
    Failed,
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
    stats: Arc<RwLock<PositionManagerStats>>,
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
            stats: Arc::new(RwLock::new(PositionManagerStats::default())),
        }
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
    ) -> AppResult<OpenPosition> {
        let position_id = Uuid::new_v4();
        let now = Utc::now();

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
            current_price: entry_price,
            current_value_base: entry_amount_base,
            unrealized_pnl: 0.0,
            unrealized_pnl_percent: 0.0,
            high_water_mark: entry_price,
            exit_config,
            partial_exits: Vec::new(),
            status: PositionStatus::Open,
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
            by_token.get(token_mint).cloned().unwrap_or_default()
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
            return None;
        }

        position.current_price = current_price;
        position.current_value_base =
            (position.entry_token_amount * current_price) / position.entry_price
                * position.entry_amount_base
                / position.entry_token_amount;

        let current_value = position.entry_token_amount * current_price;
        let entry_value = position.entry_token_amount * position.entry_price;
        position.unrealized_pnl = current_value - entry_value;
        position.unrealized_pnl_percent =
            ((current_price - position.entry_price) / position.entry_price) * 100.0;

        if current_price > position.high_water_mark {
            position.high_water_mark = current_price;
        }

        let config = &position.exit_config;
        let now = Utc::now();

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

    pub async fn close_position(
        &self,
        position_id: Uuid,
        exit_price: f64,
        realized_pnl: f64,
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
        }

        self.clear_exit_signal(position_id).await;

        info!(
            "📉 Position closed: {} | {} | Exit: {} | P&L: {:.4} {}",
            position_id,
            closed_position
                .token_symbol
                .as_deref()
                .unwrap_or(&closed_position.token_mint[..8]),
            exit_price,
            realized_pnl,
            closed_position.exit_config.base_currency.symbol(),
        );

        Ok(closed_position)
    }

    pub async fn get_stats(&self) -> PositionManagerStats {
        let stats = self.stats.read().await;
        let positions = self.positions.read().await;

        let mut current_stats = stats.clone();
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

        let position = manager
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

        let position = manager
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
            )
            .await
            .unwrap();

        let signals = manager.update_price("TokenMint123", 0.00130).await;

        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].reason, ExitReason::TakeProfit);
    }
}
