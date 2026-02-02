use chrono::{DateTime, Datelike, Duration, Utc};
use std::collections::HashMap;
use std::sync::Arc;

use crate::database::PositionRepository;
use crate::engrams::{DailyMetrics, EngramsClient, StrategyMetrics, TradeHighlight, VenueMetrics};

pub struct MetricsAggregator {
    position_repo: Arc<PositionRepository>,
    engrams_client: Arc<EngramsClient>,
    wallet_address: String,
}

impl MetricsAggregator {
    pub fn new(
        position_repo: Arc<PositionRepository>,
        engrams_client: Arc<EngramsClient>,
        wallet_address: String,
    ) -> Self {
        Self {
            position_repo,
            engrams_client,
            wallet_address,
        }
    }

    pub async fn aggregate_daily_metrics(
        &self,
        date: DateTime<Utc>,
    ) -> Result<DailyMetrics, String> {
        let start_of_day = date
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| "Invalid date".to_string())?;
        let start = DateTime::<Utc>::from_naive_utc_and_offset(start_of_day, Utc);
        let end = start + Duration::days(1);

        let positions = self
            .position_repo
            .get_closed_positions_for_period(start, end)
            .await
            .map_err(|e| format!("Failed to fetch positions: {}", e))?;

        if positions.is_empty() {
            return Ok(DailyMetrics {
                period: date.format("%Y-%m-%d").to_string(),
                total_trades: 0,
                winning_trades: 0,
                win_rate: 0.0,
                total_pnl_sol: 0.0,
                avg_trade_pnl: 0.0,
                max_drawdown_percent: 0.0,
                best_trade: None,
                worst_trade: None,
                by_venue: HashMap::new(),
                by_strategy: HashMap::new(),
            });
        }

        let total_trades = positions.len() as u32;
        let mut total_pnl = 0.0;
        let mut winning_trades = 0u32;
        let mut best_trade: Option<TradeHighlight> = None;
        let mut worst_trade: Option<TradeHighlight> = None;
        let mut by_venue: HashMap<String, VenueMetrics> = HashMap::new();
        let mut by_strategy: HashMap<String, StrategyMetrics> = HashMap::new();
        let mut cumulative_pnl = 0.0;
        let mut peak_pnl = 0.0;
        let mut max_drawdown = 0.0;

        for pos in &positions {
            let pnl = pos.unrealized_pnl;
            total_pnl += pnl;
            cumulative_pnl += pnl;

            if pnl > 0.0 {
                winning_trades += 1;
            }

            if cumulative_pnl > peak_pnl {
                peak_pnl = cumulative_pnl;
            }
            let drawdown = if peak_pnl > 0.0 {
                (peak_pnl - cumulative_pnl) / peak_pnl * 100.0
            } else {
                0.0
            };
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }

            let token_display = pos
                .token_symbol
                .clone()
                .unwrap_or_else(|| pos.token_mint[..8.min(pos.token_mint.len())].to_string());

            match &best_trade {
                None => {
                    best_trade = Some(TradeHighlight {
                        token: token_display.clone(),
                        pnl_sol: pnl,
                        tx_signature: pos.entry_tx_signature.clone(),
                    });
                }
                Some(bt) if pnl > bt.pnl_sol => {
                    best_trade = Some(TradeHighlight {
                        token: token_display.clone(),
                        pnl_sol: pnl,
                        tx_signature: pos.entry_tx_signature.clone(),
                    });
                }
                _ => {}
            }

            match &worst_trade {
                None => {
                    worst_trade = Some(TradeHighlight {
                        token: token_display.clone(),
                        pnl_sol: pnl,
                        tx_signature: pos.entry_tx_signature.clone(),
                    });
                }
                Some(wt) if pnl < wt.pnl_sol => {
                    worst_trade = Some(TradeHighlight {
                        token: token_display,
                        pnl_sol: pnl,
                        tx_signature: pos.entry_tx_signature.clone(),
                    });
                }
                _ => {}
            }

