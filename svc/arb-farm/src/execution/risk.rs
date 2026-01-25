use serde::{Deserialize, Serialize};
use sqlx::PgPool;
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
    db_pool: Option<PgPool>,
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
    #[serde(default = "default_take_profit")]
    pub take_profit_percent: f64,
    #[serde(default = "default_trailing_stop")]
    pub trailing_stop_percent: f64,
    #[serde(default = "default_time_limit")]
    pub time_limit_minutes: u32,
}

// Unified defaults - matches ExitConfig::for_curve_bonding()
fn default_take_profit() -> f64 { 100.0 }  // 100% (2x) - tiered exit starts here
fn default_trailing_stop() -> f64 { 20.0 } // 20% trailing for moon bag
fn default_time_limit() -> u32 { 15 }      // 15 min - let winners run

impl Default for RiskConfig {
    fn default() -> Self {
        // Unified config - matches ExitConfig::for_curve_bonding()
        Self {
            max_position_sol: 0.3,              // 0.3 SOL per position (medium risk)
            daily_loss_limit_sol: 1.0,          // 1 SOL daily loss limit
            max_drawdown_percent: 40.0,         // 40% stop loss - allow curve volatility (increased from 30% per LLM consensus)
            max_concurrent_positions: 10,       // 10 concurrent positions
            max_position_per_token_sol: 0.3,    // Same as max_position
            cooldown_after_loss_ms: 5000,
            volatility_scaling_enabled: true,
            auto_pause_on_drawdown: true,
            take_profit_percent: 100.0,         // 100% (2x) - tiered exit starts here
            trailing_stop_percent: 20.0,        // 20% trailing for moon bag
            time_limit_minutes: 15,             // 15 min - let winners run
        }
    }
}

impl RiskConfig {
    /// LOW risk profile - default for safety
    pub fn low() -> Self {
        Self {
            max_position_sol: 0.02,
            daily_loss_limit_sol: 0.1,
            max_drawdown_percent: 15.0,
            max_concurrent_positions: 2,
            max_position_per_token_sol: 0.02,
            cooldown_after_loss_ms: 10000,
            volatility_scaling_enabled: true,
            auto_pause_on_drawdown: true,
            take_profit_percent: 10.0,
            trailing_stop_percent: 8.0,
            time_limit_minutes: 5,
        }
    }

    pub fn dev_testing() -> Self {
        Self {
            max_position_sol: 5.0,
            daily_loss_limit_sol: 2.0,
            max_drawdown_percent: 40.0,         // 40% stop loss (matches default)
            max_concurrent_positions: 10,
            max_position_per_token_sol: 2.0,
            cooldown_after_loss_ms: 2000,
            volatility_scaling_enabled: true,
            auto_pause_on_drawdown: false,
            take_profit_percent: 100.0,         // Same tiered exit
            trailing_stop_percent: 20.0,        // Same trailing
            time_limit_minutes: 15,             // Same time limit
        }
    }

    pub fn conservative() -> Self {
        Self {
            max_position_sol: 1.0,
            daily_loss_limit_sol: 0.5,
            max_drawdown_percent: 15.0,
            max_concurrent_positions: 3,
            max_position_per_token_sol: 0.5,
            cooldown_after_loss_ms: 10000,
            volatility_scaling_enabled: true,
            auto_pause_on_drawdown: true,
            take_profit_percent: 12.0,
            trailing_stop_percent: 10.0,
            time_limit_minutes: 5,
        }
    }

    pub fn medium() -> Self {
        // Default is already medium - just return it
        Self::default()
    }

    pub fn aggressive() -> Self {
        Self {
            max_position_sol: 10.0,
            daily_loss_limit_sol: 5.0,
            max_drawdown_percent: 40.0,         // 40% stop loss (matches default)
            max_concurrent_positions: 20,
            max_position_per_token_sol: 5.0,
            cooldown_after_loss_ms: 1000,
            volatility_scaling_enabled: false,
            auto_pause_on_drawdown: false,
            take_profit_percent: 100.0,         // Same tiered exit
            trailing_stop_percent: 20.0,        // Same trailing
            time_limit_minutes: 15,             // Same time limit
        }
    }
}

