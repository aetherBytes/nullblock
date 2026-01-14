use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::events::Significance;
use crate::models::{Signal, SignalType, VenueType};
use crate::venues::{MevVenue, ProfitEstimate, Quote, QuoteParams};

pub struct MarginfiVenue {
    id: Uuid,
    client: Client,
    base_url: String,
    rpc_url: String,
}

impl MarginfiVenue {
    pub fn new(base_url: String, rpc_url: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            client: Client::new(),
            base_url,
            rpc_url,
        }
    }

    pub async fn get_accounts_at_risk(&self) -> AppResult<Vec<MarginfiAccount>> {
        let url = format!("{}/accounts/at-risk", self.base_url);

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(15))
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Marginfi accounts request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::ExternalApi(format!(
                "Marginfi returned error: {}",
                response.status()
            )));
        }

        let accounts: MarginfiAccountsResponse = response
            .json()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse Marginfi response: {}", e)))?;

        Ok(accounts.accounts)
    }

    pub async fn get_account_health(&self, account_address: &str) -> AppResult<AccountHealth> {
        let url = format!("{}/accounts/{}/health", self.base_url, account_address);

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Marginfi health request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::ExternalApi(format!(
                "Marginfi health returned error: {}",
                response.status()
            )));
        }

        response
            .json()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse health response: {}", e)))
    }

    pub async fn get_liquidatable_accounts(&self, min_shortfall: f64) -> AppResult<Vec<LiquidatableAccount>> {
        let accounts = self.get_accounts_at_risk().await?;
        let mut liquidatable = Vec::new();

        for account in accounts {
            if account.health_factor < 1.0 {
                let shortfall = account.total_borrowed_usd - (account.total_collateral_usd * account.liquidation_threshold);

                if shortfall >= min_shortfall {
                    let liquidation_bonus = account.total_collateral_usd * 0.05;
                    let estimated_profit = (liquidation_bonus - 0.001) * 1e9;

                    liquidatable.push(LiquidatableAccount {
                        address: account.address.clone(),
                        health_factor: account.health_factor,
                        total_collateral_usd: account.total_collateral_usd,
                        total_borrowed_usd: account.total_borrowed_usd,
                        shortfall_usd: shortfall,
                        liquidation_bonus_usd: liquidation_bonus,
                        estimated_profit_lamports: estimated_profit as i64,
                        collateral_token: account.collateral_token.clone(),
                        debt_token: account.debt_token.clone(),
                    });
                }
            }
        }

        liquidatable.sort_by(|a, b| {
            b.estimated_profit_lamports.cmp(&a.estimated_profit_lamports)
        });

        Ok(liquidatable)
    }

    pub async fn get_pool_stats(&self) -> AppResult<Vec<MarginfiPool>> {
        let url = format!("{}/pools", self.base_url);

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Marginfi pools request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::ExternalApi(format!(
                "Marginfi pools returned error: {}",
                response.status()
            )));
        }

        let pools: MarginfiPoolsResponse = response
            .json()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse pools response: {}", e)))?;

        Ok(pools.pools)
    }

    fn account_to_signal(&self, account: &LiquidatableAccount) -> Signal {
        let profit_bps = if account.total_collateral_usd > 0.0 {
            ((account.liquidation_bonus_usd / account.total_collateral_usd) * 10000.0) as i32
        } else {
            0
        };

        let significance = if account.shortfall_usd > 10000.0 {
            Significance::Critical
        } else if account.shortfall_usd > 1000.0 {
            Significance::High
        } else {
            Significance::Medium
        };

        let confidence = if account.health_factor < 0.5 {
            0.95
        } else if account.health_factor < 0.8 {
            0.8
        } else {
            0.6
        };

        Signal {
            id: Uuid::new_v4(),
            signal_type: SignalType::Liquidation,
            venue_id: self.id,
            venue_type: VenueType::Lending,
            token_mint: account.collateral_token.clone(),
            pool_address: Some(account.address.clone()),
            estimated_profit_bps: profit_bps,
            confidence,
            significance,
            metadata: serde_json::json!({
                "venue": "marginfi",
                "account_address": account.address,
                "health_factor": account.health_factor,
                "total_collateral_usd": account.total_collateral_usd,
                "total_borrowed_usd": account.total_borrowed_usd,
                "shortfall_usd": account.shortfall_usd,
                "liquidation_bonus_usd": account.liquidation_bonus_usd,
                "estimated_profit_lamports": account.estimated_profit_lamports,
                "collateral_token": account.collateral_token,
                "debt_token": account.debt_token,
                "atomicity": "non_atomic",
            }),
            detected_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + chrono::Duration::minutes(1),
        }
    }
}

