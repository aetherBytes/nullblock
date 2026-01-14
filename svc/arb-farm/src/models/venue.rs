use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Venue {
    pub id: Uuid,
    pub venue_type: VenueType,
    pub name: String,
    pub address: Option<String>,
    pub config: serde_json::Value,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum VenueType {
    DexAmm,
    BondingCurve,
    Lending,
    Orderbook,
}

impl std::fmt::Display for VenueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VenueType::DexAmm => write!(f, "dex_amm"),
            VenueType::BondingCurve => write!(f, "bonding_curve"),
            VenueType::Lending => write!(f, "lending"),
            VenueType::Orderbook => write!(f, "orderbook"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DexProvider {
    Jupiter,
    Raydium,
    Orca,
    Phoenix,
    OpenBook,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CurveProvider {
    PumpFun,
    Moonshot,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LendingProvider {
    Marginfi,
    Kamino,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVenueRequest {
    pub venue_type: VenueType,
    pub name: String,
    pub address: Option<String>,
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DexAmmConfig {
    pub provider: DexProvider,
    pub pool_address: Option<String>,
    pub token_a_mint: Option<String>,
    pub token_b_mint: Option<String>,
    pub min_liquidity: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BondingCurveConfig {
    pub provider: CurveProvider,
    pub graduation_threshold: f64,
    pub min_progress_to_track: f64,
    pub track_new_tokens: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LendingConfig {
    pub provider: LendingProvider,
    pub min_health_factor: f64,
    pub min_liquidation_profit: i64,
}
