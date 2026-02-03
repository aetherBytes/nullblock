use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::database::repositories::strategies::UpdateStrategyRecord;
use crate::execution::risk::RiskConfig;
use crate::server::AppState;

#[derive(Debug, Serialize)]
pub struct RiskSettingsResponse {
    pub config: RiskConfigDto,
    pub presets: Vec<RiskPreset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConfigDto {
    pub max_position_sol: f64,
    pub daily_loss_limit_sol: f64,
    pub max_drawdown_percent: f64,
    pub max_concurrent_positions: u32,
    pub max_position_per_token_sol: f64,
    pub cooldown_after_loss_ms: u64,
    pub volatility_scaling_enabled: bool,
    pub auto_pause_on_drawdown: bool,
    #[serde(default = "default_take_profit")]
    pub take_profit_percent: f64,
    #[serde(default = "default_trailing_stop")]
    pub trailing_stop_percent: f64,
    #[serde(default = "default_time_limit")]
    pub time_limit_minutes: u32,
}

fn default_take_profit() -> f64 {
    15.0
}
fn default_trailing_stop() -> f64 {
    12.0
}
fn default_time_limit() -> u32 {
    7
}

impl From<RiskConfig> for RiskConfigDto {
    fn from(config: RiskConfig) -> Self {
        Self {
            max_position_sol: config.max_position_sol,
            daily_loss_limit_sol: config.daily_loss_limit_sol,
            max_drawdown_percent: config.max_drawdown_percent,
            max_concurrent_positions: config.max_concurrent_positions,
            max_position_per_token_sol: config.max_position_per_token_sol,
            cooldown_after_loss_ms: config.cooldown_after_loss_ms,
            volatility_scaling_enabled: config.volatility_scaling_enabled,
            auto_pause_on_drawdown: config.auto_pause_on_drawdown,
            take_profit_percent: config.take_profit_percent,
            trailing_stop_percent: config.trailing_stop_percent,
            time_limit_minutes: config.time_limit_minutes,
        }
    }
}

impl From<RiskConfigDto> for RiskConfig {
    fn from(dto: RiskConfigDto) -> Self {
        Self {
            max_position_sol: dto.max_position_sol,
            daily_loss_limit_sol: dto.daily_loss_limit_sol,
            max_drawdown_percent: dto.max_drawdown_percent,
            max_concurrent_positions: dto.max_concurrent_positions,
            max_position_per_token_sol: dto.max_position_per_token_sol,
            cooldown_after_loss_ms: dto.cooldown_after_loss_ms,
            volatility_scaling_enabled: dto.volatility_scaling_enabled,
            auto_pause_on_drawdown: dto.auto_pause_on_drawdown,
            take_profit_percent: dto.take_profit_percent,
            trailing_stop_percent: dto.trailing_stop_percent,
            time_limit_minutes: dto.time_limit_minutes,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RiskPreset {
    pub name: String,
    pub description: String,
    pub config: RiskConfigDto,
}

fn get_risk_presets() -> Vec<RiskPreset> {
    vec![
        RiskPreset {
            name: "low".to_string(),
            description: "Conservative. SL: 15%, TP: 10%, Time: 5min".to_string(),
            config: RiskConfigDto {
                max_position_sol: 0.02,
                daily_loss_limit_sol: 0.1,
                max_drawdown_percent: 15.0,
                max_concurrent_positions: 2,
                max_position_per_token_sol: 0.02,
                cooldown_after_loss_ms: 10000,
                volatility_scaling_enabled: true,
                auto_pause_on_drawdown: true,
                take_profit_percent: 10.0,
                trailing_stop_percent: 8.0,
                time_limit_minutes: 5,
            },
        },
        RiskPreset {
            name: "medium".to_string(),
            description: "DEFENSIVE (DEFAULT). SL: 10%, TP: 15% (strong momentum extends), Trail: 8%, Time: 5min".to_string(),
            config: RiskConfigDto {
                max_position_sol: 0.3,
                daily_loss_limit_sol: 1.0,
                max_drawdown_percent: 10.0,     // DEFENSIVE: 10% tight stop
                max_concurrent_positions: 10,
                max_position_per_token_sol: 0.3,
                cooldown_after_loss_ms: 3000,
                volatility_scaling_enabled: true,
                auto_pause_on_drawdown: true,
                take_profit_percent: 15.0,      // DEFENSIVE: 15% TP
                trailing_stop_percent: 8.0,     // DEFENSIVE: 8% trailing
                time_limit_minutes: 5,          // DEFENSIVE: 5 min
            },
        },
        RiskPreset {
            name: "conservative".to_string(),
            description: "Larger positions, tight exits. SL: 15%, TP: 12%, Time: 5min".to_string(),
            config: RiskConfigDto {
                max_position_sol: 1.0,
                daily_loss_limit_sol: 0.5,
                max_drawdown_percent: 15.0,
                max_concurrent_positions: 3,
                max_position_per_token_sol: 0.5,
                cooldown_after_loss_ms: 10000,
                volatility_scaling_enabled: true,
                auto_pause_on_drawdown: true,
                take_profit_percent: 12.0,
                trailing_stop_percent: 10.0,
                time_limit_minutes: 5,
            },
        },
        RiskPreset {
            name: "dev_testing".to_string(),
            description: "Dev testing. SL: 25%, TP: 20%, Time: 10min".to_string(),
            config: RiskConfigDto {
                max_position_sol: 5.0,
                daily_loss_limit_sol: 2.0,
                max_drawdown_percent: 25.0,
                max_concurrent_positions: 10,
                max_position_per_token_sol: 2.0,
                cooldown_after_loss_ms: 2000,
                volatility_scaling_enabled: true,
                auto_pause_on_drawdown: false,
                take_profit_percent: 20.0,
                trailing_stop_percent: 15.0,
                time_limit_minutes: 10,
            },
        },
        RiskPreset {
            name: "aggressive".to_string(),
            description: "High risk. SL: 25%, TP: 20%, Time: 10min".to_string(),
            config: RiskConfigDto {
                max_position_sol: 10.0,
                daily_loss_limit_sol: 5.0,
                max_drawdown_percent: 25.0,
                max_concurrent_positions: 20,
                max_position_per_token_sol: 5.0,
                cooldown_after_loss_ms: 1000,
                volatility_scaling_enabled: false,
                auto_pause_on_drawdown: false,
                take_profit_percent: 20.0,
                trailing_stop_percent: 15.0,
                time_limit_minutes: 10,
            },
        },
    ]
}

pub async fn get_risk_settings(State(state): State<AppState>) -> impl IntoResponse {
    let config = state.risk_config.read().await;
    let config_dto = RiskConfigDto::from(config.clone());

    (
        StatusCode::OK,
        Json(RiskSettingsResponse {
            config: config_dto,
            presets: get_risk_presets(),
        }),
    )
}

#[derive(Debug, Deserialize)]
pub struct UpdateRiskSettingsRequest {
    pub preset: Option<String>,
    pub custom: Option<RiskConfigDto>,
}

pub async fn update_risk_settings(
    State(state): State<AppState>,
    Json(request): Json<UpdateRiskSettingsRequest>,
) -> impl IntoResponse {
    // Preserve current dynamic max_position_sol (set based on wallet balance at startup)
    let current_max_position = {
        let config = state.risk_config.read().await;
        config.max_position_sol
    };

    let mut new_config: RiskConfig = if let Some(ref preset_name) = request.preset {
        let presets = get_risk_presets();
        match presets.iter().find(|p| &p.name == preset_name) {
            Some(preset) => preset.config.clone().into(),
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "success": false,
                        "error": format!("Unknown preset: {}", preset_name),
                    })),
                );
            }
        }
    } else if let Some(custom) = request.custom {
        custom.into()
    } else {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "success": false,
                "error": "Must provide either preset or custom config",
            })),
        );
    };

    let wallet_max = *state.wallet_max_position_sol.read().await;
    if request.preset.is_some() {
        new_config.max_position_sol = current_max_position;
        new_config.max_position_per_token_sol = current_max_position;
    } else {
        let capped = new_config.max_position_sol.min(wallet_max);
        if capped < new_config.max_position_sol {
            tracing::warn!(
                requested = new_config.max_position_sol,
                capped = capped,
                wallet_max = wallet_max,
                "Custom max_position_sol capped at wallet-based limit"
            );
        }
        new_config.max_position_sol = capped;
        new_config.max_position_per_token_sol = capped;
    }

    {
        let mut config = state.risk_config.write().await;
        *config = new_config.clone();
    }

    let config_dto = RiskConfigDto::from(new_config.clone());

    tracing::info!(
        "⚙️ Risk settings updated: max_pos={} SOL, SL={}%, TP={}%, Trail={}%, Time={}min",
        new_config.max_position_sol,
        new_config.max_drawdown_percent,
        new_config.take_profit_percent,
        new_config.trailing_stop_percent,
        new_config.time_limit_minutes
    );

    // Sync strategies with global risk config
    // NOTE: curve_arb and graduation_snipe have strategy-specific exit params - only sync position sizing
    let strategies = state.strategy_engine.list_strategies().await;
    let mut synced_count = 0;
    for strategy in &strategies {
        let mut updated_params = strategy.risk_params.clone();
        updated_params.max_position_sol = new_config.max_position_sol;
        updated_params.daily_loss_limit_sol = new_config.daily_loss_limit_sol;
        updated_params.concurrent_positions = Some(new_config.max_concurrent_positions);

        // Only sync exit params for non-curve strategies
        // curve_arb and graduation_snipe have their own strategy-specific exit configs
        let is_curve_strategy =
            strategy.strategy_type == "curve_arb" || strategy.strategy_type == "graduation_snipe";
        if !is_curve_strategy {
            updated_params.stop_loss_percent = Some(new_config.max_drawdown_percent);
            updated_params.take_profit_percent = Some(new_config.take_profit_percent);
            updated_params.trailing_stop_percent = Some(new_config.trailing_stop_percent);
            updated_params.time_limit_minutes = Some(new_config.time_limit_minutes);
        }

        if let Err(e) = state
            .strategy_engine
            .set_risk_params(strategy.id, updated_params.clone())
            .await
        {
            tracing::warn!(strategy_id = %strategy.id, error = %e, "Failed to sync strategy risk params");
            continue;
        }

        if let Err(e) = state
            .strategy_repo
            .update(
                strategy.id,
                UpdateStrategyRecord {
                    name: None,
                    venue_types: None,
                    execution_mode: None,
                    risk_params: Some(updated_params),
                    is_active: None,
                },
            )
            .await
        {
            tracing::warn!(strategy_id = %strategy.id, error = %e, "Failed to persist synced risk params");
        }

        synced_count += 1;
    }

    tracing::info!(
        "✅ Synced {} strategies with new risk settings (curve strategies preserve exit params)",
        synced_count
    );

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "config": config_dto,
            "synced_strategies": synced_count,
        })),
    )
}

