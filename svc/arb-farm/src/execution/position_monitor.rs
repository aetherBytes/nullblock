use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::events::{AgentType, ArbEvent, EventSource};
use crate::helius::HeliusSender;
use crate::wallet::turnkey::SignRequest;
use crate::wallet::DevWalletSigner;

use super::curve_builder::{CurveSellParams, CurveTransactionBuilder};
use super::jito::{BundleState, JitoClient};
use super::position_manager::{
    BaseCurrency, ExitReason, ExitSignal, ExitUrgency, OpenPosition, PositionManager,
    PositionStatus,
};
use super::transaction_builder::TransactionBuilder;

const MIN_PROFIT_LAMPORTS: i64 = 500_000;

pub struct PositionMonitor {
    position_manager: Arc<PositionManager>,
    tx_builder: Arc<TransactionBuilder>,
    jito_client: Arc<JitoClient>,
    event_tx: broadcast::Sender<ArbEvent>,
    config: MonitorConfig,
    curve_builder: Option<Arc<CurveTransactionBuilder>>,
    helius_sender: Option<Arc<HeliusSender>>,
}

#[derive(Debug, Clone)]
pub struct MonitorConfig {
    pub price_check_interval_secs: u64,
    pub exit_slippage_bps: u16,
    pub max_exit_retries: u32,
    pub emergency_slippage_bps: u16,
    pub bundle_timeout_secs: u64,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            price_check_interval_secs: 30,
            exit_slippage_bps: 500,       // 5% - curves are volatile
            max_exit_retries: 3,
            emergency_slippage_bps: 1500, // 15% - aggressive for emergencies
            bundle_timeout_secs: 60,
        }
    }
}

impl PositionMonitor {
    pub fn new(
        position_manager: Arc<PositionManager>,
        tx_builder: Arc<TransactionBuilder>,
        jito_client: Arc<JitoClient>,
        event_tx: broadcast::Sender<ArbEvent>,
        config: MonitorConfig,
    ) -> Self {
        Self {
            position_manager,
            tx_builder,
            jito_client,
            event_tx,
            config,
            curve_builder: None,
            helius_sender: None,
        }
    }

    pub fn with_curve_support(
        mut self,
        curve_builder: Arc<CurveTransactionBuilder>,
        helius_sender: Arc<HeliusSender>,
    ) -> Self {
        self.curve_builder = Some(curve_builder);
        self.helius_sender = Some(helius_sender);
        self
    }

    pub async fn start_monitoring(&self, signer: Arc<DevWalletSigner>) {
        info!(
            "🔭 Position monitor started (checking every {}s)",
            self.config.price_check_interval_secs
        );

        loop {
            if let Err(e) = self.check_and_process_exits(&signer).await {
                error!("Position monitor error: {}", e);
            }

            tokio::time::sleep(Duration::from_secs(self.config.price_check_interval_secs)).await;
        }
    }

    async fn check_and_process_exits(&self, signer: &DevWalletSigner) -> AppResult<()> {
        let positions = self.position_manager.get_open_positions().await;

        if positions.is_empty() {
            return Ok(());
        }

        debug!("Checking {} open positions for exit conditions", positions.len());

        let token_mints: Vec<String> = positions.iter().map(|p| p.token_mint.clone()).collect();

        let unique_mints: Vec<String> = token_mints
            .into_iter()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        // Try Jupiter first, then fallback to bonding curve for pre-graduation tokens
        let mut prices = match self
            .tx_builder
            .get_multiple_token_prices(&unique_mints, BaseCurrency::Sol)
            .await
        {
            Ok(p) => p,
            Err(e) => {
                debug!("Jupiter price fetch failed (expected for pre-grad tokens): {}", e);
                std::collections::HashMap::new()
            }
        };

        // For mints without Jupiter prices, try fetching from bonding curve
        if let Some(curve_builder) = &self.curve_builder {
            for mint in &unique_mints {
                if !prices.contains_key(mint) {
                    match curve_builder.get_curve_state(mint).await {
                        Ok(state) => {
                            // Calculate price from curve state: virtual_sol_reserves / virtual_token_reserves
                            if state.virtual_token_reserves > 0 {
                                let price = state.virtual_sol_reserves as f64 / state.virtual_token_reserves as f64;
                                debug!(
                                    mint = %mint,
                                    price = price,
                                    "Fetched price from bonding curve"
                                );
                                prices.insert(mint.clone(), price);
                            }
                        }
                        Err(e) => {
                            debug!(mint = %mint, error = %e, "Failed to fetch curve state for price");
                        }
                    }
                }
            }
        }

        if prices.is_empty() {
            warn!("No prices available for any positions");
            return Ok(());
        }

        let mut all_signals = Vec::new();
        for (mint, price) in &prices {
            let signals = self.position_manager.update_price(mint, *price).await;
            all_signals.extend(signals);
        }

        if all_signals.is_empty() {
            return Ok(());
        }

        info!("🚨 {} exit signals triggered", all_signals.len());

        for signal in all_signals {
            self.emit_exit_signal_event(&signal).await;

            if let Err(e) = self.process_exit_signal(&signal, signer).await {
                error!(
                    "Failed to process exit for position {}: {}",
                    signal.position_id, e
                );

                if signal.urgency == ExitUrgency::Critical {
                    warn!(
                        "Critical exit failed, will retry with higher slippage: {}",
                        signal.position_id
                    );
                }
            }
        }

        Ok(())
    }

