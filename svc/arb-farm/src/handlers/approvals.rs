use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::agents::start_autonomous_executor;
use crate::models::{
    ApproveRequest, ExecutionToggleRequest, HecateRecommendation,
    RejectRequest, UpdateExecutionConfigRequest,
};
use crate::server::AppState;

pub async fn list_approvals(State(state): State<AppState>) -> impl IntoResponse {
    let approvals = state.approval_manager.list_all().await;
    (StatusCode::OK, Json(serde_json::json!({
        "approvals": approvals,
        "total": approvals.len()
    })))
}

pub async fn list_pending_approvals(State(state): State<AppState>) -> impl IntoResponse {
    let approvals = state.approval_manager.list_pending().await;
    (StatusCode::OK, Json(serde_json::json!({
        "approvals": approvals,
        "total": approvals.len()
    })))
}

pub async fn get_approval(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.approval_manager.get_approval(id).await {
        Some(approval) => (StatusCode::OK, Json(approval)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Approval not found"})),
        )
            .into_response(),
    }
}

pub async fn approve_approval(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<ApproveRequest>,
) -> impl IntoResponse {
    match state.approval_manager.approve(id, request.notes).await {
        Ok(approval) => (StatusCode::OK, Json(approval)).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn reject_approval(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<RejectRequest>,
) -> impl IntoResponse {
    match state.approval_manager.reject(id, request.reason).await {
        Ok(approval) => (StatusCode::OK, Json(approval)).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn get_execution_config(State(state): State<AppState>) -> impl IntoResponse {
    let mut config = state.approval_manager.get_config().await;

    // Sync with actual RiskConfig values so frontend shows real settings
    let risk_config = state.risk_config.read().await;
    config.auto_max_position_sol = risk_config.max_position_sol;

    // Use autonomous executor's actual running state as source of truth
    let executor_stats = state.autonomous_executor.get_stats().await;
    config.auto_execution_enabled = executor_stats.is_running;

    // Check for state mismatch between executor and strategy execution_mode
    let strategies = state.strategy_engine.list_strategies().await;
    let curve_strategies: Vec<_> = strategies.iter()
        .filter(|s| s.strategy_type == "curve_arb" && s.is_active)
        .collect();
    let any_autonomous = curve_strategies.iter().any(|s| s.execution_mode == "autonomous");

    if executor_stats.is_running && !any_autonomous && !curve_strategies.is_empty() {
        tracing::warn!(
            executor_running = executor_stats.is_running,
            curve_strategies_count = curve_strategies.len(),
            "⚠️ STATE MISMATCH: executor is running but no curve_arb strategy has autonomous mode - auto-fixing"
        );

        // Log each strategy's current state for debugging
        for strategy in &curve_strategies {
            tracing::warn!(
                strategy_id = %strategy.id,
                strategy_name = %strategy.name,
                execution_mode = %strategy.execution_mode,
                auto_execute_enabled = strategy.risk_params.auto_execute_enabled,
                is_active = strategy.is_active,
                "  └─ Mismatched strategy details"
            );
        }

        // Auto-fix: update strategies to autonomous
        for strategy in curve_strategies {
            if let Err(e) = state.strategy_engine.set_execution_mode(strategy.id, "autonomous".to_string()).await {
                tracing::warn!(strategy_id = %strategy.id, error = %e, "Failed to auto-fix execution_mode");
            } else {
                // Also update database
                use crate::database::repositories::strategies::UpdateStrategyRecord;
                let _ = state.strategy_repo.update(strategy.id, UpdateStrategyRecord {
                    name: None,
                    venue_types: None,
                    execution_mode: Some("autonomous".to_string()),
                    risk_params: None,
                    is_active: None,
                }).await;
                tracing::info!(strategy_id = %strategy.id, strategy_name = %strategy.name, "✅ Auto-fixed execution_mode to autonomous");
            }
        }
    }

    (StatusCode::OK, Json(config))
}

pub async fn update_execution_config(
    State(state): State<AppState>,
    Json(request): Json<UpdateExecutionConfigRequest>,
) -> impl IntoResponse {
    let auto_exec_requested = request.auto_execution_enabled;
    let config = state.approval_manager.update_config(request.clone()).await;

    if let Some(enabled) = auto_exec_requested {
        if enabled {
            start_autonomous_executor(state.autonomous_executor.clone());
            tracing::info!("Auto-execution enabled via config update - starting executor");
        } else {
            state.autonomous_executor.stop().await;
            tracing::info!("Auto-execution disabled via config update - stopping executor");
        }
    }

    {
        let mut risk_config = state.risk_config.write().await;
        if let Some(v) = request.auto_max_position_sol {
            risk_config.max_position_sol = v;
            risk_config.max_position_per_token_sol = v;
            tracing::info!(max_position_sol = v, "Updated global RiskConfig max_position_sol");
        }
    }

    let strategies = state.strategy_engine.list_strategies().await;
    for strategy in strategies.iter().filter(|s| s.strategy_type == "curve_arb") {
        let mut updated_params = strategy.risk_params.clone();

        if let Some(enabled) = request.auto_execution_enabled {
            updated_params.auto_execute_enabled = enabled;
        }
        if let Some(v) = request.require_simulation {
            updated_params.require_simulation = v;
        }
        if let Some(v) = request.auto_max_position_sol {
            updated_params.max_position_sol = v;
        }

        // Determine execution_mode based on auto_execution setting
        let new_execution_mode = if updated_params.auto_execute_enabled {
            "autonomous"
        } else {
            "agent_directed"
        };

        if let Err(e) = state.strategy_engine.set_risk_params(strategy.id, updated_params.clone()).await {
            tracing::warn!(strategy_id = %strategy.id, error = %e, "Failed to update strategy risk params");
        }

        // Update execution_mode in memory
        if let Err(e) = state.strategy_engine.set_execution_mode(strategy.id, new_execution_mode.to_string()).await {
            tracing::warn!(strategy_id = %strategy.id, error = %e, "Failed to update strategy execution_mode");
        }

        use crate::database::repositories::strategies::UpdateStrategyRecord;
        if let Err(e) = state.strategy_repo.update(strategy.id, UpdateStrategyRecord {
            name: None,
            venue_types: None,
            execution_mode: Some(new_execution_mode.to_string()),
            risk_params: Some(updated_params.clone()),
            is_active: None,
        }).await {
            tracing::warn!(strategy_id = %strategy.id, error = %e, "Failed to persist execution config to database");
        }

        let wallet = state.config.wallet_address.clone().unwrap_or_else(|| "default".to_string());
        let risk_params_json = serde_json::to_value(&updated_params).unwrap_or_default();
        if let Err(e) = state.engrams_client.save_strategy_full(
            &wallet,
            &strategy.id.to_string(),
            &strategy.name,
            &strategy.strategy_type,
            &strategy.venue_types,
            new_execution_mode,
            &risk_params_json,
            strategy.is_active,
        ).await {
            tracing::warn!(strategy_id = %strategy.id, error = %e, "Failed to persist execution config to engrams");
        }

        tracing::info!(
            strategy_id = %strategy.id,
            strategy_name = %strategy.name,
            execution_mode = new_execution_mode,
            "Persisted execution config update"
        );
    }

    // Post-update verification logging
    let updated_strategies = state.strategy_engine.list_strategies().await;
    for strategy in updated_strategies.iter().filter(|s| s.strategy_type == "curve_arb") {
        tracing::info!(
            strategy_id = %strategy.id,
            strategy_name = %strategy.name,
            execution_mode = %strategy.execution_mode,
            auto_execute_enabled = strategy.risk_params.auto_execute_enabled,
            "✅ Post-config-update strategy state"
        );
    }

    (StatusCode::OK, Json(config))
}

pub async fn toggle_execution(
    State(state): State<AppState>,
    Json(request): Json<ExecutionToggleRequest>,
) -> impl IntoResponse {
    let config = state.approval_manager.toggle_execution(request.enabled).await;

    // Also control the autonomous executor
    if request.enabled {
        start_autonomous_executor(state.autonomous_executor.clone());
        tracing::info!("Auto-execution enabled via global toggle - starting executor");
    } else {
        // Stop the autonomous executor when globally disabled
        state.autonomous_executor.stop().await;
        tracing::info!("Auto-execution disabled via global toggle - stopping executor");
    }

    // Persist the toggle state for the default Curve Graduation strategy
    // This syncs the global toggle with the per-strategy auto_execute_enabled setting
    // AND updates execution_mode to match (autonomous when enabled, agent_directed when disabled)
    let strategies = state.strategy_engine.list_strategies().await;
    for strategy in strategies.iter().filter(|s| s.strategy_type == "curve_arb") {
        let mut updated_params = strategy.risk_params.clone();
        updated_params.auto_execute_enabled = request.enabled;

        // Set execution_mode based on toggle state
        let new_execution_mode = if request.enabled { "autonomous" } else { "agent_directed" };

        // Update in-memory engine (risk params)
        if let Err(e) = state.strategy_engine.set_risk_params(strategy.id, updated_params.clone()).await {
            tracing::warn!(strategy_id = %strategy.id, error = %e, "Failed to update strategy auto_execute_enabled");
        }

        // Update in-memory engine (execution mode)
        if let Err(e) = state.strategy_engine.set_execution_mode(strategy.id, new_execution_mode.to_string()).await {
            tracing::warn!(strategy_id = %strategy.id, error = %e, "Failed to update strategy execution_mode");
        }

        // Persist to database
        use crate::database::repositories::strategies::UpdateStrategyRecord;
        if let Err(e) = state.strategy_repo.update(strategy.id, UpdateStrategyRecord {
            name: None,
            venue_types: None,
            execution_mode: Some(new_execution_mode.to_string()),
            risk_params: Some(updated_params.clone()),
            is_active: None,
        }).await {
            tracing::warn!(strategy_id = %strategy.id, error = %e, "Failed to persist auto_execute toggle to database");
        }

        // Persist to engrams
        let wallet = state.config.wallet_address.clone().unwrap_or_else(|| "default".to_string());
        let risk_params_json = serde_json::to_value(&updated_params).unwrap_or_default();
        if let Err(e) = state.engrams_client.save_strategy_full(
            &wallet,
            &strategy.id.to_string(),
            &strategy.name,
            &strategy.strategy_type,
            &strategy.venue_types,
            new_execution_mode,
            &risk_params_json,
            strategy.is_active,
        ).await {
            tracing::warn!(strategy_id = %strategy.id, error = %e, "Failed to persist auto_execute toggle to engrams");
        }

        tracing::info!(
            strategy_id = %strategy.id,
            strategy_name = %strategy.name,
            auto_execute_enabled = request.enabled,
            execution_mode = new_execution_mode,
            "Persisted auto_execute_enabled and execution_mode settings"
        );
    }

    // Post-toggle verification logging
    let updated_strategies = state.strategy_engine.list_strategies().await;
    for strategy in updated_strategies.iter().filter(|s| s.strategy_type == "curve_arb") {
        tracing::info!(
            strategy_id = %strategy.id,
            strategy_name = %strategy.name,
            execution_mode = %strategy.execution_mode,
            auto_execute_enabled = strategy.risk_params.auto_execute_enabled,
            "✅ Post-toggle strategy state"
        );
    }

    (StatusCode::OK, Json(serde_json::json!({
        "enabled": config.auto_execution_enabled,
        "message": if config.auto_execution_enabled {
            "Auto-execution enabled"
        } else {
            "Auto-execution disabled"
        }
    })))
}

pub async fn add_hecate_recommendation(
    State(state): State<AppState>,
    Json(recommendation): Json<HecateRecommendation>,
) -> impl IntoResponse {
    match state.approval_manager.add_hecate_recommendation(recommendation).await {
        Ok(approval) => (StatusCode::OK, Json(approval)).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn cleanup_expired(State(state): State<AppState>) -> impl IntoResponse {
    let expired = state.approval_manager.cleanup_expired().await;
    (StatusCode::OK, Json(serde_json::json!({
        "expired_count": expired.len(),
        "expired_ids": expired
    })))
}
