use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::agents::start_autonomous_executor;
use crate::models::{CreateStrategyRequest, Strategy, UpdateStrategyRequest};
use crate::server::AppState;

#[derive(Debug, Serialize)]
pub struct StrategiesListResponse {
    pub strategies: Vec<Strategy>,
    pub total: usize,
}

pub async fn list_strategies(State(state): State<AppState>) -> impl IntoResponse {
    let strategies = state.strategy_engine.list_strategies().await;
    let total = strategies.len();

    (
        StatusCode::OK,
        Json(StrategiesListResponse { strategies, total }),
    )
}

pub async fn get_strategy(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.strategy_engine.get_strategy(id).await {
        Some(strategy) => (StatusCode::OK, Json(strategy)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Strategy not found"})),
        )
            .into_response(),
    }
}

pub async fn create_strategy(
    State(state): State<AppState>,
    Json(request): Json<CreateStrategyRequest>,
) -> impl IntoResponse {
    use crate::database::repositories::strategies::CreateStrategyRecord;

    // Create in database first for persistence
    let db_record = match state
        .strategy_repo
        .create(CreateStrategyRecord {
            wallet_address: request.wallet_address.clone(),
            name: request.name.clone(),
            strategy_type: request.strategy_type.clone(),
            venue_types: request.venue_types.clone(),
            execution_mode: request.execution_mode.clone(),
            risk_params: request.risk_params.clone(),
        })
        .await
    {
        Ok(record) => record,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Failed to persist strategy: {}", e)})),
            )
                .into_response();
        }
    };

    // Convert DB record to Strategy model
    let strategy = Strategy {
        id: db_record.id,
        wallet_address: db_record.wallet_address,
        name: db_record.name,
        strategy_type: db_record.strategy_type,
        venue_types: db_record.venue_types,
        execution_mode: db_record.execution_mode,
        risk_params: serde_json::from_value(db_record.risk_params).unwrap_or(request.risk_params),
        is_active: db_record.is_active,
        created_at: db_record.created_at,
        updated_at: db_record.updated_at,
        last_tested_at: None,
        last_executed_at: None,
        test_results: None,
    };

    // Add to in-memory engine for fast access
    state.strategy_engine.add_strategy(strategy.clone()).await;

    // Persist to engrams for cross-session persistence
    let wallet = state
        .config
        .wallet_address
        .clone()
        .unwrap_or_else(|| "default".to_string());
    let risk_params_json = serde_json::to_value(&strategy.risk_params).unwrap_or_default();
    if let Err(e) = state
        .engrams_client
        .save_strategy_full(
            &wallet,
            &strategy.id.to_string(),
            &strategy.name,
            &strategy.strategy_type,
            &strategy.venue_types,
            &strategy.execution_mode,
            &risk_params_json,
            strategy.is_active,
        )
        .await
    {
        tracing::warn!(strategy_id = %strategy.id, error = %e, "Failed to persist strategy to engrams");
    }

    (StatusCode::CREATED, Json(strategy)).into_response()
}

pub async fn delete_strategy(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // Delete from database first for persistence
    if let Err(e) = state.strategy_repo.delete(id).await {
        tracing::warn!(strategy_id = %id, error = %e, "Failed to delete strategy from database");
    }

    // Remove from in-memory engine
    if state.strategy_engine.remove_strategy(id).await {
        (
            StatusCode::OK,
            Json(serde_json::json!({"deleted": true, "id": id})),
        )
            .into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Strategy not found"})),
        )
            .into_response()
    }
}

#[derive(Debug, Deserialize)]
pub struct ToggleRequest {
    pub enabled: bool,
}

