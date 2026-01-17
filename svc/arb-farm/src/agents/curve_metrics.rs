use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::AppResult;
use crate::venues::curves::OnChainFetcher;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedCurveMetrics {
    pub mint: String,
    pub venue: String,
    pub volume_1h: f64,
    pub volume_24h: f64,
    pub volume_velocity: f64,
    pub trade_count_1h: u32,
    pub trade_count_24h: u32,
    pub unique_buyers_1h: u32,
    pub unique_buyers_24h: u32,
    pub holder_count: u32,
    pub holder_growth_1h: i32,
    pub holder_growth_24h: i32,
    pub top_10_concentration: f64,
    pub top_20_concentration: f64,
    pub creator_holdings_percent: f64,
    pub price_momentum_1h: f64,
    pub price_momentum_24h: f64,
    pub buy_sell_ratio_1h: f64,
    pub avg_trade_size_sol: f64,
    pub graduation_progress: f64,
    pub market_cap_sol: f64,
    pub liquidity_depth_sol: f64,
    pub last_updated: DateTime<Utc>,
}

impl DetailedCurveMetrics {
    pub fn new(mint: String, venue: String) -> Self {
        Self {
            mint,
            venue,
            volume_1h: 0.0,
            volume_24h: 0.0,
            volume_velocity: 0.0,
            trade_count_1h: 0,
            trade_count_24h: 0,
            unique_buyers_1h: 0,
            unique_buyers_24h: 0,
            holder_count: 0,
            holder_growth_1h: 0,
            holder_growth_24h: 0,
            top_10_concentration: 0.0,
            top_20_concentration: 0.0,
            creator_holdings_percent: 0.0,
            price_momentum_1h: 0.0,
            price_momentum_24h: 0.0,
            buy_sell_ratio_1h: 1.0,
            avg_trade_size_sol: 0.0,
            graduation_progress: 0.0,
            market_cap_sol: 0.0,
            liquidity_depth_sol: 0.0,
            last_updated: Utc::now(),
        }
    }

    pub fn is_stale(&self, max_age_seconds: i64) -> bool {
        (Utc::now() - self.last_updated).num_seconds() > max_age_seconds
    }

    pub fn volume_acceleration(&self) -> f64 {
        if self.volume_24h == 0.0 {
            return 0.0;
        }
        let avg_hourly = self.volume_24h / 24.0;
        if avg_hourly == 0.0 {
            return 0.0;
        }
        (self.volume_1h - avg_hourly) / avg_hourly
    }

    pub fn holder_quality_score(&self) -> f64 {
        let mut score: f64 = 0.0;

        if self.holder_count >= 100 {
            score += 20.0;
        } else if self.holder_count >= 50 {
            score += 10.0;
        }

        if self.top_10_concentration < 30.0 {
            score += 30.0;
        } else if self.top_10_concentration < 50.0 {
            score += 20.0;
        } else if self.top_10_concentration < 70.0 {
            score += 10.0;
        }

        if self.creator_holdings_percent < 5.0 {
            score += 20.0;
        } else if self.creator_holdings_percent < 10.0 {
            score += 10.0;
        }

        if self.holder_growth_1h > 5 {
            score += 15.0;
        } else if self.holder_growth_1h > 0 {
            score += 10.0;
        }

        if self.unique_buyers_1h > 10 {
            score += 15.0;
        } else if self.unique_buyers_1h > 5 {
            score += 10.0;
        }

        score.min(100.0)
    }

    pub fn activity_score(&self) -> f64 {
        let mut score: f64 = 0.0;

        if self.volume_1h > 10.0 {
            score += 25.0;
        } else if self.volume_1h > 5.0 {
            score += 15.0;
        } else if self.volume_1h > 1.0 {
            score += 10.0;
        }

        let acceleration = self.volume_acceleration();
        if acceleration > 1.0 {
            score += 25.0;
        } else if acceleration > 0.5 {
            score += 15.0;
        } else if acceleration > 0.0 {
            score += 10.0;
        }

        if self.trade_count_1h > 50 {
            score += 25.0;
        } else if self.trade_count_1h > 20 {
            score += 15.0;
        } else if self.trade_count_1h > 10 {
            score += 10.0;
        }

        if self.buy_sell_ratio_1h > 1.5 {
            score += 25.0;
        } else if self.buy_sell_ratio_1h > 1.2 {
            score += 15.0;
        } else if self.buy_sell_ratio_1h > 1.0 {
            score += 10.0;
        }

        score.min(100.0)
    }

