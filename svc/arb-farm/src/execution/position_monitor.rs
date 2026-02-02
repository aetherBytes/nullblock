use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use chrono::Utc;

use crate::error::AppResult;
use crate::events::{AgentType, ArbEvent, EventSource, topics};

use super::curve_builder::CurveTransactionBuilder;
use super::position_command::{CommandSource, ExitCommand, PositionCommand};
use super::position_manager::{
    BaseCurrency, ExitReason, ExitSignal, ExitUrgency, PositionManager,
};
use super::transaction_builder::TransactionBuilder;

const PRICE_FETCH_TIMEOUT_SECS: u64 = 10;
const GLOBAL_PRICE_FETCH_TIMEOUT_SECS: u64 = 60;
const MAX_STALE_PRICE_SECS: u64 = 300;

pub struct PositionMonitor {
    position_manager: Arc<PositionManager>,
    tx_builder: Arc<TransactionBuilder>,
    event_tx: broadcast::Sender<ArbEvent>,
    config: MonitorConfig,
    curve_builder: Option<Arc<CurveTransactionBuilder>>,
    command_tx: mpsc::Sender<PositionCommand>,
    shutdown_flag: Arc<AtomicBool>,
}

#[derive(Debug, Clone)]
pub struct MonitorConfig {
    pub price_check_interval_secs: u64,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            price_check_interval_secs: 2,
        }
    }
}

