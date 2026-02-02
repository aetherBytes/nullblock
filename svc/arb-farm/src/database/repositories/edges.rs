use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::events::AtomicityLevel;
use crate::models::EdgeStatus;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EdgeRecord {
    pub id: Uuid,
    pub strategy_id: Option<Uuid>,
    pub edge_type: String,
    pub execution_mode: String,
    pub atomicity: String,
    pub simulated_profit_guaranteed: bool,
    pub simulation_tx_hash: Option<String>,
    pub max_gas_cost_lamports: Option<i64>,
    pub estimated_profit_lamports: Option<i64>,
    pub risk_score: Option<i32>,
    pub route_data: serde_json::Value,
    pub status: String,
    pub rejection_reason: Option<String>,
    pub executed_at: Option<DateTime<Utc>>,
    pub actual_profit_lamports: Option<i64>,
    pub actual_gas_cost_lamports: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEdgeRecord {
    pub strategy_id: Option<Uuid>,
    pub edge_type: String,
    pub execution_mode: String,
    pub atomicity: AtomicityLevel,
    pub simulated_profit_guaranteed: bool,
    pub estimated_profit_lamports: Option<i64>,
    pub risk_score: Option<i32>,
    pub route_data: serde_json::Value,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEdgeRecord {
    pub status: Option<EdgeStatus>,
    pub rejection_reason: Option<String>,
    pub executed_at: Option<DateTime<Utc>>,
    pub actual_profit_lamports: Option<i64>,
    pub actual_gas_cost_lamports: Option<i64>,
    pub simulation_tx_hash: Option<String>,
    pub max_gas_cost_lamports: Option<i64>,
    pub simulated_profit_guaranteed: Option<bool>,
}

pub struct EdgeRepository {
    pool: PgPool,
}

impl EdgeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, edge: CreateEdgeRecord) -> AppResult<EdgeRecord> {
        let atomicity_str = match edge.atomicity {
            AtomicityLevel::FullyAtomic => "fully_atomic",
            AtomicityLevel::PartiallyAtomic => "partially_atomic",
            AtomicityLevel::NonAtomic => "non_atomic",
        };

        let record = sqlx::query_as::<_, EdgeRecord>(
            r#"
            INSERT INTO arb_edges (
                strategy_id, edge_type, execution_mode, atomicity,
                simulated_profit_guaranteed, estimated_profit_lamports,
                risk_score, route_data, status, expires_at, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'detected', $9, NOW())
            RETURNING *
            "#,
        )
        .bind(edge.strategy_id)
        .bind(&edge.edge_type)
        .bind(&edge.execution_mode)
        .bind(atomicity_str)
        .bind(edge.simulated_profit_guaranteed)
        .bind(edge.estimated_profit_lamports)
        .bind(edge.risk_score)
        .bind(&edge.route_data)
        .bind(edge.expires_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(record)
    }

    pub async fn get_by_id(&self, id: Uuid) -> AppResult<Option<EdgeRecord>> {
        let record = sqlx::query_as::<_, EdgeRecord>(r#"SELECT * FROM arb_edges WHERE id = $1"#)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(record)
    }

    pub async fn update(&self, id: Uuid, update: UpdateEdgeRecord) -> AppResult<EdgeRecord> {
        let current = self
            .get_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Edge {} not found", id)))?;

        let status_str = update
            .status
            .map(|s| status_to_string(&s))
            .unwrap_or_else(|| current.status.clone());

        let record = sqlx::query_as::<_, EdgeRecord>(
            r#"
            UPDATE arb_edges SET
                status = $2,
                rejection_reason = COALESCE($3, rejection_reason),
                executed_at = COALESCE($4, executed_at),
                actual_profit_lamports = COALESCE($5, actual_profit_lamports),
                actual_gas_cost_lamports = COALESCE($6, actual_gas_cost_lamports),
                simulation_tx_hash = COALESCE($7, simulation_tx_hash),
                max_gas_cost_lamports = COALESCE($8, max_gas_cost_lamports),
                simulated_profit_guaranteed = COALESCE($9, simulated_profit_guaranteed)
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&status_str)
        .bind(&update.rejection_reason)
        .bind(update.executed_at)
        .bind(update.actual_profit_lamports)
        .bind(update.actual_gas_cost_lamports)
        .bind(&update.simulation_tx_hash)
        .bind(update.max_gas_cost_lamports)
        .bind(update.simulated_profit_guaranteed)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(record)
    }

    pub async fn list(
        &self,
        status: Option<&str>,
        edge_type: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> AppResult<Vec<EdgeRecord>> {
        let mut query = String::from("SELECT * FROM arb_edges WHERE 1=1");
        let mut params_count = 0;

        if status.is_some() {
            params_count += 1;
            query.push_str(&format!(" AND status = ${}", params_count));
        }
        if edge_type.is_some() {
            params_count += 1;
            query.push_str(&format!(" AND edge_type = ${}", params_count));
        }

        query.push_str(&format!(
            " ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            params_count + 1,
            params_count + 2
        ));

        let mut query_builder = sqlx::query_as::<_, EdgeRecord>(&query);

        if let Some(s) = status {
            query_builder = query_builder.bind(s);
        }
        if let Some(t) = edge_type {
            query_builder = query_builder.bind(t);
        }
        query_builder = query_builder.bind(limit).bind(offset);

        let records = query_builder
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(records)
    }

    pub async fn list_pending_approval(&self, limit: i64) -> AppResult<Vec<EdgeRecord>> {
        let records = sqlx::query_as::<_, EdgeRecord>(
            r#"
            SELECT * FROM arb_edges
            WHERE status = 'pending_approval'
            ORDER BY created_at ASC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(records)
    }

    pub async fn list_atomic_opportunities(
        &self,
        min_profit: i64,
        limit: i64,
    ) -> AppResult<Vec<EdgeRecord>> {
        let records = sqlx::query_as::<_, EdgeRecord>(
            r#"
            SELECT * FROM arb_edges
            WHERE atomicity = 'fully_atomic'
              AND simulated_profit_guaranteed = true
              AND status = 'detected'
              AND estimated_profit_lamports >= $1
              AND (expires_at IS NULL OR expires_at > NOW())
            ORDER BY estimated_profit_lamports DESC
            LIMIT $2
            "#,
        )
        .bind(min_profit)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(records)
    }

    pub async fn mark_expired(&self) -> AppResult<u64> {
        let result = sqlx::query(
            r#"
            UPDATE arb_edges
            SET status = 'expired'
            WHERE status IN ('detected', 'pending_approval')
              AND expires_at < NOW()
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(result.rows_affected())
    }

    pub async fn count_by_status(&self) -> AppResult<Vec<StatusCount>> {
        let records = sqlx::query_as::<_, StatusCount>(
            r#"
            SELECT status, COUNT(*) as count
            FROM arb_edges
            WHERE created_at > NOW() - INTERVAL '24 hours'
            GROUP BY status
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(records)
    }
}

fn status_to_string(status: &EdgeStatus) -> String {
    match status {
        EdgeStatus::Detected => "detected".to_string(),
        EdgeStatus::PendingApproval => "pending_approval".to_string(),
        EdgeStatus::Executing => "executing".to_string(),
        EdgeStatus::Executed => "executed".to_string(),
        EdgeStatus::Expired => "expired".to_string(),
        EdgeStatus::Failed => "failed".to_string(),
        EdgeStatus::Rejected => "rejected".to_string(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StatusCount {
    pub status: String,
    pub count: i64,
}
