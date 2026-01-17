use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::agents::EngramHarvester;
use crate::models::{
    ArbEngram, AvoidanceSeverity, CreateEngramRequest, EngramQuery,
    EngramType, PatternMatch, PatternMatchRequest,
};
use crate::server::AppState;

lazy_static::lazy_static! {
    static ref ENGRAM_HARVESTER: std::sync::RwLock<Option<EngramHarvester>> = std::sync::RwLock::new(None);
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: u16,
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        let status = StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, Json(self)).into_response()
    }
}

fn json_error(status: StatusCode, message: &str) -> ErrorResponse {
    ErrorResponse {
        error: message.to_string(),
        code: status.as_u16(),
    }
}

pub fn init_harvester(harvester: EngramHarvester) {
    let mut guard = ENGRAM_HARVESTER.write().unwrap();
    *guard = Some(harvester);
}

fn get_harvester_clone() -> Result<EngramHarvester, ErrorResponse> {
    let guard = ENGRAM_HARVESTER
        .read()
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "Lock poisoned"))?;
    guard
        .as_ref()
        .cloned()
        .ok_or_else(|| json_error(StatusCode::SERVICE_UNAVAILABLE, "Harvester not initialized"))
}

#[derive(Debug, Serialize)]
pub struct EngramResponse {
    pub success: bool,
    pub engram: Option<ArbEngram>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EngramListResponse {
    pub success: bool,
    pub engrams: Vec<ArbEngram>,
    pub total: u64,
}

#[derive(Debug, Serialize)]
pub struct PatternMatchResponse {
    pub success: bool,
    pub matches: Vec<PatternMatch>,
    pub count: usize,
}

#[derive(Debug, Serialize)]
pub struct AvoidanceCheckResponse {
    pub should_avoid: bool,
    pub reason: Option<String>,
    pub category: Option<String>,
    pub severity: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAvoidanceRequest {
    pub entity_type: String,
    pub address: String,
    pub reason: String,
    pub category: String,
    pub severity: String,
}

#[derive(Debug, Deserialize)]
pub struct CreatePatternRequest {
    pub edge_type: String,
    pub venue_type: String,
    pub route_signature: String,
    pub success_rate: f64,
    pub avg_profit_bps: f64,
    pub sample_count: u32,
}

#[derive(Debug, Serialize)]
pub struct HarvesterStatsResponse {
    pub total_engrams: u64,
    pub engrams_by_type: std::collections::HashMap<String, u64>,
    pub patterns_matched: u64,
    pub avoidances_created: u64,
    pub last_harvest_at: Option<String>,
}

pub async fn create_engram(
    State(_state): State<AppState>,
    Json(request): Json<CreateEngramRequest>,
) -> Result<Json<EngramResponse>, ErrorResponse> {
    let harvester = get_harvester_clone()?;

    let mut engram = ArbEngram::new(
        request.key,
        request.engram_type,
        request.content,
        crate::models::EngramSource::Manual("api".to_string()),
    );

    if let Some(confidence) = request.confidence {
        engram = engram.with_confidence(confidence);
    }

    if let Some(tags) = request.tags {
        engram.metadata.tags = tags;
    }

    if let Some(hours) = request.expires_in_hours {
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(hours as i64);
        engram = engram.with_expiry(expires_at);
    }

    match harvester.store_engram(engram.clone()).await {
        Ok(_) => Ok(Json(EngramResponse {
            success: true,
            engram: Some(engram),
            message: None,
        })),
        Err(e) => Err(json_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())),
    }
}

pub async fn get_engram(
    State(_state): State<AppState>,
    Path(key): Path<String>,
) -> Result<Json<EngramResponse>, ErrorResponse> {
    let harvester = get_harvester_clone()?;

    match harvester.get_engram(&key).await {
        Some(engram) => Ok(Json(EngramResponse {
            success: true,
            engram: Some(engram),
            message: None,
        })),
        None => Err(json_error(StatusCode::NOT_FOUND, &format!("Engram not found: {}", key))),
    }
}

pub async fn search_engrams(
    State(_state): State<AppState>,
    Query(query): Query<EngramQueryParams>,
) -> Result<Json<EngramListResponse>, ErrorResponse> {
    let harvester = get_harvester_clone()?;

    let engram_type = query.engram_type.and_then(|t| parse_engram_type(&t));

    let search_query = EngramQuery {
        engram_type,
        key_prefix: query.key_prefix,
        tag: query.tag,
        min_confidence: query.min_confidence,
        limit: query.limit,
        offset: query.offset,
    };

    let result = harvester.search_engrams(&search_query).await;

    Ok(Json(EngramListResponse {
        success: true,
        engrams: result.engrams,
        total: result.total,
    }))
}

#[derive(Debug, Deserialize)]
pub struct EngramQueryParams {
    pub engram_type: Option<String>,
    pub key_prefix: Option<String>,
    pub tag: Option<String>,
    pub min_confidence: Option<f64>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

pub async fn find_patterns(
    State(_state): State<AppState>,
    Json(request): Json<PatternMatchRequest>,
) -> Result<Json<PatternMatchResponse>, ErrorResponse> {
    let harvester = get_harvester_clone()?;

    let matches = harvester.find_matching_patterns(&request).await;

    Ok(Json(PatternMatchResponse {
        success: true,
        count: matches.len(),
        matches,
    }))
}

pub async fn check_avoidance(
    State(_state): State<AppState>,
    Path((entity_type, address)): Path<(String, String)>,
) -> Result<Json<AvoidanceCheckResponse>, ErrorResponse> {
    let harvester = get_harvester_clone()?;

    match harvester.should_avoid(&entity_type, &address).await {
        Some(avoidance) => Ok(Json(AvoidanceCheckResponse {
            should_avoid: true,
            reason: Some(avoidance.reason),
            category: Some(avoidance.category),
            severity: Some(format!("{:?}", avoidance.severity)),
        })),
        None => Ok(Json(AvoidanceCheckResponse {
            should_avoid: false,
            reason: None,
            category: None,
            severity: None,
        })),
    }
}

pub async fn create_avoidance(
    State(_state): State<AppState>,
    Json(request): Json<CreateAvoidanceRequest>,
) -> Result<Json<EngramResponse>, ErrorResponse> {
    let harvester = get_harvester_clone()?;

    let severity = parse_severity(&request.severity)
        .ok_or_else(|| json_error(StatusCode::BAD_REQUEST, "Invalid severity"))?;

    match harvester
        .create_avoidance_engram(
            &request.entity_type,
            &request.address,
            &request.reason,
            &request.category,
            severity,
        )
        .await
    {
        Ok(id) => {
            let key = format!("arb.avoid.{}.{}", request.entity_type, request.address);
            let engram = harvester.get_engram(&key).await;
            Ok(Json(EngramResponse {
                success: true,
                engram,
                message: Some(format!("Created avoidance engram: {}", id)),
            }))
        }
        Err(e) => Err(json_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())),
    }
}

pub async fn create_pattern(
    State(_state): State<AppState>,
    Json(request): Json<CreatePatternRequest>,
) -> Result<Json<EngramResponse>, ErrorResponse> {
    let harvester = get_harvester_clone()?;

    match harvester
        .create_edge_pattern_engram(
            &request.edge_type,
            &request.venue_type,
            &request.route_signature,
            request.success_rate,
            request.avg_profit_bps,
            request.sample_count,
        )
        .await
    {
        Ok(id) => Ok(Json(EngramResponse {
            success: true,
            engram: None,
            message: Some(format!("Created pattern engram: {}", id)),
        })),
        Err(e) => Err(json_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())),
    }
}

