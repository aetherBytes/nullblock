use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};
use tokio::time::Instant;
use uuid::Uuid;

use crate::events::{swarm as swarm_topics, AgentType, ArbEvent, EventSource};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Dead,
}

#[derive(Debug, Clone)]
pub struct AgentStatus {
    pub agent_type: AgentType,
    pub agent_id: Uuid,
    pub health: AgentHealth,
    pub last_heartbeat: Instant,
    pub consecutive_failures: u32,
    pub restart_count: u32,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub error_message: Option<String>,
}

impl AgentStatus {
    pub fn new(agent_type: AgentType, agent_id: Uuid) -> Self {
        Self {
            agent_type,
            agent_id,
            health: AgentHealth::Healthy,
            last_heartbeat: Instant::now(),
            consecutive_failures: 0,
            restart_count: 0,
            started_at: chrono::Utc::now(),
            error_message: None,
        }
    }

    pub fn record_heartbeat(&mut self) {
        self.last_heartbeat = Instant::now();
        self.consecutive_failures = 0;
        self.health = AgentHealth::Healthy;
        self.error_message = None;
    }

    pub fn record_failure(&mut self, error: &str) {
        self.consecutive_failures += 1;
        self.error_message = Some(error.to_string());

        self.health = match self.consecutive_failures {
            0..=2 => AgentHealth::Degraded,
            3..=5 => AgentHealth::Unhealthy,
            _ => AgentHealth::Dead,
        };
    }

    pub fn record_restart(&mut self) {
        self.restart_count += 1;
        self.consecutive_failures = 0;
        self.health = AgentHealth::Healthy;
        self.error_message = None;
        self.last_heartbeat = Instant::now();
    }

    pub fn seconds_since_heartbeat(&self) -> u64 {
        self.last_heartbeat.elapsed().as_secs()
    }
}

#[derive(Debug, Clone)]
pub struct SwarmHealth {
    pub total_agents: u32,
    pub healthy_agents: u32,
    pub degraded_agents: u32,
    pub unhealthy_agents: u32,
    pub dead_agents: u32,
    pub overall_health: AgentHealth,
    pub is_paused: bool,
}

#[derive(Debug, Clone)]
pub struct OverseerConfig {
    pub heartbeat_interval_secs: u64,
    pub heartbeat_timeout_secs: u64,
    pub max_restart_attempts: u32,
    pub restart_cooldown_secs: u64,
    pub auto_recovery_enabled: bool,
}

impl Default for OverseerConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval_secs: 10,
            heartbeat_timeout_secs: 30,
            max_restart_attempts: 3,
            restart_cooldown_secs: 60,
            auto_recovery_enabled: true,
        }
    }
}

pub struct ResilienceOverseer {
    id: Uuid,
    agents: Arc<RwLock<HashMap<Uuid, AgentStatus>>>,
    config: OverseerConfig,
    is_paused: Arc<RwLock<bool>>,
    event_tx: broadcast::Sender<ArbEvent>,
}

