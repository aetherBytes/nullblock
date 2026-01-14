use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::{Signal, SignalType, VenueType};
use crate::events::Significance;
use crate::venues::{MevVenue, ProfitEstimate, Quote, QuoteParams};

pub struct PumpFunVenue {
    id: Uuid,
    client: Client,
    base_url: String,
}

impl PumpFunVenue {
    pub fn new(base_url: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            client: Client::new(),
            base_url,
        }
    }

    pub async fn get_token_info(&self, mint: &str) -> AppResult<PumpFunToken> {
        let url = format!("{}/coins/{}", self.base_url, mint);

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("pump.fun token request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::ExternalApi(format!(
                "pump.fun returned error: {}",
                response.status()
            )));
        }

        response
            .json()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse pump.fun response: {}", e)))
    }

    pub async fn get_new_tokens(&self, limit: u32) -> AppResult<Vec<PumpFunToken>> {
        let url = format!("{}/coins?offset=0&limit={}&sort=created_timestamp&order=DESC",
            self.base_url, limit);

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(15))
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("pump.fun list request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::ExternalApi(format!(
                "pump.fun list returned error: {}",
                response.status()
            )));
        }

        response
            .json()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse pump.fun list: {}", e)))
    }

    pub async fn get_graduation_progress(&self, mint: &str) -> AppResult<GraduationProgress> {
        let token = self.get_token_info(mint).await?;

        let progress_percent = if token.bonding_curve_complete {
            100.0
        } else {
            (token.market_cap / token.graduation_threshold.unwrap_or(69000.0)) * 100.0
        };

        let estimated_blocks = if progress_percent >= 100.0 {
            0
        } else {
            let remaining_percent = 100.0 - progress_percent;
            let velocity = token.volume_24h / 24.0; // Rough hourly volume
            if velocity > 0.0 {
                ((remaining_percent / velocity) * 600.0) as u64 // ~600 blocks/hour on Solana
            } else {
                u64::MAX
            }
        };

        Ok(GraduationProgress {
            mint: mint.to_string(),
            progress_percent,
            market_cap: token.market_cap,
            graduation_threshold: token.graduation_threshold.unwrap_or(69000.0),
            is_graduated: token.bonding_curve_complete,
            raydium_pool: token.raydium_pool,
            estimated_blocks_to_graduation: estimated_blocks,
        })
    }

    pub async fn get_holder_stats(&self, mint: &str) -> AppResult<HolderStats> {
        let url = format!("{}/coins/{}/holders", self.base_url, mint);

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("pump.fun holders request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::ExternalApi(format!(
                "pump.fun holders returned error: {}",
                response.status()
            )));
        }

        let holders: Vec<PumpFunHolder> = response
            .json()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse holders: {}", e)))?;

        let total_supply: f64 = holders.iter().map(|h| h.balance).sum();
        let top_10_balance: f64 = holders.iter().take(10).map(|h| h.balance).sum();
        let top_10_concentration = if total_supply > 0.0 {
            (top_10_balance / total_supply) * 100.0
        } else {
            0.0
        };

        Ok(HolderStats {
            mint: mint.to_string(),
            total_holders: holders.len() as u32,
            top_10_concentration,
            top_holders: holders.into_iter().take(10).collect(),
        })
    }

    pub async fn compute_buy_quote(
        &self,
        mint: &str,
        sol_amount: f64,
    ) -> AppResult<PumpFunQuote> {
        let token = self.get_token_info(mint).await?;

        if token.bonding_curve_complete {
            return Err(AppError::Internal(
                "Token has graduated, use DEX for trading".to_string()
            ));
        }

        // Simplified bonding curve calculation
        // Real implementation would use actual curve parameters
        let price_per_token = token.market_cap / token.total_supply;
        let tokens_out = sol_amount / price_per_token;
        let price_impact = (sol_amount / token.market_cap) * 100.0;

        Ok(PumpFunQuote {
            mint: mint.to_string(),
            is_buy: true,
            sol_amount,
            token_amount: tokens_out,
            price_per_token,
            price_impact_percent: price_impact,
            fee_sol: sol_amount * 0.01, // 1% fee estimate
        })
    }

    pub async fn compute_sell_quote(
        &self,
        mint: &str,
        token_amount: f64,
    ) -> AppResult<PumpFunQuote> {
        let token = self.get_token_info(mint).await?;

        if token.bonding_curve_complete {
            return Err(AppError::Internal(
                "Token has graduated, use DEX for trading".to_string()
            ));
        }

        let price_per_token = token.market_cap / token.total_supply;
        let sol_out = token_amount * price_per_token;
        let price_impact = (sol_out / token.market_cap) * 100.0;

        Ok(PumpFunQuote {
            mint: mint.to_string(),
            is_buy: false,
            sol_amount: sol_out,
            token_amount,
            price_per_token,
            price_impact_percent: price_impact,
            fee_sol: sol_out * 0.01,
        })
    }

    pub async fn detect_graduation_opportunities(&self) -> AppResult<Vec<Signal>> {
        let tokens = self.get_new_tokens(50).await?;
        let mut signals = Vec::new();

        for token in tokens {
            if token.bonding_curve_complete {
                continue;
            }

            let progress = (token.market_cap / token.graduation_threshold.unwrap_or(69000.0)) * 100.0;

            // Look for tokens approaching graduation (70-95% progress)
            if progress >= 70.0 && progress < 95.0 {
                let significance = if progress >= 85.0 {
                    Significance::High
                } else {
                    Significance::Medium
                };

                signals.push(Signal {
                    id: Uuid::new_v4(),
                    signal_type: SignalType::CurveGraduation,
                    venue_id: self.id,
                    venue_type: VenueType::BondingCurve,
                    token_mint: Some(token.mint.clone()),
                    pool_address: None,
                    estimated_profit_bps: ((100.0 - progress) * 10.0) as i32, // Rough estimate
                    confidence: progress / 100.0,
                    significance,
                    metadata: serde_json::json!({
                        "token_name": token.name,
                        "token_symbol": token.symbol,
                        "progress_percent": progress,
                        "market_cap": token.market_cap,
                        "volume_24h": token.volume_24h,
                        "graduation_threshold": token.graduation_threshold,
                    }),
                    detected_at: chrono::Utc::now(),
                    expires_at: chrono::Utc::now() + chrono::Duration::minutes(5),
                });
            }
        }

        Ok(signals)
    }
}