pub async fn delete_engram(
    State(_state): State<AppState>,
    Path(key): Path<String>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let harvester = get_harvester_clone()?;

    if harvester.delete_engram(&key).await {
        Ok(Json(serde_json::json!({
            "success": true,
            "message": format!("Deleted engram: {}", key)
        })))
    } else {
        Err(json_error(StatusCode::NOT_FOUND, &format!("Engram not found: {}", key)))
    }
}

pub async fn get_harvester_stats(
    State(_state): State<AppState>,
) -> Result<Json<HarvesterStatsResponse>, ErrorResponse> {
    let harvester = get_harvester_clone()?;

    let stats = harvester.get_stats().await;

    Ok(Json(HarvesterStatsResponse {
        total_engrams: stats.total_engrams,
        engrams_by_type: stats.engrams_by_type,
        patterns_matched: stats.patterns_matched,
        avoidances_created: stats.avoidances_created,
        last_harvest_at: stats.last_harvest_at.map(|t| t.to_rfc3339()),
    }))
}

fn parse_engram_type(s: &str) -> Option<EngramType> {
    match s.to_lowercase().as_str() {
        "edge_pattern" => Some(EngramType::EdgePattern),
        "avoidance" => Some(EngramType::Avoidance),
        "strategy" => Some(EngramType::Strategy),
        "threat_intel" => Some(EngramType::ThreatIntel),
        "consensus_outcome" => Some(EngramType::ConsensusOutcome),
        "trade_result" => Some(EngramType::TradeResult),
        "market_condition" => Some(EngramType::MarketCondition),
        _ => None,
    }
}

fn parse_severity(s: &str) -> Option<AvoidanceSeverity> {
    match s.to_lowercase().as_str() {
        "low" => Some(AvoidanceSeverity::Low),
        "medium" => Some(AvoidanceSeverity::Medium),
        "high" => Some(AvoidanceSeverity::High),
        "critical" => Some(AvoidanceSeverity::Critical),
        _ => None,
    }
}
