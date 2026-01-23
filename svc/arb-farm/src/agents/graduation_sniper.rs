use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock, Semaphore};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::events::{ArbEvent, AgentType, EventSource, Significance};
use crate::execution::{CurveTransactionBuilder, CurveSellParams, ExitConfig, JitoClient, MomentumAdaptiveConfig, MomentumData, MomentumStrength, PositionManager};
use crate::execution::risk::RiskConfig;
use crate::helius::HeliusSender;
use crate::models::{Signal, SignalType, VenueType};
use crate::venues::curves::OnChainFetcher;
use crate::wallet::DevWalletSigner;
use crate::wallet::turnkey::SignRequest;

use super::graduation_tracker::GraduationTracker;
use super::strategy_engine::StrategyEngine;

const DEFAULT_JUPITER_API_URL: &str = "https://lite-api.jup.ag/swap/v1";
const MAX_CONCURRENT_SELLS: usize = 5;

const DEFAULT_SELL_DELAY_MS: u64 = 50;  // Reduced from 500ms to beat front-runners
const MAX_SELL_RETRIES: u32 = 3;
const DEFAULT_SLIPPAGE_BPS: u32 = 300;
const DEFAULT_MAX_CONCURRENT_POSITIONS: u32 = 5;
const DEFAULT_TAKE_PROFIT_PERCENT: f64 = 12.0;  // Achievable target (was 30%)
const DEFAULT_STOP_LOSS_PERCENT: f64 = 20.0;   // Matches volatility (was 15%)
const MIN_ENTRY_VELOCITY: f64 = 0.0;           // Minimum velocity for entry (% per min)

