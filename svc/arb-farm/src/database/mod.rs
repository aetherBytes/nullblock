pub mod repositories;

pub use repositories::{
    CreateTradeRecord, EdgeRepository, PendingExitSignalRow, PositionRepository,
    SettingsRepository, StrategyRepository, TradeRepository,
};

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::error::AppResult;

const DB_MAX_CONNECTIONS: u32 = 30;
const DB_ACQUIRE_TIMEOUT_SECS: u64 = 30;

pub async fn create_pool(database_url: &str) -> AppResult<PgPool> {
    const MAX_RETRIES: u32 = 3;
    let mut last_err = None;

    for attempt in 0..=MAX_RETRIES {
        match PgPoolOptions::new()
            .max_connections(DB_MAX_CONNECTIONS)
            .acquire_timeout(std::time::Duration::from_secs(DB_ACQUIRE_TIMEOUT_SECS))
            .connect(database_url)
            .await
        {
            Ok(pool) => return Ok(pool),
            Err(e) => {
                last_err = Some(e);
                if attempt < MAX_RETRIES {
                    let backoff = std::time::Duration::from_secs(1 << attempt);
                    tracing::warn!(
                        "Database connection attempt {} failed, retrying in {:?}...",
                        attempt + 1,
                        backoff
                    );
                    tokio::time::sleep(backoff).await;
                }
            }
        }
    }

    Err(crate::error::AppError::Database(format!(
        "Failed to connect after {} retries: {}",
        MAX_RETRIES,
        last_err.map(|e| e.to_string()).unwrap_or_default()
    )))
}
