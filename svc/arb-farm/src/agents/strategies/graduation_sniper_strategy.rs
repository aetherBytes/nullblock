use async_trait::async_trait;
use std::sync::atomic::{AtomicBool, Ordering};
use uuid::Uuid;

use super::{BehavioralStrategy, VenueSnapshot};
use crate::error::AppResult;
use crate::models::{Signal, SignalSignificance, SignalType, VenueType};

pub struct GraduationSniperStrategy {
    name: String,
    is_active: AtomicBool,
    min_progress: f64,
    min_velocity_threshold: f64,
}

impl GraduationSniperStrategy {
    pub fn new() -> Self {
        Self {
            name: "Graduation Sniper".to_string(),
            is_active: AtomicBool::new(true),
            min_progress: 85.0,
            min_velocity_threshold: 0.1,
        }
    }

    pub fn with_min_progress(mut self, min_progress: f64) -> Self {
        self.min_progress = min_progress;
        self
    }

    pub fn with_velocity_threshold(mut self, threshold: f64) -> Self {
        self.min_velocity_threshold = threshold;
        self
    }
}

impl Default for GraduationSniperStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BehavioralStrategy for GraduationSniperStrategy {
    fn strategy_type(&self) -> &str {
        "graduation_snipe"
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
            if token.graduation_progress >= self.min_progress && token.graduation_progress <= 100.0
            {
                let velocity = if token.market_cap_sol > 0.0 {
                    token.volume_24h_sol / token.market_cap_sol
                } else {
                    0.0
                };

                let volume_available = token.volume_24h_sol > 0.0;
                if !volume_available
                    || velocity >= self.min_velocity_threshold
                    || token.graduation_progress >= 95.0
                {
                    let confidence = calculate_snipe_confidence(
                        token.graduation_progress,
                        velocity,
                        token.holder_count,
                    );

                    let significance = if token.graduation_progress >= 95.0 {
                        SignalSignificance::Critical
                    } else if token.graduation_progress >= 90.0 {
                        SignalSignificance::High
                    } else {
                        SignalSignificance::Medium
                    };

                    let signal = Signal {
                        id: Uuid::new_v4(),
                        signal_type: SignalType::CurveGraduation,
                        venue_id: snapshot.venue_id,
                        venue_type: snapshot.venue_type.clone(),
                        token_mint: Some(token.mint.clone()),
                        pool_address: token.bonding_curve_address.clone(),
                        estimated_profit_bps: estimate_snipe_profit_bps(token.graduation_progress),
                        confidence,
                        significance,
                        metadata: serde_json::json!({
                            "token_name": token.name,
                            "token_symbol": token.symbol,
                            "progress_percent": token.graduation_progress,
                            "velocity": velocity,
                            "volume_24h_sol": token.volume_24h_sol,
                            "market_cap_sol": token.market_cap_sol,
                            "holder_count": token.holder_count,
                            "strategy": "graduation_sniper",
                            "is_imminent": token.graduation_progress >= 95.0,
                        }),
                        detected_at: chrono::Utc::now(),
                        expires_at: chrono::Utc::now() + chrono::Duration::minutes(5),
                    };

                    signals.push(signal);
                }
            }
        }

        if !signals.is_empty() {
            tracing::info!(
                "GraduationSniper found {} near-graduation tokens",
                signals.len()
            );
        }

        Ok(signals)
    }

    fn is_active(&self) -> bool {
        self.is_active.load(Ordering::SeqCst)
    }

    async fn set_active(&self, active: bool) {
        self.is_active.store(active, Ordering::SeqCst);
    }
}

fn calculate_snipe_confidence(progress: f64, velocity: f64, holders: u32) -> f64 {
    let progress_factor = ((progress - 85.0) / 15.0).min(1.0).max(0.0);
    let velocity_factor = (velocity * 5.0).min(1.0);

    if holders == 0 {
        (progress_factor * 0.75 + velocity_factor * 0.25).min(1.0)
    } else {
        let holder_factor = (holders as f64 / 50.0).min(1.0);
        (progress_factor * 0.6 + velocity_factor * 0.25 + holder_factor * 0.15).min(1.0)
    }
}

fn estimate_snipe_profit_bps(progress: f64) -> i32 {
    if progress >= 98.0 {
        1500
    } else if progress >= 95.0 {
        1000
    } else if progress >= 90.0 {
        750
    } else {
        500
    }
}
