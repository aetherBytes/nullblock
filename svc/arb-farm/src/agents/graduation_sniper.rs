use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock, Semaphore};
use uuid::Uuid;

use crate::error::AppResult;
use crate::events::{ArbEvent, AgentType, EventSource, Significance};
use crate::execution::{CurveTransactionBuilder, CurveSellParams, JitoClient, MomentumAdaptiveConfig, MomentumData, MomentumStrength};
use crate::models::{Signal, SignalType, VenueType};
use crate::venues::curves::OnChainFetcher;

use super::graduation_tracker::GraduationTracker;
use super::strategy_engine::StrategyEngine;

const DEFAULT_JUPITER_API_URL: &str = "https://lite-api.jup.ag/swap/v1";
const MAX_CONCURRENT_SELLS: usize = 5;

const DEFAULT_SELL_DELAY_MS: u64 = 50;  // Reduced from 500ms to beat front-runners
const MAX_SELL_RETRIES: u32 = 3;
const DEFAULT_SLIPPAGE_BPS: u32 = 300;
const DEFAULT_MAX_CONCURRENT_POSITIONS: u32 = 3;
const DEFAULT_TAKE_PROFIT_PERCENT: f64 = 12.0;  // Achievable target (was 30%)
const DEFAULT_STOP_LOSS_PERCENT: f64 = 20.0;   // Matches volatility (was 15%)
const MIN_ENTRY_VELOCITY: f64 = 0.0;           // Minimum velocity for entry (% per min)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SniperConfig {
    pub sell_delay_ms: u64,
    pub max_sell_retries: u32,
    pub slippage_bps: u32,
    pub max_concurrent_positions: u32,
    pub take_profit_percent: f64,
    pub stop_loss_percent: f64,
    pub auto_sell_on_graduation: bool,
}

