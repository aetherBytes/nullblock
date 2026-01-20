use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::agents::{GraduationTrackerStats, TrackedToken, TrackerConfig};
use crate::error::{AppError, AppResult};
use crate::server::AppState;

#[derive(Debug, Deserialize)]
pub struct TrackTokenRequest {
    pub mint: String,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub venue: Option<String>,
    pub strategy_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TrackTokenResponse {
    pub success: bool,
    pub message: String,
    pub mint: String,
}

pub async fn track_token(
    State(state): State<AppState>,
    Json(request): Json<TrackTokenRequest>,
) -> AppResult<Json<TrackTokenResponse>> {
    let mint = &request.mint;
    let name = request.name.as_deref().unwrap_or("Unknown");
    let symbol = request.symbol.as_deref().unwrap_or("???");
    let venue = request.venue.as_deref().unwrap_or("pump_fun");
    let strategy_id = request.strategy_id
        .as_ref()
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or(Uuid::nil());

    let progress = match state.on_chain_fetcher.is_token_graduated(mint).await {
        Ok(status) => status.progress(),
        Err(_) => 0.0,
    };

    let success = state.graduation_tracker
        .track_with_persistence(mint, name, symbol, venue, strategy_id, progress)
        .await;

    if success {
        Ok(Json(TrackTokenResponse {
            success: true,
            message: format!("Now tracking {} ({}) at {:.1}% progress", symbol, mint, progress),
            mint: mint.to_string(),
        }))
    } else {
        Ok(Json(TrackTokenResponse {
            success: false,
            message: "Failed to persist tracking - token tracked in memory only".to_string(),
            mint: mint.to_string(),
        }))
    }
}

#[derive(Debug, Serialize)]
pub struct UntrackTokenResponse {
    pub success: bool,
    pub message: String,
    pub mint: String,
}

pub async fn untrack_token(
    State(state): State<AppState>,
    Path(mint): Path<String>,
) -> AppResult<Json<UntrackTokenResponse>> {
    let was_tracked = state.graduation_tracker.is_token_tracked(&mint).await;

    if !was_tracked {
        return Ok(Json(UntrackTokenResponse {
            success: false,
            message: format!("Token {} is not being tracked", mint),
            mint,
        }));
    }

    let success = state.graduation_tracker.untrack_with_persistence(&mint).await;

    Ok(Json(UntrackTokenResponse {
        success,
        message: if success {
            format!("Stopped tracking {}", mint)
        } else {
            format!("Removed {} from memory but failed to update engrams", mint)
        },
        mint,
    }))
}

#[derive(Debug, Serialize)]
pub struct ClearAllResponse {
    pub success: bool,
    pub message: String,
    pub cleared: usize,
}

pub async fn clear_all_tracked(
    State(state): State<AppState>,
) -> AppResult<Json<ClearAllResponse>> {
    let cleared = state.graduation_tracker.clear_all_with_persistence().await;

    Ok(Json(ClearAllResponse {
        success: true,
        message: format!("Cleared {} tracked tokens", cleared),
        cleared,
    }))
}

#[derive(Debug, Serialize)]
pub struct ListTrackedResponse {
    pub tokens: Vec<TrackedToken>,
    pub total: usize,
}

pub async fn list_tracked(
    State(state): State<AppState>,
) -> AppResult<Json<ListTrackedResponse>> {
    let tokens = state.graduation_tracker.list_tracked().await;
    let total = tokens.len();

    Ok(Json(ListTrackedResponse { tokens, total }))
}

#[derive(Debug, Serialize)]
pub struct IsTrackedResponse {
    pub mint: String,
    pub is_tracked: bool,
    pub token: Option<TrackedToken>,
}

pub async fn is_tracked(
    State(state): State<AppState>,
    Path(mint): Path<String>,
) -> AppResult<Json<IsTrackedResponse>> {
    let is_tracked = state.graduation_tracker.is_token_tracked(&mint).await;
    let token = if is_tracked {
        state.graduation_tracker.get_tracked(&mint).await
    } else {
        None
    };

    Ok(Json(IsTrackedResponse {
        mint,
        is_tracked,
        token,
    }))
}

pub async fn get_tracker_stats(
    State(state): State<AppState>,
) -> AppResult<Json<GraduationTrackerStats>> {
    let stats = state.graduation_tracker.get_stats().await;
    Ok(Json(stats))
}

#[derive(Debug, Serialize)]
pub struct TrackerControlResponse {
    pub success: bool,
    pub message: String,
    pub is_running: bool,
}

pub async fn start_tracker(
    State(state): State<AppState>,
) -> AppResult<Json<TrackerControlResponse>> {
    state.graduation_tracker.start().await;
    let stats = state.graduation_tracker.get_stats().await;

    Ok(Json(TrackerControlResponse {
        success: true,
        message: "Graduation tracker started".to_string(),
        is_running: stats.is_running,
    }))
}

pub async fn stop_tracker(
    State(state): State<AppState>,
) -> AppResult<Json<TrackerControlResponse>> {
    state.graduation_tracker.stop().await;
    let stats = state.graduation_tracker.get_stats().await;

    Ok(Json(TrackerControlResponse {
        success: true,
        message: "Graduation tracker stopped".to_string(),
        is_running: stats.is_running,
    }))
}

#[derive(Debug, Serialize)]
pub struct TrackerConfigResponse {
    pub config: TrackerConfig,
}

pub async fn get_tracker_config(
    State(state): State<AppState>,
) -> AppResult<Json<TrackerConfigResponse>> {
    let config = state.graduation_tracker.get_config().await;
    Ok(Json(TrackerConfigResponse { config }))
}

#[derive(Debug, Deserialize)]
pub struct UpdateTrackerConfigRequest {
    pub graduation_threshold: Option<f64>,
    pub fast_poll_interval_ms: Option<u64>,
    pub normal_poll_interval_ms: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct UpdateTrackerConfigResponse {
    pub success: bool,
    pub message: String,
    pub config: TrackerConfig,
}

pub async fn update_tracker_config(
    State(state): State<AppState>,
    Json(request): Json<UpdateTrackerConfigRequest>,
) -> AppResult<Json<UpdateTrackerConfigResponse>> {
    let mut config = state.graduation_tracker.get_config().await;

    if let Some(v) = request.graduation_threshold {
        if v < 0.0 || v > 100.0 {
            return Err(AppError::Validation(
                "graduation_threshold must be between 0 and 100".to_string()
            ));
        }
        config.graduation_threshold = v;
    }
    if let Some(v) = request.fast_poll_interval_ms {
        if v < 100 {
            return Err(AppError::Validation(
                "fast_poll_interval_ms must be at least 100ms".to_string()
            ));
        }
        config.fast_poll_interval_ms = v;
    }
    if let Some(v) = request.normal_poll_interval_ms {
        if v < 100 {
            return Err(AppError::Validation(
                "normal_poll_interval_ms must be at least 100ms".to_string()
            ));
        }
        config.normal_poll_interval_ms = v;
    }

    state.graduation_tracker.update_config(config.clone()).await;

    Ok(Json(UpdateTrackerConfigResponse {
        success: true,
        message: format!(
            "Tracker config updated: threshold={:.1}%, fast_poll={}ms, normal_poll={}ms",
            config.graduation_threshold,
            config.fast_poll_interval_ms,
            config.normal_poll_interval_ms
        ),
        config,
    }))
}
