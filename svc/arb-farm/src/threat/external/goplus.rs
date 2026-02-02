use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct GoPlusClient {
    base_url: String,
    client: reqwest::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoPlusResponse {
    pub code: i32,
    pub message: String,
    pub result: HashMap<String, GoPlusTokenInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoPlusTokenInfo {
    pub is_honeypot: Option<String>,
    pub is_open_source: Option<String>,
    pub is_proxy: Option<String>,
    pub is_mintable: Option<String>,
    pub can_take_back_ownership: Option<String>,
    pub owner_change_balance: Option<String>,
    pub hidden_owner: Option<String>,
    pub selfdestruct: Option<String>,
    pub external_call: Option<String>,
    pub buy_tax: Option<String>,
    pub sell_tax: Option<String>,
    pub is_blacklisted: Option<String>,
    pub is_whitelisted: Option<String>,
    pub is_in_dex: Option<String>,
    pub transfer_pausable: Option<String>,
    pub cannot_buy: Option<String>,
    pub cannot_sell_all: Option<String>,
    pub slippage_modifiable: Option<String>,
    pub personal_slippage_modifiable: Option<String>,
    pub trading_cooldown: Option<String>,
    pub is_anti_whale: Option<String>,
    pub anti_whale_modifiable: Option<String>,
    pub holder_count: Option<String>,
    pub total_supply: Option<String>,
    pub creator_address: Option<String>,
    pub creator_balance: Option<String>,
    pub creator_percent: Option<String>,
    pub lp_holder_count: Option<String>,
    pub lp_total_supply: Option<String>,
    pub is_true_token: Option<String>,
    pub is_airdrop_scam: Option<String>,
    pub trust_list: Option<String>,
    pub other_potential_risks: Option<String>,
    pub note: Option<String>,
    pub honeypot_with_same_creator: Option<String>,
    pub fake_token: Option<String>,
}

impl GoPlusClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    pub async fn check_token(&self, mint: &str) -> AppResult<GoPlusTokenInfo> {
        let url = format!("{}/solana/token_security/{}", self.base_url, mint);

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AppError::ExternalApi(format!(
                "GoPlus API error: {}",
                response.status()
            )));
        }

        let data: GoPlusResponse = response.json().await.map_err(|e| {
            AppError::ExternalApi(format!("Failed to parse GoPlus response: {}", e))
        })?;

        if data.code != 1 {
            return Err(AppError::ExternalApi(format!(
                "GoPlus API returned error code: {}",
                data.code
            )));
        }

        data.result
            .get(mint)
            .cloned()
            .ok_or_else(|| AppError::ExternalApi("Token not found in GoPlus response".to_string()))
    }

    pub fn analyze(&self, info: &GoPlusTokenInfo) -> GoPlusAnalysis {
        let is_honeypot = info.is_honeypot.as_ref().map(|v| v == "1").unwrap_or(false);

        let is_proxy = info.is_proxy.as_ref().map(|v| v == "1").unwrap_or(false);

        let is_mintable = info.is_mintable.as_ref().map(|v| v == "1").unwrap_or(false);

        let hidden_owner = info
            .hidden_owner
            .as_ref()
            .map(|v| v == "1")
            .unwrap_or(false);

        let can_take_back_ownership = info
            .can_take_back_ownership
            .as_ref()
            .map(|v| v == "1")
            .unwrap_or(false);

        let is_blacklisted = info
            .is_blacklisted
            .as_ref()
            .map(|v| v == "1")
            .unwrap_or(false);

        let transfer_pausable = info
            .transfer_pausable
            .as_ref()
            .map(|v| v == "1")
            .unwrap_or(false);

        let cannot_buy = info.cannot_buy.as_ref().map(|v| v == "1").unwrap_or(false);

        let cannot_sell_all = info
            .cannot_sell_all
            .as_ref()
            .map(|v| v == "1")
            .unwrap_or(false);

        let is_airdrop_scam = info
            .is_airdrop_scam
            .as_ref()
            .map(|v| v == "1")
            .unwrap_or(false);

        let fake_token = info.fake_token.as_ref().map(|v| v == "1").unwrap_or(false);

        let buy_tax = info
            .buy_tax
            .as_ref()
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0);

        let sell_tax = info
            .sell_tax
            .as_ref()
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0);

        let creator_percent = info
            .creator_percent
            .as_ref()
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0);

        let holder_count = info
            .holder_count
            .as_ref()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        let mut risk_score: f64 = 0.0;

        if is_honeypot {
            risk_score += 0.4;
        }
        if is_mintable {
            risk_score += 0.15;
        }
        if hidden_owner {
            risk_score += 0.1;
        }
        if can_take_back_ownership {
            risk_score += 0.1;
        }
        if transfer_pausable {
            risk_score += 0.1;
        }
        if cannot_sell_all {
            risk_score += 0.2;
        }
        if is_airdrop_scam {
            risk_score += 0.3;
        }
        if fake_token {
            risk_score += 0.3;
        }
        if buy_tax > 10.0 {
            risk_score += 0.1;
        }
        if sell_tax > 10.0 {
            risk_score += 0.15;
        }
        if creator_percent > 20.0 {
            risk_score += 0.1;
        }

        GoPlusAnalysis {
            is_honeypot,
            is_proxy,
            is_mintable,
            hidden_owner,
            can_take_back_ownership,
            is_blacklisted,
            transfer_pausable,
            cannot_buy,
            cannot_sell_all,
            is_airdrop_scam,
            fake_token,
            buy_tax,
            sell_tax,
            creator_percent,
            holder_count,
            risk_score: risk_score.min(1.0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoPlusAnalysis {
    pub is_honeypot: bool,
    pub is_proxy: bool,
    pub is_mintable: bool,
    pub hidden_owner: bool,
    pub can_take_back_ownership: bool,
    pub is_blacklisted: bool,
    pub transfer_pausable: bool,
    pub cannot_buy: bool,
    pub cannot_sell_all: bool,
    pub is_airdrop_scam: bool,
    pub fake_token: bool,
    pub buy_tax: f64,
    pub sell_tax: f64,
    pub creator_percent: f64,
    pub holder_count: u64,
    pub risk_score: f64,
}

impl Default for GoPlusClient {
    fn default() -> Self {
        Self::new("https://api.gopluslabs.io/api/v1".to_string())
    }
}
