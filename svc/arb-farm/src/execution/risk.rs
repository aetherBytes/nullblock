use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::events::AtomicityLevel;
use crate::models::{Edge, RiskParams};

pub struct RiskManager {
    config: RiskConfig,
    daily_stats: Arc<RwLock<DailyStats>>,
    position_tracker: Arc<RwLock<PositionTracker>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConfig {
    pub max_position_sol: f64,
    pub daily_loss_limit_sol: f64,
    pub max_drawdown_percent: f64,
    pub max_concurrent_positions: u32,
    pub max_position_per_token_sol: f64,
    pub cooldown_after_loss_ms: u64,
    pub volatility_scaling_enabled: bool,
    pub auto_pause_on_drawdown: bool,
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            max_position_sol: 1.0,
            daily_loss_limit_sol: 0.5,
            max_drawdown_percent: 20.0,
            max_concurrent_positions: 5,
            max_position_per_token_sol: 0.5,
            cooldown_after_loss_ms: 5000,
            volatility_scaling_enabled: true,
            auto_pause_on_drawdown: true,
        }
    }
}

impl RiskConfig {
    pub fn dev_testing() -> Self {
        Self {
            max_position_sol: 5.0,
            daily_loss_limit_sol: 2.0,
            max_drawdown_percent: 40.0,
            max_concurrent_positions: 10,
            max_position_per_token_sol: 2.0,
            cooldown_after_loss_ms: 2000,
            volatility_scaling_enabled: true,
            auto_pause_on_drawdown: false,
        }
    }

    pub fn conservative() -> Self {
        Self {
            max_position_sol: 1.0,
            daily_loss_limit_sol: 0.5,
            max_drawdown_percent: 10.0,
            max_concurrent_positions: 3,
            max_position_per_token_sol: 0.5,
            cooldown_after_loss_ms: 10000,
            volatility_scaling_enabled: true,
            auto_pause_on_drawdown: true,
        }
    }

    pub fn aggressive() -> Self {
        Self {
            max_position_sol: 10.0,
            daily_loss_limit_sol: 5.0,
            max_drawdown_percent: 50.0,
            max_concurrent_positions: 20,
            max_position_per_token_sol: 5.0,
            cooldown_after_loss_ms: 1000,
            volatility_scaling_enabled: false,
            auto_pause_on_drawdown: false,
        }
    }
}

#[derive(Debug, Clone, Default)]
struct DailyStats {
    date: chrono::NaiveDate,
    total_profit_lamports: i64,
    total_loss_lamports: i64,
    trade_count: u32,
    winning_trades: u32,
    losing_trades: u32,
    last_loss_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Default)]
struct PositionTracker {
    active_positions: HashMap<Uuid, ActivePosition>,
    token_exposure: HashMap<String, f64>, // token_mint -> SOL exposure
}