    async fn process_exit_signal(
        &self,
        signal: &ExitSignal,
        signer: &DevWalletSigner,
    ) -> AppResult<()> {
        let position = match self.position_manager.get_position(signal.position_id).await {
            Some(p) => p,
            None => {
                warn!("Position {} no longer exists", signal.position_id);
                return Ok(());
            }
        };

        let wallet_status = signer.get_status().await;
        let user_wallet = match &wallet_status.wallet_address {
            Some(addr) => addr.clone(),
            None => {
                error!("No wallet configured for exit");
                return Ok(());
            }
        };

        let slippage = match signal.urgency {
            ExitUrgency::Critical => self.config.emergency_slippage_bps,
            ExitUrgency::High => self.config.exit_slippage_bps + 50,
            _ => self.config.exit_slippage_bps,
        };

        info!(
            "📤 Processing {} exit for {} | {}% @ {} | slippage: {} bps",
            format!("{:?}", signal.reason),
            position
                .token_symbol
                .as_deref()
                .unwrap_or(&position.token_mint[..8]),
            signal.exit_percent,
            signal.current_price,
            slippage
        );

        // Check if token is still on bonding curve (pre-graduation)
        let use_curve_sell = if let Some(ref curve_builder) = self.curve_builder {
            match curve_builder.get_curve_state(&position.token_mint).await {
                Ok(state) => {
                    if !state.is_complete {
                        info!(
                            "📈 Token {} still on bonding curve ({}% progress), using curve sell",
                            &position.token_mint[..8],
                            state.graduation_progress() * 100.0
                        );
                        true
                    } else {
                        info!(
                            "🎓 Token {} graduated, using DEX sell",
                            &position.token_mint[..8]
                        );
                        false
                    }
                }
                Err(e) => {
                    warn!(
                        "Failed to get curve state for {}, falling back to DEX: {}",
                        &position.token_mint[..8],
                        e
                    );
                    false
                }
            }
        } else {
            false
        };

        // Execute curve sell if appropriate
        if use_curve_sell {
            return self.execute_curve_exit(&position, signal, signer, &user_wallet, slippage).await;
        }

        // Standard DEX exit path
        let exit_build = self
            .tx_builder
            .build_exit_swap(&position, signal, &user_wallet, slippage)
            .await?;

        let sign_request = SignRequest {
            transaction_base64: exit_build.transaction_base64.clone(),
            estimated_amount_lamports: exit_build.token_amount_in,
            estimated_profit_lamports: None,
            edge_id: Some(position.edge_id),
            description: format!(
                "Exit {} {} -> {} ({})",
                position
                    .token_symbol
                    .as_deref()
                    .unwrap_or(&position.token_mint[..8]),
                exit_build.token_amount_in,
                position.exit_config.base_currency.symbol(),
                format!("{:?}", signal.reason)
            ),
        };

        let sign_result = signer.sign_transaction(sign_request).await?;

        if !sign_result.success {
            let error_msg = sign_result
                .error
                .or_else(|| sign_result.policy_violation.map(|v| v.message))
                .unwrap_or_else(|| "Unknown signing error".to_string());
            error!("Exit signing failed: {}", error_msg);
            return Ok(());
        }

        let signed_tx = match sign_result.signed_transaction_base64 {
            Some(tx) => tx,
            None => {
                error!("No signed transaction returned for exit");
                return Ok(());
            }
        };

        let tip = 10_000; // 0.00001 SOL tip for exits
        let tx_base58 = base64_to_base58(&signed_tx)?;

        let bundle_result = self.jito_client.send_bundle(vec![tx_base58], tip).await?;
        let bundle_id = bundle_result.id.to_string();

        info!("📦 Exit bundle submitted: {}", bundle_id);

        let status = self
            .jito_client
            .wait_for_bundle(&bundle_id, self.config.bundle_timeout_secs)
            .await?;

        match status.status {
            BundleState::Landed => {
                let exit_price = signal.current_price;
                let pnl_percent = (exit_price - position.entry_price) / position.entry_price;
                let exit_reason = format!("{:?}", signal.reason);

                // Use remaining amount for P&L calculation (fixed for partial exit bug)
                let effective_base = if position.remaining_amount_base > 0.0 {
                    position.remaining_amount_base
                } else {
                    position.entry_amount_base
                };

                let is_partial_exit = signal.exit_percent < 100.0;

                if is_partial_exit {
                    let partial_base = effective_base * (signal.exit_percent / 100.0);
                    let realized_pnl_sol = partial_base * pnl_percent;

                    self.position_manager
                        .record_partial_exit(
                            signal.position_id,
                            signal.exit_percent,
                            exit_price,
                            realized_pnl_sol,
                            sign_result.signature.clone(),
                            &exit_reason,
                        )
                        .await?;

                    self.emit_exit_completed_event(
                        &position,
                        signal,
                        realized_pnl_sol,
                        sign_result.signature.as_deref(),
                    )
                    .await;

                    info!(
                        "✅ Partial exit completed: {} | {}% exited | P&L: {:.6} {} ({:.2}%) | Reason: {:?}",
                        position
                            .token_symbol
                            .as_deref()
                            .unwrap_or(&position.token_mint[..8]),
                        signal.exit_percent,
                        realized_pnl_sol,
                        position.exit_config.base_currency.symbol(),
                        pnl_percent * 100.0,
                        signal.reason
                    );
                } else {
                    let realized_pnl_sol = effective_base * pnl_percent;

                    self.position_manager
                        .close_position(
                            signal.position_id,
                            exit_price,
                            realized_pnl_sol,
                            &exit_reason,
                            sign_result.signature.clone(),
                        )
                        .await?;

                    self.emit_exit_completed_event(
                        &position,
                        signal,
                        realized_pnl_sol,
                        sign_result.signature.as_deref(),
                    )
                    .await;

                    info!(
                        "✅ Exit completed: {} | P&L: {:.6} {} ({:.2}%) | Reason: {:?}",
                        position
                            .token_symbol
                            .as_deref()
                            .unwrap_or(&position.token_mint[..8]),
                        realized_pnl_sol,
                        position.exit_config.base_currency.symbol(),
                        pnl_percent * 100.0,
                        signal.reason
                    );
                }
            }
            BundleState::Failed | BundleState::Dropped => {
                warn!(
                    "Exit bundle {} failed: {:?}",
                    bundle_id, status.status
                );

                self.emit_exit_failed_event(&position, signal, &format!("{:?}", status.status))
                    .await;
            }
            BundleState::Pending => {
                warn!("Exit bundle {} timed out", bundle_id);
            }
        }

        Ok(())
    }

