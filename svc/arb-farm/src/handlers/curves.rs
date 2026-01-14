use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::server::AppState;
use crate::venues::MevVenue;
use crate::venues::curves::{
    moonshot::{MoonshotGraduationProgress, MoonshotHolderStats, MoonshotQuote, CurveParameters},
    pump_fun::{GraduationProgress, HolderStats, PumpFunQuote},
};

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
pub struct GraduationCandidate {
    pub mint: String,
    pub name: String,
    pub symbol: String,
    pub venue: String,
    pub progress_percent: f64,
    pub market_cap: f64,
    pub graduation_threshold: f64,
    pub estimated_blocks_to_graduation: u64,
    pub volume_24h: f64,
    pub volume_velocity: f64,
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
                let estimated_blocks = if velocity > 0.0 {
                    ((remaining_mc / velocity) * 600.0) as u64
                } else {
                    u64::MAX
                };

                candidates.push(GraduationCandidate {
                    mint: token.mint,
                    name: token.name,
                    symbol: token.symbol,
                    venue: "pump_fun".to_string(),
                    progress_percent: progress,
                    market_cap: token.market_cap,
                    graduation_threshold: threshold,
                    estimated_blocks_to_graduation: estimated_blocks,
                    volume_24h: token.volume_24h,
                    volume_velocity: velocity,
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
                let estimated_blocks = if velocity > 0.0 {
                    ((remaining_mc / velocity) * 600.0) as u64
                } else {
                    u64::MAX
                };

                candidates.push(GraduationCandidate {
                    mint: token.mint,
                    name: token.name,
                    symbol: token.symbol,
                    venue: "moonshot".to_string(),
                    progress_percent: progress,
                    market_cap: token.market_cap_usd,
                    graduation_threshold: threshold,
                    estimated_blocks_to_graduation: estimated_blocks,
                    volume_24h: token.volume_24h_usd,
                    volume_velocity: velocity,
                });
            }
        }
    }

    candidates.sort_by(|a, b| {
        b.progress_percent
            .partial_cmp(&a.progress_percent)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    candidates.truncate(limit as usize);
    let total = candidates.len();

    Ok(Json(GraduationCandidatesResponse { candidates, total }))
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
