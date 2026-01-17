use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use crate::error::AppResult;
use crate::events::{ArbEvent, AgentType, EventSource, Significance};
use crate::execution::CurveTransactionBuilder;
use crate::models::{CurveStrategyParams, SignalType, VenueType};
use crate::venues::curves::{GraduationStatus, OnChainFetcher};

const NEAR_GRADUATION_THRESHOLD: f64 = 95.0;
const CHECK_INTERVAL_FAST_MS: u64 = 1000;
const CHECK_INTERVAL_NORMAL_MS: u64 = 5000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackedState {
    Monitoring,
    NearGraduation,
    Graduating,
    Graduated,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedToken {
    pub mint: String,
    pub name: String,
    pub symbol: String,
    pub venue: String,
    pub strategy_id: Uuid,
    pub state: TrackedState,
    pub progress: f64,
    pub last_progress: f64,
    pub progress_velocity: f64,
    pub started_tracking_at: DateTime<Utc>,
    pub last_checked_at: DateTime<Utc>,
    pub state_changed_at: DateTime<Utc>,
    pub entry_price_sol: Option<f64>,
    pub entry_tokens: Option<u64>,
    pub raydium_pool: Option<String>,
    pub check_count: u64,
}

impl TrackedToken {
    pub fn new(
        mint: String,
        name: String,
        symbol: String,
        venue: String,
        strategy_id: Uuid,
        initial_progress: f64,
    ) -> Self {
        let now = Utc::now();
        Self {
            mint,
            name,
            symbol,
            venue,
            strategy_id,
            state: TrackedState::Monitoring,
            progress: initial_progress,
            last_progress: initial_progress,
            progress_velocity: 0.0,
            started_tracking_at: now,
            last_checked_at: now,
            state_changed_at: now,
            entry_price_sol: None,
            entry_tokens: None,
            raydium_pool: None,
            check_count: 0,
        }
    }

    pub fn update_progress(&mut self, new_progress: f64) {
        let time_delta = (Utc::now() - self.last_checked_at).num_seconds().max(1) as f64;
        let progress_delta = new_progress - self.progress;

        self.progress_velocity = progress_delta / time_delta * 60.0;
        self.last_progress = self.progress;
        self.progress = new_progress;
        self.last_checked_at = Utc::now();
        self.check_count += 1;
    }

    pub fn transition_to(&mut self, new_state: TrackedState) {
        if self.state != new_state {
            self.state = new_state;
            self.state_changed_at = Utc::now();
        }
    }

    pub fn estimated_time_to_graduation_seconds(&self) -> Option<u64> {
        if self.progress_velocity <= 0.0 || self.progress >= 100.0 {
            return None;
        }

        let remaining = 100.0 - self.progress;
        Some((remaining / self.progress_velocity * 60.0) as u64)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraduationTrackerStats {
    pub tokens_tracked: usize,
    pub tokens_near_graduation: usize,
    pub tokens_graduated: usize,
    pub tokens_failed: usize,
    pub total_checks: u64,
    pub is_running: bool,
}

pub struct GraduationTracker {
    tracked_tokens: Arc<RwLock<HashMap<String, TrackedToken>>>,
    event_tx: broadcast::Sender<ArbEvent>,
    on_chain_fetcher: Arc<OnChainFetcher>,
    curve_builder: Arc<CurveTransactionBuilder>,
    is_running: Arc<RwLock<bool>>,
    total_checks: Arc<RwLock<u64>>,
}

impl GraduationTracker {
    pub fn new(
        event_tx: broadcast::Sender<ArbEvent>,
        on_chain_fetcher: Arc<OnChainFetcher>,
        curve_builder: Arc<CurveTransactionBuilder>,
    ) -> Self {
        Self {
            tracked_tokens: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            on_chain_fetcher,
            curve_builder,
            is_running: Arc::new(RwLock::new(false)),
            total_checks: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn track(
        &self,
        mint: &str,
        name: &str,
        symbol: &str,
        venue: &str,
        strategy_id: Uuid,
        initial_progress: f64,
    ) {
        let token = TrackedToken::new(
            mint.to_string(),
            name.to_string(),
            symbol.to_string(),
            venue.to_string(),
            strategy_id,
            initial_progress,
        );

        let mut tracked = self.tracked_tokens.write().await;
        tracked.insert(mint.to_string(), token);

        tracing::info!(
            "🎯 Tracking {} ({}) for graduation - progress: {:.1}%",
            symbol,
            mint,
            initial_progress
        );
    }

    pub async fn untrack(&self, mint: &str) {
        let mut tracked = self.tracked_tokens.write().await;
        if let Some(token) = tracked.remove(mint) {
            tracing::info!(
                "🔕 Stopped tracking {} ({}) - final state: {:?}",
                token.symbol,
                mint,
                token.state
            );
        }
    }

    pub async fn get_tracked(&self, mint: &str) -> Option<TrackedToken> {
        let tracked = self.tracked_tokens.read().await;
        tracked.get(mint).cloned()
    }

    pub async fn list_tracked(&self) -> Vec<TrackedToken> {
        let tracked = self.tracked_tokens.read().await;
        tracked.values().cloned().collect()
    }

    pub async fn get_stats(&self) -> GraduationTrackerStats {
        let tracked = self.tracked_tokens.read().await;
        let is_running = *self.is_running.read().await;
        let total_checks = *self.total_checks.read().await;

        let tokens_near = tracked
            .values()
            .filter(|t| t.state == TrackedState::NearGraduation)
            .count();
        let tokens_graduated = tracked
            .values()
            .filter(|t| t.state == TrackedState::Graduated)
            .count();
        let tokens_failed = tracked
            .values()
            .filter(|t| t.state == TrackedState::Failed)
            .count();

        GraduationTrackerStats {
            tokens_tracked: tracked.len(),
            tokens_near_graduation: tokens_near,
            tokens_graduated,
            tokens_failed,
            total_checks,
            is_running,
        }
    }

    pub async fn start(&self) {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            tracing::warn!("Graduation tracker already running");
            return;
        }
        *is_running = true;
        drop(is_running);

        tracing::info!("🚀 Graduation tracker started");

        let tracked_tokens = self.tracked_tokens.clone();
        let event_tx = self.event_tx.clone();
        let on_chain_fetcher = self.on_chain_fetcher.clone();
        let is_running = self.is_running.clone();
        let total_checks = self.total_checks.clone();

        tokio::spawn(async move {
            loop {
                if !*is_running.read().await {
                    break;
                }

                let mints_to_check: Vec<(String, TrackedState)> = {
                    let tracked = tracked_tokens.read().await;
                    tracked
                        .iter()
                        .filter(|(_, t)| {
                            t.state != TrackedState::Graduated && t.state != TrackedState::Failed
                        })
                        .map(|(k, v)| (k.clone(), v.state))
                        .collect()
                };

                for (mint, current_state) in mints_to_check {
                    let check_result = Self::check_token_progress(
                        &on_chain_fetcher,
                        &mint,
                    ).await;

                    let mut tracked = tracked_tokens.write().await;
                    let mut checks = total_checks.write().await;
                    *checks += 1;

                    if let Some(token) = tracked.get_mut(&mint) {
                        match check_result {
                            Ok(status) => {
                                let progress = status.progress();
                                token.update_progress(progress);

                                let new_state = match &status {
                                    GraduationStatus::Graduated { raydium_pool, .. } => {
                                        token.raydium_pool = raydium_pool.clone();
                                        TrackedState::Graduated
                                    }
                                    GraduationStatus::Graduating => TrackedState::Graduating,
                                    GraduationStatus::NearGraduation { .. } => {
                                        TrackedState::NearGraduation
                                    }
                                    GraduationStatus::PreGraduation { .. } => {
                                        TrackedState::Monitoring
                                    }
                                    GraduationStatus::Failed { .. } => TrackedState::Failed,
                                };

                                if token.state != new_state {
                                    let old_state = token.state;
                                    token.transition_to(new_state);

                                    Self::emit_state_change_event(
                                        &event_tx,
                                        token,
                                        old_state,
                                        new_state,
                                    );
                                }
                            }
                            Err(e) => {
                                tracing::warn!(
                                    "Failed to check {} progress: {}",
                                    token.symbol,
                                    e
                                );
                            }
                        }
                    }
                }

                let has_near_graduation = {
                    let tracked = tracked_tokens.read().await;
                    tracked.values().any(|t| {
                        t.state == TrackedState::NearGraduation
                            || t.state == TrackedState::Graduating
                    })
                };

                let delay = if has_near_graduation {
                    CHECK_INTERVAL_FAST_MS
                } else {
                    CHECK_INTERVAL_NORMAL_MS
                };

                tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
            }

            tracing::info!("🛑 Graduation tracker stopped");
        });
    }

    pub async fn stop(&self) {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        tracing::info!("Graduation tracker stopping...");
    }

    async fn check_token_progress(
        on_chain_fetcher: &OnChainFetcher,
        mint: &str,
    ) -> AppResult<GraduationStatus> {
        on_chain_fetcher.is_token_graduated(mint).await
    }

    fn emit_state_change_event(
        event_tx: &broadcast::Sender<ArbEvent>,
        token: &TrackedToken,
        old_state: TrackedState,
        new_state: TrackedState,
    ) {
        let event_type = match new_state {
            TrackedState::NearGraduation => "arb.curve.graduation_imminent",
            TrackedState::Graduating => "arb.curve.graduating",
            TrackedState::Graduated => "arb.curve.graduated",
            TrackedState::Failed => "arb.curve.graduation_failed",
            _ => return,
        };

        let significance = match new_state {
            TrackedState::NearGraduation => Significance::High,
            TrackedState::Graduating | TrackedState::Graduated => Significance::Critical,
            _ => Significance::Medium,
        };

        let payload = serde_json::json!({
            "mint": token.mint,
            "name": token.name,
            "symbol": token.symbol,
            "venue": token.venue,
            "strategy_id": token.strategy_id,
            "progress": token.progress,
            "progress_velocity": token.progress_velocity,
            "old_state": format!("{:?}", old_state),
            "new_state": format!("{:?}", new_state),
            "raydium_pool": token.raydium_pool,
            "time_tracking_seconds": (Utc::now() - token.started_tracking_at).num_seconds(),
            "significance": format!("{:?}", significance),
        });

        let event = ArbEvent::new(
            event_type,
            EventSource::Agent(AgentType::Scanner),
            event_type,
            payload,
        );

        let _ = event_tx.send(event);

        tracing::info!(
            "📡 {} state change: {:?} -> {:?} (progress: {:.1}%)",
            token.symbol,
            old_state,
            new_state,
            token.progress
        );
    }

    pub async fn track_from_candidate(
        &self,
        mint: &str,
        name: &str,
        symbol: &str,
        venue: &str,
        strategy_id: Uuid,
        params: &CurveStrategyParams,
    ) -> AppResult<bool> {
        let status = self.on_chain_fetcher.is_token_graduated(mint).await?;
        let progress = status.progress();

        if !params.matches_candidate(progress, 0.0, 0.0, 0) {
            return Ok(false);
        }

        self.track(mint, name, symbol, venue, strategy_id, progress)
            .await;

        Ok(true)
    }

    pub async fn set_entry(
        &self,
        mint: &str,
        entry_price_sol: f64,
        entry_tokens: u64,
    ) {
        let mut tracked = self.tracked_tokens.write().await;
        if let Some(token) = tracked.get_mut(mint) {
            token.entry_price_sol = Some(entry_price_sol);
            token.entry_tokens = Some(entry_tokens);
            tracing::info!(
                "📝 Set entry for {}: {} SOL, {} tokens",
                token.symbol,
                entry_price_sol,
                entry_tokens
            );
        }
    }
}