    pub fn momentum_score(&self) -> f64 {
        let mut score: f64 = 0.0;

        if self.price_momentum_1h > 20.0 {
            score += 30.0;
        } else if self.price_momentum_1h > 10.0 {
            score += 20.0;
        } else if self.price_momentum_1h > 5.0 {
            score += 15.0;
        } else if self.price_momentum_1h > 0.0 {
            score += 10.0;
        }

        if self.graduation_progress > 90.0 {
            score += 40.0;
        } else if self.graduation_progress > 80.0 {
            score += 30.0;
        } else if self.graduation_progress > 70.0 {
            score += 20.0;
        }

        let velocity_factor = (self.volume_velocity / 10.0).min(30.0);
        score += velocity_factor;

        score.min(100.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSample {
    pub timestamp: DateTime<Utc>,
    pub price_sol: f64,
    pub volume_sol: f64,
    pub holder_count: u32,
    pub trade_count: u32,
}

pub struct CurveMetricsCollector {
    samples: Arc<RwLock<HashMap<String, Vec<MetricsSample>>>>,
    metrics_cache: Arc<RwLock<HashMap<String, DetailedCurveMetrics>>>,
    on_chain_fetcher: Arc<OnChainFetcher>,
    max_samples_per_token: usize,
    sample_interval_seconds: u64,
}

impl CurveMetricsCollector {
    pub fn new(on_chain_fetcher: Arc<OnChainFetcher>) -> Self {
        Self {
            samples: Arc::new(RwLock::new(HashMap::new())),
            metrics_cache: Arc::new(RwLock::new(HashMap::new())),
            on_chain_fetcher,
            max_samples_per_token: 1440,
            sample_interval_seconds: 60,
        }
    }

    #[cfg(test)]
    pub fn new_mock() -> Self {
        Self {
            samples: Arc::new(RwLock::new(HashMap::new())),
            metrics_cache: Arc::new(RwLock::new(HashMap::new())),
            on_chain_fetcher: Arc::new(OnChainFetcher::new_mock()),
            max_samples_per_token: 1440,
            sample_interval_seconds: 60,
        }
    }

    pub fn with_sample_config(mut self, max_samples: usize, interval_seconds: u64) -> Self {
        self.max_samples_per_token = max_samples;
        self.sample_interval_seconds = interval_seconds;
        self
    }

    pub async fn record_sample(
        &self,
        mint: &str,
        price_sol: f64,
        volume_sol: f64,
        holder_count: u32,
        trade_count: u32,
    ) {
        let sample = MetricsSample {
            timestamp: Utc::now(),
            price_sol,
            volume_sol,
            holder_count,
            trade_count,
        };

        let mut samples = self.samples.write().await;
        let token_samples = samples.entry(mint.to_string()).or_insert_with(Vec::new);

        token_samples.push(sample);

        if token_samples.len() > self.max_samples_per_token {
            token_samples.remove(0);
        }
    }

    pub async fn calculate_metrics(&self, mint: &str, venue: &str) -> AppResult<DetailedCurveMetrics> {
        let samples = self.samples.read().await;
        let token_samples = samples.get(mint);

        let mut metrics = DetailedCurveMetrics::new(mint.to_string(), venue.to_string());

        if let Ok(state) = self.on_chain_fetcher.get_bonding_curve_state(mint).await {
            metrics.graduation_progress = state.graduation_progress();
            metrics.market_cap_sol = state.market_cap_sol();
            metrics.liquidity_depth_sol = state.real_sol_reserves as f64 / 1e9;
        }

        if let Some(samples) = token_samples {
            if !samples.is_empty() {
                let now = Utc::now();
                let one_hour_ago = now - Duration::hours(1);
                let twenty_four_hours_ago = now - Duration::hours(24);

                let samples_1h: Vec<_> = samples.iter()
                    .filter(|s| s.timestamp >= one_hour_ago)
                    .collect();

                let samples_24h: Vec<_> = samples.iter()
                    .filter(|s| s.timestamp >= twenty_four_hours_ago)
                    .collect();

                if !samples_1h.is_empty() {
                    metrics.volume_1h = samples_1h.iter().map(|s| s.volume_sol).sum();
                    metrics.trade_count_1h = samples_1h.iter().map(|s| s.trade_count).sum();

                    let first = samples_1h.first().unwrap();
                    let last = samples_1h.last().unwrap();
                    if first.price_sol > 0.0 {
                        metrics.price_momentum_1h = ((last.price_sol - first.price_sol) / first.price_sol) * 100.0;
                    }

                    let first_holders = samples_1h.first().map(|s| s.holder_count).unwrap_or(0);
                    let last_holders = samples_1h.last().map(|s| s.holder_count).unwrap_or(0);
                    metrics.holder_growth_1h = last_holders as i32 - first_holders as i32;
                }

                if !samples_24h.is_empty() {
                    metrics.volume_24h = samples_24h.iter().map(|s| s.volume_sol).sum();
                    metrics.trade_count_24h = samples_24h.iter().map(|s| s.trade_count).sum();

                    let first = samples_24h.first().unwrap();
                    let last = samples_24h.last().unwrap();
                    if first.price_sol > 0.0 {
                        metrics.price_momentum_24h = ((last.price_sol - first.price_sol) / first.price_sol) * 100.0;
                    }

                    let first_holders = samples_24h.first().map(|s| s.holder_count).unwrap_or(0);
                    let last_holders = samples_24h.last().map(|s| s.holder_count).unwrap_or(0);
                    metrics.holder_growth_24h = last_holders as i32 - first_holders as i32;

                    if metrics.trade_count_24h > 0 {
                        metrics.avg_trade_size_sol = metrics.volume_24h / metrics.trade_count_24h as f64;
                    }
                }

                if let Some(latest) = samples.last() {
                    metrics.holder_count = latest.holder_count;
                }

                if samples_1h.len() >= 2 {
                    let first_vol = samples_1h.first().unwrap().volume_sol;
                    let last_vol = samples_1h.last().unwrap().volume_sol;
                    let time_diff = (samples_1h.last().unwrap().timestamp - samples_1h.first().unwrap().timestamp)
                        .num_minutes()
                        .max(1) as f64;
                    metrics.volume_velocity = (last_vol - first_vol) / time_diff * 60.0;
                }
            }
        }

        metrics.last_updated = Utc::now();

        {
            let mut cache = self.metrics_cache.write().await;
            cache.insert(mint.to_string(), metrics.clone());
        }

        Ok(metrics)
    }

    pub async fn get_cached_metrics(&self, mint: &str) -> Option<DetailedCurveMetrics> {
        let cache = self.metrics_cache.read().await;
        cache.get(mint).cloned()
    }

    pub async fn get_or_calculate_metrics(&self, mint: &str, venue: &str, max_age_seconds: i64) -> AppResult<DetailedCurveMetrics> {
        if let Some(cached) = self.get_cached_metrics(mint).await {
            if !cached.is_stale(max_age_seconds) {
                return Ok(cached);
            }
        }

        self.calculate_metrics(mint, venue).await
    }

    pub async fn get_samples(&self, mint: &str) -> Vec<MetricsSample> {
        let samples = self.samples.read().await;
        samples.get(mint).cloned().unwrap_or_default()
    }

    pub async fn clear_old_samples(&self) {
        let cutoff = Utc::now() - Duration::hours(25);
        let mut samples = self.samples.write().await;

        for token_samples in samples.values_mut() {
            token_samples.retain(|s| s.timestamp > cutoff);
        }

        samples.retain(|_, v| !v.is_empty());
    }

    pub async fn list_tracked_tokens(&self) -> Vec<String> {
        let samples = self.samples.read().await;
        samples.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_scoring() {
        let mut metrics = DetailedCurveMetrics::new("test".to_string(), "pump_fun".to_string());

        metrics.holder_count = 150;
        metrics.top_10_concentration = 25.0;
        metrics.creator_holdings_percent = 3.0;
        metrics.holder_growth_1h = 10;
        metrics.unique_buyers_1h = 15;

        let holder_score = metrics.holder_quality_score();
        assert!(holder_score >= 80.0, "Expected high holder quality score");

        metrics.volume_1h = 15.0;
        metrics.volume_24h = 100.0;
        metrics.trade_count_1h = 60;
        metrics.buy_sell_ratio_1h = 1.8;

        let activity_score = metrics.activity_score();
        assert!(activity_score >= 80.0, "Expected high activity score");

        metrics.price_momentum_1h = 25.0;
        metrics.graduation_progress = 92.0;
        metrics.volume_velocity = 15.0;

        let momentum_score = metrics.momentum_score();
        assert!(momentum_score >= 70.0, "Expected high momentum score");
    }

    #[test]
    fn test_volume_acceleration() {
        let mut metrics = DetailedCurveMetrics::new("test".to_string(), "pump_fun".to_string());

        metrics.volume_24h = 240.0;
        metrics.volume_1h = 20.0;

        let accel = metrics.volume_acceleration();
        assert!(accel > 0.5, "Expected positive acceleration when hourly > daily avg");

        metrics.volume_1h = 5.0;
        let accel = metrics.volume_acceleration();
        assert!(accel < 0.0, "Expected negative acceleration when hourly < daily avg");
    }
}
