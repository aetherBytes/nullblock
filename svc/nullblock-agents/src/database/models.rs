use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
// Future model imports for when other features are implemented
// use crate::models::{TaskStatus, TaskPriority, TaskType, TaskCategory, TaskOutcome};

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct TaskEntity {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub task_type: String,
    pub category: String,

    // A2A Protocol required fields
    pub context_id: Uuid,
    pub kind: String,
    pub status: String,
    pub status_message: Option<String>,
    pub status_timestamp: Option<DateTime<Utc>>,

    // A2A Protocol optional fields
    pub history: serde_json::Value,
    pub artifacts: serde_json::Value,

    pub priority: String,
    pub user_id: Option<Uuid>,
    pub assigned_agent_id: Option<Uuid>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,

    pub progress: i16,
    pub estimated_duration: Option<i64>,
    pub actual_duration: Option<i64>,

    pub sub_tasks: serde_json::Value,
    pub dependencies: serde_json::Value,
    pub context: serde_json::Value,
    pub parameters: serde_json::Value,
    pub outcome: Option<serde_json::Value>,
    pub logs: serde_json::Value,
    pub triggers: serde_json::Value,
    pub required_capabilities: serde_json::Value,

    pub auto_retry: bool,
    pub max_retries: i32,
    pub current_retries: i32,
    pub user_approval_required: bool,
    pub user_notifications: bool,

    // Action tracking fields
    pub actioned_at: Option<DateTime<Utc>>,
    pub action_result: Option<String>,
    pub action_metadata: serde_json::Value,
    pub action_duration: Option<i64>,

    // Source tracking fields
    pub source_identifier: Option<String>,
    pub source_metadata: serde_json::Value,
}

impl TaskEntity {
    pub fn to_domain_model(self) -> Result<crate::models::Task, serde_json::Error> {
        let task_state: crate::models::TaskState = serde_json::from_str(&format!("\"{}\"", self.status))?;

        Ok(crate::models::Task {
            id: self.id.to_string(),
            name: self.name,
            description: self.description.unwrap_or_default(),
            task_type: serde_json::from_str(&format!("\"{}\"", self.task_type))?,
            category: serde_json::from_str(&format!("\"{}\"", self.category))?,

            context_id: self.context_id.to_string(),
            kind: self.kind,
            status: crate::models::TaskStatus {
                state: task_state,
                message: self.status_message,
                timestamp: self.status_timestamp.map(|ts| ts.to_rfc3339()),
            },
            history: if self.history == serde_json::json!([]) {
                None
            } else {
                Some(serde_json::from_value(self.history)?)
            },
            artifacts: if self.artifacts == serde_json::json!([]) {
                None
            } else {
                Some(serde_json::from_value(self.artifacts)?)
            },
            metadata: None,

            priority: serde_json::from_str(&format!("\"{}\"", self.priority))?,
            created_at: self.created_at,
            updated_at: self.updated_at,
            started_at: self.started_at,
            completed_at: self.completed_at,
            progress: self.progress as u8,
            estimated_duration: self.estimated_duration.map(|d| d as u64),
            actual_duration: self.actual_duration.map(|d| d as u64),
            sub_tasks: serde_json::from_value(self.sub_tasks)?,
            dependencies: serde_json::from_value(self.dependencies)?,
            context: serde_json::from_value(self.context)?,
            parameters: serde_json::from_value(self.parameters)?,
            outcome: if let Some(outcome_val) = self.outcome {
                Some(serde_json::from_value(outcome_val)?)
            } else {
                None
            },
            logs: serde_json::from_value(self.logs)?,
            triggers: serde_json::from_value(self.triggers)?,
            assigned_agent: None,
            auto_retry: self.auto_retry,
            max_retries: self.max_retries as u32,
            current_retries: self.current_retries as u32,
            required_capabilities: serde_json::from_value(self.required_capabilities)?,
            user_approval_required: self.user_approval_required,
            user_notifications: self.user_notifications,

            actioned_at: self.actioned_at,
            action_result: self.action_result,
            action_metadata: serde_json::from_value(self.action_metadata)?,
            action_duration: self.action_duration.map(|d| d as u64),

            source_identifier: self.source_identifier,
            source_metadata: serde_json::from_value(self.source_metadata)?,
        })
    }