    async fn execute_curve_exit(
        &self,
        position: &OpenPosition,
        signal: &ExitSignal,
        signer: &DevWalletSigner,
        user_wallet: &str,
        initial_slippage: u16,
    ) -> AppResult<()> {
        let curve_builder = self.curve_builder.as_ref()
            .ok_or_else(|| AppError::Internal("Curve builder not configured".into()))?;
        let helius_sender = self.helius_sender.as_ref()
            .ok_or_else(|| AppError::Internal("Helius sender not configured".into()))?;

        // Use remaining_token_amount if available (after partial exits), otherwise entry_token_amount
        let effective_token_amount = if position.remaining_token_amount > 0.0 {
            position.remaining_token_amount
        } else {
            position.entry_token_amount
        };
        let token_amount = (effective_token_amount * (signal.exit_percent / 100.0)) as u64;
        let max_retries = self.config.max_exit_retries;
        let max_slippage: u16 = 2500; // Cap at 25%

        let mut current_slippage = initial_slippage;
        let mut last_error = String::new();

        for attempt in 0..=max_retries {
            if attempt > 0 {
                // Increase slippage by 50% each retry, cap at max
                current_slippage = (current_slippage as u32 * 150 / 100).min(max_slippage as u32) as u16;
                info!(
                    "🔄 Retry {} with increased slippage: {} bps",
                    attempt, current_slippage
                );
                // Small delay before retry
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }

            info!(
                "📈 Building curve sell for {} tokens @ mint {} (slippage: {} bps)",
                token_amount,
                &position.token_mint[..8],
                current_slippage
            );

            let sell_params = CurveSellParams {
                mint: position.token_mint.clone(),
                token_amount,
                slippage_bps: current_slippage,
                user_wallet: user_wallet.to_string(),
            };

            let build_result = match curve_builder.build_pump_fun_sell(&sell_params).await {
                Ok(r) => r,
                Err(e) => {
                    last_error = e.to_string();
                    warn!("Build failed: {}", last_error);
                    continue;
                }
            };

            let sign_request = SignRequest {
                transaction_base64: build_result.transaction_base64.clone(),
                estimated_amount_lamports: build_result.expected_sol_out.unwrap_or(0) as u64,
                estimated_profit_lamports: None,
                edge_id: Some(position.edge_id),
                description: format!(
                    "Curve exit {} {} -> SOL ({}) [slippage: {}bps]",
                    position
                        .token_symbol
                        .as_deref()
                        .unwrap_or(&position.token_mint[..8]),
                    token_amount,
                    format!("{:?}", signal.reason),
                    current_slippage
                ),
            };

            let sign_result = match signer.sign_transaction(sign_request).await {
                Ok(r) => r,
                Err(e) => {
                    last_error = e.to_string();
                    warn!("Signing failed: {}", last_error);
                    continue;
                }
            };

            if !sign_result.success {
                last_error = sign_result
                    .error
                    .or_else(|| sign_result.policy_violation.map(|v| v.message))
                    .unwrap_or_else(|| "Unknown signing error".to_string());
                warn!("Curve exit signing rejected: {}", last_error);
                continue;
            }

            let signed_tx = match sign_result.signed_transaction_base64 {
                Some(tx) => tx,
                None => {
                    last_error = "No signed transaction returned".to_string();
                    continue;
                }
            };

            info!("📤 Sending curve sell via Helius (attempt {})...", attempt + 1);

            match helius_sender.send_transaction(&signed_tx, true).await {
                Ok(signature) => {
                    let exit_price = signal.current_price;
                    let pnl_percent = (exit_price - position.entry_price) / position.entry_price;
                    let exit_reason = format!("{:?}", signal.reason);

                    // Handle partial vs full exit differently
                    let is_partial_exit = signal.exit_percent < 100.0;

                    if is_partial_exit {
                        // For partial exits, use remaining amount for P&L calc
                        let effective_base = if position.remaining_amount_base > 0.0 {
                            position.remaining_amount_base
                        } else {
                            position.entry_amount_base
                        };
                        let partial_base = effective_base * (signal.exit_percent / 100.0);
                        let realized_pnl_sol = partial_base * pnl_percent;

                        self.position_manager
                            .record_partial_exit(
                                signal.position_id,
                                signal.exit_percent,
                                exit_price,
                                realized_pnl_sol,
                                Some(signature.clone()),
                                &exit_reason,
                            )
                            .await?;

                        self.emit_exit_completed_event(
                            position,
                            signal,
                            realized_pnl_sol,
                            Some(&signature),
                        )
                        .await;

                        info!(
                            "✅ Partial curve exit completed: {} | {}% exited | P&L: {:.6} SOL ({:.2}%) | Reason: {:?} | Sig: {}",
                            position
                                .token_symbol
                                .as_deref()
                                .unwrap_or(&position.token_mint[..8]),
                            signal.exit_percent,
                            realized_pnl_sol,
                            pnl_percent * 100.0,
                            signal.reason,
                            &signature[..16]
                        );
                    } else {
                        // Full exit - use remaining base for P&L if available
                        let effective_base = if position.remaining_amount_base > 0.0 {
                            position.remaining_amount_base
                        } else {
                            position.entry_amount_base
                        };
                        let realized_pnl_sol = effective_base * pnl_percent;

                        self.position_manager
                            .close_position(
                                signal.position_id,
                                exit_price,
                                realized_pnl_sol,
                                &exit_reason,
                                Some(signature.clone()),
                            )
                            .await?;

                        self.emit_exit_completed_event(
                            position,
                            signal,
                            realized_pnl_sol,
                            Some(&signature),
                        )
                        .await;

                        info!(
                            "✅ Curve exit completed: {} | P&L: {:.6} SOL ({:.2}%) | Reason: {:?} | Sig: {}",
                            position
                                .token_symbol
                                .as_deref()
                                .unwrap_or(&position.token_mint[..8]),
                            realized_pnl_sol,
                            pnl_percent * 100.0,
                            signal.reason,
                            &signature[..16]
                        );
                    }
                    return Ok(());
                }
                Err(e) => {
                    last_error = e.to_string();
                    let is_slippage_error = last_error.contains("6003")
                        || last_error.to_lowercase().contains("slippage");

                    if is_slippage_error && attempt < max_retries {
                        warn!(
                            "⚠️ Slippage error on attempt {}, will retry: {}",
                            attempt + 1, last_error
                        );
                        continue;
                    } else {
                        error!("Curve exit failed: {}", last_error);
                        break;
                    }
                }
            }
        }

        // All retries exhausted
        error!(
            "❌ Curve exit failed after {} attempts: {}",
            max_retries + 1, last_error
        );
        self.emit_exit_failed_event(position, signal, &last_error).await;
        Ok(())
    }