impl ResilienceOverseer {
    pub fn new(config: OverseerConfig, event_tx: broadcast::Sender<ArbEvent>) -> Self {
        Self {
            id: Uuid::new_v4(),
            agents: Arc::new(RwLock::new(HashMap::new())),
            config,
            is_paused: Arc::new(RwLock::new(false)),
            event_tx,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub async fn register_agent(&self, agent_type: AgentType, agent_id: Uuid) {
        let status = AgentStatus::new(agent_type.clone(), agent_id);
        self.agents.write().await.insert(agent_id, status);

        crate::events::broadcast_event(
            &self.event_tx,
            ArbEvent::new(
                "agent_started",
                EventSource::Agent(AgentType::Overseer),
                swarm_topics::AGENT_STARTED,
                serde_json::json!({
                    "agent_type": format!("{:?}", agent_type),
                    "agent_id": agent_id.to_string(),
                }),
            ),
        );

        tracing::info!("Registered agent {:?} with id {}", agent_type, agent_id);
    }

    pub async fn unregister_agent(&self, agent_id: Uuid) {
        if let Some(status) = self.agents.write().await.remove(&agent_id) {
            crate::events::broadcast_event(
                &self.event_tx,
                ArbEvent::new(
                    "agent_stopped",
                    EventSource::Agent(AgentType::Overseer),
                    swarm_topics::AGENT_STOPPED,
                    serde_json::json!({
                        "agent_type": format!("{:?}", status.agent_type),
                        "agent_id": agent_id.to_string(),
                    }),
                ),
            );

            tracing::info!(
                "Unregistered agent {:?} with id {}",
                status.agent_type,
                agent_id
            );
        }
    }

    pub async fn record_heartbeat(&self, agent_id: Uuid) {
        if let Some(status) = self.agents.write().await.get_mut(&agent_id) {
            status.record_heartbeat();
        }
    }

    pub async fn record_agent_failure(&self, agent_id: Uuid, error: &str) {
        let should_emit_event = {
            let mut agents = self.agents.write().await;
            if let Some(status) = agents.get_mut(&agent_id) {
                let was_healthy = status.health == AgentHealth::Healthy;
                status.record_failure(error);
                was_healthy && status.health != AgentHealth::Healthy
            } else {
                false
            }
        };

        if should_emit_event {
            let agents = self.agents.read().await;
            if let Some(status) = agents.get(&agent_id) {
                crate::events::broadcast_event(
                    &self.event_tx,
                    ArbEvent::new(
                        "agent_failed",
                        EventSource::Agent(AgentType::Overseer),
                        swarm_topics::AGENT_FAILED,
                        serde_json::json!({
                            "agent_type": format!("{:?}", status.agent_type),
                            "agent_id": agent_id.to_string(),
                            "health": format!("{:?}", status.health),
                            "error": error,
                            "consecutive_failures": status.consecutive_failures,
                        }),
                    ),
                );
            }
        }
    }

    pub async fn record_agent_recovery(&self, agent_id: Uuid) {
        let should_emit_event = {
            let mut agents = self.agents.write().await;
            if let Some(status) = agents.get_mut(&agent_id) {
                status.record_restart();
                true
            } else {
                false
            }
        };

        if should_emit_event {
            let agents = self.agents.read().await;
            if let Some(status) = agents.get(&agent_id) {
                crate::events::broadcast_event(
                    &self.event_tx,
                    ArbEvent::new(
                        "agent_recovered",
                        EventSource::Agent(AgentType::Overseer),
                        swarm_topics::AGENT_RECOVERED,
                        serde_json::json!({
                            "agent_type": format!("{:?}", status.agent_type),
                            "agent_id": agent_id.to_string(),
                            "restart_count": status.restart_count,
                        }),
                    ),
                );
            }
        }
    }

    pub async fn get_agent_status(&self, agent_id: Uuid) -> Option<AgentStatus> {
        self.agents.read().await.get(&agent_id).cloned()
    }

    pub async fn get_all_agent_statuses(&self) -> Vec<AgentStatus> {
        self.agents.read().await.values().cloned().collect()
    }

    pub async fn get_swarm_health(&self) -> SwarmHealth {
        let agents = self.agents.read().await;
        let is_paused = *self.is_paused.read().await;

        let mut healthy = 0;
        let mut degraded = 0;
        let mut unhealthy = 0;
        let mut dead = 0;

        for status in agents.values() {
            match status.health {
                AgentHealth::Healthy => healthy += 1,
                AgentHealth::Degraded => degraded += 1,
                AgentHealth::Unhealthy => unhealthy += 1,
                AgentHealth::Dead => dead += 1,
            }
        }

        let total = agents.len() as u32;
        let overall_health = if dead > 0 {
            AgentHealth::Dead
        } else if unhealthy > 0 {
            AgentHealth::Unhealthy
        } else if degraded > 0 {
            AgentHealth::Degraded
        } else {
            AgentHealth::Healthy
        };

        SwarmHealth {
            total_agents: total,
            healthy_agents: healthy,
            degraded_agents: degraded,
            unhealthy_agents: unhealthy,
            dead_agents: dead,
            overall_health,
            is_paused,
        }
    }

    pub async fn pause_swarm(&self) {
        *self.is_paused.write().await = true;

        crate::events::broadcast_event(
            &self.event_tx,
            ArbEvent::new(
                "swarm_paused",
                EventSource::Agent(AgentType::Overseer),
                swarm_topics::PAUSED,
                serde_json::json!({
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                }),
            ),
        );

        tracing::warn!("Swarm paused by overseer");
    }

    pub async fn resume_swarm(&self) {
        *self.is_paused.write().await = false;

        crate::events::broadcast_event(
            &self.event_tx,
            ArbEvent::new(
                "swarm_resumed",
                EventSource::Agent(AgentType::Overseer),
                swarm_topics::RESUMED,
                serde_json::json!({
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                }),
            ),
        );

        tracing::info!("Swarm resumed by overseer");
    }

    pub async fn is_paused(&self) -> bool {
        *self.is_paused.read().await
    }

    pub async fn check_heartbeats(&self) -> Vec<Uuid> {
        let timeout = Duration::from_secs(self.config.heartbeat_timeout_secs);
        let mut stale_agents = Vec::new();

        let agents = self.agents.read().await;
        for (id, status) in agents.iter() {
            if status.last_heartbeat.elapsed() > timeout {
                stale_agents.push(*id);
            }
        }

        stale_agents
    }

    pub async fn get_agents_needing_restart(&self) -> Vec<(Uuid, AgentType)> {
        if !self.config.auto_recovery_enabled {
            return Vec::new();
        }

        let agents = self.agents.read().await;
        let mut needs_restart = Vec::new();

        for (id, status) in agents.iter() {
            if status.health == AgentHealth::Dead
                && status.restart_count < self.config.max_restart_attempts
            {
                needs_restart.push((*id, status.agent_type.clone()));
            }
        }

        needs_restart
    }

    pub fn config(&self) -> &OverseerConfig {
        &self.config
    }
}

impl Clone for ResilienceOverseer {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            agents: Arc::clone(&self.agents),
            config: self.config.clone(),
            is_paused: Arc::clone(&self.is_paused),
            event_tx: self.event_tx.clone(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct OverseerStats {
    pub total_agents_registered: u64,
    pub total_restarts: u64,
    pub total_failures: u64,
    pub uptime_secs: u64,
    pub last_health_check: Option<chrono::DateTime<chrono::Utc>>,
}