            let venue = "bondingcurve".to_string();
            let venue_metrics = by_venue.entry(venue).or_insert(VenueMetrics {
                trades: 0,
                pnl_sol: 0.0,
                win_rate: 0.0,
            });
            venue_metrics.trades += 1;
            venue_metrics.pnl_sol += pnl;

            let strategy_id = pos.strategy_id.to_string();
            let strategy_metrics = by_strategy.entry(strategy_id).or_insert(StrategyMetrics {
                trades: 0,
                pnl_sol: 0.0,
                win_rate: 0.0,
            });
            strategy_metrics.trades += 1;
            strategy_metrics.pnl_sol += pnl;
        }

        for (_, metrics) in by_venue.iter_mut() {
            if metrics.trades > 0 {
                let winning_count = positions.iter().filter(|p| p.unrealized_pnl > 0.0).count();
                metrics.win_rate = winning_count as f64 / metrics.trades as f64 * 100.0;
            }
        }

        for (strategy_id, metrics) in by_strategy.iter_mut() {
            if metrics.trades > 0 {
                let winning_count = positions
                    .iter()
                    .filter(|p| p.strategy_id.to_string() == *strategy_id && p.unrealized_pnl > 0.0)
                    .count();
                metrics.win_rate = winning_count as f64 / metrics.trades as f64 * 100.0;
            }
        }

        let win_rate = if total_trades > 0 {
            winning_trades as f64 / total_trades as f64 * 100.0
        } else {
            0.0
        };

        let avg_trade_pnl = if total_trades > 0 {
            total_pnl / total_trades as f64
        } else {
            0.0
        };

        Ok(DailyMetrics {
            period: date.format("%Y-%m-%d").to_string(),
            total_trades,
            winning_trades,
            win_rate,
            total_pnl_sol: total_pnl,
            avg_trade_pnl,
            max_drawdown_percent: max_drawdown,
            best_trade,
            worst_trade,
            by_venue,
            by_strategy,
        })
    }

    pub async fn aggregate_and_save_yesterday(&self) -> Result<(), String> {
        let yesterday = Utc::now() - Duration::days(1);
        let metrics = self.aggregate_daily_metrics(yesterday).await?;

        if metrics.total_trades == 0 {
            tracing::info!(
                period = %metrics.period,
                "No trades to aggregate for period"
            );
            return Ok(());
        }

        self.engrams_client
            .save_daily_metrics(&self.wallet_address, &metrics)
            .await
            .map_err(|e| format!("Failed to save daily metrics: {}", e))?;

        tracing::info!(
            period = %metrics.period,
            total_trades = metrics.total_trades,
            win_rate = %format!("{:.1}%", metrics.win_rate),
            total_pnl = %format!("{:.6} SOL", metrics.total_pnl_sol),
            "Daily metrics aggregated and saved"
        );

        Ok(())
    }
}

pub fn calculate_time_until_midnight_utc() -> std::time::Duration {
    let now = Utc::now();
    let tomorrow = (now + Duration::days(1)).date_naive();
    let midnight = tomorrow.and_hms_opt(0, 5, 0).unwrap();
    let target = DateTime::<Utc>::from_naive_utc_and_offset(midnight, Utc);

    let duration = target.signed_duration_since(now);
    std::time::Duration::from_secs(duration.num_seconds().max(0) as u64)
}

pub async fn start_daily_metrics_scheduler(
    position_repo: Arc<PositionRepository>,
    engrams_client: Arc<EngramsClient>,
    wallet_address: String,
) {
    let aggregator = MetricsAggregator::new(position_repo, engrams_client, wallet_address);

    loop {
        let sleep_duration = calculate_time_until_midnight_utc();
        tracing::info!(
            sleep_seconds = sleep_duration.as_secs(),
            "Daily metrics scheduler sleeping until next aggregation"
        );

        tokio::time::sleep(sleep_duration).await;

        if let Err(e) = aggregator.aggregate_and_save_yesterday().await {
            tracing::error!(error = %e, "Failed to aggregate daily metrics");
        }
    }
}
