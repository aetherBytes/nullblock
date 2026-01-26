use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use crate::agents::StrategyEngine;
use crate::consensus::{ConsensusConfig, ConsensusEngine, format_edge_context};
use crate::engrams::client::EngramsClient;
use crate::engrams::schemas::{TransactionSummary, TransactionAction, TransactionMetadata, ExecutionError, ExecutionErrorType, ErrorContext};
use crate::error::{AppError, AppResult};
use crate::events::{edge as edge_topics, kol as kol_topics, ArbEvent, AgentType, EventSource};
use crate::execution::{CurveBuyParams, CurveTransactionBuilder, ExitConfig, PositionManager, CopyTradeExecutor};
use crate::execution::risk::RiskConfig;
use crate::helius::HeliusSender;
use crate::models::Signal;
use crate::wallet::DevWalletSigner;
use crate::wallet::turnkey::SignRequest;

const MAX_EXECUTION_RETRIES: u32 = 2;
const EXECUTION_COOLDOWN_MS: u64 = 1000;
const MINT_COOLDOWN_SECONDS: i64 = 300;
const MIN_PROFIT_THRESHOLD_LAMPORTS: u64 = 500_000;
const ESTIMATED_GAS_COST_LAMPORTS: u64 = 250_000;
const EVENT_RETRY_ATTEMPTS: u32 = 3;
const EVENT_RETRY_DELAY_MS: u64 = 50;
const MAX_RECENT_MINTS_SIZE: usize = 10_000;

async fn send_event(tx: &broadcast::Sender<ArbEvent>, event: ArbEvent) {
    send_event_with_retry(tx, event, false).await;
}

async fn send_critical_event(tx: &broadcast::Sender<ArbEvent>, event: ArbEvent) {
    send_event_with_retry(tx, event, true).await;
}

