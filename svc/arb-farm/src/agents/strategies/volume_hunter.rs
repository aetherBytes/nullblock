use async_trait::async_trait;
use std::sync::atomic::{AtomicBool, Ordering};
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::{Signal, SignalType, SignalSignificance, VenueType};
use super::{BehavioralStrategy, VenueSnapshot};

pub struct VolumeHunterStrategy {
    name: String,
    is_active: AtomicBool,
    min_progress: f64,
    max_progress: f64,
    min_volume_sol: f64,
}

impl VolumeHunterStrategy {
    pub fn new() -> Self {
        Self {
            name: "Volume Hunter".to_string(),
            is_active: AtomicBool::new(false),
            min_progress: 30.0,
            max_progress: 85.0,
            min_volume_sol: 1.0,
        }
    }

    pub fn with_progress_range(mut self, min: f64, max: f64) -> Self {
        self.min_progress = min;
        self.max_progress = max;
        self
    }

    pub fn with_min_volume(mut self, min_volume_sol: f64) -> Self {
        self.min_volume_sol = min_volume_sol;
        self
    }
}

impl Default for VolumeHunterStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BehavioralStrategy for VolumeHunterStrategy {
    fn strategy_type(&self) -> &str {
        "volume_hunter"
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn supported_venues(&self) -> Vec<VenueType> {
        vec![VenueType::BondingCurve]
    }

    async fn scan(&self, snapshot: &VenueSnapshot) -> AppResult<Vec<Signal>> {
        let mut signals = Vec::new();

        for token in &snapshot.tokens {
            if token.graduation_progress >= self.min_progress
                && token.graduation_progress <= self.max_progress
                && token.volume_24h_sol >= self.min_volume_sol
            {
                let confidence = calculate_confidence(
                    token.graduation_progress,
                    token.volume_24h_sol,
                    token.holder_count,
                );

                let significance = if token.graduation_progress >= 70.0 && confidence >= 0.7 {
                    SignalSignificance::High
                } else if token.graduation_progress >= 50.0 && confidence >= 0.5 {
                    SignalSignificance::Medium
                } else {
                    SignalSignificance::Low
                };

                let signal = Signal {
                    id: Uuid::new_v4(),
                    signal_type: SignalType::CurveGraduation,
                    venue_id: snapshot.venue_id,
                    venue_type: snapshot.venue_type.clone(),
                    token_mint: Some(token.mint.clone()),
                    pool_address: token.bonding_curve_address.clone(),
                    estimated_profit_bps: estimate_profit_bps(token.graduation_progress),
                    confidence,
                    significance,
                    metadata: serde_json::json!({
                        "token_name": token.name,
                        "token_symbol": token.symbol,
                        "progress_percent": token.graduation_progress,
                        "volume_24h_sol": token.volume_24h_sol,
                        "market_cap_sol": token.market_cap_sol,
                        "holder_count": token.holder_count,
                        "strategy": "volume_hunter",
                    }),
                    detected_at: chrono::Utc::now(),
                    expires_at: chrono::Utc::now() + chrono::Duration::minutes(10),
                };

                signals.push(signal);
            }
        }

        tracing::debug!(
            "VolumeHunter scanned {} tokens, found {} signals",
            snapshot.tokens.len(),
            signals.len()
        );

        Ok(signals)
    }

    fn is_active(&self) -> bool {
        self.is_active.load(Ordering::SeqCst)
    }

    async fn set_active(&self, active: bool) {
        self.is_active.store(active, Ordering::SeqCst);
    }
}

fn calculate_confidence(progress: f64, volume_sol: f64, holders: u32) -> f64 {
    let progress_factor = (progress / 100.0).min(1.0);
    let volume_factor = (volume_sol / 10.0).min(1.0);
    let holder_factor = (holders as f64 / 100.0).min(1.0);

    (progress_factor * 0.5 + volume_factor * 0.3 + holder_factor * 0.2).min(1.0)
}

fn estimate_profit_bps(progress: f64) -> i32 {
    if progress >= 80.0 {
        500
    } else if progress >= 60.0 {
        300
    } else if progress >= 40.0 {
        200
    } else {
        100
    }
}
