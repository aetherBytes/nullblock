use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::RiskParams;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StrategyRecord {
    pub id: Uuid,
    pub wallet_address: String,
    pub name: String,
    pub strategy_type: String,
    pub venue_types: Vec<String>,
    pub execution_mode: String,
    pub risk_params: serde_json::Value,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStrategyRecord {
    pub wallet_address: String,
    pub name: String,
    pub strategy_type: String,
    pub venue_types: Vec<String>,
    pub execution_mode: String,
    pub risk_params: RiskParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStrategyRecord {
    pub name: Option<String>,
    pub venue_types: Option<Vec<String>>,
    pub execution_mode: Option<String>,
    pub risk_params: Option<RiskParams>,
    pub is_active: Option<bool>,
}

pub struct StrategyRepository {
    pool: PgPool,
}

impl StrategyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, strategy: CreateStrategyRecord) -> AppResult<StrategyRecord> {
        let risk_params_json = serde_json::to_value(&strategy.risk_params)
            .map_err(|e| AppError::Serialization(e.to_string()))?;

        let record = sqlx::query_as::<_, StrategyRecord>(
            r#"
            INSERT INTO arb_strategies (
                wallet_address, name, strategy_type, venue_types,
                execution_mode, risk_params, is_active, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, true, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(&strategy.wallet_address)
        .bind(&strategy.name)
        .bind(&strategy.strategy_type)
        .bind(&strategy.venue_types)
        .bind(&strategy.execution_mode)
        .bind(&risk_params_json)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(record)
    }

    pub async fn get_by_id(&self, id: Uuid) -> AppResult<Option<StrategyRecord>> {
        let record =
            sqlx::query_as::<_, StrategyRecord>(r#"SELECT * FROM arb_strategies WHERE id = $1"#)
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(record)
    }

    pub async fn update(
        &self,
        id: Uuid,
        update: UpdateStrategyRecord,
    ) -> AppResult<StrategyRecord> {
        let current = self
            .get_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Strategy {} not found", id)))?;

        let risk_params_json = if let Some(params) = &update.risk_params {
            serde_json::to_value(params).map_err(|e| AppError::Serialization(e.to_string()))?
        } else {
            current.risk_params.clone()
        };

        let record = sqlx::query_as::<_, StrategyRecord>(
            r#"
            UPDATE arb_strategies SET
                name = COALESCE($2, name),
                venue_types = COALESCE($3, venue_types),
                execution_mode = COALESCE($4, execution_mode),
                risk_params = $5,
                is_active = COALESCE($6, is_active),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&update.name)
        .bind(&update.venue_types)
        .bind(&update.execution_mode)
        .bind(&risk_params_json)
        .bind(update.is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(record)
    }

    pub async fn delete(&self, id: Uuid) -> AppResult<()> {
        sqlx::query(r#"DELETE FROM arb_strategies WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn list(
        &self,
        wallet_address: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> AppResult<Vec<StrategyRecord>> {
        let query = if wallet_address.is_some() {
            r#"
            SELECT * FROM arb_strategies
            WHERE wallet_address = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#
        } else {
            r#"
            SELECT * FROM arb_strategies
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#
        };

        let records = if let Some(wallet) = wallet_address {
            sqlx::query_as::<_, StrategyRecord>(query)
                .bind(wallet)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
        } else {
            sqlx::query_as::<_, StrategyRecord>(query)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
        }
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(records)
    }

    pub async fn list_active(&self) -> AppResult<Vec<StrategyRecord>> {
        let records = sqlx::query_as::<_, StrategyRecord>(
            r#"
            SELECT * FROM arb_strategies
            WHERE is_active = true
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(records)
    }

    pub async fn list_by_venue_type(&self, venue_type: &str) -> AppResult<Vec<StrategyRecord>> {
        let records = sqlx::query_as::<_, StrategyRecord>(
            r#"
            SELECT * FROM arb_strategies
            WHERE $1 = ANY(venue_types)
              AND is_active = true
            ORDER BY created_at DESC
            "#,
        )
        .bind(venue_type)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(records)
    }

    pub async fn toggle(&self, id: Uuid, enabled: bool) -> AppResult<StrategyRecord> {
        let record = sqlx::query_as::<_, StrategyRecord>(
            r#"
            UPDATE arb_strategies SET
                is_active = $2,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(enabled)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(record)
    }

    pub async fn get_strategy_type_map(
        &self,
    ) -> AppResult<std::collections::HashMap<Uuid, String>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: Uuid,
            strategy_type: String,
        }
        let rows: Vec<Row> = sqlx::query_as("SELECT id, strategy_type FROM arb_strategies")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(rows.into_iter().map(|r| (r.id, r.strategy_type)).collect())
    }

    pub async fn get_stats(&self, id: Uuid) -> AppResult<StrategyStats> {
        let stats = sqlx::query_as::<_, StrategyStats>(
            r#"
            SELECT
                s.id as strategy_id,
                s.name as strategy_name,
                COUNT(t.id) as total_trades,
                COUNT(t.id) FILTER (WHERE t.profit_lamports > 0) as winning_trades,
                COUNT(t.id) FILTER (WHERE t.profit_lamports < 0) as losing_trades,
                COALESCE(SUM(t.profit_lamports), 0)::BIGINT as total_pnl_lamports,
                COALESCE(AVG(t.profit_lamports), 0) as avg_profit_lamports
            FROM arb_strategies s
            LEFT JOIN arb_trades t ON s.id = t.strategy_id
            WHERE s.id = $1
            GROUP BY s.id, s.name
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(stats)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StrategyStats {
    pub strategy_id: Uuid,
    pub strategy_name: String,
    pub total_trades: i64,
    pub winning_trades: i64,
    pub losing_trades: i64,
    pub total_pnl_lamports: i64,
    pub avg_profit_lamports: rust_decimal::Decimal,
}
