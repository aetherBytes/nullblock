use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::events::Significance;
use crate::models::{Signal, SignalType, VenueType};
use crate::venues::{MevVenue, ProfitEstimate, Quote, QuoteParams};

pub struct MoonshotVenue {
    id: Uuid,
    client: Client,
    base_url: String,
}

impl MoonshotVenue {
    pub fn new(base_url: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            client: Client::new(),
            base_url,
        }
    }

    pub async fn get_token_info(&self, mint: &str) -> AppResult<MoonshotToken> {
        // Use DexScreener to get token info
        let url = format!("{}/tokens/{}", self.base_url, mint);

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("DexScreener token request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::ExternalApi(format!(
                "DexScreener returned error: {}",
                response.status()
            )));
        }

        let dex_response: DexScreenerTokenResponse = response
            .json()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse DexScreener response: {}", e)))?;

        // Find a moonshot pair
        let pair = dex_response.pairs
            .into_iter()
            .find(|p| p.dex_id == "moonshot")
            .ok_or_else(|| AppError::NotFound(format!("No moonshot pair found for {}", mint)))?;

        Ok(MoonshotToken::from_dexscreener(pair))
    }

    pub async fn get_new_tokens(&self, limit: u32) -> AppResult<Vec<MoonshotToken>> {
        // Use DexScreener search for moonshot tokens
        let url = format!("{}/search?q=moonshot", self.base_url);

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(15))
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("DexScreener search failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::ExternalApi(format!(
                "DexScreener returned error: {}",
                response.status()
            )));
        }

        let dex_response: DexScreenerSearchResponse = response
            .json()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse DexScreener response: {}", e)))?;

        let tokens: Vec<MoonshotToken> = dex_response.pairs
            .into_iter()
            .filter(|p| p.dex_id == "moonshot")
            .take(limit as usize)
            .map(MoonshotToken::from_dexscreener)
            .collect();

        Ok(tokens)
    }

    pub async fn get_graduation_progress(&self, mint: &str) -> AppResult<MoonshotGraduationProgress> {
        let token = self.get_token_info(mint).await?;

        let progress_percent = if token.is_graduated {
            100.0
        } else {
            let threshold = token.graduation_market_cap.unwrap_or(500_000.0);
            (token.market_cap_usd / threshold) * 100.0
        };

        let estimated_blocks = if progress_percent >= 100.0 {
            0
        } else {
            let remaining_percent = 100.0 - progress_percent;
            let velocity = token.volume_24h_usd / 24.0;
            if velocity > 0.0 {
                ((remaining_percent / velocity) * 600.0) as u64
            } else {
                u64::MAX
            }
        };

        Ok(MoonshotGraduationProgress {
            mint: mint.to_string(),
            progress_percent,
            market_cap_usd: token.market_cap_usd,
            graduation_threshold_usd: token.graduation_market_cap.unwrap_or(500_000.0),
            is_graduated: token.is_graduated,
            dex_pool: token.dex_pool_address,
            estimated_blocks_to_graduation: estimated_blocks,
            curve_type: token.curve_type.clone(),
        })
    }

    pub async fn get_holder_stats(&self, mint: &str) -> AppResult<MoonshotHolderStats> {
        let url = format!("{}/token/{}/holders", self.base_url, mint);

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| {
                AppError::ExternalApi(format!("moonshot holders request failed: {}", e))
            })?;

        if !response.status().is_success() {
            return Err(AppError::ExternalApi(format!(
                "moonshot holders returned error: {}",
                response.status()
            )));
        }

        let holders: Vec<MoonshotHolder> = response
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

        Ok(MoonshotHolderStats {
            mint: mint.to_string(),
            total_holders: holders.len() as u32,
            top_10_concentration,
            top_holders: holders.into_iter().take(10).collect(),
        })
    }

    pub async fn compute_buy_quote(&self, mint: &str, sol_amount: f64) -> AppResult<MoonshotQuote> {
        let token = self.get_token_info(mint).await?;

        if token.is_graduated {
            return Err(AppError::Internal(
                "Token has graduated, use DEX for trading".to_string(),
            ));
        }

        let price_per_token = token.price_sol;
        let tokens_out = sol_amount / price_per_token;
        let price_impact = self.calculate_price_impact(&token, sol_amount, true);

        Ok(MoonshotQuote {
            mint: mint.to_string(),
            is_buy: true,
            sol_amount,
            token_amount: tokens_out,
            price_per_token,
            price_impact_percent: price_impact,
            fee_sol: sol_amount * 0.01, // 1% fee estimate
            curve_type: token.curve_type,
        })
    }

    pub async fn compute_sell_quote(
        &self,
        mint: &str,
        token_amount: f64,
    ) -> AppResult<MoonshotQuote> {
        let token = self.get_token_info(mint).await?;

        if token.is_graduated {
            return Err(AppError::Internal(
                "Token has graduated, use DEX for trading".to_string(),
            ));
        }

        let price_per_token = token.price_sol;
        let sol_out = token_amount * price_per_token;
        let price_impact = self.calculate_price_impact(&token, sol_out, false);

        Ok(MoonshotQuote {
            mint: mint.to_string(),
            is_buy: false,
            sol_amount: sol_out,
            token_amount,
            price_per_token,
            price_impact_percent: price_impact,
            fee_sol: sol_out * 0.01,
            curve_type: token.curve_type,
        })
    }

    fn calculate_price_impact(&self, token: &MoonshotToken, sol_amount: f64, is_buy: bool) -> f64 {
        let market_cap_sol = token.market_cap_usd / token.sol_price_usd.unwrap_or(100.0);
        let base_impact = (sol_amount / market_cap_sol) * 100.0;

        match token.curve_type.as_str() {
            "linear" => base_impact,
            "exponential" => base_impact * 1.5,
            "sigmoid" => {
                let progress = token.market_cap_usd
                    / token.graduation_market_cap.unwrap_or(500_000.0);
                if progress > 0.8 {
                    base_impact * 2.0
                } else {
                    base_impact
                }
            }
            _ => base_impact,
        }
    }

    pub async fn detect_graduation_opportunities(&self) -> AppResult<Vec<Signal>> {
        let tokens = self.get_new_tokens(50).await?;
        let mut signals = Vec::new();

        for token in tokens {
            if token.is_graduated {
                continue;
            }

            let threshold = token.graduation_market_cap.unwrap_or(500_000.0);
            let progress = (token.market_cap_usd / threshold) * 100.0;

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
                    estimated_profit_bps: ((100.0 - progress) * 10.0) as i32,
                    confidence: progress / 100.0,
                    significance,
                    metadata: serde_json::json!({
                        "token_name": token.name,
                        "token_symbol": token.symbol,
                        "progress_percent": progress,
                        "market_cap_usd": token.market_cap_usd,
                        "volume_24h_usd": token.volume_24h_usd,
                        "graduation_threshold_usd": threshold,
                        "curve_type": token.curve_type,
                        "venue": "moonshot",
                    }),
                    detected_at: chrono::Utc::now(),
                    expires_at: chrono::Utc::now() + chrono::Duration::minutes(5),
                });
            }
        }

        Ok(signals)
    }

    pub async fn get_curve_parameters(&self, mint: &str) -> AppResult<CurveParameters> {
        let token = self.get_token_info(mint).await?;

        Ok(CurveParameters {
            mint: mint.to_string(),
            curve_type: token.curve_type,
            initial_price: token.initial_price_sol.unwrap_or(0.0),
            current_price: token.price_sol,
            graduation_market_cap: token.graduation_market_cap.unwrap_or(500_000.0),
            total_supply: token.total_supply,
            circulating_supply: token.circulating_supply,
            curve_progress: token.market_cap_usd
                / token.graduation_market_cap.unwrap_or(500_000.0),
        })
    }
}

