use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use crate::agents::StrategyEngine;
use crate::error::{AppError, AppResult};
use crate::events::{edge as edge_topics, ArbEvent, AgentType, EventSource};
use crate::execution::{CurveBuyParams, CurveTransactionBuilder, ExitConfig, PositionManager};
use crate::execution::risk::RiskConfig;
use crate::helius::HeliusSender;
use crate::models::{Edge, EdgeStatus, Strategy};
use crate::wallet::DevWalletSigner;
use crate::wallet::turnkey::SignRequest;

const MAX_EXECUTION_RETRIES: u32 = 2;
const EXECUTION_COOLDOWN_MS: u64 = 1000;
const MINT_COOLDOWN_SECONDS: i64 = 300;
const MIN_PROFIT_THRESHOLD_LAMPORTS: u64 = 500_000;
const ESTIMATED_GAS_COST_LAMPORTS: u64 = 250_000;

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
    event_tx: broadcast::Sender<ArbEvent>,
    executions: Arc<RwLock<HashMap<Uuid, AutoExecutionRecord>>>,
    recent_mints: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
    stats: Arc<RwLock<AutoExecutorStats>>,
    is_running: Arc<RwLock<bool>>,
    default_wallet: String,
    default_slippage_bps: u16,
}

impl AutonomousExecutor {
    pub fn new(
        strategy_engine: Arc<StrategyEngine>,
        curve_builder: Arc<CurveTransactionBuilder>,
        dev_signer: Arc<DevWalletSigner>,
        helius_sender: Arc<HeliusSender>,
        position_manager: Arc<PositionManager>,
        risk_config: Arc<RwLock<RiskConfig>>,
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
        }
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
        let event_tx = self.event_tx.clone();
        let executions = self.executions.clone();
        let recent_mints = self.recent_mints.clone();
        let stats = self.stats.clone();
        let is_running = self.is_running.clone();
        let default_wallet = self.default_wallet.clone();
        let default_slippage_bps = self.default_slippage_bps;

