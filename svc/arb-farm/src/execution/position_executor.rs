use chrono::Utc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc, Mutex};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::database::{CreateTradeRecord, TradeRepository};
use crate::engrams::schemas::{TransactionAction, TransactionMetadata, TransactionSummary};
use crate::engrams::EngramsClient;
use crate::error::{AppError, AppResult};
use crate::events::{topics, AgentType, ArbEvent, EventSource};
use crate::helius::{HeliusClient, HeliusSender};
use crate::wallet::turnkey::SignRequest;
use crate::wallet::DevWalletSigner;

use super::tx_settlement::{resolve_inferred_settlement, resolve_settlement, TxSettlement};

use super::capital_manager::CapitalManager;
use super::curve_builder::{CurveSellParams, CurveTransactionBuilder};
use super::jito::{BundleState, JitoClient};
use super::position_command::{CommandSource, ExitCommand, PositionCommand};
use super::position_manager::{
    ExitReason, ExitSignal, ExitUrgency, OpenPosition, PositionManager, PositionStatus,
};
use super::transaction_builder::TransactionBuilder;

const MIN_DUST_VALUE_SOL: f64 = 0.0001;

#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    pub exit_slippage_bps: u16,
    pub max_exit_retries: u32,
    pub emergency_slippage_bps: u16,
    pub bundle_timeout_secs: u64,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            exit_slippage_bps: 1500,
            max_exit_retries: 3,
            emergency_slippage_bps: 2500,
            bundle_timeout_secs: 60,
        }
    }
}

pub struct PositionExecutor {
    command_rx: Mutex<mpsc::Receiver<PositionCommand>>,
    position_manager: Arc<PositionManager>,
    tx_builder: Arc<TransactionBuilder>,
    jito_client: Arc<JitoClient>,
    event_tx: broadcast::Sender<ArbEvent>,
    curve_builder: Option<Arc<CurveTransactionBuilder>>,
    helius_sender: Option<Arc<HeliusSender>>,
    helius_client: Option<Arc<HeliusClient>>,
    engrams_client: Option<Arc<EngramsClient>>,
    trade_repo: Option<Arc<TradeRepository>>,
    capital_manager: Option<Arc<CapitalManager>>,
    signer: Arc<DevWalletSigner>,
    rate_limit_backoff_until: Arc<tokio::sync::RwLock<Option<std::time::Instant>>>,
    consecutive_rate_limits: Arc<tokio::sync::RwLock<u32>>,
    shutdown_flag: Arc<AtomicBool>,
    config: ExecutorConfig,
}

