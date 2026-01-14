use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration, Timelike};

use crate::error::AppResult;
use super::strategy_extract::{ExtractedStrategy, ConditionType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConfig {
    pub period_days: u32,
    pub initial_capital_sol: f64,
    pub max_position_size_sol: f64,
    pub slippage_bps: u16,
    pub fee_bps: u16,
    pub include_gas_costs: bool,
    pub gas_cost_per_trade_sol: f64,
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            period_days: 30,
            initial_capital_sol: 10.0,
            max_position_size_sol: 1.0,
            slippage_bps: 50,
            fee_bps: 30,
            include_gas_costs: true,
            gas_cost_per_trade_sol: 0.001,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResult {
    pub id: Uuid,
    pub strategy_id: Uuid,
    pub config: BacktestConfig,
    pub summary: BacktestSummary,
    pub trades: Vec<SimulatedTrade>,
    pub equity_curve: Vec<EquityPoint>,
    pub metrics: BacktestMetrics,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestSummary {
    pub total_trades: u32,
    pub winning_trades: u32,
    pub losing_trades: u32,
    pub win_rate: f64,
    pub total_profit_sol: f64,
    pub total_profit_percent: f64,
    pub max_drawdown_percent: f64,
    pub sharpe_ratio: f64,
    pub profit_factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatedTrade {
    pub id: Uuid,
    pub entry_time: DateTime<Utc>,
    pub exit_time: DateTime<Utc>,
    pub entry_price: f64,
    pub exit_price: f64,
    pub position_size_sol: f64,
    pub profit_sol: f64,
    pub profit_percent: f64,
    pub fees_paid_sol: f64,
    pub entry_reason: String,
    pub exit_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquityPoint {
    pub timestamp: DateTime<Utc>,
    pub equity_sol: f64,
    pub drawdown_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestMetrics {
    pub avg_trade_duration_minutes: f64,
    pub avg_profit_per_trade_sol: f64,
    pub avg_profit_per_trade_percent: f64,
    pub best_trade_profit_percent: f64,
    pub worst_trade_loss_percent: f64,
    pub longest_winning_streak: u32,
    pub longest_losing_streak: u32,
    pub avg_trades_per_day: f64,
    pub sortino_ratio: f64,
    pub calmar_ratio: f64,
}

pub struct BacktestEngine {
    historical_data: Vec<PriceCandle>,
}

#[derive(Debug, Clone)]
pub struct PriceCandle {
    pub timestamp: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

impl BacktestEngine {
    pub fn new() -> Self {
        Self {
            historical_data: Vec::new(),
        }
    }

    pub fn with_data(mut self, data: Vec<PriceCandle>) -> Self {
        self.historical_data = data;
        self
    }

    pub async fn run(&self, strategy: &ExtractedStrategy, config: BacktestConfig) -> AppResult<BacktestResult> {
        let started_at = Utc::now();

        let data = if self.historical_data.is_empty() {
            self.generate_simulated_data(&config)
        } else {
            self.historical_data.clone()
        };

        let (trades, equity_curve) = self.simulate_strategy(strategy, &data, &config);

        let summary = self.calculate_summary(&trades, config.initial_capital_sol);
        let metrics = self.calculate_metrics(&trades, &equity_curve, config.period_days);

        Ok(BacktestResult {
            id: Uuid::new_v4(),
            strategy_id: strategy.id,
            config,
            summary,
            trades,
            equity_curve,
            metrics,
            started_at,
            completed_at: Utc::now(),
        })
    }

    fn generate_simulated_data(&self, config: &BacktestConfig) -> Vec<PriceCandle> {
        let mut candles = Vec::new();
        let mut price = 1.0;
        let now = Utc::now();
        let start = now - Duration::days(config.period_days as i64);

        let candles_per_day = 24;
        let total_candles = config.period_days * candles_per_day;

        for i in 0..total_candles {
            let timestamp = start + Duration::hours(i as i64);

            let volatility = 0.02;
            let drift = 0.0001;
            let random_return = (rand_simple() - 0.5) * volatility + drift;

            let open = price;
            price *= 1.0 + random_return;
            let close = price;

            let high = open.max(close) * (1.0 + rand_simple() * 0.01);
            let low = open.min(close) * (1.0 - rand_simple() * 0.01);
            let volume = 10000.0 * (0.5 + rand_simple());

            candles.push(PriceCandle {
                timestamp,
                open,
                high,
                low,
                close,
                volume,
            });
        }

        candles
    }

    fn simulate_strategy(
        &self,
        strategy: &ExtractedStrategy,
        data: &[PriceCandle],
        config: &BacktestConfig,
    ) -> (Vec<SimulatedTrade>, Vec<EquityPoint>) {
        let mut trades = Vec::new();
        let mut equity_curve = Vec::new();
        let mut equity = config.initial_capital_sol;
        let mut peak_equity = equity;
        let mut in_position = false;
        let mut entry_price = 0.0;
        let mut entry_time = Utc::now();
        let mut entry_reason = String::new();

        for (i, candle) in data.iter().enumerate() {
            let lookback = if i >= 20 { &data[i - 20..i] } else { &data[0..i] };

            if !in_position {
                if let Some(reason) = self.check_entry_conditions(strategy, candle, lookback) {
                    in_position = true;
                    entry_price = candle.close * (1.0 + config.slippage_bps as f64 / 10000.0);
                    entry_time = candle.timestamp;
                    entry_reason = reason;
                }
            } else if let Some(reason) = self.check_exit_conditions(strategy, candle, entry_price, lookback) {
                let exit_price = candle.close * (1.0 - config.slippage_bps as f64 / 10000.0);
                let position_size = config.max_position_size_sol.min(equity * 0.5);

                let gross_profit_percent = (exit_price - entry_price) / entry_price * 100.0;
                let fee_percent = config.fee_bps as f64 / 50.0;
                let net_profit_percent = gross_profit_percent - fee_percent;

                let profit_sol = position_size * net_profit_percent / 100.0;
                let fees = position_size * fee_percent / 100.0;

                let gas = if config.include_gas_costs { config.gas_cost_per_trade_sol * 2.0 } else { 0.0 };

                equity += profit_sol - gas;

                trades.push(SimulatedTrade {
                    id: Uuid::new_v4(),
                    entry_time,
                    exit_time: candle.timestamp,
                    entry_price,
                    exit_price,
                    position_size_sol: position_size,
                    profit_sol: profit_sol - gas,
                    profit_percent: net_profit_percent,
                    fees_paid_sol: fees + gas,
                    entry_reason: entry_reason.clone(),
                    exit_reason: reason,
                });

                in_position = false;
            }

            if candle.timestamp.hour() == 0 {
                let drawdown = if peak_equity > 0.0 {
                    (peak_equity - equity) / peak_equity * 100.0
                } else {
                    0.0
                };

                equity_curve.push(EquityPoint {
                    timestamp: candle.timestamp,
                    equity_sol: equity,
                    drawdown_percent: drawdown.max(0.0),
                });

                if equity > peak_equity {
                    peak_equity = equity;
                }
            }
        }

        (trades, equity_curve)
    }

    fn check_entry_conditions(
        &self,
        strategy: &ExtractedStrategy,
        candle: &PriceCandle,
        lookback: &[PriceCandle],
    ) -> Option<String> {
        for condition in &strategy.entry_conditions {
            match condition.condition_type {
                ConditionType::PriceAbove => {
                    if let Some(threshold) = condition.parameters.get("threshold").and_then(|v| v.as_f64()) {
                        if candle.close > threshold {
                            return Some(format!("Price above {}", threshold));
                        }
                    }
                }
                ConditionType::PriceBelow => {
                    if let Some(threshold) = condition.parameters.get("threshold").and_then(|v| v.as_f64()) {
                        if candle.close < threshold {
                            return Some(format!("Price below {}", threshold));
                        }
                    }
                }
                ConditionType::VolumeSpike => {
                    if !lookback.is_empty() {
                        let avg_volume: f64 = lookback.iter().map(|c| c.volume).sum::<f64>() / lookback.len() as f64;
                        let threshold_multiplier = condition.parameters
                            .get("multiplier")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(2.0);

                        if candle.volume > avg_volume * threshold_multiplier {
                            return Some("Volume spike detected".to_string());
                        }
                    }
                }
                ConditionType::PercentageGain => {
                    if !lookback.is_empty() {
                        let first_price = lookback.first().map(|c| c.close).unwrap_or(candle.close);
                        let gain = (candle.close - first_price) / first_price * 100.0;
                        let threshold = condition.parameters
                            .get("threshold")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(5.0);

                        if gain >= threshold {
                            return Some(format!("{}% gain achieved", gain.round()));
                        }
                    }
                }
                ConditionType::CurveProgress => {
                    let progress = condition.parameters
                        .get("threshold")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(30.0);

                    let simulated_progress = (candle.timestamp.timestamp() % 100) as f64;
                    if simulated_progress <= progress {
                        return Some(format!("Curve at {}%", progress));
                    }
                }
                _ => {
                    if rand_simple() < 0.05 {
                        return Some(condition.description.clone());
                    }
                }
            }
        }

        if strategy.entry_conditions.is_empty() && rand_simple() < 0.03 {
            return Some("Random entry (no conditions defined)".to_string());
        }

        None
    }

    fn check_exit_conditions(
        &self,
        strategy: &ExtractedStrategy,
        candle: &PriceCandle,
        entry_price: f64,
        _lookback: &[PriceCandle],
    ) -> Option<String> {
        let current_profit_percent = (candle.close - entry_price) / entry_price * 100.0;

        if let Some(stop_loss) = strategy.risk_params.stop_loss_percent {
            if current_profit_percent <= -stop_loss {
                return Some(format!("Stop loss hit at {}%", current_profit_percent.round()));
            }
        }

        if let Some(take_profit) = strategy.risk_params.take_profit_percent {
            if current_profit_percent >= take_profit {
                return Some(format!("Take profit hit at {}%", current_profit_percent.round()));
            }
        }

        for condition in &strategy.exit_conditions {
            match condition.condition_type {
                ConditionType::PercentageGain => {
                    let target = condition.parameters
                        .get("multiplier")
                        .and_then(|v| v.as_f64())
                        .map(|m| (m - 1.0) * 100.0)
                        .or_else(|| condition.parameters.get("threshold").and_then(|v| v.as_f64()))
                        .unwrap_or(50.0);

                    if current_profit_percent >= target {
                        return Some(format!("Target {}% reached", target));
                    }
                }
                ConditionType::PercentageLoss => {
                    let threshold = condition.parameters
                        .get("threshold")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(10.0);

                    if current_profit_percent <= -threshold {
                        return Some(format!("Loss limit {}% hit", threshold));
                    }
                }
                ConditionType::CurveProgress => {
                    let exit_progress = condition.parameters
                        .get("threshold")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(80.0);

                    let simulated_progress = (candle.timestamp.timestamp() % 100) as f64;
                    if simulated_progress >= exit_progress {
                        return Some(format!("Curve reached {}%", exit_progress));
                    }
                }
                _ => {}
            }
        }

        if rand_simple() < 0.01 {
            return Some("Time-based exit".to_string());
        }

        None
    }

    fn calculate_summary(&self, trades: &[SimulatedTrade], initial_capital: f64) -> BacktestSummary {
        let total_trades = trades.len() as u32;
        let winning_trades = trades.iter().filter(|t| t.profit_sol > 0.0).count() as u32;
        let losing_trades = trades.iter().filter(|t| t.profit_sol < 0.0).count() as u32;

        let win_rate = if total_trades > 0 {
            winning_trades as f64 / total_trades as f64 * 100.0
        } else {
            0.0
        };

        let total_profit_sol: f64 = trades.iter().map(|t| t.profit_sol).sum();
        let total_profit_percent = total_profit_sol / initial_capital * 100.0;

        let gross_profits: f64 = trades.iter()
            .filter(|t| t.profit_sol > 0.0)
            .map(|t| t.profit_sol)
            .sum();
        let gross_losses: f64 = trades.iter()
            .filter(|t| t.profit_sol < 0.0)
            .map(|t| t.profit_sol.abs())
            .sum();

        let profit_factor = if gross_losses > 0.0 {
            gross_profits / gross_losses
        } else if gross_profits > 0.0 {
            f64::INFINITY
        } else {
            0.0
        };

        let mut equity = initial_capital;
        let mut peak = initial_capital;
        let mut max_drawdown = 0.0;

        for trade in trades {
            equity += trade.profit_sol;
            if equity > peak {
                peak = equity;
            }
            let drawdown = (peak - equity) / peak * 100.0;
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
        }

        let returns: Vec<f64> = trades.iter().map(|t| t.profit_percent).collect();
        let sharpe_ratio = self.calculate_sharpe_ratio(&returns);

        BacktestSummary {
            total_trades,
            winning_trades,
            losing_trades,
            win_rate,
            total_profit_sol,
            total_profit_percent,
            max_drawdown_percent: max_drawdown,
            sharpe_ratio,
            profit_factor,
        }
    }

    fn calculate_sharpe_ratio(&self, returns: &[f64]) -> f64 {
        if returns.is_empty() {
            return 0.0;
        }

        let mean: f64 = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance: f64 = returns.iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>() / returns.len() as f64;
        let std_dev = variance.sqrt();

        if std_dev > 0.0 {
            mean / std_dev * (252.0_f64).sqrt()
        } else {
            0.0
        }
    }

    fn calculate_metrics(&self, trades: &[SimulatedTrade], equity_curve: &[EquityPoint], period_days: u32) -> BacktestMetrics {
        let avg_duration: f64 = if !trades.is_empty() {
            trades.iter()
                .map(|t| (t.exit_time - t.entry_time).num_minutes() as f64)
                .sum::<f64>() / trades.len() as f64
        } else {
            0.0
        };

        let avg_profit_sol = if !trades.is_empty() {
            trades.iter().map(|t| t.profit_sol).sum::<f64>() / trades.len() as f64
        } else {
            0.0
        };

        let avg_profit_percent = if !trades.is_empty() {
            trades.iter().map(|t| t.profit_percent).sum::<f64>() / trades.len() as f64
        } else {
            0.0
        };

        let best_trade = trades.iter()
            .map(|t| t.profit_percent)
            .fold(f64::NEG_INFINITY, f64::max);

        let worst_trade = trades.iter()
            .map(|t| t.profit_percent)
            .fold(f64::INFINITY, f64::min);

        let (win_streak, lose_streak) = self.calculate_streaks(trades);

        let avg_trades_per_day = trades.len() as f64 / period_days.max(1) as f64;

        let max_drawdown = equity_curve.iter()
            .map(|e| e.drawdown_percent)
            .fold(0.0, f64::max);

        let total_return = if !equity_curve.is_empty() {
            let first = equity_curve.first().map(|e| e.equity_sol).unwrap_or(1.0);
            let last = equity_curve.last().map(|e| e.equity_sol).unwrap_or(1.0);
            (last - first) / first * 100.0
        } else {
            0.0
        };

        let calmar_ratio = if max_drawdown > 0.0 {
            total_return / max_drawdown
        } else {
            0.0
        };

        let negative_returns: Vec<f64> = trades.iter()
            .filter(|t| t.profit_percent < 0.0)
            .map(|t| t.profit_percent)
            .collect();

        let downside_deviation = if !negative_returns.is_empty() {
            let variance: f64 = negative_returns.iter()
                .map(|r| r.powi(2))
                .sum::<f64>() / negative_returns.len() as f64;
            variance.sqrt()
        } else {
            0.0
        };

        let sortino_ratio = if downside_deviation > 0.0 {
            avg_profit_percent / downside_deviation * (252.0_f64).sqrt()
        } else {
            0.0
        };

        BacktestMetrics {
            avg_trade_duration_minutes: avg_duration,
            avg_profit_per_trade_sol: avg_profit_sol,
            avg_profit_per_trade_percent: avg_profit_percent,
            best_trade_profit_percent: if best_trade.is_finite() { best_trade } else { 0.0 },
            worst_trade_loss_percent: if worst_trade.is_finite() { worst_trade } else { 0.0 },
            longest_winning_streak: win_streak,
            longest_losing_streak: lose_streak,
            avg_trades_per_day,
            sortino_ratio,
            calmar_ratio,
        }
    }

    fn calculate_streaks(&self, trades: &[SimulatedTrade]) -> (u32, u32) {
        let mut max_win_streak = 0u32;
        let mut max_lose_streak = 0u32;
        let mut current_win_streak = 0u32;
        let mut current_lose_streak = 0u32;

        for trade in trades {
            if trade.profit_sol > 0.0 {
                current_win_streak += 1;
                current_lose_streak = 0;
                max_win_streak = max_win_streak.max(current_win_streak);
            } else if trade.profit_sol < 0.0 {
                current_lose_streak += 1;
                current_win_streak = 0;
                max_lose_streak = max_lose_streak.max(current_lose_streak);
            }
        }

        (max_win_streak, max_lose_streak)
    }
}

impl Default for BacktestEngine {
    fn default() -> Self {
        Self::new()
    }
}

fn rand_simple() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos as f64 / 1_000_000_000.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backtest_config_default() {
        let config = BacktestConfig::default();
        assert_eq!(config.period_days, 30);
        assert_eq!(config.initial_capital_sol, 10.0);
    }

    #[test]
    fn test_calculate_sharpe_ratio() {
        let engine = BacktestEngine::new();
        let returns = vec![1.0, 2.0, -1.0, 3.0, 0.5];
        let sharpe = engine.calculate_sharpe_ratio(&returns);
        assert!(sharpe > 0.0);
    }
}
