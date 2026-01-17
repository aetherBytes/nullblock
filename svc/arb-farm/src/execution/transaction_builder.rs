use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::Edge;
use super::blockhash::BlockhashCache;
use super::position_manager::{BaseCurrency, ExitSignal, OpenPosition, SOL_MINT, USDC_MINT, USDT_MINT};

pub struct TransactionBuilder {
    client: reqwest::Client,
    jupiter_api_url: String,
    rpc_url: String,
    blockhash_cache: BlockhashCache,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapParams {
    pub input_mint: String,
    pub output_mint: String,
    pub amount_lamports: u64,
    pub slippage_bps: u16,
    pub user_public_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResult {
    pub edge_id: Uuid,
    pub transaction_base64: String,
    pub last_valid_block_height: u64,
    pub priority_fee_lamports: u64,
    pub estimated_compute_units: u64,
    pub route_info: RouteInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteInfo {
    pub input_mint: String,
    pub output_mint: String,
    pub in_amount: u64,
    pub out_amount: u64,
    pub price_impact_bps: i32,
    pub route_plan: serde_json::Value,
}

impl TransactionBuilder {
    pub fn new(jupiter_api_url: String, rpc_url: String) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            jupiter_api_url,
            rpc_url: rpc_url.clone(),
            blockhash_cache: BlockhashCache::new(rpc_url),
        }
    }

    pub async fn build_swap(
        &self,
        edge: &Edge,
        user_public_key: &str,
        slippage_bps: u16,
    ) -> AppResult<BuildResult> {
        let (input_mint, output_mint) = self.extract_swap_mints(edge)?;
        let amount_lamports = self.extract_swap_amount(edge)?;

        let params = SwapParams {
            input_mint: input_mint.clone(),
            output_mint: output_mint.clone(),
            amount_lamports,
            slippage_bps,
            user_public_key: user_public_key.to_string(),
        };

        self.build_jupiter_swap(&params, edge.id).await
    }

    pub async fn build_jupiter_swap(
        &self,
        params: &SwapParams,
        edge_id: Uuid,
    ) -> AppResult<BuildResult> {
        // Step 1: Get quote from Jupiter
        let quote = self.get_jupiter_quote(params).await?;

        // Step 2: Get swap transaction from Jupiter
        let swap_response = self.get_jupiter_swap_transaction(&quote, &params.user_public_key).await?;

        // Step 3: Get recent blockhash for reference
        let blockhash = self.blockhash_cache.get_blockhash().await?;

        // Extract amounts from quote
        let in_amount: u64 = quote.in_amount.parse().unwrap_or(params.amount_lamports);
        let out_amount: u64 = quote.out_amount.parse().unwrap_or(0);

        Ok(BuildResult {
            edge_id,
            transaction_base64: swap_response.swap_transaction,
            last_valid_block_height: blockhash.last_valid_block_height,
            priority_fee_lamports: swap_response.priority_fee_lamports.unwrap_or(0),
            estimated_compute_units: swap_response.compute_unit_limit.unwrap_or(200_000),
            route_info: RouteInfo {
                input_mint: quote.input_mint.clone(),
                output_mint: quote.output_mint.clone(),
                in_amount,
                out_amount,
                price_impact_bps: (quote.price_impact_pct * 10000.0) as i32,
                route_plan: serde_json::to_value(&quote.route_plan).unwrap_or_default(),
            },
        })
    }

    async fn get_jupiter_quote(&self, params: &SwapParams) -> AppResult<JupiterQuoteResponse> {
        let url = format!(
            "{}/quote?inputMint={}&outputMint={}&amount={}&slippageBps={}&onlyDirectRoutes=false",
            self.jupiter_api_url,
            params.input_mint,
            params.output_mint,
            params.amount_lamports,
            params.slippage_bps
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Jupiter quote request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ExternalApi(format!(
                "Jupiter quote error: {}",
                error_text
            )));
        }

        response
            .json::<JupiterQuoteResponse>()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse Jupiter quote: {}", e)))
    }