impl PositionExecutor {
    pub fn new(
        command_rx: mpsc::Receiver<PositionCommand>,
        position_manager: Arc<PositionManager>,
        tx_builder: Arc<TransactionBuilder>,
        jito_client: Arc<JitoClient>,
        event_tx: broadcast::Sender<ArbEvent>,
        signer: Arc<DevWalletSigner>,
        config: ExecutorConfig,
    ) -> Self {
        Self {
            command_rx: Mutex::new(command_rx),
            position_manager,
            tx_builder,
            jito_client,
            event_tx,
            curve_builder: None,
            helius_sender: None,
            helius_client: None,
            engrams_client: None,
            trade_repo: None,
            capital_manager: None,
            signer,
            rate_limit_backoff_until: Arc::new(tokio::sync::RwLock::new(None)),
            consecutive_rate_limits: Arc::new(tokio::sync::RwLock::new(0)),
            shutdown_flag: Arc::new(AtomicBool::new(false)),
            config,
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

    pub fn with_trade_repo(mut self, trade_repo: Arc<TradeRepository>) -> Self {
        self.trade_repo = Some(trade_repo);
        self
    }

    pub fn with_capital_manager(mut self, capital_manager: Arc<CapitalManager>) -> Self {
        self.capital_manager = Some(capital_manager);
        self
    }

    pub fn with_helius_client(mut self, helius_client: Arc<HeliusClient>) -> Self {
        self.helius_client = Some(helius_client);
        self
    }

    pub fn get_shutdown_flag(&self) -> Arc<AtomicBool> {
        self.shutdown_flag.clone()
    }

    pub async fn run(self: Arc<Self>) {
        info!("PositionExecutor started - listening for commands");

        loop {
            if self.shutdown_flag.load(Ordering::Relaxed) {
                info!("PositionExecutor shutting down");
                break;
            }

            let mut commands = Vec::new();

            {
                let mut rx = self.command_rx.lock().await;
                match tokio::time::timeout(Duration::from_secs(1), rx.recv()).await {
                    Ok(Some(cmd)) => {
                        commands.push(cmd);
                        while let Ok(cmd) = rx.try_recv() {
                            commands.push(cmd);
                        }
                    }
                    Ok(None) => {
                        info!("PositionExecutor command channel closed");
                        break;
                    }
                    Err(_) => continue,
                }
            }

            if commands.is_empty() {
                continue;
            }

            commands.sort_by_key(|cmd| cmd.urgency_sort_key());

            let deduped = Self::dedup_commands(commands);

            info!("PositionExecutor processing {} commands", deduped.len());

            for cmd in deduped {
                self.handle_command(cmd).await;
            }
        }
    }

    fn dedup_commands(commands: Vec<PositionCommand>) -> Vec<PositionCommand> {
        let mut seen = std::collections::HashSet::new();
        let mut result = Vec::new();
        for cmd in commands {
            if seen.insert(cmd.position_id()) {
                result.push(cmd);
            }
        }
        result
    }

    async fn handle_command(&self, cmd: PositionCommand) {
        match cmd {
            PositionCommand::Exit(exit_cmd) => {
                let position_id = exit_cmd.signal.position_id;
                let source = exit_cmd.source.clone();
                info!(
                    position_id = %position_id,
                    source = %source,
                    reason = ?exit_cmd.signal.reason,
                    urgency = ?exit_cmd.signal.urgency,
                    "Executing exit command"
                );

                if let Err(e) = self.execute_exit(&exit_cmd.signal).await {
                    error!(
                        position_id = %position_id,
                        source = %source,
                        error = %e,
                        "Exit command failed"
                    );
                }
            }
        }
    }

    async fn is_rate_limited(&self) -> bool {
        self.is_rate_limited_for_urgency(ExitUrgency::Low).await
    }

    async fn is_rate_limited_for_urgency(&self, urgency: ExitUrgency) -> bool {
        if matches!(urgency, ExitUrgency::Critical | ExitUrgency::High) {
            return false;
        }

        let backoff_until = self.rate_limit_backoff_until.read().await;
        if let Some(until) = *backoff_until {
            if std::time::Instant::now() < until {
                return true;
            }
        }
        false
    }

    async fn record_rate_limit(&self) {
        const MAX_CONSECUTIVE_RATE_LIMITS: u32 = 10;
        const MAX_BACKOFF_SECS: u64 = 60;

        let mut consecutive = self.consecutive_rate_limits.write().await;
        *consecutive = (*consecutive + 1).min(MAX_CONSECUTIVE_RATE_LIMITS);

        let backoff_secs = (5u64 * (1 << (*consecutive - 1).min(4))).min(MAX_BACKOFF_SECS);
        let jitter = (backoff_secs as f64 * 0.2 * rand::random::<f64>()) as u64;
        let final_backoff = backoff_secs + jitter;

        let mut backoff_until = self.rate_limit_backoff_until.write().await;
        *backoff_until = Some(std::time::Instant::now() + Duration::from_secs(final_backoff));

        warn!(
            "Rate limit detected (consecutive: {}/{}) - backing off for {}s (with jitter)",
            *consecutive, MAX_CONSECUTIVE_RATE_LIMITS, final_backoff
        );
    }

    async fn clear_rate_limit(&self) {
        let mut consecutive = self.consecutive_rate_limits.write().await;
        if *consecutive > 0 {
            debug!(
                "Rate limit cleared after {} consecutive rate limits",
                *consecutive
            );
            *consecutive = 0;
        }

        let mut backoff_until = self.rate_limit_backoff_until.write().await;
        *backoff_until = None;
    }

    fn calculate_profit_aware_slippage(&self, position: &OpenPosition, signal: &ExitSignal) -> u16 {
        const MIN_SLIPPAGE_BPS: u16 = 500;
        const MAX_SLIPPAGE_BPS: u16 = 2000;
        const SALVAGE_SLIPPAGE_BPS: u16 = 5000;
        const ABSOLUTE_MAX_SLIPPAGE_BPS: u16 = 5000;
        const PROFIT_SACRIFICE_RATIO: f64 = 0.25;

        let is_dead_token = signal.reason == ExitReason::Salvage
            || position
                .exit_config
                .custom_exit_instructions
                .as_ref()
                .map(|s| s.contains("DEAD TOKEN"))
                .unwrap_or(false);

        if is_dead_token {
            let salvage_slippage = SALVAGE_SLIPPAGE_BPS.min(ABSOLUTE_MAX_SLIPPAGE_BPS);
            info!(
                "Dead token exit: using salvage slippage {}bps ({}%)",
                salvage_slippage,
                salvage_slippage as f64 / 100.0
            );
            return salvage_slippage;
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
            "Slippage calc: PnL={:.2}% | base={}bps | urgency={:.1}x | final={}bps",
            pnl_percent,
            calculated_slippage,
            urgency_multiplier,
            final_slippage.min(MAX_SLIPPAGE_BPS)
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

        if let Err(e) = engrams_client
            .save_transaction_summary(wallet_address, &tx_summary)
            .await
        {
            warn!("Failed to save exit transaction summary engram: {}", e);
        } else {
            info!(
                "Saved exit transaction summary engram for {} (PnL: {:.6} SOL)",
                &tx_signature.unwrap_or("unknown")
                    [..12.min(tx_signature.unwrap_or("unknown").len())],
                realized_pnl_sol
            );
        }
    }

    async fn save_sell_trade_record(
        &self,
        position: &OpenPosition,
        signal: &ExitSignal,
        realized_pnl_sol: f64,
        tx_signature: Option<&str>,
        slippage_bps: u16,
        settlement: Option<&TxSettlement>,
    ) {
        let Some(repo) = &self.trade_repo else { return };

        let exit_price_decimal = rust_decimal::Decimal::try_from(signal.current_price).ok();
        let entry_price_decimal = rust_decimal::Decimal::try_from(position.entry_price).ok();
        let profit_lamports = (realized_pnl_sol * 1_000_000_000.0) as i64;

        let trade_record = CreateTradeRecord {
            edge_id: Some(position.edge_id),
            strategy_id: Some(position.strategy_id),
            tx_signature: tx_signature.map(|s| s.to_string()),
            bundle_id: None,
            entry_price: entry_price_decimal,
            exit_price: exit_price_decimal,
            profit_lamports: Some(profit_lamports),
            gas_cost_lamports: settlement.map(|s| s.gas_lamports as i64),
            slippage_bps: Some(slippage_bps as i32),
            entry_gas_lamports: None,
            exit_gas_lamports: settlement.map(|s| s.gas_lamports as i64),
            pnl_source: Some(
                settlement
                    .map(|s| s.source.to_string())
                    .unwrap_or_else(|| "estimated".to_string()),
            ),
        };

        if let Err(e) = repo.create(trade_record).await {
            warn!("Failed to save sell trade record to DB: {}", e);
        } else {
            debug!(
                "Saved sell trade record to arb_trades for {} (PnL: {:.6} SOL, source: {})",
                position
                    .token_symbol
                    .as_deref()
                    .unwrap_or(&position.token_mint[..8]),
                realized_pnl_sol,
                settlement.map(|s| s.source).unwrap_or("estimated"),
            );
        }
    }

    async fn resolve_sell_settlement(
        &self,
        signature: &str,
        user_wallet: &str,
    ) -> Option<TxSettlement> {
        let helius_client = self.helius_client.as_ref()?;
        let settlement = resolve_settlement(helius_client, signature, user_wallet).await;
        Some(settlement)
    }

    fn compute_pnl_with_settlement(
        &self,
        settlement: Option<&TxSettlement>,
        estimated_pnl: f64,
        position: &OpenPosition,
    ) -> f64 {
        if let Some(s) = settlement {
            if s.source == "onchain" || s.source == "inferred-onchain" {
                let onchain_pnl = s.sol_delta_sol();
                let diff = (onchain_pnl - estimated_pnl).abs();
                if diff > 0.0001 {
                    info!(
                        "[PnL correction] {} | estimated={:.6} SOL | {}={:.6} SOL | diff={:.6} SOL",
                        position
                            .token_symbol
                            .as_deref()
                            .unwrap_or(&position.token_mint[..8]),
                        estimated_pnl,
                        s.source,
                        onchain_pnl,
                        onchain_pnl - estimated_pnl,
                    );
                }
                return onchain_pnl;
            }
        }
        estimated_pnl
    }

    async fn execute_exit(&self, signal: &ExitSignal) -> AppResult<()> {
        let position = match self.position_manager.get_position(signal.position_id).await {
            Some(p) => p,
            None => {
                warn!("Position {} no longer exists", signal.position_id);
                return Ok(());
            }
        };

        let cas_succeeded = match position.status {
            PositionStatus::Open => {
                self.position_manager
                    .transition_to_pending_exit(signal.position_id)
                    .await
            }
            PositionStatus::PartiallyExited => self
                .position_manager
                .compare_and_swap_status(
                    signal.position_id,
                    PositionStatus::PartiallyExited,
                    PositionStatus::PendingExit,
                )
                .await
                .unwrap_or_default(),
            PositionStatus::PendingExit => {
                debug!(
                    "Position {} already in PendingExit, proceeding with exit",
                    signal.position_id
                );
                true
            }
            PositionStatus::Closed | PositionStatus::Failed | PositionStatus::Orphaned => {
                debug!(
                    "Position {} already closed/failed/orphaned, skipping exit",
                    signal.position_id
                );
                return Ok(());
            }
        };

        if !cas_succeeded {
            warn!(
                "Position {} status CAS failed - another thread is handling this exit",
                signal.position_id
            );
            return Ok(());
        }

        let wallet_status = self.signer.get_status().await;
        let user_wallet = match &wallet_status.wallet_address {
            Some(addr) => addr.clone(),
            None => {
                error!(
                    position_id = %signal.position_id,
                    token = %position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8]),
                    "No wallet configured for exit - cannot process exit signal"
                );
                return Err(AppError::Internal(
                    "No wallet configured for exit - position exit cannot proceed".to_string(),
                ));
            }
        };

        let slippage = self.calculate_profit_aware_slippage(&position, signal);

        info!(
            "Processing {} exit for {} | {}% @ {} | slippage: {} bps",
            format!("{:?}", signal.reason),
            position
                .token_symbol
                .as_deref()
                .unwrap_or(&position.token_mint[..8]),
            signal.exit_percent,
            signal.current_price,
            slippage
        );

        let use_curve_sell = if let Some(ref curve_builder) = self.curve_builder {
            match curve_builder.get_curve_state(&position.token_mint).await {
                Ok(state) => {
                    if !state.is_complete {
                        info!(
                            "Token {} still on bonding curve ({:.2}% progress), using curve sell",
                            &position.token_mint[..8],
                            state.graduation_progress()
                        );
                        true
                    } else {
                        info!(
                            "Token {} graduated, using DEX sell",
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

        if use_curve_sell {
            return self
                .execute_curve_exit(&position, signal, &user_wallet, slippage)
                .await;
        }

        let (exit_tx_base64, expected_base_out, token_amount_in, route_label) = if let Some(
            ref curve_builder,
        ) =
            self.curve_builder
        {
            let token_balance = self
                .tx_builder
                .get_token_balance(&user_wallet, &position.token_mint)
                .await?;

            let token_value_sol = token_balance as f64 * signal.current_price;
            if token_value_sol < MIN_DUST_VALUE_SOL {
                warn!(
                        "Skipping exit for {} - value {:.6} SOL below dust threshold {:.4} SOL (balance: {} tokens)",
                        position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8]),
                        token_value_sol,
                        MIN_DUST_VALUE_SOL,
                        token_balance
                    );
                self.position_manager
                    .close_position(
                        signal.position_id,
                        signal.current_price,
                        -position.entry_amount_base,
                        "DustBalance",
                        None,
                        Some(position.momentum.momentum_score),
                    )
                    .await?;
                return Ok(());
            }

            let exit_amount = if signal.exit_percent >= 100.0 {
                token_balance
            } else {
                ((token_balance as f64) * (signal.exit_percent / 100.0))
                    .round()
                    .min(token_balance as f64) as u64
            };

            let exit_value_sol = exit_amount as f64 * signal.current_price;
            if exit_value_sol < MIN_DUST_VALUE_SOL {
                warn!(
                    "Calculated exit value {:.6} SOL below dust threshold for {}",
                    exit_value_sol,
                    position
                        .token_symbol
                        .as_deref()
                        .unwrap_or(&position.token_mint[..8])
                );
                return Err(AppError::Execution(format!(
                    "Exit value {:.6} SOL below dust threshold {:.4} SOL",
                    exit_value_sol, MIN_DUST_VALUE_SOL
                )));
            }

            let sell_params = CurveSellParams {
                mint: position.token_mint.clone(),
                token_amount: exit_amount,
                slippage_bps: slippage,
                user_wallet: user_wallet.clone(),
            };

            info!(
                "Graduated token exit for {} - trying Raydium first",
                position
                    .token_symbol
                    .as_deref()
                    .unwrap_or(&position.token_mint[..8])
            );

            match curve_builder.build_raydium_sell(&sell_params).await {
                Ok(raydium_result) => {
                    info!(
                        "Built Raydium exit tx for {}: expected {} SOL, impact {:.2}%",
                        position
                            .token_symbol
                            .as_deref()
                            .unwrap_or(&position.token_mint[..8]),
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
                        "Raydium exit failed for {}: {}, falling back to Jupiter",
                        position
                            .token_symbol
                            .as_deref()
                            .unwrap_or(&position.token_mint[..8]),
                        raydium_err
                    );
                    let exit_build = self
                        .tx_builder
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
            let exit_build = self
                .tx_builder
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

        let sign_result = self.signer.sign_transaction(sign_request).await?;

        if !sign_result.success {
            let error_msg = sign_result
                .error
                .or_else(|| sign_result.policy_violation.map(|v| v.message))
                .unwrap_or_else(|| "Unknown signing error".to_string());
            error!(
                position_id = %signal.position_id,
                token = %position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8]),
                reason = ?signal.reason,
                "Exit signing failed: {}", error_msg
            );
            return Err(AppError::ExternalApi(format!(
                "Signing failed: {}",
                error_msg
            )));
        }

        let signed_tx = match sign_result.signed_transaction_base64 {
            Some(tx) => tx,
            None => {
                error!(
                    position_id = %signal.position_id,
                    token = %position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8]),
                    "No signed transaction returned for exit"
                );
                return Err(AppError::ExternalApi(
                    "No signed transaction returned".to_string(),
                ));
            }
        };

        let tip = 10_000;
        let tx_base58 = base64_to_base58(&signed_tx)?;

        let mut use_helius_fallback = false;
        let mut helius_signature: Option<String> = None;

        match self.jito_client.send_bundle(vec![tx_base58], tip).await {
            Ok(bundle_result) => {
                let bundle_id = bundle_result.id.to_string();
                info!("Exit bundle submitted: {}", bundle_id);

                match self
                    .jito_client
                    .wait_for_bundle(&bundle_id, self.config.bundle_timeout_secs)
                    .await
                {
                    Ok(status) => match status.status {
                        BundleState::Landed => {}
                        BundleState::Failed | BundleState::Dropped | BundleState::Pending => {
                            warn!(
                                "Jito bundle {} status: {:?} - trying Helius fallback",
                                bundle_id, status.status
                            );
                            use_helius_fallback = true;
                        }
                    },
                    Err(e) => {
                        warn!(
                            position_id = %signal.position_id,
                            token = %position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8]),
                            bundle_id = %bundle_id,
                            "Jito bundle wait failed: {} - trying Helius fallback", e
                        );
                        use_helius_fallback = true;
                    }
                }
            }
            Err(e) => {
                warn!(
                    position_id = %signal.position_id,
                    token = %position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8]),
                    "Jito bundle send failed: {} - trying Helius fallback", e
                );
                use_helius_fallback = true;
            }
        }

