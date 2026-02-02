use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::VenueType;
use crate::events::Significance;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub id: Uuid,
    pub signal_type: SignalType,
    pub venue_id: Uuid,
    pub venue_type: VenueType,
    pub token_mint: Option<String>,
    pub pool_address: Option<String>,
    pub estimated_profit_bps: i32,
    pub confidence: f64,
    pub significance: Significance,
    pub metadata: serde_json::Value,
    pub detected_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl Signal {
    pub fn new(
        signal_type: SignalType,
        venue_id: Uuid,
        venue_type: VenueType,
        significance: Significance,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            signal_type,
            venue_id,
            venue_type,
            token_mint: None,
            pool_address: None,
            estimated_profit_bps: 0,
            confidence: 0.0,
            significance,
            metadata: serde_json::json!({}),
            detected_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::minutes(5),
        }
    }

    pub fn with_token(mut self, mint: String) -> Self {
        self.token_mint = Some(mint);
        self
    }

    pub fn with_pool(mut self, address: String) -> Self {
        self.pool_address = Some(address);
        self
    }

    pub fn with_profit(mut self, profit_bps: i32, confidence: f64) -> Self {
        self.estimated_profit_bps = profit_bps;
        self.confidence = confidence;
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn with_expiry(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = expires_at;
        self
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SignalType {
    PriceDiscrepancy,
    VolumeSpike,
    LiquidityChange,
    NewToken,
    CurveGraduation,
    LargeOrder,
    Liquidation,
    PoolImbalance,
    DexArb,
    JitLiquidity,
    Backrun,
    KolTrade,
}

impl std::fmt::Display for SignalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignalType::PriceDiscrepancy => write!(f, "price_discrepancy"),
            SignalType::VolumeSpike => write!(f, "volume_spike"),
            SignalType::LiquidityChange => write!(f, "liquidity_change"),
            SignalType::NewToken => write!(f, "new_token"),
            SignalType::CurveGraduation => write!(f, "curve_graduation"),
            SignalType::LargeOrder => write!(f, "large_order"),
            SignalType::Liquidation => write!(f, "liquidation"),
            SignalType::PoolImbalance => write!(f, "pool_imbalance"),
            SignalType::DexArb => write!(f, "dex_arb"),
            SignalType::JitLiquidity => write!(f, "jit_liquidity"),
            SignalType::Backrun => write!(f, "backrun"),
            SignalType::KolTrade => write!(f, "kol_trade"),
        }
    }
}