#[async_trait]
impl MevVenue for PumpFunVenue {
    fn venue_id(&self) -> Uuid {
        self.id
    }

    fn venue_type(&self) -> VenueType {
        VenueType::BondingCurve
    }

    fn name(&self) -> &str {
        "pump.fun"
    }

    async fn scan_for_signals(&self) -> AppResult<Vec<Signal>> {
        self.detect_graduation_opportunities().await
    }

    async fn estimate_profit(&self, signal: &Signal) -> AppResult<ProfitEstimate> {
        let profit_estimate = match signal.signal_type {
            SignalType::CurveGraduation => {
                // Estimate profit from buying pre-graduation and selling post-graduation
                let progress = signal.metadata.get("progress_percent")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);

                // Higher progress = lower profit potential but higher confidence
                let estimated_profit = if progress >= 90.0 {
                    5000 // 0.5% in lamports basis
                } else if progress >= 80.0 {
                    10000 // 1%
                } else {
                    20000 // 2%
                };

                ProfitEstimate {
                    signal_id: signal.id,
                    estimated_profit_lamports: estimated_profit,
                    estimated_gas_lamports: 10000, // ~0.00001 SOL
                    net_profit_lamports: estimated_profit - 10000,
                    profit_bps: (estimated_profit / 100) as i32,
                    confidence: signal.confidence,
                    route: Some(serde_json::json!({
                        "type": "curve_graduation",
                        "action": "buy_pre_graduation_sell_post",
                    })),
                }
            }
            _ => ProfitEstimate {
                signal_id: signal.id,
                estimated_profit_lamports: 0,
                estimated_gas_lamports: 5000,
                net_profit_lamports: 0,
                profit_bps: 0,
                confidence: 0.0,
                route: None,
            },
        };

        Ok(profit_estimate)
    }

    async fn get_quote(&self, params: &QuoteParams) -> AppResult<Quote> {
        // For bonding curves, we treat the curve token as output_mint
        let quote = self.compute_buy_quote(&params.output_mint, params.amount_lamports as f64 / 1e9).await?;

        Ok(Quote {
            input_mint: params.input_mint.clone(),
            output_mint: params.output_mint.clone(),
            input_amount: params.amount_lamports,
            output_amount: (quote.token_amount * 1e9) as u64,
            price_impact_bps: (quote.price_impact_percent * 100.0) as i32,
            route_plan: serde_json::json!({
                "venue": "pump.fun",
                "type": "bonding_curve_buy",
                "fee_sol": quote.fee_sol,
            }),
            expires_at: chrono::Utc::now() + chrono::Duration::seconds(30),
        })
    }

    async fn is_healthy(&self) -> bool {
        let url = format!("{}/coins?limit=1", self.base_url);
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
#[serde(rename_all = "camelCase")]
pub struct PumpFunToken {
    pub mint: String,
    pub name: String,
    pub symbol: String,
    pub description: Option<String>,
    pub image_uri: Option<String>,
    pub creator: String,
    pub created_timestamp: i64,
    pub market_cap: f64,
    pub total_supply: f64,
    pub volume_24h: f64,
    pub price_change_24h: Option<f64>,
    #[serde(default)]
    pub bonding_curve_complete: bool,
    pub raydium_pool: Option<String>,
    pub graduation_threshold: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PumpFunHolder {
    pub address: String,
    pub balance: f64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraduationProgress {
    pub mint: String,
    pub progress_percent: f64,
    pub market_cap: f64,
    pub graduation_threshold: f64,
    pub is_graduated: bool,
    pub raydium_pool: Option<String>,
    pub estimated_blocks_to_graduation: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HolderStats {
    pub mint: String,
    pub total_holders: u32,
    pub top_10_concentration: f64,
    pub top_holders: Vec<PumpFunHolder>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PumpFunQuote {
    pub mint: String,
    pub is_buy: bool,
    pub sol_amount: f64,
    pub token_amount: f64,
    pub price_per_token: f64,
    pub price_impact_percent: f64,
    pub fee_sol: f64,
}
