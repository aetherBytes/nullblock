use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::server::AppState;
use crate::database::repositories::strategies::UpdateStrategyRecord;

#[derive(Debug, Deserialize)]
pub struct SetRiskRequest {
    pub level: String, // "low", "medium", "high"
}

#[derive(Debug, Deserialize)]
pub struct SetCustomRiskRequest {
    pub max_position_sol: Option<f64>,
    pub max_concurrent_positions: Option<u32>,
    pub daily_loss_limit_sol: Option<f64>,
    pub max_drawdown_percent: Option<f64>,
    pub take_profit_percent: Option<f64>,
    pub trailing_stop_percent: Option<f64>,
    pub time_limit_minutes: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct RiskLevelParams {
    pub level: String,
    pub max_position_sol: f64,
    pub max_concurrent_positions: u32,
    pub max_liquidity_contribution_pct: f64,
    pub stop_loss_pct: f64,
    pub take_profit_pct: f64,
    pub trailing_stop_pct: f64,
    pub time_limit_minutes: u32,
    pub daily_loss_limit_sol: f64,
}

pub async fn set_risk_level(
    State(state): State<AppState>,
    Json(request): Json<SetRiskRequest>,
) -> impl IntoResponse {
    let params = match request.level.to_lowercase().as_str() {
        "low" => RiskLevelParams {
            level: "low".to_string(),
            max_position_sol: 0.02,
            max_concurrent_positions: 2,
            max_liquidity_contribution_pct: 5.0,
            stop_loss_pct: 15.0,        // Was 10.0
            take_profit_pct: 10.0,      // Was 20.0 (conservative)
            trailing_stop_pct: 8.0,
            time_limit_minutes: 5,
            daily_loss_limit_sol: 0.1,
        },
        "high" | "aggressive" => RiskLevelParams {
            level: "aggressive".to_string(),
            max_position_sol: 10.0,
            max_concurrent_positions: 20,
            max_liquidity_contribution_pct: 50.0,
            stop_loss_pct: 10.0,        // DEFENSIVE: 10% tight stop
            take_profit_pct: 15.0,      // DEFENSIVE: 15% TP (strong momentum extends)
            trailing_stop_pct: 8.0,     // DEFENSIVE: 8% trailing
            time_limit_minutes: 5,      // DEFENSIVE: 5 min
            daily_loss_limit_sol: 5.0,
        },
        "conservative" => RiskLevelParams {
            level: "conservative".to_string(),
            max_position_sol: 1.0,
            max_concurrent_positions: 3,
            max_liquidity_contribution_pct: 10.0,
            stop_loss_pct: 15.0,        // Was 10.0
            take_profit_pct: 12.0,      // Was 25.0
            trailing_stop_pct: 10.0,
            time_limit_minutes: 5,
            daily_loss_limit_sol: 0.5,
        },
        _ => RiskLevelParams {
            level: "medium".to_string(),
            max_position_sol: 0.3,
            max_concurrent_positions: 10,
            max_liquidity_contribution_pct: 10.0,
            stop_loss_pct: 10.0,        // DEFENSIVE: 10% tight stop
            take_profit_pct: 15.0,      // DEFENSIVE: 15% TP (strong momentum extends)
            trailing_stop_pct: 8.0,     // DEFENSIVE: 8% trailing
            time_limit_minutes: 5,      // DEFENSIVE: 5 min
            daily_loss_limit_sol: 1.0,
        },
    };

    // Preserve current dynamic max_position_sol (set based on wallet balance at startup)
    let current_max_position = {
        let config = state.risk_config.read().await;
        config.max_position_sol
    };

    // Update the shared risk config (preserve wallet-based max_position)
    {
        let mut risk_config = state.risk_config.write().await;
        // Preserve dynamic max_position_sol - don't use preset value
        // risk_config.max_position_sol stays the same (from wallet calculation)
        risk_config.max_concurrent_positions = params.max_concurrent_positions;
        risk_config.daily_loss_limit_sol = params.daily_loss_limit_sol;
        risk_config.max_drawdown_percent = params.stop_loss_pct;
    }

    tracing::info!(
        "⚙️ Risk level set to {}: max_pos={:.2} SOL (preserved), concurrent={}, SL={}%",
        params.level,
        current_max_position,
        params.max_concurrent_positions,
        params.stop_loss_pct
    );

    // Sync ALL strategies with new risk config (overwrites all strategy risk params, preserves wallet-based max_position)
    let strategies = state.strategy_engine.list_strategies().await;
    let mut synced_count = 0;
    for strategy in &strategies {
        let mut updated_params = strategy.risk_params.clone();
        updated_params.max_position_sol = current_max_position; // Preserve wallet-based value
        updated_params.daily_loss_limit_sol = params.daily_loss_limit_sol;
        updated_params.stop_loss_percent = Some(params.stop_loss_pct);
        updated_params.take_profit_percent = Some(params.take_profit_pct);
        updated_params.trailing_stop_percent = Some(params.trailing_stop_pct);
        updated_params.time_limit_minutes = Some(params.time_limit_minutes);
        updated_params.concurrent_positions = Some(params.max_concurrent_positions);

        // Update in-memory
        if let Err(e) = state.strategy_engine.set_risk_params(strategy.id, updated_params.clone()).await {
            tracing::warn!(strategy_id = %strategy.id, error = %e, "Failed to sync strategy risk params");
            continue;
        }

        // Persist to database
        if let Err(e) = state.strategy_repo.update(strategy.id, UpdateStrategyRecord {
            name: None,
            venue_types: None,
            execution_mode: None,
            risk_params: Some(updated_params),
            is_active: None,
        }).await {
            tracing::warn!(strategy_id = %strategy.id, error = %e, "Failed to persist synced risk params");
        }

        synced_count += 1;
    }

    tracing::info!("✅ Synced {} strategies with new risk config (SL={}%, TP={}%, Trail={}%, Time={}min)",
        synced_count, params.stop_loss_pct, params.take_profit_pct, params.trailing_stop_pct, params.time_limit_minutes);

    (StatusCode::OK, Json(serde_json::json!({
        "success": true,
        "message": format!("Risk level set to {} - {} strategies synced", params.level, synced_count),
        "params": params,
    })))
}

pub async fn get_risk_level(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let risk_config = state.risk_config.read().await;

    let level = if risk_config.max_position_sol <= 0.05 {
        "low"
    } else if risk_config.max_position_sol > 0.5 {
        "high"
    } else {
        "medium"
    };

    (StatusCode::OK, Json(serde_json::json!({
        "level": level,
        "max_position_sol": risk_config.max_position_sol,
        "max_concurrent_positions": risk_config.max_concurrent_positions,
        "daily_loss_limit_sol": risk_config.daily_loss_limit_sol,
        "max_drawdown_percent": risk_config.max_drawdown_percent,
        "take_profit_percent": risk_config.take_profit_percent,
        "trailing_stop_percent": risk_config.trailing_stop_percent,
        "time_limit_minutes": risk_config.time_limit_minutes,
        "max_position_per_token_sol": risk_config.max_position_per_token_sol,
        "cooldown_after_loss_ms": risk_config.cooldown_after_loss_ms,
        "volatility_scaling_enabled": risk_config.volatility_scaling_enabled,
        "auto_pause_on_drawdown": risk_config.auto_pause_on_drawdown,
    })))
}

pub async fn set_custom_risk(
    State(state): State<AppState>,
    Json(request): Json<SetCustomRiskRequest>,
) -> impl IntoResponse {
    let mut risk_config = state.risk_config.write().await;

    if let Some(max_pos) = request.max_position_sol {
        if max_pos < 0.001 || max_pos > 10.0 {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "success": false,
                "error": "max_position_sol must be between 0.001 and 10.0 SOL"
            })));
        }
        risk_config.max_position_sol = max_pos;
        risk_config.max_position_per_token_sol = max_pos;
    }

    if let Some(max_concurrent) = request.max_concurrent_positions {
        if max_concurrent < 1 || max_concurrent > 50 {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "success": false,
                "error": "max_concurrent_positions must be between 1 and 50"
            })));
        }
        risk_config.max_concurrent_positions = max_concurrent;
    }

    if let Some(daily_limit) = request.daily_loss_limit_sol {
        if daily_limit < 0.01 || daily_limit > 100.0 {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "success": false,
                "error": "daily_loss_limit_sol must be between 0.01 and 100.0 SOL"
            })));
        }
        risk_config.daily_loss_limit_sol = daily_limit;
    }

    if let Some(max_dd) = request.max_drawdown_percent {
        if max_dd < 1.0 || max_dd > 100.0 {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "success": false,
                "error": "max_drawdown_percent must be between 1 and 100"
            })));
        }
        risk_config.max_drawdown_percent = max_dd;
    }

    if let Some(take_profit) = request.take_profit_percent {
        if take_profit < 1.0 || take_profit > 200.0 {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "success": false,
                "error": "take_profit_percent must be between 1 and 200"
            })));
        }
        risk_config.take_profit_percent = take_profit;
    }

    if let Some(trailing_stop) = request.trailing_stop_percent {
        if trailing_stop > 100.0 {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "success": false,
                "error": "trailing_stop_percent must be between 0 and 100"
            })));
        }
        risk_config.trailing_stop_percent = trailing_stop;
    }

    if let Some(time_limit) = request.time_limit_minutes {
        if time_limit < 1 || time_limit > 60 {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "success": false,
                "error": "time_limit_minutes must be between 1 and 60"
            })));
        }
        risk_config.time_limit_minutes = time_limit;
    }

    let max_pos = risk_config.max_position_sol;
    let daily_limit = risk_config.daily_loss_limit_sol;
    let max_dd = risk_config.max_drawdown_percent;
    let take_profit = risk_config.take_profit_percent;
    let trailing_stop = risk_config.trailing_stop_percent;
    let time_limit = risk_config.time_limit_minutes;
    drop(risk_config); // Release lock before async operations

    tracing::info!(
        "⚙️ Custom risk config updated: max_pos={} SOL, concurrent={}, daily_limit={} SOL",
        max_pos,
        request.max_concurrent_positions.unwrap_or(0),
        daily_limit
    );

    // Sync ALL active strategies with new risk config
    let strategies = state.strategy_engine.list_strategies().await;
    let max_concurrent = request.max_concurrent_positions;
    let mut synced_count = 0;
    for strategy in strategies.iter().filter(|s| s.is_active) {
        let mut updated_params = strategy.risk_params.clone();

        // Only sync fields that were explicitly provided
        if request.max_position_sol.is_some() {
            updated_params.max_position_sol = max_pos;
        }
        if request.daily_loss_limit_sol.is_some() {
            updated_params.daily_loss_limit_sol = daily_limit;
        }
        if request.max_drawdown_percent.is_some() {
            updated_params.stop_loss_percent = Some(max_dd);
        }
        if max_concurrent.is_some() {
            updated_params.concurrent_positions = max_concurrent;
        }
        if request.take_profit_percent.is_some() {
            updated_params.take_profit_percent = Some(take_profit);
        }
        if request.trailing_stop_percent.is_some() {
            updated_params.trailing_stop_percent = Some(trailing_stop);
        }
        if request.time_limit_minutes.is_some() {
            updated_params.time_limit_minutes = Some(time_limit);
        }

        // Update in-memory
        if let Err(e) = state.strategy_engine.set_risk_params(strategy.id, updated_params.clone()).await {
            tracing::warn!(strategy_id = %strategy.id, error = %e, "Failed to sync strategy risk params");
            continue;
        }

        // Persist to database
        if let Err(e) = state.strategy_repo.update(strategy.id, UpdateStrategyRecord {
            name: None,
            venue_types: None,
            execution_mode: None,
            risk_params: Some(updated_params),
            is_active: None,
        }).await {
            tracing::warn!(strategy_id = %strategy.id, error = %e, "Failed to persist synced risk params");
        }

        synced_count += 1;
    }

    tracing::info!("✅ Synced {} strategies with custom risk config", synced_count);

    let final_config = state.risk_config.read().await;
    (StatusCode::OK, Json(serde_json::json!({
        "success": true,
        "message": format!("Risk config updated - {} strategies synced", synced_count),
        "config": {
            "max_position_sol": final_config.max_position_sol,
            "max_concurrent_positions": final_config.max_concurrent_positions,
            "daily_loss_limit_sol": final_config.daily_loss_limit_sol,
            "max_drawdown_percent": final_config.max_drawdown_percent,
            "max_position_per_token_sol": final_config.max_position_per_token_sol,
            "take_profit_percent": final_config.take_profit_percent,
            "trailing_stop_percent": final_config.trailing_stop_percent,
            "time_limit_minutes": final_config.time_limit_minutes,
            "cooldown_after_loss_ms": final_config.cooldown_after_loss_ms,
            "volatility_scaling_enabled": final_config.volatility_scaling_enabled,
            "auto_pause_on_drawdown": final_config.auto_pause_on_drawdown,
        }
    })))
}
