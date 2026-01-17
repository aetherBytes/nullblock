use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppResult;
use crate::research::{
    url_ingest::{UrlIngester, IngestResult},
    strategy_extract::{StrategyExtractor, ExtractedStrategy, TextStrategyExtractor},
    backtest::{BacktestEngine, BacktestConfig, BacktestResult},
    social_monitor::{SocialMonitor, MonitoredSource, SourceType, TrackType, SocialAlert, MonitorStats},
};
use crate::server::AppState;

#[derive(Debug, Deserialize)]
pub struct IngestUrlRequest {
    pub url: String,
    pub context: Option<String>,
    pub extract_strategy: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct IngestUrlResponse {
    pub ingest_result: IngestResult,
    pub extracted_strategy: Option<ExtractedStrategy>,
}

pub async fn ingest_url(
    State(state): State<AppState>,
    Json(request): Json<IngestUrlRequest>,
) -> impl IntoResponse {
    let ingester = UrlIngester::new();

    let ingest_result = match ingester.ingest(&request.url).await {
        Ok(result) => result,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": format!("Failed to ingest URL: {}", e)
                })),
            ).into_response();
        }
    };

    let extracted_strategy = if request.extract_strategy.unwrap_or(true) {
        let extractor = StrategyExtractor::new(
            state.config.openrouter_api_url.clone(),
            state.config.openrouter_api_key.clone(),
        );

        match extractor.extract(&ingest_result).await {
            Ok(strategy) => strategy,
            Err(_) => extractor.extract_without_llm(&ingest_result),
        }
    } else {
        None
    };

    (
        StatusCode::OK,
        Json(IngestUrlResponse {
            ingest_result,
            extracted_strategy,
        }),
    ).into_response()
}

#[derive(Debug, Deserialize)]
pub struct ExtractFromTextRequest {
    pub description: String,
    pub context: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExtractFromTextResponse {
    pub extracted_strategy: Option<ExtractedStrategy>,
    pub success: bool,
    pub error: Option<String>,
}

pub async fn extract_strategy_from_text(
    State(state): State<AppState>,
    Json(request): Json<ExtractFromTextRequest>,
) -> impl IntoResponse {
    if request.description.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ExtractFromTextResponse {
                extracted_strategy: None,
                success: false,
                error: Some("Description is required".to_string()),
            }),
        ).into_response();
    }

    let extractor = TextStrategyExtractor::new(
        state.config.openrouter_api_url.clone(),
        state.config.openrouter_api_key.clone(),
    );

    match extractor.extract_from_text(&request.description, request.context.as_deref()).await {
        Ok(Some(strategy)) => (
            StatusCode::OK,
            Json(ExtractFromTextResponse {
                extracted_strategy: Some(strategy),
                success: true,
                error: None,
            }),
        ).into_response(),
        Ok(None) => (
            StatusCode::OK,
            Json(ExtractFromTextResponse {
                extracted_strategy: None,
                success: false,
                error: Some("Could not extract a clear trading strategy from the description. Try being more specific about entry/exit conditions.".to_string()),
            }),
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ExtractFromTextResponse {
                extracted_strategy: None,
                success: false,
                error: Some(format!("Extraction failed: {}", e)),
            }),
        ).into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListDiscoveriesQuery {
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct DiscoveryListResponse {
    pub discoveries: Vec<ExtractedStrategy>,
    pub total: usize,
}

pub async fn list_discoveries(
    State(_state): State<AppState>,
    Query(_query): Query<ListDiscoveriesQuery>,
) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(DiscoveryListResponse {
            discoveries: Vec::new(),
            total: 0,
        }),
    )
}

pub async fn get_discovery(
    State(_state): State<AppState>,
    Path(_discovery_id): Path<Uuid>,
) -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
            "error": "Discovery not found"
        })),
    )
}

#[derive(Debug, Deserialize)]
pub struct ApproveDiscoveryRequest {
    pub notes: Option<String>,
}

pub async fn approve_discovery(
    State(_state): State<AppState>,
    Path(discovery_id): Path<Uuid>,
    Json(_request): Json<ApproveDiscoveryRequest>,
) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "message": "Discovery approved",
            "discovery_id": discovery_id,
            "status": "approved"
        })),
    )
}

#[derive(Debug, Deserialize)]
pub struct RejectDiscoveryRequest {
    pub reason: String,
}

pub async fn reject_discovery(
    State(_state): State<AppState>,
    Path(discovery_id): Path<Uuid>,
    Json(request): Json<RejectDiscoveryRequest>,
) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "message": "Discovery rejected",
            "discovery_id": discovery_id,
            "status": "rejected",
            "reason": request.reason
        })),
    )
}

#[derive(Debug, Deserialize)]
pub struct BacktestRequest {
    pub strategy_id: Uuid,
    pub period_days: Option<u32>,
    pub initial_capital_sol: Option<f64>,
    pub max_position_size_sol: Option<f64>,
}

