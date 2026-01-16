use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::server::AppState;
use crate::execution::risk::RiskConfig;

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
            name: "conservative".to_string(),
            description: "Low risk, suitable for testing. Max 1 SOL position, 0.5 SOL daily loss limit.".to_string(),
            config: RiskConfigDto {
                max_position_sol: 1.0,
                daily_loss_limit_sol: 0.5,
                max_drawdown_percent: 10.0,
                max_concurrent_positions: 3,
                max_position_per_token_sol: 0.5,
                cooldown_after_loss_ms: 10000,
                volatility_scaling_enabled: true,
                auto_pause_on_drawdown: true,
            },
        },
        RiskPreset {
            name: "dev_testing".to_string(),
            description: "Development testing profile. Max 5 SOL position, 2 SOL daily loss limit.".to_string(),
            config: RiskConfigDto {
                max_position_sol: 5.0,
                daily_loss_limit_sol: 2.0,
                max_drawdown_percent: 40.0,
                max_concurrent_positions: 10,
                max_position_per_token_sol: 2.0,
                cooldown_after_loss_ms: 2000,
                volatility_scaling_enabled: true,
                auto_pause_on_drawdown: false,
            },
        },
        RiskPreset {
            name: "aggressive".to_string(),
            description: "Higher risk for experienced traders. Max 10 SOL position, 5 SOL daily loss limit.".to_string(),
            config: RiskConfigDto {
                max_position_sol: 10.0,
                daily_loss_limit_sol: 5.0,
                max_drawdown_percent: 50.0,
                max_concurrent_positions: 20,
                max_position_per_token_sol: 5.0,
                cooldown_after_loss_ms: 1000,
                volatility_scaling_enabled: false,
                auto_pause_on_drawdown: false,
            },
        },
    ]
}

pub async fn get_risk_settings(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let config = state.risk_config.read().await;
    let config_dto = RiskConfigDto::from(config.clone());

    (StatusCode::OK, Json(RiskSettingsResponse {
        config: config_dto,
        presets: get_risk_presets(),
    }))
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
    let new_config = if let Some(preset_name) = request.preset {
        // Find preset by name
        let presets = get_risk_presets();
        match presets.iter().find(|p| p.name == preset_name) {
            Some(preset) => preset.config.clone().into(),
            None => {
                return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                    "success": false,
                    "error": format!("Unknown preset: {}", preset_name),
                })));
            }
        }
    } else if let Some(custom) = request.custom {
        custom.into()
    } else {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "success": false,
            "error": "Must provide either preset or custom config",
        })));
    };

    let mut config = state.risk_config.write().await;
    *config = new_config;

    let config_dto = RiskConfigDto::from(config.clone());

    (StatusCode::OK, Json(serde_json::json!({
        "success": true,
        "config": config_dto,
    })))
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

pub async fn get_venue_settings(
    State(state): State<AppState>,
) -> impl IntoResponse {
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

pub async fn get_api_key_status(
    State(state): State<AppState>,
) -> impl IntoResponse {
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

pub async fn get_all_settings(
    State(state): State<AppState>,
) -> impl IntoResponse {
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

    (StatusCode::OK, Json(AllSettingsResponse {
        risk: RiskConfigDto::from(risk_config.clone()),
        risk_presets: get_risk_presets(),
        venues,
        api_keys,
        wallet_connected: wallet_status.is_connected,
    }))
}
