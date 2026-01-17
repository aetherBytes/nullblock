use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::{Signal, SignalType, VenueType};
use crate::events::Significance;
use crate::venues::{MevVenue, ProfitEstimate, Quote, QuoteParams};

pub struct JupiterVenue {
    id: Uuid,
    client: Client,
    base_url: String,
}

impl JupiterVenue {
    pub fn new(base_url: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            client: Client::new(),
            base_url,
        }
    }

    pub async fn get_quote_internal(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        slippage_bps: u16,
    ) -> AppResult<JupiterQuoteResponse> {
        let url = format!(
            "{}/quote?inputMint={}&outputMint={}&amount={}&slippageBps={}",
            self.base_url, input_mint, output_mint, amount, slippage_bps
        );

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Jupiter quote request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::ExternalApi(format!(
                "Jupiter returned error status: {}",
                response.status()
            )));
        }

        response
            .json::<JupiterQuoteResponse>()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse Jupiter response: {}", e)))
    }

    pub async fn get_token_price(&self, mint: &str) -> AppResult<f64> {
        let url = format!("{}/price?ids={}", self.base_url, mint);

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Jupiter price request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::ExternalApi(format!(
                "Jupiter price returned error: {}",
                response.status()
            )));
        }

        let price_response: JupiterPriceResponse = response
            .json()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse price response: {}", e)))?;

        price_response
            .data
            .get(mint)
            .map(|p| p.price)
            .ok_or_else(|| AppError::NotFound(format!("Price not found for {}", mint)))
    }
}

#[async_trait]
impl MevVenue for JupiterVenue {
    fn venue_id(&self) -> Uuid {
        self.id
    }

    fn venue_type(&self) -> VenueType {
        VenueType::DexAmm
    }

    fn name(&self) -> &str {
        "Jupiter"
    }

    async fn scan_for_signals(&self) -> AppResult<Vec<Signal>> {
        Ok(vec![])
    }

    async fn estimate_profit(&self, signal: &Signal) -> AppResult<ProfitEstimate> {
        Ok(ProfitEstimate {
            signal_id: signal.id,
            estimated_profit_lamports: 0,
            estimated_gas_lamports: 5000,
            net_profit_lamports: 0,
            profit_bps: 0,
            confidence: 0.0,
            route: None,
        })
    }

    async fn get_quote(&self, params: &QuoteParams) -> AppResult<Quote> {
        let jupiter_quote = self
            .get_quote_internal(
                &params.input_mint,
                &params.output_mint,
                params.amount_lamports,
                params.slippage_bps,
            )
            .await?;

        Ok(Quote {
            input_mint: params.input_mint.clone(),
            output_mint: params.output_mint.clone(),
            input_amount: jupiter_quote.in_amount.parse().unwrap_or(0),
            output_amount: jupiter_quote.out_amount.parse().unwrap_or(0),
            price_impact_bps: (jupiter_quote.price_impact_pct * 10000.0) as i32,
            route_plan: serde_json::to_value(&jupiter_quote.route_plan).unwrap_or_default(),
            expires_at: chrono::Utc::now() + chrono::Duration::seconds(30),
        })
    }

    async fn is_healthy(&self) -> bool {
        // Test with a minimal SOL->USDC quote to verify the API is working
        let url = format!(
            "{}/quote?inputMint=So11111111111111111111111111111111111111112&outputMint=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v&amount=1000000",
            self.base_url
        );
        self.client
            .get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }
}

#[derive(Debug, Deserialize)]
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
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JupiterRoutePlan {
    pub swap_info: JupiterSwapInfo,
    pub percent: u8,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct JupiterPriceResponse {
    pub data: std::collections::HashMap<String, JupiterPrice>,
}

#[derive(Debug, Deserialize)]
pub struct JupiterPrice {
    pub id: String,
    pub price: f64,
}
