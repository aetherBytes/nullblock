use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use chrono::Utc;

use crate::engrams::EngramsClient;
use crate::engrams::schemas::{TransactionAction, TransactionMetadata, TransactionSummary};
use crate::error::{AppError, AppResult};
use crate::events::{AgentType, ArbEvent, EventSource, topics};
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
    engrams_client: Option<Arc<EngramsClient>>,
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
            price_check_interval_secs: 2,
            exit_slippage_bps: 1500,      // 15% - pump.fun curves are extremely volatile
            max_exit_retries: 3,
            emergency_slippage_bps: 2500, // 25% - prioritize getting out over profit retention
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
            engrams_client: None,
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

    pub fn with_engrams(mut self, engrams_client: Arc<EngramsClient>) -> Self {
        self.engrams_client = Some(engrams_client);
        self
    }

    fn calculate_profit_aware_slippage(&self, position: &OpenPosition, signal: &ExitSignal) -> u16 {
        const MIN_SLIPPAGE_BPS: u16 = 500;  // 5% floor - pump.fun curves move 10-20% in seconds
        const MAX_SLIPPAGE_BPS: u16 = 2000; // 20% cap - prioritize execution over profit retention
        const SALVAGE_SLIPPAGE_BPS: u16 = 9000; // 90% for dead token salvage - any recovery is better than zero
        const PROFIT_SACRIFICE_RATIO: f64 = 0.25; // 25% of profits - better to capture 75% than 0%

        let is_dead_token = signal.reason == ExitReason::Salvage
            || position.exit_config.custom_exit_instructions
                .as_ref()
                .map(|s| s.contains("DEAD TOKEN"))
                .unwrap_or(false);

        if is_dead_token {
            info!("💀 Dead token exit: using salvage slippage {}bps (90%)", SALVAGE_SLIPPAGE_BPS);
            return SALVAGE_SLIPPAGE_BPS;
        }

        let pnl_percent = if position.entry_price > 0.0 {
            ((signal.current_price - position.entry_price) / position.entry_price) * 100.0
        } else {
            0.0
        };

        let calculated_slippage = if pnl_percent > 0.0 {
            let profit_based = (pnl_percent * PROFIT_SACRIFICE_RATIO * 100.0) as u16;
            profit_based.max(MIN_SLIPPAGE_BPS)
        } else {
            MIN_SLIPPAGE_BPS
        };

        let urgency_multiplier = match signal.urgency {
            ExitUrgency::Critical => 1.5,
            ExitUrgency::High => 1.25,
            _ => 1.0,
        };

        let final_slippage = ((calculated_slippage as f64) * urgency_multiplier) as u16;

        info!(
            "📊 Slippage calc: PnL={:.2}% | base={}bps | urgency={:.1}x | final={}bps",
            pnl_percent, calculated_slippage, urgency_multiplier, final_slippage.min(MAX_SLIPPAGE_BPS)
        );

        final_slippage.min(MAX_SLIPPAGE_BPS)
    }

    async fn save_exit_to_engrams(
        &self,
        position: &OpenPosition,
        signal: &ExitSignal,
        realized_pnl_sol: f64,
        tx_signature: Option<&str>,
        wallet_address: &str,
    ) {
        let Some(engrams_client) = &self.engrams_client else {
            return;
        };

        let pnl_percent = if position.entry_amount_base > 0.0 {
            Some((realized_pnl_sol / position.entry_amount_base) * 100.0)
        } else {
            None
        };

        let venue = if self.curve_builder.is_some() {
            "pump_fun".to_string()
        } else {
            "jupiter".to_string()
        };

        let tx_summary = TransactionSummary {
            tx_signature: tx_signature.unwrap_or("unknown").to_string(),
            action: TransactionAction::Sell,
            token_mint: position.token_mint.clone(),
            token_symbol: position.token_symbol.clone(),
            venue,
            entry_sol: position.entry_amount_base,
            exit_sol: Some(position.entry_amount_base + realized_pnl_sol),
            pnl_sol: Some(realized_pnl_sol),
            pnl_percent,
            slippage_bps: self.config.exit_slippage_bps as i32,
            execution_time_ms: 0,
            strategy_id: Some(position.strategy_id),
            timestamp: Utc::now(),
            metadata: TransactionMetadata {
                graduation_progress: None,
                holder_count: None,
                volume_24h_sol: None,
                market_cap_sol: None,
                bonding_curve_percent: None,
            },
        };

        if let Err(e) = engrams_client.save_transaction_summary(wallet_address, &tx_summary).await {
            warn!("Failed to save exit transaction summary engram: {}", e);
        } else {
            info!(
                "📝 Saved exit transaction summary engram for {} (PnL: {:.6} SOL)",
                &tx_signature.unwrap_or("unknown")[..12.min(tx_signature.unwrap_or("unknown").len())],
                realized_pnl_sol
            );
        }
    }

    pub async fn start_monitoring(&self, signer: Arc<DevWalletSigner>) {
        info!(
            "🔭 Position monitor started (base interval {}s, adaptive)",
            self.config.price_check_interval_secs
        );

        let mut pending_exit_retry_counter: u64 = 0;

        loop {
            // Process HIGH PRIORITY exits first (failed sells that need immediate retry)
            if let Err(e) = self.process_priority_exits(&signer).await {
                error!("Priority exit processing error: {}", e);
            }

            // Retry pending exits every ~30 seconds (positions that failed to sell)
            pending_exit_retry_counter += 1;
            if pending_exit_retry_counter >= 10 {
                pending_exit_retry_counter = 0;
                if let Err(e) = self.retry_pending_exits(&signer).await {
                    error!("Pending exit retry error: {}", e);
                }
            }

            // Then check regular exit conditions
            if let Err(e) = self.check_and_process_exits(&signer).await {
                error!("Position monitor error: {}", e);
            }

            // Use adaptive interval based on position risk profile
            let interval = self.calculate_adaptive_interval().await;
            tokio::time::sleep(Duration::from_secs(interval)).await;
        }
    }

    async fn retry_pending_exits(&self, signer: &DevWalletSigner) -> AppResult<()> {
        // Get positions that are stuck in PendingExit (may have lost their signals on restart)
        let pending_exit_positions = self.position_manager.get_pending_exit_positions().await;

        if pending_exit_positions.is_empty() {
            return Ok(());
        }

        // Rate limit: Only retry ONE position per cycle to avoid API rate limits
        // With 30-second retry cycles, this means each position gets retried every ~3 minutes
        // if there are 6 positions (6 * 30s = 180s = 3 minutes)
        let retry_index = self.position_manager.get_and_increment_retry_index().await;
        let position_index = retry_index % pending_exit_positions.len();
        let position = &pending_exit_positions[position_index];

        info!(
            "🔄 Retrying PendingExit {}/{} for {} (cycle {})",
            position_index + 1,
            pending_exit_positions.len(),
            position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8]),
            retry_index
        );

        // Check if token is dead (zero liquidity) - skip retries for dead tokens
        let (current_price, is_dead_token) = if let Some(ref curve_builder) = self.curve_builder {
            match curve_builder.get_curve_state(&position.token_mint).await {
                Ok(state) if state.virtual_token_reserves > 0 && state.virtual_sol_reserves > 0 => {
                    let price = state.virtual_sol_reserves as f64 / state.virtual_token_reserves as f64;
                    (price, false)
                }
                Ok(state) if state.is_complete => {
                    // Token graduated, try Jupiter price
                    match self.tx_builder.get_token_price(&position.token_mint, BaseCurrency::Sol).await {
                        Ok(price) if price > 1e-10 => (price, false),
                        Ok(_) => {
                            // Price near zero = dead token
                            warn!(
                                "💀 DEAD TOKEN DETECTED (graduated, zero price): {} - skipping retries",
                                position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8])
                            );
                            (0.0, true)
                        }
                        Err(_) => {
                            // No Jupiter price for graduated token = likely dead
                            warn!(
                                "💀 DEAD TOKEN DETECTED (graduated, no DEX price): {} - skipping retries",
                                position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8])
                            );
                            (0.0, true)
                        }
                    }
                }
                Ok(state) if state.virtual_token_reserves == 0 || state.virtual_sol_reserves == 0 => {
                    // Zero reserves = dead token on curve
                    warn!(
                        "💀 DEAD TOKEN DETECTED (zero reserves): {} - skipping retries",
                        position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8])
                    );
                    (0.0, true)
                }
                _ => (position.current_price, false),
            }
        } else {
            (position.current_price, false)
        };

        let (reason, urgency) = if is_dead_token {
            warn!(
                "💀 Dead token {} - attempting salvage sell with maximum slippage",
                position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8])
            );
            (ExitReason::Salvage, ExitUrgency::Critical)
        } else {
            (ExitReason::Emergency, ExitUrgency::High)
        };

        let signal = ExitSignal {
            position_id: position.id,
            reason,
            exit_percent: 100.0,
            current_price: if is_dead_token { position.entry_price } else { current_price },
            triggered_at: chrono::Utc::now(),
            urgency,
        };

        info!(
            "🔄 Retrying {} for {} | Price: {:.10} | Entry: {:.10}",
            if is_dead_token { "Salvage" } else { "PendingExit" },
            position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8]),
            signal.current_price,
            position.entry_price
        );

        if let Err(e) = self.process_exit_signal(&signal, signer).await {
            if is_dead_token {
                warn!(
                    "💀 Salvage sell failed for {}: {} - marking as orphaned",
                    position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8]),
                    e
                );
                if let Err(e) = self.position_manager.mark_position_orphaned(position.id).await {
                    error!("Failed to mark dead token position as orphaned: {}", e);
                }
            } else {
                warn!(
                    "Pending exit retry failed for {}: {}",
                    position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8]),
                    e
                );
            }
        }

        Ok(())
    }

    async fn calculate_adaptive_interval(&self) -> u64 {
        let positions = self.position_manager.get_open_positions().await;

        if positions.is_empty() {
            return self.config.price_check_interval_secs;
        }

        // Check if any positions are at-risk (profitable but losing momentum)
        let has_at_risk = positions.iter().any(|p|
            p.unrealized_pnl_percent > 10.0 && p.momentum.velocity < 0.0
        );

        // Check if any positions are moderately profitable
        let has_profitable = positions.iter().any(|p| p.unrealized_pnl_percent > 5.0);

        if has_at_risk {
            1  // 1 second for at-risk positions
        } else if has_profitable {
            2  // 2 seconds for profitable positions
        } else {
            self.config.price_check_interval_secs  // Default for others
        }
    }

    async fn process_priority_exits(&self, signer: &DevWalletSigner) -> AppResult<()> {
        let priority_ids = self.position_manager.drain_priority_exits().await;
        if priority_ids.is_empty() {
            return Ok(());
        }

        info!("🔥🔥🔥 Processing {} HIGH PRIORITY exit retries with maximum slippage", priority_ids.len());

        for position_id in priority_ids {
            let position = match self.position_manager.get_position(position_id).await {
                Some(p) => p,
                None => {
                    warn!("Priority exit position {} no longer exists", position_id);
                    continue;
                }
            };

            // Check if token is dead (zero liquidity) - skip priority retries for dead tokens
            let (current_price, is_dead_token) = if let Some(curve_builder) = &self.curve_builder {
                match curve_builder.get_curve_state(&position.token_mint).await {
                    Ok(state) if state.virtual_token_reserves > 0 && state.virtual_sol_reserves > 0 => {
                        let price = state.virtual_sol_reserves as f64 / state.virtual_token_reserves as f64;
                        (price, false)
                    }
                    Ok(state) if state.is_complete => {
                        // Graduated - check DEX price
                        match self.tx_builder.get_token_price(&position.token_mint, BaseCurrency::Sol).await {
                            Ok(price) if price > 1e-10 => (price, false),
                            _ => {
                                warn!(
                                    "💀 DEAD TOKEN (priority): {} - graduated but no DEX liquidity",
                                    position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8])
                                );
                                (0.0, true)
                            }
                        }
                    }
                    Ok(_) => {
                        // Zero reserves
                        warn!(
                            "💀 DEAD TOKEN (priority): {} - zero reserves",
                            position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8])
                        );
                        (0.0, true)
                    }
                    Err(_) => (position.current_price, false),
                }
            } else {
                (position.current_price, false)
            };

            let (reason, price_to_use, log_prefix) = if is_dead_token {
                warn!(
                    "💀 Dead token {} in priority queue - attempting salvage sell",
                    position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8])
                );
                (ExitReason::Salvage, position.entry_price, "SALVAGE")
            } else {
                (ExitReason::Emergency, current_price, "PRIORITY RETRY")
            };

            let signal = ExitSignal {
                position_id,
                reason,
                exit_percent: 100.0,
                current_price: price_to_use,
                triggered_at: chrono::Utc::now(),
                urgency: ExitUrgency::Critical,
            };

            info!(
                "🔥 {}: {} | Using {} slippage",
                log_prefix,
                position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8]),
                if is_dead_token { "SALVAGE (90%)" } else { "EMERGENCY" }
            );

            if let Err(e) = self.process_exit_signal(&signal, signer).await {
                let error_msg = e.to_string();
                let is_rate_limited = error_msg.contains("429")
                    || error_msg.contains("rate limit")
                    || error_msg.contains("Rate limit");

                if is_dead_token {
                    error!(
                        "💀 Salvage sell failed for {}: {} - marking as orphaned",
                        position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8]),
                        e
                    );
                    if let Err(e) = self.position_manager.mark_position_orphaned(position_id).await {
                        error!("Failed to mark priority dead token as orphaned: {}", e);
                    }
                } else {
                    error!(
                        "🔴 Priority exit retry failed for {}: {}{}",
                        position_id,
                        e,
                        if is_rate_limited { " [RATE LIMITED]" } else { "" }
                    );
                    self.position_manager
                        .record_priority_exit_failure(position_id, is_rate_limited)
                        .await;
                }
            }
        }

        Ok(())
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

        // For dead tokens (no price from Jupiter or curve), use fallback price
        // This ensures time-based exits still trigger even when prices aren't available
        for position in &positions {
            if !prices.contains_key(&position.token_mint) {
                // Check if this is a dead token (has salvage sell instruction)
                let is_dead_token = position.exit_config.custom_exit_instructions
                    .as_ref()
                    .map(|s| s.contains("DEAD TOKEN"))
                    .unwrap_or(false);

                if is_dead_token {
                    // Use entry price as fallback - we just need to trigger time-based exit
                    info!(
                        "💀 Using fallback price for dead token {} (no market price available)",
                        &position.token_mint[..12]
                    );
                    prices.insert(position.token_mint.clone(), position.entry_price);
                } else if position.exit_config.time_limit_minutes.is_some() {
                    // Non-dead token with time limit but no price - still need to check time exit
                    debug!(
                        "Using fallback price for {} (time limit exit pending)",
                        &position.token_mint[..12]
                    );
                    prices.insert(position.token_mint.clone(), position.current_price);
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

        let slippage = self.calculate_profit_aware_slippage(&position, signal);

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
                            "📈 Token {} still on bonding curve ({:.2}% progress), using curve sell",
                            &position.token_mint[..8],
                            state.graduation_progress()
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

        // Graduated token DEX exit path - try Raydium first, then Jupiter fallback
        let (exit_tx_base64, expected_base_out, token_amount_in, route_label) =
            if let Some(ref curve_builder) = self.curve_builder {
                // Get actual token balance for the sell
                let token_balance = self.tx_builder.get_token_balance(&user_wallet, &position.token_mint).await?;
                let exit_amount = if signal.exit_percent >= 100.0 {
                    token_balance
                } else {
                    ((token_balance as f64) * (signal.exit_percent / 100.0)) as u64
                };

                let sell_params = CurveSellParams {
                    mint: position.token_mint.clone(),
                    token_amount: exit_amount,
                    slippage_bps: slippage,
                    user_wallet: user_wallet.clone(),
                };

                info!(
                    "🎓 Graduated token exit for {} - trying Raydium first",
                    position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8])
                );

                match curve_builder.build_raydium_sell(&sell_params).await {
                    Ok(raydium_result) => {
                        info!(
                            "📤 Built Raydium exit tx for {}: expected {} SOL, impact {:.2}%",
                            position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8]),
                            raydium_result.expected_sol_out as f64 / 1e9,
                            raydium_result.price_impact_percent
                        );
                        (
                            raydium_result.transaction_base64,
                            raydium_result.expected_sol_out,
                            exit_amount,
                            "Raydium".to_string(),
                        )
                    }
                    Err(raydium_err) => {
                        warn!(
                            "⚠️ Raydium exit failed for {}: {}, falling back to Jupiter",
                            position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8]),
                            raydium_err
                        );
                        let exit_build = self.tx_builder
                            .build_exit_swap(&position, signal, &user_wallet, slippage)
                            .await?;
                        (
                            exit_build.transaction_base64,
                            exit_build.expected_base_out,
                            exit_build.token_amount_in,
                            "Jupiter".to_string(),
                        )
                    }
                }
            } else {
                // No curve_builder, fall back to Jupiter directly
                let exit_build = self.tx_builder
                    .build_exit_swap(&position, signal, &user_wallet, slippage)
                    .await?;
                (
                    exit_build.transaction_base64,
                    exit_build.expected_base_out,
                    exit_build.token_amount_in,
                    "Jupiter".to_string(),
                )
            };

        let sign_request = SignRequest {
            transaction_base64: exit_tx_base64.clone(),
            estimated_amount_lamports: expected_base_out,
            estimated_profit_lamports: None,
            edge_id: Some(position.edge_id),
            description: format!(
                "Exit {} {} -> {} ({}) via {}",
                position
                    .token_symbol
                    .as_deref()
                    .unwrap_or(&position.token_mint[..8]),
                token_amount_in,
                position.exit_config.base_currency.symbol(),
                format!("{:?}", signal.reason),
                route_label
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

        // Try Jito first, fall back to Helius on failure
        let mut use_helius_fallback = false;
        let mut helius_signature: Option<String> = None;

        match self.jito_client.send_bundle(vec![tx_base58], tip).await {
            Ok(bundle_result) => {
                let bundle_id = bundle_result.id.to_string();
                info!("📦 Exit bundle submitted: {}", bundle_id);

                match self.jito_client.wait_for_bundle(&bundle_id, self.config.bundle_timeout_secs).await {
                    Ok(status) => {
                        match status.status {
                            BundleState::Landed => {
                                // Success - continue to position close logic below
                            }
                            BundleState::Failed | BundleState::Dropped | BundleState::Pending => {
                                warn!("🔴 Jito bundle {} status: {:?} - trying Helius fallback", bundle_id, status.status);
                                use_helius_fallback = true;
                            }
                        }
                    }
                    Err(e) => {
                        warn!("🔴 Jito bundle wait failed: {} - trying Helius fallback", e);
                        use_helius_fallback = true;
                    }
                }
            }
            Err(e) => {
                warn!("🔴 Jito bundle send failed: {} - trying Helius fallback", e);
                use_helius_fallback = true;
            }
        }

        // Helius fallback for graduated tokens when Jito fails
        if use_helius_fallback {
            if let Some(helius_sender) = &self.helius_sender {
                info!("📤 Sending DEX exit via Helius fallback...");
                let confirmation_timeout = std::time::Duration::from_secs(60); // Increased from 30s
                match helius_sender.send_and_confirm(&signed_tx, confirmation_timeout).await {
                    Ok(sig) => {
                        info!("✅ DEX exit confirmed via Helius: {}", sig);
                        helius_signature = Some(sig);
                    }
                    Err(e) => {
                        let error_str = e.to_string();

                        // Check if this is a timeout - the sell may have actually succeeded
                        if error_str.contains("Timeout") || error_str.contains("timeout") {
                            // Check wallet balance to see if tokens are actually gone
                            // user_wallet is already defined at the top of process_exit_signal
                            match self.tx_builder.get_token_balance(&user_wallet, &position.token_mint).await {
                                Ok(balance) => {
                                    // If balance is essentially 0 (< 1000 raw units = dust), the sell succeeded
                                    if balance < 1000 {
                                        info!("✅ Confirmation timed out but tokens are gone - treating as successful exit for {}",
                                            position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8]));
                                        // Proceed to close position as if successful
                                        helius_signature = Some("timeout-verified-success".to_string());
                                    } else {
                                        warn!("⚠️ Confirmation timed out and {} tokens still in wallet - will retry", balance);
                                        self.emit_exit_failed_event(&position, signal, &error_str).await;
                                        if let Err(reset_err) = self.position_manager.reset_position_status(signal.position_id).await {
                                            error!("Failed to reset position status: {}", reset_err);
                                        } else {
                                            self.position_manager.queue_priority_exit(signal.position_id).await;
                                            info!("🔥 Position {} queued for HIGH PRIORITY retry", signal.position_id);
                                        }
                                        return Ok(());
                                    }
                                }
                                Err(balance_err) => {
                                    warn!("Could not verify balance after timeout: {} - will retry", balance_err);
                                    self.emit_exit_failed_event(&position, signal, &error_str).await;
                                    if let Err(reset_err) = self.position_manager.reset_position_status(signal.position_id).await {
                                        error!("Failed to reset position status: {}", reset_err);
                                    } else {
                                        self.position_manager.queue_priority_exit(signal.position_id).await;
                                        info!("🔥 Position {} queued for HIGH PRIORITY retry", signal.position_id);
                                    }
                                    return Ok(());
                                }
                            }
                        } else {
                            // Non-timeout error - proceed with normal retry
                            error!("❌ Helius fallback also failed: {}", error_str);
                            self.emit_exit_failed_event(&position, signal, &error_str).await;
                            if let Err(reset_err) = self.position_manager.reset_position_status(signal.position_id).await {
                                error!("Failed to reset position status: {}", reset_err);
                            } else {
                                self.position_manager.queue_priority_exit(signal.position_id).await;
                                info!("🔥 Position {} queued for HIGH PRIORITY retry", signal.position_id);
                            }
                            return Ok(());
                        }
                    }
                }
            } else {
                error!("❌ No Helius sender available for fallback");
                self.emit_exit_failed_event(&position, signal, "No Helius fallback available").await;
                return Ok(());
            }
        }

        // Exit succeeded (either via Jito or Helius)
        let final_signature = helius_signature.or(sign_result.signature.clone());
        {
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
                            final_signature.clone(),
                            &exit_reason,
                        )
                        .await?;

                    self.emit_exit_completed_event(
                        &position,
                        signal,
                        realized_pnl_sol,
                        final_signature.as_deref(),
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

                    self.save_exit_to_engrams(
                        &position,
                        signal,
                        realized_pnl_sol,
                        final_signature.as_deref(),
                        &user_wallet,
                    )
                    .await;
                } else {
                    let realized_pnl_sol = effective_base * pnl_percent;

                    self.position_manager
                        .close_position(
                            signal.position_id,
                            exit_price,
                            realized_pnl_sol,
                            &exit_reason,
                            final_signature.clone(),
                        )
                        .await?;

                    self.emit_exit_completed_event(
                        &position,
                        signal,
                        realized_pnl_sol,
                        final_signature.as_deref(),
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

                    self.save_exit_to_engrams(
                        &position,
                        signal,
                        realized_pnl_sol,
                        final_signature.as_deref(),
                        &user_wallet,
                    )
                    .await;
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

        // CRITICAL: Fetch ACTUAL on-chain balance instead of using tracked amount (which can be stale)
        let actual_balance = curve_builder
            .get_actual_token_balance(user_wallet, &position.token_mint)
            .await
            .unwrap_or(0);

        if actual_balance == 0 {
            // Calculate PnL based on current price vs entry price
            // We use current_price as a proxy for exit price since the token was sold externally
            let pnl_percent = if position.entry_price > 0.0 {
                (position.current_price - position.entry_price) / position.entry_price
            } else {
                0.0
            };
            let effective_base = if position.remaining_amount_base > 0.0 {
                position.remaining_amount_base
            } else {
                position.entry_amount_base
            };
            let estimated_pnl = effective_base * pnl_percent;

            warn!(
                "⚠️ Token {} has zero on-chain balance - already sold or transferred. Closing position with estimated PnL: {:.6} SOL ({:.2}%)",
                &position.token_mint[..8],
                estimated_pnl,
                pnl_percent * 100.0
            );

            self.position_manager
                .close_position(
                    position.id,
                    position.current_price,
                    estimated_pnl,
                    "AlreadySold",
                    None,
                )
                .await?;
            return Ok(());
        }

        // Use actual on-chain balance, applying exit percent
        let token_amount = (actual_balance as f64 * (signal.exit_percent / 100.0)) as u64;

        info!(
            "📊 Actual on-chain balance: {} tokens (tracked: {:.0})",
            actual_balance,
            if position.remaining_token_amount > 0.0 {
                position.remaining_token_amount
            } else {
                position.entry_token_amount
            }
        );
        let max_retries = self.config.max_exit_retries;
        let mut current_slippage = initial_slippage;
        let mut last_error = String::new();
        let mut used_emergency = false;

        for attempt in 0..=max_retries {
            if attempt > 0 {
                // On ANY retry, immediately jump to emergency slippage
                // No gradual increase - if profit-aware failed, we need max tolerance
                if !used_emergency {
                    current_slippage = self.config.emergency_slippage_bps;
                    used_emergency = true;
                    warn!(
                        "🚨 EMERGENCY SLIPPAGE: Jumping to {}bps after failure (was {}bps)",
                        current_slippage, initial_slippage
                    );
                }
                // Brief delay before retry to let mempool clear
                tokio::time::sleep(std::time::Duration::from_millis(300)).await;
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

                    // If token graduated mid-exit, switch to Raydium/Jupiter path immediately
                    if last_error.contains("graduated") || last_error.contains("is_complete") {
                        warn!(
                            "🎓 Token {} graduated mid-exit, switching to DEX path",
                            position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8])
                        );

                        // Try Raydium first
                        match curve_builder.build_raydium_sell(&sell_params).await {
                            Ok(raydium_result) => {
                                info!(
                                    "📤 Built Raydium sell for graduated {}: expected {} SOL",
                                    position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8]),
                                    raydium_result.expected_sol_out as f64 / 1e9
                                );
                                // Use a simple struct to pass the result
                                super::curve_builder::CurveBuildResult {
                                    transaction_base64: raydium_result.transaction_base64,
                                    expected_tokens_out: None,
                                    expected_sol_out: Some(raydium_result.expected_sol_out),
                                    min_tokens_out: None,
                                    min_sol_out: None,
                                    price_impact_percent: raydium_result.price_impact_percent,
                                    fee_lamports: 0,
                                    compute_units: 200_000,
                                    priority_fee_lamports: 0,
                                }
                            }
                            Err(raydium_err) => {
                                warn!(
                                    "⚠️ Raydium failed for graduated {}: {}, falling back to Jupiter",
                                    position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8]),
                                    raydium_err
                                );
                                // Fall back to Jupiter via tx_builder
                                match self.tx_builder.build_exit_swap(position, signal, user_wallet, current_slippage).await {
                                    Ok(jupiter_result) => {
                                        super::curve_builder::CurveBuildResult {
                                            transaction_base64: jupiter_result.transaction_base64,
                                            expected_tokens_out: None,
                                            expected_sol_out: Some(jupiter_result.expected_base_out),
                                            min_tokens_out: None,
                                            min_sol_out: None,
                                            price_impact_percent: jupiter_result.price_impact_bps as f64 / 100.0,
                                            fee_lamports: 0,
                                            compute_units: 200_000,
                                            priority_fee_lamports: 0,
                                        }
                                    }
                                    Err(jupiter_err) => {
                                        last_error = format!("Raydium: {} | Jupiter: {}", raydium_err, jupiter_err);
                                        warn!("Both DEX paths failed: {}", last_error);
                                        continue;
                                    }
                                }
                            }
                        }
                    } else {
                        warn!("Build failed: {}", last_error);
                        continue;
                    }
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

            info!("📤 Sending curve sell via Helius (attempt {}) - waiting for confirmation...", attempt + 1);

            // Use send_and_confirm to wait for transaction to land before closing position
            let confirmation_timeout = Duration::from_secs(60); // Increased from 30s
            match helius_sender.send_and_confirm(&signed_tx, confirmation_timeout).await {
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

                        self.save_exit_to_engrams(
                            position,
                            signal,
                            realized_pnl_sol,
                            Some(&signature),
                            user_wallet,
                        )
                        .await;
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

                        self.save_exit_to_engrams(
                            position,
                            signal,
                            realized_pnl_sol,
                            Some(&signature),
                            user_wallet,
                        )
                        .await;
                    }
                    return Ok(());
                }
                Err(e) => {
                    last_error = e.to_string();
                    let is_slippage_error = last_error.contains("6003")
                        || last_error.to_lowercase().contains("slippage");
                    let is_timeout_error = last_error.contains("Timeout") || last_error.contains("timeout");

                    // On timeout, check if the sell actually succeeded
                    if is_timeout_error {
                        match self.tx_builder.get_token_balance(user_wallet, &position.token_mint).await {
                            Ok(balance) => {
                                if balance < 1000 {
                                    info!("✅ Curve sell confirmation timed out but tokens are gone - treating as successful exit for {}",
                                        position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8]));
                                    // Calculate P&L and close position
                                    let exit_price = signal.current_price;
                                    let pnl_percent = (exit_price - position.entry_price) / position.entry_price;
                                    let exit_reason = format!("{:?}", signal.reason);
                                    let effective_base = if position.remaining_amount_base > 0.0 {
                                        position.remaining_amount_base
                                    } else {
                                        position.entry_amount_base
                                    };
                                    let realized_pnl_sol = effective_base * pnl_percent;

                                    let _ = self.position_manager
                                        .close_position(
                                            signal.position_id,
                                            exit_price,
                                            realized_pnl_sol,
                                            &exit_reason,
                                            Some("timeout-verified-success".to_string()),
                                        )
                                        .await;

                                    self.emit_exit_completed_event(
                                        position,
                                        signal,
                                        realized_pnl_sol,
                                        Some("timeout-verified"),
                                    )
                                    .await;

                                    info!(
                                        "✅ Curve exit completed (timeout-verified): {} | P&L: {:.6} SOL ({:.2}%) | Reason: {:?}",
                                        position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8]),
                                        realized_pnl_sol,
                                        pnl_percent * 100.0,
                                        signal.reason
                                    );

                                    self.save_exit_to_engrams(
                                        position,
                                        signal,
                                        realized_pnl_sol,
                                        Some("timeout-verified"),
                                        user_wallet,
                                    )
                                    .await;

                                    return Ok(());
                                } else {
                                    warn!("⚠️ Curve sell timed out and {} tokens still in wallet - will retry", balance);
                                }
                            }
                            Err(balance_err) => {
                                warn!("Could not verify balance after curve sell timeout: {}", balance_err);
                            }
                        }
                    }

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

        // All retries exhausted - queue for high priority retry
        error!(
            "❌ Curve exit failed after {} attempts: {} - QUEUING HIGH PRIORITY RETRY",
            max_retries + 1, last_error
        );
        self.emit_exit_failed_event(position, signal, &last_error).await;

        // Reset position and queue for high-priority retry
        if let Err(e) = self.position_manager.reset_position_status(signal.position_id).await {
            error!("Failed to reset position status for retry: {}", e);
        } else {
            self.position_manager.queue_priority_exit(signal.position_id).await;
            info!("🔥 Position {} queued for HIGH PRIORITY retry after curve exit failure", signal.position_id);
        }

        Ok(())
    }

    async fn emit_exit_signal_event(&self, signal: &ExitSignal) {
        let event = ArbEvent::new(
            "position.exit_signal",
            EventSource::Agent(AgentType::Executor),
            topics::position::EXIT_PENDING,
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
            topics::position::CLOSED,
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
            topics::position::EXIT_FAILED,
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
