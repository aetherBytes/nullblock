use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::{
    CreateEngramRequest, Engram, EngramHistory, ForkEngramRequest, SearchEngramsRequest,
    UpdateEngramRequest,
};

pub struct EngramRepository {
    pool: PgPool,
}

impl EngramRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, req: &CreateEngramRequest) -> AppResult<Engram> {
        let id = Uuid::new_v4();

        let engram = sqlx::query_as::<_, Engram>(
            r#"
            INSERT INTO engrams (
                id, wallet_address, engram_type, key, tags, content, summary,
                priority, ttl_seconds, created_by, lineage_root_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $1)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&req.wallet_address)
        .bind(&req.engram_type)
        .bind(&req.key)
        .bind(&req.tags)
        .bind(&req.content)
        .bind(&req.summary)
        .bind(req.priority)
        .bind(req.ttl_seconds)
        .bind(&req.created_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(engram)
    }

    pub async fn get_by_id(&self, id: Uuid) -> AppResult<Engram> {
        let engram = sqlx::query_as::<_, Engram>(
            r#"
            UPDATE engrams SET accessed_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Engram {} not found", id)))?;

        Ok(engram)
    }

    pub async fn get_by_wallet(&self, wallet: &str) -> AppResult<Vec<Engram>> {
        let engrams = sqlx::query_as::<_, Engram>(
            r#"
            SELECT * FROM engrams
            WHERE wallet_address = $1
            ORDER BY priority DESC, updated_at DESC
            "#,
        )
        .bind(wallet)
        .fetch_all(&self.pool)
        .await?;

        Ok(engrams)
    }

    pub async fn get_by_wallet_and_key(&self, wallet: &str, key: &str) -> AppResult<Engram> {
        let engram = sqlx::query_as::<_, Engram>(
            r#"
            UPDATE engrams SET accessed_at = NOW()
            WHERE wallet_address = $1 AND key = $2
            ORDER BY version DESC
            LIMIT 1
            RETURNING *
            "#,
        )
        .bind(wallet)
        .bind(key)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!("Engram with key '{}' not found for wallet", key))
        })?;

        Ok(engram)
    }

    pub async fn update(&self, id: Uuid, req: &UpdateEngramRequest) -> AppResult<Engram> {
        // Get current engram to get version
        let current = self.get_by_id(id).await?;

        // Save current version to history
        sqlx::query(
            r#"
            INSERT INTO engram_history (id, engram_id, version, content, changed_by, change_reason)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(id)
        .bind(current.version)
        .bind(&current.content)
        .bind(&req.changed_by)
        .bind(&req.reason)
        .execute(&self.pool)
        .await?;

        // Update engram with new version
        let engram = sqlx::query_as::<_, Engram>(
            r#"
            UPDATE engrams
            SET content = $1, summary = $2, version = version + 1, updated_at = NOW()
            WHERE id = $3
            RETURNING *
            "#,
        )
        .bind(&req.content)
        .bind(&req.summary)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(engram)
    }

    pub async fn delete(&self, id: Uuid) -> AppResult<()> {
        let result = sqlx::query("DELETE FROM engrams WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Engram {} not found", id)));
        }

        Ok(())
    }

    pub async fn search(&self, req: &SearchEngramsRequest) -> AppResult<(Vec<Engram>, i64)> {
        let limit = req.limit.unwrap_or(50);
        let offset = req.offset.unwrap_or(0);

        // Build dynamic query
        let mut conditions = vec!["1=1".to_string()];
        let mut param_count = 0;

        if req.wallet_address.is_some() {
            param_count += 1;
            conditions.push(format!("wallet_address = ${}", param_count));
        }
        if req.engram_type.is_some() {
            param_count += 1;
            conditions.push(format!("engram_type = ${}", param_count));
        }
        if req.is_public.is_some() {
            param_count += 1;
            conditions.push(format!("is_public = ${}", param_count));
        }
        if req.tags.is_some() {
            param_count += 1;
            conditions.push(format!("tags && ${}", param_count));
        }

        let where_clause = conditions.join(" AND ");

        // Count query
        let count_query = format!(
            "SELECT COUNT(*) as count FROM engrams WHERE {}",
            where_clause
        );

        // Build the count query with bindings
        let mut count_builder = sqlx::query_scalar::<_, i64>(&count_query);

        if let Some(ref wallet) = req.wallet_address {
            count_builder = count_builder.bind(wallet);
        }
        if let Some(ref engram_type) = req.engram_type {
            count_builder = count_builder.bind(engram_type);
        }
        if let Some(is_public) = req.is_public {
            count_builder = count_builder.bind(is_public);
        }
        if let Some(ref tags) = req.tags {
            count_builder = count_builder.bind(tags);
        }

        let total = count_builder.fetch_one(&self.pool).await?;

        // Main query
        let query = format!(
            r#"
            SELECT * FROM engrams
            WHERE {}
            ORDER BY priority DESC, updated_at DESC
            LIMIT {} OFFSET {}
            "#,
            where_clause, limit, offset
        );

        let mut builder = sqlx::query_as::<_, Engram>(&query);

        if let Some(ref wallet) = req.wallet_address {
            builder = builder.bind(wallet);
        }
        if let Some(ref engram_type) = req.engram_type {
            builder = builder.bind(engram_type);
        }
        if let Some(is_public) = req.is_public {
            builder = builder.bind(is_public);
        }
        if let Some(ref tags) = req.tags {
            builder = builder.bind(tags);
        }

        let engrams = builder.fetch_all(&self.pool).await?;

        Ok((engrams, total))
    }

    pub async fn fork(&self, id: Uuid, req: &ForkEngramRequest) -> AppResult<Engram> {
        let source = self.get_by_id(id).await?;

        // Check if source is public or belongs to target wallet
        if !source.is_public && source.wallet_address != req.target_wallet {
            return Err(AppError::Forbidden(
                "Cannot fork private engram from another wallet".to_string(),
            ));
        }

        let new_id = Uuid::new_v4();
        let new_key = req.new_key.clone().unwrap_or(source.key.clone());

        let engram = sqlx::query_as::<_, Engram>(
            r#"
            INSERT INTO engrams (
                id, wallet_address, engram_type, key, tags, content, summary,
                version, parent_id, lineage_root_id, priority, ttl_seconds, created_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, 1, $8, $9, $10, $11, $12)
            RETURNING *
            "#,
        )
        .bind(new_id)
        .bind(&req.target_wallet)
        .bind(&source.engram_type)
        .bind(&new_key)
        .bind(&source.tags)
        .bind(&source.content)
        .bind(&source.summary)
        .bind(id) // parent_id
        .bind(source.lineage_root_id.unwrap_or(id)) // lineage_root_id
        .bind(source.priority)
        .bind(source.ttl_seconds)
        .bind(format!("forked_from:{}", id))
        .fetch_one(&self.pool)
        .await?;

        Ok(engram)
    }

    pub async fn publish(&self, id: Uuid) -> AppResult<Engram> {
        let engram = sqlx::query_as::<_, Engram>(
            r#"
            UPDATE engrams
            SET is_public = true, updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Engram {} not found", id)))?;

        Ok(engram)
    }

    pub async fn list(&self, limit: i64, offset: i64) -> AppResult<(Vec<Engram>, i64)> {
        let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM engrams")
            .fetch_one(&self.pool)
            .await?;

        let engrams = sqlx::query_as::<_, Engram>(
            r#"
            SELECT * FROM engrams
            ORDER BY updated_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok((engrams, total))
    }

    pub async fn get_history(&self, engram_id: Uuid) -> AppResult<Vec<EngramHistory>> {
        let history = sqlx::query_as::<_, EngramHistory>(
            r#"
            SELECT * FROM engram_history
            WHERE engram_id = $1
            ORDER BY version DESC
            "#,
        )
        .bind(engram_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(history)
    }
}
