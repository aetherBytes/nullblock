use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::agents::{AgentHealth, AgentStatus, ResilienceOverseer, SwarmHealth};
use crate::resilience::{CircuitBreakerRegistry, CircuitState};
use crate::server::AppState;

lazy_static::lazy_static! {
    static ref OVERSEER: std::sync::RwLock<Option<ResilienceOverseer>> = std::sync::RwLock::new(None);
    static ref CIRCUIT_BREAKERS: std::sync::RwLock<Option<CircuitBreakerRegistry>> = std::sync::RwLock::new(None);
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

pub fn init_overseer(overseer: ResilienceOverseer) {
    let mut guard = OVERSEER.write().unwrap();
    *guard = Some(overseer);
}

pub fn init_circuit_breakers(registry: CircuitBreakerRegistry) {
    let mut guard = CIRCUIT_BREAKERS.write().unwrap();
    *guard = Some(registry);
}

fn get_overseer_clone() -> Result<ResilienceOverseer, ErrorResponse> {
    let guard = OVERSEER
        .read()
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "Lock poisoned"))?;
    guard
        .as_ref()
        .cloned()
        .ok_or_else(|| json_error(StatusCode::SERVICE_UNAVAILABLE, "Overseer not initialized"))
}

fn get_circuit_breakers_clone() -> Result<CircuitBreakerRegistry, ErrorResponse> {
    let guard = CIRCUIT_BREAKERS
        .read()
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "Lock poisoned"))?;
    guard
        .as_ref()
        .cloned()
        .ok_or_else(|| json_error(StatusCode::SERVICE_UNAVAILABLE, "Circuit breakers not initialized"))
}

#[derive(Debug, Serialize)]
pub struct SwarmHealthResponse {
    pub total_agents: u32,
    pub healthy_agents: u32,
    pub degraded_agents: u32,
    pub unhealthy_agents: u32,
    pub dead_agents: u32,
    pub overall_health: String,
    pub is_paused: bool,
}

impl From<SwarmHealth> for SwarmHealthResponse {
    fn from(health: SwarmHealth) -> Self {
        Self {
            total_agents: health.total_agents,
            healthy_agents: health.healthy_agents,
            degraded_agents: health.degraded_agents,
            unhealthy_agents: health.unhealthy_agents,
            dead_agents: health.dead_agents,
            overall_health: format!("{:?}", health.overall_health),
            is_paused: health.is_paused,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AgentStatusResponse {
    pub agent_type: String,
    pub agent_id: String,
    pub health: String,
    pub seconds_since_heartbeat: u64,
    pub consecutive_failures: u32,
    pub restart_count: u32,
    pub started_at: String,
    pub error_message: Option<String>,
}

impl From<AgentStatus> for AgentStatusResponse {
    fn from(status: AgentStatus) -> Self {
        Self {
            agent_type: format!("{:?}", status.agent_type),
            agent_id: status.agent_id.to_string(),
            health: format!("{:?}", status.health),
            seconds_since_heartbeat: status.seconds_since_heartbeat(),
            consecutive_failures: status.consecutive_failures,
            restart_count: status.restart_count,
            started_at: status.started_at.to_rfc3339(),
            error_message: status.error_message,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CircuitBreakerStatusResponse {
    pub name: String,
    pub state: String,
}

#[derive(Debug, Serialize)]
pub struct SwarmStatusResponse {
    pub health: SwarmHealthResponse,
    pub agents: Vec<AgentStatusResponse>,
    pub circuit_breakers: Vec<CircuitBreakerStatusResponse>,
}

pub async fn get_swarm_status(
    State(_state): State<AppState>,
) -> Result<Json<SwarmStatusResponse>, ErrorResponse> {
    let overseer = get_overseer_clone()?;
    let circuit_breakers = get_circuit_breakers_clone().ok();

    let health = overseer.get_swarm_health().await;
    let agents = overseer.get_all_agent_statuses().await;

    let breaker_states = if let Some(registry) = circuit_breakers {
        let states = registry.get_all_states().await;
        states
            .into_iter()
            .map(|(name, state)| CircuitBreakerStatusResponse {
                name,
                state: format!("{:?}", state),
            })
            .collect()
    } else {
        Vec::new()
    };

    Ok(Json(SwarmStatusResponse {
        health: health.into(),
        agents: agents.into_iter().map(Into::into).collect(),
        circuit_breakers: breaker_states,
    }))
}

pub async fn get_swarm_health(
    State(_state): State<AppState>,
) -> Result<Json<SwarmHealthResponse>, ErrorResponse> {
    let overseer = get_overseer_clone()?;
    let health = overseer.get_swarm_health().await;
    Ok(Json(health.into()))
}

pub async fn list_agents(
    State(_state): State<AppState>,
) -> Result<Json<Vec<AgentStatusResponse>>, ErrorResponse> {
    let overseer = get_overseer_clone()?;
    let agents = overseer.get_all_agent_statuses().await;
    Ok(Json(agents.into_iter().map(Into::into).collect()))
}

pub async fn get_agent_status(
    State(_state): State<AppState>,
    Path(agent_id): Path<String>,
) -> Result<Json<AgentStatusResponse>, ErrorResponse> {
    let overseer = get_overseer_clone()?;

    let id = Uuid::parse_str(&agent_id)
        .map_err(|_| json_error(StatusCode::BAD_REQUEST, "Invalid agent ID"))?;

    match overseer.get_agent_status(id).await {
        Some(status) => Ok(Json(status.into())),
        None => Err(json_error(StatusCode::NOT_FOUND, &format!("Agent not found: {}", agent_id))),
    }
}

pub async fn pause_swarm(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let overseer = get_overseer_clone()?;
    overseer.pause_swarm().await;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Swarm paused"
    })))
}

pub async fn resume_swarm(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let overseer = get_overseer_clone()?;
    overseer.resume_swarm().await;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Swarm resumed"
    })))
}

