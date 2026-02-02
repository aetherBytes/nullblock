use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TradeRecord {
    pub id: Uuid,
    pub edge_id: Option<Uuid>,
    pub strategy_id: Option<Uuid>,
    pub tx_signature: Option<String>,
    pub bundle_id: Option<String>,
    pub entry_price: Option<Decimal>,
    pub exit_price: Option<Decimal>,
    pub profit_lamports: Option<i64>,
    pub gas_cost_lamports: Option<i64>,
    pub slippage_bps: Option<i32>,
    pub executed_at: DateTime<Utc>,
    pub entry_gas_lamports: Option<i64>,
    pub exit_gas_lamports: Option<i64>,
    pub pnl_source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTradeRecord {
    pub edge_id: Option<Uuid>,
    pub strategy_id: Option<Uuid>,
    pub tx_signature: Option<String>,
    pub bundle_id: Option<String>,
    pub entry_price: Option<Decimal>,
    pub exit_price: Option<Decimal>,
    pub profit_lamports: Option<i64>,
    pub gas_cost_lamports: Option<i64>,
    pub slippage_bps: Option<i32>,
    pub entry_gas_lamports: Option<i64>,
    pub exit_gas_lamports: Option<i64>,
    pub pnl_source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeStats {
    pub total_trades: i64,
    pub winning_trades: i64,
    pub losing_trades: i64,
    pub total_profit_lamports: i64,
    pub total_loss_lamports: i64,
    pub net_pnl_lamports: i64,
    pub total_gas_cost_lamports: i64,
    pub avg_profit_lamports: f64,
    pub win_rate: f64,
    pub largest_win_lamports: i64,
    pub largest_loss_lamports: i64,
}

pub struct TradeRepository {
    pool: PgPool,
}

impl TradeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, trade: CreateTradeRecord) -> AppResult<TradeRecord> {
        let record = sqlx::query_as::<_, TradeRecord>(
            r#"
            INSERT INTO arb_trades (
                edge_id, strategy_id, tx_signature, bundle_id,
                entry_price, exit_price, profit_lamports,
                gas_cost_lamports, slippage_bps, executed_at,
                entry_gas_lamports, exit_gas_lamports, pnl_source
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), $10, $11, $12)
            RETURNING *
            "#,
        )
        .bind(trade.edge_id)
        .bind(trade.strategy_id)
        .bind(&trade.tx_signature)
        .bind(&trade.bundle_id)
        .bind(trade.entry_price)
        .bind(trade.exit_price)
        .bind(trade.profit_lamports)
        .bind(trade.gas_cost_lamports)
        .bind(trade.slippage_bps)
        .bind(trade.entry_gas_lamports)
        .bind(trade.exit_gas_lamports)
        .bind(&trade.pnl_source)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(record)
    }

    pub async fn get_by_id(&self, id: Uuid) -> AppResult<Option<TradeRecord>> {
        let record = sqlx::query_as::<_, TradeRecord>(
            r#"SELECT * FROM arb_trades WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(record)
    }

    pub async fn get_by_edge_id(&self, edge_id: Uuid) -> AppResult<Option<TradeRecord>> {
        let record = sqlx::query_as::<_, TradeRecord>(
            r#"SELECT * FROM arb_trades WHERE edge_id = $1"#,
        )
        .bind(edge_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(record)
    }

    pub async fn list(&self, limit: i64, offset: i64) -> AppResult<Vec<TradeRecord>> {
        let records = sqlx::query_as::<_, TradeRecord>(
            r#"
            SELECT * FROM arb_trades
            ORDER BY executed_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(records)
    }

    pub async fn list_by_strategy(
        &self,
        strategy_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> AppResult<Vec<TradeRecord>> {
        let records = sqlx::query_as::<_, TradeRecord>(
            r#"
            SELECT * FROM arb_trades
            WHERE strategy_id = $1
            ORDER BY executed_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(strategy_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(records)
    }

    pub async fn get_stats(&self, period_days: Option<i32>) -> AppResult<TradeStats> {
        let period_clause = if let Some(days) = period_days {
            format!("WHERE executed_at > NOW() - INTERVAL '{} days'", days)
        } else {
            String::new()
        };

        let query = format!(
            r#"
            SELECT
                COUNT(*)::BIGINT as total_trades,
                COUNT(*) FILTER (WHERE profit_lamports > 0)::BIGINT as winning_trades,
                COUNT(*) FILTER (WHERE profit_lamports < 0)::BIGINT as losing_trades,
                COALESCE(SUM(profit_lamports) FILTER (WHERE profit_lamports > 0), 0)::BIGINT as total_profit,
                COALESCE(SUM(ABS(profit_lamports)) FILTER (WHERE profit_lamports < 0), 0)::BIGINT as total_loss,
                COALESCE(SUM(profit_lamports), 0)::BIGINT as net_pnl,
                COALESCE(SUM(gas_cost_lamports), 0)::BIGINT as total_gas,
                COALESCE(AVG(profit_lamports), 0) as avg_profit,
                COALESCE(MAX(profit_lamports) FILTER (WHERE profit_lamports > 0), 0)::BIGINT as largest_win,
                COALESCE(MIN(profit_lamports) FILTER (WHERE profit_lamports < 0), 0)::BIGINT as largest_loss
            FROM arb_trades
            {}
            "#,
            period_clause
        );

        #[derive(sqlx::FromRow)]
        struct StatsRow {
            total_trades: i64,
            winning_trades: i64,
            losing_trades: i64,
            total_profit: i64,
            total_loss: i64,
            net_pnl: i64,
            total_gas: i64,
            avg_profit: Decimal,
            largest_win: i64,
            largest_loss: i64,
        }

        let row = sqlx::query_as::<_, StatsRow>(&query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let win_rate = if row.total_trades > 0 {
            row.winning_trades as f64 / row.total_trades as f64
        } else {
            0.0
        };

        Ok(TradeStats {
            total_trades: row.total_trades,
            winning_trades: row.winning_trades,
            losing_trades: row.losing_trades,
            total_profit_lamports: row.total_profit,
            total_loss_lamports: row.total_loss,
            net_pnl_lamports: row.net_pnl,
            total_gas_cost_lamports: row.total_gas,
            avg_profit_lamports: row.avg_profit.try_into().unwrap_or(0.0),
            win_rate,
            largest_win_lamports: row.largest_win,
            largest_loss_lamports: row.largest_loss,
        })
    }

    pub async fn get_daily_stats(&self, days: i32) -> AppResult<Vec<DailyStats>> {
        let records = sqlx::query_as::<_, DailyStats>(
            r#"
            SELECT
                DATE(executed_at) as date,
                COUNT(*) as trade_count,
                COUNT(*) FILTER (WHERE profit_lamports > 0) as wins,
                COUNT(*) FILTER (WHERE profit_lamports < 0) as losses,
                COALESCE(SUM(profit_lamports), 0)::BIGINT as net_pnl_lamports,
                COALESCE(SUM(gas_cost_lamports), 0)::BIGINT as gas_cost_lamports
            FROM arb_trades
            WHERE executed_at > NOW() - INTERVAL '1 day' * $1
            GROUP BY DATE(executed_at)
            ORDER BY DATE(executed_at) DESC
            "#,
        )
        .bind(days)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(records)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DailyStats {
    pub date: chrono::NaiveDate,
    pub trade_count: i64,
    pub wins: i64,
    pub losses: i64,
    pub net_pnl_lamports: i64,
    pub gas_cost_lamports: i64,
}
