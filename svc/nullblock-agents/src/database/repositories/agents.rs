use sqlx::PgPool;
use uuid::Uuid;
use anyhow::Result;
use chrono::Utc;

use crate::database::models::AgentEntity;

pub struct AgentRepository {
    pool: PgPool,
}

impl AgentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, name: &str, agent_type: &str, description: Option<&str>, capabilities: &[String]) -> Result<AgentEntity> {
        let agent_id = Uuid::new_v4();
        let now = Utc::now();

        let agent = sqlx::query_as::<_, AgentEntity>(
            r#"
            INSERT INTO agents (
                id, name, agent_type, description, status, capabilities,
                endpoint_url, metadata, performance_metrics, health_status,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *
            "#
        )
        .bind(agent_id)
        .bind(name)
        .bind(agent_type)
        .bind(description)
        .bind("active")
        .bind(serde_json::to_value(capabilities).unwrap())
        .bind(None::<String>) // endpoint_url
        .bind(serde_json::json!({})) // metadata
        .bind(serde_json::json!({})) // performance_metrics
        .bind("unknown") // health_status
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(agent)
    }

    pub async fn get_by_id(&self, agent_id: &Uuid) -> Result<Option<AgentEntity>> {
        let agent = sqlx::query_as::<_, AgentEntity>(
            "SELECT * FROM agents WHERE id = $1"
        )
        .bind(agent_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(agent)
    }

    pub async fn get_by_name_and_type(&self, name: &str, agent_type: &str) -> Result<Option<AgentEntity>> {
        let agent = sqlx::query_as::<_, AgentEntity>(
            "SELECT * FROM agents WHERE name = $1 AND agent_type = $2"
        )
        .bind(name)
        .bind(agent_type)
        .fetch_optional(&self.pool)
        .await?;

        Ok(agent)
    }

    pub async fn list(&self, agent_type_filter: Option<&str>, status_filter: Option<&str>) -> Result<Vec<AgentEntity>> {
        let mut query_builder = sqlx::QueryBuilder::new("SELECT * FROM agents WHERE 1=1");

        if let Some(agent_type) = agent_type_filter {
            query_builder.push(" AND agent_type = ");
            query_builder.push_bind(agent_type);
        }

        if let Some(status) = status_filter {
            query_builder.push(" AND status = ");
            query_builder.push_bind(status);
        }

        query_builder.push(" ORDER BY created_at DESC");

        let query = query_builder.build_query_as::<AgentEntity>();
        let agents = query.fetch_all(&self.pool).await?;

        Ok(agents)
    }

    pub async fn update_health_status(&self, agent_id: &Uuid, health_status: &str) -> Result<Option<AgentEntity>> {
        let now = Utc::now();

        let agent = sqlx::query_as::<_, AgentEntity>(
            r#"
            UPDATE agents SET
                health_status = $2,
                last_health_check = $3,
                updated_at = $4
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(agent_id)
        .bind(health_status)
        .bind(now)
        .bind(now)
        .fetch_optional(&self.pool)
        .await?;

        Ok(agent)
    }

    pub async fn update_performance_metrics(&self, agent_id: &Uuid, metrics: &serde_json::Value) -> Result<Option<AgentEntity>> {
        let now = Utc::now();

        let agent = sqlx::query_as::<_, AgentEntity>(
            r#"
            UPDATE agents SET
                performance_metrics = $2,
                updated_at = $3
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(agent_id)
        .bind(metrics)
        .bind(now)
        .fetch_optional(&self.pool)
        .await?;

        Ok(agent)
    }

    // Activity tracking methods
    pub async fn update_task_processing_stats(&self, agent_id: &Uuid, task_id: &Uuid, processing_time_ms: u64) -> Result<Option<AgentEntity>> {
        let now = Utc::now();

        // Get current stats to calculate new average
        let current_agent = self.get_by_id(agent_id).await?;

        let agent = if let Some(existing) = current_agent {
            let new_count = existing.tasks_processed_count + 1;
            let new_total_time = existing.total_processing_time + (processing_time_ms as i64);
            let new_avg_time = new_total_time / (new_count as i64);

            sqlx::query_as::<_, AgentEntity>(
                r#"
                UPDATE agents SET
                    last_task_processed = $2,
                    tasks_processed_count = $3,
                    last_action_at = $4,
                    average_processing_time = $5,
                    total_processing_time = $6,
                    updated_at = $7
                WHERE id = $1
                RETURNING *
                "#
            )
            .bind(agent_id)
            .bind(task_id)
            .bind(new_count)
            .bind(now)
            .bind(new_avg_time)
            .bind(new_total_time)
            .bind(now)
            .fetch_optional(&self.pool)
            .await?
        } else {
            None
        };

        Ok(agent)
    }

    pub async fn get_agent_stats(&self, agent_id: &Uuid) -> Result<Option<serde_json::Value>> {
        let agent = self.get_by_id(agent_id).await?;

        if let Some(agent) = agent {
            Ok(Some(serde_json::json!({
                "id": agent.id,
                "name": agent.name,
                "agent_type": agent.agent_type,
                "status": agent.status,
                "tasks_processed": agent.tasks_processed_count,
                "last_action_at": agent.last_action_at,
                "average_processing_time_ms": agent.average_processing_time,
                "health_status": agent.health_status,
                "last_health_check": agent.last_health_check,
                "created_at": agent.created_at
            })))
        } else {
            Ok(None)
        }
    }

    pub async fn get_agents_by_activity(&self, limit: Option<i64>) -> Result<Vec<AgentEntity>> {
        let mut query_builder = sqlx::QueryBuilder::new(
            "SELECT * FROM agents WHERE status = 'active'"
        );

        query_builder.push(" ORDER BY last_action_at DESC NULLS LAST, tasks_processed_count DESC");

        if let Some(limit_val) = limit {
            query_builder.push(" LIMIT ");
            query_builder.push_bind(limit_val);
        }

        let query = query_builder.build_query_as::<AgentEntity>();
        let agents = query.fetch_all(&self.pool).await?;

        Ok(agents)
    }
}