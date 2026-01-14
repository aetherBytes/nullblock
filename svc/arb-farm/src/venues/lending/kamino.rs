use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::events::Significance;
use crate::models::{Signal, SignalType, VenueType};
use crate::venues::{MevVenue, ProfitEstimate, Quote, QuoteParams};

pub struct KaminoVenue {
    id: Uuid,
    client: Client,
    base_url: String,
    rpc_url: String,
}

impl KaminoVenue {
    pub fn new(base_url: String, rpc_url: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            client: Client::new(),
            base_url,
            rpc_url,
        }
    }

    pub async fn get_obligations_at_risk(&self) -> AppResult<Vec<KaminoObligation>> {
        let url = format!("{}/obligations/at-risk", self.base_url);

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(15))
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Kamino obligations request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::ExternalApi(format!(
                "Kamino returned error: {}",
                response.status()
            )));
        }

        let obligations: KaminoObligationsResponse = response
            .json()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse Kamino response: {}", e)))?;

        Ok(obligations.obligations)
    }

    pub async fn get_obligation_details(&self, obligation_address: &str) -> AppResult<KaminoObligationDetail> {
        let url = format!("{}/obligations/{}", self.base_url, obligation_address);

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Kamino obligation request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::ExternalApi(format!(
                "Kamino obligation returned error: {}",
                response.status()
            )));
        }

        response
            .json()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse obligation response: {}", e)))
    }

    pub async fn get_liquidatable_obligations(&self, min_shortfall: f64) -> AppResult<Vec<LiquidatableObligation>> {
        let obligations = self.get_obligations_at_risk().await?;
        let mut liquidatable = Vec::new();

        for obligation in obligations {
            let ltv = if obligation.deposited_value_usd > 0.0 {
                obligation.borrowed_value_usd / obligation.deposited_value_usd
            } else {
                0.0
            };

            if ltv > obligation.liquidation_ltv {
                let shortfall = obligation.borrowed_value_usd - (obligation.deposited_value_usd * obligation.liquidation_ltv);

                if shortfall >= min_shortfall {
                    let max_liquidation = obligation.borrowed_value_usd * 0.5;
                    let liquidation_bonus = max_liquidation * obligation.liquidation_bonus_bps as f64 / 10000.0;
                    let estimated_profit = (liquidation_bonus - 0.001) * 1e9;

                    liquidatable.push(LiquidatableObligation {
                        address: obligation.address.clone(),
                        owner: obligation.owner.clone(),
                        ltv,
                        liquidation_ltv: obligation.liquidation_ltv,
                        deposited_value_usd: obligation.deposited_value_usd,
                        borrowed_value_usd: obligation.borrowed_value_usd,
                        shortfall_usd: shortfall,
                        max_liquidation_usd: max_liquidation,
                        liquidation_bonus_usd: liquidation_bonus,
                        estimated_profit_lamports: estimated_profit as i64,
                        collateral_reserves: obligation.collateral_reserves.clone(),
                        borrow_reserves: obligation.borrow_reserves.clone(),
                    });
                }
            }
        }

        liquidatable.sort_by(|a, b| {
            b.estimated_profit_lamports.cmp(&a.estimated_profit_lamports)
        });

        Ok(liquidatable)
    }

    pub async fn get_reserves(&self) -> AppResult<Vec<KaminoReserve>> {
        let url = format!("{}/reserves", self.base_url);

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Kamino reserves request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::ExternalApi(format!(
                "Kamino reserves returned error: {}",
                response.status()
            )));
        }

        let reserves: KaminoReservesResponse = response
            .json()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse reserves response: {}", e)))?;

        Ok(reserves.reserves)
    }

    fn obligation_to_signal(&self, obligation: &LiquidatableObligation) -> Signal {
        let profit_bps = if obligation.deposited_value_usd > 0.0 {
            ((obligation.liquidation_bonus_usd / obligation.max_liquidation_usd) * 10000.0) as i32
        } else {
            0
        };

        let significance = if obligation.shortfall_usd > 10000.0 {
            Significance::Critical
        } else if obligation.shortfall_usd > 1000.0 {
            Significance::High
        } else {
            Significance::Medium
        };

        let confidence = if obligation.ltv > obligation.liquidation_ltv * 1.2 {
            0.95
        } else if obligation.ltv > obligation.liquidation_ltv * 1.1 {
            0.8
        } else {
            0.6
        };

        let collateral_token = obligation
            .collateral_reserves
            .first()
            .map(|r| r.token_mint.clone());

        Signal {
            id: Uuid::new_v4(),
            signal_type: SignalType::Liquidation,
            venue_id: self.id,
            venue_type: VenueType::Lending,
            token_mint: collateral_token,
            pool_address: Some(obligation.address.clone()),
            estimated_profit_bps: profit_bps,
            confidence,
            significance,
            metadata: serde_json::json!({
                "venue": "kamino",
                "obligation_address": obligation.address,
                "owner": obligation.owner,
                "ltv": obligation.ltv,
                "liquidation_ltv": obligation.liquidation_ltv,
                "deposited_value_usd": obligation.deposited_value_usd,
                "borrowed_value_usd": obligation.borrowed_value_usd,
                "shortfall_usd": obligation.shortfall_usd,
                "max_liquidation_usd": obligation.max_liquidation_usd,
                "liquidation_bonus_usd": obligation.liquidation_bonus_usd,
                "estimated_profit_lamports": obligation.estimated_profit_lamports,
                "collateral_reserves": obligation.collateral_reserves,
                "borrow_reserves": obligation.borrow_reserves,
                "atomicity": "non_atomic",
            }),
            detected_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + chrono::Duration::minutes(1),
        }
    }
}

