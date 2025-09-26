use sqlx::PgPool;
use uuid::Uuid;
use anyhow::Result;
use chrono::{DateTime, Utc};

use crate::database::models::UserReferenceEntity;

pub struct UserReferenceRepository {
    pool: PgPool,
}

impl UserReferenceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_from_kafka_event(
        &self,
        user_id: Uuid,
        source_identifier: Option<&str>,
        chain: Option<&str>,
        source_type: &serde_json::Value,
        wallet_type: Option<&str>,
        email: Option<&str>,
        metadata: &serde_json::Value,
        erebus_created_at: Option<DateTime<Utc>>,
        erebus_updated_at: Option<DateTime<Utc>>,
    ) -> Result<UserReferenceEntity> {
        let now = Utc::now();

        let user_ref = sqlx::query_as::<_, UserReferenceEntity>(
            r#"
            INSERT INTO user_references (
                id, source_identifier, chain, user_type, source_type, wallet_type, email, metadata,
                preferences, is_active
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (source_identifier, chain) DO UPDATE SET
                user_type = EXCLUDED.user_type,
                source_type = EXCLUDED.source_type,
                wallet_type = EXCLUDED.wallet_type,
                email = EXCLUDED.email,
                metadata = EXCLUDED.metadata,
                is_active = EXCLUDED.is_active
            RETURNING *
            "#
        )
        .bind(user_id)
        .bind(source_identifier)
        .bind(chain)
        .bind("external") // user_type - default value
        .bind(source_type)
        .bind(wallet_type)
        .bind(email)
        .bind(metadata)
        .bind(serde_json::json!({})) // preferences
        .bind(true) // is_active
        .fetch_one(&self.pool)
        .await?;

        Ok(user_ref)
    }

    pub async fn get_by_id(&self, user_id: &Uuid) -> Result<Option<UserReferenceEntity>> {
        let user_ref = sqlx::query_as::<_, UserReferenceEntity>(
            "SELECT * FROM user_references WHERE id = $1 AND is_active = true"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user_ref)
    }

    pub async fn get_by_source(&self, source_identifier: &str, chain: &str) -> Result<Option<UserReferenceEntity>> {
        let user_ref = sqlx::query_as::<_, UserReferenceEntity>(
            "SELECT * FROM user_references WHERE source_identifier = $1 AND chain = $2 AND is_active = true"
        )
        .bind(source_identifier)
        .bind(chain)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user_ref)
    }

    pub async fn get_by_email(&self, email: &str) -> Result<Option<UserReferenceEntity>> {
        let user_ref = sqlx::query_as::<_, UserReferenceEntity>(
            "SELECT * FROM user_references WHERE email = $1 AND is_active = true"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user_ref)
    }

    pub async fn deactivate_user(&self, user_id: &Uuid) -> Result<Option<UserReferenceEntity>> {
        let now = Utc::now();

        let user_ref = sqlx::query_as::<_, UserReferenceEntity>(
            r#"
            UPDATE user_references SET
                is_active = false,
                synced_at = $2
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(user_id)
        .bind(now)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user_ref)
    }

    pub async fn list_active(&self, limit: Option<i64>) -> Result<Vec<UserReferenceEntity>> {
        let limit_val = limit.unwrap_or(100);

        let user_refs = sqlx::query_as::<_, UserReferenceEntity>(
            "SELECT * FROM user_references WHERE is_active = true ORDER BY synced_at DESC LIMIT $1"
        )
        .bind(limit_val)
        .fetch_all(&self.pool)
        .await?;

        Ok(user_refs)
    }

    pub async fn create(&self, user_ref: &crate::models::UserReference) -> Result<UserReferenceEntity> {
        let now = Utc::now();

        let user_entity = sqlx::query_as::<_, UserReferenceEntity>(
            r#"
            INSERT INTO user_references (
                id, source_identifier, chain, user_type, source_type, wallet_type, email, metadata,
                preferences, is_active
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#
        )
        .bind(user_ref.id)
        .bind(&user_ref.source_identifier)
        .bind(&user_ref.chain)
        .bind("external") // user_type - default value
        .bind(&user_ref.source_type)
        .bind(&user_ref.wallet_type)
        .bind(None::<String>) // email
        .bind(serde_json::json!({})) // metadata
        .bind(serde_json::json!({})) // preferences
        .bind(true) // is_active
        .fetch_one(&self.pool)
        .await?;

        Ok(user_entity)
    }

    pub async fn list(
        &self,
        wallet_address: Option<&str>,
        chain: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<UserReferenceEntity>> {
        let limit_val = limit.unwrap_or(100) as i64;

        let query = if let (Some(wallet), Some(chain)) = (wallet_address, chain) {
            sqlx::query_as::<_, UserReferenceEntity>(
                "SELECT * FROM user_references WHERE wallet_address = $1 AND chain = $2 AND is_active = true ORDER BY synced_at DESC LIMIT $3"
            )
            .bind(wallet)
            .bind(chain)
            .bind(limit_val)
        } else if let Some(wallet) = wallet_address {
            sqlx::query_as::<_, UserReferenceEntity>(
                "SELECT * FROM user_references WHERE wallet_address = $1 AND is_active = true ORDER BY synced_at DESC LIMIT $2"
            )
            .bind(wallet)
            .bind(limit_val)
        } else if let Some(chain) = chain {
            sqlx::query_as::<_, UserReferenceEntity>(
                "SELECT * FROM user_references WHERE chain = $1 AND is_active = true ORDER BY synced_at DESC LIMIT $2"
            )
            .bind(chain)
            .bind(limit_val)
        } else {
            sqlx::query_as::<_, UserReferenceEntity>(
                "SELECT * FROM user_references WHERE is_active = true ORDER BY synced_at DESC LIMIT $1"
            )
            .bind(limit_val)
        };

        let user_refs = query.fetch_all(&self.pool).await?;
        Ok(user_refs)
    }
}