use sqlx::PgPool;
use uuid::Uuid;
use anyhow::Result;
use chrono::Utc;

use crate::database::models::TaskEntity;
use crate::models::{CreateTaskRequest, UpdateTaskRequest};

pub struct TaskRepository {
    pool: PgPool,
}

impl TaskRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, request: &CreateTaskRequest, user_id: Option<Uuid>, assigned_agent_id: Option<Uuid>) -> Result<TaskEntity> {
        let task_id = Uuid::new_v4();
        let context_id = Uuid::new_v4();
        let now = Utc::now();

        let status_str = if request.auto_start.unwrap_or(false) {
            "working"
        } else {
            "submitted"
        };

        let started_at = if status_str == "working" {
            Some(now)
        } else {
            None
        };

        let task = sqlx::query_as::<_, TaskEntity>(
            r#"
            INSERT INTO tasks (
                id, name, description, task_type, category,
                context_id, kind, status, status_timestamp,
                priority, user_id, assigned_agent_id,
                created_at, updated_at, started_at, progress,
                sub_tasks, dependencies, context, parameters, logs, triggers,
                required_capabilities, auto_retry, max_retries, current_retries,
                user_approval_required, user_notifications
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12,
                $13, $14, $15, $16, $17, $18, $19, $20, $21, $22,
                $23, $24, $25, $26, $27, $28
            )
            RETURNING *
            "#
        )
        .bind(task_id)
        .bind(&request.name)
        .bind(&request.description)
        .bind(serde_json::to_string(&request.task_type).unwrap().trim_matches('"').to_string())
        .bind(serde_json::to_string(&request.category.as_ref().unwrap_or(&crate::models::TaskCategory::UserAssigned)).unwrap().trim_matches('"').to_string())
        .bind(context_id)
        .bind("task")
        .bind(status_str)
        .bind(now)
        .bind(serde_json::to_string(&request.priority.as_ref().unwrap_or(&crate::models::TaskPriority::Medium)).unwrap().trim_matches('"').to_string())
        .bind(user_id)
        .bind(assigned_agent_id)
        .bind(now)
        .bind(now)
        .bind(started_at)
        .bind(0i16)
        .bind(serde_json::json!([]))
        .bind(serde_json::to_value(&request.dependencies.as_ref().unwrap_or(&vec![])).unwrap())
        .bind(serde_json::json!({}))
        .bind(serde_json::to_value(&request.parameters.as_ref().unwrap_or(&std::collections::HashMap::new())).unwrap())
        .bind(serde_json::json!([]))
        .bind(serde_json::json!([]))
        .bind(serde_json::json!([]))
        .bind(true)
        .bind(3i32)
        .bind(0i32)
        .bind(request.user_approval_required.unwrap_or(false))
        .bind(true)
        .fetch_one(&self.pool)
        .await?;

