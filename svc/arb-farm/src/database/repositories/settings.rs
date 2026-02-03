use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SettingRecord {
    pub key: String,
    pub value: String,
    pub updated_at: DateTime<Utc>,
}

pub struct SettingsRepository {
    pool: PgPool,
}

impl SettingsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, key: &str) -> AppResult<Option<SettingRecord>> {
        sqlx::query_as::<_, SettingRecord>(
            "SELECT key, value, updated_at FROM arb_runtime_settings WHERE key = $1",
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))
    }

    pub async fn get_bool(&self, key: &str) -> AppResult<Option<bool>> {
        Ok(self.get(key).await?.map(|r| r.value == "true"))
    }

    pub async fn set_bool(&self, key: &str, value: bool) -> AppResult<()> {
        let val = if value { "true" } else { "false" };
        self.set(key, val).await
    }

    pub async fn set(&self, key: &str, value: &str) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO arb_runtime_settings (key, value, updated_at)
            VALUES ($1, $2, NOW())
            ON CONFLICT (key) DO UPDATE SET value = $2, updated_at = NOW()
            "#,
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }
}