#[derive(Debug, Deserialize)]
pub struct HeartbeatRequest {
    pub agent_id: String,
}

pub async fn record_heartbeat(
    State(_state): State<AppState>,
    Json(request): Json<HeartbeatRequest>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let overseer = get_overseer_clone()?;

    let id = Uuid::parse_str(&request.agent_id)
        .map_err(|_| json_error(StatusCode::BAD_REQUEST, "Invalid agent ID"))?;

    overseer.record_heartbeat(id).await;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Heartbeat recorded"
    })))
}

#[derive(Debug, Deserialize)]
pub struct ReportFailureRequest {
    pub agent_id: String,
    pub error: String,
}

pub async fn report_failure(
    State(_state): State<AppState>,
    Json(request): Json<ReportFailureRequest>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let overseer = get_overseer_clone()?;

    let id = Uuid::parse_str(&request.agent_id)
        .map_err(|_| json_error(StatusCode::BAD_REQUEST, "Invalid agent ID"))?;

    overseer.record_agent_failure(id, &request.error).await;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Failure recorded"
    })))
}

pub async fn list_circuit_breakers(
    State(_state): State<AppState>,
) -> Result<Json<Vec<CircuitBreakerStatusResponse>>, ErrorResponse> {
    let registry = get_circuit_breakers_clone()?;
    let states = registry.get_all_states().await;

    let breakers: Vec<CircuitBreakerStatusResponse> = states
        .into_iter()
        .map(|(name, state)| CircuitBreakerStatusResponse {
            name,
            state: format!("{:?}", state),
        })
        .collect();

    Ok(Json(breakers))
}

pub async fn reset_circuit_breaker(
    State(_state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let registry = get_circuit_breakers_clone()?;

    if let Some(breaker) = registry.get(&name).await {
        breaker.reset().await;
        Ok(Json(serde_json::json!({
            "success": true,
            "message": format!("Circuit breaker '{}' reset", name)
        })))
    } else {
        Err(json_error(StatusCode::NOT_FOUND, &format!("Circuit breaker not found: {}", name)))
    }
}

pub async fn reset_all_circuit_breakers(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let registry = get_circuit_breakers_clone()?;
    registry.reset_all().await;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "All circuit breakers reset"
    })))
}