const DEFAULT_POST_GRAD_ENTRY_SOL: f64 = 0.15;  // Conservative entry for post-grad quick flip
const DEFAULT_POST_GRAD_TAKE_PROFIT: f64 = 8.0; // 8% quick flip target
const DEFAULT_POST_GRAD_STOP_LOSS: f64 = 5.0;   // 5% tight stop loss
const DEFAULT_POST_GRAD_MAX_DELAY_MS: u64 = 200; // Max 200ms after graduation to enter

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SniperConfig {
    pub sell_delay_ms: u64,
    pub max_sell_retries: u32,
    pub slippage_bps: u32,
    pub max_concurrent_positions: u32,
    pub take_profit_percent: f64,
    pub stop_loss_percent: f64,
    pub auto_sell_on_graduation: bool,
    pub enable_post_graduation_entry: bool,
    pub post_graduation_entry_sol: f64,
    pub post_graduation_take_profit: f64,
    pub post_graduation_stop_loss: f64,
    pub post_graduation_max_delay_ms: u64,
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
            enable_post_graduation_entry: true,
            post_graduation_entry_sol: DEFAULT_POST_GRAD_ENTRY_SOL,
            post_graduation_take_profit: DEFAULT_POST_GRAD_TAKE_PROFIT,
            post_graduation_stop_loss: DEFAULT_POST_GRAD_STOP_LOSS,
            post_graduation_max_delay_ms: DEFAULT_POST_GRAD_MAX_DELAY_MS,
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
    dev_signer: Option<Arc<DevWalletSigner>>,
    helius_sender: Option<Arc<HeliusSender>>,
    position_manager: Option<Arc<PositionManager>>,
    risk_config: Option<Arc<RwLock<RiskConfig>>>,
    in_flight_buys: Arc<RwLock<HashSet<String>>>,
    in_flight_sells: Arc<RwLock<HashSet<String>>>,
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
            dev_signer: None,
            helius_sender: None,
            position_manager: None,
            risk_config: None,
            in_flight_buys: Arc::new(RwLock::new(HashSet::new())),
            in_flight_sells: Arc::new(RwLock::new(HashSet::new())),
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

    pub fn with_transaction_support(
        mut self,
        dev_signer: Arc<DevWalletSigner>,
        helius_sender: Arc<HeliusSender>,
    ) -> Self {
        self.dev_signer = Some(dev_signer);
        self.helius_sender = Some(helius_sender);
        self
    }

    pub fn with_position_manager(mut self, position_manager: Arc<PositionManager>) -> Self {
        self.position_manager = Some(position_manager);
        self
    }

    pub fn with_risk_config(mut self, risk_config: Arc<RwLock<RiskConfig>>) -> Self {
        self.risk_config = Some(risk_config);
        self
    }

    fn calculate_adaptive_slippage(position: &SnipePosition, is_post_graduation: bool) -> u32 {
        const MIN_SLIPPAGE_BPS: u32 = 500;  // 5% floor - post-grad markets can be volatile
        const MAX_SLIPPAGE_BPS: u32 = 2000; // 20% cap - prioritize execution
        const POST_GRAD_SLIPPAGE_BPS: u32 = 1500; // 15% for post-graduation sells (thin liquidity)
        const PROFIT_SACRIFICE_RATIO: f64 = 0.25; // 25% of profits

        if is_post_graduation {
            tracing::info!("🎓 Post-graduation sell: using base slippage {}bps (15%)", POST_GRAD_SLIPPAGE_BPS);
            return POST_GRAD_SLIPPAGE_BPS;
        }

        let pnl_percent = if position.entry_price_sol > 0.0 {
            let current_value_estimate = position.entry_price_sol; // Use entry as baseline since we don't track current
            ((current_value_estimate - position.entry_price_sol) / position.entry_price_sol) * 100.0
        } else {
            0.0
        };

        let calculated_slippage = if pnl_percent > 0.0 {
            let profit_based = (pnl_percent * PROFIT_SACRIFICE_RATIO * 100.0) as u32;
            profit_based.max(MIN_SLIPPAGE_BPS)
        } else {
            MIN_SLIPPAGE_BPS
        };

        tracing::info!(
            "📊 Sniper slippage: entry={:.6} SOL | base={}bps | final={}bps",
            position.entry_price_sol, calculated_slippage, calculated_slippage.min(MAX_SLIPPAGE_BPS)
        );

        calculated_slippage.min(MAX_SLIPPAGE_BPS)
    }

    pub async fn get_config(&self) -> SniperConfig {
        self.config.read().await.clone()
    }

    pub async fn update_config(&self, config: SniperConfig) {
        let mut current = self.config.write().await;
        *current = config;
        tracing::info!(
            "🔧 Sniper config updated: sell_delay={}ms, retries={}, slippage={}bps, max_positions={}, TP={:.1}%, SL={:.1}%, auto_sell={}, post_grad_entry={} ({:.2} SOL, TP={:.1}%, SL={:.1}%)",
            current.sell_delay_ms,
            current.max_sell_retries,
            current.slippage_bps,
            current.max_concurrent_positions,
            current.take_profit_percent,
            current.stop_loss_percent,
            current.auto_sell_on_graduation,
            current.enable_post_graduation_entry,
            current.post_graduation_entry_sol,
            current.post_graduation_take_profit,
            current.post_graduation_stop_loss
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

        let initial_config = self.config.read().await.clone();
        tracing::info!(
            "🔫 Graduation sniper started | auto_sell={} | post_grad_entry={} ({:.2} SOL) | TP={:.1}% | SL={:.1}%",
            initial_config.auto_sell_on_graduation,
            initial_config.enable_post_graduation_entry,
            initial_config.post_graduation_entry_sol,
            initial_config.take_profit_percent,
            initial_config.stop_loss_percent
        );

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
        let dev_signer = self.dev_signer.clone();
        let helius_sender = self.helius_sender.clone();
        let position_manager = self.position_manager.clone();
        let risk_config = self.risk_config.clone();
        let in_flight_buys = self.in_flight_buys.clone();
        let in_flight_sells = self.in_flight_sells.clone();

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
                                    let symbol = event.payload.get("symbol")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("???");
                                    let raydium_pool = event.payload.get("raydium_pool")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_string());

                                    let current_config = config.read().await.clone();

                                    // Check if we have an existing position to SELL
                                    let (has_position, should_sell) = {
                                        let mut positions_lock = positions.write().await;
                                        if let Some(p) = positions_lock.get_mut(mint) {
                                            if p.status == SnipeStatus::Waiting {
                                                p.status = SnipeStatus::Selling;
                                                (true, true)
                                            } else {
                                                tracing::debug!(
                                                    "Position {} already in {:?} state, ignoring duplicate graduation event",
                                                    mint, p.status
                                                );
                                                (true, false)
                                            }
                                        } else {
                                            (false, false)
                                        }
                                    };

                                    if has_position && should_sell {
                                        // EXISTING POSITION: Execute SELL
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

                                        // Check if sell is already in flight to prevent duplicates
                                        let adaptive_slippage = {
                                            let mut in_flight = in_flight_sells.write().await;
                                            if in_flight.contains(mint) {
                                                tracing::warn!(
                                                    "⚠️ Sell already in flight for {} - skipping duplicate",
                                                    mint
                                                );
                                                let mut positions_lock = positions.write().await;
                                                if let Some(p) = positions_lock.get_mut(mint) {
                                                    p.status = SnipeStatus::Waiting;
                                                }
                                                continue;
                                            }
                                            in_flight.insert(mint.to_string());

                                            // Calculate adaptive slippage based on position
                                            let positions_lock = positions.read().await;
                                            if let Some(pos) = positions_lock.get(mint) {
                                                Self::calculate_adaptive_slippage(pos, true) // Post-graduation = true
                                            } else {
                                                current_config.slippage_bps
                                            }
                                        };

                                        tracing::info!(
                                            "🎓 Graduation detected for position {}! Spawning sell task (delay={}ms, slippage={}bps, raydium_pool={:?})...",
                                            mint, current_config.sell_delay_ms, adaptive_slippage, raydium_pool
                                        );

                                        let positions_clone = positions.clone();
                                        let mint_owned = mint.to_string();
                                        let curve_builder_clone = curve_builder.clone();
                                        let jito_client_clone = jito_client.clone();
                                        let event_tx_clone = event_tx.clone();
                                        let wallet_clone = default_wallet.clone();
                                        let semaphore_clone = sell_semaphore.clone();
                                        let jupiter_url_clone = jupiter_api_url.clone();
                                        let in_flight_sells_clone = in_flight_sells.clone();

                                        tokio::spawn(async move {
                                            let _permit = match semaphore_clone.acquire().await {
                                                Ok(p) => p,
                                                Err(_) => {
                                                    tracing::error!("Semaphore closed, cannot execute sell for {}", mint_owned);
                                                    in_flight_sells_clone.write().await.remove(&mint_owned);
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
                                                adaptive_slippage,
                                                current_config.max_sell_retries,
                                                &jupiter_url_clone,
                                                raydium_pool.as_deref(),
                                            ).await;

                                            // Clear in-flight status after completion
                                            in_flight_sells_clone.write().await.remove(&mint_owned);
                                        });
                                    } else if !has_position && current_config.enable_post_graduation_entry {
                                        // NO POSITION + POST-GRAD ENTRY ENABLED: Try quick-flip BUY via Jupiter

                                        // Check max positions limit
                                        let current_position_count = {
                                            let positions_lock = positions.read().await;
                                            positions_lock.values()
                                                .filter(|p| p.status == SnipeStatus::Waiting || p.status == SnipeStatus::Selling)
                                                .count() as u32
                                        };

                                        if current_position_count >= current_config.max_concurrent_positions {
                                            tracing::info!(
                                                "🎓 Skipping post-grad entry for {} - max positions reached ({}/{})",
                                                symbol, current_position_count, current_config.max_concurrent_positions
                                            );
                                            continue;
                                        }

                                        // Check if we have the signer/sender for execution
                                        let (signer, sender) = match (&dev_signer, &helius_sender) {
                                            (Some(s), Some(h)) => (s.clone(), h.clone()),
                                            _ => {
                                                tracing::warn!(
                                                    "🎓❌ Post-grad entry skipped for {} - transaction support not configured",
                                                    symbol
                                                );
                                                continue;
                                            }
                                        };

                                        // Check if buy is already in flight to prevent duplicates
                                        {
                                            let mut in_flight = in_flight_buys.write().await;
                                            if in_flight.contains(mint) {
                                                tracing::warn!(
                                                    "⚠️ Post-grad buy already in flight for {} - skipping duplicate",
                                                    symbol
                                                );
                                                continue;
                                            }
                                            in_flight.insert(mint.to_string());
                                        }

                                        tracing::info!(
                                            "🎓🔫 Post-graduation entry opportunity for {} - executing quick-flip buy via Jupiter",
                                            symbol
                                        );

                                        // Emit event for post-graduation entry signal
                                        let entry_signal_event = ArbEvent::new(
                                            "arb.curve.post_grad_entry_signal",
                                            EventSource::Agent(AgentType::Scanner),
                                            "arb.curve.post_grad_entry_signal",
                                            serde_json::json!({
                                                "mint": mint,
                                                "symbol": symbol,
                                                "entry_sol": current_config.post_graduation_entry_sol,
                                                "take_profit_percent": current_config.post_graduation_take_profit,
                                                "stop_loss_percent": current_config.post_graduation_stop_loss,
                                                "max_delay_ms": current_config.post_graduation_max_delay_ms,
                                                "entry_type": "post_graduation_quick_flip",
                                            }),
                                        );

                                        if let Err(e) = event_tx.send(entry_signal_event) {
                                            tracing::warn!("Failed to send post_grad_entry_signal event: {}", e);
                                        }

                                        // Check wallet balance before attempting buy
                                        // Use global risk_config max_position_sol if available, else fall back to config
                                        let entry_sol = if let Some(ref rc) = risk_config {
                                            rc.read().await.max_position_sol
                                        } else {
                                            current_config.post_graduation_entry_sol
                                        };
                                        const GAS_RESERVE_SOL: f64 = 0.02;
                                        let required_sol = entry_sol + GAS_RESERVE_SOL;

                                        let wallet_balance_sol = match curve_builder.get_wallet_balance(&default_wallet).await {
                                            Ok(lamports) => lamports as f64 / 1_000_000_000.0,
                                            Err(e) => {
                                                tracing::warn!(
                                                    "⚠️ Failed to check balance for post-grad buy {}: {} - skipping",
                                                    symbol, e
                                                );
                                                in_flight_buys.write().await.remove(mint);
                                                continue;
                                            }
                                        };

                                        if wallet_balance_sol < required_sol {
                                            tracing::warn!(
                                                "⚠️ Post-grad buy skipped for {} - insufficient balance ({:.4} SOL < {:.4} SOL needed)",
                                                symbol,
                                                wallet_balance_sol,
                                                required_sol
                                            );
                                            in_flight_buys.write().await.remove(mint);
                                            continue;
                                        }

                                        // Spawn the actual buy execution with retry logic
                                        let mint_owned = mint.to_string();
                                        let symbol_owned = symbol.to_string();
                                        let positions_clone = positions.clone();
                                        let curve_builder_clone = curve_builder.clone();
                                        let event_tx_clone = event_tx.clone();
                                        let wallet_clone = default_wallet.clone();
                                        let jupiter_url_clone = jupiter_api_url.clone();
                                        let slippage_bps = current_config.slippage_bps;
                                        let take_profit = current_config.post_graduation_take_profit;
                                        let stop_loss = current_config.post_graduation_stop_loss;
                                        let position_manager_clone = position_manager.clone();
                                        let in_flight_buys_clone = in_flight_buys.clone();

                                        tokio::spawn(async move {
                                            Self::execute_post_graduation_buy_with_retry(
                                                &positions_clone,
                                                &mint_owned,
                                                &symbol_owned,
                                                &curve_builder_clone,
                                                &signer,
                                                &sender,
                                                &event_tx_clone,
                                                &wallet_clone,
                                                &jupiter_url_clone,
                                                entry_sol,
                                                slippage_bps as u16,
                                                take_profit,
                                                stop_loss,
                                                position_manager_clone.as_ref(),
                                                &in_flight_buys_clone,
                                            ).await;
                                        });
                                    }
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

                                    let (should_proceed, adaptive_slippage) = {
                                        let mut positions_lock = positions_clone.write().await;
                                        if let Some(p) = positions_lock.get_mut(&mint) {
                                            if p.status == SnipeStatus::Waiting {
                                                p.status = SnipeStatus::Selling;
                                                let slippage = Self::calculate_adaptive_slippage(p, true);
                                                (true, slippage)
                                            } else {
                                                (false, 0)
                                            }
                                        } else {
                                            (false, 0)
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
                                            adaptive_slippage,
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
        raydium_pool: Option<&str>,
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
            raydium_pool,
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
        raydium_pool: Option<&str>,
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
                            "🎓 Token {} has graduated, attempting Raydium direct sell (pool={:?})",
                            position.symbol,
                            raydium_pool
                        );

                        match curve_builder.build_raydium_sell(&sell_params).await {
                            Ok(raydium_result) => {
                                tracing::info!(
                                    "📤 Built Raydium sell tx for {}: expected {} SOL, impact {:.2}%",
                                    position.symbol,
                                    raydium_result.expected_sol_out as f64 / 1e9,
                                    raydium_result.price_impact_percent
                                );
                                (
                                    raydium_result.transaction_base64,
                                    raydium_result.expected_sol_out,
                                    raydium_result.price_impact_percent,
                                    raydium_result.route_label,
                                )
                            }
                            Err(raydium_err) => {
                                tracing::warn!(
                                    "⚠️ Raydium sell failed for {}: {}, falling back to Jupiter",
                                    position.symbol, raydium_err
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
                                            &format!("Raydium: {} | Jupiter: {}", raydium_err, jupiter_err),
                                        ).await;
                                        return;
                                    }
                                }
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

        let (should_sell, adaptive_slippage) = {
            let mut positions = self.positions.write().await;
            if let Some(p) = positions.get_mut(mint) {
                if p.status == SnipeStatus::Waiting {
                    p.status = SnipeStatus::Selling;
                    let slippage = Self::calculate_adaptive_slippage(p, true); // Manual = post-grad context
                    (true, slippage)
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
                adaptive_slippage,
                config.max_sell_retries,
                &self.jupiter_api_url,
                None,
            ).await;
        }

        Ok(())
    }

    /// Execute a post-graduation buy with retry logic for TOKEN_NOT_TRADABLE errors
    /// Jupiter may not have indexed the new Raydium pool immediately after graduation
    async fn execute_post_graduation_buy_with_retry(
        positions: &Arc<RwLock<HashMap<String, SnipePosition>>>,
        mint: &str,
        symbol: &str,
        curve_builder: &Arc<CurveTransactionBuilder>,
        dev_signer: &Arc<DevWalletSigner>,
        helius_sender: &Arc<HeliusSender>,
        event_tx: &broadcast::Sender<ArbEvent>,
        wallet: &str,
        jupiter_api_url: &str,
        entry_sol: f64,
        slippage_bps: u16,
        take_profit: f64,
        stop_loss: f64,
        position_manager: Option<&Arc<PositionManager>>,
        in_flight_buys: &Arc<RwLock<HashSet<String>>>,
    ) {
        const MAX_RETRIES: u32 = 5;
        const INITIAL_DELAY_SECS: u64 = 10;

        for attempt in 0..MAX_RETRIES {
            if attempt > 0 {
                let delay_secs = INITIAL_DELAY_SECS * (attempt as u64 + 1);
                tracing::info!(
                    "🔄 Retry {}/{} for post-grad buy {} - waiting {}s for Jupiter indexing...",
                    attempt + 1, MAX_RETRIES, symbol, delay_secs
                );
                tokio::time::sleep(tokio::time::Duration::from_secs(delay_secs)).await;
            }

            let result = Self::execute_post_graduation_buy_inner(
                positions,
                mint,
                symbol,
                curve_builder,
                dev_signer,
                helius_sender,
                event_tx,
                wallet,
                jupiter_api_url,
                entry_sol,
                slippage_bps,
                take_profit,
                stop_loss,
                position_manager,
            ).await;

            match result {
                Ok(()) => {
                    // Success - clear in-flight and return
                    in_flight_buys.write().await.remove(mint);
                    return;
                }
                Err(e) => {
                    let err_str = e.to_string();
                    let is_retryable = err_str.contains("TOKEN_NOT_TRADABLE")
                        || err_str.contains("not tradable")
                        || err_str.contains("No route found")
                        || err_str.contains("error decoding response");

                    if is_retryable && attempt < MAX_RETRIES - 1 {
                        tracing::warn!(
                            "⚠️ Post-grad buy for {} failed with retryable error (attempt {}/{}): {}",
                            symbol, attempt + 1, MAX_RETRIES, err_str
                        );
                        continue;
                    } else {
                        tracing::error!(
                            "❌ Post-grad buy for {} failed permanently after {} attempts: {}",
                            symbol, attempt + 1, err_str
                        );

                        let fail_event = ArbEvent::new(
                            "arb.curve.post_grad_buy_failed",
                            EventSource::Agent(AgentType::Scanner),
                            "arb.curve.post_grad_buy_failed",
                            serde_json::json!({
                                "mint": mint,
                                "symbol": symbol,
                                "error": err_str,
                                "stage": "exhausted_retries",
                                "attempts": attempt + 1,
                            }),
                        );
                        let _ = event_tx.send(fail_event);

                        // Clear in-flight and return
                        in_flight_buys.write().await.remove(mint);
                        return;
                    }
                }
            }
        }

        // Should not reach here, but clear in-flight just in case
        in_flight_buys.write().await.remove(mint);
    }

    /// Inner implementation of post-graduation buy that returns a Result for retry handling
    async fn execute_post_graduation_buy_inner(
        positions: &Arc<RwLock<HashMap<String, SnipePosition>>>,
        mint: &str,
        symbol: &str,
        curve_builder: &Arc<CurveTransactionBuilder>,
        dev_signer: &Arc<DevWalletSigner>,
        helius_sender: &Arc<HeliusSender>,
        event_tx: &broadcast::Sender<ArbEvent>,
        wallet: &str,
        jupiter_api_url: &str,
        entry_sol: f64,
        slippage_bps: u16,
        take_profit: f64,
        stop_loss: f64,
        position_manager: Option<&Arc<PositionManager>>,
    ) -> Result<(), AppError> {
        tracing::info!(
            "🎓🔫 Executing post-graduation BUY for {} via Jupiter ({} SOL, {}bps slippage)",
            symbol,
            entry_sol,
            slippage_bps
        );

        // Convert SOL to lamports
        let sol_amount_lamports = (entry_sol * 1_000_000_000.0) as u64;

        // Build the Jupiter swap transaction (SOL -> Token)
        let buy_result = curve_builder.build_post_graduation_buy(
            mint,
            sol_amount_lamports,
            slippage_bps,
            wallet,
            jupiter_api_url,
        ).await?;

        tracing::info!(
            "📦 Built Jupiter buy tx for {}: expected {} tokens, impact {:.2}%, route: {}",
            symbol,
            buy_result.expected_tokens_out,
            buy_result.price_impact_percent,
            buy_result.route_label
        );

        // Sign the transaction
        let sign_request = SignRequest {
            transaction_base64: buy_result.transaction_base64.clone(),
            estimated_amount_lamports: sol_amount_lamports,
            estimated_profit_lamports: None,
            edge_id: None,
            description: format!(
                "Post-grad buy: {} for {} SOL",
                symbol,
                entry_sol
            ),
        };

        let sign_result = dev_signer.sign_transaction(sign_request).await
            .map_err(|e| AppError::Internal(format!("Signing error: {}", e)))?;

        if !sign_result.success {
            let error = sign_result.error.unwrap_or_else(|| "Unknown signing error".to_string());
            tracing::error!("❌ Signing rejected for post-graduation buy {}: {}", symbol, error);

            let fail_event = ArbEvent::new(
                "arb.curve.post_grad_buy_failed",
                EventSource::Agent(AgentType::Scanner),
                "arb.curve.post_grad_buy_failed",
                serde_json::json!({
                    "mint": mint,
                    "symbol": symbol,
                    "error": error,
                    "stage": "sign_rejected",
                    "entry_sol": entry_sol,
                }),
            );
            let _ = event_tx.send(fail_event);
            return Err(AppError::Internal(format!("Signing rejected: {}", error)));
        }

        let signed_tx = sign_result.signed_transaction_base64
            .ok_or_else(|| AppError::Internal("No signed transaction returned".to_string()))?;

        // Send the transaction
        let signature = helius_sender.send_transaction(&signed_tx, true).await
            .map_err(|e| AppError::Internal(format!("Send error: {}", e)))?;

        let tokens_received = buy_result.expected_tokens_out;
        let edge_id = Uuid::new_v4();

        // Create the position with actual data
        let position = SnipePosition {
            mint: mint.to_string(),
            symbol: symbol.to_string(),
            strategy_id: Uuid::nil(),
            entry_tokens: tokens_received,
            entry_price_sol: entry_sol,
            entry_time: Utc::now(),
            status: SnipeStatus::Waiting,
            sell_attempts: 0,
            last_sell_attempt: None,
            sell_tx_signature: None,
            exit_sol: None,
            pnl_sol: None,
        };

        // FIX #3: Check for duplicate position before inserting
        {
            let mut positions_lock = positions.write().await;
            if positions_lock.contains_key(mint) {
                tracing::warn!(
                    "⚠️ Position already exists for {} - skipping duplicate insertion",
                    symbol
                );
            } else {
                positions_lock.insert(mint.to_string(), position);
            }
        }

        // FIX #1: Register with PositionManager for exit monitoring (TP/SL triggers)
        if let Some(pm) = position_manager {
            // Pump.fun tokens have 6 decimals - convert raw amount to actual tokens
            let actual_tokens = tokens_received as f64 / 1e6;
            let entry_price = if actual_tokens > 0.0 {
                entry_sol / actual_tokens
            } else {
                0.0
            };

            let exit_config = ExitConfig {
                stop_loss_percent: Some(stop_loss),
                take_profit_percent: Some(take_profit),
                trailing_stop_percent: Some(8.0),
                time_limit_minutes: Some(30),
                ..Default::default()
            };

            match pm.open_position(
                edge_id,
                Uuid::nil(),
                mint.to_string(),
                Some(symbol.to_string()),
                entry_sol,
                actual_tokens,
                entry_price,
                exit_config,
                Some(signature.clone()),
                Some("jupiter".to_string()),
                Some("graduation_sniper".to_string()),
            ).await {
                Ok(pos) => {
                    tracing::info!(
                        "🎯 Post-grad position registered with PositionManager: {} (pos_id={})",
                        symbol,
                        pos.id
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        "⚠️ Failed to register post-grad position with PositionManager: {} - {}",
                        symbol,
                        e
                    );
                }
            }
        } else {
            tracing::warn!(
                "⚠️ No PositionManager configured - post-grad position {} will NOT have exit monitoring",
                symbol
            );
        }

        tracing::info!(
            "✅ Post-graduation BUY executed for {} | {} tokens @ {} SOL | TP={:.1}% SL={:.1}% | sig={}",
            symbol,
            tokens_received,
            entry_sol,
            take_profit,
            stop_loss,
            &signature[..16.min(signature.len())]
        );

        // Emit success event
        let success_event = ArbEvent::new(
            "arb.curve.post_grad_buy_success",
            EventSource::Agent(AgentType::Scanner),
            "arb.curve.post_grad_buy_success",
            serde_json::json!({
                "mint": mint,
                "symbol": symbol,
                "tokens_received": tokens_received,
                "entry_sol": entry_sol,
                "take_profit_percent": take_profit,
                "stop_loss_percent": stop_loss,
                "tx_signature": signature,
                "price_impact_percent": buy_result.price_impact_percent,
                "route": buy_result.route_label,
                "signal_source": "graduation_sniper",
                "position_manager_registered": position_manager.is_some(),
            }),
        );

        if let Err(e) = event_tx.send(success_event) {
            tracing::warn!("Failed to send post_grad_buy_success event: {}", e);
        }

        Ok(())
    }
}