    pub fn from_domain_model(task: &crate::models::Task, user_id: Option<Uuid>, assigned_agent_id: Option<Uuid>) -> Result<TaskEntity, serde_json::Error> {
        Ok(TaskEntity {
            id: Uuid::parse_str(&task.id).unwrap_or_else(|_| Uuid::new_v4()),
            name: task.name.clone(),
            description: if task.description.is_empty() { None } else { Some(task.description.clone()) },
            task_type: serde_json::to_string(&task.task_type).unwrap().trim_matches('"').to_string(),
            category: serde_json::to_string(&task.category).unwrap().trim_matches('"').to_string(),

            context_id: Uuid::parse_str(&task.context_id).unwrap_or_else(|_| Uuid::new_v4()),
            kind: task.kind.clone(),
            status: serde_json::to_string(&task.status.state).unwrap().trim_matches('"').to_string(),
            status_message: task.status.message.clone(),
            status_timestamp: task.status.timestamp.as_ref().and_then(|ts| chrono::DateTime::parse_from_rfc3339(ts).ok().map(|dt| dt.with_timezone(&chrono::Utc))),

            history: task.history.as_ref().map(|h| serde_json::to_value(h).unwrap()).unwrap_or_else(|| serde_json::json!([])),
            artifacts: task.artifacts.as_ref().map(|a| serde_json::to_value(a).unwrap()).unwrap_or_else(|| serde_json::json!([])),

            priority: serde_json::to_string(&task.priority).unwrap().trim_matches('"').to_string(),
            user_id,
            assigned_agent_id,
            created_at: task.created_at,
            updated_at: task.updated_at,
            started_at: task.started_at,
            completed_at: task.completed_at,
            progress: task.progress as i16,
            estimated_duration: task.estimated_duration.map(|d| d as i64),
            actual_duration: task.actual_duration.map(|d| d as i64),
            sub_tasks: serde_json::to_value(&task.sub_tasks)?,
            dependencies: serde_json::to_value(&task.dependencies)?,
            context: serde_json::to_value(&task.context)?,
            parameters: serde_json::to_value(&task.parameters)?,
            outcome: if let Some(ref outcome) = task.outcome {
                Some(serde_json::to_value(outcome)?)
            } else {
                None
            },
            logs: serde_json::to_value(&task.logs)?,
            triggers: serde_json::to_value(&task.triggers)?,
            required_capabilities: serde_json::to_value(&task.required_capabilities)?,
            auto_retry: task.auto_retry,
            max_retries: task.max_retries as i32,
            current_retries: task.current_retries as i32,
            user_approval_required: task.user_approval_required,
            user_notifications: task.user_notifications,

            actioned_at: task.actioned_at,
            action_result: task.action_result.clone(),
            action_metadata: serde_json::to_value(&task.action_metadata)?,
            action_duration: task.action_duration.map(|d| d as i64),

            source_identifier: task.source_identifier.clone(),
            source_metadata: serde_json::to_value(&task.source_metadata)?,
        })
    }
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AgentEntity {
    pub id: Uuid,
    pub name: String,
    pub agent_type: String,
    pub description: Option<String>,
    pub status: String,
    pub capabilities: serde_json::Value,
    pub endpoint_url: Option<String>,
    pub metadata: serde_json::Value,
    pub performance_metrics: serde_json::Value,
    pub last_health_check: Option<DateTime<Utc>>,
    pub health_status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // Activity tracking fields
    pub last_task_processed: Option<Uuid>,
    pub tasks_processed_count: i32,
    pub last_action_at: Option<DateTime<Utc>>,
    pub average_processing_time: i64,
    pub total_processing_time: i64,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UserReferenceEntity {
    pub id: Uuid,
    pub source_identifier: Option<String>,
    pub network: Option<String>,
    pub user_type: String,
    pub email: Option<String>,
    pub metadata: serde_json::Value,
    pub preferences: serde_json::Value,
    pub additional_metadata: serde_json::Value,
    pub source_type: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}