#[derive(Debug, Clone)]
struct ActivePosition {
    edge_id: Uuid,
    token_mint: Option<String>,
    size_sol: f64,
    opened_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskCheck {
    pub edge_id: Uuid,
    pub passed: bool,
    pub violations: Vec<RiskViolation>,
    pub adjusted_size_sol: Option<f64>,
    pub risk_score: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskViolation {
    pub rule: String,
    pub message: String,
    pub severity: ViolationSeverity,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ViolationSeverity {
    Warning,
    Block,
    Critical,
}

impl RiskManager {
    pub fn new(config: RiskConfig) -> Self {
        Self {
            config,
            daily_stats: Arc::new(RwLock::new(DailyStats::default())),
            position_tracker: Arc::new(RwLock::new(PositionTracker::default())),
        }
    }

    pub async fn check_edge(&self, edge: &Edge, strategy_params: &RiskParams) -> RiskCheck {
        let mut violations = Vec::new();
        let mut passed = true;

        // Check 1: Daily loss limit
        if let Some(violation) = self.check_daily_loss_limit().await {
            if violation.severity == ViolationSeverity::Block {
                passed = false;
            }
            violations.push(violation);
        }

        // Check 2: Position size limit
        let estimated_size_sol = edge.estimated_profit_lamports.unwrap_or(0) as f64 / 1e9;
        if estimated_size_sol > self.config.max_position_sol {
            violations.push(RiskViolation {
                rule: "max_position_size".to_string(),
                message: format!(
                    "Position size {} SOL exceeds max {} SOL",
                    estimated_size_sol, self.config.max_position_sol
                ),
                severity: ViolationSeverity::Block,
            });
            passed = false;
        }

        // Check 3: Concurrent positions
        if let Some(violation) = self.check_concurrent_positions().await {
            if violation.severity == ViolationSeverity::Block {
                passed = false;
            }
            violations.push(violation);
        }

        // Check 4: Cooldown after loss
        if let Some(violation) = self.check_loss_cooldown().await {
            if violation.severity == ViolationSeverity::Block {
                passed = false;
            }
            violations.push(violation);
        }

        // Check 5: Risk score threshold
        let risk_score = edge.risk_score.unwrap_or(50);
        if risk_score > strategy_params.max_risk_score {
            violations.push(RiskViolation {
                rule: "max_risk_score".to_string(),
                message: format!(
                    "Risk score {} exceeds max {} for strategy",
                    risk_score, strategy_params.max_risk_score
                ),
                severity: ViolationSeverity::Block,
            });
            passed = false;
        }

        // Check 6: Minimum profit threshold
        let profit_bps = edge.estimated_profit_lamports.map(|p| (p / 10000) as u16).unwrap_or(0);
        if profit_bps < strategy_params.min_profit_bps {
            violations.push(RiskViolation {
                rule: "min_profit".to_string(),
                message: format!(
                    "Estimated profit {} bps below minimum {} bps",
                    profit_bps, strategy_params.min_profit_bps
                ),
                severity: ViolationSeverity::Block,
            });
            passed = false;
        }

        // For atomic trades, relax some checks
        if edge.atomicity == AtomicityLevel::FullyAtomic && edge.simulated_profit_guaranteed {
            // Remove blocking violations for guaranteed-profit atomic trades
            violations.retain(|v| {
                v.rule != "max_risk_score" && v.rule != "min_profit"
            });
            passed = violations.iter().all(|v| v.severity != ViolationSeverity::Block);
        }

        // Calculate adjusted size based on volatility if enabled
        let adjusted_size = if self.config.volatility_scaling_enabled {
            Some(self.calculate_volatility_adjusted_size(estimated_size_sol, risk_score))
        } else {
            None
        };

        RiskCheck {
            edge_id: edge.id,
            passed,
            violations,
            adjusted_size_sol: adjusted_size,
            risk_score,
        }
    }

    async fn check_daily_loss_limit(&self) -> Option<RiskViolation> {
        let stats = self.daily_stats.read().await;
        let today = chrono::Utc::now().date_naive();

        if stats.date != today {
            return None; // Stats not for today, will be reset
        }

        let net_pnl_sol = (stats.total_profit_lamports - stats.total_loss_lamports.abs()) as f64 / 1e9;

        if net_pnl_sol < -self.config.daily_loss_limit_sol {
            return Some(RiskViolation {
                rule: "daily_loss_limit".to_string(),
                message: format!(
                    "Daily loss {} SOL exceeds limit {} SOL",
                    net_pnl_sol.abs(),
                    self.config.daily_loss_limit_sol
                ),
                severity: ViolationSeverity::Block,
            });
        }

        if net_pnl_sol < -self.config.daily_loss_limit_sol * 0.8 {
            return Some(RiskViolation {
                rule: "daily_loss_warning".to_string(),
                message: format!(
                    "Approaching daily loss limit: {} SOL of {} SOL",
                    net_pnl_sol.abs(),
                    self.config.daily_loss_limit_sol
                ),
                severity: ViolationSeverity::Warning,
            });
        }

        None
    }

    async fn check_concurrent_positions(&self) -> Option<RiskViolation> {
        let tracker = self.position_tracker.read().await;

        if tracker.active_positions.len() >= self.config.max_concurrent_positions as usize {
            return Some(RiskViolation {
                rule: "max_concurrent_positions".to_string(),
                message: format!(
                    "Max concurrent positions ({}) reached",
                    self.config.max_concurrent_positions
                ),
                severity: ViolationSeverity::Block,
            });
        }

        None
    }

    async fn check_loss_cooldown(&self) -> Option<RiskViolation> {
        let stats = self.daily_stats.read().await;

        if let Some(last_loss) = stats.last_loss_at {
            let elapsed = chrono::Utc::now().signed_duration_since(last_loss);
            let cooldown = chrono::Duration::milliseconds(self.config.cooldown_after_loss_ms as i64);

            if elapsed < cooldown {
                let remaining = cooldown - elapsed;
                return Some(RiskViolation {
                    rule: "loss_cooldown".to_string(),
                    message: format!(
                        "In cooldown period after loss. {} ms remaining",
                        remaining.num_milliseconds()
                    ),
                    severity: ViolationSeverity::Warning,
                });
            }
        }

        None
    }

    fn calculate_volatility_adjusted_size(&self, base_size: f64, risk_score: i32) -> f64 {
        // Higher risk score = smaller position
        let risk_factor = 1.0 - (risk_score as f64 / 200.0); // 0.5 to 1.0
        let adjusted = base_size * risk_factor.max(0.25);
        adjusted.min(self.config.max_position_sol)
    }

    pub async fn record_trade_result(&self, profit_lamports: i64) {
        let mut stats = self.daily_stats.write().await;
        let today = chrono::Utc::now().date_naive();

        // Reset stats if new day
        if stats.date != today {
            *stats = DailyStats {
                date: today,
                ..Default::default()
            };
        }

        stats.trade_count += 1;

        if profit_lamports >= 0 {
            stats.total_profit_lamports += profit_lamports;
            stats.winning_trades += 1;
        } else {
            stats.total_loss_lamports += profit_lamports.abs();
            stats.losing_trades += 1;
            stats.last_loss_at = Some(chrono::Utc::now());
        }
    }

    pub async fn open_position(&self, edge_id: Uuid, token_mint: Option<String>, size_sol: f64) {
        let mut tracker = self.position_tracker.write().await;

        tracker.active_positions.insert(
            edge_id,
            ActivePosition {
                edge_id,
                token_mint: token_mint.clone(),
                size_sol,
                opened_at: chrono::Utc::now(),
            },
        );

        if let Some(mint) = token_mint {
            *tracker.token_exposure.entry(mint).or_insert(0.0) += size_sol;
        }
    }

    pub async fn close_position(&self, edge_id: Uuid) {
        let mut tracker = self.position_tracker.write().await;

        if let Some(position) = tracker.active_positions.remove(&edge_id) {
            if let Some(mint) = position.token_mint {
                if let Some(exposure) = tracker.token_exposure.get_mut(&mint) {
                    *exposure -= position.size_sol;
                    if *exposure <= 0.0 {
                        tracker.token_exposure.remove(&mint);
                    }
                }
            }
        }
    }

    pub async fn get_stats(&self) -> DailyRiskStats {
        let stats = self.daily_stats.read().await;
        let tracker = self.position_tracker.read().await;

        DailyRiskStats {
            date: stats.date.to_string(),
            total_profit_sol: stats.total_profit_lamports as f64 / 1e9,
            total_loss_sol: stats.total_loss_lamports as f64 / 1e9,
            net_pnl_sol: (stats.total_profit_lamports - stats.total_loss_lamports.abs()) as f64 / 1e9,
            trade_count: stats.trade_count,
            win_rate: if stats.trade_count > 0 {
                stats.winning_trades as f64 / stats.trade_count as f64
            } else {
                0.0
            },
            active_positions: tracker.active_positions.len() as u32,
            daily_loss_remaining_sol: self.config.daily_loss_limit_sol
                + (stats.total_profit_lamports - stats.total_loss_lamports.abs()) as f64 / 1e9,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyRiskStats {
    pub date: String,
    pub total_profit_sol: f64,
    pub total_loss_sol: f64,
    pub net_pnl_sol: f64,
    pub trade_count: u32,
    pub win_rate: f64,
    pub active_positions: u32,
    pub daily_loss_remaining_sol: f64,
}