#[derive(Debug, Clone, Default)]
struct DailyStats {
    db_id: Option<Uuid>,
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
            db_pool: None,
        }
    }

    pub fn with_db_pool(mut self, pool: PgPool) -> Self {
        self.db_pool = Some(pool);
        self
    }

    pub async fn load_daily_stats_from_db(&self) -> AppResult<()> {
        let Some(pool) = &self.db_pool else {
            return Ok(());
        };

        let today = chrono::Utc::now().date_naive();

        let row: Option<(Uuid, chrono::NaiveDate, i64, i64, i32, i32, i32, Option<chrono::DateTime<chrono::Utc>>)> = sqlx::query_as(
            r#"
            SELECT id, date, total_profit_lamports, total_loss_lamports,
                   trade_count, winning_trades, losing_trades, last_loss_at
            FROM daily_risk_stats
            WHERE date = $1
            "#,
        )
        .bind(today)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to load daily stats: {}", e)))?;

        if let Some((id, date, profit, loss, trades, wins, losses, last_loss)) = row {
            let mut stats = self.daily_stats.write().await;
            stats.db_id = Some(id);
            stats.date = date;
            stats.total_profit_lamports = profit;
            stats.total_loss_lamports = loss;
            stats.trade_count = trades as u32;
            stats.winning_trades = wins as u32;
            stats.losing_trades = losses as u32;
            stats.last_loss_at = last_loss;

            let net_pnl = (profit - loss.abs()) as f64 / 1e9;
            tracing::info!(
                "📊 Loaded daily risk stats from DB: date={}, net_pnl={:.4} SOL, trades={}, wins={}, losses={}",
                date, net_pnl, trades, wins, losses
            );
        }

        Ok(())
    }

    async fn persist_daily_stats(&self, stats: &DailyStats) {
        let Some(pool) = &self.db_pool else {
            return;
        };

        let result = if let Some(id) = stats.db_id {
            sqlx::query(
                r#"
                UPDATE daily_risk_stats
                SET total_profit_lamports = $2,
                    total_loss_lamports = $3,
                    trade_count = $4,
                    winning_trades = $5,
                    losing_trades = $6,
                    last_loss_at = $7,
                    updated_at = NOW()
                WHERE id = $1
                "#,
            )
            .bind(id)
            .bind(stats.total_profit_lamports)
            .bind(stats.total_loss_lamports)
            .bind(stats.trade_count as i32)
            .bind(stats.winning_trades as i32)
            .bind(stats.losing_trades as i32)
            .bind(stats.last_loss_at)
            .execute(pool)
            .await
        } else {
            sqlx::query(
                r#"
                INSERT INTO daily_risk_stats (date, total_profit_lamports, total_loss_lamports,
                                               trade_count, winning_trades, losing_trades, last_loss_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (date) DO UPDATE SET
                    total_profit_lamports = EXCLUDED.total_profit_lamports,
                    total_loss_lamports = EXCLUDED.total_loss_lamports,
                    trade_count = EXCLUDED.trade_count,
                    winning_trades = EXCLUDED.winning_trades,
                    losing_trades = EXCLUDED.losing_trades,
                    last_loss_at = EXCLUDED.last_loss_at,
                    updated_at = NOW()
                "#,
            )
            .bind(stats.date)
            .bind(stats.total_profit_lamports)
            .bind(stats.total_loss_lamports)
            .bind(stats.trade_count as i32)
            .bind(stats.winning_trades as i32)
            .bind(stats.losing_trades as i32)
            .bind(stats.last_loss_at)
            .execute(pool)
            .await
        };

        if let Err(e) = result {
            tracing::warn!("Failed to persist daily risk stats: {}", e);
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
        // Use proper rounding: (p + 5000) / 10000 to avoid truncation bias
        let profit_bps = edge.estimated_profit_lamports.map(|p| ((p + 5000) / 10000) as u16).unwrap_or(0);
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
        // Use write lock to atomically check date and reset if needed
        // This prevents race conditions at midnight UTC where one thread
        // could check old limits while another resets
        let mut stats = self.daily_stats.write().await;
        let today = chrono::Utc::now().date_naive();

        // Atomically reset if new day - prevents race condition at day boundary
        if stats.date != today {
            let old_date = stats.date;
            *stats = DailyStats {
                db_id: None,
                date: today,
                ..Default::default()
            };
            tracing::info!(
                old_date = %old_date,
                new_date = %today,
                "📊 Daily risk stats reset for new day"
            );
            // After reset, no losses yet today
            return None;
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
        let stats_clone = {
            let mut stats = self.daily_stats.write().await;
            let today = chrono::Utc::now().date_naive();

            // Reset stats if new day
            if stats.date != today {
                *stats = DailyStats {
                    db_id: None, // New day, no DB record yet
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

            stats.clone()
        };

        // Persist to DB (fire-and-forget, don't block trading)
        self.persist_daily_stats(&stats_clone).await;
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
