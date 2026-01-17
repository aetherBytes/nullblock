use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use crate::error::AppResult;
use crate::events::{ArbEvent, AgentType, EventSource, Significance};
use crate::execution::{CurveTransactionBuilder, CurveSellParams, JitoClient};
use crate::models::CurveStrategyParams;
use crate::venues::curves::OnChainFetcher;

use super::graduation_tracker::{GraduationTracker, TrackedState, TrackedToken};

const DEFAULT_SELL_DELAY_MS: u64 = 500;
const MAX_SELL_RETRIES: u32 = 3;

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
    sell_delay_ms: u64,
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
            sell_delay_ms: DEFAULT_SELL_DELAY_MS,
        }
    }

    pub fn with_sell_delay(mut self, delay_ms: u64) -> Self {
        self.sell_delay_ms = delay_ms;
        self
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

        tracing::info!("🔫 Graduation sniper started");

        let mut event_rx = self.event_tx.subscribe();
        let positions = self.positions.clone();
        let curve_builder = self.curve_builder.clone();
        let jito_client = self.jito_client.clone();
        let event_tx = self.event_tx.clone();
        let default_wallet = self.default_wallet.clone();
        let is_running = self.is_running.clone();
        let sell_delay_ms = self.sell_delay_ms;

        tokio::spawn(async move {
            loop {
                if !*is_running.read().await {
                    break;
                }

                tokio::select! {
                    Ok(event) = event_rx.recv() => {
                        if event.event_type == "arb.curve.graduated" {
                            if let Some(mint) = event.payload.get("mint").and_then(|v| v.as_str()) {
                                    let has_position = {
                                        let positions = positions.read().await;
                                        positions.contains_key(mint)
                                    };

                                    if has_position {
                                        tracing::info!(
                                            "🎓 Graduation detected for position {}! Executing sell...",
                                            mint
                                        );

                                        tokio::time::sleep(tokio::time::Duration::from_millis(sell_delay_ms)).await;

                                        Self::execute_graduation_sell(
                                            &positions,
                                            mint,
                                            &curve_builder,
                                            &jito_client,
                                            &event_tx,
                                            &default_wallet,
                                        ).await;
                                    }
                                }
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
    ) {
        let position = {
            let mut pos = positions.write().await;
            if let Some(p) = pos.get_mut(mint) {
                if p.status != SnipeStatus::Waiting {
                    tracing::warn!("Position {} not in Waiting state, skipping", mint);
                    return;
                }
                p.status = SnipeStatus::Selling;
                p.sell_attempts += 1;
                p.last_sell_attempt = Some(Utc::now());
                p.clone()
            } else {
                return;
            }
        };

        let sell_params = CurveSellParams {
            mint: mint.to_string(),
            token_amount: position.entry_tokens,
            slippage_bps: 300,
            user_wallet: wallet.to_string(),
        };

        match curve_builder.build_pump_fun_sell(&sell_params).await {
            Ok(build_result) => {
                tracing::info!(
                    "📤 Built sell tx for {}: expected {} SOL, impact {:.2}%",
                    position.symbol,
                    build_result.expected_sol_out.unwrap_or(0) as f64 / 1e9,
                    build_result.price_impact_percent
                );

                let tx_bytes = match base64::Engine::decode(
                    &base64::engine::general_purpose::STANDARD,
                    &build_result.transaction_base64,
                ) {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        Self::mark_position_failed(positions, mint, &format!("Decode error: {}", e)).await;
                        return;
                    }
                };

                match jito_client.send_bundle_fast(&[tx_bytes]).await {
                    Ok(bundle_id) => {
                        let expected_sol = build_result.expected_sol_out.unwrap_or(0) as f64 / 1e9;
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

                        let pnl_percent = (pnl / position.entry_price_sol) * 100.0;
                        tracing::info!(
                            "✅ Sold {} for {} SOL (PnL: {:.4} SOL / {:.2}%)",
                            position.symbol,
                            expected_sol,
                            pnl,
                            pnl_percent
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

                        let should_retry = {
                            let pos = positions.read().await;
                            pos.get(mint)
                                .map(|p| p.sell_attempts < MAX_SELL_RETRIES)
                                .unwrap_or(false)
                        };

                        if should_retry {
                            {
                                let mut pos = positions.write().await;
                                if let Some(p) = pos.get_mut(mint) {
                                    p.status = SnipeStatus::Waiting;
                                }
                            }
                            tracing::info!("Will retry sell for {} (attempt {})", mint, position.sell_attempts);
                        } else {
                            Self::mark_position_failed(positions, mint, &format!("Jito error: {}", e)).await;
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to build sell tx for {}: {}", mint, e);
                Self::mark_position_failed(positions, mint, &format!("Build error: {}", e)).await;
            }
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

        let _ = event_tx.send(event);
    }

    pub async fn manual_sell(&self, mint: &str) -> AppResult<()> {
        let position = {
            let positions = self.positions.read().await;
            positions.get(mint).cloned()
        };

        match position {
            Some(pos) if pos.status == SnipeStatus::Waiting => {
                Self::execute_graduation_sell(
                    &self.positions,
                    mint,
                    &self.curve_builder,
                    &self.jito_client,
                    &self.event_tx,
                    &self.default_wallet,
                ).await;
                Ok(())
            }
            Some(pos) => {
                Err(crate::error::AppError::Validation(format!(
                    "Position {} is in {:?} state, cannot sell",
                    mint,
                    pos.status
                )))
            }
            None => {
                Err(crate::error::AppError::NotFound(format!(
                    "No position found for {}",
                    mint
                )))
            }
        }
    }
}