    async fn emit_exit_signal_event(&self, signal: &ExitSignal) {
        let event = ArbEvent::new(
            "position.exit_signal",
            EventSource::Agent(AgentType::Executor),
            "arb.position.exit_signal",
            serde_json::json!({
                "position_id": signal.position_id,
                "reason": format!("{:?}", signal.reason),
                "exit_percent": signal.exit_percent,
                "current_price": signal.current_price,
                "urgency": format!("{:?}", signal.urgency),
                "triggered_at": signal.triggered_at,
            }),
        );

        let _ = self.event_tx.send(event);
    }

    async fn emit_exit_completed_event(
        &self,
        position: &OpenPosition,
        signal: &ExitSignal,
        realized_pnl_sol: f64,
        tx_signature: Option<&str>,
    ) {
        let pnl_percent = if position.entry_amount_base > 0.0 {
            (realized_pnl_sol / position.entry_amount_base) * 100.0
        } else {
            0.0
        };

        let event = ArbEvent::new(
            "position.exit_completed",
            EventSource::Agent(AgentType::Executor),
            "arb.position.exit_completed",
            serde_json::json!({
                "position_id": position.id,
                "edge_id": position.edge_id,
                "strategy_id": position.strategy_id,
                "token_mint": position.token_mint,
                "token_symbol": position.token_symbol,
                "exit_reason": format!("{:?}", signal.reason),
                "entry_price": position.entry_price,
                "exit_price": signal.current_price,
                "entry_amount_sol": position.entry_amount_base,
                "realized_pnl_sol": realized_pnl_sol,
                "realized_pnl_percent": pnl_percent,
                "base_currency": position.exit_config.base_currency.symbol(),
                "tx_signature": tx_signature,
            }),
        );

        let _ = self.event_tx.send(event);
    }

