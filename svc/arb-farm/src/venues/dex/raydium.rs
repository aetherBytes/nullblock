use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::{Signal, VenueType};
use crate::venues::{MevVenue, ProfitEstimate, Quote, QuoteParams, PoolInfo};

pub struct RaydiumVenue {
    id: Uuid,
    client: Client,
    base_url: String,
}

impl RaydiumVenue {
    pub fn new(base_url: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            client: Client::new(),
            base_url,
        }
    }

    pub async fn get_pools(&self, mint: Option<&str>) -> AppResult<Vec<RaydiumPool>> {
        let mut url = format!("{}/pools/info/list", self.base_url);
        if let Some(m) = mint {
            url.push_str(&format!("?mint={}", m));
        }

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(15))
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Raydium pools request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::ExternalApi(format!(
                "Raydium returned error status: {}",
                response.status()
            )));
        }

        let pool_response: RaydiumPoolResponse = response
            .json()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse Raydium response: {}", e)))?;

        Ok(pool_response.data)
    }

    pub async fn get_pool_info(&self, pool_id: &str) -> AppResult<PoolInfo> {
        let pools = self.get_pools(None).await?;

        let pool = pools
            .into_iter()
            .find(|p| p.id == pool_id)
            .ok_or_else(|| AppError::NotFound(format!("Pool {} not found", pool_id)))?;

        Ok(PoolInfo {
            pool_address: pool.id,
            token_a_mint: pool.mint_a.address,
            token_b_mint: pool.mint_b.address,
            token_a_reserve: (pool.mint_a.vault_amount * 1e9) as u64,
            token_b_reserve: (pool.mint_b.vault_amount * 1e9) as u64,
            fee_bps: (pool.fee_rate * 10000.0) as u16,
            liquidity_usd: pool.tvl,
        })
    }

    pub async fn compute_swap(
        &self,
        pool_id: &str,
        input_mint: &str,
        amount: u64,
    ) -> AppResult<RaydiumSwapResult> {
        let url = format!(
            "{}/compute/swap?poolId={}&inputMint={}&amount={}",
            self.base_url, pool_id, input_mint, amount
        );

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Raydium swap compute failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::ExternalApi(format!(
                "Raydium swap compute returned error: {}",
                response.status()
            )));
        }

        response
            .json()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse swap result: {}", e)))
    }
}

#[async_trait]
impl MevVenue for RaydiumVenue {
    fn venue_id(&self) -> Uuid {
        self.id
    }

    fn venue_type(&self) -> VenueType {
        VenueType::DexAmm
    }

    fn name(&self) -> &str {
        "Raydium"
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
        Err(AppError::Internal("Raydium quote requires pool ID".to_string()))
    }

    async fn is_healthy(&self) -> bool {
        let url = format!("{}/main/health", self.base_url);
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
pub struct RaydiumPoolResponse {
    pub success: bool,
    pub data: Vec<RaydiumPool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RaydiumPool {
    pub id: String,
    #[serde(rename = "type")]
    pub pool_type: String,
    pub program_id: String,
    pub mint_a: RaydiumMint,
    pub mint_b: RaydiumMint,
    pub fee_rate: f64,
    pub open_time: String,
    pub tvl: f64,
    pub day: RaydiumStats,
    pub week: RaydiumStats,
    pub month: RaydiumStats,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RaydiumMint {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub vault_amount: f64,
}

#[derive(Debug, Deserialize)]
pub struct RaydiumStats {
    pub volume: f64,
    pub fee: f64,
    pub apr: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RaydiumSwapResult {
    pub amount_in: String,
    pub amount_out: String,
    pub min_amount_out: String,
    pub price_impact: f64,
    pub fee: String,
}
