use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};

pub struct RugCheckClient {
    base_url: String,
    client: reqwest::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RugCheckResponse {
    pub mint: String,
    pub score: f64,
    pub risks: Vec<RugCheckRisk>,
    pub token_meta: Option<TokenMeta>,
    pub creator: Option<String>,
    pub top_holders: Vec<TopHolder>,
    pub markets: Vec<Market>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RugCheckRisk {
    pub name: String,
    pub description: String,
    pub level: String,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMeta {
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub uri: Option<String>,
    pub mutable: bool,
    pub update_authority: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopHolder {
    pub address: String,
    pub amount: u64,
    pub percentage: f64,
    pub insider: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    pub pubkey: String,
    pub lp_type: String,
    pub liquidity_locked: bool,
    pub lp_burned_percent: f64,
}

impl RugCheckClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    pub async fn check_token(&self, mint: &str) -> AppResult<RugCheckResponse> {
        let url = format!("{}/tokens/{}/report", self.base_url, mint);

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AppError::ExternalApi(format!(
                "RugCheck API error: {}",
                response.status()
            )));
        }

        let data: RugCheckResponse = response.json().await.map_err(|e| {
            AppError::ExternalApi(format!("Failed to parse RugCheck response: {}", e))
        })?;

        Ok(data)
    }

    pub fn analyze_risks(&self, response: &RugCheckResponse) -> RugCheckAnalysis {
        let mut has_mint_authority = false;
        let mut has_freeze_authority = false;
        let mut is_mutable = false;
        let mut lp_locked = false;
        let mut high_risk_count = 0;

        for risk in &response.risks {
            match risk.name.to_lowercase().as_str() {
                name if name.contains("mint") => has_mint_authority = true,
                name if name.contains("freeze") => has_freeze_authority = true,
                _ => {}
            }
            if risk.level.to_lowercase() == "high" || risk.level.to_lowercase() == "critical" {
                high_risk_count += 1;
            }
        }

        if let Some(meta) = &response.token_meta {
            is_mutable = meta.mutable;
        }

        for market in &response.markets {
            if market.liquidity_locked || market.lp_burned_percent > 90.0 {
                lp_locked = true;
                break;
            }
        }

        let top_10_concentration: f64 = response
            .top_holders
            .iter()
            .take(10)
            .map(|h| h.percentage)
            .sum();

        let insider_concentration: f64 = response
            .top_holders
            .iter()
            .filter(|h| h.insider)
            .map(|h| h.percentage)
            .sum();

        RugCheckAnalysis {
            score: response.score,
            has_mint_authority,
            has_freeze_authority,
            is_mutable,
            lp_locked,
            high_risk_count,
            top_10_concentration,
            insider_concentration,
            creator: response.creator.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RugCheckAnalysis {
    pub score: f64,
    pub has_mint_authority: bool,
    pub has_freeze_authority: bool,
    pub is_mutable: bool,
    pub lp_locked: bool,
    pub high_risk_count: u32,
    pub top_10_concentration: f64,
    pub insider_concentration: f64,
    pub creator: Option<String>,
}

impl Default for RugCheckClient {
    fn default() -> Self {
        Self::new("https://api.rugcheck.xyz/v1".to_string())
    }
}
