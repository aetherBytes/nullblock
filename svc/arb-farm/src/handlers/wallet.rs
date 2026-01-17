use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::error::AppResult;
use crate::server::AppState;
use crate::wallet::{ArbFarmPolicy, WalletStatus, SignRequest, SignResult};
use crate::wallet::turnkey::WalletSetupRequest;

#[derive(Debug, Serialize)]
pub struct WalletStatusResponse {
    pub status: WalletStatus,
}

pub async fn get_wallet_status(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let status = state.turnkey_signer.get_status().await;
    (StatusCode::OK, Json(WalletStatusResponse { status }))
}

#[derive(Debug, Deserialize)]
pub struct SetupWalletRequest {
    pub user_wallet_address: String,
    pub wallet_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SetupWalletResponse {
    pub success: bool,
    pub wallet_status: Option<WalletStatus>,
    pub error: Option<String>,
}

pub async fn setup_wallet(
    State(state): State<AppState>,
    Json(request): Json<SetupWalletRequest>,
) -> impl IntoResponse {
    let setup_request = WalletSetupRequest {
        user_wallet_address: request.user_wallet_address,
        wallet_name: request.wallet_name,
    };

    match state.turnkey_signer.setup_wallet(setup_request).await {
        Ok(result) => {
            if result.success {
                let status = state.turnkey_signer.get_status().await;
                (
                    StatusCode::OK,
                    Json(SetupWalletResponse {
                        success: true,
                        wallet_status: Some(status),
                        error: None,
                    }),
                )
            } else {
                (
                    StatusCode::BAD_REQUEST,
                    Json(SetupWalletResponse {
                        success: false,
                        wallet_status: None,
                        error: result.error,
                    }),
                )
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(SetupWalletResponse {
                success: false,
                wallet_status: None,
                error: Some(e.to_string()),
            }),
        ),
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdatePolicyRequest {
    pub max_transaction_amount_sol: Option<f64>,
    pub daily_volume_limit_sol: Option<f64>,
    pub max_transactions_per_day: Option<u32>,
    pub require_simulation: Option<bool>,
    pub min_profit_threshold_sol: Option<f64>,
}

pub async fn update_policy(
    State(state): State<AppState>,
    Json(request): Json<UpdatePolicyRequest>,
) -> impl IntoResponse {
    let current_status = state.turnkey_signer.get_status().await;
    let mut policy = current_status.policy;

    if let Some(max_tx) = request.max_transaction_amount_sol {
        policy.max_transaction_amount_lamports = (max_tx * 1_000_000_000.0) as u64;
    }
    if let Some(daily_limit) = request.daily_volume_limit_sol {
        policy.daily_volume_limit_lamports = (daily_limit * 1_000_000_000.0) as u64;
    }
    if let Some(max_txs) = request.max_transactions_per_day {
        policy.max_transactions_per_day = max_txs;
    }
    if let Some(require_sim) = request.require_simulation {
        policy.require_simulation = require_sim;
    }
    if let Some(min_profit) = request.min_profit_threshold_sol {
        policy.min_profit_threshold_lamports = (min_profit * 1_000_000_000.0) as u64;
    }

    match state.turnkey_signer.update_policy(policy).await {
        Ok(_) => {
            let status = state.turnkey_signer.get_status().await;
            (StatusCode::OK, Json(serde_json::json!({
                "success": true,
                "policy": status.policy,
            })))
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "success": false,
            "error": e.to_string(),
        }))),
    }
}

#[derive(Debug, Serialize)]
pub struct BalanceResponse {
    pub balance_lamports: u64,
    pub balance_sol: f64,
}

pub async fn get_balance(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let status = state.turnkey_signer.get_status().await;

    match status.wallet_address {
        Some(address) => {
            // Fetch balance from RPC
            match fetch_balance(&state.config.rpc_url, &address).await {
                Ok(balance) => {
                    // Update cached balance
                    let _ = state.turnkey_signer.update_balance(balance).await;

                    (StatusCode::OK, Json(BalanceResponse {
                        balance_lamports: balance,
                        balance_sol: balance as f64 / 1_000_000_000.0,
                    }))
                }
                Err(e) => {
                    // Return cached balance if RPC fails
                    if let Some(cached) = status.balance_lamports {
                        (StatusCode::OK, Json(BalanceResponse {
                            balance_lamports: cached,
                            balance_sol: cached as f64 / 1_000_000_000.0,
                        }))
                    } else {
                        (StatusCode::SERVICE_UNAVAILABLE, Json(BalanceResponse {
                            balance_lamports: 0,
                            balance_sol: 0.0,
                        }))
                    }
                }
            }
        }
        None => (StatusCode::BAD_REQUEST, Json(BalanceResponse {
            balance_lamports: 0,
            balance_sol: 0.0,
        })),
    }
}

async fn fetch_balance(rpc_url: &str, address: &str) -> AppResult<u64> {
    let client = reqwest::Client::new();

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getBalance",
        "params": [address]
    });

    let response = client
        .post(rpc_url)
        .json(&request)
        .send()
        .await
        .map_err(|e| crate::error::AppError::ExternalApi(format!("RPC error: {}", e)))?;

    let result: serde_json::Value = response.json().await
        .map_err(|e| crate::error::AppError::ExternalApi(format!("Parse error: {}", e)))?;

    let balance = result["result"]["value"]
        .as_u64()
        .unwrap_or(0);

    Ok(balance)
}

