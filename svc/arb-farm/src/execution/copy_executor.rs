use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio::time::Instant;
use uuid::Uuid;

use crate::database::repositories::{KolRepository, CreateCopyTradeRecord, UpdateCopyTradeRecord};
use crate::models::CopyTradeStatus;
use crate::engrams::client::EngramsClient;
use crate::error::{AppError, AppResult};
use crate::events::{ArbEvent, AgentType, EventSource};
use crate::execution::{CurveBuyParams, CurveSellParams, CurveTransactionBuilder, PositionManager};
use crate::helius::HeliusSender;
use crate::models::Signal;
use crate::wallet::DevWalletSigner;
use crate::wallet::turnkey::SignRequest;

const MIN_COPY_INTERVAL_MS: u64 = 1000;
const MAX_COPIES_PER_MINUTE: u32 = 10;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyExecutorConfig {
    pub enabled: bool,
    pub default_copy_percentage: f64,
    pub max_position_sol: f64,
    pub min_trust_score: f64,
    pub copy_delay_ms: u64,
    pub require_whitelist: bool,
}

impl Default for CopyExecutorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_copy_percentage: 0.5,
            max_position_sol: 0.5,
            min_trust_score: 60.0,
            copy_delay_ms: 500,
            require_whitelist: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyTradeResult {
    pub copy_trade_id: Uuid,
    pub kol_id: Uuid,
    pub kol_trade_id: Uuid,
    pub token_mint: String,
    pub trade_type: String,
    pub sol_amount: f64,
    pub success: bool,
    pub tx_signature: Option<String>,
    pub error: Option<String>,
    pub latency_ms: u64,
    pub executed_at: DateTime<Utc>,
}

struct RateLimiter {
    last_copy_at: Option<Instant>,
    copies_this_minute: u32,
    minute_started: Instant,
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self {
            last_copy_at: None,
            copies_this_minute: 0,
            minute_started: Instant::now(),
        }
    }
}

pub struct CopyTradeExecutor {
    kol_repo: Arc<KolRepository>,
    curve_builder: Arc<CurveTransactionBuilder>,
    dev_signer: Arc<DevWalletSigner>,
    helius_sender: Arc<HeliusSender>,
    position_manager: Arc<PositionManager>,
    engrams_client: Arc<EngramsClient>,
    event_tx: broadcast::Sender<ArbEvent>,
    config: Arc<RwLock<CopyExecutorConfig>>,
    default_wallet: String,
    rate_limiter: Arc<RwLock<RateLimiter>>,
    copy_to_position: Arc<RwLock<HashMap<Uuid, Uuid>>>,
}

