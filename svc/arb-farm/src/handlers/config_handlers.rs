use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::server::AppState;

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
}

#[derive(Debug, Serialize)]
pub struct RiskLevelParams {
    pub level: String,
    pub max_position_sol: f64,
    pub max_concurrent_positions: u32,
    pub max_liquidity_contribution_pct: f64,
    pub stop_loss_pct: f64,
    pub take_profit_pct: f64,
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
            stop_loss_pct: 10.0,
            take_profit_pct: 20.0,
            daily_loss_limit_sol: 0.1,
        },
        "high" => RiskLevelParams {
            level: "high".to_string(),
            max_position_sol: 1.0,
            max_concurrent_positions: 20,
            max_liquidity_contribution_pct: 20.0,
            stop_loss_pct: 30.0,
            take_profit_pct: 75.0,
            daily_loss_limit_sol: 5.0,
        },
        _ => RiskLevelParams {
            level: "medium".to_string(),
            max_position_sol: 0.25,
            max_concurrent_positions: 10,
            max_liquidity_contribution_pct: 10.0,
            stop_loss_pct: 15.0,
            take_profit_pct: 50.0,
            daily_loss_limit_sol: 1.0,
        },
    };

    // Update the shared risk config
    {
        let mut risk_config = state.risk_config.write().await;
        risk_config.max_position_sol = params.max_position_sol;
        risk_config.max_concurrent_positions = params.max_concurrent_positions;
        risk_config.daily_loss_limit_sol = params.daily_loss_limit_sol;
        risk_config.max_drawdown_percent = params.stop_loss_pct;
    }

    tracing::info!(
        "⚙️ Risk level set to {}: max_pos={} SOL, concurrent={}, SL={}%",
        params.level,
        params.max_position_sol,
        params.max_concurrent_positions,
        params.stop_loss_pct
    );

    (StatusCode::OK, Json(serde_json::json!({
        "success": true,
        "message": format!("Risk level set to {}", params.level),
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

    tracing::info!(
        "⚙️ Custom risk config updated: max_pos={} SOL, concurrent={}, daily_limit={} SOL",
        risk_config.max_position_sol,
        risk_config.max_concurrent_positions,
        risk_config.daily_loss_limit_sol
    );

    (StatusCode::OK, Json(serde_json::json!({
        "success": true,
        "message": "Risk config updated",
        "config": {
            "max_position_sol": risk_config.max_position_sol,
            "max_concurrent_positions": risk_config.max_concurrent_positions,
            "daily_loss_limit_sol": risk_config.daily_loss_limit_sol,
            "max_drawdown_percent": risk_config.max_drawdown_percent,
            "max_position_per_token_sol": risk_config.max_position_per_token_sol,
        }
    })))
}
