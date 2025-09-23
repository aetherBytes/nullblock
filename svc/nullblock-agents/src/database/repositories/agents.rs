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
}