pub async fn run_backtest(
    State(_state): State<AppState>,
    Json(request): Json<BacktestRequest>,
) -> impl IntoResponse {
    let config = BacktestConfig {
        period_days: request.period_days.unwrap_or(30),
        initial_capital_sol: request.initial_capital_sol.unwrap_or(10.0),
        max_position_size_sol: request.max_position_size_sol.unwrap_or(1.0),
        ..BacktestConfig::default()
    };

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "message": "Backtest queued",
            "strategy_id": request.strategy_id,
            "config": config
        })),
    )
}

pub async fn get_backtest_result(
    State(_state): State<AppState>,
    Path(_backtest_id): Path<Uuid>,
) -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
            "error": "Backtest result not found"
        })),
    )
}

#[derive(Debug, Deserialize)]
pub struct AddSourceRequest {
    pub source_type: String,
    pub handle: String,
    pub track_type: String,
    pub display_name: Option<String>,
    pub keywords: Option<Vec<String>>,
}

pub async fn add_source(
    State(_state): State<AppState>,
    Json(request): Json<AddSourceRequest>,
) -> impl IntoResponse {
    let source_type = SourceType::from_str(&request.source_type);
    let track_type = TrackType::from_str(&request.track_type);

    let mut source = MonitoredSource::new(source_type, request.handle, track_type);

    if let Some(name) = request.display_name {
        source = source.with_display_name(name);
    }

    if let Some(keywords) = request.keywords {
        source = source.with_keywords(keywords);
    }

    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "message": "Source added",
            "source": source
        })),
    )
}

#[derive(Debug, Deserialize)]
pub struct ListSourcesQuery {
    pub source_type: Option<String>,
    pub track_type: Option<String>,
    pub active_only: Option<bool>,
}

pub async fn list_sources(
    State(_state): State<AppState>,
    Query(_query): Query<ListSourcesQuery>,
) -> impl IntoResponse {
    let default_sources: Vec<MonitoredSource> = SocialMonitor::get_default_alpha_accounts()
        .into_iter()
        .map(|(handle, name)| {
            MonitoredSource::new(
                SourceType::Twitter,
                handle.trim_start_matches('@').to_string(),
                TrackType::Alpha,
            ).with_display_name(name.to_string())
        })
        .chain(
            SocialMonitor::get_default_threat_accounts()
                .into_iter()
                .map(|(handle, name)| {
                    MonitoredSource::new(
                        SourceType::Twitter,
                        handle.trim_start_matches('@').to_string(),
                        TrackType::Threat,
                    ).with_display_name(name.to_string())
                })
        )
        .collect();

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "sources": default_sources,
            "total": default_sources.len()
        })),
    )
}

pub async fn get_source(
    State(_state): State<AppState>,
    Path(source_id): Path<Uuid>,
) -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
            "error": "Source not found",
            "source_id": source_id
        })),
    )
}

pub async fn delete_source(
    State(_state): State<AppState>,
    Path(source_id): Path<Uuid>,
) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "message": "Source deleted",
            "source_id": source_id
        })),
    )
}

#[derive(Debug, Deserialize)]
pub struct ToggleSourceRequest {
    pub active: bool,
}

pub async fn toggle_source(
    State(_state): State<AppState>,
    Path(source_id): Path<Uuid>,
    Json(request): Json<ToggleSourceRequest>,
) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "message": if request.active { "Source activated" } else { "Source deactivated" },
            "source_id": source_id,
            "active": request.active
        })),
    )
}

#[derive(Debug, Deserialize)]
pub struct ListAlertsQuery {
    pub source_id: Option<Uuid>,
    pub alert_type: Option<String>,
    pub limit: Option<usize>,
}

pub async fn list_alerts(
    State(_state): State<AppState>,
    Query(_query): Query<ListAlertsQuery>,
) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "alerts": Vec::<SocialAlert>::new(),
            "total": 0
        })),
    )
}

pub async fn get_monitor_stats(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let stats = MonitorStats {
        total_sources: 8,
        active_sources: 8,
        total_alerts: 0,
        alerts_last_24h: 0,
        sources_by_type: std::collections::HashMap::from([
            ("Twitter".to_string(), 8),
        ]),
    };

    (StatusCode::OK, Json(stats))
}

#[derive(Debug, Deserialize)]
pub struct MonitorAccountRequest {
    pub handle: String,
    pub track_type: String,
}

pub async fn monitor_account(
    State(_state): State<AppState>,
    Json(request): Json<MonitorAccountRequest>,
) -> impl IntoResponse {
    let handle = request.handle.trim_start_matches('@').to_string();
    let track_type = TrackType::from_str(&request.track_type);

    let source = MonitoredSource::new(SourceType::Twitter, handle.clone(), track_type);

    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "message": format!("Now monitoring @{}", handle),
            "source": source
        })),
    )
}
