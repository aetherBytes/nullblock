use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::{Signal, VenueType};

#[async_trait]
pub trait MevVenue: Send + Sync {
    fn venue_id(&self) -> Uuid;
    fn venue_type(&self) -> VenueType;
    fn name(&self) -> &str;

    async fn scan_for_signals(&self) -> AppResult<Vec<Signal>>;

    async fn estimate_profit(&self, signal: &Signal) -> AppResult<ProfitEstimate>;

    async fn get_quote(&self, params: &QuoteParams) -> AppResult<Quote>;

    async fn is_healthy(&self) -> bool;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfitEstimate {
    pub signal_id: Uuid,
    pub estimated_profit_lamports: i64,
    pub estimated_gas_lamports: i64,
    pub net_profit_lamports: i64,
    pub profit_bps: i32,
    pub confidence: f64,
    pub route: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteParams {
    pub input_mint: String,
    pub output_mint: String,
    pub amount_lamports: u64,
    pub slippage_bps: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub input_mint: String,
    pub output_mint: String,
    pub input_amount: u64,
    pub output_amount: u64,
    pub price_impact_bps: i32,
    pub route_plan: serde_json::Value,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPrice {
    pub mint: String,
    pub price_usd: f64,
    pub price_sol: f64,
    pub volume_24h: f64,
    pub liquidity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolInfo {
    pub pool_address: String,
    pub token_a_mint: String,
    pub token_b_mint: String,
    pub token_a_reserve: u64,
    pub token_b_reserve: u64,
    pub fee_bps: u16,
    pub liquidity_usd: f64,
}
