use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::{
    CreateEngramRequest, EngramResponse, EngramsListResponse, ForkEngramRequest,
    SearchEngramsRequest, UpdateEngramRequest,
};
use crate::server::AppState;

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn create_engram(
    State(state): State<AppState>,
    Json(req): Json<CreateEngramRequest>,
) -> AppResult<Json<EngramResponse>> {
    tracing::info!(
        "Creating engram: wallet={}, type={}, key={}",
        req.wallet_address,
        req.engram_type,
        req.key
    );

    let engram = state.engram_repo.create(&req).await?;

    Ok(Json(EngramResponse {
        success: true,
        data: Some(engram),
        error: None,
    }))
}

pub async fn get_engram(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<EngramResponse>> {
    let engram = state.engram_repo.get_by_id(id).await?;

    Ok(Json(EngramResponse {
        success: true,
        data: Some(engram),
        error: None,
    }))
}

pub async fn list_engrams(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<EngramsListResponse>> {
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let (engrams, total) = state.engram_repo.list(limit, offset).await?;

    Ok(Json(EngramsListResponse {
        success: true,
        data: engrams,
        total,
        limit,
        offset,
    }))
}

pub async fn update_engram(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateEngramRequest>,
) -> AppResult<Json<EngramResponse>> {
    tracing::info!("Updating engram: id={}", id);

    let engram = state.engram_repo.update(id, &req).await?;

    Ok(Json(EngramResponse {
        success: true,
        data: Some(engram),
        error: None,
    }))
}

pub async fn delete_engram(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    tracing::info!("Deleting engram: id={}", id);

    state.engram_repo.delete(id).await?;

    Ok(Json(json!({
        "success": true,
        "message": format!("Engram {} deleted", id)
    })))
}

pub async fn get_engrams_by_wallet(
    State(state): State<AppState>,
    Path(wallet): Path<String>,
) -> AppResult<Json<EngramsListResponse>> {
    let engrams = state.engram_repo.get_by_wallet(&wallet).await?;
    let total = engrams.len() as i64;

    Ok(Json(EngramsListResponse {
        success: true,
        data: engrams,
        total,
        limit: total,
        offset: 0,
    }))
}

pub async fn get_engram_by_wallet_key(
    State(state): State<AppState>,
    Path((wallet, key)): Path<(String, String)>,
) -> AppResult<Json<EngramResponse>> {
    let engram = state.engram_repo.get_by_wallet_and_key(&wallet, &key).await?;

    Ok(Json(EngramResponse {
        success: true,
        data: Some(engram),
        error: None,
    }))
}

pub async fn search_engrams(
    State(state): State<AppState>,
    Json(req): Json<SearchEngramsRequest>,
) -> AppResult<Json<EngramsListResponse>> {
    let limit = req.limit.unwrap_or(50);
    let offset = req.offset.unwrap_or(0);

    let (engrams, total) = state.engram_repo.search(&req).await?;

    Ok(Json(EngramsListResponse {
        success: true,
        data: engrams,
        total,
        limit,
        offset,
    }))
}

pub async fn fork_engram(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<ForkEngramRequest>,
) -> AppResult<Json<EngramResponse>> {
    tracing::info!("Forking engram: id={} to wallet={}", id, req.target_wallet);

    let engram = state.engram_repo.fork(id, &req).await?;

    Ok(Json(EngramResponse {
        success: true,
        data: Some(engram),
        error: None,
    }))
}

pub async fn publish_engram(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<EngramResponse>> {
    tracing::info!("Publishing engram: id={}", id);

    let engram = state.engram_repo.publish(id).await?;

    Ok(Json(EngramResponse {
        success: true,
        data: Some(engram),
        error: None,
    }))
}