impl Default for SniperConfig {
    fn default() -> Self {
        Self {
            sell_delay_ms: DEFAULT_SELL_DELAY_MS,
            max_sell_retries: MAX_SELL_RETRIES,
            slippage_bps: DEFAULT_SLIPPAGE_BPS,
            max_concurrent_positions: DEFAULT_MAX_CONCURRENT_POSITIONS,
            take_profit_percent: DEFAULT_TAKE_PROFIT_PERCENT,
            stop_loss_percent: DEFAULT_STOP_LOSS_PERCENT,
            auto_sell_on_graduation: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnipePosition {
    pub mint: String,
    pub symbol: String,
    pub strategy_id: Uuid,
    pub entry_tokens: u64,
    pub entry_price_sol: f64,
    pub entry_time: DateTime<Utc>,
    pub status: SnipeStatus,
    pub sell_attempts: u32,
    pub last_sell_attempt: Option<DateTime<Utc>>,
    pub sell_tx_signature: Option<String>,
    pub exit_sol: Option<f64>,
    pub pnl_sol: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnipeStatus {
    Waiting,
    Selling,
    Sold,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SniperStats {
    pub positions_waiting: usize,
    pub positions_sold: usize,
    pub positions_failed: usize,
    pub total_pnl_sol: f64,
    pub is_running: bool,
}

pub struct GraduationSniper {
    positions: Arc<RwLock<HashMap<String, SnipePosition>>>,
    graduation_tracker: Arc<GraduationTracker>,
    curve_builder: Arc<CurveTransactionBuilder>,
    jito_client: Arc<JitoClient>,
    on_chain_fetcher: Arc<OnChainFetcher>,
    event_tx: broadcast::Sender<ArbEvent>,
    default_wallet: String,
    is_running: Arc<RwLock<bool>>,
    config: Arc<RwLock<SniperConfig>>,
    strategy_engine: Option<Arc<StrategyEngine>>,
    jupiter_api_url: String,
    sell_semaphore: Arc<Semaphore>,
}

impl GraduationSniper {
    pub fn new(
        graduation_tracker: Arc<GraduationTracker>,
        curve_builder: Arc<CurveTransactionBuilder>,
        jito_client: Arc<JitoClient>,
        on_chain_fetcher: Arc<OnChainFetcher>,
        event_tx: broadcast::Sender<ArbEvent>,
        default_wallet: String,
    ) -> Self {
        Self {
            positions: Arc::new(RwLock::new(HashMap::new())),
            graduation_tracker,
            curve_builder,
            jito_client,
            on_chain_fetcher,
            event_tx,
            default_wallet,
            is_running: Arc::new(RwLock::new(false)),
            config: Arc::new(RwLock::new(SniperConfig::default())),
            strategy_engine: None,
            jupiter_api_url: DEFAULT_JUPITER_API_URL.to_string(),
            sell_semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_SELLS)),
        }
    }

    pub fn with_jupiter_api_url(mut self, url: String) -> Self {
        self.jupiter_api_url = url;
        self
    }

    pub fn with_config(self, config: SniperConfig) -> Self {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            *self.config.write().await = config;
        });
        self
    }

    pub fn with_strategy_engine(mut self, engine: Arc<StrategyEngine>) -> Self {
        self.strategy_engine = Some(engine);
        self
    }

    pub async fn get_config(&self) -> SniperConfig {
        self.config.read().await.clone()
    }

    pub async fn update_config(&self, config: SniperConfig) {
        let mut current = self.config.write().await;
        *current = config;
        tracing::info!(
            "🔧 Sniper config updated: sell_delay={}ms, retries={}, slippage={}bps, max_positions={}, TP={:.1}%, SL={:.1}%, auto_sell={}",
            current.sell_delay_ms,
            current.max_sell_retries,
            current.slippage_bps,
            current.max_concurrent_positions,
            current.take_profit_percent,
            current.stop_loss_percent,
            current.auto_sell_on_graduation
        );
    }

    fn create_graduation_signal(
        mint: &str,
        symbol: &str,
        progress: f64,
        progress_velocity: f64,
        strategy_id: Uuid,
    ) -> Signal {
        // Higher progress = higher confidence but lower profit potential
        let confidence = if progress >= 98.0 {
            0.95
        } else if progress >= 95.0 {
            0.85
        } else if progress >= 90.0 {
            0.75
        } else {
            0.60  // Lower confidence for earlier entries
        };

        // FIXED profit estimation: Account for realistic post-graduation dynamics
        // Post-graduation typically sees 5-15% pump then dump
        // Entry at 95%+ progress with positive velocity has best odds
        let estimated_profit_bps = if progress >= 95.0 && progress_velocity > 0.5 {
            600  // 6% realistic profit for late entry with momentum
        } else if progress >= 90.0 && progress_velocity > 0.0 {
            400  // 4% for medium entry with some momentum
        } else {
            200  // 2% conservative estimate
        };

        Signal::new(
            SignalType::CurveGraduation,
            Uuid::new_v4(),
            VenueType::BondingCurve,
            if progress >= 98.0 { Significance::Critical } else { Significance::High },
        )
        .with_token(mint.to_string())
        .with_profit(estimated_profit_bps, confidence)
        .with_metadata(serde_json::json!({
            "symbol": symbol,
            "progress": progress,
            "progress_velocity": progress_velocity,
            "strategy_id": strategy_id.to_string(),
            "signal_source": "graduation_sniper",
            "entry_type": "pre_graduation",
        }))
    }

    pub async fn add_position(
        &self,
        mint: &str,
        symbol: &str,
        strategy_id: Uuid,
        entry_tokens: u64,
        entry_price_sol: f64,
    ) {
        let position = SnipePosition {
            mint: mint.to_string(),
            symbol: symbol.to_string(),
            strategy_id,
            entry_tokens,
            entry_price_sol,
            entry_time: Utc::now(),
            status: SnipeStatus::Waiting,
            sell_attempts: 0,
            last_sell_attempt: None,
            sell_tx_signature: None,
            exit_sol: None,
            pnl_sol: None,
        };

        let mut positions = self.positions.write().await;
        positions.insert(mint.to_string(), position);

        tracing::info!(
            "🎯 Sniper position added: {} - {} tokens @ {} SOL",
            symbol,
            entry_tokens,
            entry_price_sol
        );
    }

    pub async fn remove_position(&self, mint: &str) -> Option<SnipePosition> {
        let mut positions = self.positions.write().await;
        positions.remove(mint)
    }

    pub async fn get_position(&self, mint: &str) -> Option<SnipePosition> {
        let positions = self.positions.read().await;
        positions.get(mint).cloned()
    }

    pub async fn list_positions(&self) -> Vec<SnipePosition> {
        let positions = self.positions.read().await;
        positions.values().cloned().collect()
    }

    pub async fn get_stats(&self) -> SniperStats {
        let positions = self.positions.read().await;
        let is_running = *self.is_running.read().await;

        let waiting = positions.values().filter(|p| p.status == SnipeStatus::Waiting).count();
        let sold = positions.values().filter(|p| p.status == SnipeStatus::Sold).count();
        let failed = positions.values().filter(|p| p.status == SnipeStatus::Failed).count();
        let total_pnl: f64 = positions
            .values()
            .filter_map(|p| p.pnl_sol)
            .sum();

        SniperStats {
            positions_waiting: waiting,
            positions_sold: sold,
            positions_failed: failed,
            total_pnl_sol: total_pnl,
            is_running,
        }
    }

    pub async fn start(&self) {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            tracing::warn!("Graduation sniper already running");
            return;
        }
        *is_running = true;
        drop(is_running);

        tracing::info!("🔫 Graduation sniper started (listening for graduation_imminent and graduated events)");

        let mut event_rx = self.event_tx.subscribe();
        let positions = self.positions.clone();
        let curve_builder = self.curve_builder.clone();
        let jito_client = self.jito_client.clone();
        let event_tx = self.event_tx.clone();
        let default_wallet = self.default_wallet.clone();
        let is_running = self.is_running.clone();
        let config = self.config.clone();
        let strategy_engine = self.strategy_engine.clone();
        let sell_semaphore = self.sell_semaphore.clone();
        let jupiter_api_url = self.jupiter_api_url.clone();

        tokio::spawn(async move {
            loop {
                if !*is_running.read().await {
                    break;
                }

                tokio::select! {
                    Ok(event) = event_rx.recv() => {
                        match event.event_type.as_str() {
                            "arb.curve.graduation_imminent" => {
                                if let Some(ref engine) = strategy_engine {
                                    let mint = event.payload.get("mint").and_then(|v| v.as_str()).unwrap_or("");
                                    let symbol = event.payload.get("symbol").and_then(|v| v.as_str()).unwrap_or("???");
                                    let progress = event.payload.get("progress").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                    let velocity = event.payload.get("progress_velocity").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                    let strategy_id = event.payload.get("strategy_id")
                                        .and_then(|v| v.as_str())
                                        .and_then(|s| Uuid::parse_str(s).ok())
                                        .unwrap_or(Uuid::nil());

                                    if !mint.is_empty() {
                                        // MOMENTUM ENTRY FILTER: Skip if velocity is negative/flat
                                        if velocity < MIN_ENTRY_VELOCITY {
                                            tracing::info!(
                                                "⏭️ Skipping {} ({:.1}%) - velocity {:.2}%/min below threshold {:.1}%/min",
                                                symbol, progress, velocity, MIN_ENTRY_VELOCITY
                                            );
                                            continue;
                                        }

                                        let has_position = {
                                            let positions = positions.read().await;
                                            positions.contains_key(mint)
                                        };

                                        if !has_position {
                                            tracing::info!(
                                                "🎯 Graduation imminent for {} ({:.1}%, velocity={:.2}%/min) - creating signal",
                                                symbol, progress, velocity
                                            );

                                            let signal = Self::create_graduation_signal(
                                                mint,
                                                symbol,
                                                progress,
                                                velocity,
                                                strategy_id,
                                            );

                                            if let Some(result) = engine.match_signal(&signal).await {
                                                if result.approved {
                                                    tracing::info!(
                                                        "✅ Signal matched strategy {} - edge created for autonomous execution",
                                                        result.strategy_id
                                                    );
                                                } else {
                                                    tracing::debug!(
                                                        "Signal rejected by strategy {}: {:?}",
                                                        result.strategy_id,
                                                        result.reason
                                                    );
                                                }
                                            } else {
                                                tracing::debug!(
                                                    "No strategy matched graduation signal for {}",
                                                    symbol
                                                );
                                            }
                                        } else {
                                            tracing::debug!(
                                                "Already have position for {} - skipping signal",
                                                symbol
                                            );
                                        }
                                    }
                                }
                            }
                            "arb.curve.graduated" => {
                                if let Some(mint) = event.payload.get("mint").and_then(|v| v.as_str()) {
                                    let raydium_pool = event.payload.get("raydium_pool")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_string());

                                    let should_spawn = {
                                        let mut positions_lock = positions.write().await;
                                        if let Some(p) = positions_lock.get_mut(mint) {
                                            if p.status == SnipeStatus::Waiting {
                                                p.status = SnipeStatus::Selling;
                                                true
                                            } else {
                                                tracing::debug!(
                                                    "Position {} already in {:?} state, ignoring duplicate graduation event",
                                                    mint, p.status
                                                );
                                                false
                                            }
                                        } else {
                                            false
                                        }
                                    };

                                    if !should_spawn {
                                        continue;
                                    }

                                    let current_config = config.read().await.clone();

                                    if !current_config.auto_sell_on_graduation {
                                        tracing::info!(
                                            "🎓 Graduation detected for position {} but auto_sell_on_graduation=false, reverting to Waiting...",
                                            mint
                                        );
                                        let mut positions_lock = positions.write().await;
                                        if let Some(p) = positions_lock.get_mut(mint) {
                                            p.status = SnipeStatus::Waiting;
                                        }
                                        continue;
                                    }

                                    tracing::info!(
                                        "🎓 Graduation detected for position {}! Spawning sell task (delay={}ms, slippage={}bps, raydium_pool={:?})...",
                                        mint, current_config.sell_delay_ms, current_config.slippage_bps, raydium_pool
                                    );

                                    let positions_clone = positions.clone();
                                    let mint_owned = mint.to_string();
                                    let curve_builder_clone = curve_builder.clone();
                                    let jito_client_clone = jito_client.clone();
                                    let event_tx_clone = event_tx.clone();
                                    let wallet_clone = default_wallet.clone();
                                    let semaphore_clone = sell_semaphore.clone();
                                    let jupiter_url_clone = jupiter_api_url.clone();

                                    tokio::spawn(async move {
                                        let _permit = match semaphore_clone.acquire().await {
                                            Ok(p) => p,
                                            Err(_) => {
                                                tracing::error!("Semaphore closed, cannot execute sell for {}", mint_owned);
                                                return;
                                            }
                                        };

                                        if current_config.sell_delay_ms > 0 {
                                            tokio::time::sleep(tokio::time::Duration::from_millis(current_config.sell_delay_ms)).await;
                                        }

                                        Self::execute_graduation_sell(
                                            &positions_clone,
                                            &mint_owned,
                                            &curve_builder_clone,
                                            &jito_client_clone,
                                            &event_tx_clone,
                                            &wallet_clone,
                                            current_config.slippage_bps,
                                            current_config.max_sell_retries,
                                            &jupiter_url_clone,
                                            raydium_pool.as_deref(),
                                        ).await;
                                    });
                                }
                            }
                            "auto_execution_succeeded" => {
                                let mint = event.payload.get("mint").and_then(|v| v.as_str()).unwrap_or("");
                                let tokens = event.payload.get("tokens_received").and_then(|v| v.as_u64()).unwrap_or(0);
                                let sol_amount = event.payload.get("sol_amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                let strategy_id = event.payload.get("strategy_id")
                                    .and_then(|v| v.as_str())
                                    .and_then(|s| Uuid::parse_str(s).ok())
                                    .unwrap_or(Uuid::nil());

                                let is_graduation_signal = event.payload.get("signal_source")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s == "graduation_sniper")
                                    .unwrap_or(false);

                                if !mint.is_empty() && tokens > 0 && is_graduation_signal {
                                    let mut positions_lock = positions.write().await;
                                    if !positions_lock.contains_key(mint) {
                                        let entry_price = if tokens > 0 { sol_amount / (tokens as f64) } else { 0.0 };
                                        let position = SnipePosition {
                                            mint: mint.to_string(),
                                            symbol: event.payload.get("symbol").and_then(|v| v.as_str()).unwrap_or("???").to_string(),
                                            strategy_id,
                                            status: SnipeStatus::Waiting,
                                            entry_tokens: tokens,
                                            entry_price_sol: entry_price,
                                            entry_time: Utc::now(),
                                            sell_attempts: 0,
                                            last_sell_attempt: None,
                                            sell_tx_signature: None,
                                            exit_sol: None,
                                            pnl_sol: None,
                                        };
                                        positions_lock.insert(mint.to_string(), position);
                                        tracing::info!(
                                            "🔫 Auto-tracked graduation snipe position: {} ({} tokens @ {} SOL)",
                                            mint, tokens, sol_amount
                                        );
                                    }
                                }
                            }
                            "arb.curve.sell_retry_scheduled" => {
                                let mint = event.payload.get("mint")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                let backoff_ms = event.payload.get("backoff_ms")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(1000);

                                if mint.is_empty() {
                                    continue;
                                }

                                let positions_clone = positions.clone();
                                let curve_builder_clone = curve_builder.clone();
                                let jito_client_clone = jito_client.clone();
                                let event_tx_clone = event_tx.clone();
                                let wallet_clone = default_wallet.clone();
                                let semaphore_clone = sell_semaphore.clone();
                                let jupiter_url_clone = jupiter_api_url.clone();
                                let current_config = config.read().await.clone();

                                tokio::spawn(async move {
                                    tokio::time::sleep(tokio::time::Duration::from_millis(backoff_ms)).await;

                                    let _permit = match semaphore_clone.acquire().await {
                                        Ok(p) => p,
                                        Err(_) => {
                                            tracing::error!("Semaphore closed, cannot retry sell for {}", mint);
                                            return;
                                        }
                                    };

                                    let should_proceed = {
                                        let mut positions_lock = positions_clone.write().await;
                                        if let Some(p) = positions_lock.get_mut(&mint) {
                                            if p.status == SnipeStatus::Waiting {
                                                p.status = SnipeStatus::Selling;
                                                true
                                            } else {
                                                false
                                            }
                                        } else {
                                            false
                                        }
                                    };

                                    if should_proceed {
                                        Self::execute_graduation_sell(
                                            &positions_clone,
                                            &mint,
                                            &curve_builder_clone,
                                            &jito_client_clone,
                                            &event_tx_clone,
                                            &wallet_clone,
                                            current_config.slippage_bps,
                                            current_config.max_sell_retries,
                                            &jupiter_url_clone,
                                            None,
                                        ).await;
                                    }
                                });
                            }
                            _ => {}
                        }
                    }
                    _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)) => {
                        // Periodic check for stuck positions
                    }
                }
            }

