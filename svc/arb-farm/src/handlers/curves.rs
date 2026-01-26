use axum::{
    extract::{Path, Query, State},
    Json,
};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use crate::agents::{
    DetailedCurveMetrics, OpportunityScore, Recommendation, ScoringWeights, ScoringThresholds,
};
use crate::error::{AppError, AppResult};
use crate::execution::{CurveBuildResult, CurveBuyParams, CurveSellParams, SimulatedTrade};
use crate::server::AppState;
use crate::venues::curves::{
    derive_pump_fun_bonding_curve, GraduationStatus, OnChainCurveState, RaydiumPoolInfo,
    HolderDistribution,
    moonshot::{MoonshotGraduationProgress, MoonshotHolderStats, MoonshotQuote, CurveParameters},
    pump_fun::{GraduationProgress, HolderStats, PumpFunQuote},
};
use crate::venues::MevVenue;

#[derive(Debug, Deserialize)]
pub struct GetProgressQuery {
    pub venue: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum GraduationProgressResponse {
    PumpFun(GraduationProgress),
    Moonshot(MoonshotGraduationProgress),
}

pub async fn get_graduation_progress(
    State(state): State<AppState>,
    Path(mint): Path<String>,
    Query(query): Query<GetProgressQuery>,
) -> AppResult<Json<GraduationProgressResponse>> {
    let venue = query.venue.as_deref().unwrap_or("pump_fun");

    match venue {
        "pump_fun" | "pumpfun" => {
            let progress = state.pump_fun_venue.get_graduation_progress(&mint).await?;
            Ok(Json(GraduationProgressResponse::PumpFun(progress)))
        }
        "moonshot" => {
            let progress = state.moonshot_venue.get_graduation_progress(&mint).await?;
            Ok(Json(GraduationProgressResponse::Moonshot(progress)))
        }
        _ => Err(AppError::BadRequest(format!("Unknown venue: {}", venue))),
    }
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum HolderStatsResponse {
    PumpFun(HolderStats),
    Moonshot(MoonshotHolderStats),
}

pub async fn get_holder_stats(
    State(state): State<AppState>,
    Path(mint): Path<String>,
    Query(query): Query<GetProgressQuery>,
) -> AppResult<Json<HolderStatsResponse>> {
    let venue = query.venue.as_deref().unwrap_or("pump_fun");

    match venue {
        "pump_fun" | "pumpfun" => {
            let stats = state.pump_fun_venue.get_holder_stats(&mint).await?;
            Ok(Json(HolderStatsResponse::PumpFun(stats)))
        }
        "moonshot" => {
            let stats = state.moonshot_venue.get_holder_stats(&mint).await?;
            Ok(Json(HolderStatsResponse::Moonshot(stats)))
        }
        _ => Err(AppError::BadRequest(format!("Unknown venue: {}", venue))),
    }
}

#[derive(Debug, Deserialize)]
pub struct QuoteRequest {
    pub venue: Option<String>,
    pub is_buy: bool,
    pub amount: f64,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum QuoteResponse {
    PumpFun(PumpFunQuote),
    Moonshot(MoonshotQuote),
}

pub async fn get_curve_quote(
    State(state): State<AppState>,
    Path(mint): Path<String>,
    Json(request): Json<QuoteRequest>,
) -> AppResult<Json<QuoteResponse>> {
    let venue = request.venue.as_deref().unwrap_or("pump_fun");

    match venue {
        "pump_fun" | "pumpfun" => {
            let quote = if request.is_buy {
                state
                    .pump_fun_venue
                    .compute_buy_quote(&mint, request.amount)
                    .await?
            } else {
                state
                    .pump_fun_venue
                    .compute_sell_quote(&mint, request.amount)
                    .await?
            };
            Ok(Json(QuoteResponse::PumpFun(quote)))
        }
        "moonshot" => {
            let quote = if request.is_buy {
                state
                    .moonshot_venue
                    .compute_buy_quote(&mint, request.amount)
                    .await?
            } else {
                state
                    .moonshot_venue
                    .compute_sell_quote(&mint, request.amount)
                    .await?
            };
            Ok(Json(QuoteResponse::Moonshot(quote)))
        }
        _ => Err(AppError::BadRequest(format!("Unknown venue: {}", venue))),
    }
}

pub async fn get_curve_parameters(
    State(state): State<AppState>,
    Path(mint): Path<String>,
) -> AppResult<Json<CurveParameters>> {
    let params = state.moonshot_venue.get_curve_parameters(&mint).await?;
    Ok(Json(params))
}

#[derive(Debug, Deserialize)]
pub struct ListTokensQuery {
    pub venue: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct CurveToken {
    pub mint: String,
    pub name: String,
    pub symbol: String,
    pub market_cap: f64,
    pub volume_24h: f64,
    pub progress_percent: f64,
    pub is_graduated: bool,
    pub venue: String,
}

#[derive(Debug, Serialize)]
pub struct ListTokensResponse {
    pub tokens: Vec<CurveToken>,
    pub total: usize,
}

pub async fn list_curve_tokens(
    State(state): State<AppState>,
    Query(query): Query<ListTokensQuery>,
) -> AppResult<Json<ListTokensResponse>> {
    let limit = query.limit.unwrap_or(50);
    let venue = query.venue.as_deref();

    let mut all_tokens = Vec::new();

    if venue.is_none() || venue == Some("pump_fun") || venue == Some("pumpfun") {
        let pump_tokens = state.pump_fun_venue.get_new_tokens(limit).await?;
        for token in pump_tokens {
            let progress = (token.market_cap / token.graduation_threshold.unwrap_or(69000.0)) * 100.0;
            all_tokens.push(CurveToken {
                mint: token.mint,
                name: token.name,
                symbol: token.symbol,
                market_cap: token.market_cap,
                volume_24h: token.volume_24h,
                progress_percent: progress.min(100.0),
                is_graduated: token.bonding_curve_complete,
                venue: "pump_fun".to_string(),
            });
        }
    }

    if venue.is_none() || venue == Some("moonshot") {
        let moonshot_tokens = state.moonshot_venue.get_new_tokens(limit).await?;
        for token in moonshot_tokens {
            let threshold = token.graduation_market_cap.unwrap_or(500_000.0);
            let progress = (token.market_cap_usd / threshold) * 100.0;
            all_tokens.push(CurveToken {
                mint: token.mint,
                name: token.name,
                symbol: token.symbol,
                market_cap: token.market_cap_usd,
                volume_24h: token.volume_24h_usd,
                progress_percent: progress.min(100.0),
                is_graduated: token.is_graduated,
                venue: "moonshot".to_string(),
            });
        }
    }

    let total = all_tokens.len();

    Ok(Json(ListTokensResponse {
        tokens: all_tokens,
        total,
    }))
}

#[derive(Debug, Serialize)]
pub struct CrossVenueArbitrageOpportunity {
    pub token_mint: String,
    pub token_name: String,
    pub token_symbol: String,
    pub curve_venue: String,
    pub curve_price: f64,
    pub dex_venue: String,
    pub dex_price: f64,
    pub price_diff_percent: f64,
    pub estimated_profit_bps: i32,
    pub direction: String,
    pub is_graduated: bool,
}

#[derive(Debug, Serialize)]
pub struct CrossVenueArbitrageResponse {
    pub opportunities: Vec<CrossVenueArbitrageOpportunity>,
    pub total: usize,
}

#[derive(Debug, Deserialize)]
pub struct CrossVenueArbQuery {
    pub min_diff_percent: Option<f64>,
    pub limit: Option<u32>,
}

pub async fn detect_cross_venue_arb(
    State(state): State<AppState>,
    Query(query): Query<CrossVenueArbQuery>,
) -> AppResult<Json<CrossVenueArbitrageResponse>> {
    let min_diff = query.min_diff_percent.unwrap_or(1.0);
    let limit = query.limit.unwrap_or(20) as usize;

    let mut opportunities = Vec::new();

    let pump_tokens = state.pump_fun_venue.get_new_tokens(50).await?;

    for token in pump_tokens {
        if !token.bonding_curve_complete {
            continue;
        }

        if let Some(raydium_pool) = &token.raydium_pool {
            let curve_price = token.market_cap / token.total_supply;

            if let Ok(jupiter_quote) = state
                .jupiter_venue
                .get_quote(&crate::venues::QuoteParams {
                    input_mint: "So11111111111111111111111111111111111111112".to_string(),
                    output_mint: token.mint.clone(),
                    amount_lamports: 1_000_000_000,
                    slippage_bps: 50,
                })
                .await
            {
                let dex_price = 1_000_000_000.0 / jupiter_quote.output_amount as f64;
                let price_diff = ((dex_price - curve_price) / curve_price * 100.0).abs();

                if price_diff >= min_diff {
                    let direction = if dex_price > curve_price {
                        "buy_curve_sell_dex"
                    } else {
                        "buy_dex_sell_curve"
                    };

                    opportunities.push(CrossVenueArbitrageOpportunity {
                        token_mint: token.mint.clone(),
                        token_name: token.name.clone(),
                        token_symbol: token.symbol.clone(),
                        curve_venue: "pump_fun".to_string(),
                        curve_price,
                        dex_venue: "jupiter".to_string(),
                        dex_price,
                        price_diff_percent: price_diff,
                        estimated_profit_bps: (price_diff * 100.0) as i32,
                        direction: direction.to_string(),
                        is_graduated: token.bonding_curve_complete,
                    });
                }
            }
        }
    }

    opportunities.sort_by(|a, b| {
        b.price_diff_percent
            .partial_cmp(&a.price_diff_percent)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    opportunities.truncate(limit);
    let total = opportunities.len();

    Ok(Json(CrossVenueArbitrageResponse {
        opportunities,
        total,
    }))
}

#[derive(Debug, Serialize)]
pub struct CurveTokenInfo {
    pub mint: String,
    pub name: String,
    pub symbol: String,
    pub venue: String,
    pub creator: String,
    pub graduation_progress: f64,
    pub current_price_sol: f64,
    pub market_cap_sol: f64,
    pub volume_24h_sol: f64,
    pub holder_count: u32,
    pub is_graduated: bool,
    pub raydium_pool: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct GraduationCandidate {
    pub token: CurveTokenInfo,
    pub graduation_eta_minutes: Option<u64>,
    pub momentum_score: u32,
    pub risk_score: u32,
}

#[derive(Debug, Serialize)]
pub struct GraduationCandidatesResponse {
    pub candidates: Vec<GraduationCandidate>,
    pub total: usize,
}

#[derive(Debug, Deserialize)]
pub struct GraduationCandidatesQuery {
    pub min_progress: Option<f64>,
    pub max_progress: Option<f64>,
    pub venue: Option<String>,
    pub limit: Option<u32>,
}

pub async fn list_graduation_candidates(
    State(state): State<AppState>,
    Query(query): Query<GraduationCandidatesQuery>,
) -> AppResult<Json<GraduationCandidatesResponse>> {
    let min_progress = query.min_progress.unwrap_or(50.0);
    let max_progress = query.max_progress.unwrap_or(95.0);
    let limit = query.limit.unwrap_or(20);
    let venue = query.venue.as_deref();

    let mut candidates = Vec::new();
    let now = chrono::Utc::now();

    if venue.is_none() || venue == Some("pump_fun") || venue == Some("pumpfun") {
        let tokens = state.pump_fun_venue.get_new_tokens(100).await?;

        for token in tokens {
            if token.bonding_curve_complete {
                continue;
            }

            let threshold = token.graduation_threshold.unwrap_or(69000.0);
            let progress = (token.market_cap / threshold) * 100.0;

            if progress >= min_progress && progress <= max_progress {
                let velocity = token.volume_24h / 24.0;
                let remaining_mc = threshold - token.market_cap;
                let eta_minutes = if velocity > 0.0 {
                    Some((remaining_mc / velocity * 60.0) as u64)
                } else {
                    None
                };

                // Calculate price from market cap and total supply
                let price_sol = if token.total_supply > 0.0 {
                    token.market_cap / token.total_supply / 200.0 // Approximate USD to SOL
                } else {
                    0.0
                };

                // Estimate holder count from market cap and volume (heuristic)
                let estimated_holders = estimate_holder_count(token.market_cap, token.volume_24h);

                // Calculate scores using available data
                let momentum_score = calculate_momentum_score(progress, velocity, estimated_holders);
                let risk_score = calculate_risk_score_from_volume(token.volume_24h, token.market_cap, estimated_holders);

                // Convert timestamp to RFC3339
                let created_at = chrono::DateTime::from_timestamp(token.created_timestamp, 0)
                    .map(|t| t.to_rfc3339())
                    .unwrap_or_else(|| now.to_rfc3339());

                candidates.push(GraduationCandidate {
                    token: CurveTokenInfo {
                        mint: token.mint,
                        name: token.name,
                        symbol: token.symbol,
                        venue: "pump_fun".to_string(),
                        creator: token.creator,
                        graduation_progress: progress,
                        current_price_sol: price_sol,
                        market_cap_sol: token.market_cap / 200.0,
                        volume_24h_sol: token.volume_24h / 200.0,
                        holder_count: estimated_holders,
                        is_graduated: false,
                        raydium_pool: token.raydium_pool,
                        created_at,
                        updated_at: now.to_rfc3339(),
                    },
                    graduation_eta_minutes: eta_minutes,
                    momentum_score,
                    risk_score,
                });
            }
        }
    }

    if venue.is_none() || venue == Some("moonshot") {
        let tokens = state.moonshot_venue.get_new_tokens(100).await?;

        for token in tokens {
            if token.is_graduated {
                continue;
            }

            let threshold = token.graduation_market_cap.unwrap_or(500_000.0);
            let progress = (token.market_cap_usd / threshold) * 100.0;

            if progress >= min_progress && progress <= max_progress {
                let velocity = token.volume_24h_usd / 24.0;
                let remaining_mc = threshold - token.market_cap_usd;
                let eta_minutes = if velocity > 0.0 {
                    Some((remaining_mc / velocity * 60.0) as u64)
                } else {
                    None
                };

                let estimated_holders = estimate_holder_count(token.market_cap_usd, token.volume_24h_usd);
                let momentum_score = calculate_momentum_score(progress, velocity, estimated_holders);
                let risk_score = calculate_risk_score_from_volume(token.volume_24h_usd, token.market_cap_usd, estimated_holders);

                let created_at = chrono::DateTime::from_timestamp(token.created_at, 0)
                    .map(|t| t.to_rfc3339())
                    .unwrap_or_else(|| now.to_rfc3339());

                candidates.push(GraduationCandidate {
                    token: CurveTokenInfo {
                        mint: token.mint,
                        name: token.name,
                        symbol: token.symbol,
                        venue: "moonshot".to_string(),
                        creator: token.creator,
                        graduation_progress: progress,
                        current_price_sol: token.price_sol,
                        market_cap_sol: token.market_cap_usd / 200.0,
                        volume_24h_sol: token.volume_24h_usd / 200.0,
                        holder_count: estimated_holders,
                        is_graduated: false,
                        raydium_pool: token.dex_pool_address,
                        created_at,
                        updated_at: now.to_rfc3339(),
                    },
                    graduation_eta_minutes: eta_minutes,
                    momentum_score,
                    risk_score,
                });
            }
        }
    }

    candidates.sort_by(|a, b| {
        b.token.graduation_progress
            .partial_cmp(&a.token.graduation_progress)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    candidates.truncate(limit as usize);
    let total = candidates.len();

    Ok(Json(GraduationCandidatesResponse { candidates, total }))
}

fn estimate_holder_count(market_cap: f64, volume_24h: f64) -> u32 {
    // Heuristic: tokens with higher market cap and volume tend to have more holders
    // This is a rough estimate until we integrate holder data APIs
    let mc_factor = (market_cap / 1000.0).sqrt().min(500.0);
    let vol_factor = (volume_24h / 100.0).sqrt().min(200.0);
    (mc_factor + vol_factor).max(10.0) as u32
}

fn calculate_momentum_score(progress: f64, volume_velocity: f64, holder_count: u32) -> u32 {
    let mut score: f64 = 0.0;

    // Progress contributes up to 40 points (higher progress = higher momentum)
    score += (progress / 100.0) * 40.0;

    // Volume velocity contributes up to 35 points
    let velocity_factor = (volume_velocity / 1000.0).min(1.0);
    score += velocity_factor * 35.0;

    // Holder count contributes up to 25 points
    let holder_factor = (holder_count as f64 / 500.0).min(1.0);
    score += holder_factor * 25.0;

    score.round().min(100.0) as u32
}

fn calculate_risk_score_from_volume(volume_24h: f64, market_cap: f64, holder_count: u32) -> u32 {
    let mut risk: f64 = 20.0; // Base risk for bonding curves

    // Low volume relative to market cap = higher risk
    let volume_ratio = if market_cap > 0.0 { volume_24h / market_cap } else { 0.0 };
    if volume_ratio < 0.1 {
        risk += 20.0 * (1.0 - volume_ratio / 0.1);
    }

    // Low holder count = higher risk
    if holder_count < 50 {
        risk += 30.0 * (1.0 - (holder_count as f64 / 50.0));
    }

    // Very new tokens (low market cap) have higher risk
    if market_cap < 10000.0 {
        risk += 15.0 * (1.0 - market_cap / 10000.0);
    }

    risk.min(100.0).round() as u32
}

#[derive(Debug, Serialize)]
pub struct VenueHealthResponse {
    pub pump_fun: VenueHealth,
    pub moonshot: VenueHealth,
}

#[derive(Debug, Serialize)]
pub struct VenueHealth {
    pub name: String,
    pub is_healthy: bool,
    pub last_check: String,
}

pub async fn get_venues_health(
    State(state): State<AppState>,
) -> AppResult<Json<VenueHealthResponse>> {
    use crate::venues::MevVenue;

    let pump_healthy = state.pump_fun_venue.is_healthy().await;
    let moonshot_healthy = state.moonshot_venue.is_healthy().await;
    let now = chrono::Utc::now().to_rfc3339();

    Ok(Json(VenueHealthResponse {
        pump_fun: VenueHealth {
            name: "pump.fun".to_string(),
            is_healthy: pump_healthy,
            last_check: now.clone(),
        },
        moonshot: VenueHealth {
            name: "moonshot".to_string(),
            is_healthy: moonshot_healthy,
            last_check: now,
        },
    }))
}

#[derive(Debug, Serialize)]
pub struct OnChainStateResponse {
    pub mint: String,
    pub bonding_curve_address: String,
    pub associated_bonding_curve: String,
    pub virtual_sol_reserves: u64,
    pub virtual_token_reserves: u64,
    pub real_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub graduation_progress_percent: f64,
    pub is_complete: bool,
    pub current_price_sol: f64,
    pub market_cap_sol: f64,
}

pub async fn get_on_chain_state(
    State(state): State<AppState>,
    Path(mint): Path<String>,
) -> AppResult<Json<OnChainStateResponse>> {
    let curve_state = state.curve_builder.get_curve_state(&mint).await?;

    let graduation_progress = curve_state.to_params().graduation_progress();
    let price = curve_state.virtual_sol_reserves as f64 / curve_state.virtual_token_reserves as f64;
    let market_cap = price * curve_state.token_total_supply as f64;

    Ok(Json(OnChainStateResponse {
        mint: curve_state.mint,
        bonding_curve_address: curve_state.bonding_curve_address,
        associated_bonding_curve: curve_state.associated_bonding_curve,
        virtual_sol_reserves: curve_state.virtual_sol_reserves,
        virtual_token_reserves: curve_state.virtual_token_reserves,
        real_sol_reserves: curve_state.real_sol_reserves,
        real_token_reserves: curve_state.real_token_reserves,
        graduation_progress_percent: graduation_progress,
        is_complete: curve_state.is_complete,
        current_price_sol: price,
        market_cap_sol: market_cap / 1_000_000_000.0,
    }))
}

#[derive(Debug, Deserialize)]
pub struct CurveBuyRequest {
    pub sol_amount: f64,
    pub slippage_bps: Option<u16>,
    pub user_wallet: String,
    pub simulate: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct CurveBuyResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_base64: Option<String>,
    pub expected_tokens_out: u64,
    pub min_tokens_out: u64,
    pub price_impact_percent: f64,
    pub fee_lamports: u64,
    pub simulated: bool,
}

pub async fn buy_curve_token(
    State(state): State<AppState>,
    Path(mint): Path<String>,
    Json(request): Json<CurveBuyRequest>,
) -> AppResult<Json<CurveBuyResponse>> {
    let sol_lamports = (request.sol_amount * 1_000_000_000.0) as u64;
    let slippage = request.slippage_bps.unwrap_or(100);
    let simulate_only = request.simulate.unwrap_or(false);

    let params = CurveBuyParams {
        mint: mint.clone(),
        sol_amount_lamports: sol_lamports,
        slippage_bps: slippage,
        user_wallet: request.user_wallet.clone(),
    };

    if simulate_only {
        let simulation = state.curve_builder.simulate_buy(&params).await?;
        return Ok(Json(CurveBuyResponse {
            transaction_base64: None,
            expected_tokens_out: simulation.output_amount,
            min_tokens_out: simulation.min_output,
            price_impact_percent: simulation.price_impact_percent,
            fee_lamports: simulation.fee_lamports,
            simulated: true,
        }));
    }

    let result = state.curve_builder.build_pump_fun_buy(&params).await?;

    Ok(Json(CurveBuyResponse {
        transaction_base64: Some(result.transaction_base64),
        expected_tokens_out: result.expected_tokens_out.unwrap_or(0),
        min_tokens_out: result.min_tokens_out.unwrap_or(0),
        price_impact_percent: result.price_impact_percent,
        fee_lamports: result.fee_lamports,
        simulated: false,
    }))
}

#[derive(Debug, Deserialize)]
pub struct CurveSellRequest {
    pub token_amount: u64,
    pub slippage_bps: Option<u16>,
    pub user_wallet: String,
    pub simulate: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct CurveSellResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_base64: Option<String>,
    pub expected_sol_out: u64,
    pub min_sol_out: u64,
    pub price_impact_percent: f64,
    pub fee_lamports: u64,
    pub simulated: bool,
}

pub async fn sell_curve_token(
    State(state): State<AppState>,
    Path(mint): Path<String>,
    Json(request): Json<CurveSellRequest>,
) -> AppResult<Json<CurveSellResponse>> {
    let slippage = request.slippage_bps.unwrap_or(100);
    let simulate_only = request.simulate.unwrap_or(false);

    let params = CurveSellParams {
        mint: mint.clone(),
        token_amount: request.token_amount,
        slippage_bps: slippage,
        user_wallet: request.user_wallet.clone(),
    };

    if simulate_only {
        let simulation = state.curve_builder.simulate_sell(&params).await?;
        return Ok(Json(CurveSellResponse {
            transaction_base64: None,
            expected_sol_out: simulation.output_amount,
            min_sol_out: simulation.min_output,
            price_impact_percent: simulation.price_impact_percent,
            fee_lamports: simulation.fee_lamports,
            simulated: true,
        }));
    }

    let result = state.curve_builder.build_pump_fun_sell(&params).await?;

    Ok(Json(CurveSellResponse {
        transaction_base64: Some(result.transaction_base64),
        expected_sol_out: result.expected_sol_out.unwrap_or(0),
        min_sol_out: result.min_sol_out.unwrap_or(0),
        price_impact_percent: result.price_impact_percent,
        fee_lamports: result.fee_lamports,
        simulated: false,
    }))
}

#[derive(Debug, Serialize)]
pub struct PostGraduationPoolResponse {
    pub mint: String,
    pub graduation_status: String,
    pub raydium_pool: Option<RaydiumPoolInfo>,
    pub graduation_progress: f64,
}

pub async fn get_post_graduation_pool(
    State(state): State<AppState>,
    Path(mint): Path<String>,
) -> AppResult<Json<PostGraduationPoolResponse>> {
    let status = state.on_chain_fetcher.is_token_graduated(&mint).await?;

    let (status_str, raydium_pool, progress) = match &status {
        GraduationStatus::PreGraduation { progress } => {
            ("pre_graduation".to_string(), None, *progress)
        }
        GraduationStatus::NearGraduation { progress } => {
            ("near_graduation".to_string(), None, *progress)
        }
        GraduationStatus::Graduating => {
            ("graduating".to_string(), None, 99.0)
        }
        GraduationStatus::Graduated { raydium_pool, .. } => {
            let pool_info = if raydium_pool.is_some() {
                state.on_chain_fetcher.find_raydium_pool(&mint).await?
            } else {
                None
            };
            ("graduated".to_string(), pool_info, 100.0)
        }
        GraduationStatus::Failed { reason } => {
            return Err(AppError::Internal(format!("Graduation failed: {}", reason)));
        }
    };

    Ok(Json(PostGraduationPoolResponse {
        mint,
        graduation_status: status_str,
        raydium_pool,
        graduation_progress: progress,
    }))
}

#[derive(Debug, Serialize)]
pub struct CurveAddressesResponse {
    pub mint: String,
    pub bonding_curve: String,
    pub associated_bonding_curve: String,
}

pub async fn get_curve_addresses(
    Path(mint): Path<String>,
) -> AppResult<Json<CurveAddressesResponse>> {
    let (bonding_curve, associated) = derive_pump_fun_bonding_curve(&mint)?;

    Ok(Json(CurveAddressesResponse {
        mint,
        bonding_curve,
        associated_bonding_curve: associated,
    }))
}

// ============================================================================
// Metrics & Scoring Endpoints
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct MetricsQuery {
    pub venue: Option<String>,
    pub max_age_seconds: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct DetailedMetricsResponse {
    pub mint: String,
    pub venue: String,
    pub volume_1h: f64,
    pub volume_24h: f64,
    pub volume_velocity: f64,
    pub volume_acceleration: f64,
    pub trade_count_1h: u32,
    pub trade_count_24h: u32,
    pub unique_buyers_1h: u32,
    pub unique_buyers_24h: u32,
    pub holder_count: u32,
    pub holder_growth_1h: i32,
    pub holder_growth_24h: i32,
    pub top_10_concentration: f64,
    pub top_20_concentration: f64,
    pub creator_holdings_percent: f64,
    pub price_momentum_1h: f64,
    pub price_momentum_24h: f64,
    pub buy_sell_ratio_1h: f64,
    pub avg_trade_size_sol: f64,
    pub graduation_progress: f64,
    pub market_cap_sol: f64,
    pub liquidity_depth_sol: f64,
    pub holder_quality_score: f64,
    pub activity_score: f64,
    pub momentum_score: f64,
    pub last_updated: String,
}

impl From<DetailedCurveMetrics> for DetailedMetricsResponse {
    fn from(m: DetailedCurveMetrics) -> Self {
        let volume_acceleration = m.volume_acceleration();
        let holder_quality_score = m.holder_quality_score();
        let activity_score = m.activity_score();
        let momentum_score = m.momentum_score();
        Self {
            mint: m.mint,
            venue: m.venue,
            volume_1h: m.volume_1h,
            volume_24h: m.volume_24h,
            volume_velocity: m.volume_velocity,
            volume_acceleration,
            trade_count_1h: m.trade_count_1h,
            trade_count_24h: m.trade_count_24h,
            unique_buyers_1h: m.unique_buyers_1h,
            unique_buyers_24h: m.unique_buyers_24h,
            holder_count: m.holder_count,
            holder_growth_1h: m.holder_growth_1h,
            holder_growth_24h: m.holder_growth_24h,
            top_10_concentration: m.top_10_concentration,
            top_20_concentration: m.top_20_concentration,
            creator_holdings_percent: m.creator_holdings_percent,
            price_momentum_1h: m.price_momentum_1h,
            price_momentum_24h: m.price_momentum_24h,
            buy_sell_ratio_1h: m.buy_sell_ratio_1h,
            avg_trade_size_sol: m.avg_trade_size_sol,
            graduation_progress: m.graduation_progress,
            market_cap_sol: m.market_cap_sol,
            liquidity_depth_sol: m.liquidity_depth_sol,
            holder_quality_score,
            activity_score,
            momentum_score,
            last_updated: m.last_updated.to_rfc3339(),
        }
    }
}

pub async fn get_curve_metrics(
    State(state): State<AppState>,
    Path(mint): Path<String>,
    Query(query): Query<MetricsQuery>,
) -> AppResult<Json<DetailedMetricsResponse>> {
    let venue = query.venue.as_deref().unwrap_or("pump_fun");
    let max_age = query.max_age_seconds.unwrap_or(300);

    if let Some(cached) = state.metrics_collector.get_cached_metrics(&mint).await {
        if !cached.is_stale(max_age) {
            return Ok(Json(cached.into()));
        }
    }

    let mut metrics = match venue {
        "pump_fun" | "pumpfun" => {
            let token_result = state.pump_fun_venue.get_token_info(&mint).await;
            let holder_result = state.pump_fun_venue.get_holder_stats(&mint).await;

            match (token_result, holder_result) {
                (Ok(token), Ok(holder_stats)) => {
                    state.metrics_collector.populate_from_pump_fun(&mint, &token, &holder_stats)
                }
                _ => {
                    state.metrics_collector.get_or_calculate_metrics(&mint, venue, max_age).await?
                }
            }
        }
        "moonshot" => {
            let token_result = state.moonshot_venue.get_token_info(&mint).await;
            let holder_result = state.moonshot_venue.get_holder_stats(&mint).await;

            match (token_result, holder_result) {
                (Ok(token), Ok(holder_stats)) => {
                    state.metrics_collector.populate_from_moonshot(
                        &mint,
                        &token,
                        holder_stats.total_holders,
                        holder_stats.top_10_concentration,
                    )
                }
                _ => {
                    state.metrics_collector.get_or_calculate_metrics(&mint, venue, max_age).await?
                }
            }
        }
        _ => {
            state.metrics_collector.get_or_calculate_metrics(&mint, venue, max_age).await?
        }
    };

    if let Ok(on_chain) = state.on_chain_fetcher.get_bonding_curve_state(&mint).await {
        metrics.graduation_progress = on_chain.graduation_progress();
        metrics.market_cap_sol = on_chain.market_cap_sol();
        metrics.liquidity_depth_sol = on_chain.real_sol_reserves as f64 / 1e9;
    }

    state.metrics_collector.cache_metrics(&mint, metrics.clone()).await;

    Ok(Json(metrics.into()))
}

#[derive(Debug, Serialize)]
pub struct HolderAnalysisResponse {
    pub mint: String,
    pub total_holders: u32,
    pub total_supply: u64,
    pub circulating_supply: u64,
    pub top_10_holders: Vec<TopHolderInfo>,
    pub top_10_concentration: f64,
    pub top_20_concentration: f64,
    pub top_50_concentration: f64,
    pub creator_address: Option<String>,
    pub creator_holdings_percent: f64,
    pub gini_coefficient: f64,
    pub unique_wallets_24h: u32,
    pub new_holders_24h: i32,
    pub wash_trade_likelihood: f64,
    pub is_healthy: bool,
    pub health_score: f64,
    pub analyzed_at: String,
}

#[derive(Debug, Serialize)]
pub struct TopHolderInfo {
    pub address: String,
    pub balance: u64,
    pub balance_percent: f64,
    pub is_creator: bool,
    pub is_suspicious: bool,
}

impl From<HolderDistribution> for HolderAnalysisResponse {
    fn from(h: HolderDistribution) -> Self {
        let is_healthy = h.is_healthy();
        let health_score = h.health_score();
        Self {
            mint: h.mint,
            total_holders: h.total_holders,
            total_supply: h.total_supply,
            circulating_supply: h.circulating_supply,
            top_10_holders: h.top_10_holders.into_iter().map(|holder| TopHolderInfo {
                address: holder.address,
                balance: holder.balance,
                balance_percent: holder.balance_percent,
                is_creator: holder.is_creator,
                is_suspicious: holder.is_suspicious,
            }).collect(),
            top_10_concentration: h.top_10_concentration,
            top_20_concentration: h.top_20_concentration,
            top_50_concentration: h.top_50_concentration,
            creator_address: h.creator_address,
            creator_holdings_percent: h.creator_holdings_percent,
            gini_coefficient: h.gini_coefficient,
            unique_wallets_24h: h.unique_wallets_24h,
            new_holders_24h: h.new_holders_24h,
            wash_trade_likelihood: h.wash_trade_likelihood,
            is_healthy,
            health_score,
            analyzed_at: h.analyzed_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct HolderAnalysisQuery {
    pub creator: Option<String>,
    pub max_age_seconds: Option<i64>,
    pub venue: Option<String>,
}

pub async fn get_holder_analysis(
    State(state): State<AppState>,
    Path(mint): Path<String>,
    Query(query): Query<HolderAnalysisQuery>,
) -> AppResult<Json<HolderAnalysisResponse>> {
    let max_age = query.max_age_seconds.unwrap_or(600);
    let venue = query.venue.as_deref().unwrap_or("pump_fun");

    let venue_holder_count = match venue {
        "pump_fun" | "pumpfun" => {
            state.pump_fun_venue.get_holder_stats(&mint).await.ok().map(|s| s.total_holders)
        }
        "moonshot" => {
            state.moonshot_venue.get_holder_stats(&mint).await.ok().map(|s| s.total_holders)
        }
        _ => None,
    };

    let distribution = state
        .holder_analyzer
        .get_or_analyze_with_count(&mint, query.creator.as_deref(), max_age, venue_holder_count)
        .await?;

    Ok(Json(distribution.into()))
}

#[derive(Debug, Deserialize)]
pub struct ScoreQuery {
    pub venue: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OpportunityScoreResponse {
    pub mint: String,
    pub venue: String,
    pub overall_score: f64,
    pub graduation_factor: f64,
    pub volume_factor: f64,
    pub holder_factor: f64,
    pub momentum_factor: f64,
    pub risk_penalty: f64,
    pub recommendation: String,
    pub is_actionable: bool,
    pub risk_warnings: Vec<String>,
    pub positive_signals: Vec<String>,
}

impl From<OpportunityScore> for OpportunityScoreResponse {
    fn from(s: OpportunityScore) -> Self {
        let recommendation = s.recommendation.as_str().to_string();
        let is_actionable = s.is_actionable();
        Self {
            mint: s.mint,
            venue: s.venue,
            overall_score: s.overall,
            graduation_factor: s.graduation_factor,
            volume_factor: s.volume_factor,
            holder_factor: s.holder_factor,
            momentum_factor: s.momentum_factor,
            risk_penalty: s.risk_penalty,
            recommendation,
            is_actionable,
            risk_warnings: s.risk_warnings,
            positive_signals: s.positive_signals,
        }
    }
}

pub async fn get_opportunity_score(
    State(state): State<AppState>,
    Path(mint): Path<String>,
    Query(query): Query<ScoreQuery>,
) -> AppResult<Json<OpportunityScoreResponse>> {
    let venue = query.venue.as_deref().unwrap_or("pump_fun");

    let score = state
        .curve_scorer
        .score_opportunity(&mint, venue)
        .await?;

    Ok(Json(score.into()))
}

#[derive(Debug, Deserialize)]
pub struct TopOpportunitiesQuery {
    pub venue: Option<String>,
    pub min_progress: Option<f64>,
    pub max_progress: Option<f64>,
    pub min_score: Option<f64>,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct TopOpportunitiesResponse {
    pub opportunities: Vec<RankedOpportunity>,
    pub total: usize,
    pub scoring_weights: ScoringWeightsInfo,
}

#[derive(Debug, Serialize)]
pub struct RankedOpportunity {
    pub rank: usize,
    pub mint: String,
    pub name: String,
    pub symbol: String,
    pub venue: String,
    pub overall_score: f64,
    pub graduation_progress: f64,
    pub market_cap_sol: f64,
    pub volume_1h: f64,
    pub recommendation: String,
    pub top_signals: Vec<String>,
    pub top_warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ScoringWeightsInfo {
    pub graduation: f64,
    pub volume: f64,
    pub holders: f64,
    pub momentum: f64,
    pub risk: f64,
}

pub async fn get_top_opportunities(
    State(state): State<AppState>,
    Query(query): Query<TopOpportunitiesQuery>,
) -> AppResult<Json<TopOpportunitiesResponse>> {
    let min_progress = query.min_progress.unwrap_or(70.0);
    let max_progress = query.max_progress.unwrap_or(99.0);
    let min_score = query.min_score.unwrap_or(0.0);
    let limit = query.limit.unwrap_or(20);
    let venue_filter = query.venue.as_deref();

    let mut candidates: Vec<(String, String, String, String)> = Vec::new();

    if venue_filter.is_none() || venue_filter == Some("pump_fun") || venue_filter == Some("pumpfun") {
        let tokens = state.pump_fun_venue.get_new_tokens(100).await?;
        for token in tokens {
            if token.bonding_curve_complete {
                continue;
            }
            let threshold = token.graduation_threshold.unwrap_or(69000.0);
            let progress = (token.market_cap / threshold) * 100.0;
            if progress >= min_progress && progress <= max_progress {
                candidates.push((token.mint, token.name, token.symbol, "pump_fun".to_string()));
            }
        }
    }

    if venue_filter.is_none() || venue_filter == Some("moonshot") {
        let tokens = state.moonshot_venue.get_new_tokens(100).await?;
        for token in tokens {
            if token.is_graduated {
                continue;
            }
            let threshold = token.graduation_market_cap.unwrap_or(500_000.0);
            let progress = (token.market_cap_usd / threshold) * 100.0;
            if progress >= min_progress && progress <= max_progress {
                candidates.push((token.mint, token.name, token.symbol, "moonshot".to_string()));
            }
        }
    }

    let candidate_pairs: Vec<(String, String)> = candidates
        .iter()
        .map(|(mint, _, _, venue)| (mint.clone(), venue.clone()))
        .collect();

    let scores = state
        .curve_scorer
        .rank_opportunities(&candidate_pairs, limit * 2)
        .await;

    let mut opportunities: Vec<RankedOpportunity> = Vec::new();

    for (idx, score) in scores.into_iter().enumerate() {
        if score.overall < min_score {
            continue;
        }

        let (name, symbol) = candidates
            .iter()
            .find(|(m, _, _, _)| m == &score.mint)
            .map(|(_, n, s, _)| (n.clone(), s.clone()))
            .unwrap_or(("Unknown".to_string(), "???".to_string()));

        let metrics = state
            .metrics_collector
            .get_cached_metrics(&score.mint)
            .await;

        let (graduation_progress, market_cap_sol, volume_1h) = metrics
            .map(|m| (m.graduation_progress, m.market_cap_sol, m.volume_1h))
            .unwrap_or((0.0, 0.0, 0.0));

        opportunities.push(RankedOpportunity {
            rank: idx + 1,
            mint: score.mint,
            name,
            symbol,
            venue: score.venue,
            overall_score: score.overall,
            graduation_progress,
            market_cap_sol,
            volume_1h,
            recommendation: score.recommendation.as_str().to_string(),
            top_signals: score.positive_signals.into_iter().take(3).collect(),
            top_warnings: score.risk_warnings.into_iter().take(3).collect(),
        });

        if opportunities.len() >= limit {
            break;
        }
    }

    let weights = state.curve_scorer.get_weights();

    Ok(Json(TopOpportunitiesResponse {
        total: opportunities.len(),
        opportunities,
        scoring_weights: ScoringWeightsInfo {
            graduation: weights.graduation,
            volume: weights.volume,
            holders: weights.holders,
            momentum: weights.momentum,
            risk: weights.risk,
        },
    }))
}

#[derive(Debug, Serialize)]
pub struct ScoringConfigResponse {
    pub weights: ScoringWeightsInfo,
    pub thresholds: ScoringThresholdsInfo,
}

#[derive(Debug, Serialize)]
pub struct ScoringThresholdsInfo {
    pub min_graduation_progress: f64,
    pub max_graduation_progress: f64,
    pub min_volume_1h_sol: f64,
    pub min_holder_count: u32,
    pub max_top_10_concentration: f64,
    pub max_creator_holdings: f64,
    pub max_wash_trade_likelihood: f64,
    pub min_unique_buyers_1h: u32,
}

pub async fn get_scoring_config(
    State(state): State<AppState>,
) -> AppResult<Json<ScoringConfigResponse>> {
    let weights = state.curve_scorer.get_weights();
    let thresholds = state.curve_scorer.get_thresholds();

    Ok(Json(ScoringConfigResponse {
        weights: ScoringWeightsInfo {
            graduation: weights.graduation,
            volume: weights.volume,
            holders: weights.holders,
            momentum: weights.momentum,
            risk: weights.risk,
        },
        thresholds: ScoringThresholdsInfo {
            min_graduation_progress: thresholds.min_graduation_progress,
            max_graduation_progress: thresholds.max_graduation_progress,
            min_volume_1h_sol: thresholds.min_volume_1h_sol,
            min_holder_count: thresholds.min_holder_count,
            max_top_10_concentration: thresholds.max_top_10_concentration,
            max_creator_holdings: thresholds.max_creator_holdings,
            max_wash_trade_likelihood: thresholds.max_wash_trade_likelihood,
            min_unique_buyers_1h: thresholds.min_unique_buyers_1h,
        },
    }))
}

const MARKET_DATA_CACHE_TTL_SECS: u64 = 30;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DexScreenerPair {
    #[serde(rename = "chainId")]
    pub chain_id: Option<String>,
    #[serde(rename = "dexId")]
    pub dex_id: Option<String>,
    #[serde(rename = "pairAddress")]
    pub pair_address: Option<String>,
    #[serde(rename = "baseToken")]
    pub base_token: Option<DexScreenerToken>,
    #[serde(rename = "quoteToken")]
    pub quote_token: Option<DexScreenerToken>,
    #[serde(rename = "priceNative")]
    pub price_native: Option<String>,
    #[serde(rename = "priceUsd")]
    pub price_usd: Option<String>,
    pub txns: Option<DexScreenerTxns>,
    pub volume: Option<DexScreenerVolume>,
    #[serde(rename = "priceChange")]
    pub price_change: Option<DexScreenerPriceChange>,
    pub liquidity: Option<DexScreenerLiquidity>,
    pub fdv: Option<f64>,
    #[serde(rename = "marketCap")]
    pub market_cap: Option<f64>,
    #[serde(rename = "pairCreatedAt")]
    pub pair_created_at: Option<i64>,
    pub info: Option<DexScreenerInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DexScreenerToken {
    pub address: Option<String>,
    pub name: Option<String>,
    pub symbol: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DexScreenerTxns {
    pub m5: Option<DexScreenerTxnPeriod>,
    pub h1: Option<DexScreenerTxnPeriod>,
    pub h6: Option<DexScreenerTxnPeriod>,
    pub h24: Option<DexScreenerTxnPeriod>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DexScreenerTxnPeriod {
    pub buys: Option<u64>,
    pub sells: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DexScreenerVolume {
    pub m5: Option<f64>,
    pub h1: Option<f64>,
    pub h6: Option<f64>,
    pub h24: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DexScreenerPriceChange {
    pub m5: Option<f64>,
    pub h1: Option<f64>,
    pub h6: Option<f64>,
    pub h24: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DexScreenerLiquidity {
    pub usd: Option<f64>,
    pub base: Option<f64>,
    pub quote: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DexScreenerInfo {
    #[serde(rename = "imageUrl")]
    pub image_url: Option<String>,
    pub websites: Option<Vec<DexScreenerWebsite>>,
    pub socials: Option<Vec<DexScreenerSocial>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DexScreenerWebsite {
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DexScreenerSocial {
    #[serde(rename = "type")]
    pub social_type: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DexScreenerResponse {
    pairs: Option<Vec<DexScreenerPair>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketDataResponse {
    pub mint: String,
    pub symbol: Option<String>,
    pub name: Option<String>,
    pub price_usd: f64,
    pub price_native: f64,
    pub market_cap: f64,
    pub fdv: f64,
    pub liquidity: MarketDataLiquidity,
    pub volume: MarketDataVolume,
    pub price_change: MarketDataPriceChange,
    pub txns: MarketDataTxns,
    pub pair_created_at: Option<i64>,
    pub dex_id: Option<String>,
    pub image_url: Option<String>,
    pub cached_at: i64,
    pub cache_ttl_secs: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketDataLiquidity {
    pub usd: f64,
    pub base: f64,
    pub quote: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketDataVolume {
    pub m5: f64,
    pub h1: f64,
    pub h6: f64,
    pub h24: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketDataPriceChange {
    pub m5: f64,
    pub h1: f64,
    pub h6: f64,
    pub h24: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketDataTxns {
    pub m5: MarketDataTxnPeriod,
    pub h1: MarketDataTxnPeriod,
    pub h6: MarketDataTxnPeriod,
    pub h24: MarketDataTxnPeriod,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketDataTxnPeriod {
    pub buys: u64,
    pub sells: u64,
}

struct CachedMarketData {
    data: MarketDataResponse,
    fetched_at: Instant,
}

lazy_static! {
    static ref MARKET_DATA_CACHE: RwLock<HashMap<String, CachedMarketData>> = RwLock::new(HashMap::new());
}

pub async fn get_market_data(
    Path(mint): Path<String>,
) -> AppResult<Json<MarketDataResponse>> {
    let cache_ttl = Duration::from_secs(MARKET_DATA_CACHE_TTL_SECS);

    {
        let cache = MARKET_DATA_CACHE.read().map_err(|_| {
            AppError::Internal("Cache lock poisoned".to_string())
        })?;

        if let Some(cached) = cache.get(&mint) {
            if cached.fetched_at.elapsed() < cache_ttl {
                tracing::debug!(mint = %mint, "Returning cached market data");
                return Ok(Json(cached.data.clone()));
            }
        }
    }

    tracing::info!(mint = %mint, "Fetching fresh market data from DexScreener");

    let client = reqwest::Client::new();
    let url = format!("https://api.dexscreener.com/latest/dex/tokens/{}", mint);

    let response = client
        .get(&url)
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| AppError::ExternalApi(format!("DexScreener request failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(AppError::ExternalApi(format!(
            "DexScreener returned error: {}",
            response.status()
        )));
    }

    let dex_response: DexScreenerResponse = response
        .json()
        .await
        .map_err(|e| AppError::ExternalApi(format!("Failed to parse DexScreener response: {}", e)))?;

    let pair = dex_response.pairs
        .and_then(|pairs| pairs.into_iter().next())
        .ok_or_else(|| AppError::NotFound(format!("No pair found for {}", mint)))?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let market_data = MarketDataResponse {
        mint: mint.clone(),
        symbol: pair.base_token.as_ref().and_then(|t| t.symbol.clone()),
        name: pair.base_token.as_ref().and_then(|t| t.name.clone()),
        price_usd: pair.price_usd.as_ref().and_then(|p| p.parse().ok()).unwrap_or(0.0),
        price_native: pair.price_native.as_ref().and_then(|p| p.parse().ok()).unwrap_or(0.0),
        market_cap: pair.market_cap.unwrap_or(0.0),
        fdv: pair.fdv.unwrap_or(0.0),
        liquidity: MarketDataLiquidity {
            usd: pair.liquidity.as_ref().and_then(|l| l.usd).unwrap_or(0.0),
            base: pair.liquidity.as_ref().and_then(|l| l.base).unwrap_or(0.0),
            quote: pair.liquidity.as_ref().and_then(|l| l.quote).unwrap_or(0.0),
        },
        volume: MarketDataVolume {
            m5: pair.volume.as_ref().and_then(|v| v.m5).unwrap_or(0.0),
            h1: pair.volume.as_ref().and_then(|v| v.h1).unwrap_or(0.0),
            h6: pair.volume.as_ref().and_then(|v| v.h6).unwrap_or(0.0),
            h24: pair.volume.as_ref().and_then(|v| v.h24).unwrap_or(0.0),
        },
        price_change: MarketDataPriceChange {
            m5: pair.price_change.as_ref().and_then(|p| p.m5).unwrap_or(0.0),
            h1: pair.price_change.as_ref().and_then(|p| p.h1).unwrap_or(0.0),
            h6: pair.price_change.as_ref().and_then(|p| p.h6).unwrap_or(0.0),
            h24: pair.price_change.as_ref().and_then(|p| p.h24).unwrap_or(0.0),
        },
        txns: MarketDataTxns {
            m5: MarketDataTxnPeriod {
                buys: pair.txns.as_ref().and_then(|t| t.m5.as_ref()).and_then(|p| p.buys).unwrap_or(0),
                sells: pair.txns.as_ref().and_then(|t| t.m5.as_ref()).and_then(|p| p.sells).unwrap_or(0),
            },
            h1: MarketDataTxnPeriod {
                buys: pair.txns.as_ref().and_then(|t| t.h1.as_ref()).and_then(|p| p.buys).unwrap_or(0),
                sells: pair.txns.as_ref().and_then(|t| t.h1.as_ref()).and_then(|p| p.sells).unwrap_or(0),
            },
            h6: MarketDataTxnPeriod {
                buys: pair.txns.as_ref().and_then(|t| t.h6.as_ref()).and_then(|p| p.buys).unwrap_or(0),
                sells: pair.txns.as_ref().and_then(|t| t.h6.as_ref()).and_then(|p| p.sells).unwrap_or(0),
            },
            h24: MarketDataTxnPeriod {
                buys: pair.txns.as_ref().and_then(|t| t.h24.as_ref()).and_then(|p| p.buys).unwrap_or(0),
                sells: pair.txns.as_ref().and_then(|t| t.h24.as_ref()).and_then(|p| p.sells).unwrap_or(0),
            },
        },
        pair_created_at: pair.pair_created_at,
        dex_id: pair.dex_id,
        image_url: pair.info.and_then(|i| i.image_url),
        cached_at: now,
        cache_ttl_secs: MARKET_DATA_CACHE_TTL_SECS,
    };

    {
        let mut cache = MARKET_DATA_CACHE.write().map_err(|_| {
            AppError::Internal("Cache lock poisoned".to_string())
        })?;

        cache.insert(mint.clone(), CachedMarketData {
            data: market_data.clone(),
            fetched_at: Instant::now(),
        });

        let expired_keys: Vec<String> = cache
            .iter()
            .filter(|(_, v)| v.fetched_at.elapsed() > Duration::from_secs(300))
            .map(|(k, _)| k.clone())
            .collect();

        for key in expired_keys {
            cache.remove(&key);
        }
    }

    Ok(Json(market_data))
}