#[async_trait]
impl MevVenue for MarginfiVenue {
    fn venue_id(&self) -> Uuid {
        self.id
    }

    fn venue_type(&self) -> VenueType {
        VenueType::Lending
    }

    fn name(&self) -> &str {
        "Marginfi"
    }

    async fn scan_for_signals(&self) -> AppResult<Vec<Signal>> {
        let liquidatable = self.get_liquidatable_accounts(100.0).await?;

        let signals: Vec<Signal> = liquidatable
            .iter()
            .map(|acc| self.account_to_signal(acc))
            .collect();

        Ok(signals)
    }

    async fn estimate_profit(&self, signal: &Signal) -> AppResult<ProfitEstimate> {
        let estimated_profit = signal
            .metadata
            .get("estimated_profit_lamports")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        let gas_estimate = 50_000i64;

        Ok(ProfitEstimate {
            signal_id: signal.id,
            estimated_profit_lamports: estimated_profit,
            estimated_gas_lamports: gas_estimate,
            net_profit_lamports: estimated_profit - gas_estimate,
            profit_bps: signal.estimated_profit_bps,
            confidence: signal.confidence,
            route: Some(serde_json::json!({
                "type": "liquidation",
                "venue": "marginfi",
                "action": "liquidate_account",
            })),
        })
    }

    async fn get_quote(&self, _params: &QuoteParams) -> AppResult<Quote> {
        Err(AppError::BadRequest(
            "Lending venues don't support direct quotes".to_string(),
        ))
    }

    async fn is_healthy(&self) -> bool {
        let url = format!("{}/health", self.base_url);
        self.client
            .get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginfiAccount {
    pub address: String,
    pub owner: String,
    pub health_factor: f64,
    pub total_collateral_usd: f64,
    pub total_borrowed_usd: f64,
    pub liquidation_threshold: f64,
    pub collateral_token: Option<String>,
    pub debt_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginfiAccountsResponse {
    pub accounts: Vec<MarginfiAccount>,
    pub total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountHealth {
    pub address: String,
    pub health_factor: f64,
    pub utilization_rate: f64,
    pub at_risk: bool,
    pub positions: Vec<AccountPosition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountPosition {
    pub token_mint: String,
    pub token_symbol: String,
    pub position_type: String,
    pub amount: f64,
    pub value_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidatableAccount {
    pub address: String,
    pub health_factor: f64,
    pub total_collateral_usd: f64,
    pub total_borrowed_usd: f64,
    pub shortfall_usd: f64,
    pub liquidation_bonus_usd: f64,
    pub estimated_profit_lamports: i64,
    pub collateral_token: Option<String>,
    pub debt_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginfiPool {
    pub address: String,
    pub token_mint: String,
    pub token_symbol: String,
    pub total_deposits_usd: f64,
    pub total_borrows_usd: f64,
    pub utilization_rate: f64,
    pub deposit_apy: f64,
    pub borrow_apy: f64,
    pub liquidation_threshold: f64,
    pub liquidation_bonus: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginfiPoolsResponse {
    pub pools: Vec<MarginfiPool>,
}