#[async_trait]
impl MevVenue for KaminoVenue {
    fn venue_id(&self) -> Uuid {
        self.id
    }

    fn venue_type(&self) -> VenueType {
        VenueType::Lending
    }

    fn name(&self) -> &str {
        "Kamino"
    }

    async fn scan_for_signals(&self) -> AppResult<Vec<Signal>> {
        let liquidatable = self.get_liquidatable_obligations(100.0).await?;

        let signals: Vec<Signal> = liquidatable
            .iter()
            .map(|obl| self.obligation_to_signal(obl))
            .collect();

        Ok(signals)
    }

    async fn estimate_profit(&self, signal: &Signal) -> AppResult<ProfitEstimate> {
        let estimated_profit = signal
            .metadata
            .get("estimated_profit_lamports")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        let gas_estimate = 60_000i64;

        Ok(ProfitEstimate {
            signal_id: signal.id,
            estimated_profit_lamports: estimated_profit,
            estimated_gas_lamports: gas_estimate,
            net_profit_lamports: estimated_profit - gas_estimate,
            profit_bps: signal.estimated_profit_bps,
            confidence: signal.confidence,
            route: Some(serde_json::json!({
                "type": "liquidation",
                "venue": "kamino",
                "action": "liquidate_obligation",
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
pub struct KaminoObligation {
    pub address: String,
    pub owner: String,
    pub deposited_value_usd: f64,
    pub borrowed_value_usd: f64,
    pub liquidation_ltv: f64,
    pub liquidation_bonus_bps: u16,
    pub collateral_reserves: Vec<ReservePosition>,
    pub borrow_reserves: Vec<ReservePosition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReservePosition {
    pub reserve_address: String,
    pub token_mint: String,
    pub token_symbol: String,
    pub amount: f64,
    pub value_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KaminoObligationsResponse {
    pub obligations: Vec<KaminoObligation>,
    pub total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KaminoObligationDetail {
    pub obligation: KaminoObligation,
    pub health_metrics: HealthMetrics,
    pub liquidation_info: Option<LiquidationInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    pub current_ltv: f64,
    pub max_ltv: f64,
    pub liquidation_ltv: f64,
    pub available_borrow_usd: f64,
    pub utilization_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidationInfo {
    pub is_liquidatable: bool,
    pub shortfall_usd: f64,
    pub max_liquidation_usd: f64,
    pub liquidation_bonus_bps: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidatableObligation {
    pub address: String,
    pub owner: String,
    pub ltv: f64,
    pub liquidation_ltv: f64,
    pub deposited_value_usd: f64,
    pub borrowed_value_usd: f64,
    pub shortfall_usd: f64,
    pub max_liquidation_usd: f64,
    pub liquidation_bonus_usd: f64,
    pub estimated_profit_lamports: i64,
    pub collateral_reserves: Vec<ReservePosition>,
    pub borrow_reserves: Vec<ReservePosition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KaminoReserve {
    pub address: String,
    pub token_mint: String,
    pub token_symbol: String,
    pub total_supply_usd: f64,
    pub total_borrow_usd: f64,
    pub utilization_rate: f64,
    pub supply_apy: f64,
    pub borrow_apy: f64,
    pub liquidation_threshold: f64,
    pub liquidation_bonus_bps: u16,
    pub max_ltv: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KaminoReservesResponse {
    pub reserves: Vec<KaminoReserve>,
}