async fn send_event_with_retry(tx: &broadcast::Sender<ArbEvent>, event: ArbEvent, is_critical: bool) {
    let max_attempts = if is_critical { EVENT_RETRY_ATTEMPTS } else { EVENT_RETRY_ATTEMPTS };

    for attempt in 0..max_attempts {
        match tx.send(event.clone()) {
            Ok(_) => return,
            Err(_e) => {
                if attempt < max_attempts - 1 {
                    tokio::time::sleep(std::time::Duration::from_millis(EVENT_RETRY_DELAY_MS)).await;
                } else {
                    if is_critical {
                        tracing::error!(
                            event_type = %event.event_type,
                            topic = %event.topic,
                            attempts = max_attempts,
                            "❌ CRITICAL event broadcast failed after {} attempts",
                            max_attempts
                        );
                    } else {
                        tracing::warn!(
                            event_type = %event.event_type,
                            topic = %event.topic,
                            "⚠️ Event broadcast failed after {} attempts (no receivers or channel full)",
                            max_attempts
                        );
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoExecutionRecord {
    pub edge_id: Uuid,
    pub strategy_id: Uuid,
    pub mint: String,
    pub sol_amount_lamports: u64,
    pub tokens_received: Option<u64>,
    pub signature: Option<String>,
    pub status: AutoExecutionStatus,
    pub attempts: u32,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoExecutionStatus {
    Pending,
    Building,
    Signing,
    Submitting,
    Confirmed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoExecutorStats {
    pub executions_attempted: u64,
    pub executions_succeeded: u64,
    pub executions_failed: u64,
    pub total_sol_deployed: f64,
    pub is_running: bool,
}

pub struct AutonomousExecutor {
    strategy_engine: Arc<StrategyEngine>,
    curve_builder: Arc<CurveTransactionBuilder>,
    dev_signer: Arc<DevWalletSigner>,
    helius_sender: Arc<HeliusSender>,
    position_manager: Arc<PositionManager>,
    risk_config: Arc<RwLock<RiskConfig>>,
    engrams_client: Arc<EngramsClient>,
    consensus_engine: Option<Arc<ConsensusEngine>>,
    consensus_config: Arc<RwLock<ConsensusConfig>>,
    event_tx: broadcast::Sender<ArbEvent>,
    executions: Arc<RwLock<HashMap<Uuid, AutoExecutionRecord>>>,
    recent_mints: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
    stats: Arc<RwLock<AutoExecutorStats>>,
    is_running: Arc<RwLock<bool>>,
    default_wallet: String,
    default_slippage_bps: u16,
    copy_executor: Arc<RwLock<Option<Arc<CopyTradeExecutor>>>>,
}

impl AutonomousExecutor {
    pub fn new(
        strategy_engine: Arc<StrategyEngine>,
        curve_builder: Arc<CurveTransactionBuilder>,
        dev_signer: Arc<DevWalletSigner>,
        helius_sender: Arc<HeliusSender>,
        position_manager: Arc<PositionManager>,
        risk_config: Arc<RwLock<RiskConfig>>,
        engrams_client: Arc<EngramsClient>,
        consensus_engine: Option<Arc<ConsensusEngine>>,
        consensus_config: Arc<RwLock<ConsensusConfig>>,
        event_tx: broadcast::Sender<ArbEvent>,
        default_wallet: String,
    ) -> Self {
        Self {
            strategy_engine,
            curve_builder,
            dev_signer,
            helius_sender,
            position_manager,
            risk_config,
            engrams_client,
            consensus_engine,
            consensus_config,
            event_tx,
            executions: Arc::new(RwLock::new(HashMap::new())),
            recent_mints: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(AutoExecutorStats {
                executions_attempted: 0,
                executions_succeeded: 0,
                executions_failed: 0,
                total_sol_deployed: 0.0,
                is_running: false,
            })),
            is_running: Arc::new(RwLock::new(false)),
            default_wallet,
            default_slippage_bps: 500,
            copy_executor: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn set_copy_executor(&self, executor: Arc<CopyTradeExecutor>) {
        let mut ce = self.copy_executor.write().await;
        *ce = Some(executor);
        tracing::info!("🔗 Autonomous executor: Copy trade executor connected");
    }

    pub async fn start(&self) {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            tracing::warn!("Autonomous executor already running");
            return;
        }
        *is_running = true;
        drop(is_running);

        {
            let mut stats = self.stats.write().await;
            stats.is_running = true;
        }

        tracing::info!("🤖 Autonomous executor started - listening for edge_detected events");

        let mut event_rx = self.event_tx.subscribe();
        let strategy_engine = self.strategy_engine.clone();
        let curve_builder = self.curve_builder.clone();
        let dev_signer = self.dev_signer.clone();
        let helius_sender = self.helius_sender.clone();
        let position_manager = self.position_manager.clone();
        let risk_config = self.risk_config.clone();
        let engrams_client = self.engrams_client.clone();
        let consensus_engine = self.consensus_engine.clone();
        let consensus_config = self.consensus_config.clone();
        let event_tx = self.event_tx.clone();
        let executions = self.executions.clone();
        let recent_mints = self.recent_mints.clone();
        let stats = self.stats.clone();
        let is_running = self.is_running.clone();
        let default_wallet = self.default_wallet.clone();
        let default_slippage_bps = self.default_slippage_bps;
        let copy_executor = self.copy_executor.clone();

        tokio::spawn(async move {
            tracing::info!("🤖 Autonomous executor event loop started, waiting for events...");
            let mut events_received = 0u64;
            let mut last_heartbeat = std::time::Instant::now();

            loop {
                let running = { *is_running.read().await };
                if !running {
                    tracing::info!("🤖 Executor loop: is_running=false, breaking out of loop");
                    break;
                }

                // Heartbeat every 60 seconds
                if last_heartbeat.elapsed() > std::time::Duration::from_secs(60) {
                    tracing::info!(
                        "🤖 Executor heartbeat: events_received={}, is_running=true",
                        events_received
                    );
                    last_heartbeat = std::time::Instant::now();
                }

                tokio::select! {
                    result = event_rx.recv() => {
                        match result {
                            Ok(event) => {
                                events_received += 1;
                                tracing::debug!(
                                    "🤖 Executor received event #{}: topic={}, event_type={}",
                                    events_received,
                                    event.topic,
                                    event.event_type
                                );
                                if event.topic == edge_topics::DETECTED {
                                    if let Err(e) = Self::handle_edge_detected(
                                        &event,
                                        &strategy_engine,
                                        &curve_builder,
                                        &dev_signer,
                                        &helius_sender,
                                        &position_manager,
                                        &risk_config,
                                        &engrams_client,
                                        &consensus_engine,
                                        &consensus_config,
                                        &event_tx,
                                        &executions,
                                        &recent_mints,
                                        &stats,
                                        &default_wallet,
                                        default_slippage_bps,
                                    ).await {
                                        tracing::warn!("Auto-execution failed: {}", e);
                                    }
                                } else if event.topic == kol_topics::TRADE_DETECTED {
                                    if let Err(e) = Self::handle_kol_trade(
                                        &event,
                                        &copy_executor,
                                        &event_tx,
                                        &stats,
                                    ).await {
                                        tracing::warn!("KOL copy trade failed: {}", e);
                                    }
                                }
                            }
                            Err(broadcast::error::RecvError::Lagged(skipped)) => {
                                tracing::warn!(
                                    "🤖 ⚠️ Executor event channel lagged! Skipped {} events. This may cause missed opportunities.",
                                    skipped
                                );
                            }
                            Err(broadcast::error::RecvError::Closed) => {
                                tracing::error!("🤖 ❌ Executor event channel CLOSED! Event bus may have been dropped.");
                                break;
                            }
                        }
                    }
                    _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)) => {}
                }
            }

            tracing::warn!("🤖 Autonomous executor event loop EXITED (events_received={})", events_received);
        });
    }

    pub async fn stop(&self) {
        let mut is_running = self.is_running.write().await;
        *is_running = false;

        let mut stats = self.stats.write().await;
        stats.is_running = false;

        tracing::info!("🤖 Autonomous executor stopping...");
    }

    pub async fn get_stats(&self) -> AutoExecutorStats {
        self.stats.read().await.clone()
    }

    pub async fn list_executions(&self) -> Vec<AutoExecutionRecord> {
        self.executions.read().await.values().cloned().collect()
    }

    pub fn get_recent_mints(&self) -> Arc<RwLock<HashMap<String, DateTime<Utc>>>> {
        self.recent_mints.clone()
    }

    async fn handle_edge_detected(
        event: &ArbEvent,
        strategy_engine: &Arc<StrategyEngine>,
        curve_builder: &Arc<CurveTransactionBuilder>,
        dev_signer: &Arc<DevWalletSigner>,
        helius_sender: &Arc<HeliusSender>,
        position_manager: &Arc<PositionManager>,
        risk_config: &Arc<RwLock<RiskConfig>>,
        engrams_client: &Arc<EngramsClient>,
        consensus_engine: &Option<Arc<ConsensusEngine>>,
        consensus_config: &Arc<RwLock<ConsensusConfig>>,
        event_tx: &broadcast::Sender<ArbEvent>,
        executions: &Arc<RwLock<HashMap<Uuid, AutoExecutionRecord>>>,
        recent_mints: &Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
        stats: &Arc<RwLock<AutoExecutorStats>>,
        default_wallet: &str,
        default_slippage_bps: u16,
    ) -> AppResult<()> {
        let edge_id = event.payload.get("edge_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or_else(|| AppError::Validation("Missing edge_id in event".into()))?;

        let strategy_id = event.payload.get("strategy_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or_else(|| AppError::Validation("Missing strategy_id in event".into()))?;

        // IMPORTANT: Check the CURRENT strategy state, not the stale event payload
        // This allows toggling autonomous mode to take effect immediately for new edges
        let strategy = strategy_engine.get_strategy(strategy_id).await
            .ok_or_else(|| AppError::NotFound(format!("Strategy {} not found", strategy_id)))?;

        let is_autonomous_mode = strategy.execution_mode.to_lowercase() == "autonomous";
        let can_execute = is_autonomous_mode || strategy.can_auto_execute();

        if !can_execute {
            tracing::debug!(
                edge_id = %edge_id,
                strategy_id = %strategy_id,
                execution_mode = %strategy.execution_mode,
                auto_execute_enabled = strategy.risk_params.auto_execute_enabled,
                "Skipping non-autonomous edge (current strategy state)"
            );
            return Ok(());
        }

        if !strategy.is_active {
            tracing::debug!(
                edge_id = %edge_id,
                strategy_id = %strategy_id,
                "Strategy is not active, skipping auto-execution"
            );
            return Ok(());
        }

        let config = consensus_config.read().await;
        let global_consensus_enabled = config.consensus_enabled_for_execution;
        let fail_open = config.fail_open_on_consensus_error;
        drop(config);
        let strategy_requires_consensus = strategy.risk_params.require_consensus;

        if global_consensus_enabled && strategy_requires_consensus {
            if let Some(engine) = consensus_engine {
                let edge_context = format_edge_context(
                    event.payload.get("edge_type").and_then(|v| v.as_str()).unwrap_or("curve_buy"),
                    event.payload.get("venue").and_then(|v| v.as_str()).unwrap_or("pump_fun"),
                    &[
                        event.payload.get("token_mint").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
                        "SOL".to_string(),
                    ],
                    event.payload.get("estimated_profit_lamports").and_then(|v| v.as_i64()).unwrap_or(0),
                    event.payload.get("risk_score").and_then(|v| v.as_i64()).unwrap_or(50) as i32,
                    event.payload.get("route_data").unwrap_or(&serde_json::json!({})),
                );

                match engine.request_consensus(edge_id, &edge_context, None).await {
                    Ok(result) => {
                        // Save consensus decision to engrams
                        let decision = crate::engrams::ConsensusDecision {
                            decision_id: uuid::Uuid::new_v4(),
                            edge_id,
                            strategy_id: Some(strategy_id),
                            approved: result.approved,
                            agreement_score: result.agreement_score,
                            weighted_confidence: result.weighted_confidence,
                            model_votes: result.model_votes.iter().map(|v| v.model.clone()).collect(),
                            reasoning_summary: result.reasoning_summary.clone(),
                            edge_context: edge_context.clone(),
                            total_latency_ms: result.total_latency_ms,
                            created_at: chrono::Utc::now(),
                        };
                        if let Err(e) = engrams_client.save_consensus_decision(default_wallet, &decision).await {
                            tracing::warn!("Failed to save consensus decision engram: {}", e);
                        }

                        if !result.approved {
                            tracing::info!(
                                edge_id = %edge_id,
                                strategy_id = %strategy_id,
                                agreement = result.agreement_score,
                                reasoning = %result.reasoning_summary,
                                "🚫 Edge rejected by consensus - skipping execution"
                            );
                            send_event(&event_tx,ArbEvent::new(
                                "consensus.rejected",
                                EventSource::Agent(AgentType::Executor),
                                "arb.edge.rejected",
                                serde_json::json!({
                                    "edge_id": edge_id,
                                    "strategy_id": strategy_id,
                                    "agreement_score": result.agreement_score,
                                    "reasoning": result.reasoning_summary,
                                }),
                            )).await;
                            return Ok(());
                        }
                        tracing::info!(
                            edge_id = %edge_id,
                            agreement = result.agreement_score,
                            "✅ Edge approved by consensus"
                        );
                    }
                    Err(e) => {
                        if fail_open {
                            tracing::warn!(
                                edge_id = %edge_id,
                                error = %e,
                                "⚠️ Consensus check failed, proceeding anyway (fail-open mode)"
                            );
                        } else {
                            tracing::error!(
                                edge_id = %edge_id,
                                error = %e,
                                "❌ Consensus check failed, aborting execution (fail-closed mode)"
                            );
                            return Err(AppError::ConsensusFailed(e.to_string()));
                        }
                    }
                }
            } else {
                tracing::warn!(
                    edge_id = %edge_id,
                    "Consensus required but engine not configured, {}",
                    if fail_open { "proceeding anyway" } else { "aborting" }
                );
                if !fail_open {
                    return Err(AppError::ConsensusFailed("Consensus engine not configured".to_string()));
                }
            }
        }

        if !dev_signer.is_configured() {
            tracing::warn!("Cannot auto-execute: dev signer not configured");
            return Err(AppError::Internal("Dev signer not configured".into()));
        }

        let route_data = event.payload.get("route_data")
            .cloned()
            .unwrap_or(serde_json::json!({}));

        let mint = event.payload.get("token_mint")
            .or_else(|| event.payload.get("mint"))
            .or_else(|| route_data.get("token_mint"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let mint = match mint {
            Some(m) => m,
            None => {
                tracing::debug!(edge_id = %edge_id, "No token mint in edge, skipping");
                return Ok(());
            }
        };

        // Extract token_symbol from route_data (populated by signal metadata)
        let token_symbol = route_data.get("token_symbol")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        if position_manager.has_open_position_for_mint_and_strategy(&mint, &strategy_id).await {
            tracing::info!(
                edge_id = %edge_id,
                mint = %mint,
                strategy_id = %strategy_id,
                strategy_name = %strategy.name,
                "⏭️ Skipping: already have open position for this mint in this strategy"
            );
            return Ok(());
        }

        {
            let now = Utc::now();
            let cooldown = Duration::seconds(MINT_COOLDOWN_SECONDS);
            let mut mints = recent_mints.write().await;

            // Time-based cleanup: remove expired entries
            mints.retain(|_, last_exec| now.signed_duration_since(*last_exec) < cooldown);

            // Size-based cleanup: if still over limit, evict oldest entries
            if mints.len() >= MAX_RECENT_MINTS_SIZE {
                let mut entries: Vec<_> = mints.iter().map(|(k, v)| (k.clone(), *v)).collect();
                entries.sort_by(|a, b| a.1.cmp(&b.1)); // Sort by timestamp (oldest first)
                let to_remove = entries.len() - MAX_RECENT_MINTS_SIZE / 2; // Remove half
                for (mint_to_remove, _) in entries.into_iter().take(to_remove) {
                    mints.remove(&mint_to_remove);
                }
                tracing::debug!(
                    "🗑️ Evicted {} oldest entries from recent_mints (size: {})",
                    to_remove,
                    mints.len()
                );
            }

            if let Some(last_exec) = mints.get(&mint) {
                let elapsed = now.signed_duration_since(*last_exec);
                tracing::info!(
                    edge_id = %edge_id,
                    mint = %mint,
                    elapsed_secs = elapsed.num_seconds(),
                    cooldown_secs = MINT_COOLDOWN_SECONDS,
                    "⏭️ Skipping: mint on cooldown ({}s remaining)",
                    MINT_COOLDOWN_SECONDS - elapsed.num_seconds()
                );
                return Ok(());
            }
            // Note: cooldown insert moved to AFTER successful transaction to avoid
            // blocking retries on failed transactions (see line ~848)
        }

        let global_max_sol = risk_config.read().await.max_position_sol;
        let strategy_max_sol = strategy.risk_params.max_position_sol;

        // Check if this is a graduation snipe - use aggressive sizing
        let is_graduation_snipe = route_data.get("signal_source")
            .and_then(|v| v.as_str())
            .map(|s| s == "graduation_sniper")
            .unwrap_or(false)
            || strategy.strategy_type == "graduation_snipe";

        // For graduation snipes: use MAX of strategy/global (aggressive)
        // For regular trades: use MIN of strategy/global (conservative)
        let base_sol = if is_graduation_snipe {
            strategy_max_sol.max(global_max_sol)
        } else {
            strategy_max_sol.min(global_max_sol)
        };

        // Velocity-based position scaling for snipes
        let velocity = route_data.get("progress_velocity")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let velocity_multiplier = if is_graduation_snipe && velocity > 0.0 {
            if velocity >= 2.0 {
                1.0  // Full position for strong momentum (>2%/min)
            } else if velocity >= 1.0 {
                0.7  // 70% for good momentum (1-2%/min)
            } else if velocity >= 0.5 {
                0.5  // 50% for moderate momentum (0.5-1%/min)
            } else {
                0.3  // 30% for weak momentum (<0.5%/min)
            }
        } else {
            1.0  // No scaling for non-snipes
        };

        let capped_sol = base_sol * velocity_multiplier;

        tracing::info!(
            edge_id = %edge_id,
            strategy_max = strategy_max_sol,
            global_max = global_max_sol,
            base = base_sol,
            velocity = velocity,
            velocity_multiplier = velocity_multiplier,
            capped = capped_sol,
            is_snipe = is_graduation_snipe,
            "💰 Position size: {} SOL (base={}, v={:.2}%/min, mult={:.0}%)",
            capped_sol, base_sol, velocity, velocity_multiplier * 100.0
        );

        let sol_amount_lamports = (capped_sol * 1_000_000_000.0) as u64;

        // Validate non-zero and minimum SOL amount to prevent wasting network fees
        const MIN_SOL_LAMPORTS: u64 = 1_000_000; // 0.001 SOL
        if sol_amount_lamports < MIN_SOL_LAMPORTS {
            tracing::warn!(
                edge_id = %edge_id,
                mint = %mint,
                sol_amount_lamports = sol_amount_lamports,
                capped_sol = capped_sol,
                base_sol = base_sol,
                velocity_multiplier = velocity_multiplier,
                "⏭️ Skipping: calculated SOL amount {} lamports below minimum {} (base_sol={}, mult={:.2})",
                sol_amount_lamports,
                MIN_SOL_LAMPORTS,
                base_sol,
                velocity_multiplier
            );
            return Ok(());
        }

        let curve_state = match curve_builder.get_curve_state(&mint).await {
            Ok(state) => state,
            Err(e) => {
                tracing::warn!(
                    edge_id = %edge_id,
                    mint = %mint,
                    error = %e,
                    "⏭️ Skipping: failed to fetch curve state"
                );
                return Ok(());
            }
        };

        if curve_state.is_complete {
            tracing::info!(
                edge_id = %edge_id,
                mint = %mint,
                "⏭️ Skipping: token has already graduated"
            );
            return Ok(());
        }

        let max_liquidity_contribution = 0.10;
        let our_contribution = sol_amount_lamports as f64 / curve_state.real_sol_reserves as f64;
        if our_contribution > max_liquidity_contribution {
            tracing::info!(
                edge_id = %edge_id,
                mint = %mint,
                our_sol = sol_amount_lamports as f64 / 1e9,
                pool_sol = curve_state.real_sol_reserves as f64 / 1e9,
                contribution_pct = our_contribution * 100.0,
                max_pct = max_liquidity_contribution * 100.0,
                "⏭️ Skipping: would contribute {:.1}% of liquidity (max {:.0}%)",
                our_contribution * 100.0,
                max_liquidity_contribution * 100.0
            );
            return Ok(());
        }

        let min_pool_sol = 5.0;
        let pool_sol = curve_state.real_sol_reserves as f64 / 1e9;
        if pool_sol < min_pool_sol {
            tracing::info!(
                edge_id = %edge_id,
                mint = %mint,
                pool_sol = pool_sol,
                min_required = min_pool_sol,
                "⏭️ Skipping: pool has only {:.2} SOL (min {:.0} SOL required)",
                pool_sol,
                min_pool_sol
            );
            return Ok(());
        }

        // Entry quality check: calculate price velocity from recent curve state changes
        // We check if the price momentum is favorable for entry
        let current_price = curve_state.virtual_sol_reserves as f64 / curve_state.virtual_token_reserves as f64;

        // Check recent price change from event payload (if available)
        let price_change_1m = event.payload.get("price_change_1m")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let velocity = event.payload.get("velocity")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let progress_velocity = event.payload.get("progress_velocity")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        // ENTRY FILTER 1: Require positive momentum (or at least not declining)
        // Tightened from -1.0 to 0.0 - don't enter during any decline
        if velocity < 0.0 {
            tracing::info!(
                edge_id = %edge_id,
                mint = %mint,
                velocity = velocity,
                "⏭️ Skipping: negative momentum at entry (velocity: {:.2}%/min)",
                velocity
            );
            return Ok(());
        }

        // ENTRY FILTER 2: Anti-FOMO check - don't buy after big spike
        // Strategy-dependent thresholds:
        // - Graduation snipes WANT momentum - tokens often pump 20%+ at graduation
        // - Regular curve_arb: tighter filter to avoid chasing pumps
        let max_recent_pump = if is_graduation_snipe {
            50.0  // Allow up to 50% pump for graduation snipes (momentum is desired)
        } else {
            15.0  // Conservative 15% for regular curve_arb (avoid FOMO entries)
        };
        if price_change_1m > max_recent_pump {
            tracing::info!(
                edge_id = %edge_id,
                mint = %mint,
                price_change_1m = price_change_1m,
                max_allowed = max_recent_pump,
                is_snipe = is_graduation_snipe,
                "⏭️ Skipping: already pumped {:.1}% in last minute (max {:.0}% for {})",
                price_change_1m,
                max_recent_pump,
                if is_graduation_snipe { "snipe" } else { "curve_arb" }
            );
            return Ok(());
        }

        // ENTRY FILTER 3: Require positive progress velocity (graduation accelerating)
        let min_progress_velocity = 0.5;  // % per minute
        if progress_velocity < min_progress_velocity && progress_velocity != 0.0 {
            tracing::info!(
                edge_id = %edge_id,
                mint = %mint,
                progress_velocity = progress_velocity,
                min_required = min_progress_velocity,
                "⏭️ Skipping: progress velocity {:.2}%/min below threshold {:.1}%/min",
                progress_velocity,
                min_progress_velocity
            );
            return Ok(());
        }

        tracing::info!(
            edge_id = %edge_id,
            strategy_id = %strategy_id,
            mint = %mint,
            sol_amount = sol_amount_lamports as f64 / 1e9,
            pool_sol = pool_sol,
            contribution_pct = our_contribution * 100.0,
            "🚀 Auto-executing curve buy"
        );

        let record = AutoExecutionRecord {
            edge_id,
            strategy_id,
            mint: mint.clone(),
            sol_amount_lamports,
            tokens_received: None,
            signature: None,
            status: AutoExecutionStatus::Pending,
            attempts: 0,
            started_at: Utc::now(),
            completed_at: None,
            error: None,
        };

        {
            let mut execs = executions.write().await;
            execs.insert(edge_id, record.clone());
        }

        {
            let mut s = stats.write().await;
            s.executions_attempted += 1;
        }

        send_event(&event_tx,ArbEvent::new(
            "auto_execution_started",
            EventSource::Agent(AgentType::Executor),
            edge_topics::EXECUTING,
            serde_json::json!({
                "edge_id": edge_id,
                "strategy_id": strategy_id,
                "mint": mint,
                "sol_amount": sol_amount_lamports as f64 / 1e9,
                "mode": "autonomous",
            }),
        )).await;

        let result = Self::execute_curve_buy(
            &mint,
            sol_amount_lamports,
            default_slippage_bps,
            default_wallet,
            curve_builder,
            dev_signer,
            helius_sender,
        ).await;

        match result {
            Ok((signature, tokens_out)) => {
                tracing::info!(
                    edge_id = %edge_id,
                    signature = %signature,
                    tokens = tokens_out.unwrap_or(0),
                    "✅ Auto-execution succeeded"
                );

                {
                    let mut execs = executions.write().await;
                    if let Some(rec) = execs.get_mut(&edge_id) {
                        rec.status = AutoExecutionStatus::Confirmed;
                        rec.signature = Some(signature.clone());
                        rec.tokens_received = tokens_out;
                        rec.completed_at = Some(Utc::now());
                    }
                }

                {
                    let mut s = stats.write().await;
                    s.executions_succeeded += 1;
                    s.total_sol_deployed += sol_amount_lamports as f64 / 1e9;
                }

                let signal_source = route_data.get("signal_source")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let symbol = route_data.get("symbol")
                    .and_then(|v| v.as_str())
                    .unwrap_or("???");

                send_critical_event(&event_tx, ArbEvent::new(
                    "auto_execution_succeeded",
                    EventSource::Agent(AgentType::Executor),
                    edge_topics::EXECUTED,
                    serde_json::json!({
                        "edge_id": edge_id,
                        "strategy_id": strategy_id,
                        "mint": mint,
                        "symbol": symbol,
                        "signature": signature,
                        "tokens_received": tokens_out,
                        "sol_amount": sol_amount_lamports as f64 / 1e9,
                        "sol_spent": sol_amount_lamports as f64 / 1e9,
                        "signal_source": signal_source,
                        "significance": "critical",
                    }),
                )).await;

                {
                    let mut mints = recent_mints.write().await;
                    mints.insert(mint.clone(), Utc::now());
                }

                let tokens_received = tokens_out.unwrap_or(0);
                if tokens_received > 0 {
                    let entry_price = sol_amount_lamports as f64 / tokens_received as f64;
                    // DEFENSIVE MODE (default): 15% TP, strong momentum can run
                    // All strategies now use defensive config for capital preservation
                    let exit_config = ExitConfig::for_defensive();
                    tracing::info!(
                        edge_id = %edge_id,
                        strategy_type = %strategy.strategy_type,
                        "🛡️ Using DEFENSIVE exit config (15% TP, strong momentum extends)"
                    );

                    let venue = route_data.get("venue")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .or_else(|| Some("pump_fun".to_string()));
                    let signal_src = if signal_source.is_empty() {
                        None
                    } else {
                        Some(signal_source.to_string())
                    };
                    let is_snipe = signal_source == "graduation_sniper";
                    let snipe_indicator = if is_snipe { "🔫 " } else { "" };

                    if let Err(e) = position_manager.open_position(
                        edge_id,
                        strategy_id,
                        mint.clone(),
                        token_symbol.clone(),
                        sol_amount_lamports as f64 / 1e9,
                        tokens_received as f64,
                        entry_price,
                        exit_config,
                        Some(signature.clone()),
                        venue,
                        signal_src,
                    ).await {
                        tracing::warn!(
                            edge_id = %edge_id,
                            error = %e,
                            "Failed to create position tracking (buy succeeded)"
                        );
                    } else {
                        tracing::info!(
                            edge_id = %edge_id,
                            mint = %mint,
                            symbol = ?token_symbol,
                            tokens = tokens_received,
                            is_snipe = is_snipe,
                            "{}📊 Position opened for tracking", snipe_indicator
                        );
                    }

                    // Save buy transaction summary to engrams
                    let tx_summary = TransactionSummary {
                        tx_signature: signature.clone(),
                        action: TransactionAction::Buy,
                        token_mint: mint.clone(),
                        token_symbol: token_symbol.clone(),
                        venue: "pump_fun".to_string(),
                        entry_sol: sol_amount_lamports as f64 / 1e9,
                        exit_sol: None,
                        pnl_sol: None,
                        pnl_percent: None,
                        slippage_bps: default_slippage_bps as i32,
                        execution_time_ms: 0,
                        strategy_id: Some(strategy_id),
                        timestamp: Utc::now(),
                        metadata: TransactionMetadata {
                            graduation_progress: None,
                            holder_count: None,
                            volume_24h_sol: None,
                            market_cap_sol: None,
                            bonding_curve_percent: None,
                        },
                    };

                    if let Err(e) = engrams_client.save_transaction_summary(default_wallet, &tx_summary).await {
                        tracing::warn!("Failed to save buy transaction summary engram: {}", e);
                    } else {
                        tracing::debug!("📝 Saved buy transaction summary engram for {}", &signature[..12.min(signature.len())]);
                    }
                }

                Ok(())
            }
            Err(e) => {
                tracing::error!(
                    edge_id = %edge_id,
                    error = %e,
                    "❌ Auto-execution failed"
                );

                {
                    let mut execs = executions.write().await;
                    if let Some(rec) = execs.get_mut(&edge_id) {
                        rec.status = AutoExecutionStatus::Failed;
                        rec.error = Some(e.to_string());
                        rec.completed_at = Some(Utc::now());
                    }
                }

                {
                    let mut s = stats.write().await;
                    s.executions_failed += 1;
                }

                send_critical_event(&event_tx, ArbEvent::new(
                    "auto_execution_failed",
                    EventSource::Agent(AgentType::Executor),
                    edge_topics::FAILED,
                    serde_json::json!({
                        "edge_id": edge_id,
                        "strategy_id": strategy_id,
                        "mint": mint,
                        "error": e.to_string(),
                    }),
                )).await;

                // Save execution error to engrams
                let error_str = e.to_string();
                let error_type = if error_str.contains("slippage") {
                    ExecutionErrorType::SlippageExceeded
                } else if error_str.contains("timeout") || error_str.contains("timed out") {
                    ExecutionErrorType::RpcTimeout
                } else if error_str.contains("insufficient") || error_str.contains("balance") {
                    ExecutionErrorType::InsufficientFunds
                } else if error_str.contains("simulation") {
                    ExecutionErrorType::SimulationFailed
                } else if error_str.contains("signing") || error_str.contains("sign") {
                    ExecutionErrorType::SigningFailed
                } else if error_str.contains("rate limit") {
                    ExecutionErrorType::RateLimited
                } else if error_str.contains("network") || error_str.contains("connection") {
                    ExecutionErrorType::NetworkError
                } else {
                    ExecutionErrorType::TxFailed
                };

                let exec_error = ExecutionError {
                    error_type,
                    message: error_str,
                    context: ErrorContext {
                        action: Some("buy".to_string()),
                        token_mint: Some(mint.clone()),
                        attempted_amount_sol: Some(sol_amount_lamports as f64 / 1e9),
                        venue: Some("pump_fun".to_string()),
                        strategy_id: Some(strategy_id),
                        edge_id: Some(edge_id),
                    },
                    stack_trace: None,
                    recoverable: true,
                    timestamp: Utc::now(),
                };

                if let Err(save_err) = engrams_client.save_execution_error(default_wallet, &exec_error).await {
                    tracing::warn!("Failed to save execution error engram: {}", save_err);
                } else {
                    tracing::debug!("📝 Saved execution error engram for failed buy of {}", &mint[..12.min(mint.len())]);
                }

                Err(e)
            }
        }
    }

    async fn execute_curve_buy(
        mint: &str,
        sol_amount_lamports: u64,
        slippage_bps: u16,
        user_wallet: &str,
        curve_builder: &Arc<CurveTransactionBuilder>,
        dev_signer: &Arc<DevWalletSigner>,
        helius_sender: &Arc<HeliusSender>,
    ) -> AppResult<(String, Option<u64>)> {
        let params = CurveBuyParams {
            mint: mint.to_string(),
            sol_amount_lamports,
            slippage_bps,
            user_wallet: user_wallet.to_string(),
        };

        tracing::debug!(mint = %mint, "Building curve buy transaction");

        let build_result = curve_builder.build_pump_fun_buy(&params).await?;

        tracing::debug!(
            mint = %mint,
            expected_tokens = build_result.expected_tokens_out,
            price_impact = build_result.price_impact_percent,
            "Transaction built, signing..."
        );

        let sign_request = SignRequest {
            transaction_base64: build_result.transaction_base64.clone(),
            estimated_amount_lamports: sol_amount_lamports,
            estimated_profit_lamports: None,
            edge_id: None,
            description: format!("Auto curve buy: {} for {} SOL", mint, sol_amount_lamports as f64 / 1e9),
        };

        let sign_result = dev_signer.sign_transaction(sign_request).await?;

        if !sign_result.success {
            return Err(AppError::Internal(format!(
                "Signing failed: {}",
                sign_result.error.unwrap_or_else(|| "Unknown error".to_string())
            )));
        }

        let signed_tx = sign_result.signed_transaction_base64
            .ok_or_else(|| AppError::Internal("No signed transaction returned".into()))?;

        tracing::debug!(mint = %mint, "Transaction signed, submitting...");

        let signature = helius_sender.send_transaction(&signed_tx, true).await?;

        Ok((signature, build_result.expected_tokens_out))
    }

    async fn handle_kol_trade(
        event: &ArbEvent,
        copy_executor: &Arc<RwLock<Option<Arc<CopyTradeExecutor>>>>,
        event_tx: &broadcast::Sender<ArbEvent>,
        stats: &Arc<RwLock<AutoExecutorStats>>,
    ) -> AppResult<()> {
        let copy_exec_guard = copy_executor.read().await;
        let copy_exec = match copy_exec_guard.as_ref() {
            Some(exec) => exec.clone(),
            None => {
                tracing::warn!(
                    "⚠️ KOL trade detected but copy executor not initialized! \
                    Copy trading will not work. Set copy_executor on AutonomousExecutor to enable."
                );
                return Ok(());
            }
        };
        drop(copy_exec_guard);

        let kol_id = event.payload.get("kol_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Validation("Missing kol_id in event".into()))?;

        let token_mint = event.payload.get("token_mint")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Validation("Missing token_mint in event".into()))?;

        let trade_type = event.payload.get("trade_type")
            .and_then(|v| v.as_str())
            .unwrap_or("buy");

        tracing::info!(
            kol_id = %kol_id,
            token_mint = %token_mint,
            trade_type = %trade_type,
            "🔗 Processing KOL trade for copy execution"
        );

        let signal = Signal {
            id: Uuid::new_v4(),
            signal_type: crate::models::SignalType::KolTrade,
            venue_id: Uuid::nil(),
            venue_type: crate::models::VenueType::BondingCurve,
            token_mint: Some(token_mint.to_string()),
            pool_address: None,
            estimated_profit_bps: 100,
            confidence: event.payload.get("kol_trust_score")
                .and_then(|v| v.as_f64())
                .map(|s| s / 100.0)
                .unwrap_or(0.7),
            significance: crate::events::Significance::High,
            metadata: event.payload.clone(),
            detected_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + chrono::Duration::minutes(2),
        };

        {
            let mut s = stats.write().await;
            s.executions_attempted += 1;
        }

        match copy_exec.execute_copy(&signal).await {
            Ok(result) => {
                tracing::info!(
                    kol_id = %kol_id,
                    token_mint = %token_mint,
                    copy_trade_id = %result.copy_trade_id,
                    tx_signature = ?result.tx_signature,
                    latency_ms = result.latency_ms,
                    "✅ KOL copy trade executed successfully"
                );

                {
                    let mut s = stats.write().await;
                    s.executions_succeeded += 1;
                    s.total_sol_deployed += result.sol_amount;
                }

                send_critical_event(&event_tx, ArbEvent::new(
                    "kol_trade_copied",
                    EventSource::Agent(AgentType::Executor),
                    kol_topics::TRADE_COPIED,
                    serde_json::json!({
                        "kol_id": kol_id,
                        "token_mint": token_mint,
                        "trade_type": trade_type,
                        "copy_trade_id": result.copy_trade_id,
                        "sol_amount": result.sol_amount,
                        "tx_signature": result.tx_signature,
                        "latency_ms": result.latency_ms,
                    }),
                )).await;

                Ok(())
            }
            Err(e) => {
                tracing::error!(
                    kol_id = %kol_id,
                    token_mint = %token_mint,
                    error = %e,
                    "❌ KOL copy trade failed"
                );

                {
                    let mut s = stats.write().await;
                    s.executions_failed += 1;
                }

                Err(e)
            }
        }
    }
}

pub fn spawn_autonomous_executor(
    strategy_engine: Arc<StrategyEngine>,
    curve_builder: Arc<CurveTransactionBuilder>,
    dev_signer: Arc<DevWalletSigner>,
    helius_sender: Arc<HeliusSender>,
    position_manager: Arc<PositionManager>,
    risk_config: Arc<RwLock<RiskConfig>>,
    engrams_client: Arc<EngramsClient>,
    consensus_engine: Option<Arc<ConsensusEngine>>,
    consensus_config: Arc<RwLock<ConsensusConfig>>,
    event_tx: broadcast::Sender<ArbEvent>,
    default_wallet: String,
) -> Arc<AutonomousExecutor> {
    Arc::new(AutonomousExecutor::new(
        strategy_engine,
        curve_builder,
        dev_signer,
        helius_sender,
        position_manager,
        risk_config,
        engrams_client,
        consensus_engine,
        consensus_config,
        event_tx,
        default_wallet,
    ))
}

pub fn start_autonomous_executor(executor: Arc<AutonomousExecutor>) {
    let executor_clone = executor.clone();
    tokio::spawn(async move {
        executor_clone.start().await;
    });
}