        if use_helius_fallback {
            if let Some(helius_sender) = &self.helius_sender {
                info!("Sending DEX exit via Helius fallback...");
                let confirmation_timeout = std::time::Duration::from_secs(30);
                match helius_sender
                    .send_and_confirm(&signed_tx, confirmation_timeout)
                    .await
                {
                    Ok(sig) => {
                        info!("DEX exit confirmed via Helius: {}", sig);
                        helius_signature = Some(sig);
                    }
                    Err(e) => {
                        let error_str = e.to_string();

                        if error_str.contains("Timeout") || error_str.contains("timeout") {
                            match self
                                .tx_builder
                                .get_token_balance(&user_wallet, &position.token_mint)
                                .await
                            {
                                Ok(balance) => {
                                    if balance < 1000 {
                                        tokio::time::sleep(Duration::from_secs(2)).await;
                                        let recheck_balance = self
                                            .tx_builder
                                            .get_token_balance(&user_wallet, &position.token_mint)
                                            .await
                                            .unwrap_or(0);

                                        if recheck_balance < 1000 {
                                            warn!(
                                                "Exit confirmation timed out but token balance is 0 for {} (verified twice) - inferring successful exit (NO REAL SIGNATURE)",
                                                position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8])
                                            );
                                            helius_signature = Some(format!(
                                                "INFERRED_EXIT_{}_{}_balance_zero",
                                                signal.position_id.to_string()[..8].to_string(),
                                                chrono::Utc::now().timestamp()
                                            ));
                                        } else {
                                            warn!("Balance changed between checks ({} -> {}) - queuing for retry", balance, recheck_balance);
                                            self.emit_exit_failed_event(
                                                &position,
                                                signal,
                                                "Balance inconsistent",
                                            )
                                            .await;
                                            if let Err(reset_err) = self
                                                .position_manager
                                                .reset_position_status(signal.position_id)
                                                .await
                                            {
                                                error!(
                                                    "Failed to reset position status: {}",
                                                    reset_err
                                                );
                                            } else {
                                                self.position_manager
                                                    .queue_priority_exit(signal.position_id)
                                                    .await;
                                            }
                                            return Err(AppError::ExternalApi(
                                                "Balance inconsistent between checks".to_string(),
                                            ));
                                        }
                                    } else {
                                        warn!(
                                            position_id = %signal.position_id,
                                            token = %position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8]),
                                            remaining_balance = balance,
                                            "Confirmation timed out and tokens still in wallet - will retry"
                                        );
                                        self.emit_exit_failed_event(&position, signal, &error_str)
                                            .await;
                                        if let Err(reset_err) = self
                                            .position_manager
                                            .reset_position_status(signal.position_id)
                                            .await
                                        {
                                            error!(position_id = %signal.position_id, "Failed to reset position status: {}", reset_err);
                                        } else {
                                            self.position_manager
                                                .queue_priority_exit(signal.position_id)
                                                .await;
                                            info!(
                                                "Position {} queued for HIGH PRIORITY retry",
                                                signal.position_id
                                            );
                                        }
                                        return Err(AppError::ExternalApi(format!(
                                            "Sell timed out, tokens still in wallet ({})",
                                            balance
                                        )));
                                    }
                                }
                                Err(balance_err) => {
                                    warn!(
                                        position_id = %signal.position_id,
                                        token = %position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8]),
                                        "Could not verify balance after timeout: {} - will retry", balance_err
                                    );
                                    self.emit_exit_failed_event(&position, signal, &error_str)
                                        .await;
                                    if let Err(reset_err) = self
                                        .position_manager
                                        .reset_position_status(signal.position_id)
                                        .await
                                    {
                                        error!(position_id = %signal.position_id, "Failed to reset position status: {}", reset_err);
                                    } else {
                                        self.position_manager
                                            .queue_priority_exit(signal.position_id)
                                            .await;
                                        info!(
                                            "Position {} queued for HIGH PRIORITY retry",
                                            signal.position_id
                                        );
                                    }
                                    return Err(AppError::ExternalApi(format!(
                                        "Sell timed out, could not verify: {}",
                                        balance_err
                                    )));
                                }
                            }
                        } else {
                            error!(
                                position_id = %signal.position_id,
                                token = %position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8]),
                                "Helius fallback also failed: {}", error_str
                            );
                            self.emit_exit_failed_event(&position, signal, &error_str)
                                .await;
                            if let Err(reset_err) = self
                                .position_manager
                                .reset_position_status(signal.position_id)
                                .await
                            {
                                error!(
                                    position_id = %signal.position_id,
                                    "Failed to reset position status: {}", reset_err
                                );
                            } else {
                                self.position_manager
                                    .queue_priority_exit(signal.position_id)
                                    .await;
                                info!(
                                    "Position {} queued for HIGH PRIORITY retry",
                                    signal.position_id
                                );
                            }
                            return Err(AppError::ExternalApi(format!(
                                "Helius fallback failed: {}",
                                error_str
                            )));
                        }
                    }
                }
            } else {
                error!(
                    position_id = %signal.position_id,
                    token = %position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8]),
                    "No Helius sender available for fallback"
                );
                self.emit_exit_failed_event(&position, signal, "No Helius fallback available")
                    .await;
                return Err(AppError::ExternalApi(
                    "No Helius sender available for fallback".to_string(),
                ));
            }
        }

        let final_signature = helius_signature.or(sign_result.signature.clone());
        {
            let settlement = if let Some(ref sig) = final_signature {
                self.resolve_sell_settlement(sig, &user_wallet).await
            } else {
                None
            };

            let exit_price = signal.current_price;
            let pnl_percent = if position.entry_price > 0.0 {
                (exit_price - position.entry_price) / position.entry_price
            } else {
                0.0
            };
            let exit_reason = format!("{:?}", signal.reason);

            let effective_base = if position.remaining_amount_base > 0.0 {
                position.remaining_amount_base
            } else {
                position.entry_amount_base
            };

            let is_partial_exit = signal.exit_percent < 100.0;

            if is_partial_exit {
                let partial_base = effective_base * (signal.exit_percent / 100.0);
                let estimated_pnl = partial_base * pnl_percent;
                let realized_pnl_sol =
                    self.compute_pnl_with_settlement(settlement.as_ref(), estimated_pnl, &position);

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

                if let Some(capital_mgr) = &self.capital_manager {
                    if let Some(released) = capital_mgr
                        .release_partial_capital(signal.position_id, signal.exit_percent)
                        .await
                    {
                        debug!(
                            "Released {} SOL partial capital for position {} ({}% exit)",
                            released as f64 / 1_000_000_000.0,
                            signal.position_id,
                            signal.exit_percent
                        );
                    }
                }

                self.emit_exit_completed_event(
                    &position,
                    signal,
                    realized_pnl_sol,
                    final_signature.as_deref(),
                )
                .await;

                info!(
                        "Partial exit completed: {} | {}% exited | P&L: {:.6} {} ({:.2}%) | Reason: {:?} | source: {}",
                        position
                            .token_symbol
                            .as_deref()
                            .unwrap_or(&position.token_mint[..8]),
                        signal.exit_percent,
                        realized_pnl_sol,
                        position.exit_config.base_currency.symbol(),
                        pnl_percent * 100.0,
                        signal.reason,
                        settlement.as_ref().map(|s| s.source).unwrap_or("estimated"),
                    );

                self.save_exit_to_engrams(
                    &position,
                    signal,
                    realized_pnl_sol,
                    final_signature.as_deref(),
                    &user_wallet,
                )
                .await;

                self.save_sell_trade_record(
                    &position,
                    signal,
                    realized_pnl_sol,
                    final_signature.as_deref(),
                    slippage,
                    settlement.as_ref(),
                )
                .await;
            } else {
                let estimated_pnl = effective_base * pnl_percent;
                let realized_pnl_sol =
                    self.compute_pnl_with_settlement(settlement.as_ref(), estimated_pnl, &position);

                self.position_manager
                    .close_position(
                        signal.position_id,
                        exit_price,
                        realized_pnl_sol,
                        &exit_reason,
                        final_signature.clone(),
                        Some(position.momentum.momentum_score),
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
                    "Exit completed: {} | P&L: {:.6} {} ({:.2}%) | Reason: {:?} | source: {}",
                    position
                        .token_symbol
                        .as_deref()
                        .unwrap_or(&position.token_mint[..8]),
                    realized_pnl_sol,
                    position.exit_config.base_currency.symbol(),
                    pnl_percent * 100.0,
                    signal.reason,
                    settlement.as_ref().map(|s| s.source).unwrap_or("estimated"),
                );

                self.save_exit_to_engrams(
                    &position,
                    signal,
                    realized_pnl_sol,
                    final_signature.as_deref(),
                    &user_wallet,
                )
                .await;

                self.save_sell_trade_record(
                    &position,
                    signal,
                    realized_pnl_sol,
                    final_signature.as_deref(),
                    slippage,
                    settlement.as_ref(),
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
        user_wallet: &str,
        initial_slippage: u16,
    ) -> AppResult<()> {
        let curve_builder = self
            .curve_builder
            .as_ref()
            .ok_or_else(|| AppError::Internal("Curve builder not configured".into()))?;
        let helius_sender = self
            .helius_sender
            .as_ref()
            .ok_or_else(|| AppError::Internal("Helius sender not configured".into()))?;

        let actual_balance = curve_builder
            .get_actual_token_balance(user_wallet, &position.token_mint)
            .await
            .unwrap_or(0);

        if actual_balance == 0 {
            warn!(
                "INFERRED CLOSE: Token {} has zero on-chain balance - sold externally or transferred. Entry: {:.6} SOL",
                &position.token_mint[..8],
                position.entry_amount_base,
            );

            let effective_base = if position.remaining_amount_base > 0.0 {
                position.remaining_amount_base
            } else {
                position.entry_amount_base
            };

            let (realized_pnl, pnl_source) = if let Some(helius) = &self.helius_client {
                let settlement = resolve_inferred_settlement(
                    helius,
                    user_wallet,
                    effective_base,
                    Some(&position.token_mint),
                )
                .await;
                let pnl = if settlement.source == "inferred-onchain" {
                    settlement.sol_delta_sol() - effective_base
                } else {
                    warn!(
                        "[inferred PnL] Unknown settlement for {} — recording 0 PnL for manual review",
                        &position.token_mint[..8]
                    );
                    0.0
                };
                info!(
                    "[inferred PnL] {} | source={} | realized={:.6} SOL | entry={:.6} SOL",
                    &position.token_mint[..8],
                    settlement.source,
                    pnl,
                    effective_base
                );
                (pnl, settlement.source)
            } else {
                warn!(
                    "[inferred PnL] No helius client for {} — recording 0 PnL (unknown)",
                    &position.token_mint[..8]
                );
                (0.0, "unknown")
            };

            let inferred_sig = format!(
                "INFERRED_CLOSE_{}_{}",
                position.token_mint[..8].to_string(),
                chrono::Utc::now().timestamp()
            );

            self.position_manager
                .close_position(
                    position.id,
                    position.current_price,
                    realized_pnl,
                    &format!("AlreadySold-Inferred({})", pnl_source),
                    Some(inferred_sig),
                    Some(position.momentum.momentum_score),
                )
                .await?;
            return Ok(());
        }

        let token_amount = (actual_balance as f64 * (signal.exit_percent / 100.0)).round() as u64;
        let token_amount = token_amount.min(actual_balance);

        info!(
            "Actual on-chain balance: {} tokens (tracked: {:.0})",
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
                if !used_emergency {
                    current_slippage = self.config.emergency_slippage_bps;
                    used_emergency = true;
                    warn!(
                        "EMERGENCY SLIPPAGE: Jumping to {}bps after failure (was {}bps)",
                        current_slippage, initial_slippage
                    );
                }
                tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            }

            info!(
                "Building curve sell for {} tokens @ mint {} (slippage: {} bps)",
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

                    if last_error.contains("graduated") || last_error.contains("is_complete") {
                        warn!(
                            "Token {} graduated mid-exit, switching to DEX path",
                            position
                                .token_symbol
                                .as_deref()
                                .unwrap_or(&position.token_mint[..8])
                        );

                        match curve_builder.build_raydium_sell(&sell_params).await {
                            Ok(raydium_result) => {
                                info!(
                                    "Built Raydium sell for graduated {}: expected {} SOL",
                                    position
                                        .token_symbol
                                        .as_deref()
                                        .unwrap_or(&position.token_mint[..8]),
                                    raydium_result.expected_sol_out as f64 / 1e9
                                );
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
                                    "Raydium failed for graduated {}: {}, falling back to Jupiter",
                                    position
                                        .token_symbol
                                        .as_deref()
                                        .unwrap_or(&position.token_mint[..8]),
                                    raydium_err
                                );
                                match self
                                    .tx_builder
                                    .build_exit_swap(
                                        position,
                                        signal,
                                        user_wallet,
                                        current_slippage,
                                    )
                                    .await
                                {
                                    Ok(jupiter_result) => super::curve_builder::CurveBuildResult {
                                        transaction_base64: jupiter_result.transaction_base64,
                                        expected_tokens_out: None,
                                        expected_sol_out: Some(jupiter_result.expected_base_out),
                                        min_tokens_out: None,
                                        min_sol_out: None,
                                        price_impact_percent: jupiter_result.price_impact_bps
                                            as f64
                                            / 100.0,
                                        fee_lamports: 0,
                                        compute_units: 200_000,
                                        priority_fee_lamports: 0,
                                    },
                                    Err(jupiter_err) => {
                                        last_error = format!(
                                            "Raydium: {} | Jupiter: {}",
                                            raydium_err, jupiter_err
                                        );
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

            let sign_result = match self.signer.sign_transaction(sign_request).await {
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

            info!(
                "Sending curve sell via Helius (attempt {}) - waiting for confirmation...",
                attempt + 1
            );

            let confirmation_timeout = Duration::from_secs(30);
            match helius_sender
                .send_and_confirm(&signed_tx, confirmation_timeout)
                .await
            {
                Ok(signature) => {
                    let settlement = self.resolve_sell_settlement(&signature, user_wallet).await;

                    let exit_price = signal.current_price;
                    let pnl_percent = if position.entry_price > 0.0 {
                        (exit_price - position.entry_price) / position.entry_price
                    } else {
                        0.0
                    };
                    let exit_reason = format!("{:?}", signal.reason);

                    let is_partial_exit = signal.exit_percent < 100.0;

                    if is_partial_exit {
                        let effective_base = if position.remaining_amount_base > 0.0 {
                            position.remaining_amount_base
                        } else {
                            position.entry_amount_base
                        };
                        let partial_base = effective_base * (signal.exit_percent / 100.0);
                        let estimated_pnl = partial_base * pnl_percent;
                        let realized_pnl_sol = self.compute_pnl_with_settlement(
                            settlement.as_ref(),
                            estimated_pnl,
                            position,
                        );

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

                        if let Some(capital_mgr) = &self.capital_manager {
                            if let Some(released) = capital_mgr
                                .release_partial_capital(signal.position_id, signal.exit_percent)
                                .await
                            {
                                debug!(
                                    "Released {} SOL partial capital for position {} ({}% exit)",
                                    released as f64 / 1_000_000_000.0,
                                    signal.position_id,
                                    signal.exit_percent
                                );
                            }
                        }

                        self.emit_exit_completed_event(
                            position,
                            signal,
                            realized_pnl_sol,
                            Some(&signature),
                        )
                        .await;

                        info!(
                            "Partial curve exit completed: {} | {}% exited | P&L: {:.6} SOL ({:.2}%) | Reason: {:?} | Sig: {} | source: {}",
                            position
                                .token_symbol
                                .as_deref()
                                .unwrap_or(&position.token_mint[..8]),
                            signal.exit_percent,
                            realized_pnl_sol,
                            pnl_percent * 100.0,
                            signal.reason,
                            &signature[..16],
                            settlement.as_ref().map(|s| s.source).unwrap_or("estimated"),
                        );

                        self.save_exit_to_engrams(
                            position,
                            signal,
                            realized_pnl_sol,
                            Some(&signature),
                            user_wallet,
                        )
                        .await;

                        self.save_sell_trade_record(
                            position,
                            signal,
                            realized_pnl_sol,
                            Some(&signature),
                            current_slippage,
                            settlement.as_ref(),
                        )
                        .await;
                    } else {
                        let effective_base = if position.remaining_amount_base > 0.0 {
                            position.remaining_amount_base
                        } else {
                            position.entry_amount_base
                        };
                        let estimated_pnl = effective_base * pnl_percent;
                        let realized_pnl_sol = self.compute_pnl_with_settlement(
                            settlement.as_ref(),
                            estimated_pnl,
                            position,
                        );

                        self.position_manager
                            .close_position(
                                signal.position_id,
                                exit_price,
                                realized_pnl_sol,
                                &exit_reason,
                                Some(signature.clone()),
                                Some(position.momentum.momentum_score),
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
                            "Curve exit completed: {} | P&L: {:.6} SOL ({:.2}%) | Reason: {:?} | Sig: {} | source: {}",
                            position
                                .token_symbol
                                .as_deref()
                                .unwrap_or(&position.token_mint[..8]),
                            realized_pnl_sol,
                            pnl_percent * 100.0,
                            signal.reason,
                            &signature[..16],
                            settlement.as_ref().map(|s| s.source).unwrap_or("estimated"),
                        );

                        self.save_exit_to_engrams(
                            position,
                            signal,
                            realized_pnl_sol,
                            Some(&signature),
                            user_wallet,
                        )
                        .await;

                        self.save_sell_trade_record(
                            position,
                            signal,
                            realized_pnl_sol,
                            Some(&signature),
                            current_slippage,
                            settlement.as_ref(),
                        )
                        .await;
                    }
                    return Ok(());
                }
                Err(e) => {
                    last_error = e.to_string();
                    let is_slippage_error = last_error.contains("6003")
                        || last_error.to_lowercase().contains("slippage");
                    let is_timeout_error =
                        last_error.contains("Timeout") || last_error.contains("timeout");

                    if is_timeout_error {
                        match self
                            .tx_builder
                            .get_token_balance(user_wallet, &position.token_mint)
                            .await
                        {
                            Ok(balance) => {
                                if balance < 1000 {
                                    warn!(
                                        "Curve sell confirmation timed out but token balance is 0 for {} - resolving via on-chain lookup",
                                        position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8])
                                    );
                                    let exit_price = signal.current_price;
                                    let exit_reason = format!("{:?}", signal.reason);
                                    let effective_base = if position.remaining_amount_base > 0.0 {
                                        position.remaining_amount_base
                                    } else {
                                        position.entry_amount_base
                                    };

                                    let (realized_pnl_sol, pnl_source) = if let Some(helius) =
                                        &self.helius_client
                                    {
                                        let settlement = resolve_inferred_settlement(
                                            helius,
                                            user_wallet,
                                            effective_base,
                                            Some(&position.token_mint),
                                        )
                                        .await;
                                        let pnl = if settlement.source == "inferred-onchain" {
                                            settlement.sol_delta_sol() - effective_base
                                        } else {
                                            warn!(
                                                "[timeout-inferred PnL] Unknown settlement for {} — recording 0 PnL for manual review",
                                                position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8])
                                            );
                                            0.0
                                        };
                                        info!(
                                            "[timeout-inferred PnL] {} | source={} | realized={:.6} SOL",
                                            position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8]),
                                            settlement.source, pnl
                                        );
                                        (pnl, settlement.source)
                                    } else {
                                        warn!("[timeout-inferred PnL] No helius client — recording 0 PnL (unknown)");
                                        (0.0, "unknown")
                                    };

                                    let inferred_sig = format!(
                                        "INFERRED_EXIT_{}_{}_balance_zero",
                                        signal.position_id.to_string()[..8].to_string(),
                                        chrono::Utc::now().timestamp()
                                    );

                                    if let Err(e) = self
                                        .position_manager
                                        .close_position(
                                            signal.position_id,
                                            exit_price,
                                            realized_pnl_sol,
                                            &format!("{}({})", exit_reason, pnl_source),
                                            Some(inferred_sig.clone()),
                                            Some(position.momentum.momentum_score),
                                        )
                                        .await
                                    {
                                        error!(
                                            "CRITICAL: Curve exit inferred successful but failed to close position {} in DB: {}",
                                            signal.position_id, e
                                        );
                                    }

                                    self.emit_exit_completed_event(
                                        position,
                                        signal,
                                        realized_pnl_sol,
                                        Some(&inferred_sig),
                                    )
                                    .await;

                                    info!(
                                        "Curve exit completed (INFERRED - {}): {} | P&L: {:.6} SOL | Reason: {:?}",
                                        pnl_source,
                                        position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8]),
                                        realized_pnl_sol,
                                        signal.reason
                                    );

                                    self.save_exit_to_engrams(
                                        position,
                                        signal,
                                        realized_pnl_sol,
                                        Some(&inferred_sig),
                                        user_wallet,
                                    )
                                    .await;

                                    return Ok(());
                                } else {
                                    warn!("Curve sell timed out and {} tokens still in wallet - will retry", balance);
                                }
                            }
                            Err(balance_err) => {
                                warn!(
                                    "Could not verify balance after curve sell timeout: {}",
                                    balance_err
                                );
                            }
                        }
                    }

                    let is_graduated_error = last_error.contains("6023");
                    if is_graduated_error {
                        warn!(
                            "Error 6023 (graduated) during tx - switching to DEX path for {}",
                            position
                                .token_symbol
                                .as_deref()
                                .unwrap_or(&position.token_mint[..8])
                        );
                        break;
                    }

                    if is_slippage_error && attempt < max_retries {
                        warn!(
                            "Slippage error on attempt {}, will retry: {}",
                            attempt + 1,
                            last_error
                        );
                        continue;
                    } else {
                        error!("Curve exit failed: {}", last_error);
                        break;
                    }
                }
            }
        }

        if last_error.contains("6023") {
            warn!(
                "Curve sell failed with 6023 - attempting DEX fallback for {}",
                position
                    .token_symbol
                    .as_deref()
                    .unwrap_or(&position.token_mint[..8])
            );

            let token_balance = match self
                .tx_builder
                .get_token_balance(user_wallet, &position.token_mint)
                .await
            {
                Ok(b) => b,
                Err(e) => {
                    error!("Failed to get token balance for DEX fallback: {}", e);
                    return Err(AppError::ExternalApi(format!("DEX fallback failed: {}", e)));
                }
            };

            if token_balance == 0 {
                warn!("Token balance is 0 - position may have been sold externally");
                if let Err(e) = self
                    .position_manager
                    .close_position(
                        signal.position_id,
                        signal.current_price,
                        0.0,
                        "AlreadySold",
                        None,
                        Some(position.momentum.momentum_score),
                    )
                    .await
                {
                    error!(
                        "CRITICAL: Failed to close already-sold position {} in database: {}",
                        signal.position_id, e
                    );
                }
                return Ok(());
            }

            let exit_amount = if signal.exit_percent >= 100.0 {
                token_balance
            } else {
                ((token_balance as f64) * (signal.exit_percent / 100.0))
                    .round()
                    .min(token_balance as f64) as u64
            };

            let sell_params = CurveSellParams {
                mint: position.token_mint.clone(),
                token_amount: exit_amount,
                slippage_bps: self.config.emergency_slippage_bps,
                user_wallet: user_wallet.to_string(),
            };

            if let Some(ref curve_builder) = self.curve_builder {
                match curve_builder.build_raydium_sell(&sell_params).await {
                    Ok(raydium_result) => {
                        info!(
                            "Built Raydium fallback sell: expected {} SOL",
                            raydium_result.expected_sol_out as f64 / 1e9
                        );

                        let sign_request = SignRequest {
                            transaction_base64: raydium_result.transaction_base64,
                            estimated_amount_lamports: raydium_result.expected_sol_out,
                            estimated_profit_lamports: None,
                            edge_id: Some(position.edge_id),
                            description: format!(
                                "Raydium fallback exit {}",
                                &position.token_mint[..8]
                            ),
                        };

                        if let Ok(sign_result) = self.signer.sign_transaction(sign_request).await {
                            if sign_result.success {
                                if let Some(signed_tx) = sign_result.signed_transaction_base64 {
                                    if let Some(ref helius_sender) = self.helius_sender {
                                        if let Ok(signature) = helius_sender
                                            .send_and_confirm(&signed_tx, Duration::from_secs(60))
                                            .await
                                        {
                                            let settlement = self
                                                .resolve_sell_settlement(&signature, user_wallet)
                                                .await;

                                            let exit_price = signal.current_price;
                                            let pnl_percent = if position.entry_price > 0.0 {
                                                (exit_price - position.entry_price)
                                                    / position.entry_price
                                            } else {
                                                0.0
                                            };
                                            let effective_base =
                                                if position.remaining_amount_base > 0.0 {
                                                    position.remaining_amount_base
                                                } else {
                                                    position.entry_amount_base
                                                };
                                            let estimated_pnl = effective_base * pnl_percent;
                                            let realized_pnl_sol = self
                                                .compute_pnl_with_settlement(
                                                    settlement.as_ref(),
                                                    estimated_pnl,
                                                    position,
                                                );

                                            if let Err(e) = self
                                                .position_manager
                                                .close_position(
                                                    signal.position_id,
                                                    exit_price,
                                                    realized_pnl_sol,
                                                    &format!(
                                                        "{:?}-raydium-fallback",
                                                        signal.reason
                                                    ),
                                                    Some(signature.clone()),
                                                    Some(position.momentum.momentum_score),
                                                )
                                                .await
                                            {
                                                error!(
                                                    "CRITICAL: Raydium exit confirmed (sig: {}) but failed to close position {} in DB: {}",
                                                    &signature[..16], signal.position_id, e
                                                );
                                            }

                                            info!(
                                                "Raydium fallback exit succeeded: {} | P&L: {:.6} SOL | Sig: {} | source: {}",
                                                position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8]),
                                                realized_pnl_sol,
                                                &signature[..16],
                                                settlement.as_ref().map(|s| s.source).unwrap_or("estimated"),
                                            );
                                            self.save_sell_trade_record(
                                                position,
                                                signal,
                                                realized_pnl_sol,
                                                Some(&signature),
                                                self.config.emergency_slippage_bps,
                                                settlement.as_ref(),
                                            )
                                            .await;
                                            return Ok(());
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Raydium fallback failed: {} - trying Jupiter", e);
                    }
                }
            }

            match self
                .tx_builder
                .build_exit_swap(
                    position,
                    signal,
                    user_wallet,
                    self.config.emergency_slippage_bps,
                )
                .await
            {
                Ok(jupiter_result) => {
                    let sign_request = SignRequest {
                        transaction_base64: jupiter_result.transaction_base64,
                        estimated_amount_lamports: jupiter_result.expected_base_out,
                        estimated_profit_lamports: None,
                        edge_id: Some(position.edge_id),
                        description: format!("Jupiter fallback exit {}", &position.token_mint[..8]),
                    };

                    if let Ok(sign_result) = self.signer.sign_transaction(sign_request).await {
                        if sign_result.success {
                            if let Some(signed_tx) = sign_result.signed_transaction_base64 {
                                if let Some(ref helius_sender) = self.helius_sender {
                                    if let Ok(signature) = helius_sender
                                        .send_and_confirm(&signed_tx, Duration::from_secs(60))
                                        .await
                                    {
                                        let settlement = self
                                            .resolve_sell_settlement(&signature, user_wallet)
                                            .await;

                                        let exit_price = signal.current_price;
                                        let pnl_percent = if position.entry_price > 0.0 {
                                            (exit_price - position.entry_price)
                                                / position.entry_price
                                        } else {
                                            0.0
                                        };
                                        let effective_base = if position.remaining_amount_base > 0.0
                                        {
                                            position.remaining_amount_base
                                        } else {
                                            position.entry_amount_base
                                        };
                                        let estimated_pnl = effective_base * pnl_percent;
                                        let realized_pnl_sol = self.compute_pnl_with_settlement(
                                            settlement.as_ref(),
                                            estimated_pnl,
                                            position,
                                        );

                                        if let Err(e) = self
                                            .position_manager
                                            .close_position(
                                                signal.position_id,
                                                exit_price,
                                                realized_pnl_sol,
                                                &format!("{:?}-jupiter-fallback", signal.reason),
                                                Some(signature.clone()),
                                                Some(position.momentum.momentum_score),
                                            )
                                            .await
                                        {
                                            error!(
                                                "CRITICAL: Jupiter exit confirmed (sig: {}) but failed to close position {} in DB: {}",
                                                &signature[..16], signal.position_id, e
                                            );
                                        }

                                        info!(
                                            "Jupiter fallback exit succeeded: {} | P&L: {:.6} SOL | Sig: {} | source: {}",
                                            position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8]),
                                            realized_pnl_sol,
                                            &signature[..16],
                                            settlement.as_ref().map(|s| s.source).unwrap_or("estimated"),
                                        );
                                        self.save_sell_trade_record(
                                            position,
                                            signal,
                                            realized_pnl_sol,
                                            Some(&signature),
                                            self.config.emergency_slippage_bps,
                                            settlement.as_ref(),
                                        )
                                        .await;
                                        return Ok(());
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Jupiter fallback also failed: {}", e);
                }
            }

            warn!("All DEX fallbacks failed for graduated token - queuing for retry");
        }

        error!(
            "Curve exit failed after {} attempts: {} - QUEUING HIGH PRIORITY RETRY",
            max_retries + 1,
            last_error
        );
        self.emit_exit_failed_event(position, signal, &last_error)
            .await;

        if let Err(e) = self
            .position_manager
            .reset_position_status(signal.position_id)
            .await
        {
            error!("Failed to reset position status for retry: {}", e);
        } else {
            self.position_manager
                .queue_priority_exit(signal.position_id)
                .await;
            info!(
                "Position {} queued for HIGH PRIORITY retry after curve exit failure",
                signal.position_id
            );
        }

        Err(AppError::ExternalApi(format!(
            "Curve exit failed after {} attempts: {}",
            max_retries + 1,
            last_error
        )))
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

        if let Err(e) = self.event_tx.send(event) {
            tracing::warn!("Event broadcast failed (channel full/closed): {}", e);
        }
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

        if let Err(e) = self.event_tx.send(event) {
            tracing::warn!("Event broadcast failed (channel full/closed): {}", e);
        }
    }
}

fn base64_to_base58(base64_str: &str) -> AppResult<String> {
    use base64::{engine::general_purpose::STANDARD, Engine};

    let bytes = STANDARD
        .decode(base64_str)
        .map_err(|e| crate::error::AppError::Execution(format!("Invalid base64: {}", e)))?;

    Ok(bs58::encode(bytes).into_string())
}
