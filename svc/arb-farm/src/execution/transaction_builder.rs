use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::Edge;
use super::blockhash::BlockhashCache;

pub struct TransactionBuilder {
    client: reqwest::Client,
    jupiter_api_url: String,
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