        Ok(task)
    }

    pub async fn get_by_id(&self, task_id: &str) -> Result<Option<TaskEntity>> {
        let uuid = Uuid::parse_str(task_id)?;

        let task = sqlx::query_as::<_, TaskEntity>(
            "SELECT * FROM tasks WHERE id = $1"
        )
        .bind(uuid)
        .fetch_optional(&self.pool)
        .await?;

        Ok(task)
    }

    pub async fn list(&self, user_id: Option<Uuid>, status_filter: Option<&str>, task_type_filter: Option<&str>, limit: Option<i64>) -> Result<Vec<TaskEntity>> {
        let mut query_builder = sqlx::QueryBuilder::new("SELECT * FROM tasks WHERE 1=1");

        if let Some(uid) = user_id {
            query_builder.push(" AND user_id = ");
            query_builder.push_bind(uid);
        }

        if let Some(status) = status_filter {
            query_builder.push(" AND status = ");
            query_builder.push_bind(status.to_lowercase());
        }

        if let Some(task_type) = task_type_filter {
            query_builder.push(" AND task_type = ");
            query_builder.push_bind(task_type.to_lowercase());
        }

        query_builder.push(" ORDER BY created_at DESC");

        if let Some(limit_val) = limit {
            query_builder.push(" LIMIT ");
            query_builder.push_bind(limit_val);
        }

        let query = query_builder.build_query_as::<TaskEntity>();
        let tasks = query.fetch_all(&self.pool).await?;

        Ok(tasks)
    }

    pub async fn update(&self, task_id: &str, request: &UpdateTaskRequest) -> Result<Option<TaskEntity>> {
        let uuid = Uuid::parse_str(task_id)?;
        let now = Utc::now();

        let task = sqlx::query_as::<_, TaskEntity>(
            r#"
            UPDATE tasks SET
                name = COALESCE($2, name),
                description = COALESCE($3, description),
                status = COALESCE($4, status),
                priority = COALESCE($5, priority),
                progress = COALESCE($6, progress),
                parameters = COALESCE($7, parameters),
                started_at = COALESCE($8, started_at),
                completed_at = COALESCE($9, completed_at),
                outcome = COALESCE($10, outcome),
                updated_at = $11
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(uuid)
        .bind(request.name.as_ref())
        .bind(request.description.as_ref())
        .bind(request.status.as_ref().map(|s| serde_json::to_string(s).unwrap().trim_matches('"').to_string()))
        .bind(request.priority.as_ref().map(|p| serde_json::to_string(p).unwrap().trim_matches('"').to_string()))
        .bind(request.progress.map(|p| p as i16))
        .bind(request.parameters.as_ref().and_then(|p| serde_json::to_value(p).ok()))
        .bind(request.started_at)
        .bind(request.completed_at)
        .bind(request.outcome.as_ref().and_then(|o| serde_json::to_value(o).ok()))
        .bind(now)
        .fetch_optional(&self.pool)
        .await?;

        Ok(task)
    }

    pub async fn delete(&self, task_id: &str) -> Result<Option<TaskEntity>> {
        let uuid = Uuid::parse_str(task_id)?;

        let task = sqlx::query_as::<_, TaskEntity>(
            "DELETE FROM tasks WHERE id = $1 RETURNING *"
        )
        .bind(uuid)
        .fetch_optional(&self.pool)
        .await?;

        Ok(task)
    }

    pub async fn update_status(&self, task_id: &str, status: crate::models::TaskState) -> Result<Option<TaskEntity>> {
        let uuid = Uuid::parse_str(task_id)?;
        let now = Utc::now();
        let status_str = serde_json::to_string(&status).unwrap().trim_matches('"').to_string();

        let started_at = if status == crate::models::TaskState::Working {
            Some(now)
        } else {
            None
        };

        let completed_at = if matches!(status, crate::models::TaskState::Completed | crate::models::TaskState::Failed | crate::models::TaskState::Canceled) {
            Some(now)
        } else {
            None
        };

        let task = sqlx::query_as::<_, TaskEntity>(
            r#"
            UPDATE tasks SET
                status = $2,
                status_timestamp = $3,
                started_at = COALESCE($4, started_at),
                completed_at = COALESCE($5, completed_at),
                updated_at = $6
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(uuid)
        .bind(status_str)
        .bind(now)
        .bind(started_at)
        .bind(completed_at)
        .bind(now)
        .fetch_optional(&self.pool)
        .await?;

        Ok(task)
    }

    // Action tracking methods
    pub async fn mark_task_actioned(&self, task_id: &str, action_metadata: Option<serde_json::Value>) -> Result<Option<TaskEntity>> {
        let uuid = Uuid::parse_str(task_id)?;
        let now = Utc::now();

        let task = sqlx::query_as::<_, TaskEntity>(
            r#"
            UPDATE tasks SET
                actioned_at = $2,
                action_metadata = $3,
                updated_at = $4
            WHERE id = $1 AND actioned_at IS NULL
            RETURNING *
            "#
        )
        .bind(uuid)
        .bind(now)
        .bind(action_metadata.unwrap_or_else(|| serde_json::json!({})))
        .bind(now)
        .fetch_optional(&self.pool)
        .await?;

        Ok(task)
    }

    pub async fn update_action_result(&self, task_id: &str, action_result: &str, action_duration: Option<u64>) -> Result<Option<TaskEntity>> {
        let uuid = Uuid::parse_str(task_id)?;
        let now = Utc::now();

        let task = sqlx::query_as::<_, TaskEntity>(
            r#"
            UPDATE tasks SET
                action_result = $2,
                action_duration = $3,
                updated_at = $4
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(uuid)
        .bind(action_result)
        .bind(action_duration.map(|d| d as i64))
        .bind(now)
        .fetch_optional(&self.pool)
        .await?;

        Ok(task)
    }

    pub async fn get_unactioned_tasks(&self, agent_id: Option<Uuid>, limit: Option<i64>) -> Result<Vec<TaskEntity>> {
        let mut query_builder = sqlx::QueryBuilder::new(
            "SELECT * FROM tasks WHERE status = 'working' AND actioned_at IS NULL"
        );

        if let Some(agent) = agent_id {
            query_builder.push(" AND assigned_agent_id = ");
            query_builder.push_bind(agent);
        }

        query_builder.push(" ORDER BY priority DESC, created_at ASC");

        if let Some(limit_val) = limit {
            query_builder.push(" LIMIT ");
            query_builder.push_bind(limit_val);
        }

        let query = query_builder.build_query_as::<TaskEntity>();
        let tasks = query.fetch_all(&self.pool).await?;

        Ok(tasks)
    }

    pub async fn get_tasks_for_agent(&self, agent_id: Uuid, status_filter: Option<&str>) -> Result<Vec<TaskEntity>> {
        let mut query_builder = sqlx::QueryBuilder::new(
            "SELECT * FROM tasks WHERE assigned_agent_id = "
        );
        query_builder.push_bind(agent_id);

        if let Some(status) = status_filter {
            query_builder.push(" AND status = ");
            query_builder.push_bind(status);
        }

        query_builder.push(" ORDER BY actioned_at DESC NULLS FIRST, created_at DESC");

        let query = query_builder.build_query_as::<TaskEntity>();
        let tasks = query.fetch_all(&self.pool).await?;

        Ok(tasks)
    }
}