#[derive(Debug, Serialize)]
pub struct VenueSettingsResponse {
    pub venues: Vec<VenueConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VenueConfig {
    pub name: String,
    pub venue_type: String,
    pub enabled: bool,
    pub api_url: String,
    pub has_api_key: bool,
}

pub async fn get_venue_settings(State(state): State<AppState>) -> impl IntoResponse {
    let venues = vec![
        VenueConfig {
            name: "Jupiter".to_string(),
            venue_type: "dex".to_string(),
            enabled: true,
            api_url: state.config.jupiter_api_url.clone(),
            has_api_key: false,
        },
        VenueConfig {
            name: "Raydium".to_string(),
            venue_type: "dex".to_string(),
            enabled: true,
            api_url: state.config.raydium_api_url.clone(),
            has_api_key: false,
        },
        VenueConfig {
            name: "pump.fun".to_string(),
            venue_type: "curve".to_string(),
            enabled: true,
            api_url: state.config.pump_fun_api_url.clone(),
            has_api_key: false,
        },
        VenueConfig {
            name: "Moonshot".to_string(),
            venue_type: "curve".to_string(),
            enabled: true,
            api_url: state.config.moonshot_api_url.clone(),
            has_api_key: false,
        },
        VenueConfig {
            name: "Marginfi".to_string(),
            venue_type: "lending".to_string(),
            enabled: false, // Not fully implemented yet
            api_url: state.config.marginfi_api_url.clone(),
            has_api_key: false,
        },
        VenueConfig {
            name: "Kamino".to_string(),
            venue_type: "lending".to_string(),
            enabled: false, // Not fully implemented yet
            api_url: state.config.kamino_api_url.clone(),
            has_api_key: false,
        },
    ];

    (StatusCode::OK, Json(VenueSettingsResponse { venues }))
}

#[derive(Debug, Serialize)]
pub struct ApiKeyStatusResponse {
    pub services: Vec<ApiKeyStatus>,
}

#[derive(Debug, Serialize)]
pub struct ApiKeyStatus {
    pub name: String,
    pub configured: bool,
    pub required: bool,
}

pub async fn get_api_key_status(State(state): State<AppState>) -> impl IntoResponse {
    let services = vec![
        ApiKeyStatus {
            name: "Helius".to_string(),
            configured: state.config.helius_api_key.is_some(),
            required: false,
        },
        ApiKeyStatus {
            name: "Birdeye".to_string(),
            configured: state.config.birdeye_api_key.is_some(),
            required: false,
        },
        ApiKeyStatus {
            name: "OpenRouter".to_string(),
            configured: state.config.openrouter_api_key.is_some(),
            required: false,
        },
        ApiKeyStatus {
            name: "Turnkey".to_string(),
            configured: state.config.turnkey_api_public_key.is_some(),
            required: true,
        },
    ];

    (StatusCode::OK, Json(ApiKeyStatusResponse { services }))
}

#[derive(Debug, Serialize)]
pub struct AllSettingsResponse {
    pub risk: RiskConfigDto,
    pub risk_presets: Vec<RiskPreset>,
    pub venues: Vec<VenueConfig>,
    pub api_keys: Vec<ApiKeyStatus>,
    pub wallet_connected: bool,
}

pub async fn get_all_settings(State(state): State<AppState>) -> impl IntoResponse {
    let risk_config = state.risk_config.read().await;
    let wallet_status = state.turnkey_signer.get_status().await;

    let venues = vec![
        VenueConfig {
            name: "Jupiter".to_string(),
            venue_type: "dex".to_string(),
            enabled: true,
            api_url: state.config.jupiter_api_url.clone(),
            has_api_key: false,
        },
        VenueConfig {
            name: "Raydium".to_string(),
            venue_type: "dex".to_string(),
            enabled: true,
            api_url: state.config.raydium_api_url.clone(),
            has_api_key: false,
        },
        VenueConfig {
            name: "pump.fun".to_string(),
            venue_type: "curve".to_string(),
            enabled: true,
            api_url: state.config.pump_fun_api_url.clone(),
            has_api_key: false,
        },
        VenueConfig {
            name: "Moonshot".to_string(),
            venue_type: "curve".to_string(),
            enabled: true,
            api_url: state.config.moonshot_api_url.clone(),
            has_api_key: false,
        },
    ];

    let api_keys = vec![
        ApiKeyStatus {
            name: "Helius".to_string(),
            configured: state.config.helius_api_key.is_some(),
            required: false,
        },
        ApiKeyStatus {
            name: "Birdeye".to_string(),
            configured: state.config.birdeye_api_key.is_some(),
            required: false,
        },
        ApiKeyStatus {
            name: "OpenRouter".to_string(),
            configured: state.config.openrouter_api_key.is_some(),
            required: false,
        },
        ApiKeyStatus {
            name: "Turnkey".to_string(),
            configured: state.config.turnkey_api_public_key.is_some(),
            required: true,
        },
    ];

    (
        StatusCode::OK,
        Json(AllSettingsResponse {
            risk: RiskConfigDto::from(risk_config.clone()),
            risk_presets: get_risk_presets(),
            venues,
            api_keys,
            wallet_connected: wallet_status.is_connected,
        }),
    )
}
