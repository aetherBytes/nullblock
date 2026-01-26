pub mod repositories;

pub use repositories::{
    EdgeRepository, PositionRepository, StrategyRepository, TradeRepository,
    PendingExitSignalRow,
};

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::error::AppResult;

const DB_MAX_CONNECTIONS: u32 = 30;
const DB_ACQUIRE_TIMEOUT_SECS: u64 = 30;

pub async fn create_pool(database_url: &str) -> AppResult<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(DB_MAX_CONNECTIONS)
        .acquire_timeout(std::time::Duration::from_secs(DB_ACQUIRE_TIMEOUT_SECS))
        .connect(database_url)
        .await
        .map_err(|e| crate::error::AppError::Database(e.to_string()))?;

    Ok(pool)
}