#[async_trait]
impl MevVenue for MoonshotVenue {
    fn venue_id(&self) -> Uuid {
        self.id
    }

    fn venue_type(&self) -> VenueType {
        VenueType::BondingCurve
    }

    fn name(&self) -> &str {
        "moonshot"
    }

    async fn scan_for_signals(&self) -> AppResult<Vec<Signal>> {
        self.detect_graduation_opportunities().await
    }

    async fn estimate_profit(&self, signal: &Signal) -> AppResult<ProfitEstimate> {
        let profit_estimate = match signal.signal_type {
            SignalType::CurveGraduation => {
                let progress = signal
                    .metadata
                    .get("progress_percent")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);

                let curve_type = signal
                    .metadata
                    .get("curve_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("linear");

                let base_profit = if progress >= 90.0 {
                    5000
                } else if progress >= 80.0 {
                    10000
                } else {
                    20000
                };

                let adjusted_profit = match curve_type {
                    "exponential" => base_profit * 3 / 2,
                    "sigmoid" => {
                        if progress > 80.0 {
                            base_profit * 2
                        } else {
                            base_profit
                        }
                    }
                    _ => base_profit,
                };

                ProfitEstimate {
                    signal_id: signal.id,
                    estimated_profit_lamports: adjusted_profit,
                    estimated_gas_lamports: 10000,
                    net_profit_lamports: adjusted_profit - 10000,
                    profit_bps: (adjusted_profit / 100) as i32,
                    confidence: signal.confidence,
                    route: Some(serde_json::json!({
                        "type": "curve_graduation",
                        "venue": "moonshot",
                        "curve_type": curve_type,
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
        let quote = self
            .compute_buy_quote(&params.output_mint, params.amount_lamports as f64 / 1e9)
            .await?;

        Ok(Quote {
            input_mint: params.input_mint.clone(),
            output_mint: params.output_mint.clone(),
            input_amount: params.amount_lamports,
            output_amount: (quote.token_amount * 1e9) as u64,
            price_impact_bps: (quote.price_impact_percent * 100.0) as i32,
            route_plan: serde_json::json!({
                "venue": "moonshot",
                "type": "bonding_curve_buy",
                "curve_type": quote.curve_type,
                "fee_sol": quote.fee_sol,
            }),
            expires_at: chrono::Utc::now() + chrono::Duration::seconds(30),
        })
    }

    async fn is_healthy(&self) -> bool {
        // Test DexScreener API with a moonshot search
        let url = format!("{}/search?q=moonshot", self.base_url);
        self.client
            .get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }
}

// DexScreener response types
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DexScreenerTokenResponse {
    pub pairs: Vec<DexScreenerPair>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DexScreenerSearchResponse {
    pub pairs: Vec<DexScreenerPair>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DexScreenerPair {
    pub chain_id: String,
    pub dex_id: String,
    pub pair_address: String,
    pub base_token: DexScreenerToken,
    pub quote_token: DexScreenerToken,
    pub price_native: Option<String>,
    pub price_usd: Option<String>,
    pub volume: Option<DexScreenerVolume>,
    pub price_change: Option<DexScreenerPriceChange>,
    pub fdv: Option<f64>,
    pub market_cap: Option<f64>,
    pub pair_created_at: Option<i64>,
    pub moonshot: Option<DexScreenerMoonshotInfo>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DexScreenerToken {
    pub address: String,
    pub name: String,
    pub symbol: String,
}

#[derive(Debug, Deserialize)]
pub struct DexScreenerVolume {
    pub h24: Option<f64>,
    pub h6: Option<f64>,
    pub h1: Option<f64>,
    pub m5: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct DexScreenerPriceChange {
    pub h24: Option<f64>,
    pub h6: Option<f64>,
    pub h1: Option<f64>,
    pub m5: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DexScreenerMoonshotInfo {
    pub progress: Option<f64>,
    pub creator: Option<String>,
    pub curve_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoonshotToken {
    pub mint: String,
    pub name: String,
    pub symbol: String,
    pub description: Option<String>,
    pub image_uri: Option<String>,
    pub creator: String,
    pub created_at: i64,
    pub market_cap_usd: f64,
    pub price_sol: f64,
    pub price_usd: f64,
    pub sol_price_usd: Option<f64>,
    pub total_supply: f64,
    pub circulating_supply: f64,
    pub volume_24h_usd: f64,
    pub price_change_24h: Option<f64>,
    #[serde(default)]
    pub is_graduated: bool,
    pub dex_pool_address: Option<String>,
    pub graduation_market_cap: Option<f64>,
    #[serde(default = "default_curve_type")]
    pub curve_type: String,
    pub initial_price_sol: Option<f64>,
}

impl MoonshotToken {
    pub fn from_dexscreener(pair: DexScreenerPair) -> Self {
        let market_cap = pair.market_cap.or(pair.fdv).unwrap_or(0.0);
        let volume_24h = pair.volume.as_ref().and_then(|v| v.h24).unwrap_or(0.0);
        let price_change_24h = pair.price_change.as_ref().and_then(|p| p.h24);
        let price_usd = pair.price_usd.as_ref()
            .and_then(|p| p.parse::<f64>().ok())
            .unwrap_or(0.0);
        let price_native = pair.price_native.as_ref()
            .and_then(|p| p.parse::<f64>().ok())
            .unwrap_or(0.0);

        let moonshot_info = pair.moonshot.as_ref();
        let progress = moonshot_info.and_then(|m| m.progress).unwrap_or(0.0);
        let curve_type = moonshot_info
            .and_then(|m| m.curve_type.clone())
            .unwrap_or_else(|| "linear".to_string());
        let creator = moonshot_info
            .and_then(|m| m.creator.clone())
            .unwrap_or_default();

        // Moonshot graduation is typically at 100% progress
        let is_graduated = progress >= 100.0;

        Self {
            mint: pair.base_token.address,
            name: pair.base_token.name,
            symbol: pair.base_token.symbol,
            description: None,
            image_uri: None,
            creator,
            created_at: pair.pair_created_at.unwrap_or(0) / 1000,
            market_cap_usd: market_cap,
            price_sol: price_native,
            price_usd,
            sol_price_usd: Some(price_usd / price_native.max(0.0001)),
            total_supply: 1_000_000_000.0,
            circulating_supply: 1_000_000_000.0,
            volume_24h_usd: volume_24h,
            price_change_24h,
            is_graduated,
            dex_pool_address: if is_graduated { Some(pair.pair_address) } else { None },
            graduation_market_cap: Some(500_000.0), // Typical moonshot threshold
            curve_type,
            initial_price_sol: None,
        }
    }
}

fn default_curve_type() -> String {
    "linear".to_string()
}

#[derive(Debug, Deserialize)]
pub struct MoonshotListResponse {
    pub tokens: Vec<MoonshotToken>,
    pub total: Option<u64>,
    pub has_more: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoonshotHolder {
    pub address: String,
    pub balance: f64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoonshotGraduationProgress {
    pub mint: String,
    pub progress_percent: f64,
    pub market_cap_usd: f64,
    pub graduation_threshold_usd: f64,
    pub is_graduated: bool,
    pub dex_pool: Option<String>,
    pub estimated_blocks_to_graduation: u64,
    pub curve_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoonshotHolderStats {
    pub mint: String,
    pub total_holders: u32,
    pub top_10_concentration: f64,
    pub top_holders: Vec<MoonshotHolder>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoonshotQuote {
    pub mint: String,
    pub is_buy: bool,
    pub sol_amount: f64,
    pub token_amount: f64,
    pub price_per_token: f64,
    pub price_impact_percent: f64,
    pub fee_sol: f64,
    pub curve_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurveParameters {
    pub mint: String,
    pub curve_type: String,
    pub initial_price: f64,
    pub current_price: f64,
    pub graduation_market_cap: f64,
    pub total_supply: f64,
    pub circulating_supply: f64,
    pub curve_progress: f64,
}
