pub mod repositories;

pub use repositories::{
    EdgeRepository, StrategyRepository, TradeRepository,
};

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::error::AppResult;

pub async fn create_pool(database_url: &str) -> AppResult<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .connect(database_url)
        .await
        .map_err(|e| crate::error::AppError::Database(e.to_string()))?;

    Ok(pool)
}
