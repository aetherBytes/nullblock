use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use crate::engrams::{EngramsClient, WatchlistToken};
use crate::error::AppResult;
use crate::events::{ArbEvent, AgentType, EventSource, Significance};
use crate::execution::CurveTransactionBuilder;
use crate::models::{CurveStrategyParams, SignalType, VenueType};
use crate::venues::curves::{GraduationStatus, OnChainFetcher};

const DEFAULT_GRADUATION_THRESHOLD: f64 = 95.0;
const DEFAULT_CHECK_INTERVAL_FAST_MS: u64 = 1000;
const DEFAULT_CHECK_INTERVAL_NORMAL_MS: u64 = 5000;
const RPC_TIMEOUT_SECS: u64 = 10;
const TOKEN_EVICTION_HOURS: i64 = 24;
const MAX_CONSECUTIVE_RPC_FAILURES: u32 = 5;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerConfig {
    pub graduation_threshold: f64,
    pub fast_poll_interval_ms: u64,
    pub normal_poll_interval_ms: u64,
    pub rpc_timeout_secs: u64,
    pub eviction_hours: i64,
}

impl Default for TrackerConfig {
    fn default() -> Self {
        Self {
            graduation_threshold: DEFAULT_GRADUATION_THRESHOLD,
            fast_poll_interval_ms: DEFAULT_CHECK_INTERVAL_FAST_MS,
            normal_poll_interval_ms: DEFAULT_CHECK_INTERVAL_NORMAL_MS,
            rpc_timeout_secs: RPC_TIMEOUT_SECS,
            eviction_hours: TOKEN_EVICTION_HOURS,
        }
    }
}

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
    engrams_client: Option<Arc<EngramsClient>>,
    owner_wallet: Option<String>,
    config: Arc<RwLock<TrackerConfig>>,
    consecutive_rpc_failures: Arc<RwLock<u32>>,
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
            engrams_client: None,
            owner_wallet: None,
            config: Arc::new(RwLock::new(TrackerConfig::default())),
            consecutive_rpc_failures: Arc::new(RwLock::new(0)),
        }
    }

    pub fn with_engrams(mut self, client: Arc<EngramsClient>, wallet: String) -> Self {
        self.engrams_client = Some(client);
        self.owner_wallet = Some(wallet);
        self
    }

    pub fn with_config(mut self, config: TrackerConfig) -> Self {
        self.config = Arc::new(RwLock::new(config));
        self
    }

    pub async fn get_config(&self) -> TrackerConfig {
        self.config.read().await.clone()
    }

    pub async fn update_config(&self, config: TrackerConfig) {
        let mut current = self.config.write().await;
        *current = config;
        tracing::info!(
            "🔧 Tracker config updated: threshold={:.1}%, fast_poll={}ms, normal_poll={}ms",
            current.graduation_threshold,
            current.fast_poll_interval_ms,
            current.normal_poll_interval_ms
        );
    }

    pub async fn restore_from_engrams(&self) -> usize {
        let (client, wallet) = match (&self.engrams_client, &self.owner_wallet) {
            (Some(c), Some(w)) => (c, w),
            _ => {
                tracing::debug!("Engrams not configured for graduation tracker - skipping restore");
                return 0;
            }
        };

        match client.get_watchlist_tokens(wallet).await {
            Ok(tokens) => {
                let mut tracked = self.tracked_tokens.write().await;
                let count = tokens.len();

                for wt in tokens {
                    let token = TrackedToken::new(
                        wt.mint.clone(),
                        wt.name.clone(),
                        wt.symbol.clone(),
                        wt.venue.clone(),
                        Uuid::nil(),
                        wt.last_progress.unwrap_or(0.0),
                    );
                    tracked.insert(wt.mint, token);
                }

                tracing::info!("🔄 Restored {} tracked tokens from engrams", count);
                count
            }
            Err(e) => {
                tracing::warn!("Failed to restore watchlist from engrams: {}", e);
                0
            }
        }
    }

    pub async fn track_with_persistence(
        &self,
        mint: &str,
        name: &str,
        symbol: &str,
        venue: &str,
        strategy_id: Uuid,
        initial_progress: f64,
    ) -> bool {
        self.track(mint, name, symbol, venue, strategy_id, initial_progress).await;

        if let (Some(client), Some(wallet)) = (&self.engrams_client, &self.owner_wallet) {
            let wt = WatchlistToken {
                mint: mint.to_string(),
                name: name.to_string(),
                symbol: symbol.to_string(),
                venue: venue.to_string(),
                tracked_at: Utc::now(),
                notes: None,
                last_progress: Some(initial_progress),
            };

            if let Err(e) = client.save_watchlist_token(wallet, &wt).await {
                tracing::warn!("Failed to persist tracked token to engrams: {}", e);
                return false;
            }
            tracing::info!("💾 Persisted tracked token {} to engrams", symbol);
        }

        true
    }

    pub async fn untrack_with_persistence(&self, mint: &str) -> bool {
        self.untrack(mint).await;

        if let (Some(client), Some(wallet)) = (&self.engrams_client, &self.owner_wallet) {
            if let Err(e) = client.remove_watchlist_token(wallet, mint).await {
                tracing::warn!("Failed to remove tracked token from engrams: {}", e);
                return false;
            }
            tracing::info!("🗑️ Removed tracked token {} from engrams", mint);
        }

        true
    }

    pub async fn clear_all_with_persistence(&self) -> usize {
        let mints: Vec<String> = {
            let tracked = self.tracked_tokens.read().await;
            tracked.keys().cloned().collect()
        };

        let count = mints.len();

        {
            let mut tracked = self.tracked_tokens.write().await;
            tracked.clear();
        }

        if let (Some(client), Some(wallet)) = (&self.engrams_client, &self.owner_wallet) {
            if let Err(e) = client.clear_all_watchlist_tokens(wallet).await {
                tracing::warn!("Failed to clear watchlist from engrams: {}", e);
            } else {
                tracing::info!("🗑️ Cleared all {} tracked tokens from engrams", count);
            }
        }

        count
    }

    pub async fn is_token_tracked(&self, mint: &str) -> bool {
        let tracked = self.tracked_tokens.read().await;
        tracked.contains_key(mint)
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

        let initial_config = self.config.read().await.clone();
        tracing::info!(
            "🚀 Graduation tracker started (threshold={:.1}%, fast={}ms, normal={}ms)",
            initial_config.graduation_threshold,
            initial_config.fast_poll_interval_ms,
            initial_config.normal_poll_interval_ms
        );

        let tracked_tokens = self.tracked_tokens.clone();
        let event_tx = self.event_tx.clone();
        let on_chain_fetcher = self.on_chain_fetcher.clone();
        let is_running = self.is_running.clone();
        let total_checks = self.total_checks.clone();
        let config = self.config.clone();
        let consecutive_rpc_failures = self.consecutive_rpc_failures.clone();

        tokio::spawn(async move {
            let mut last_eviction_check = Utc::now();

            loop {
                if !*is_running.read().await {
                    break;
                }

                let current_config = config.read().await.clone();

                if (Utc::now() - last_eviction_check).num_minutes() >= 60 {
                    Self::evict_old_tokens(&tracked_tokens, current_config.eviction_hours).await;
                    last_eviction_check = Utc::now();
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

                let rpc_failures = *consecutive_rpc_failures.read().await;
                if rpc_failures >= MAX_CONSECUTIVE_RPC_FAILURES {
                    let backoff_secs = (2_u64.pow(rpc_failures.min(6))) * 5;
                    tracing::warn!(
                        "RPC experiencing issues ({} consecutive failures), backing off for {}s",
                        rpc_failures, backoff_secs
                    );
                    tokio::time::sleep(tokio::time::Duration::from_secs(backoff_secs)).await;
                }

                for (mint, _current_state) in mints_to_check {
                    let timeout_duration = std::time::Duration::from_secs(current_config.rpc_timeout_secs);

                    let check_result = tokio::time::timeout(
                        timeout_duration,
                        Self::check_token_progress(&on_chain_fetcher, &mint)
                    ).await;

                    let mut tracked = tracked_tokens.write().await;
                    let mut checks = total_checks.write().await;
                    *checks += 1;

                    if let Some(token) = tracked.get_mut(&mint) {
                        match check_result {
                            Ok(Ok(status)) => {
                                {
                                    let mut failures = consecutive_rpc_failures.write().await;
                                    *failures = 0;
                                }

                                let progress = status.progress();
                                token.update_progress(progress);

                                let new_state = Self::determine_state_from_status(
                                    &status,
                                    progress,
                                    current_config.graduation_threshold,
                                    token,
                                );

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
                            Ok(Err(e)) => {
                                {
                                    let mut failures = consecutive_rpc_failures.write().await;
                                    *failures += 1;
                                }
                                tracing::warn!(
                                    "RPC error checking {} progress: {}",
                                    token.symbol,
                                    e
                                );
                            }
                            Err(_) => {
                                {
                                    let mut failures = consecutive_rpc_failures.write().await;
                                    *failures += 1;
                                }
                                tracing::warn!(
                                    "RPC timeout checking {} progress ({}s limit exceeded)",
                                    token.symbol,
                                    current_config.rpc_timeout_secs
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
                    current_config.fast_poll_interval_ms
                } else {
                    current_config.normal_poll_interval_ms
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

    async fn evict_old_tokens(
        tracked_tokens: &Arc<RwLock<HashMap<String, TrackedToken>>>,
        eviction_hours: i64,
    ) {
        let mut tracked = tracked_tokens.write().await;
        let now = Utc::now();
        let before_count = tracked.len();

        tracked.retain(|_, t| {
            if t.state == TrackedState::Graduated || t.state == TrackedState::Failed {
                let hours_since_terminal = (now - t.state_changed_at).num_hours();
                if hours_since_terminal >= eviction_hours {
                    tracing::debug!(
                        "Evicting {} ({}) - in terminal state {:?} for {}h",
                        t.symbol, t.mint, t.state, hours_since_terminal
                    );
                    false
                } else {
                    true
                }
            } else {
                true
            }
        });

        let evicted = before_count - tracked.len();
        if evicted > 0 {
            tracing::info!(
                "🧹 Evicted {} tokens in terminal state (>{}h old)",
                evicted, eviction_hours
            );
        }
    }

    pub async fn get_rpc_health(&self) -> u32 {
        *self.consecutive_rpc_failures.read().await
    }

    fn determine_state_from_status(
        status: &GraduationStatus,
        progress: f64,
        graduation_threshold: f64,
        token: &mut TrackedToken,
    ) -> TrackedState {
        match status {
            GraduationStatus::Graduated { raydium_pool, .. } => {
                token.raydium_pool = raydium_pool.clone();
                TrackedState::Graduated
            }
            GraduationStatus::Graduating => TrackedState::Graduating,
            GraduationStatus::NearGraduation { .. } => TrackedState::NearGraduation,
            GraduationStatus::PreGraduation { .. } => {
                if progress >= graduation_threshold {
                    TrackedState::NearGraduation
                } else {
                    TrackedState::Monitoring
                }
            }
            GraduationStatus::Failed { .. } => TrackedState::Failed,
        }
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

        if let Err(e) = event_tx.send(event) {
            tracing::warn!("Failed to send {} event: {}", event_type, e);
        }

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
