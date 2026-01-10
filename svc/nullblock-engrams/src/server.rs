use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Arc;

use crate::config::Config;
use crate::database::repositories::EngramRepository;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db_pool: PgPool,
    pub engram_repo: Arc<EngramRepository>,
}

impl AppState {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        // Create database connection pool
        let db_pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(&config.database_url)
            .await?;

        tracing::info!("✅ Database connection pool created");

        // Note: Migrations are run via shell script (start-engrams.sh) before service starts
        // This allows for idempotent migrations with IF NOT EXISTS clauses
        tracing::info!("✅ Database ready (migrations handled externally)");

        // Create repositories
        let engram_repo = Arc::new(EngramRepository::new(db_pool.clone()));

        Ok(Self {
            config,
            db_pool,
            engram_repo,
        })
    }
}
