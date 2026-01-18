use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::error::AppError;
use crate::execution::{OpenPosition, PositionStatus, ExitReason, BaseCurrency, WalletTokenHolding, ReconciliationResult, ExitConfig};
use crate::server::AppState;

#[derive(Debug, Serialize)]
pub struct PositionsResponse {
    pub positions: Vec<OpenPosition>,
    pub stats: PositionStatsResponse,
}

#[derive(Debug, Serialize)]
pub struct PositionStatsResponse {
    pub total_positions_opened: u64,
    pub total_positions_closed: u64,
    pub active_positions: u32,
    pub total_realized_pnl: f64,
    pub total_unrealized_pnl: f64,
    pub stop_losses_triggered: u32,
    pub take_profits_triggered: u32,
    pub time_exits_triggered: u32,
}

pub async fn get_positions(
    State(state): State<AppState>,
) -> Result<Json<PositionsResponse>, AppError> {
    let positions = state.position_manager.get_open_positions().await;
    let stats = state.position_manager.get_stats().await;

    Ok(Json(PositionsResponse {
        positions,
        stats: PositionStatsResponse {
            total_positions_opened: stats.total_positions_opened,
            total_positions_closed: stats.total_positions_closed,
            active_positions: stats.active_positions,
            total_realized_pnl: stats.total_realized_pnl,
            total_unrealized_pnl: stats.total_unrealized_pnl,
            stop_losses_triggered: stats.stop_losses_triggered,
            take_profits_triggered: stats.take_profits_triggered,
            time_exits_triggered: stats.time_exits_triggered,
        },
    }))
}

pub async fn get_position(
    State(state): State<AppState>,
    Path(position_id): Path<Uuid>,
) -> Result<Json<OpenPosition>, AppError> {
    state
        .position_manager
        .get_position(position_id)
        .await
        .ok_or_else(|| AppError::NotFound(format!("Position {} not found", position_id)))
        .map(Json)
}