pub async fn disconnect_wallet(
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.turnkey_signer.disconnect().await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({
            "success": true,
            "message": "Wallet disconnected",
        }))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "success": false,
            "error": e.to_string(),
        }))),
    }
}

#[derive(Debug, Serialize)]
pub struct DevModeResponse {
    pub dev_mode_available: bool,
    pub wallet_address: Option<String>,
    pub has_private_key: bool,
}

pub async fn get_dev_mode(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let has_address = state.config.wallet_address.is_some();
    let has_key = state.config.wallet_private_key.is_some();

    (StatusCode::OK, Json(DevModeResponse {
        dev_mode_available: has_address && has_key,
        wallet_address: state.config.wallet_address.clone(),
        has_private_key: has_key,
    }))
}

pub async fn connect_dev_wallet(
    State(state): State<AppState>,
) -> impl IntoResponse {
    if !state.dev_signer.is_configured() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "success": false,
            "error": "Dev wallet not configured - ARB_FARM_WALLET_PRIVATE_KEY missing",
        })));
    }

    let wallet_address = match state.dev_signer.get_address() {
        Some(addr) => addr.to_string(),
        None => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "success": false,
                "error": "Dev signer has no wallet address",
            })));
        }
    };

    match state.dev_signer.connect().await {
        Ok(_) => {
            let dev_wallet_id = format!("dev_wallet_{}", wallet_address.chars().take(8).collect::<String>());
            let _ = state.turnkey_signer.set_wallet(wallet_address.clone(), dev_wallet_id).await;

            let status = state.dev_signer.get_status().await;
            (StatusCode::OK, Json(serde_json::json!({
                "success": true,
                "message": "Dev wallet connected with real signing enabled",
                "wallet_address": wallet_address,
                "signing_mode": "dev_private_key",
                "status": status,
            })))
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "success": false,
            "error": e.to_string(),
        }))),
    }
}

#[derive(Debug, Deserialize)]
pub struct TestSignRequest {
    pub amount_sol: f64,
    pub description: String,
}

pub async fn test_sign(
    State(state): State<AppState>,
    Json(request): Json<TestSignRequest>,
) -> impl IntoResponse {
    let sign_request = SignRequest {
        transaction_base64: "test_transaction".to_string(),
        estimated_amount_lamports: (request.amount_sol * 1_000_000_000.0) as u64,
        estimated_profit_lamports: Some(1_000_000), // Mock profit
        edge_id: None,
        description: request.description,
    };

    let dev_connected = state.dev_signer.get_status().await.is_connected;

    if dev_connected {
        match state.dev_signer.validate_transaction(&sign_request).await {
            Ok(_) => (StatusCode::OK, Json(serde_json::json!({
                "success": true,
                "message": "Transaction would pass policy checks",
                "signing_mode": "dev_private_key",
                "amount_sol": request.amount_sol,
            }))),
            Err(violation) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "success": false,
                "violation": violation,
            }))),
        }
    } else {
        match state.turnkey_signer.validate_transaction(&sign_request).await {
            Ok(_) => (StatusCode::OK, Json(serde_json::json!({
                "success": true,
                "message": "Transaction would pass policy checks",
                "signing_mode": "turnkey",
                "amount_sol": request.amount_sol,
            }))),
            Err(violation) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "success": false,
                "violation": violation,
            }))),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RealSignRequest {
    pub transaction_base64: String,
    pub amount_lamports: u64,
    pub estimated_profit_lamports: Option<i64>,
    pub edge_id: Option<uuid::Uuid>,
    pub description: String,
}

pub async fn sign_transaction(
    State(state): State<AppState>,
    Json(request): Json<RealSignRequest>,
) -> impl IntoResponse {
    let sign_request = SignRequest {
        transaction_base64: request.transaction_base64,
        estimated_amount_lamports: request.amount_lamports,
        estimated_profit_lamports: request.estimated_profit_lamports,
        edge_id: request.edge_id,
        description: request.description,
    };

    let dev_connected = state.dev_signer.get_status().await.is_connected;

    if dev_connected {
        match state.dev_signer.sign_transaction(sign_request).await {
            Ok(result) => {
                if result.success {
                    (StatusCode::OK, Json(serde_json::json!({
                        "success": true,
                        "signing_mode": "dev_private_key",
                        "signed_transaction_base64": result.signed_transaction_base64,
                        "signature": result.signature,
                    })))
                } else {
                    (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                        "success": false,
                        "error": result.error,
                        "policy_violation": result.policy_violation,
                    })))
                }
            }
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "success": false,
                "error": e.to_string(),
            }))),
        }
    } else {
        match state.turnkey_signer.sign_transaction(sign_request).await {
            Ok(result) => {
                if result.success {
                    (StatusCode::OK, Json(serde_json::json!({
                        "success": true,
                        "signing_mode": "turnkey",
                        "signed_transaction_base64": result.signed_transaction_base64,
                        "signature": result.signature,
                    })))
                } else {
                    (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                        "success": false,
                        "error": result.error,
                        "policy_violation": result.policy_violation,
                    })))
                }
            }
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "success": false,
                "error": e.to_string(),
            }))),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct DailyUsageResponse {
    pub date: String,
    pub volume_lamports: u64,
    pub volume_sol: f64,
    pub transaction_count: u32,
    pub remaining_volume_sol: f64,
    pub remaining_transactions: u32,
}