impl PositionMonitor {
    pub fn new(
        position_manager: Arc<PositionManager>,
        tx_builder: Arc<TransactionBuilder>,
        event_tx: broadcast::Sender<ArbEvent>,
        command_tx: mpsc::Sender<PositionCommand>,
        config: MonitorConfig,
    ) -> Self {
        Self {
            position_manager,
            tx_builder,
            event_tx,
            config,
            curve_builder: None,
            command_tx,
            shutdown_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn get_shutdown_flag(&self) -> Arc<AtomicBool> {
        self.shutdown_flag.clone()
    }

    pub fn request_shutdown(&self) {
        self.shutdown_flag.store(true, Ordering::SeqCst);
    }

    pub fn with_curve_state_checker(mut self, curve_builder: Arc<CurveTransactionBuilder>) -> Self {
        self.curve_builder = Some(curve_builder);
        self
    }

    pub async fn start_monitoring(&self) {
        info!(
            "Position monitor started (base interval {}s, adaptive)",
            self.config.price_check_interval_secs
        );

        let mut pending_exit_retry_counter: u64 = 0;

        loop {
            if self.shutdown_flag.load(Ordering::SeqCst) {
                info!("Position monitor shutting down gracefully");
                break;
            }

            match self.process_priority_exits().await {
                Ok(_) => {}
                Err(e) => {
                    error!("Priority exit processing error: {}", e);
                }
            }

            pending_exit_retry_counter += 1;
            if pending_exit_retry_counter >= 10 {
                pending_exit_retry_counter = 0;
                match self.retry_pending_exits().await {
                    Ok(_) => {}
                    Err(e) => {
                        error!("Pending exit retry error: {}", e);
                    }
                }
            }

            match self.check_and_process_exits().await {
                Ok(_) => {}
                Err(e) => {
                    error!("Position monitor error: {}", e);
                }
            }

            let interval = self.calculate_adaptive_interval().await;
            tokio::time::sleep(Duration::from_secs(interval)).await;
        }
    }

    async fn retry_pending_exits(&self) -> AppResult<()> {
        let pending_exit_positions = self.position_manager.get_pending_exit_positions().await;

        if pending_exit_positions.is_empty() {
            return Ok(());
        }

        let retry_index = self.position_manager.get_and_increment_retry_index().await;
        let position_index = retry_index % pending_exit_positions.len();
        let position = &pending_exit_positions[position_index];

        info!(
            "Retrying PendingExit {}/{} for {} (cycle {})",
            position_index + 1,
            pending_exit_positions.len(),
            position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8]),
            retry_index
        );

        let (current_price, is_dead_token) = if let Some(ref curve_builder) = self.curve_builder {
            match curve_builder.get_curve_state(&position.token_mint).await {
                Ok(state) if state.virtual_token_reserves > 0 && state.virtual_sol_reserves > 0 => {
                    let price = state.virtual_sol_reserves as f64 / state.virtual_token_reserves as f64;
                    (price, false)
                }
                Ok(state) if state.is_complete => {
                    match self.tx_builder.get_token_price(&position.token_mint, BaseCurrency::Sol).await {
                        Ok(price) if price > 1e-10 => (price, false),
                        Ok(_) => {
                            warn!(
                                "DEAD TOKEN DETECTED (graduated, zero price): {} - skipping retries",
                                position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8])
                            );
                            (0.0, true)
                        }
                        Err(_) => {
                            warn!(
                                "DEAD TOKEN DETECTED (graduated, no DEX price): {} - skipping retries",
                                position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8])
                            );
                            (0.0, true)
                        }
                    }
                }
                Ok(state) if state.virtual_token_reserves == 0 || state.virtual_sol_reserves == 0 => {
                    warn!(
                        "DEAD TOKEN DETECTED (zero reserves): {} - skipping retries",
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
                "Dead token {} - attempting salvage sell with maximum slippage",
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
            "Retrying {} for {} | Price: {:.10} | Entry: {:.10}",
            if is_dead_token { "Salvage" } else { "PendingExit" },
            position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8]),
            signal.current_price,
            position.entry_price
        );

        let cmd = PositionCommand::Exit(ExitCommand::new(signal.clone(), CommandSource::PendingRetry));
        if let Err(e) = self.command_tx.send(cmd).await {
            if is_dead_token {
                warn!(
                    "Salvage sell command failed for {}: {} - marking as orphaned",
                    position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8]),
                    e
                );
                if let Err(e) = self.position_manager.mark_position_orphaned(position.id).await {
                    error!("Failed to mark dead token position as orphaned: {}", e);
                }
            } else {
                warn!(
                    "Pending exit retry command failed for {}: {}",
                    position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8]),
                    e
                );
            }
        } else {
            info!(
                "Queued exit command for {} (source: PendingRetry)",
                position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8])
            );
        }

        Ok(())
    }

    async fn calculate_adaptive_interval(&self) -> u64 {
        let positions = self.position_manager.get_open_positions().await;

        if positions.is_empty() {
            return self.config.price_check_interval_secs;
        }

        let has_at_risk = positions.iter().any(|p|
            p.unrealized_pnl_percent > 10.0 && p.momentum.velocity < 0.0
        );

        let has_profitable = positions.iter().any(|p| p.unrealized_pnl_percent > 5.0);

        if has_at_risk {
            1
        } else if has_profitable {
            2
        } else {
            self.config.price_check_interval_secs
        }
    }

    async fn process_priority_exits(&self) -> AppResult<()> {
        let priority_ids = self.position_manager.drain_priority_exits().await;
        if priority_ids.is_empty() {
            return Ok(());
        }

        info!("Processing {} HIGH PRIORITY exit retries", priority_ids.len());

        for position_id in priority_ids {
            let position = match self.position_manager.get_position(position_id).await {
                Some(p) => p,
                None => {
                    warn!("Priority exit position {} no longer exists", position_id);
                    continue;
                }
            };

            let (current_price, is_dead_token) = if let Some(curve_builder) = &self.curve_builder {
                match curve_builder.get_curve_state(&position.token_mint).await {
                    Ok(state) if state.virtual_token_reserves > 0 && state.virtual_sol_reserves > 0 => {
                        let price = state.virtual_sol_reserves as f64 / state.virtual_token_reserves as f64;
                        (price, false)
                    }
                    Ok(state) if state.is_complete => {
                        match self.tx_builder.get_token_price(&position.token_mint, BaseCurrency::Sol).await {
                            Ok(price) if price > 1e-10 => (price, false),
                            _ => {
                                warn!(
                                    "DEAD TOKEN (priority): {} - graduated but no DEX liquidity",
                                    position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8])
                                );
                                (0.0, true)
                            }
                        }
                    }
                    Ok(_) => {
                        warn!(
                            "DEAD TOKEN (priority): {} - zero reserves",
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
                    "Dead token {} in priority queue - attempting salvage sell",
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
                "{}: {} | Using {} slippage",
                log_prefix,
                position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8]),
                if is_dead_token { "SALVAGE (90%)" } else { "EMERGENCY" }
            );

            let cmd = PositionCommand::Exit(ExitCommand::new(signal, CommandSource::PriorityRetry));
            if let Err(e) = self.command_tx.send(cmd).await {
                if is_dead_token {
                    error!(
                        "Salvage sell command failed for {}: {} - marking as orphaned",
                        position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8]),
                        e
                    );
                    if let Err(e) = self.position_manager.mark_position_orphaned(position_id).await {
                        error!("Failed to mark priority dead token as orphaned: {}", e);
                    }
                } else {
                    error!(
                        "Priority exit command failed for {}: {}",
                        position_id, e
                    );
                    self.position_manager
                        .record_priority_exit_failure(position_id, false)
                        .await;
                }
            }
        }

        Ok(())
    }

    async fn check_and_process_exits(&self) -> AppResult<()> {
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

        let mut prices = match tokio::time::timeout(
            Duration::from_secs(PRICE_FETCH_TIMEOUT_SECS),
            self.tx_builder.get_multiple_token_prices(&unique_mints, BaseCurrency::Sol)
        ).await {
            Ok(Ok(p)) => p,
            Ok(Err(e)) => {
                debug!("Jupiter price fetch failed (expected for pre-grad tokens): {}", e);
                std::collections::HashMap::new()
            }
            Err(_) => {
                warn!("Jupiter price fetch timed out after {}s", PRICE_FETCH_TIMEOUT_SECS);
                std::collections::HashMap::new()
            }
        };

        if let Some(curve_builder) = &self.curve_builder {
            let curve_fetch_start = std::time::Instant::now();
            let global_deadline = curve_fetch_start + Duration::from_secs(GLOBAL_PRICE_FETCH_TIMEOUT_SECS);

            for mint in &unique_mints {
                if std::time::Instant::now() > global_deadline {
                    warn!(
                        "Global price fetch timeout reached after {}s - {} mints may have stale prices",
                        GLOBAL_PRICE_FETCH_TIMEOUT_SECS,
                        unique_mints.len() - prices.len()
                    );
                    break;
                }

                if !prices.contains_key(mint) {
                    match tokio::time::timeout(
                        Duration::from_secs(PRICE_FETCH_TIMEOUT_SECS),
                        curve_builder.get_curve_state(mint)
                    ).await {
                        Ok(Ok(state)) => {
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
                        Ok(Err(e)) => {
                            debug!(mint = %mint, error = %e, "Failed to fetch curve state for price");
                        }
                        Err(_) => {
                            debug!(mint = %mint, "Curve state fetch timed out");
                        }
                    }
                }
            }
        }

        for position in &positions {
            if !prices.contains_key(&position.token_mint) {
                let is_dead_token = position.exit_config.custom_exit_instructions
                    .as_ref()
                    .map(|s| s.contains("DEAD TOKEN"))
                    .unwrap_or(false);

                let position_age_secs = (Utc::now() - position.entry_time).num_seconds() as u64;
                let price_is_stale = position_age_secs > MAX_STALE_PRICE_SECS;

                if is_dead_token {
                    info!(
                        "Using fallback price for dead token {} (no market price available)",
                        &position.token_mint[..12]
                    );
                    prices.insert(position.token_mint.clone(), position.entry_price);
                } else if position.exit_config.time_limit_minutes.is_some() {
                    if price_is_stale {
                        warn!(
                            "Using STALE fallback price for {} (position {}s old, no fresh price) - only time exits will trigger",
                            &position.token_mint[..12],
                            position_age_secs
                        );
                    } else {
                        debug!(
                            "Using fallback price for {} (time limit exit pending)",
                            &position.token_mint[..12]
                        );
                    }
                    prices.insert(position.token_mint.clone(), position.current_price);
                } else if price_is_stale {
                    warn!(
                        "Skipping {} - no fresh price available (position {}s old) and no time limit set",
                        &position.token_mint[..12],
                        position_age_secs
                    );
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

        info!("{} exit signals triggered", all_signals.len());

        for signal in all_signals {
            self.emit_exit_signal_event(&signal).await;

            let cmd = PositionCommand::Exit(ExitCommand::new(signal.clone(), CommandSource::Monitor));
            if let Err(e) = self.command_tx.send(cmd).await {
                error!(
                    "Failed to queue exit command for position {}: {}",
                    signal.position_id, e,
                );
                self.position_manager
                    .record_priority_exit_failure(signal.position_id, false)
                    .await;

                warn!(
                    "Exit signal for {} queued for priority retry",
                    signal.position_id
                );
            }
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

        if let Err(e) = self.event_tx.send(event) {
            tracing::warn!("Event broadcast failed (channel full/closed): {}", e);
        }
    }

    pub async fn trigger_manual_exit(
        &self,
        position_id: Uuid,
        exit_percent: f64,
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

        let cmd = PositionCommand::Exit(ExitCommand::new(signal, CommandSource::ManualTrigger));
        self.command_tx.send(cmd).await.map_err(|e| {
            crate::error::AppError::Internal(format!("Failed to queue manual exit: {}", e))
        })
    }

    pub async fn trigger_exit_with_reason(
        &self,
        signal: &ExitSignal,
    ) -> AppResult<()> {
        let cmd = PositionCommand::Exit(ExitCommand::new(signal.clone(), CommandSource::Monitor));
        self.command_tx.send(cmd).await.map_err(|e| {
            crate::error::AppError::Internal(format!("Failed to queue exit: {}", e))
        })
    }

    pub async fn get_position_stats(&self) -> super::position_manager::PositionManagerStats {
        self.position_manager.get_stats().await
    }
}