#[derive(Debug, Deserialize)]
pub struct ManualExitRequest {
    pub exit_percent: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct ExitResponse {
    pub success: bool,
    pub message: String,
    pub position_id: Uuid,
}

pub async fn close_position(
    State(state): State<AppState>,
    Path(position_id): Path<Uuid>,
    Json(request): Json<ManualExitRequest>,
) -> Result<Json<ExitResponse>, AppError> {
    let exit_percent = request.exit_percent.unwrap_or(100.0);

    match state
        .position_monitor
        .trigger_manual_exit(position_id, exit_percent, &state.dev_signer)
        .await
    {
        Ok(_) => {
            // Release capital if this was a full exit
            if exit_percent >= 100.0 {
                if let Some(released) = state.capital_manager.release_capital(position_id).await {
                    info!(
                        "💸 Released {} SOL capital for closed position {}",
                        released as f64 / 1_000_000_000.0,
                        position_id
                    );
                }
            }

            Ok(Json(ExitResponse {
                success: true,
                message: format!("Exit triggered for {}% of position", exit_percent),
                position_id,
            }))
        }
        Err(e) => Ok(Json(ExitResponse {
            success: false,
            message: format!("Exit failed: {}", e),
            position_id,
        })),
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateExitConfigRequest {
    #[serde(default)]
    pub stop_loss_percent: Option<f64>,
    #[serde(default)]
    pub take_profit_percent: Option<f64>,
    #[serde(default)]
    pub trailing_stop_percent: Option<f64>,
    #[serde(default)]
    pub time_limit_minutes: Option<u32>,
    /// Use a preset config: "curve", "curve_conservative", "default"
    #[serde(default)]
    pub preset: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UpdateExitConfigResponse {
    pub success: bool,
    pub position_id: Uuid,
    pub old_config: ExitConfigSummary,
    pub new_config: ExitConfigSummary,
}

fn exit_config_to_summary(config: &ExitConfig) -> ExitConfigSummary {
    ExitConfigSummary {
        stop_loss_percent: config.stop_loss_percent,
        take_profit_percent: config.take_profit_percent,
        trailing_stop_percent: config.trailing_stop_percent,
        time_limit_minutes: config.time_limit_minutes,
    }
}

pub async fn update_position_exit_config(
    State(state): State<AppState>,
    Path(position_id): Path<Uuid>,
    Json(request): Json<UpdateExitConfigRequest>,
) -> Result<Json<UpdateExitConfigResponse>, AppError> {
    let current_position = state
        .position_manager
        .get_position(position_id)
        .await
        .ok_or_else(|| AppError::NotFound(format!("Position {} not found", position_id)))?;

    let old_config = exit_config_to_summary(&current_position.exit_config);

    // Build new config from preset or individual fields
    let new_config = if let Some(preset) = &request.preset {
        match preset.as_str() {
            "curve" => ExitConfig::for_curve_bonding(),
            "curve_conservative" => ExitConfig::for_curve_bonding_conservative(),
            "default" => ExitConfig::default(),
            _ => return Err(AppError::BadRequest(format!(
                "Unknown preset '{}'. Use: curve, curve_conservative, default",
                preset
            ))),
        }
    } else {
        // Merge with existing config
        let mut config = current_position.exit_config.clone();
        if let Some(sl) = request.stop_loss_percent {
            config.stop_loss_percent = Some(sl);
        }
        if let Some(tp) = request.take_profit_percent {
            config.take_profit_percent = Some(tp);
        }
        if let Some(ts) = request.trailing_stop_percent {
            config.trailing_stop_percent = Some(ts);
        }
        if let Some(tl) = request.time_limit_minutes {
            config.time_limit_minutes = Some(tl);
        }
        config
    };

    let updated_position = state
        .position_manager
        .update_position_exit_config(position_id, new_config)
        .await?;

    let new_config_summary = exit_config_to_summary(&updated_position.exit_config);

    info!(
        position_id = %position_id,
        mint = %updated_position.token_mint[..8.min(updated_position.token_mint.len())],
        "✅ Updated exit config"
    );

    Ok(Json(UpdateExitConfigResponse {
        success: true,
        position_id,
        old_config,
        new_config: new_config_summary,
    }))
}

/// Update all open positions to use a preset exit config
pub async fn update_all_positions_exit_config(
    State(state): State<AppState>,
    Json(request): Json<UpdateExitConfigRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let preset = request.preset.as_deref().unwrap_or("curve");

    let new_config = match preset {
        "curve" => ExitConfig::for_curve_bonding(),
        "curve_conservative" => ExitConfig::for_curve_bonding_conservative(),
        "default" => ExitConfig::default(),
        _ => return Err(AppError::BadRequest(format!(
            "Unknown preset '{}'. Use: curve, curve_conservative, default",
            preset
        ))),
    };

    let positions = state.position_manager.get_open_positions().await;
    let mut updated = 0;
    let mut failed = 0;

    for position in &positions {
        match state
            .position_manager
            .update_position_exit_config(position.id, new_config.clone())
            .await
        {
            Ok(_) => updated += 1,
            Err(e) => {
                tracing::warn!(
                    position_id = %position.id,
                    error = %e,
                    "Failed to update exit config"
                );
                failed += 1;
            }
        }
    }

    info!(
        preset = preset,
        updated = updated,
        failed = failed,
        "✅ Bulk updated exit configs"
    );

    Ok(Json(serde_json::json!({
        "success": true,
        "preset": preset,
        "positions_updated": updated,
        "positions_failed": failed,
        "new_config": {
            "stop_loss_percent": new_config.stop_loss_percent,
            "take_profit_percent": new_config.take_profit_percent,
            "trailing_stop_percent": new_config.trailing_stop_percent,
            "time_limit_minutes": new_config.time_limit_minutes,
        }
    })))
}

#[derive(Debug, Serialize)]
pub struct EmergencyExitResponse {
    pub positions_exited: usize,
    pub positions_failed: usize,
    pub total_positions: usize,
    pub message: String,
    pub results: Vec<EmergencyExitResult>,
}

#[derive(Debug, Serialize)]
pub struct EmergencyExitResult {
    pub position_id: Uuid,
    pub token_mint: String,
    pub token_symbol: Option<String>,
    pub success: bool,
    pub error: Option<String>,
}

pub async fn emergency_close_all(
    State(state): State<AppState>,
) -> Result<Json<EmergencyExitResponse>, AppError> {
    info!("🚨 EMERGENCY EXIT ALL - Force selling all positions at market!");

    let positions = state.position_manager.get_open_positions().await;
    let total = positions.len();

    if total == 0 {
        return Ok(Json(EmergencyExitResponse {
            positions_exited: 0,
            positions_failed: 0,
            total_positions: 0,
            message: "No open positions to exit".to_string(),
            results: vec![],
        }));
    }

    let mut results = Vec::new();
    let mut exited = 0;
    let mut failed = 0;

    for position in positions {
        info!(
            "🔴 Force exiting: {} ({}) - {} tokens",
            position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8]),
            &position.token_mint[..12],
            position.entry_token_amount
        );

        match state
            .position_monitor
            .trigger_manual_exit(position.id, 100.0, &state.dev_signer)
            .await
        {
            Ok(_) => {
                exited += 1;
                if let Some(released) = state.capital_manager.release_capital(position.id).await {
                    info!(
                        "💸 Released {} SOL for {}",
                        released as f64 / 1_000_000_000.0,
                        position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8])
                    );
                }
                results.push(EmergencyExitResult {
                    position_id: position.id,
                    token_mint: position.token_mint.clone(),
                    token_symbol: position.token_symbol.clone(),
                    success: true,
                    error: None,
                });
            }
            Err(e) => {
                failed += 1;
                tracing::error!(
                    "❌ Failed to exit {}: {}",
                    position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8]),
                    e
                );
                results.push(EmergencyExitResult {
                    position_id: position.id,
                    token_mint: position.token_mint.clone(),
                    token_symbol: position.token_symbol.clone(),
                    success: false,
                    error: Some(e.to_string()),
                });
            }
        }
    }

    let message = if failed == 0 {
        format!("🚨 Emergency exit completed: {} positions sold", exited)
    } else {
        format!(
            "⚠️ Emergency exit partial: {} sold, {} failed",
            exited, failed
        )
    };

    info!("{}", message);

    Ok(Json(EmergencyExitResponse {
        positions_exited: exited,
        positions_failed: failed,
        total_positions: total,
        message,
        results,
    }))
}

