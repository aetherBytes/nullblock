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

    // Only validate, don't actually sign
    match state.turnkey_signer.validate_transaction(&sign_request).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({
            "success": true,
            "message": "Transaction would pass policy checks",
            "amount_sol": request.amount_sol,
        }))),
        Err(violation) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "success": false,
            "violation": violation,
        }))),
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
