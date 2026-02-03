use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ConsensusRecord {
    pub id: Uuid,
    pub edge_id: Option<Uuid>,
    pub models: Vec<String>,
    pub model_votes: serde_json::Value,
    pub approved: Option<bool>,
    pub agreement_score: Option<f64>,
    pub weighted_confidence: Option<f64>,
    pub reasoning_summary: Option<String>,
    pub edge_context: Option<String>,
    pub total_latency_ms: Option<i64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CreateConsensusRecord {
    pub edge_id: Option<Uuid>,
    pub models: Vec<String>,
    pub model_votes: serde_json::Value,
    pub approved: bool,
    pub agreement_score: f64,
    pub weighted_confidence: f64,
    pub reasoning_summary: String,
    pub edge_context: Option<String>,
    pub total_latency_ms: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ConsensusStats {
    pub total_decisions: i64,
    pub approved_count: i64,
    pub rejected_count: i64,
    pub approval_rate: f64,
    pub avg_agreement_score: f64,
    pub avg_latency_ms: f64,
    pub decisions_today: i64,
    pub decisions_this_week: i64,
}

pub struct ConsensusRepository {
    pool: PgPool,
}

impl ConsensusRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, record: CreateConsensusRecord) -> AppResult<ConsensusRecord> {
        let row = sqlx::query_as::<_, ConsensusRecord>(
            r#"
            INSERT INTO arb_consensus (
                edge_id, models, model_votes, approved, agreement_score,
                weighted_confidence, reasoning_summary, edge_context, total_latency_ms
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(record.edge_id)
        .bind(&record.models)
        .bind(&record.model_votes)
        .bind(record.approved)
        .bind(record.agreement_score)
        .bind(record.weighted_confidence)
        .bind(&record.reasoning_summary)
        .bind(&record.edge_context)
        .bind(record.total_latency_ms)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(row)
    }

    pub async fn get_by_id(&self, id: Uuid) -> AppResult<Option<ConsensusRecord>> {
        let row = sqlx::query_as::<_, ConsensusRecord>("SELECT * FROM arb_consensus WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(row)
    }

    pub async fn get_by_edge_id(&self, edge_id: Uuid) -> AppResult<Option<ConsensusRecord>> {
        let row = sqlx::query_as::<_, ConsensusRecord>(
            "SELECT * FROM arb_consensus WHERE edge_id = $1 ORDER BY created_at DESC LIMIT 1",
        )
        .bind(edge_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(row)
    }

    pub async fn list(&self, limit: i64, offset: i64) -> AppResult<Vec<ConsensusRecord>> {
        let rows = sqlx::query_as::<_, ConsensusRecord>(
            "SELECT * FROM arb_consensus ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(rows)
    }

    pub async fn list_recent(&self, limit: i64) -> AppResult<Vec<ConsensusRecord>> {
        self.list(limit, 0).await
    }

    pub async fn get_stats(&self) -> AppResult<ConsensusStats> {
        #[derive(sqlx::FromRow)]
        struct StatsRow {
            total_decisions: Option<i64>,
            approved_count: Option<i64>,
            rejected_count: Option<i64>,
            avg_agreement_score: Option<f64>,
            avg_latency_ms: Option<f64>,
            decisions_today: Option<i64>,
            decisions_this_week: Option<i64>,
        }

        let row = sqlx::query_as::<_, StatsRow>(
            r#"
            SELECT
                COUNT(*) as total_decisions,
                SUM(CASE WHEN approved = true THEN 1 ELSE 0 END) as approved_count,
                SUM(CASE WHEN approved = false THEN 1 ELSE 0 END) as rejected_count,
                AVG(agreement_score) as avg_agreement_score,
                AVG(total_latency_ms) as avg_latency_ms,
                SUM(CASE WHEN created_at >= CURRENT_DATE THEN 1 ELSE 0 END) as decisions_today,
                SUM(CASE WHEN created_at >= CURRENT_DATE - INTERVAL '7 days' THEN 1 ELSE 0 END) as decisions_this_week
            FROM arb_consensus
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let total = row.total_decisions.unwrap_or(0);
        let approved = row.approved_count.unwrap_or(0);

        Ok(ConsensusStats {
            total_decisions: total,
            approved_count: approved,
            rejected_count: row.rejected_count.unwrap_or(0),
            approval_rate: if total > 0 {
                approved as f64 / total as f64 * 100.0
            } else {
                0.0
            },
            avg_agreement_score: row.avg_agreement_score.unwrap_or(0.0),
            avg_latency_ms: row.avg_latency_ms.unwrap_or(0.0),
            decisions_today: row.decisions_today.unwrap_or(0),
            decisions_this_week: row.decisions_this_week.unwrap_or(0),
        })
    }

    pub async fn delete_old(&self, days: i64) -> AppResult<u64> {
        let result = sqlx::query(
            "DELETE FROM arb_consensus WHERE created_at < NOW() - INTERVAL '1 day' * $1",
        )
        .bind(days)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(result.rows_affected())
    }
}