            tracing::info!("🛑 Graduation sniper stopped");
        });
    }

    pub async fn stop(&self) {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        tracing::info!("Graduation sniper stopping...");
    }

    async fn execute_graduation_sell(
        positions: &Arc<RwLock<HashMap<String, SnipePosition>>>,
        mint: &str,
        curve_builder: &Arc<CurveTransactionBuilder>,
        jito_client: &Arc<JitoClient>,
        event_tx: &broadcast::Sender<ArbEvent>,
        wallet: &str,
        slippage_bps: u32,
        max_sell_retries: u32,
        jupiter_api_url: &str,
        _raydium_pool: Option<&str>,
    ) {
        Self::execute_graduation_sell_with_momentum(
            positions,
            mint,
            curve_builder,
            jito_client,
            event_tx,
            wallet,
            slippage_bps,
            max_sell_retries,
            jupiter_api_url,
            _raydium_pool,
            None,  // No momentum data
            None,  // No momentum config
        ).await
    }

    async fn execute_graduation_sell_with_momentum(
        positions: &Arc<RwLock<HashMap<String, SnipePosition>>>,
        mint: &str,
        curve_builder: &Arc<CurveTransactionBuilder>,
        jito_client: &Arc<JitoClient>,
        event_tx: &broadcast::Sender<ArbEvent>,
        wallet: &str,
        slippage_bps: u32,
        max_sell_retries: u32,
        jupiter_api_url: &str,
        _raydium_pool: Option<&str>,
        momentum_data: Option<&MomentumData>,
        momentum_config: Option<&MomentumAdaptiveConfig>,
    ) {
        let position = {
            let mut pos = positions.write().await;
            if let Some(p) = pos.get_mut(mint) {
                if p.status != SnipeStatus::Selling {
                    tracing::debug!("Position {} in unexpected state {:?}, expected Selling", mint, p.status);
                }
                p.sell_attempts += 1;
                p.last_sell_attempt = Some(Utc::now());
                p.clone()
            } else {
                tracing::warn!("Position {} not found during execute_graduation_sell", mint);
                return;
            }
        };

        // Calculate exit percentage based on momentum
        let exit_percent = Self::calculate_momentum_exit_percent(momentum_data, momentum_config);
        let token_amount_to_sell = if exit_percent < 100.0 {
            ((position.entry_tokens as f64) * (exit_percent / 100.0)) as u64
        } else {
            position.entry_tokens
        };

        tracing::info!(
            "📤 Graduation sell for {} | exit_percent={:.0}% | tokens={} of {}",
            position.symbol,
            exit_percent,
            token_amount_to_sell,
            position.entry_tokens
        );

        let sell_params = CurveSellParams {
            mint: mint.to_string(),
            token_amount: token_amount_to_sell,
            slippage_bps: slippage_bps as u16,
            user_wallet: wallet.to_string(),
        };

        let (tx_base64, expected_sol_out, price_impact, route_label) =
            match curve_builder.build_pump_fun_sell(&sell_params).await {
                Ok(build_result) => {
                    tracing::info!(
                        "📤 Built pump.fun sell tx for {}: expected {} SOL, impact {:.2}%",
                        position.symbol,
                        build_result.expected_sol_out.unwrap_or(0) as f64 / 1e9,
                        build_result.price_impact_percent
                    );
                    (
                        build_result.transaction_base64,
                        build_result.expected_sol_out.unwrap_or(0),
                        build_result.price_impact_percent,
                        "pump.fun".to_string(),
                    )
                }
                Err(e) => {
                    let err_str = e.to_string();
                    if err_str.contains("graduated") || err_str.contains("is_complete") {
                        tracing::info!(
                            "🎓 Token {} has graduated, falling back to Jupiter for sell",
                            position.symbol
                        );
                        match curve_builder.build_post_graduation_sell(&sell_params, jupiter_api_url).await {
                            Ok(jupiter_result) => {
                                tracing::info!(
                                    "📤 Built Jupiter sell tx for {}: expected {} SOL, impact {:.2}%, route: {}",
                                    position.symbol,
                                    jupiter_result.expected_sol_out as f64 / 1e9,
                                    jupiter_result.price_impact_percent,
                                    jupiter_result.route_label
                                );
                                (
                                    jupiter_result.transaction_base64,
                                    jupiter_result.expected_sol_out,
                                    jupiter_result.price_impact_percent,
                                    jupiter_result.route_label,
                                )
                            }
                            Err(jupiter_err) => {
                                tracing::error!(
                                    "Failed to build Jupiter sell for {}: {}",
                                    position.symbol, jupiter_err
                                );
                                Self::handle_sell_failure(
                                    positions, mint, &position, event_tx,
                                    max_sell_retries, jupiter_api_url,
                                    &format!("Jupiter build error: {}", jupiter_err),
                                ).await;
                                return;
                            }
                        }
                    } else {
                        tracing::error!(
                            "Failed to build pump.fun sell for {}: {}",
                            position.symbol, e
                        );
                        Self::handle_sell_failure(
                            positions, mint, &position, event_tx,
                            max_sell_retries, jupiter_api_url,
                            &format!("Build error: {}", e),
                        ).await;
                        return;
                    }
                }
            };

        let tx_bytes = match base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            &tx_base64,
        ) {
            Ok(bytes) => bytes,
            Err(e) => {
                Self::mark_position_failed(positions, mint, &format!("Decode error: {}", e)).await;
                return;
            }
        };

        match jito_client.send_bundle_fast(&[tx_bytes]).await {
            Ok(bundle_id) => {
                let expected_sol = expected_sol_out as f64 / 1e9;
                let pnl = expected_sol - position.entry_price_sol;

                {
                    let mut pos = positions.write().await;
                    if let Some(p) = pos.get_mut(mint) {
                        p.status = SnipeStatus::Sold;
                        p.sell_tx_signature = Some(bundle_id.clone());
                        p.exit_sol = Some(expected_sol);
                        p.pnl_sol = Some(pnl);
                    }
                }

                let pnl_percent = if position.entry_price_sol > 0.0 {
                    (pnl / position.entry_price_sol) * 100.0
                } else {
                    0.0
                };

                tracing::info!(
                    "✅ Sold {} via {} for {} SOL (PnL: {:.4} SOL / {:.2}%, impact: {:.2}%)",
                    position.symbol,
                    route_label,
                    expected_sol,
                    pnl,
                    pnl_percent,
                    price_impact
                );

                Self::emit_sell_event(
                    event_tx,
                    &position,
                    expected_sol,
                    pnl,
                    &bundle_id,
                    true,
                );
            }
            Err(e) => {
                tracing::error!("Failed to send bundle for {}: {}", mint, e);
                Self::handle_sell_failure(
                    positions, mint, &position, event_tx,
                    max_sell_retries, jupiter_api_url,
                    &format!("Jito error: {}", e),
                ).await;
            }
        }
    }

    async fn handle_sell_failure(
        positions: &Arc<RwLock<HashMap<String, SnipePosition>>>,
        mint: &str,
        position: &SnipePosition,
        event_tx: &broadcast::Sender<ArbEvent>,
        max_sell_retries: u32,
        jupiter_api_url: &str,
        error_msg: &str,
    ) {
        let should_retry = position.sell_attempts < max_sell_retries;

        if should_retry {
            let backoff_ms = 1000 * (2_u64.pow(position.sell_attempts.min(4)));

            {
                let mut pos = positions.write().await;
                if let Some(p) = pos.get_mut(mint) {
                    p.status = SnipeStatus::Waiting;
                }
            }

            tracing::info!(
                "⏳ Will retry sell for {} in {}ms (attempt {}/{}): {}",
                position.symbol,
                backoff_ms,
                position.sell_attempts,
                max_sell_retries,
                error_msg
            );

            let retry_event = ArbEvent::new(
                "arb.curve.sell_retry_scheduled",
                EventSource::Agent(AgentType::Scanner),
                "arb.curve.sell_retry_scheduled",
                serde_json::json!({
                    "mint": mint,
                    "symbol": position.symbol,
                    "attempt": position.sell_attempts,
                    "max_retries": max_sell_retries,
                    "backoff_ms": backoff_ms,
                    "error": error_msg,
                    "jupiter_api_url": jupiter_api_url,
                }),
            );

            if let Err(e) = event_tx.send(retry_event) {
                tracing::warn!("Failed to send sell_retry_scheduled event: {}", e);
            }
        } else {
            Self::mark_position_failed(
                positions,
                mint,
                &format!("Failed after {} retries: {}", max_sell_retries, error_msg),
            ).await;
        }
    }

    async fn mark_position_failed(
        positions: &Arc<RwLock<HashMap<String, SnipePosition>>>,
        mint: &str,
        reason: &str,
    ) {
        let mut pos = positions.write().await;
        if let Some(p) = pos.get_mut(mint) {
            p.status = SnipeStatus::Failed;
            tracing::error!("❌ Position {} failed: {}", p.symbol, reason);
        }
    }

    fn calculate_momentum_exit_percent(
        momentum_data: Option<&MomentumData>,
        config: Option<&MomentumAdaptiveConfig>,
    ) -> f64 {
        let Some(momentum) = momentum_data else {
            return 100.0;  // Full exit if no momentum data
        };
        let Some(cfg) = config else {
            return 100.0;  // Full exit if no config
        };

        let strength = momentum.classify_strength(cfg);
        match strength {
            MomentumStrength::Strong => {
                tracing::info!(
                    "🚀 Strong post-graduation momentum (v={:.2}%/min, score={:.1}) - selling 50%, letting rest run",
                    momentum.velocity,
                    momentum.momentum_score
                );
                50.0
            }
            MomentumStrength::Normal => {
                tracing::info!(
                    "📊 Normal post-graduation momentum (v={:.2}%/min, score={:.1}) - selling 75%",
                    momentum.velocity,
                    momentum.momentum_score
                );
                75.0
            }
            MomentumStrength::Weak | MomentumStrength::Reversing => {
                tracing::info!(
                    "⚠️ Weak/reversing post-graduation momentum (v={:.2}%/min, score={:.1}) - full exit",
                    momentum.velocity,
                    momentum.momentum_score
                );
                100.0
            }
        }
    }

    fn emit_sell_event(
        event_tx: &broadcast::Sender<ArbEvent>,
        position: &SnipePosition,
        exit_sol: f64,
        pnl_sol: f64,
        tx_signature: &str,
        success: bool,
    ) {
        let event_type = if success {
            "arb.curve.snipe_sold"
        } else {
            "arb.curve.snipe_failed"
        };

        let significance = if success && pnl_sol > 0.0 {
            Significance::High
        } else {
            Significance::Medium
        };

        let payload = serde_json::json!({
            "mint": position.mint,
            "symbol": position.symbol,
            "strategy_id": position.strategy_id,
            "entry_tokens": position.entry_tokens,
            "entry_price_sol": position.entry_price_sol,
            "exit_sol": exit_sol,
            "pnl_sol": pnl_sol,
            "pnl_percent": (pnl_sol / position.entry_price_sol) * 100.0,
            "tx_signature": tx_signature,
            "hold_time_seconds": (Utc::now() - position.entry_time).num_seconds(),
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
    }

    pub async fn manual_sell(&self, mint: &str) -> AppResult<()> {
        let config = self.config.read().await.clone();

        let should_sell = {
            let mut positions = self.positions.write().await;
            if let Some(p) = positions.get_mut(mint) {
                if p.status == SnipeStatus::Waiting {
                    p.status = SnipeStatus::Selling;
                    true
                } else {
                    return Err(crate::error::AppError::Validation(format!(
                        "Position {} is in {:?} state, cannot sell",
                        mint, p.status
                    )));
                }
            } else {
                return Err(crate::error::AppError::NotFound(format!(
                    "No position found for {}",
                    mint
                )));
            }
        };

        if should_sell {
            Self::execute_graduation_sell(
                &self.positions,
                mint,
                &self.curve_builder,
                &self.jito_client,
                &self.event_tx,
                &self.default_wallet,
                config.slippage_bps,
                config.max_sell_retries,
                &self.jupiter_api_url,
                None,
            ).await;
        }

        Ok(())
    }
}