#[derive(Debug, Serialize)]
pub struct SellAllResponse {
    pub tokens_found: usize,
    pub tokens_sold: usize,
    pub tokens_failed: usize,
    pub total_sol_received: f64,
    pub results: Vec<SellTokenResult>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct SellTokenResult {
    pub mint: String,
    pub symbol: Option<String>,
    pub balance: f64,
    pub success: bool,
    pub sol_received: Option<f64>,
    pub error: Option<String>,
    pub signature: Option<String>,
}

/// Sell ALL tokens in wallet to SOL (regardless of whether they're tracked positions)
/// This discovers all tokens via DAS, then sells each one via bonding curve or Jupiter
pub async fn sell_all_wallet_tokens(
    State(state): State<AppState>,
) -> Result<Json<SellAllResponse>, AppError> {
    use crate::execution::position_manager::BaseCurrency;

    let wallet_address = state.dev_signer.get_address()
        .ok_or_else(|| AppError::Validation("No wallet configured".to_string()))?;

    info!("🔥 SELL ALL TOKENS - Liquidating entire wallet to SOL");
    info!("   Wallet: {}", &wallet_address[..12]);

    // Step 1: Get all tokens in wallet via DAS
    let token_accounts = state.helius_das
        .get_token_accounts_by_owner(&wallet_address)
        .await
        .map_err(|e| AppError::ExternalApi(format!("Failed to fetch wallet tokens: {}", e)))?;

    // Filter out base currencies (SOL, USDC, etc.)
    let sellable_tokens: Vec<_> = token_accounts
        .into_iter()
        .filter(|t| !BaseCurrency::is_base_currency(&t.mint) && t.ui_amount > 0.0)
        .collect();

    let total_found = sellable_tokens.len();
    info!("📊 Found {} non-base tokens to sell", total_found);

    if total_found == 0 {
        return Ok(Json(SellAllResponse {
            tokens_found: 0,
            tokens_sold: 0,
            tokens_failed: 0,
            total_sol_received: 0.0,
            results: vec![],
            message: "No tokens to sell - wallet only contains base currencies".to_string(),
        }));
    }

    let mut results = Vec::new();
    let mut sold = 0;
    let mut failed = 0;
    let mut total_sol = 0.0;

    for token in sellable_tokens {
        info!("💰 Selling {} ({:.2} tokens)", &token.mint[..12], token.ui_amount);

        // Try to sell via bonding curve first (for pump.fun/moonshot tokens)
        let sell_result = sell_token_to_sol(
            &state,
            &token.mint,
            token.ui_amount,
            token.decimals,
            &wallet_address,
        ).await;

        match sell_result {
            Ok((sol_received, signature)) => {
                sold += 1;
                total_sol += sol_received;
                info!("   ✅ Sold for {:.6} SOL (tx: {}...)", sol_received, &signature[..12]);
                results.push(SellTokenResult {
                    mint: token.mint,
                    symbol: None,
                    balance: token.ui_amount,
                    success: true,
                    sol_received: Some(sol_received),
                    error: None,
                    signature: Some(signature),
                });
            }
            Err(e) => {
                failed += 1;
                tracing::warn!("   ❌ Failed to sell {}: {}", &token.mint[..12], e);
                results.push(SellTokenResult {
                    mint: token.mint,
                    symbol: None,
                    balance: token.ui_amount,
                    success: false,
                    sol_received: None,
                    error: Some(e.to_string()),
                    signature: None,
                });
            }
        }
    }

    let message = if failed == 0 {
        format!("🔥 Sold all {} tokens for {:.6} SOL total", sold, total_sol)
    } else {
        format!("⚠️ Sold {}/{} tokens for {:.6} SOL ({} failed)", sold, total_found, total_sol, failed)
    };

    info!("{}", message);

    Ok(Json(SellAllResponse {
        tokens_found: total_found,
        tokens_sold: sold,
        tokens_failed: failed,
        total_sol_received: total_sol,
        results,
        message,
    }))
}

/// Helper to sell a single token to SOL via bonding curve or Jupiter
async fn sell_token_to_sol(
    state: &AppState,
    mint: &str,
    amount: f64,
    decimals: u8,
    wallet_address: &str,
) -> Result<(f64, String), AppError> {
    use crate::execution::{CurveSellParams, SwapParams};
    use crate::wallet::SignRequest;

    // Convert UI amount to raw amount
    let raw_amount = (amount * 10f64.powi(decimals as i32)) as u64;

    // Try bonding curve sell first (pump.fun/moonshot)
    match state.on_chain_fetcher.get_bonding_curve_state(mint).await {
        Ok(curve_state) if !curve_state.is_complete => {
            // Token is still on bonding curve - sell via curve
            info!("   📈 Token on bonding curve, selling via pump.fun");
            sell_via_bonding_curve(state, mint, raw_amount, wallet_address).await
        }
        Ok(_) => {
            // Token has graduated - sell via Jupiter
            info!("   🪐 Token graduated, selling via Jupiter");
            sell_via_jupiter(state, mint, raw_amount, wallet_address).await
        }
        Err(_) => {
            // Not a bonding curve token - try Jupiter directly
            info!("   🪐 Not a bonding curve token, selling via Jupiter");
            sell_via_jupiter(state, mint, raw_amount, wallet_address).await
        }
    }
}

/// Sell token via bonding curve (pump.fun)
async fn sell_via_bonding_curve(
    state: &AppState,
    mint: &str,
    raw_amount: u64,
    wallet_address: &str,
) -> Result<(f64, String), AppError> {
    use crate::execution::CurveSellParams;
    use crate::wallet::SignRequest;

    const MAX_RETRIES: u32 = 3;
    const MAX_SLIPPAGE: u16 = 2500; // 25% max
    let mut slippage: u16 = 1000; // Start at 10%
    let mut last_error = String::new();

    for attempt in 0..=MAX_RETRIES {
        if attempt > 0 {
            // Increase slippage by 50% each retry
            slippage = (slippage as u32 * 150 / 100).min(MAX_SLIPPAGE as u32) as u16;
            tracing::info!(
                mint = &mint[..12],
                attempt = attempt,
                slippage_bps = slippage,
                "🔄 Retrying curve sell with increased slippage"
            );
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        }

        let sell_params = CurveSellParams {
            mint: mint.to_string(),
            token_amount: raw_amount,
            slippage_bps: slippage,
            user_wallet: wallet_address.to_string(),
        };

        let build_result = match state.curve_builder.build_pump_fun_sell(&sell_params).await {
            Ok(r) => r,
            Err(e) => {
                last_error = format!("Failed to build curve sell tx: {}", e);
                continue;
            }
        };

        let expected_sol = build_result.expected_sol_out.unwrap_or(0);

        // Sign the transaction
        let signed = match state.dev_signer.sign_transaction(SignRequest {
            transaction_base64: build_result.transaction_base64,
            estimated_amount_lamports: expected_sol,
            estimated_profit_lamports: None,
            description: format!("Curve sell {} ({}bps)", &mint[..12], slippage),
            edge_id: None,
        }).await {
            Ok(s) => s,
            Err(e) => {
                last_error = format!("Failed to sign tx: {}", e);
                continue;
            }
        };

        if !signed.success {
            last_error = format!(
                "Failed to sign: {}",
                signed.error.unwrap_or_else(|| "Unknown error".to_string())
            );
            continue;
        }

        let signed_tx = match signed.signed_transaction_base64 {
            Some(tx) => tx,
            None => {
                last_error = "Signed transaction missing".to_string();
                continue;
            }
        };

        // Send via Helius
        match state.helius_sender.send_transaction(&signed_tx, true).await {
            Ok(signature) => {
                let sol_received = expected_sol as f64 / 1_000_000_000.0;
                return Ok((sol_received, signature));
            }
            Err(e) => {
                last_error = format!("Failed to send tx: {}", e);
                let is_slippage_error = last_error.contains("6003")
                    || last_error.to_lowercase().contains("slippage");

                if is_slippage_error && attempt < MAX_RETRIES {
                    tracing::warn!(
                        mint = &mint[..12],
                        error = %last_error,
                        "⚠️ Slippage error, will retry"
                    );
                    continue;
                }
            }
        }
    }

    Err(AppError::ExternalApi(last_error))
}

/// Sell token via Jupiter (for graduated tokens or non-bonding-curve tokens)
async fn sell_via_jupiter(
    state: &AppState,
    mint: &str,
    raw_amount: u64,
    wallet_address: &str,
) -> Result<(f64, String), AppError> {
    use crate::execution::SwapParams;
    use crate::wallet::SignRequest;

    const SOL_MINT: &str = "So11111111111111111111111111111111111111112";

    // Build Jupiter swap: Token -> SOL
    let swap_params = SwapParams {
        input_mint: mint.to_string(),
        output_mint: SOL_MINT.to_string(),
        amount_lamports: raw_amount,
        slippage_bps: 1000, // 10% slippage for emergency sell
        user_public_key: wallet_address.to_string(),
    };

    let build_result = state.tx_builder.build_jupiter_swap(
        &swap_params,
        uuid::Uuid::new_v4(), // Generate a temporary edge ID
    ).await.map_err(|e| AppError::ExternalApi(format!("Jupiter swap build failed: {}", e)))?;

    let expected_sol = build_result.route_info.out_amount;

    // Sign the transaction
    let signed = state.dev_signer.sign_transaction(SignRequest {
        transaction_base64: build_result.transaction_base64,
        estimated_amount_lamports: expected_sol,
        estimated_profit_lamports: None,
        description: format!("Jupiter sell {}", &mint[..12]),
        edge_id: Some(build_result.edge_id),
    }).await.map_err(|e| AppError::ExternalApi(format!("Failed to sign tx: {}", e)))?;

    if !signed.success {
        return Err(AppError::ExternalApi(format!(
            "Failed to sign: {}",
            signed.error.unwrap_or_else(|| "Unknown error".to_string())
        )));
    }

    let signed_tx = signed.signed_transaction_base64
        .ok_or_else(|| AppError::ExternalApi("Signed transaction missing".to_string()))?;

    // Send via Helius
    let signature = state.helius_sender.send_transaction(
        &signed_tx,
        true, // skip preflight for speed
    ).await.map_err(|e| AppError::ExternalApi(format!("Failed to send tx: {}", e)))?;

    let sol_received = expected_sol as f64 / 1_000_000_000.0;
    Ok((sol_received, signature))
}

#[derive(Debug, Serialize)]
pub struct MonitorStatusResponse {
    pub monitoring_active: bool,
    pub price_check_interval_secs: u64,
    pub exit_slippage_bps: u16,
    pub active_positions: u32,
    pub pending_exit_signals: usize,
}

pub async fn get_monitor_status(
    State(state): State<AppState>,
) -> Result<Json<MonitorStatusResponse>, AppError> {
    let stats = state.position_monitor.get_position_stats().await;
    let signals = state.position_manager.get_pending_exit_signals().await;

    Ok(Json(MonitorStatusResponse {
        monitoring_active: true,
        price_check_interval_secs: 30,
        exit_slippage_bps: 100,
        active_positions: stats.active_positions,
        pending_exit_signals: signals.len(),
    }))
}

#[derive(Debug, Deserialize)]
pub struct StartMonitorRequest {
    pub price_check_interval_secs: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct StartMonitorResponse {
    pub success: bool,
    pub message: String,
}

pub async fn start_monitor(
    State(state): State<AppState>,
) -> Result<Json<StartMonitorResponse>, AppError> {
    state.start_position_monitor();

    Ok(Json(StartMonitorResponse {
        success: true,
        message: "Position monitor started".to_string(),
    }))
}

#[derive(Debug, Serialize)]
pub struct StopMonitorResponse {
    pub success: bool,
    pub message: String,
}

pub async fn stop_monitor(
    State(_state): State<AppState>,
) -> Result<Json<StopMonitorResponse>, AppError> {
    Ok(Json(StopMonitorResponse {
        success: true,
        message: "Position monitor stop requested (note: monitor will stop on next cycle)".to_string(),
    }))
}

#[derive(Debug, Serialize)]
pub struct PnLSummary {
    pub today_sol: f64,
    pub week_sol: f64,
    pub total_sol: f64,
    pub wins: u32,
    pub losses: u32,
    pub win_rate: f64,
    pub avg_hold_minutes: f64,
    pub total_trades: u32,
    pub active_positions: u32,
    pub take_profits: u32,
    pub take_profit_pnl: f64,
    pub stop_losses: u32,
    pub stop_loss_pnl: f64,
    pub trailing_stops: u32,
    pub trailing_stop_pnl: f64,
    pub manual_exits: u32,
    pub manual_pnl: f64,
    pub best_trade: Option<BestWorstTrade>,
    pub worst_trade: Option<BestWorstTrade>,
    pub recent_trades: Vec<RecentTradeInfo>,
    pub active_strategies: Vec<ActiveStrategy>,
}

#[derive(Debug, Serialize)]
pub struct BestWorstTrade {
    pub symbol: String,
    pub pnl: f64,
}

#[derive(Debug, Serialize)]
pub struct RecentTradeInfo {
    pub symbol: String,
    pub pnl: f64,
    pub pnl_percent: f64,
    pub reason: String,
    pub time_ago: String,
}

#[derive(Debug, Serialize)]
pub struct ActiveStrategy {
    pub symbol: String,
    pub mint: String,
    pub entry_sol: f64,
    pub current_pnl_percent: f64,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub trailing_stop: Option<f64>,
    pub time_limit_mins: Option<u32>,
    pub hold_time_mins: i64,
}

pub async fn get_pnl_summary(
    State(state): State<AppState>,
) -> Result<Json<PnLSummary>, AppError> {
    use crate::database::PositionRepository;

    let repo = PositionRepository::new(state.db_pool.clone());
    let db_stats = match repo.get_pnl_stats().await {
        Ok(stats) => stats,
        Err(e) => {
            tracing::warn!("Failed to get PnL stats from DB: {}", e);
            Default::default()
        }
    };
    let recent = repo.get_recent_trades(5).await.unwrap_or_default();

    let manager_stats = state.position_manager.get_stats().await;

    let wins = db_stats.take_profits + db_stats.trailing_stops;
    let losses = db_stats.stop_losses;
    let win_rate = if db_stats.total_trades > 0 {
        (wins as f64 / db_stats.total_trades as f64) * 100.0
    } else {
        0.0
    };

    let recent_trades: Vec<RecentTradeInfo> = recent.into_iter().map(|t| {
        let pnl_percent = if t.entry_sol > 0.0 {
            (t.pnl / t.entry_sol) * 100.0
        } else {
            0.0
        };
        let time_ago = t.time.map(|dt| {
            let mins = (chrono::Utc::now() - dt).num_minutes();
            if mins < 60 {
                format!("{}m ago", mins)
            } else if mins < 1440 {
                format!("{}h ago", mins / 60)
            } else {
                format!("{}d ago", mins / 1440)
            }
        }).unwrap_or_else(|| "?".to_string());
        RecentTradeInfo {
            symbol: t.symbol,
            pnl: t.pnl,
            pnl_percent,
            reason: t.reason,
            time_ago,
        }
    }).collect();

    // Get active positions with their strategies
    let open_positions = state.position_manager.get_open_positions().await;
    let active_strategies: Vec<ActiveStrategy> = open_positions.into_iter().map(|p| {
        let hold_time_mins = (chrono::Utc::now() - p.entry_time).num_minutes();
        ActiveStrategy {
            symbol: p.token_symbol.unwrap_or_else(|| p.token_mint[..8].to_string()),
            mint: p.token_mint[..12].to_string(),
            entry_sol: p.entry_amount_base,
            current_pnl_percent: p.unrealized_pnl_percent,
            stop_loss: p.exit_config.stop_loss_percent,
            take_profit: p.exit_config.take_profit_percent,
            trailing_stop: p.exit_config.trailing_stop_percent,
            time_limit_mins: p.exit_config.time_limit_minutes,
            hold_time_mins,
        }
    }).collect();

    Ok(Json(PnLSummary {
        today_sol: db_stats.today_pnl,
        week_sol: db_stats.week_pnl,
        total_sol: db_stats.total_pnl,
        wins,
        losses,
        win_rate,
        avg_hold_minutes: db_stats.avg_hold_minutes,
        total_trades: db_stats.total_trades,
        active_positions: manager_stats.active_positions,
        take_profits: db_stats.take_profits,
        take_profit_pnl: db_stats.take_profit_pnl,
        stop_losses: db_stats.stop_losses,
        stop_loss_pnl: db_stats.stop_loss_pnl,
        trailing_stops: db_stats.trailing_stops,
        trailing_stop_pnl: db_stats.trailing_stop_pnl,
        manual_exits: db_stats.manual_exits,
        manual_pnl: db_stats.manual_pnl,
        best_trade: db_stats.best_trade_symbol.map(|s| BestWorstTrade {
            symbol: s,
            pnl: db_stats.best_trade_pnl,
        }),
        worst_trade: db_stats.worst_trade_symbol.map(|s| BestWorstTrade {
            symbol: s,
            pnl: db_stats.worst_trade_pnl,
        }),
        recent_trades,
        active_strategies,
    }))
}

#[derive(Debug, Serialize)]
pub struct ExposureResponse {
    pub sol_exposure: f64,
    pub usdc_exposure: f64,
    pub usdt_exposure: f64,
    pub total_exposure_sol: f64,
}

pub async fn get_exposure(
    State(state): State<AppState>,
) -> Result<Json<ExposureResponse>, AppError> {
    let sol = state.position_manager.get_total_exposure_by_base(BaseCurrency::Sol).await;
    let usdc = state.position_manager.get_total_exposure_by_base(BaseCurrency::Usdc).await;
    let usdt = state.position_manager.get_total_exposure_by_base(BaseCurrency::Usdt).await;

    Ok(Json(ExposureResponse {
        sol_exposure: sol,
        usdc_exposure: usdc,
        usdt_exposure: usdt,
        total_exposure_sol: sol + usdc + usdt,
    }))
}

#[derive(Debug, Serialize)]
pub struct ReconciliationResponse {
    pub tracked_positions: usize,
    pub discovered_tokens: Vec<WalletTokenHolding>,
    pub orphaned_positions: Vec<String>,
    pub message: String,
}

pub async fn reconcile_wallet(
    State(state): State<AppState>,
) -> Result<Json<ReconciliationResponse>, AppError> {
    let wallet_address = state.dev_signer.get_address()
        .ok_or_else(|| AppError::Validation("No wallet configured".to_string()))?;

    info!("🔄 Starting wallet reconciliation for {}", &wallet_address[..8]);

    let token_accounts = state.helius_das
        .get_token_accounts_by_owner(&wallet_address)
        .await
        .map_err(|e| AppError::ExternalApi(format!("Failed to fetch wallet token accounts: {}", e)))?;

    let wallet_tokens: Vec<WalletTokenHolding> = token_accounts
        .into_iter()
        .map(|account| {
            WalletTokenHolding {
                mint: account.mint,
                symbol: None,
                balance: account.ui_amount,
                decimals: account.decimals,
            }
        })
        .collect();

    info!("📊 Found {} tokens with non-zero balance in wallet", wallet_tokens.len());

    let result = state.position_manager.reconcile_wallet_tokens(&wallet_tokens).await;

    for position_id in &result.orphaned_positions {
        if let Err(e) = state.position_manager.mark_position_orphaned(*position_id).await {
            tracing::warn!("Failed to mark position {} as orphaned: {}", position_id, e);
        }
    }

    let mut created_positions = 0;
    let mut reactivated_positions = 0;
    for token in &result.discovered_tokens {
        if crate::execution::position_manager::BaseCurrency::is_base_currency(&token.mint) {
            continue;
        }

        info!(
            "🔍 Processing discovered token {} ({:.6} balance)",
            &token.mint[..12],
            token.balance
        );

        let estimated_price = match state.on_chain_fetcher.get_bonding_curve_state(&token.mint).await {
            Ok(curve_state) => {
                if curve_state.virtual_token_reserves > 0 {
                    let price = curve_state.virtual_sol_reserves as f64 / curve_state.virtual_token_reserves as f64;
                    info!("   💰 On-chain price: {:.12} SOL/token", price);
                    price
                } else {
                    info!("   ⚠️ Zero token reserves, using estimate");
                    0.0000001
                }
            }
            Err(_) => {
                info!("   ⚠️ Could not fetch price, using estimate");
                0.0000001
            }
        };

        let exit_config = match state.metrics_collector.calculate_metrics(&token.mint, "pump_fun").await {
            Ok(metrics) => {
                let volume = metrics.volume_24h;
                let holders = metrics.holder_count;
                info!(
                    "   📊 Metrics: vol={:.2} SOL, holders={}",
                    volume, holders
                );
                crate::execution::ExitConfig::for_discovered_with_metrics(volume, holders)
            }
            Err(_) => {
                info!("   ⚠️ No metrics available, using default exit strategy");
                crate::execution::ExitConfig::for_discovered_token()
            }
        };

        if let Some(orphaned_position) = state.position_manager.get_orphaned_position_by_mint(&token.mint).await {
            info!(
                "   ♻️ Found orphaned position {} - reactivating with new exit strategy",
                orphaned_position.id
            );

            match state.position_manager.reactivate_orphaned_position(
                orphaned_position,
                token.balance,
                estimated_price,
                exit_config,
            ).await {
                Ok(position) => {
                    reactivated_positions += 1;
                    info!(
                        "   ✅ Reactivated position {} for {} (SL: {:?}%, TP: {:?}%)",
                        position.id,
                        &token.mint[..12],
                        position.exit_config.stop_loss_percent,
                        position.exit_config.take_profit_percent
                    );
                }
                Err(e) => {
                    tracing::warn!("   ❌ Failed to reactivate position for {}: {}", &token.mint[..12], e);
                }
            }
        } else {
            // Cap estimated entry to reasonable maximum for discovered positions
            // We don't know actual purchase price, so use conservative estimate
            const MAX_DISCOVERED_ENTRY_SOL: f64 = 0.1;
            const DEFAULT_DISCOVERED_ENTRY_SOL: f64 = 0.02;

            let raw_estimated_entry = token.balance * estimated_price;
            let estimated_entry_sol = if raw_estimated_entry > MAX_DISCOVERED_ENTRY_SOL {
                // Likely inflated estimate due to large token balance, use default
                info!("   📉 Raw estimate {:.4} SOL too high, capping to {:.4} SOL",
                    raw_estimated_entry, DEFAULT_DISCOVERED_ENTRY_SOL);
                DEFAULT_DISCOVERED_ENTRY_SOL
            } else if raw_estimated_entry < 0.001 {
                // Too low, use default
                DEFAULT_DISCOVERED_ENTRY_SOL
            } else {
                raw_estimated_entry
            };

            match state.position_manager.create_discovered_position_with_config(
                token,
                estimated_price,
                estimated_entry_sol,
                exit_config,
            ).await {
                Ok(position) => {
                    created_positions += 1;
                    info!(
                        "   ✅ Created new position {} for {} (SL: {:?}%, TP: {:?}%)",
                        position.id,
                        &token.mint[..12],
                        position.exit_config.stop_loss_percent,
                        position.exit_config.take_profit_percent
                    );
                }
                Err(e) => {
                    tracing::warn!("   ❌ Failed to create position for {}: {}", &token.mint[..12], e);
                }
            }
        }
    }

    let orphaned_ids: Vec<String> = result.orphaned_positions.iter().map(|id| id.to_string()).collect();

    let message = format!(
        "Reconciliation complete: {} tracked, {} discovered ({} new, {} reactivated), {} orphaned",
        result.tracked_positions,
        result.discovered_tokens.len(),
        created_positions,
        reactivated_positions,
        result.orphaned_positions.len()
    );

    info!("📊 {}", message);

    Ok(Json(ReconciliationResponse {
        tracked_positions: result.tracked_positions,
        discovered_tokens: result.discovered_tokens,
        orphaned_positions: orphaned_ids,
        message,
    }))
}

#[derive(Debug, Serialize)]
pub struct PositionHistoryItem {
    pub id: String,
    pub token_mint: String,
    pub token_symbol: Option<String>,
    pub status: String,
    pub entry_sol: f64,
    pub realized_pnl: Option<f64>,
    pub exit_reason: Option<String>,
    pub exit_config: ExitConfigSummary,
    pub entry_time: chrono::DateTime<chrono::Utc>,
    pub exit_time: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize)]
pub struct ExitConfigSummary {
    pub stop_loss_percent: Option<f64>,
    pub take_profit_percent: Option<f64>,
    pub trailing_stop_percent: Option<f64>,
    pub time_limit_minutes: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct PositionHistoryResponse {
    pub positions: Vec<PositionHistoryItem>,
    pub total_count: usize,
}

pub async fn get_position_history(
    State(state): State<AppState>,
) -> Result<Json<PositionHistoryResponse>, AppError> {
    use crate::database::PositionRepository;

    let repo = PositionRepository::new(state.db_pool.clone());
    let rows = repo.get_all_positions(50).await
        .map_err(|e| AppError::Database(format!("Failed to fetch positions: {}", e)))?;

    let positions: Vec<PositionHistoryItem> = rows.into_iter().map(|row| {
        let exit_config: crate::execution::ExitConfig = serde_json::from_value(row.exit_config.clone())
            .unwrap_or_default();

        PositionHistoryItem {
            id: row.id.to_string(),
            token_mint: row.token_mint.clone(),
            token_symbol: row.token_symbol.clone(),
            status: row.status.clone(),
            entry_sol: row.entry_amount_base.to_string().parse().unwrap_or(0.0),
            realized_pnl: row.realized_pnl.map(|d| d.to_string().parse().unwrap_or(0.0)),
            exit_reason: row.exit_reason.clone(),
            exit_config: ExitConfigSummary {
                stop_loss_percent: exit_config.stop_loss_percent,
                take_profit_percent: exit_config.take_profit_percent,
                trailing_stop_percent: exit_config.trailing_stop_percent,
                time_limit_minutes: exit_config.time_limit_minutes,
            },
            entry_time: row.entry_time,
            exit_time: row.exit_time,
        }
    }).collect();

    let total_count = positions.len();

    Ok(Json(PositionHistoryResponse {
        positions,
        total_count,
    }))
}
