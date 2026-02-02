use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::{CopyTradeConfig, CopyTradeStatus, KolEntityType, KolTradeType};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct KolEntityRecord {
    pub id: Uuid,
    pub entity_type: String,
    pub identifier: String,
    pub display_name: Option<String>,
    pub linked_wallet: Option<String>,
    pub trust_score: Decimal,
    pub total_trades_tracked: i32,
    pub profitable_trades: i32,
    pub avg_profit_percent: Option<Decimal>,
    pub max_drawdown: Option<Decimal>,
    pub copy_trading_enabled: bool,
    pub copy_config: serde_json::Value,
    pub is_active: bool,
    pub discovery_source: Option<String>,
    pub last_trade_at: Option<DateTime<Utc>>,
    pub total_volume_sol: Option<Decimal>,
    pub win_streak: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl KolEntityRecord {
    pub fn entity_type_enum(&self) -> KolEntityType {
        match self.entity_type.to_lowercase().as_str() {
            "wallet" => KolEntityType::Wallet,
            "twitterhandle" | "twitter_handle" => KolEntityType::TwitterHandle,
            _ => KolEntityType::Wallet,
        }
    }

    pub fn copy_config_parsed(&self) -> CopyTradeConfig {
        serde_json::from_value(self.copy_config.clone()).unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateKolEntityRecord {
    pub entity_type: KolEntityType,
    pub identifier: String,
    pub display_name: Option<String>,
    pub linked_wallet: Option<String>,
    pub discovery_source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateKolEntityRecord {
    pub display_name: Option<String>,
    pub linked_wallet: Option<String>,
    pub trust_score: Option<Decimal>,
    pub total_trades_tracked: Option<i32>,
    pub profitable_trades: Option<i32>,
    pub avg_profit_percent: Option<Decimal>,
    pub max_drawdown: Option<Decimal>,
    pub copy_trading_enabled: Option<bool>,
    pub copy_config: Option<CopyTradeConfig>,
    pub is_active: Option<bool>,
    pub last_trade_at: Option<DateTime<Utc>>,
    pub total_volume_sol: Option<Decimal>,
    pub win_streak: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct KolTradeRecord {
    pub id: Uuid,
    pub entity_id: Uuid,
    pub tx_signature: String,
    pub trade_type: String,
    pub token_mint: String,
    pub token_symbol: Option<String>,
    pub amount_sol: Option<Decimal>,
    pub token_amount: Option<Decimal>,
    pub price_at_trade: Option<Decimal>,
    pub detected_at: DateTime<Utc>,
}

impl KolTradeRecord {
    pub fn trade_type_enum(&self) -> KolTradeType {
        match self.trade_type.to_lowercase().as_str() {
            "buy" => KolTradeType::Buy,
            "sell" => KolTradeType::Sell,
            _ => KolTradeType::Buy,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateKolTradeRecord {
    pub entity_id: Uuid,
    pub tx_signature: String,
    pub trade_type: KolTradeType,
    pub token_mint: String,
    pub token_symbol: Option<String>,
    pub amount_sol: Decimal,
    pub token_amount: Option<Decimal>,
    pub price_at_trade: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CopyTradeRecord {
    pub id: Uuid,
    pub entity_id: Uuid,
    pub kol_trade_id: Uuid,
    pub our_tx_signature: Option<String>,
    pub copy_amount_sol: Option<Decimal>,
    pub delay_ms: Option<i64>,
    pub profit_loss_lamports: Option<i64>,
    pub status: String,
    pub is_copied: Option<bool>,
    pub executed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl CopyTradeRecord {
    pub fn status_enum(&self) -> CopyTradeStatus {
        match self.status.to_lowercase().as_str() {
            "pending" => CopyTradeStatus::Pending,
            "executing" => CopyTradeStatus::Executing,
            "executed" => CopyTradeStatus::Executed,
            "failed" => CopyTradeStatus::Failed,
            "skipped" => CopyTradeStatus::Skipped,
            _ => CopyTradeStatus::Pending,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCopyTradeRecord {
    pub entity_id: Uuid,
    pub kol_trade_id: Uuid,
    pub copy_amount_sol: Decimal,
    pub delay_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateCopyTradeRecord {
    pub our_tx_signature: Option<String>,
    pub profit_loss_lamports: Option<i64>,
    pub status: Option<CopyTradeStatus>,
    pub is_copied: Option<bool>,
    pub executed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct KolEntityStats {
    pub entity_id: Uuid,
    pub identifier: String,
    pub display_name: Option<String>,
    pub trust_score: Decimal,
    pub total_trades: i64,
    pub profitable_trades: i64,
    pub win_rate: f64,
    pub avg_profit_percent: Option<Decimal>,
    pub max_drawdown: Option<Decimal>,
    pub total_volume_sol: Decimal,
    pub copy_count: i64,
    pub copy_profit_lamports: i64,
    pub copy_trading_enabled: bool,
    pub last_trade_at: Option<DateTime<Utc>>,
}

pub struct KolRepository {
    pool: PgPool,
}

impl KolRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_entity(&self, entity: CreateKolEntityRecord) -> AppResult<KolEntityRecord> {
        let entity_type_str = match entity.entity_type {
            KolEntityType::Wallet => "wallet",
            KolEntityType::TwitterHandle => "twitter_handle",
        };

        let record = sqlx::query_as::<_, KolEntityRecord>(
            r#"
            INSERT INTO arb_kol_entities (
                entity_type, identifier, display_name, linked_wallet,
                discovery_source, trust_score, copy_config,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, 50.0, '{}', NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(entity_type_str)
        .bind(&entity.identifier)
        .bind(&entity.display_name)
        .bind(&entity.linked_wallet)
        .bind(&entity.discovery_source)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(record)
    }

    pub async fn get_entity(&self, id: Uuid) -> AppResult<Option<KolEntityRecord>> {
        let record =
            sqlx::query_as::<_, KolEntityRecord>(r#"SELECT * FROM arb_kol_entities WHERE id = $1"#)
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(record)
    }

    pub async fn get_by_identifier(&self, identifier: &str) -> AppResult<Option<KolEntityRecord>> {
        let record = sqlx::query_as::<_, KolEntityRecord>(
            r#"SELECT * FROM arb_kol_entities WHERE identifier = $1"#,
        )
        .bind(identifier)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(record)
    }

    pub async fn update_entity(
        &self,
        id: Uuid,
        update: UpdateKolEntityRecord,
    ) -> AppResult<KolEntityRecord> {
        let current = self
            .get_entity(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("KOL entity {} not found", id)))?;

        let copy_config_json = if let Some(config) = &update.copy_config {
            serde_json::to_value(config).map_err(|e| AppError::Serialization(e.to_string()))?
        } else {
            current.copy_config.clone()
        };

        let record = sqlx::query_as::<_, KolEntityRecord>(
            r#"
            UPDATE arb_kol_entities SET
                display_name = COALESCE($2, display_name),
                linked_wallet = COALESCE($3, linked_wallet),
                trust_score = COALESCE($4, trust_score),
                total_trades_tracked = COALESCE($5, total_trades_tracked),
                profitable_trades = COALESCE($6, profitable_trades),
                avg_profit_percent = COALESCE($7, avg_profit_percent),
                max_drawdown = COALESCE($8, max_drawdown),
                copy_trading_enabled = COALESCE($9, copy_trading_enabled),
                copy_config = $10,
                is_active = COALESCE($11, is_active),
                last_trade_at = COALESCE($12, last_trade_at),
                total_volume_sol = COALESCE($13, total_volume_sol),
                win_streak = COALESCE($14, win_streak),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&update.display_name)
        .bind(&update.linked_wallet)
        .bind(update.trust_score)
        .bind(update.total_trades_tracked)
        .bind(update.profitable_trades)
        .bind(update.avg_profit_percent)
        .bind(update.max_drawdown)
        .bind(update.copy_trading_enabled)
        .bind(&copy_config_json)
        .bind(update.is_active)
        .bind(update.last_trade_at)
        .bind(update.total_volume_sol)
        .bind(update.win_streak)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(record)
    }

    pub async fn delete_entity(&self, id: Uuid) -> AppResult<()> {
        sqlx::query(r#"DELETE FROM arb_copy_trades WHERE entity_id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        sqlx::query(r#"DELETE FROM arb_kol_trades WHERE entity_id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        sqlx::query(r#"DELETE FROM arb_kol_entities WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn list_entities(
        &self,
        is_active: Option<bool>,
        copy_enabled: Option<bool>,
        min_trust_score: Option<f64>,
        limit: i64,
        offset: i64,
    ) -> AppResult<Vec<KolEntityRecord>> {
        let records = sqlx::query_as::<_, KolEntityRecord>(
            r#"
            SELECT * FROM arb_kol_entities
            WHERE ($1::boolean IS NULL OR is_active = $1)
              AND ($2::boolean IS NULL OR copy_trading_enabled = $2)
              AND ($3::numeric IS NULL OR trust_score >= $3)
            ORDER BY trust_score DESC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(is_active)
        .bind(copy_enabled)
        .bind(min_trust_score.map(Decimal::from_f64_retain).flatten())
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(records)
    }

    pub async fn count_entities(
        &self,
        is_active: Option<bool>,
        copy_enabled: Option<bool>,
        min_trust_score: Option<f64>,
    ) -> AppResult<i64> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM arb_kol_entities
            WHERE ($1::boolean IS NULL OR is_active = $1)
              AND ($2::boolean IS NULL OR copy_trading_enabled = $2)
              AND ($3::numeric IS NULL OR trust_score >= $3)
            "#,
        )
        .bind(is_active)
        .bind(copy_enabled)
        .bind(min_trust_score.map(Decimal::from_f64_retain).flatten())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(count.0)
    }

    pub async fn get_entities_for_copy(
        &self,
        min_trust_score: f64,
    ) -> AppResult<Vec<KolEntityRecord>> {
        let records = sqlx::query_as::<_, KolEntityRecord>(
            r#"
            SELECT * FROM arb_kol_entities
            WHERE copy_trading_enabled = true
              AND is_active = true
              AND trust_score >= $1
            ORDER BY trust_score DESC
            "#,
        )
        .bind(Decimal::from_f64_retain(min_trust_score))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(records)
    }

    pub async fn record_trade(&self, trade: CreateKolTradeRecord) -> AppResult<KolTradeRecord> {
        let trade_type_str = match trade.trade_type {
            KolTradeType::Buy => "buy",
            KolTradeType::Sell => "sell",
        };

        let record = sqlx::query_as::<_, KolTradeRecord>(
            r#"
            INSERT INTO arb_kol_trades (
                entity_id, tx_signature, trade_type, token_mint,
                token_symbol, amount_sol, token_amount, price_at_trade, detected_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())
            RETURNING *
            "#,
        )
        .bind(trade.entity_id)
        .bind(&trade.tx_signature)
        .bind(trade_type_str)
        .bind(&trade.token_mint)
        .bind(&trade.token_symbol)
        .bind(trade.amount_sol)
        .bind(trade.token_amount)
        .bind(trade.price_at_trade)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        sqlx::query(
            r#"
            UPDATE arb_kol_entities SET
                last_trade_at = NOW(),
                total_volume_sol = COALESCE(total_volume_sol, 0) + COALESCE($2, 0),
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(trade.entity_id)
        .bind(trade.amount_sol)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(record)
    }

    pub async fn get_trades(
        &self,
        entity_id: Uuid,
        trade_type: Option<KolTradeType>,
        limit: i64,
        offset: i64,
    ) -> AppResult<Vec<KolTradeRecord>> {
        let trade_type_str = trade_type.map(|t| match t {
            KolTradeType::Buy => "buy",
            KolTradeType::Sell => "sell",
        });

        let records = sqlx::query_as::<_, KolTradeRecord>(
            r#"
            SELECT * FROM arb_kol_trades
            WHERE entity_id = $1
              AND ($2::text IS NULL OR trade_type = $2)
            ORDER BY detected_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(entity_id)
        .bind(trade_type_str)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(records)
    }

    pub async fn get_pending_copy_trades(&self, entity_id: Uuid) -> AppResult<Vec<KolTradeRecord>> {
        let records = sqlx::query_as::<_, KolTradeRecord>(
            r#"
            SELECT kt.* FROM arb_kol_trades kt
            LEFT JOIN arb_copy_trades ct ON kt.id = ct.kol_trade_id
            WHERE kt.entity_id = $1
              AND ct.id IS NULL
              AND kt.detected_at > NOW() - INTERVAL '5 minutes'
            ORDER BY kt.detected_at DESC
            "#,
        )
        .bind(entity_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(records)
    }

    pub async fn record_copy_trade(
        &self,
        copy: CreateCopyTradeRecord,
    ) -> AppResult<CopyTradeRecord> {
        let record = sqlx::query_as::<_, CopyTradeRecord>(
            r#"
            INSERT INTO arb_copy_trades (
                entity_id, kol_trade_id, copy_amount_sol, delay_ms,
                status, is_copied, created_at
            )
            VALUES ($1, $2, $3, $4, 'pending', false, NOW())
            RETURNING *
            "#,
        )
        .bind(copy.entity_id)
        .bind(copy.kol_trade_id)
        .bind(copy.copy_amount_sol)
        .bind(copy.delay_ms)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(record)
    }

    pub async fn update_copy_trade(
        &self,
        id: Uuid,
        update: UpdateCopyTradeRecord,
    ) -> AppResult<CopyTradeRecord> {
        let status_str = update.status.map(|s| match s {
            CopyTradeStatus::Pending => "pending",
            CopyTradeStatus::Executing => "executing",
            CopyTradeStatus::Executed => "executed",
            CopyTradeStatus::Failed => "failed",
            CopyTradeStatus::Skipped => "skipped",
        });

        let record = sqlx::query_as::<_, CopyTradeRecord>(
            r#"
            UPDATE arb_copy_trades SET
                our_tx_signature = COALESCE($2, our_tx_signature),
                profit_loss_lamports = COALESCE($3, profit_loss_lamports),
                status = COALESCE($4, status),
                is_copied = COALESCE($5, is_copied),
                executed_at = COALESCE($6, executed_at)
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&update.our_tx_signature)
        .bind(update.profit_loss_lamports)
        .bind(status_str)
        .bind(update.is_copied)
        .bind(update.executed_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(record)
    }

    pub async fn get_copy_history(
        &self,
        entity_id: Uuid,
        status: Option<CopyTradeStatus>,
        limit: i64,
        offset: i64,
    ) -> AppResult<Vec<CopyTradeRecord>> {
        let status_str = status.map(|s| match s {
            CopyTradeStatus::Pending => "pending",
            CopyTradeStatus::Executing => "executing",
            CopyTradeStatus::Executed => "executed",
            CopyTradeStatus::Failed => "failed",
            CopyTradeStatus::Skipped => "skipped",
        });

        let records = sqlx::query_as::<_, CopyTradeRecord>(
            r#"
            SELECT * FROM arb_copy_trades
            WHERE entity_id = $1
              AND ($2::text IS NULL OR status = $2)
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(entity_id)
        .bind(status_str)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(records)
    }

    pub async fn get_entity_stats(&self, entity_id: Uuid) -> AppResult<Option<KolEntityStats>> {
        let stats = sqlx::query_as::<_, KolEntityStats>(
            r#"
            SELECT
                e.id as entity_id,
                e.identifier,
                e.display_name,
                e.trust_score,
                COALESCE(e.total_trades_tracked, 0)::bigint as total_trades,
                COALESCE(e.profitable_trades, 0)::bigint as profitable_trades,
                CASE
                    WHEN COALESCE(e.total_trades_tracked, 0) > 0
                    THEN COALESCE(e.profitable_trades, 0)::float / e.total_trades_tracked::float
                    ELSE 0.0
                END as win_rate,
                e.avg_profit_percent,
                e.max_drawdown,
                COALESCE(e.total_volume_sol, 0) as total_volume_sol,
                COALESCE((SELECT COUNT(*) FROM arb_copy_trades WHERE entity_id = e.id AND status = 'executed'), 0) as copy_count,
                COALESCE((SELECT SUM(profit_loss_lamports) FROM arb_copy_trades WHERE entity_id = e.id AND status = 'executed'), 0) as copy_profit_lamports,
                e.copy_trading_enabled,
                e.last_trade_at
            FROM arb_kol_entities e
            WHERE e.id = $1
            "#,
        )
        .bind(entity_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(stats)
    }

    pub async fn update_trust_score(&self, entity_id: Uuid, trust_score: Decimal) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE arb_kol_entities SET
                trust_score = $2,
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(entity_id)
        .bind(trust_score)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn increment_trade_stats(
        &self,
        entity_id: Uuid,
        is_profitable: bool,
    ) -> AppResult<()> {
        if is_profitable {
            sqlx::query(
                r#"
                UPDATE arb_kol_entities SET
                    total_trades_tracked = total_trades_tracked + 1,
                    profitable_trades = profitable_trades + 1,
                    win_streak = COALESCE(win_streak, 0) + 1,
                    updated_at = NOW()
                WHERE id = $1
                "#,
            )
            .bind(entity_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        } else {
            sqlx::query(
                r#"
                UPDATE arb_kol_entities SET
                    total_trades_tracked = total_trades_tracked + 1,
                    win_streak = 0,
                    updated_at = NOW()
                WHERE id = $1
                "#,
            )
            .bind(entity_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        }

        Ok(())
    }

    pub async fn get_copy_stats(&self) -> AppResult<CopyStats> {
        let stats = sqlx::query_as::<_, CopyStatsRow>(
            r#"
            SELECT
                COUNT(*)::bigint as total_copies,
                COUNT(*) FILTER (WHERE status = 'executed')::bigint as executed,
                COUNT(*) FILTER (WHERE status = 'failed')::bigint as failed,
                COUNT(*) FILTER (WHERE status = 'skipped')::bigint as skipped,
                COALESCE(SUM(profit_loss_lamports) FILTER (WHERE status = 'executed'), 0)::bigint as total_profit_lamports,
                COALESCE(AVG(delay_ms) FILTER (WHERE status = 'executed'), 0)::float as avg_delay_ms
            FROM arb_copy_trades
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(CopyStats {
            total_copies: stats.total_copies as usize,
            executed: stats.executed as usize,
            failed: stats.failed as usize,
            skipped: stats.skipped as usize,
            total_profit_lamports: stats.total_profit_lamports,
            avg_delay_ms: stats.avg_delay_ms,
        })
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct CopyStatsRow {
    total_copies: i64,
    executed: i64,
    failed: i64,
    skipped: i64,
    total_profit_lamports: i64,
    avg_delay_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyStats {
    pub total_copies: usize,
    pub executed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub total_profit_lamports: i64,
    pub avg_delay_ms: f64,
}