    async fn get_jupiter_swap_transaction(
        &self,
        quote: &JupiterQuoteResponse,
        user_public_key: &str,
    ) -> AppResult<JupiterSwapResponse> {
        let url = format!("{}/swap", self.jupiter_api_url);

        let request = JupiterSwapRequest {
            user_public_key: user_public_key.to_string(),
            quote_response: quote.clone(),
            wrap_and_unwrap_sol: true,
            use_shared_accounts: true,
            fee_account: None,
            tracking_account: None,
            compute_unit_price_micro_lamports: Some(100_000), // 0.1 lamports per CU
            priority_level_with_max_lamports: Some(PriorityLevel {
                max_lamports: Some(1_000_000), // Max 0.001 SOL priority fee
                priority_level: None,
            }),
            as_legacy_transaction: false,
            use_token_ledger: false,
            destination_token_account: None,
            dynamic_compute_unit_limit: true,
            skip_user_accounts_rpc_calls: false,
            dynamic_slippage: None,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Jupiter swap request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ExternalApi(format!(
                "Jupiter swap error: {}",
                error_text
            )));
        }

        response
            .json::<JupiterSwapResponse>()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse Jupiter swap: {}", e)))
    }

    fn extract_swap_mints(&self, edge: &Edge) -> AppResult<(String, String)> {
        // Try to extract from edge metadata
        if let Some(ref signal_data) = edge.signal_data {
            if let (Some(input), Some(output)) = (
                signal_data.get("input_mint").and_then(|v| v.as_str()),
                signal_data.get("output_mint").and_then(|v| v.as_str()),
            ) {
                return Ok((input.to_string(), output.to_string()));
            }
        }

        // Try common patterns from edge type
        match edge.edge_type.as_str() {
            "arbitrage" | "dex_swap" => {
                // For arbitrage, typically SOL -> token -> SOL
                if let Some(ref token_mint) = edge.token_mint {
                    // SOL mint (native)
                    let sol_mint = "So11111111111111111111111111111111111111112";
                    return Ok((sol_mint.to_string(), token_mint.clone()));
                }
            }
            _ => {}
        }

        Err(AppError::Execution(
            "Cannot extract swap mints from edge - missing signal_data or token_mint".to_string(),
        ))
    }

    fn extract_swap_amount(&self, edge: &Edge) -> AppResult<u64> {
        // Try signal_data first
        if let Some(ref signal_data) = edge.signal_data {
            if let Some(amount) = signal_data.get("amount_lamports").and_then(|v| v.as_u64()) {
                return Ok(amount);
            }
            if let Some(amount) = signal_data.get("in_amount").and_then(|v| v.as_u64()) {
                return Ok(amount);
            }
        }

        // Fall back to estimated profit as a proxy (with 10x leverage for MEV)
        if let Some(profit) = edge.estimated_profit_lamports {
            return Ok((profit.abs() as u64) * 10);
        }

        Err(AppError::Execution(
            "Cannot extract swap amount from edge".to_string(),
        ))
    }

    pub async fn invalidate_blockhash(&self) {
        self.blockhash_cache.invalidate().await;
    }

    pub async fn build_exit_swap(
        &self,
        position: &OpenPosition,
        exit_signal: &ExitSignal,
        user_public_key: &str,
        slippage_bps: u16,
    ) -> AppResult<ExitBuildResult> {
        let token_balance = self.get_token_balance(user_public_key, &position.token_mint).await?;

        if token_balance == 0 {
            return Err(AppError::Execution(format!(
                "No token balance found for {} in wallet {}",
                position.token_mint, user_public_key
            )));
        }

        let exit_amount = if exit_signal.exit_percent >= 100.0 {
            token_balance
        } else {
            ((token_balance as f64) * (exit_signal.exit_percent / 100.0)) as u64
        };

        let base_mint = position.exit_config.base_currency.mint().to_string();

        info!(
            "🔄 Building exit swap: {} {} → {} | Balance: {} | Exit: {:.1}%",
            exit_amount,
            position.token_symbol.as_deref().unwrap_or(&position.token_mint[..8]),
            position.exit_config.base_currency.symbol(),
            token_balance,
            exit_signal.exit_percent
        );

        let params = SwapParams {
            input_mint: position.token_mint.clone(),
            output_mint: base_mint,
            amount_lamports: exit_amount,
            slippage_bps,
            user_public_key: user_public_key.to_string(),
        };

        let build_result = self.build_jupiter_swap(&params, position.edge_id).await?;

        Ok(ExitBuildResult {
            position_id: position.id,
            exit_signal: exit_signal.clone(),
            transaction_base64: build_result.transaction_base64,
            last_valid_block_height: build_result.last_valid_block_height,
            token_amount_in: exit_amount,
            expected_base_out: build_result.route_info.out_amount,
            price_impact_bps: build_result.route_info.price_impact_bps,
            route_info: build_result.route_info,
        })
    }

    pub async fn get_token_balance(&self, wallet: &str, token_mint: &str) -> AppResult<u64> {
        if token_mint == SOL_MINT {
            return self.get_sol_balance(wallet).await;
        }

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getTokenAccountsByOwner",
            "params": [
                wallet,
                { "mint": token_mint },
                { "encoding": "jsonParsed" }
            ]
        });

        let response = self
            .client
            .post(&self.rpc_url)
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("RPC request failed: {}", e)))?;

        let rpc_response: RpcResponse<TokenAccountsResult> = response
            .json()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse RPC response: {}", e)))?;

        if let Some(result) = rpc_response.result {
            for account in result.value {
                if let Some(parsed) = account.account.data.get("parsed") {
                    if let Some(info) = parsed.get("info") {
                        if let Some(token_amount) = info.get("tokenAmount") {
                            if let Some(amount_str) = token_amount.get("amount").and_then(|v| v.as_str()) {
                                return amount_str.parse().map_err(|e| {
                                    AppError::Internal(format!("Failed to parse token amount: {}", e))
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(0)
    }

    async fn get_sol_balance(&self, wallet: &str) -> AppResult<u64> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getBalance",
            "params": [wallet]
        });

        let response = self
            .client
            .post(&self.rpc_url)
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("RPC request failed: {}", e)))?;

        let rpc_response: RpcResponse<BalanceResult> = response
            .json()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse RPC response: {}", e)))?;

        Ok(rpc_response.result.map(|r| r.value).unwrap_or(0))
    }

    pub async fn get_token_price(&self, token_mint: &str, base: BaseCurrency) -> AppResult<f64> {
        let url = format!(
            "{}/price?ids={}&vsToken={}",
            self.jupiter_api_url,
            token_mint,
            base.mint()
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Jupiter price request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::ExternalApi(format!(
                "Jupiter price error: {}",
                response.status()
            )));
        }

        let price_data: JupiterPriceResponse = response
            .json()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse Jupiter price: {}", e)))?;

        if let Some(token_price) = price_data.data.get(token_mint) {
            Ok(token_price.price)
        } else {
            Err(AppError::ExternalApi(format!(
                "No price found for token {}",
                token_mint
            )))
        }
    }

    pub async fn get_multiple_token_prices(
        &self,
        token_mints: &[String],
        base: BaseCurrency,
    ) -> AppResult<std::collections::HashMap<String, f64>> {
        if token_mints.is_empty() {
            return Ok(std::collections::HashMap::new());
        }

        let ids = token_mints.join(",");
        let url = format!(
            "{}/price?ids={}&vsToken={}",
            self.jupiter_api_url,
            ids,
            base.mint()
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Jupiter price request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::ExternalApi(format!(
                "Jupiter price error: {}",
                response.status()
            )));
        }

        let price_data: JupiterPriceResponse = response
            .json()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse Jupiter price: {}", e)))?;

        Ok(price_data
            .data
            .into_iter()
            .map(|(k, v)| (k, v.price))
            .collect())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JupiterQuoteResponse {
    pub input_mint: String,
    pub output_mint: String,
    pub in_amount: String,
    pub out_amount: String,
    pub other_amount_threshold: String,
    pub swap_mode: String,
    pub slippage_bps: u16,
    pub price_impact_pct: f64,
    pub route_plan: Vec<JupiterRoutePlan>,
    #[serde(default)]
    pub context_slot: Option<u64>,
    #[serde(default)]
    pub time_taken: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JupiterRoutePlan {
    pub swap_info: JupiterSwapInfo,
    pub percent: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JupiterSwapInfo {
    pub amm_key: String,
    pub label: Option<String>,
    pub input_mint: String,
    pub output_mint: String,
    pub in_amount: String,
    pub out_amount: String,
    pub fee_amount: String,
    pub fee_mint: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct JupiterSwapRequest {
    user_public_key: String,
    quote_response: JupiterQuoteResponse,
    wrap_and_unwrap_sol: bool,
    use_shared_accounts: bool,
    fee_account: Option<String>,
    tracking_account: Option<String>,
    compute_unit_price_micro_lamports: Option<u64>,
    priority_level_with_max_lamports: Option<PriorityLevel>,
    as_legacy_transaction: bool,
    use_token_ledger: bool,
    destination_token_account: Option<String>,
    dynamic_compute_unit_limit: bool,
    skip_user_accounts_rpc_calls: bool,
    dynamic_slippage: Option<DynamicSlippage>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PriorityLevel {
    max_lamports: Option<u64>,
    priority_level: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DynamicSlippage {
    min_bps: Option<u16>,
    max_bps: Option<u16>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JupiterSwapResponse {
    pub swap_transaction: String,
    #[serde(default)]
    pub last_valid_block_height: Option<u64>,
    #[serde(default)]
    pub priority_fee_lamports: Option<u64>,
    #[serde(default)]
    pub compute_unit_limit: Option<u64>,
    #[serde(default)]
    pub prioritization_type: Option<String>,
    #[serde(default)]
    pub simulation_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitBuildResult {
    pub position_id: Uuid,
    pub exit_signal: ExitSignal,
    pub transaction_base64: String,
    pub last_valid_block_height: u64,
    pub token_amount_in: u64,
    pub expected_base_out: u64,
    pub price_impact_bps: i32,
    pub route_info: RouteInfo,
}

#[derive(Debug, Clone, Deserialize)]
struct RpcResponse<T> {
    #[allow(dead_code)]
    jsonrpc: String,
    result: Option<T>,
    #[allow(dead_code)]
    error: Option<RpcError>,
}

#[derive(Debug, Clone, Deserialize)]
struct RpcError {
    #[allow(dead_code)]
    code: i64,
    #[allow(dead_code)]
    message: String,
}

#[derive(Debug, Clone, Deserialize)]
struct TokenAccountsResult {
    value: Vec<TokenAccountInfo>,
}

#[derive(Debug, Clone, Deserialize)]
struct TokenAccountInfo {
    account: TokenAccountData,
}

#[derive(Debug, Clone, Deserialize)]
struct TokenAccountData {
    data: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
struct BalanceResult {
    value: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct JupiterPriceResponse {
    data: std::collections::HashMap<String, JupiterTokenPrice>,
}

#[derive(Debug, Clone, Deserialize)]
struct JupiterTokenPrice {
    #[allow(dead_code)]
    id: String,
    price: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swap_params_serialization() {
        let params = SwapParams {
            input_mint: "So11111111111111111111111111111111111111112".to_string(),
            output_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            amount_lamports: 1_000_000_000,
            slippage_bps: 50,
            user_public_key: "test_pubkey".to_string(),
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("So11111111111111111111111111111111111111112"));
    }

    #[test]
    fn test_build_result_serialization() {
        let result = BuildResult {
            edge_id: Uuid::new_v4(),
            transaction_base64: "test_tx".to_string(),
            last_valid_block_height: 12345,
            priority_fee_lamports: 5000,
            estimated_compute_units: 200000,
            route_info: RouteInfo {
                input_mint: "sol".to_string(),
                output_mint: "usdc".to_string(),
                in_amount: 1000000000,
                out_amount: 100000000,
                price_impact_bps: 10,
                route_plan: serde_json::json!([]),
            },
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("test_tx"));
    }
}
