use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::execution::position_manager::{ExitConfig, MomentumData, OpenPosition, PartialExit, PositionStatus, BaseCurrency, ExitMode};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PositionRow {
    pub id: Uuid,
    pub edge_id: Uuid,
    pub strategy_id: Uuid,
    pub token_mint: String,
    pub token_symbol: Option<String>,
    pub entry_amount_base: Decimal,
    pub entry_token_amount: Decimal,
    pub entry_price: Decimal,
    pub entry_time: DateTime<Utc>,
    pub entry_tx_signature: Option<String>,
    pub current_price: Decimal,
    pub current_value_base: Decimal,
    pub unrealized_pnl: Decimal,
    pub unrealized_pnl_percent: Decimal,
    pub high_water_mark: Decimal,
    pub exit_config: serde_json::Value,
    pub partial_exits: serde_json::Value,
    pub status: String,
    pub exit_price: Option<Decimal>,
    pub exit_time: Option<DateTime<Utc>>,
    pub exit_tx_signature: Option<String>,
    pub realized_pnl: Option<Decimal>,
    pub exit_reason: Option<String>,
    pub remaining_amount_base: Option<Decimal>,
    pub remaining_token_amount: Option<Decimal>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<PositionRow> for OpenPosition {
    fn from(row: PositionRow) -> Self {
        let exit_config: ExitConfig = serde_json::from_value(row.exit_config.clone())
            .unwrap_or_default();

        let partial_exits: Vec<PartialExit> = serde_json::from_value(row.partial_exits.clone())
            .unwrap_or_default();

        let status = match row.status.as_str() {
            "open" => PositionStatus::Open,
            "pending_exit" => PositionStatus::PendingExit,
            "partially_exited" => PositionStatus::PartiallyExited,
            "closed" => PositionStatus::Closed,
            "failed" => PositionStatus::Failed,
            "orphaned" => PositionStatus::Orphaned,
            _ => PositionStatus::Open,
        };

        let entry_amount_base = decimal_to_f64(row.entry_amount_base);
        let entry_token_amount = decimal_to_f64(row.entry_token_amount);

        let remaining_amount_base = row.remaining_amount_base
            .map(decimal_to_f64)
            .unwrap_or(entry_amount_base);
        let remaining_token_amount = row.remaining_token_amount
            .map(decimal_to_f64)
            .unwrap_or(entry_token_amount);

        OpenPosition {
            id: row.id,
            edge_id: row.edge_id,
            strategy_id: row.strategy_id,
            token_mint: row.token_mint,
            token_symbol: row.token_symbol,
            entry_amount_base,
            entry_token_amount,
            entry_price: decimal_to_f64(row.entry_price),
            entry_time: row.entry_time,
            entry_tx_signature: row.entry_tx_signature,
            current_price: decimal_to_f64(row.current_price),
            current_value_base: decimal_to_f64(row.current_value_base),
            unrealized_pnl: decimal_to_f64(row.unrealized_pnl),
            unrealized_pnl_percent: decimal_to_f64(row.unrealized_pnl_percent),
            high_water_mark: decimal_to_f64(row.high_water_mark),
            exit_config,
            partial_exits,
            status,
            momentum: MomentumData::default(),
            remaining_amount_base,
            remaining_token_amount,
        }
    }
}

fn decimal_to_f64(d: Decimal) -> f64 {
    use std::str::FromStr;
    f64::from_str(&d.to_string()).unwrap_or(0.0)
}

fn f64_to_decimal(f: f64) -> Decimal {
    use std::str::FromStr;
    Decimal::from_str(&format!("{:.18}", f)).unwrap_or(Decimal::ZERO)
}

pub struct PositionRepository {
    pool: PgPool,
}

impl PositionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save_position(&self, position: &OpenPosition) -> AppResult<()> {
        let status = match position.status {
            PositionStatus::Open => "open",
            PositionStatus::PendingExit => "pending_exit",
            PositionStatus::PartiallyExited => "partially_exited",
            PositionStatus::Closed => "closed",
            PositionStatus::Failed => "failed",
            PositionStatus::Orphaned => "orphaned",
        };

        let exit_config_json = serde_json::to_value(&position.exit_config)
            .map_err(|e| AppError::Database(e.to_string()))?;

        let partial_exits_json = serde_json::to_value(&position.partial_exits)
            .map_err(|e| AppError::Database(e.to_string()))?;

        sqlx::query(r#"
            INSERT INTO arb_positions (
                id, edge_id, strategy_id, token_mint, token_symbol,
                entry_amount_base, entry_token_amount, entry_price, entry_time, entry_tx_signature,
                current_price, current_value_base, unrealized_pnl, unrealized_pnl_percent, high_water_mark,
                exit_config, partial_exits, status, remaining_amount_base, remaining_token_amount
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
            ON CONFLICT (id) DO UPDATE SET
                current_price = EXCLUDED.current_price,
                current_value_base = EXCLUDED.current_value_base,
                unrealized_pnl = EXCLUDED.unrealized_pnl,
                unrealized_pnl_percent = EXCLUDED.unrealized_pnl_percent,
                high_water_mark = EXCLUDED.high_water_mark,
                partial_exits = EXCLUDED.partial_exits,
                status = EXCLUDED.status,
                remaining_amount_base = EXCLUDED.remaining_amount_base,
                remaining_token_amount = EXCLUDED.remaining_token_amount,
                updated_at = NOW()
        "#)
            .bind(position.id)
            .bind(position.edge_id)
            .bind(position.strategy_id)
            .bind(&position.token_mint)
            .bind(&position.token_symbol)
            .bind(f64_to_decimal(position.entry_amount_base))
            .bind(f64_to_decimal(position.entry_token_amount))
            .bind(f64_to_decimal(position.entry_price))
            .bind(position.entry_time)
            .bind(&position.entry_tx_signature)
            .bind(f64_to_decimal(position.current_price))
            .bind(f64_to_decimal(position.current_value_base))
            .bind(f64_to_decimal(position.unrealized_pnl))
            .bind(f64_to_decimal(position.unrealized_pnl_percent))
            .bind(f64_to_decimal(position.high_water_mark))
            .bind(exit_config_json)
            .bind(partial_exits_json)
            .bind(status)
            .bind(f64_to_decimal(position.remaining_amount_base))
            .bind(f64_to_decimal(position.remaining_token_amount))
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn close_position(
        &self,
        position_id: Uuid,
        exit_price: f64,
        realized_pnl: f64,
        exit_reason: &str,
        exit_tx_signature: Option<&str>,
    ) -> AppResult<()> {
        sqlx::query(r#"
            UPDATE arb_positions
            SET status = 'closed',
                exit_price = $2,
                exit_time = NOW(),
                exit_tx_signature = $3,
                realized_pnl = $4,
                exit_reason = $5,
                updated_at = NOW()
            WHERE id = $1
        "#)
            .bind(position_id)
            .bind(f64_to_decimal(exit_price))
            .bind(exit_tx_signature)
            .bind(f64_to_decimal(realized_pnl))
            .bind(exit_reason)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn get_open_positions(&self) -> AppResult<Vec<OpenPosition>> {
        let rows: Vec<PositionRow> = sqlx::query_as(
            "SELECT * FROM arb_positions WHERE status = 'open' OR status = 'pending_exit' ORDER BY entry_time DESC"
        )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn get_all_positions(&self, limit: i32) -> AppResult<Vec<PositionRow>> {
        let rows: Vec<PositionRow> = sqlx::query_as(
            "SELECT * FROM arb_positions ORDER BY created_at DESC LIMIT $1"
        )
            .bind(limit)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(rows)
    }

    pub async fn get_position(&self, position_id: Uuid) -> AppResult<Option<OpenPosition>> {
        let row: Option<PositionRow> = sqlx::query_as(
            "SELECT * FROM arb_positions WHERE id = $1"
        )
            .bind(position_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn get_positions_by_mint(&self, token_mint: &str) -> AppResult<Vec<OpenPosition>> {
        let rows: Vec<PositionRow> = sqlx::query_as(
            "SELECT * FROM arb_positions WHERE token_mint = $1 AND (status = 'open' OR status = 'pending_exit') ORDER BY entry_time DESC"
        )
            .bind(token_mint)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_price(&self, position_id: Uuid, current_price: f64, unrealized_pnl: f64, unrealized_pnl_percent: f64, high_water_mark: f64) -> AppResult<()> {
        sqlx::query(r#"
            UPDATE arb_positions
            SET current_price = $2,
                unrealized_pnl = $3,
                unrealized_pnl_percent = $4,
                high_water_mark = $5,
                current_value_base = entry_amount_base * (1 + $4 / 100),
                updated_at = NOW()
            WHERE id = $1
        "#)
            .bind(position_id)
            .bind(f64_to_decimal(current_price))
            .bind(f64_to_decimal(unrealized_pnl))
            .bind(f64_to_decimal(unrealized_pnl_percent))
            .bind(f64_to_decimal(high_water_mark))
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn get_closed_positions_since(&self, since: DateTime<Utc>) -> AppResult<Vec<OpenPosition>> {
        let rows: Vec<PositionRow> = sqlx::query_as(
            "SELECT * FROM arb_positions WHERE status = 'closed' AND exit_time >= $1 ORDER BY exit_time DESC"
        )
            .bind(since)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn update_status(&self, position_id: Uuid, status: &str) -> AppResult<()> {
        sqlx::query(r#"
            UPDATE arb_positions
            SET status = $2,
                updated_at = NOW()
            WHERE id = $1
        "#)
            .bind(position_id)
            .bind(status)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn get_pnl_stats(&self) -> AppResult<PnLStats> {
        #[derive(sqlx::FromRow)]
        struct StatsRow {
            exit_reason: Option<String>,
            cnt: i64,
            total_pnl: Option<Decimal>,
        }

        let rows: Vec<StatsRow> = sqlx::query_as(
            r#"SELECT exit_reason, COUNT(*) as cnt, SUM(realized_pnl) as total_pnl
               FROM arb_positions
               WHERE status = 'closed'
               GROUP BY exit_reason"#
        )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut stats = PnLStats::default();
        for row in rows {
            let pnl = decimal_to_f64(row.total_pnl.unwrap_or(Decimal::ZERO));
            let cnt = row.cnt as u32;
            match row.exit_reason.as_deref() {
                Some("TakeProfit") => {
                    stats.take_profits = cnt;
                    stats.take_profit_pnl = pnl;
                }
                Some("StopLoss") => {
                    stats.stop_losses = cnt;
                    stats.stop_loss_pnl = pnl;
                }
                Some("Manual") => {
                    stats.manual_exits = cnt;
                    stats.manual_pnl = pnl;
                }
                Some("TimeLimit") => {
                    stats.time_exits = cnt;
                    stats.time_exit_pnl = pnl;
                }
                Some("TrailingStop") => {
                    stats.trailing_stops = cnt;
                    stats.trailing_stop_pnl = pnl;
                }
                _ => {}
            }
        }
        stats.total_pnl = stats.take_profit_pnl + stats.stop_loss_pnl + stats.manual_pnl + stats.time_exit_pnl + stats.trailing_stop_pnl;
        stats.total_trades = stats.take_profits + stats.stop_losses + stats.manual_exits + stats.time_exits + stats.trailing_stops;

        // Get today's and this week's PnL
        let today_row: Option<(Option<Decimal>,)> = sqlx::query_as(
            r#"SELECT SUM(realized_pnl) FROM arb_positions
               WHERE status = 'closed' AND exit_time >= CURRENT_DATE"#
        )
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        stats.today_pnl = today_row.and_then(|r| r.0).map(decimal_to_f64).unwrap_or(0.0);

        let week_row: Option<(Option<Decimal>,)> = sqlx::query_as(
            r#"SELECT SUM(realized_pnl) FROM arb_positions
               WHERE status = 'closed' AND exit_time >= CURRENT_DATE - INTERVAL '7 days'"#
        )
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        stats.week_pnl = week_row.and_then(|r| r.0).map(decimal_to_f64).unwrap_or(0.0);

        // Get best and worst trades
        let best_row: Option<(Option<Decimal>, Option<String>)> = sqlx::query_as(
            r#"SELECT realized_pnl, token_symbol FROM arb_positions
               WHERE status = 'closed' AND realized_pnl IS NOT NULL
               ORDER BY realized_pnl DESC LIMIT 1"#
        )
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        if let Some((pnl, symbol)) = best_row {
            stats.best_trade_pnl = pnl.map(decimal_to_f64).unwrap_or(0.0);
            stats.best_trade_symbol = symbol;
        }

        let worst_row: Option<(Option<Decimal>, Option<String>)> = sqlx::query_as(
            r#"SELECT realized_pnl, token_symbol FROM arb_positions
               WHERE status = 'closed' AND realized_pnl IS NOT NULL
               ORDER BY realized_pnl ASC LIMIT 1"#
        )
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        if let Some((pnl, symbol)) = worst_row {
            stats.worst_trade_pnl = pnl.map(decimal_to_f64).unwrap_or(0.0);
            stats.worst_trade_symbol = symbol;
        }

        // Get avg hold time
        let avg_hold: Option<(Option<f64>,)> = sqlx::query_as(
            r#"SELECT AVG(EXTRACT(EPOCH FROM (exit_time - entry_time)) / 60.0)
               FROM arb_positions WHERE status = 'closed' AND exit_time IS NOT NULL"#
        )
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        stats.avg_hold_minutes = avg_hold.and_then(|r| r.0).unwrap_or(0.0);

        Ok(stats)
    }

    pub async fn get_recent_trades(&self, limit: i32) -> AppResult<Vec<RecentTrade>> {
        #[derive(sqlx::FromRow)]
        struct TradeRow {
            token_symbol: Option<String>,
            token_mint: String,
            realized_pnl: Option<Decimal>,
            exit_reason: Option<String>,
            exit_time: Option<DateTime<Utc>>,
            entry_amount_base: Decimal,
        }

        let rows: Vec<TradeRow> = sqlx::query_as(
            r#"SELECT token_symbol, token_mint, realized_pnl, exit_reason, exit_time, entry_amount_base
               FROM arb_positions
               WHERE status = 'closed' AND exit_time IS NOT NULL
               ORDER BY exit_time DESC
               LIMIT $1"#
        )
            .bind(limit)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(rows.into_iter().map(|r| RecentTrade {
            symbol: r.token_symbol.unwrap_or_else(|| r.token_mint[..8].to_string()),
            pnl: decimal_to_f64(r.realized_pnl.unwrap_or(Decimal::ZERO)),
            reason: r.exit_reason.unwrap_or_else(|| "Unknown".to_string()),
            time: r.exit_time,
            entry_sol: decimal_to_f64(r.entry_amount_base),
        }).collect())
    }

    pub async fn get_orphaned_position_by_mint(&self, token_mint: &str) -> AppResult<Option<OpenPosition>> {
        let row: Option<PositionRow> = sqlx::query_as(
            "SELECT * FROM arb_positions WHERE token_mint = $1 AND status = 'orphaned' ORDER BY entry_time DESC LIMIT 1"
        )
            .bind(token_mint)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(row.map(|r| r.into()))
    }

    pub async fn get_all_orphaned_positions(&self) -> AppResult<Vec<OpenPosition>> {
        let rows: Vec<PositionRow> = sqlx::query_as(
            "SELECT * FROM arb_positions WHERE status = 'orphaned' ORDER BY entry_time DESC"
        )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn reactivate_position(&self, position_id: Uuid, new_token_balance: f64, new_price: f64) -> AppResult<()> {
        sqlx::query(r#"
            UPDATE arb_positions
            SET status = 'open',
                entry_token_amount = $2,
                current_price = $3,
                high_water_mark = $3,
                unrealized_pnl = 0,
                unrealized_pnl_percent = 0,
                updated_at = NOW()
            WHERE id = $1
        "#)
            .bind(position_id)
            .bind(f64_to_decimal(new_token_balance))
            .bind(f64_to_decimal(new_price))
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PnLStats {
    pub total_pnl: f64,
    pub today_pnl: f64,
    pub week_pnl: f64,
    pub total_trades: u32,
    pub take_profits: u32,
    pub take_profit_pnl: f64,
    pub stop_losses: u32,
    pub stop_loss_pnl: f64,
    pub manual_exits: u32,
    pub manual_pnl: f64,
    pub time_exits: u32,
    pub time_exit_pnl: f64,
    pub trailing_stops: u32,
    pub trailing_stop_pnl: f64,
    pub best_trade_pnl: f64,
    pub best_trade_symbol: Option<String>,
    pub worst_trade_pnl: f64,
    pub worst_trade_symbol: Option<String>,
    pub avg_hold_minutes: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentTrade {
    pub symbol: String,
    pub pnl: f64,
    pub reason: String,
    pub time: Option<DateTime<Utc>>,
    pub entry_sol: f64,
}