pub async fn get_daily_usage(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let status = state.turnkey_signer.get_status().await;
    let usage = &status.daily_usage;
    let policy = &status.policy;

    let remaining_volume = if policy.daily_volume_limit_lamports > usage.total_volume_lamports {
        policy.daily_volume_limit_lamports - usage.total_volume_lamports
    } else {
        0
    };

    let remaining_txs = if policy.max_transactions_per_day > usage.transaction_count {
        policy.max_transactions_per_day - usage.transaction_count
    } else {
        0
    };

    (StatusCode::OK, Json(DailyUsageResponse {
        date: usage.date.to_string(),
        volume_lamports: usage.total_volume_lamports,
        volume_sol: usage.total_volume_lamports as f64 / 1_000_000_000.0,
        transaction_count: usage.transaction_count,
        remaining_volume_sol: remaining_volume as f64 / 1_000_000_000.0,
        remaining_transactions: remaining_txs,
    }))
}

#[derive(Debug, Serialize)]
pub struct CapitalUsageResponse {
    pub total_balance_sol: f64,
    pub global_reserved_sol: f64,
    pub available_sol: f64,
    pub strategy_allocations: Vec<StrategyAllocationInfo>,
    pub active_reservations: Vec<ReservationInfo>,
}

#[derive(Debug, Serialize)]
pub struct StrategyAllocationInfo {
    pub strategy_id: uuid::Uuid,
    pub max_allocation_percent: f64,
    pub max_allocation_sol: f64,
    pub current_reserved_sol: f64,
    pub available_sol: f64,
    pub active_positions: u32,
    pub max_positions: u32,
}

#[derive(Debug, Serialize)]
pub struct ReservationInfo {
    pub position_id: uuid::Uuid,
    pub strategy_id: uuid::Uuid,
    pub amount_sol: f64,
    pub created_at: String,
}

pub async fn get_capital_usage(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let global_usage = state.capital_manager.get_global_usage().await;
    let strategy_usages = state.capital_manager.get_all_strategy_usage().await;

    let strategy_allocations: Vec<StrategyAllocationInfo> = strategy_usages
        .into_iter()
        .map(|usage| StrategyAllocationInfo {
            strategy_id: usage.strategy_id,
            max_allocation_percent: usage.max_allocation_percent,
            max_allocation_sol: usage.max_allocation_lamports as f64 / 1_000_000_000.0,
            current_reserved_sol: usage.current_reserved_lamports as f64 / 1_000_000_000.0,
            available_sol: usage.available_lamports as f64 / 1_000_000_000.0,
            active_positions: usage.active_positions,
            max_positions: usage.max_positions,
        })
        .collect();

    let reservations = state.capital_manager.get_active_reservations().await;
    let active_reservations: Vec<ReservationInfo> = reservations
        .into_iter()
        .map(|r| ReservationInfo {
            position_id: r.position_id,
            strategy_id: r.strategy_id,
            amount_sol: r.amount_lamports as f64 / 1_000_000_000.0,
            created_at: r.created_at.to_rfc3339(),
        })
        .collect();

    (StatusCode::OK, Json(CapitalUsageResponse {
        total_balance_sol: global_usage.total_balance_lamports as f64 / 1_000_000_000.0,
        global_reserved_sol: global_usage.global_reserved_lamports as f64 / 1_000_000_000.0,
        available_sol: global_usage.available_lamports as f64 / 1_000_000_000.0,
        strategy_allocations,
        active_reservations,
    }))
}

pub async fn sync_capital_balance(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let wallet_status = state.turnkey_signer.get_status().await;

    match wallet_status.wallet_address {
        Some(address) => {
            match fetch_balance(&state.config.rpc_url, &address).await {
                Ok(balance) => {
                    let _ = state.turnkey_signer.update_balance(balance).await;
                    state.capital_manager.update_total_balance(balance).await;

                    let global_usage = state.capital_manager.get_global_usage().await;
                    (StatusCode::OK, Json(serde_json::json!({
                        "success": true,
                        "balance_sol": balance as f64 / 1_000_000_000.0,
                        "available_sol": global_usage.available_lamports as f64 / 1_000_000_000.0,
                        "reserved_sol": global_usage.global_reserved_lamports as f64 / 1_000_000_000.0,
                    })))
                }
                Err(e) => (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to fetch balance: {}", e),
                }))),
            }
        }
        None => (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "success": false,
            "error": "No wallet connected",
        }))),
    }
}
