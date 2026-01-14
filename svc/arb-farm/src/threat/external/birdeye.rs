use serde::{Deserialize, Serialize};
use crate::error::{AppError, AppResult};

pub struct BirdeyeClient {
    base_url: String,
    api_key: Option<String>,
    client: reqwest::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BirdeyeHoldersResponse {
    pub success: bool,
    pub data: Option<BirdeyeHoldersData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BirdeyeHoldersData {
    pub items: Vec<BirdeyeHolder>,
    pub total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BirdeyeHolder {
    pub address: String,
    pub amount: f64,
    pub decimals: u8,
    pub owner: String,
    pub ui_amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BirdeyeTokenResponse {
    pub success: bool,
    pub data: Option<BirdeyeTokenData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BirdeyeTokenData {
    pub address: String,
    pub symbol: Option<String>,
    pub name: Option<String>,
    pub decimals: u8,
    pub price: Option<f64>,
    pub volume_24h: Option<f64>,
    pub volume_24h_change_percent: Option<f64>,
    pub market_cap: Option<f64>,
    pub liquidity: Option<f64>,
    pub supply: Option<f64>,
    pub holder: Option<u64>,
    pub creation_time: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BirdeyeTradesResponse {
    pub success: bool,
    pub data: Option<BirdeyeTradesData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BirdeyeTradesData {
    pub items: Vec<BirdeyeTrade>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BirdeyeTrade {
    pub tx_hash: String,
    pub block_unix_time: i64,
    pub source: String,
    pub trade_type: String,
    pub token_address: String,
    pub from_address: String,
    pub to_address: String,
    pub token_amount: f64,
    pub native_amount: f64,
    pub price: f64,
}

impl BirdeyeClient {
    pub fn new(base_url: String, api_key: Option<String>) -> Self {
        Self {
            base_url,
            api_key,
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_holders(&self, mint: &str, limit: u32) -> AppResult<Vec<BirdeyeHolder>> {
        let url = format!(
            "{}/defi/token_holders?address={}&limit={}",
            self.base_url, mint, limit
        );

        let mut request = self.client
            .get(&url)
            .header("Accept", "application/json");

        if let Some(key) = &self.api_key {
            request = request.header("X-API-KEY", key);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(AppError::ExternalApi(format!(
                "Birdeye API error: {}",
                response.status()
            )));
        }

        let data: BirdeyeHoldersResponse = response.json().await.map_err(|e| {
            AppError::ExternalApi(format!("Failed to parse Birdeye response: {}", e))
        })?;

        Ok(data.data.map(|d| d.items).unwrap_or_default())
    }

    pub async fn get_token_info(&self, mint: &str) -> AppResult<BirdeyeTokenData> {
        let url = format!("{}/defi/token_overview?address={}", self.base_url, mint);

        let mut request = self.client
            .get(&url)
            .header("Accept", "application/json");

        if let Some(key) = &self.api_key {
            request = request.header("X-API-KEY", key);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(AppError::ExternalApi(format!(
                "Birdeye API error: {}",
                response.status()
            )));
        }

        let data: BirdeyeTokenResponse = response.json().await.map_err(|e| {
            AppError::ExternalApi(format!("Failed to parse Birdeye response: {}", e))
        })?;

        data.data.ok_or_else(|| {
            AppError::ExternalApi("Token not found in Birdeye".to_string())
        })
    }

    pub async fn get_recent_trades(&self, mint: &str, limit: u32) -> AppResult<Vec<BirdeyeTrade>> {
        let url = format!(
            "{}/defi/txs/token?address={}&limit={}",
            self.base_url, mint, limit
        );

        let mut request = self.client
            .get(&url)
            .header("Accept", "application/json");

        if let Some(key) = &self.api_key {
            request = request.header("X-API-KEY", key);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(AppError::ExternalApi(format!(
                "Birdeye API error: {}",
                response.status()
            )));
        }

        let data: BirdeyeTradesResponse = response.json().await.map_err(|e| {
            AppError::ExternalApi(format!("Failed to parse Birdeye response: {}", e))
        })?;

        Ok(data.data.map(|d| d.items).unwrap_or_default())
    }

    pub fn analyze_holders(&self, holders: &[BirdeyeHolder], total_supply: f64) -> HolderAnalysis {
        if holders.is_empty() || total_supply == 0.0 {
            return HolderAnalysis::default();
        }

        let mut top_10_amount = 0.0;
        let mut top_20_amount = 0.0;

        for (i, holder) in holders.iter().enumerate() {
            if i < 10 {
                top_10_amount += holder.ui_amount;
            }
            if i < 20 {
                top_20_amount += holder.ui_amount;
            }
        }

        let top_10_concentration = (top_10_amount / total_supply) * 100.0;
        let top_20_concentration = (top_20_amount / total_supply) * 100.0;

        let top_holder = holders.first().map(|h| {
            (h.owner.clone(), (h.ui_amount / total_supply) * 100.0)
        });

        let unique_holders = holders.iter()
            .map(|h| &h.owner)
            .collect::<std::collections::HashSet<_>>()
            .len();

        let concentration_score = if top_10_concentration > 80.0 {
            1.0
        } else if top_10_concentration > 60.0 {
            0.7
        } else if top_10_concentration > 40.0 {
            0.4
        } else {
            0.2
        };

        HolderAnalysis {
            total_holders: holders.len() as u64,
            unique_holders: unique_holders as u64,
            top_10_concentration,
            top_20_concentration,
            top_holder_address: top_holder.as_ref().map(|(a, _)| a.clone()),
            top_holder_percent: top_holder.map(|(_, p)| p).unwrap_or(0.0),
            concentration_risk_score: concentration_score,
        }
    }

    pub fn detect_wash_trading(&self, trades: &[BirdeyeTrade]) -> WashTradingAnalysis {
        if trades.is_empty() {
            return WashTradingAnalysis::default();
        }

        let mut address_counts: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
        let mut round_trip_count = 0;

        for trade in trades {
            *address_counts.entry(trade.from_address.clone()).or_insert(0) += 1;
            *address_counts.entry(trade.to_address.clone()).or_insert(0) += 1;
        }

        for (i, trade) in trades.iter().enumerate() {
            for other in trades.iter().skip(i + 1).take(20) {
                if trade.from_address == other.to_address && trade.to_address == other.from_address {
                    round_trip_count += 1;
                }
            }
        }

        let repeated_addresses = address_counts.values()
            .filter(|&&count| count > 3)
            .count();

        let total_addresses = address_counts.len();
        let repeat_ratio = if total_addresses > 0 {
            repeated_addresses as f64 / total_addresses as f64
        } else {
            0.0
        };

        let wash_score = (repeat_ratio * 0.5) + (round_trip_count.min(10) as f64 / 10.0 * 0.5);

        WashTradingAnalysis {
            total_trades_analyzed: trades.len() as u64,
            unique_addresses: total_addresses as u64,
            repeated_addresses: repeated_addresses as u64,
            round_trip_patterns: round_trip_count,
            wash_trading_likelihood: wash_score.min(1.0),
            is_suspicious: wash_score > 0.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HolderAnalysis {
    pub total_holders: u64,
    pub unique_holders: u64,
    pub top_10_concentration: f64,
    pub top_20_concentration: f64,
    pub top_holder_address: Option<String>,
    pub top_holder_percent: f64,
    pub concentration_risk_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WashTradingAnalysis {
    pub total_trades_analyzed: u64,
    pub unique_addresses: u64,
    pub repeated_addresses: u64,
    pub round_trip_patterns: u32,
    pub wash_trading_likelihood: f64,
    pub is_suspicious: bool,
}

impl Default for BirdeyeClient {
    fn default() -> Self {
        Self::new("https://public-api.birdeye.so".to_string(), None)
    }
}