impl CopyTradeExecutor {
    pub fn new(
        kol_repo: Arc<KolRepository>,
        curve_builder: Arc<CurveTransactionBuilder>,
        dev_signer: Arc<DevWalletSigner>,
        helius_sender: Arc<HeliusSender>,
        position_manager: Arc<PositionManager>,
        engrams_client: Arc<EngramsClient>,
        event_tx: broadcast::Sender<ArbEvent>,
        default_wallet: String,
    ) -> Self {
        Self {
            kol_repo,
            curve_builder,
            dev_signer,
            helius_sender,
            position_manager,
            engrams_client,
            event_tx,
            config: Arc::new(RwLock::new(CopyExecutorConfig::default())),
            default_wallet,
            rate_limiter: Arc::new(RwLock::new(RateLimiter::default())),
            copy_to_position: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn check_rate_limit(&self) -> AppResult<()> {
        let mut limiter = self.rate_limiter.write().await;
        let now = Instant::now();

        if now.duration_since(limiter.minute_started).as_secs() >= 60 {
            limiter.copies_this_minute = 0;
            limiter.minute_started = now;
        }

        if limiter.copies_this_minute >= MAX_COPIES_PER_MINUTE {
            return Err(AppError::Validation(format!(
                "Rate limit exceeded: {} copies per minute max",
                MAX_COPIES_PER_MINUTE
            )));
        }

        if let Some(last) = limiter.last_copy_at {
            let elapsed_ms = now.duration_since(last).as_millis() as u64;
            if elapsed_ms < MIN_COPY_INTERVAL_MS {
                let wait_ms = MIN_COPY_INTERVAL_MS - elapsed_ms;
                tracing::debug!(
                    wait_ms = wait_ms,
                    "Rate limiting: waiting before next copy trade"
                );
                drop(limiter);
                tokio::time::sleep(tokio::time::Duration::from_millis(wait_ms)).await;
                limiter = self.rate_limiter.write().await;
            }
        }

        limiter.last_copy_at = Some(Instant::now());
        limiter.copies_this_minute += 1;

        Ok(())
    }

    pub async fn link_copy_to_position(&self, copy_trade_id: Uuid, position_id: Uuid) {
        let mut map = self.copy_to_position.write().await;
        map.insert(copy_trade_id, position_id);
    }

    pub async fn calculate_profit_for_closed_position(&self, position_id: Uuid, pnl_lamports: i64) {
        let map = self.copy_to_position.read().await;
        let copy_trade_id = map.iter()
            .find(|(_, &pid)| pid == position_id)
            .map(|(&cid, _)| cid);

        if let Some(copy_id) = copy_trade_id {
            drop(map);

            let update = UpdateCopyTradeRecord {
                profit_loss_lamports: Some(pnl_lamports),
                ..Default::default()
            };

            if let Err(e) = self.kol_repo.update_copy_trade(copy_id, update).await {
                tracing::warn!(
                    copy_trade_id = %copy_id,
                    position_id = %position_id,
                    error = %e,
                    "Failed to update copy trade profit"
                );
            } else {
                tracing::info!(
                    copy_trade_id = %copy_id,
                    position_id = %position_id,
                    pnl_lamports = pnl_lamports,
                    "📊 Copy trade profit recorded"
                );
            }

            let mut map = self.copy_to_position.write().await;
            map.remove(&copy_id);
        }
    }

    pub fn with_config(mut self, config: CopyExecutorConfig) -> Self {
        self.config = Arc::new(RwLock::new(config));
        self
    }

    pub async fn update_config(&self, config: CopyExecutorConfig) {
        let mut current = self.config.write().await;
        *current = config;
    }

    pub async fn get_config(&self) -> CopyExecutorConfig {
        self.config.read().await.clone()
    }

    pub async fn execute_copy(&self, signal: &Signal) -> AppResult<CopyTradeResult> {
        let start = std::time::Instant::now();
        let config = self.config.read().await.clone();

        if !config.enabled {
            return Err(AppError::Validation("Copy trading is disabled".into()));
        }

        self.check_rate_limit().await?;

        let kol_id = self.extract_kol_id(signal)?;
        let kol_trade_id = self.extract_kol_trade_id(signal)?;
        let token_mint = signal.token_mint.clone()
            .ok_or_else(|| AppError::Validation("Signal missing token_mint".into()))?;
        let trade_type = self.extract_trade_type(signal)?;
        let kol_trust_score = self.extract_trust_score(signal)?;

        if kol_trust_score < config.min_trust_score {
            return Err(AppError::Validation(format!(
                "KOL trust score {} below minimum {}",
                kol_trust_score, config.min_trust_score
            )));
        }

        let kol_amount_sol = self.extract_kol_amount(signal)?;

        let copy_percentage = signal.metadata.get("copy_percentage")
            .and_then(|v| v.as_f64())
            .unwrap_or(config.default_copy_percentage);

        let mut our_amount_sol = kol_amount_sol * copy_percentage;
        if our_amount_sol > config.max_position_sol {
            our_amount_sol = config.max_position_sol;
            tracing::info!(
                kol_id = %kol_id,
                kol_amount = kol_amount_sol,
                copy_percentage = copy_percentage,
                calculated = kol_amount_sol * copy_percentage,
                capped = our_amount_sol,
                "Copy amount capped to max_position_sol"
            );
        }

        let copy_trade_record = CreateCopyTradeRecord {
            entity_id: kol_id,
            kol_trade_id,
            copy_amount_sol: rust_decimal::Decimal::from_f64_retain(our_amount_sol)
                .unwrap_or_default(),
            delay_ms: config.copy_delay_ms as i64,
        };

        let copy_trade = self.kol_repo.record_copy_trade(copy_trade_record).await?;
        let copy_trade_id = copy_trade.id;

        if config.copy_delay_ms > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(config.copy_delay_ms)).await;
        }

        let result = match trade_type.as_str() {
            "buy" => self.execute_buy(&token_mint, our_amount_sol, &kol_id, &kol_trade_id, copy_trade_id).await,
            "sell" => self.execute_sell(&token_mint, &kol_id, &kol_trade_id).await,
            _ => Err(AppError::Validation(format!("Unknown trade type: {}", trade_type))),
        };

        let latency_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok((tx_signature, success)) => {
                let _ = self.kol_repo.update_copy_trade(
                    copy_trade_id,
                    UpdateCopyTradeRecord {
                        our_tx_signature: tx_signature.clone(),
                        status: Some(if success { CopyTradeStatus::Executed } else { CopyTradeStatus::Failed }),
                        is_copied: Some(success),
                        executed_at: Some(Utc::now()),
                        ..Default::default()
                    },
                ).await;

                let copy_result = CopyTradeResult {
                    copy_trade_id,
                    kol_id,
                    kol_trade_id,
                    token_mint: token_mint.clone(),
                    trade_type: trade_type.clone(),
                    sol_amount: our_amount_sol,
                    success,
                    tx_signature: tx_signature.clone(),
                    error: None,
                    latency_ms,
                    executed_at: Utc::now(),
                };

                self.emit_copy_event(&copy_result, true).await;

                tracing::info!(
                    kol_id = %kol_id,
                    trade_type = %trade_type,
                    token_mint = %token_mint,
                    sol_amount = our_amount_sol,
                    tx = ?tx_signature,
                    latency_ms = latency_ms,
                    "✅ Copy trade executed successfully"
                );

                Ok(copy_result)
            }
            Err(e) => {
                let _ = self.kol_repo.update_copy_trade(
                    copy_trade_id,
                    UpdateCopyTradeRecord {
                        status: Some(CopyTradeStatus::Failed),
                        is_copied: Some(false),
                        executed_at: Some(Utc::now()),
                        ..Default::default()
                    },
                ).await;

                let copy_result = CopyTradeResult {
                    copy_trade_id,
                    kol_id,
                    kol_trade_id,
                    token_mint: token_mint.clone(),
                    trade_type: trade_type.clone(),
                    sol_amount: our_amount_sol,
                    success: false,
                    tx_signature: None,
                    error: Some(e.to_string()),
                    latency_ms,
                    executed_at: Utc::now(),
                };

                self.emit_copy_event(&copy_result, false).await;

                tracing::error!(
                    kol_id = %kol_id,
                    trade_type = %trade_type,
                    token_mint = %token_mint,
                    error = %e,
                    "❌ Copy trade execution failed"
                );

                Err(e)
            }
        }
    }

    async fn execute_buy(
        &self,
        token_mint: &str,
        sol_amount: f64,
        kol_id: &Uuid,
        kol_trade_id: &Uuid,
        copy_trade_id: Uuid,
    ) -> AppResult<(Option<String>, bool)> {
        if !self.dev_signer.is_configured() {
            return Err(AppError::Internal("Dev signer not configured".into()));
        }

        let curve_state = self.curve_builder.get_curve_state(token_mint).await?;

        if curve_state.is_complete {
            return Err(AppError::Validation("Token has already graduated".into()));
        }

        let sol_lamports = (sol_amount * 1_000_000_000.0) as u64;
        let slippage_bps = 500;

        let build_result = self.curve_builder.build_pump_fun_buy(
            &CurveBuyParams {
                mint: token_mint.to_string(),
                sol_amount_lamports: sol_lamports,
                slippage_bps,
                user_wallet: self.default_wallet.clone(),
            },
        ).await?;

        let sign_request = SignRequest {
            transaction_base64: build_result.transaction_base64.clone(),
            estimated_amount_lamports: sol_lamports,
            estimated_profit_lamports: None,
            edge_id: None,
            description: format!("Copy trade buy {} for KOL {}", token_mint, kol_id),
        };

        let sign_result = self.dev_signer.sign_transaction(sign_request).await?;

        if !sign_result.success {
            return Err(AppError::Execution(
                sign_result.error.unwrap_or_else(|| "Signing failed".to_string())
            ));
        }

        let signed_tx = sign_result.signed_transaction_base64
            .ok_or_else(|| AppError::Execution("No signed transaction returned".into()))?;

        let signature = self.helius_sender.send_and_confirm(&signed_tx, std::time::Duration::from_secs(30)).await
            .map_err(|e| AppError::Execution(format!("Send failed: {}", e)))?;

        tracing::info!(
            kol_id = %kol_id,
            kol_trade_id = %kol_trade_id,
            token_mint = %token_mint,
            sol_amount = sol_amount,
            signature = %signature,
            "Copy buy transaction submitted"
        );

        // Open position in position manager for tracking and auto-exit
        let expected_tokens = build_result.expected_tokens_out.unwrap_or(0) as f64;
        let entry_price = if expected_tokens > 0.0 {
            sol_amount / expected_tokens
        } else {
            0.0
        };

        let exit_config = crate::execution::ExitConfig::default();

        if let Ok(position) = self.position_manager.open_position(
            Uuid::nil(),
            Uuid::nil(),
            token_mint.to_string(),
            None,
            sol_amount,
            expected_tokens,
            entry_price,
            exit_config,
            Some(signature.clone()),
            Some("kol_copy".to_string()),
            Some(format!("kol:{}", kol_id)),
        ).await {
            self.link_copy_to_position(copy_trade_id, position.id).await;
            tracing::info!(
                copy_trade_id = %copy_trade_id,
                position_id = %position.id,
                token_mint = %token_mint,
                "📊 Linked copy trade to position for profit tracking"
            );
        }

        Ok((Some(signature), true))
    }

    async fn execute_sell(
        &self,
        token_mint: &str,
        kol_id: &Uuid,
        kol_trade_id: &Uuid,
    ) -> AppResult<(Option<String>, bool)> {
        let position = match self.position_manager.get_open_position_for_mint(token_mint).await {
            Some(pos) => pos,
            None => {
                tracing::warn!(
                    kol_id = %kol_id,
                    kol_trade_id = %kol_trade_id,
                    token_mint = %token_mint,
                    "No position to sell - KOL sold but we don't have tokens"
                );
                return Ok((None, false));
            }
        };

        let position_id = position.id;
        self.position_manager.queue_priority_exit(position_id).await;

        tracing::info!(
            kol_id = %kol_id,
            kol_trade_id = %kol_trade_id,
            token_mint = %token_mint,
            position_id = %position_id,
            "🔥 Copy sell queued for priority exit - waiting for confirmation"
        );

        // Wait for position to be closed (position monitor handles async exits)
        // Timeout after 30 seconds - KOL may fully exit while we wait
        const COPY_SELL_TIMEOUT_SECS: u64 = 30;
        const CHECK_INTERVAL_MS: u64 = 2000;
        let max_checks = COPY_SELL_TIMEOUT_SECS * 1000 / CHECK_INTERVAL_MS;

        for check in 0..max_checks {
            tokio::time::sleep(tokio::time::Duration::from_millis(CHECK_INTERVAL_MS)).await;

            // Check if position was closed by position monitor
            if self.position_manager.get_position(position_id).await.is_none() {
                tracing::info!(
                    kol_id = %kol_id,
                    position_id = %position_id,
                    token_mint = %token_mint,
                    check = check + 1,
                    "✅ Copy sell completed by position monitor"
                );
                return Ok((None, true));
            }

            // Position still open - check if it was marked as closed/pending_exit
            if let Some(current_pos) = self.position_manager.get_position(position_id).await {
                if current_pos.status == crate::execution::PositionStatus::Closed {
                    tracing::info!(
                        kol_id = %kol_id,
                        position_id = %position_id,
                        "✅ Copy sell position marked closed"
                    );
                    return Ok((None, true));
                }
            }
        }

        // Position still not closed after timeout - force direct sell with emergency slippage
        tracing::warn!(
            kol_id = %kol_id,
            kol_trade_id = %kol_trade_id,
            token_mint = %token_mint,
            position_id = %position_id,
            timeout_secs = COPY_SELL_TIMEOUT_SECS,
            "⚠️ Copy sell timeout - forcing direct market sell with emergency slippage"
        );

        // Force direct sell with maximum slippage tolerance
        self.force_emergency_sell(token_mint, kol_id, position_id).await
    }

    async fn force_emergency_sell(
        &self,
        token_mint: &str,
        kol_id: &Uuid,
        position_id: Uuid,
    ) -> AppResult<(Option<String>, bool)> {
        const EMERGENCY_SLIPPAGE_BPS: u16 = 2500; // 25% - prioritize exit over profit

        if !self.dev_signer.is_configured() {
            return Err(AppError::Internal("Dev signer not configured for emergency sell".into()));
        }

        // Get actual token balance
        let token_balance = self.curve_builder
            .get_actual_token_balance(&self.default_wallet, token_mint)
            .await
            .unwrap_or(0);

        if token_balance == 0 {
            tracing::warn!(
                kol_id = %kol_id,
                token_mint = %token_mint,
                "Emergency sell: no tokens to sell (may have been sold externally)"
            );
            // Mark position as closed since tokens are gone
            let _ = self.position_manager.close_position(
                position_id,
                0.0,
                0.0,
                "CopySellForced-NoBalance",
                None,
                None, // No momentum data for forced close
            ).await;
            return Ok((None, true));
        }

        let sell_params = CurveSellParams {
            mint: token_mint.to_string(),
            token_amount: token_balance,
            slippage_bps: EMERGENCY_SLIPPAGE_BPS,
            user_wallet: self.default_wallet.clone(),
        };

        // Try curve sell first (pre-graduation), then DEX for graduated tokens
        let build_result = match self.curve_builder.build_pump_fun_sell(&sell_params).await {
            Ok(result) => result,
            Err(curve_err) => {
                // Token may have graduated - try Raydium
                tracing::info!(
                    kol_id = %kol_id,
                    token_mint = %token_mint,
                    "Curve sell failed ({}), trying Raydium...",
                    curve_err
                );
                match self.curve_builder.build_raydium_sell(&sell_params).await {
                    Ok(raydium_result) => {
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
                        return Err(AppError::Execution(format!(
                            "Emergency sell failed: curve={}, raydium={}",
                            curve_err, raydium_err
                        )));
                    }
                }
            }
        };

        let sign_request = SignRequest {
            transaction_base64: build_result.transaction_base64.clone(),
            estimated_amount_lamports: build_result.expected_sol_out.unwrap_or(0) as u64,
            estimated_profit_lamports: None,
            edge_id: None,
            description: format!("EMERGENCY copy sell {} for KOL {}", token_mint, kol_id),
        };

        let sign_result = self.dev_signer.sign_transaction(sign_request).await?;

        if !sign_result.success {
            return Err(AppError::Execution(
                sign_result.error.unwrap_or_else(|| "Emergency signing failed".to_string())
            ));
        }

        let signed_tx = sign_result.signed_transaction_base64
            .ok_or_else(|| AppError::Execution("No signed transaction for emergency sell".into()))?;

        let signature = self.helius_sender.send_and_confirm(&signed_tx, std::time::Duration::from_secs(60)).await
            .map_err(|e| AppError::Execution(format!("Emergency send failed: {}", e)))?;

        tracing::info!(
            kol_id = %kol_id,
            token_mint = %token_mint,
            position_id = %position_id,
            signature = %signature,
            "🔥 Emergency copy sell executed"
        );

        // Close position in tracking
        let _ = self.position_manager.close_position(
            position_id,
            0.0, // Price not known precisely
            0.0, // PnL calculated separately
            "CopySellForced",
            Some(signature.clone()),
            None, // No momentum data for forced close
        ).await;

        Ok((Some(signature), true))
    }

    async fn emit_copy_event(&self, result: &CopyTradeResult, success: bool) {
        let event = ArbEvent::new(
            if success { "copy_trade.executed" } else { "copy_trade.failed" },
            EventSource::Agent(AgentType::Executor),
            if success { "arb.kol.trade.copied" } else { "arb.kol.trade.copy_failed" },
            serde_json::json!({
                "copy_trade_id": result.copy_trade_id,
                "kol_id": result.kol_id,
                "kol_trade_id": result.kol_trade_id,
                "token_mint": result.token_mint,
                "trade_type": result.trade_type,
                "sol_amount": result.sol_amount,
                "success": result.success,
                "tx_signature": result.tx_signature,
                "error": result.error,
                "latency_ms": result.latency_ms,
            }),
        );

        let _ = self.event_tx.send(event);
    }

    fn extract_kol_id(&self, signal: &Signal) -> AppResult<Uuid> {
        signal.metadata.get("kol_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or_else(|| AppError::Validation("Signal missing kol_id".into()))
    }

    fn extract_kol_trade_id(&self, signal: &Signal) -> AppResult<Uuid> {
        signal.metadata.get("kol_trade_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or_else(|| AppError::Validation("Signal missing kol_trade_id".into()))
    }

    fn extract_trade_type(&self, signal: &Signal) -> AppResult<String> {
        signal.metadata.get("trade_type")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| AppError::Validation("Signal missing trade_type".into()))
    }

    fn extract_trust_score(&self, signal: &Signal) -> AppResult<f64> {
        signal.metadata.get("kol_trust_score")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| AppError::Validation("Signal missing kol_trust_score".into()))
    }

    fn extract_kol_amount(&self, signal: &Signal) -> AppResult<f64> {
        signal.metadata.get("amount_sol")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| AppError::Validation("Signal missing amount_sol".into()))
    }
}