pub async fn toggle_strategy(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<ToggleRequest>,
) -> impl IntoResponse {
    // Persist toggle to database first
    if let Err(e) = state.strategy_repo.toggle(id, request.enabled).await {
        tracing::warn!(strategy_id = %id, error = %e, "Failed to persist toggle to database");
    }

    // Update in-memory engine
    match state
        .strategy_engine
        .toggle_strategy(id, request.enabled)
        .await
    {
        Ok(_) => {
            // Persist toggle state to engrams
            if let Some(strategy) = state.strategy_engine.get_strategy(id).await {
                // If enabling strategy with auto_execute_enabled, start executor
                if request.enabled && strategy.risk_params.auto_execute_enabled {
                    start_autonomous_executor(state.autonomous_executor.clone());
                    tracing::info!(strategy_id = %id, "Strategy enabled with auto-execution - starting executor");
                }

                let wallet = state
                    .config
                    .wallet_address
                    .clone()
                    .unwrap_or_else(|| "default".to_string());
                let risk_params_json =
                    serde_json::to_value(&strategy.risk_params).unwrap_or_default();
                if let Err(e) = state
                    .engrams_client
                    .save_strategy_full(
                        &wallet,
                        &strategy.id.to_string(),
                        &strategy.name,
                        &strategy.strategy_type,
                        &strategy.venue_types,
                        &strategy.execution_mode,
                        &risk_params_json,
                        request.enabled,
                    )
                    .await
                {
                    tracing::warn!(strategy_id = %id, error = %e, "Failed to persist toggle to engrams");
                }
            }

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "id": id,
                    "is_active": request.enabled
                })),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn update_strategy(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateStrategyRequest>,
) -> impl IntoResponse {
    use crate::database::repositories::strategies::UpdateStrategyRecord;

    // Persist update to database first
    if let Err(e) = state
        .strategy_repo
        .update(
            id,
            UpdateStrategyRecord {
                name: request.name.clone(),
                venue_types: request.venue_types.clone(),
                execution_mode: request.execution_mode.clone(),
                risk_params: request.risk_params.clone(),
                is_active: request.is_active,
            },
        )
        .await
    {
        tracing::warn!(strategy_id = %id, error = %e, "Failed to persist update to database");
    }

    // Update in-memory engine
    match state.strategy_engine.update_strategy(id, request).await {
        Ok(strategy) => {
            // If auto_execute_enabled is now true, start the executor if not running
            if strategy.risk_params.auto_execute_enabled && strategy.is_active {
                start_autonomous_executor(state.autonomous_executor.clone());
                tracing::info!(strategy_id = %id, "Auto-execution enabled - starting autonomous executor");
            }

            // Persist updated strategy to engrams
            let wallet = state
                .config
                .wallet_address
                .clone()
                .unwrap_or_else(|| "default".to_string());
            let risk_params_json = serde_json::to_value(&strategy.risk_params).unwrap_or_default();
            if let Err(e) = state
                .engrams_client
                .save_strategy_full(
                    &wallet,
                    &strategy.id.to_string(),
                    &strategy.name,
                    &strategy.strategy_type,
                    &strategy.venue_types,
                    &strategy.execution_mode,
                    &risk_params_json,
                    strategy.is_active,
                )
                .await
            {
                tracing::warn!(strategy_id = %id, error = %e, "Failed to persist update to engrams");
            }

            (StatusCode::OK, Json(strategy)).into_response()
        }
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct SetRiskProfileRequest {
    pub profile: String, // "conservative", "moderate", "aggressive"
}

pub async fn set_risk_profile(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<SetRiskProfileRequest>,
) -> impl IntoResponse {
    use crate::database::repositories::strategies::UpdateStrategyRecord;
    use crate::models::RiskParams;

    let risk_params = RiskParams::from_profile(&request.profile);
    let risk_params_json = serde_json::to_value(&risk_params).ok();

    // Persist to database first
    if let Err(e) = state
        .strategy_repo
        .update(
            id,
            UpdateStrategyRecord {
                name: None,
                venue_types: None,
                execution_mode: None,
                risk_params: Some(risk_params.clone()),
                is_active: None,
            },
        )
        .await
    {
        tracing::warn!(strategy_id = %id, error = %e, "Failed to persist risk profile to database");
    }

    // Update in-memory engine
    match state
        .strategy_engine
        .set_risk_params(id, risk_params.clone())
        .await
    {
        Ok(strategy) => {
            // If auto_execute_enabled is now true, start the executor
            if strategy.risk_params.auto_execute_enabled && strategy.is_active {
                start_autonomous_executor(state.autonomous_executor.clone());
                tracing::info!(strategy_id = %id, profile = %request.profile, "Risk profile enables auto-execution - starting executor");
            }

            // Persist to engrams
            let wallet = state
                .config
                .wallet_address
                .clone()
                .unwrap_or_else(|| "default".to_string());
            if let Err(e) = state
                .engrams_client
                .save_strategy_full(
                    &wallet,
                    &strategy.id.to_string(),
                    &strategy.name,
                    &strategy.strategy_type,
                    &strategy.venue_types,
                    &strategy.execution_mode,
                    &risk_params_json.unwrap_or_default(),
                    strategy.is_active,
                )
                .await
            {
                tracing::warn!(strategy_id = %id, error = %e, "Failed to persist risk profile to engrams");
            }

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "profile": request.profile,
                    "strategy": strategy
                })),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct BatchToggleRequest {
    pub ids: Vec<Uuid>,
    pub enabled: bool,
}

pub async fn batch_toggle_strategies(
    State(state): State<AppState>,
    Json(request): Json<BatchToggleRequest>,
) -> impl IntoResponse {
    let mut results = Vec::new();
    for id in &request.ids {
        // Persist to database first
        if let Err(e) = state.strategy_repo.toggle(*id, request.enabled).await {
            tracing::warn!(strategy_id = %id, error = %e, "Failed to persist batch toggle to database");
        }

        // Update in-memory engine
        match state
            .strategy_engine
            .toggle_strategy(*id, request.enabled)
            .await
        {
            Ok(_) => results.push(serde_json::json!({"id": id, "success": true})),
            Err(e) => results
                .push(serde_json::json!({"id": id, "success": false, "error": e.to_string()})),
        }
    }
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "results": results,
            "enabled": request.enabled
        })),
    )
}

