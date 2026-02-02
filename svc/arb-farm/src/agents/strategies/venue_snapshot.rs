use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{Signal, VenueType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VenueSnapshot {
    pub venue_id: Uuid,
    pub venue_type: VenueType,
    pub venue_name: String,
    pub tokens: Vec<TokenData>,
    pub raw_signals: Vec<Signal>,
    pub timestamp: DateTime<Utc>,
    pub is_healthy: bool,
}

impl VenueSnapshot {
    pub fn new(venue_id: Uuid, venue_type: VenueType, venue_name: String) -> Self {
        Self {
            venue_id,
            venue_type,
            venue_name,
            tokens: Vec::new(),
            raw_signals: Vec::new(),
            timestamp: Utc::now(),
            is_healthy: true,
        }
    }

    pub fn with_tokens(mut self, tokens: Vec<TokenData>) -> Self {
        self.tokens = tokens;
        self
    }

    pub fn with_signals(mut self, signals: Vec<Signal>) -> Self {
        self.raw_signals = signals;
        self
    }

    pub fn token_count(&self) -> usize {
        self.tokens.len()
    }

    pub fn signal_count(&self) -> usize {
        self.raw_signals.len()
    }

    pub fn filter_tokens_by_progress(&self, min: f64, max: f64) -> Vec<&TokenData> {
        self.tokens
            .iter()
            .filter(|t| t.graduation_progress >= min && t.graduation_progress <= max)
            .collect()
    }

    pub fn filter_tokens_by_volume(&self, min_volume_sol: f64) -> Vec<&TokenData> {
        self.tokens
            .iter()
            .filter(|t| t.volume_24h_sol >= min_volume_sol)
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenData {
    pub mint: String,
    pub name: String,
    pub symbol: String,
    pub graduation_progress: f64,
    pub bonding_curve_address: Option<String>,
    pub market_cap_sol: f64,
    pub volume_24h_sol: f64,
    pub volume_1h_sol: f64,
    pub holder_count: u32,
    pub created_at: DateTime<Utc>,
    pub last_trade_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
}

impl TokenData {
    pub fn new(mint: String, name: String, symbol: String) -> Self {
        Self {
            mint,
            name,
            symbol,
            graduation_progress: 0.0,
            bonding_curve_address: None,
            market_cap_sol: 0.0,
            volume_24h_sol: 0.0,
            volume_1h_sol: 0.0,
            holder_count: 0,
            created_at: Utc::now(),
            last_trade_at: None,
            metadata: serde_json::json!({}),
        }
    }

    pub fn with_progress(mut self, progress: f64) -> Self {
        self.graduation_progress = progress;
        self
    }

    pub fn with_bonding_curve(mut self, address: String) -> Self {
        self.bonding_curve_address = Some(address);
        self
    }

    pub fn with_market_cap(mut self, market_cap_sol: f64) -> Self {
        self.market_cap_sol = market_cap_sol;
        self
    }

    pub fn with_volume(mut self, volume_24h_sol: f64) -> Self {
        self.volume_24h_sol = volume_24h_sol;
        self
    }

    pub fn with_holders(mut self, holder_count: u32) -> Self {
        self.holder_count = holder_count;
        self
    }

    pub fn is_graduation_candidate(&self) -> bool {
        self.graduation_progress >= 30.0 && self.graduation_progress <= 85.0
    }

    pub fn is_near_graduation(&self) -> bool {
        self.graduation_progress >= 85.0
    }

    pub fn is_high_velocity(&self, min_velocity: f64) -> bool {
        self.volume_24h_sol > min_velocity * self.market_cap_sol.max(0.01)
    }
}
