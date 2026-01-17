use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::error::AppResult;
use crate::events::{AgentType, ArbEvent, EventSource};
use crate::wallet::turnkey::SignRequest;
use crate::wallet::DevWalletSigner;

use super::jito::{BundleState, JitoClient};
use super::position_manager::{
    BaseCurrency, ExitReason, ExitSignal, ExitUrgency, OpenPosition, PositionManager,
    PositionStatus,
};
use super::transaction_builder::TransactionBuilder;

pub struct PositionMonitor {
    position_manager: Arc<PositionManager>,
    tx_builder: Arc<TransactionBuilder>,
    jito_client: Arc<JitoClient>,
    event_tx: broadcast::Sender<ArbEvent>,
    config: MonitorConfig,
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
            exit_slippage_bps: 100,
            max_exit_retries: 3,
            emergency_slippage_bps: 300,
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
        }
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

        let prices = match self
            .tx_builder
            .get_multiple_token_prices(&unique_mints, BaseCurrency::Sol)
            .await
        {
            Ok(p) => p,
            Err(e) => {
                warn!("Failed to fetch prices: {}", e);
                return Ok(());
            }
        };

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
            "📤 Processing {} exit for {} | {} @ {} | slippage: {} bps",
            format!("{:?}", signal.reason),
            position
                .token_symbol
                .as_deref()
                .unwrap_or(&position.token_mint[..8]),
            signal.exit_percent,
            signal.current_price,
            slippage
        );

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
                let entry_value =
                    position.entry_token_amount * position.entry_price;
                let exit_value = position.entry_token_amount * exit_price;
                let realized_pnl = exit_value - entry_value;

                self.position_manager
                    .close_position(
                        signal.position_id,
                        exit_price,
                        realized_pnl,
                        sign_result.signature.clone(),
                    )
                    .await?;

                self.emit_exit_completed_event(
                    &position,
                    signal,
                    realized_pnl,
                    sign_result.signature.as_deref(),
                )
                .await;

                info!(
                    "✅ Exit completed: {} | P&L: {:.6} {} | Reason: {:?}",
                    position
                        .token_symbol
                        .as_deref()
                        .unwrap_or(&position.token_mint[..8]),
                    realized_pnl / 1e9,
                    position.exit_config.base_currency.symbol(),
                    signal.reason
                );
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
        realized_pnl: f64,
        tx_signature: Option<&str>,
    ) {
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
                "realized_pnl": realized_pnl,
                "realized_pnl_sol": realized_pnl / 1e9,
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