        tokio::spawn(async move {
            tracing::info!("🤖 Autonomous executor event loop started, waiting for events...");
            let mut events_received = 0u64;

            loop {
                let running = { *is_running.read().await };
                if !running {
                    break;
                }

                tokio::select! {
                    Ok(event) = event_rx.recv() => {
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
                                &event_tx,
                                &executions,
                                &recent_mints,
                                &stats,
                                &default_wallet,
                                default_slippage_bps,
                            ).await {
                                tracing::warn!("Auto-execution failed: {}", e);
                            }
                        }
                    }
                    _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)) => {}
                }
            }

            tracing::info!("🤖 Autonomous executor stopped");
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

    async fn handle_edge_detected(
        event: &ArbEvent,
        strategy_engine: &Arc<StrategyEngine>,
        curve_builder: &Arc<CurveTransactionBuilder>,
        dev_signer: &Arc<DevWalletSigner>,
        helius_sender: &Arc<HeliusSender>,
        position_manager: &Arc<PositionManager>,
        risk_config: &Arc<RwLock<RiskConfig>>,
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

        let execution_mode = event.payload.get("execution_mode")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if !execution_mode.to_lowercase().contains("autonomous") {
            tracing::debug!(
                edge_id = %edge_id,
                mode = %execution_mode,
                "Skipping non-autonomous edge"
            );
            return Ok(());
        }

        let strategy = strategy_engine.get_strategy(strategy_id).await
            .ok_or_else(|| AppError::NotFound(format!("Strategy {} not found", strategy_id)))?;

        let is_autonomous_mode = strategy.execution_mode.to_lowercase() == "autonomous";
        let can_execute = is_autonomous_mode || strategy.can_auto_execute();

        if !can_execute {
            tracing::debug!(
                edge_id = %edge_id,
                strategy_id = %strategy_id,
                execution_mode = %strategy.execution_mode,
                "Strategy cannot auto-execute (mode: {}, can_auto_execute: {})",
                strategy.execution_mode,
                strategy.can_auto_execute()
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

        if !dev_signer.is_configured() {
            tracing::warn!("Cannot auto-execute: dev signer not configured");
            return Err(AppError::Internal("Dev signer not configured".into()));
        }

        let mint = event.payload.get("token_mint")
            .or_else(|| event.payload.get("mint"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let mint = match mint {
            Some(m) => m,
            None => {
                tracing::debug!(edge_id = %edge_id, "No token mint in edge, skipping");
                return Ok(());
            }
        };

        if position_manager.has_open_position_for_mint(&mint).await {
            tracing::info!(
                edge_id = %edge_id,
                mint = %mint,
                "⏭️ Skipping: already have open position for this mint"
            );
            return Ok(());
        }

        {
            let now = Utc::now();
            let cooldown = Duration::seconds(MINT_COOLDOWN_SECONDS);
            let mut mints = recent_mints.write().await;

            mints.retain(|_, last_exec| now.signed_duration_since(*last_exec) < cooldown);

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
        }

        let global_max_sol = risk_config.read().await.max_position_sol;
        let strategy_max_sol = strategy.risk_params.max_position_sol;
        let capped_sol = strategy_max_sol.min(global_max_sol);

        tracing::info!(
            edge_id = %edge_id,
            strategy_max = strategy_max_sol,
            global_max = global_max_sol,
            capped = capped_sol,
            "💰 Position size: strategy wants {} SOL, global cap {} SOL → using {} SOL",
            strategy_max_sol, global_max_sol, capped_sol
        );

        let sol_amount_lamports = (capped_sol * 1_000_000_000.0) as u64;

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

        // Don't enter if momentum is strongly negative (price declining)
        if velocity < -1.0 {
            tracing::info!(
                edge_id = %edge_id,
                mint = %mint,
                velocity = velocity,
                "⏭️ Skipping: negative momentum at entry (velocity: {:.2}%/min)",
                velocity
            );
            return Ok(());
        }

        // Don't enter if already pumped significantly (buying the top)
        // This prevents FOMO entries after a big spike
        let max_recent_pump = 25.0;
        if price_change_1m > max_recent_pump {
            tracing::info!(
                edge_id = %edge_id,
                mint = %mint,
                price_change_1m = price_change_1m,
                max_allowed = max_recent_pump,
                "⏭️ Skipping: already pumped {:.1}% in last minute (max {:.0}%)",
                price_change_1m,
                max_recent_pump
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

        let _ = event_tx.send(ArbEvent::new(
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
        ));

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

                let _ = event_tx.send(ArbEvent::new(
                    "auto_execution_succeeded",
                    EventSource::Agent(AgentType::Executor),
                    edge_topics::EXECUTED,
                    serde_json::json!({
                        "edge_id": edge_id,
                        "strategy_id": strategy_id,
                        "mint": mint,
                        "signature": signature,
                        "tokens_received": tokens_out,
                        "sol_spent": sol_amount_lamports as f64 / 1e9,
                        "significance": "critical",
                    }),
                ));

                {
                    let mut mints = recent_mints.write().await;
                    mints.insert(mint.clone(), Utc::now());
                }

                let tokens_received = tokens_out.unwrap_or(0);
                if tokens_received > 0 {
                    let entry_price = sol_amount_lamports as f64 / tokens_received as f64;
                    // Use curve-specific exit config with trailing stops
                    let exit_config = ExitConfig::for_curve_bonding();

                    if let Err(e) = position_manager.open_position(
                        edge_id,
                        strategy_id,
                        mint.clone(),
                        None,
                        sol_amount_lamports as f64 / 1e9,
                        tokens_received as f64,
                        entry_price,
                        exit_config,
                        Some(signature.clone()),
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
                            tokens = tokens_received,
                            "📊 Position opened for tracking"
                        );
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

                let _ = event_tx.send(ArbEvent::new(
                    "auto_execution_failed",
                    EventSource::Agent(AgentType::Executor),
                    edge_topics::FAILED,
                    serde_json::json!({
                        "edge_id": edge_id,
                        "strategy_id": strategy_id,
                        "mint": mint,
                        "error": e.to_string(),
                    }),
                ));

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
}

pub fn spawn_autonomous_executor(
    strategy_engine: Arc<StrategyEngine>,
    curve_builder: Arc<CurveTransactionBuilder>,
    dev_signer: Arc<DevWalletSigner>,
    helius_sender: Arc<HeliusSender>,
    position_manager: Arc<PositionManager>,
    risk_config: Arc<RwLock<RiskConfig>>,
    event_tx: broadcast::Sender<ArbEvent>,
    default_wallet: String,
) -> Arc<AutonomousExecutor> {
    let executor = Arc::new(AutonomousExecutor::new(
        strategy_engine,
        curve_builder,
        dev_signer,
        helius_sender,
        position_manager,
        risk_config,
        event_tx,
        default_wallet,
    ));

    let executor_clone = executor.clone();
    tokio::spawn(async move {
        executor_clone.start().await;
    });

    executor
}