pub async fn save_strategies_to_engrams(State(state): State<AppState>) -> impl IntoResponse {
    let strategies = state.strategy_engine.list_strategies().await;
    let wallet = state
        .config
        .wallet_address
        .clone()
        .unwrap_or_else(|| "default".to_string());

    // Save strategies via the engrams client with full state including is_active
    let mut saved = 0;
    for strategy in &strategies {
        let risk_params_json = serde_json::to_value(&strategy.risk_params).unwrap_or_default();
        match state
            .engrams_client
            .save_strategy_full(
                &wallet,
                &strategy.id.to_string(),
                &strategy.name,
                &strategy.strategy_type,
                &strategy.venue_types,
                &strategy.execution_mode,
                &risk_params_json,
                strategy.is_active,
            )
            .await
        {
            Ok(_) => saved += 1,
            Err(e) => {
                tracing::warn!("Failed to save strategy {}: {}", strategy.id, e);
            }
        }
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "message": format!("Saved {} strategies to engrams", saved),
            "count": saved
        })),
    )
        .into_response()
}

pub async fn reset_strategy_stats(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.strategy_engine.reset_stats(id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "id": id,
                "message": "Strategy stats reset"
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "error": e.to_string()
            })),
        )
            .into_response(),
    }
}

pub async fn kill_strategy(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.strategy_engine.kill_strategy(id).await {
        Ok(strategy_name) => {
            // Also cancel any pending approvals for this strategy
            if let Err(e) = state.approval_manager.cancel_by_strategy(id).await {
                tracing::warn!(strategy_id = %id, error = %e, "Failed to cancel pending approvals");
            }

            // Persist killed state to database
            if let Err(e) = state.strategy_repo.toggle(id, false).await {
                tracing::warn!(strategy_id = %id, error = %e, "Failed to persist kill to database");
            }

            // Persist killed state to engrams (is_active = false)
            if let Some(strategy) = state.strategy_engine.get_strategy(id).await {
                let wallet = state
                    .config
                    .wallet_address
                    .clone()
                    .unwrap_or_else(|| "default".to_string());
                let risk_params_json =
                    serde_json::to_value(&strategy.risk_params).unwrap_or_default();
                if let Err(e) = state
                    .engrams_client
                    .save_strategy_full(
                        &wallet,
                        &strategy.id.to_string(),
                        &strategy.name,
                        &strategy.strategy_type,
                        &strategy.venue_types,
                        &strategy.execution_mode,
                        &risk_params_json,
                        false, // Killed = inactive
                    )
                    .await
                {
                    tracing::warn!(strategy_id = %id, error = %e, "Failed to persist kill to engrams");
                }
            }

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "id": id,
                    "strategy_name": strategy_name,
                    "message": "Strategy killed - all operations halted",
                    "action": "emergency_stop"
                })),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "error": e.to_string()
            })),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct MomentumToggleRequest {
    pub enabled: bool,
}

pub async fn toggle_strategy_momentum(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<MomentumToggleRequest>,
) -> impl IntoResponse {
    use crate::database::repositories::strategies::UpdateStrategyRecord;

    match state.strategy_engine.get_strategy(id).await {
        Some(strategy) => {
            let mut updated_params = strategy.risk_params.clone();
            updated_params.momentum_adaptive_exits = request.enabled;

            // Update in-memory
            if let Err(e) = state
                .strategy_engine
                .set_risk_params(id, updated_params.clone())
                .await
            {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "success": false,
                        "error": format!("Failed to update strategy: {}", e)
                    })),
                )
                    .into_response();
            }

            // Persist to database
            if let Err(e) = state
                .strategy_repo
                .update(
                    id,
                    UpdateStrategyRecord {
                        name: None,
                        venue_types: None,
                        execution_mode: None,
                        risk_params: Some(updated_params.clone()),
                        is_active: None,
                    },
                )
                .await
            {
                tracing::warn!(strategy_id = %id, error = %e, "Failed to persist momentum toggle to database");
            }

            // Persist to engrams
            let wallet = state
                .config
                .wallet_address
                .clone()
                .unwrap_or_else(|| "default".to_string());
            let risk_params_json = serde_json::to_value(&updated_params).unwrap_or_default();
            if let Err(e) = state
                .engrams_client
                .save_strategy_full(
                    &wallet,
                    &strategy.id.to_string(),
                    &strategy.name,
                    &strategy.strategy_type,
                    &strategy.venue_types,
                    &strategy.execution_mode,
                    &risk_params_json,
                    strategy.is_active,
                )
                .await
            {
                tracing::warn!(strategy_id = %id, error = %e, "Failed to persist momentum toggle to engrams");
            }

            tracing::info!(
                strategy_id = %id,
                strategy_name = %strategy.name,
                momentum_enabled = request.enabled,
                "🔄 Momentum {} for strategy",
                if request.enabled { "enabled" } else { "disabled" }
            );

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "id": id,
                    "strategy_name": strategy.name,
                    "momentum_enabled": request.enabled
                })),
            )
                .into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "error": "Strategy not found"
            })),
        )
            .into_response(),
    }
}