    async fn emit_exit_failed_event(
        &self,
        position: &OpenPosition,
        signal: &ExitSignal,
        error: &str,
    ) {
        let event = ArbEvent::new(
            "position.exit_failed",
            EventSource::Agent(AgentType::Executor),
            "arb.position.exit_failed",
            serde_json::json!({
                "position_id": position.id,
                "edge_id": position.edge_id,
                "exit_reason": format!("{:?}", signal.reason),
                "error": error,
            }),
        );

        let _ = self.event_tx.send(event);
    }

    pub async fn trigger_manual_exit(
        &self,
        position_id: Uuid,
        exit_percent: f64,
        signer: &DevWalletSigner,
    ) -> AppResult<()> {
        let position = self
            .position_manager
            .get_position(position_id)
            .await
            .ok_or_else(|| {
                crate::error::AppError::NotFound(format!("Position {} not found", position_id))
            })?;

        let signal = ExitSignal {
            position_id,
            reason: ExitReason::Manual,
            exit_percent,
            current_price: position.current_price,
            triggered_at: chrono::Utc::now(),
            urgency: ExitUrgency::High,
        };

        self.process_exit_signal(&signal, signer).await
    }

    pub async fn trigger_exit_with_reason(
        &self,
        signal: &ExitSignal,
        signer: &DevWalletSigner,
    ) -> AppResult<()> {
        self.process_exit_signal(signal, signer).await
    }

    pub async fn get_position_stats(&self) -> super::position_manager::PositionManagerStats {
        self.position_manager.get_stats().await
    }
}

fn base64_to_base58(base64_str: &str) -> AppResult<String> {
    use base64::{engine::general_purpose::STANDARD, Engine};

    let bytes = STANDARD
        .decode(base64_str)
        .map_err(|e| crate::error::AppError::Execution(format!("Invalid base64: {}", e)))?;

    Ok(bs58::encode(bytes).into_string())
}
