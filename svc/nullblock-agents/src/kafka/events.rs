use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskLifecycleEvent {
    pub event_type: TaskEventType,
    pub task_id: String,
    pub user_id: Option<Uuid>,
    pub agent_id: Option<Uuid>,
    pub task_name: String,
    pub status: String,
    pub priority: String,
    pub progress: u8,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskEventType {
    TaskCreated,
    TaskStarted,
    TaskProgress,
    TaskPaused,
    TaskResumed,
    TaskCompleted,
    TaskFailed,
    TaskCancelled,
    TaskRetried,
    TaskDeleted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatusEvent {
    pub event_type: AgentEventType,
    pub agent_id: Uuid,
    pub agent_name: String,
    pub agent_type: String,
    pub status: String,
    pub health_status: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentEventType {
    AgentRegistered,
    AgentStatusChanged,
    AgentHealthCheck,
    AgentPerformanceUpdate,
    AgentDeregistered,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserEvent {
    pub event_type: UserEventType,
    pub user_id: Uuid,
    pub wallet_address: Option<String>,
    pub chain: Option<String>,
    pub user_type: String,
    pub email: Option<String>,
    pub metadata: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub erebus_created_at: Option<DateTime<Utc>>,
    pub erebus_updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserEventType {
    UserCreated,
    UserUpdated,
    UserDeleted,
}

impl TaskLifecycleEvent {
    pub fn task_created(
        task_id: String,
        user_id: Option<Uuid>,
        task_name: String,
        status: String,
        priority: String,
    ) -> Self {
        Self {
            event_type: TaskEventType::TaskCreated,
            task_id,
            user_id,
            agent_id: None,
            task_name,
            status,
            priority,
            progress: 0,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    pub fn task_status_changed(
        task_id: String,
        user_id: Option<Uuid>,
        agent_id: Option<Uuid>,
        task_name: String,
        old_status: String,
        new_status: String,
        priority: String,
        progress: u8,
    ) -> Self {
        let event_type = match new_status.as_str() {
            "running" => TaskEventType::TaskStarted,
            "paused" => TaskEventType::TaskPaused,
            "completed" => TaskEventType::TaskCompleted,
            "failed" => TaskEventType::TaskFailed,
            "cancelled" => TaskEventType::TaskCancelled,
            _ => TaskEventType::TaskProgress,
        };

        let mut metadata = HashMap::new();
        metadata.insert("old_status".to_string(), serde_json::Value::String(old_status));

        Self {
            event_type,
            task_id,
            user_id,
            agent_id,
            task_name,
            status: new_status,
            priority,
            progress,
            timestamp: Utc::now(),
            metadata,
        }
    }

    pub fn task_deleted(task_id: String, user_id: Option<Uuid>, task_name: String) -> Self {
        Self {
            event_type: TaskEventType::TaskDeleted,
            task_id,
            user_id,
            agent_id: None,
            task_name,
            status: "deleted".to_string(),
            priority: "medium".to_string(),
            progress: 0,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }
}

impl AgentStatusEvent {
    pub fn agent_registered(agent_id: Uuid, agent_name: String, agent_type: String) -> Self {
        Self {
            event_type: AgentEventType::AgentRegistered,
            agent_id,
            agent_name,
            agent_type,
            status: "active".to_string(),
            health_status: "unknown".to_string(),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    pub fn agent_health_check(
        agent_id: Uuid,
        agent_name: String,
        agent_type: String,
        health_status: String,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            event_type: AgentEventType::AgentHealthCheck,
            agent_id,
            agent_name,
            agent_type,
            status: "active".to_string(),
            health_status,
            timestamp: Utc::now(),
            metadata,
        }
    }
}