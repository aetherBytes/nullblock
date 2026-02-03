use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::ContentError;
use crate::models::{ContentMetrics, ContentQueue, ContentTemplate};

pub struct ContentRepository {
    pool: Arc<PgPool>,
}

impl ContentRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn create_content(
        &self,
        theme: &str,
        text: &str,
        tags: &[String],
        image_prompt: Option<&str>,
        metadata: &serde_json::Value,
    ) -> Result<ContentQueue, ContentError> {
        let row = sqlx::query_as::<_, ContentQueue>(
            r#"
            INSERT INTO content_queue (theme, text, tags, image_prompt, metadata)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(theme)
        .bind(text)
        .bind(tags)
        .bind(image_prompt)
        .bind(metadata)
        .fetch_one(self.pool.as_ref())
        .await?;

        Ok(row)
    }

    pub async fn create_metrics(
        &self,
        content_id: Uuid,
        likes: i32,
        retweets: i32,
        replies: i32,
        impressions: i64,
        engagement_rate: Option<f64>,
    ) -> Result<ContentMetrics, ContentError> {
        let row = sqlx::query_as::<_, ContentMetrics>(
            r#"
            INSERT INTO content_metrics (content_id, likes, retweets, replies, impressions, engagement_rate)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(content_id)
        .bind(likes)
        .bind(retweets)
        .bind(replies)
        .bind(impressions)
        .bind(engagement_rate)
        .fetch_one(self.pool.as_ref())
        .await?;

        Ok(row)
    }

    pub async fn get_content(&self, id: Uuid) -> Result<Option<ContentQueue>, ContentError> {
        let row = sqlx::query_as::<_, ContentQueue>(
            "SELECT * FROM content_queue WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(self.pool.as_ref())
        .await?;

        Ok(row)
    }

    pub async fn list_pending(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ContentQueue>, ContentError> {
        let rows = sqlx::query_as::<_, ContentQueue>(
            "SELECT * FROM content_queue WHERE status = 'pending' ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.as_ref())
        .await?;

        Ok(rows)
    }

    pub async fn list_by_theme(
        &self,
        theme: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ContentQueue>, ContentError> {
        let rows = sqlx::query_as::<_, ContentQueue>(
            "SELECT * FROM content_queue WHERE theme = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(theme)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.as_ref())
        .await?;

        Ok(rows)
    }

    pub async fn list_posted(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ContentQueue>, ContentError> {
        let rows = sqlx::query_as::<_, ContentQueue>(
            "SELECT * FROM content_queue WHERE status = 'posted' ORDER BY posted_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.as_ref())
        .await?;

        Ok(rows)
    }

    pub async fn get_metrics(
        &self,
        content_id: Uuid,
    ) -> Result<Option<ContentMetrics>, ContentError> {
        let row = sqlx::query_as::<_, ContentMetrics>(
            "SELECT * FROM content_metrics WHERE content_id = $1 ORDER BY fetched_at DESC LIMIT 1",
        )
        .bind(content_id)
        .fetch_optional(self.pool.as_ref())
        .await?;

        Ok(row)
    }

    pub async fn update_status(
        &self,
        id: Uuid,
        status: &str,
    ) -> Result<ContentQueue, ContentError> {
        let row = sqlx::query_as::<_, ContentQueue>(
            "UPDATE content_queue SET status = $1, updated_at = NOW() WHERE id = $2 RETURNING *",
        )
        .bind(status)
        .bind(id)
        .fetch_one(self.pool.as_ref())
        .await?;

        Ok(row)
    }

    pub async fn mark_posted(
        &self,
        id: Uuid,
        tweet_url: &str,
    ) -> Result<ContentQueue, ContentError> {
        let row = sqlx::query_as::<_, ContentQueue>(
            r#"
            UPDATE content_queue
            SET status = 'posted', posted_at = NOW(), tweet_url = $1, updated_at = NOW()
            WHERE id = $2
            RETURNING *
            "#,
        )
        .bind(tweet_url)
        .bind(id)
        .fetch_one(self.pool.as_ref())
        .await?;

        Ok(row)
    }

    pub async fn set_image_path(
        &self,
        id: Uuid,
        image_path: &str,
    ) -> Result<ContentQueue, ContentError> {
        let row = sqlx::query_as::<_, ContentQueue>(
            "UPDATE content_queue SET image_path = $1, updated_at = NOW() WHERE id = $2 RETURNING *",
        )
        .bind(image_path)
        .bind(id)
        .fetch_one(self.pool.as_ref())
        .await?;

        Ok(row)
    }

    pub async fn delete_content(&self, id: Uuid) -> Result<(), ContentError> {
        sqlx::query("DELETE FROM content_queue WHERE id = $1")
            .bind(id)
            .execute(self.pool.as_ref())
            .await?;

        Ok(())
    }

    pub async fn get_template(
        &self,
        id: Uuid,
    ) -> Result<Option<ContentTemplate>, ContentError> {
        let row = sqlx::query_as::<_, ContentTemplate>(
            "SELECT * FROM content_templates WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(self.pool.as_ref())
        .await?;

        Ok(row)
    }

    pub async fn list_templates(
        &self,
        active_only: bool,
    ) -> Result<Vec<ContentTemplate>, ContentError> {
        let rows = if active_only {
            sqlx::query_as::<_, ContentTemplate>(
                "SELECT * FROM content_templates WHERE active = true ORDER BY theme, name",
            )
            .fetch_all(self.pool.as_ref())
            .await?
        } else {
            sqlx::query_as::<_, ContentTemplate>(
                "SELECT * FROM content_templates ORDER BY theme, name",
            )
            .fetch_all(self.pool.as_ref())
            .await?
        };

        Ok(rows)
    }

    pub async fn create_template(
        &self,
        theme: &str,
        name: &str,
        description: Option<&str>,
        templates: &serde_json::Value,
        insights: &serde_json::Value,
        metadata: &serde_json::Value,
    ) -> Result<ContentTemplate, ContentError> {
        let row = sqlx::query_as::<_, ContentTemplate>(
            r#"
            INSERT INTO content_templates (theme, name, description, templates, insights, metadata)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(theme)
        .bind(name)
        .bind(description)
        .bind(templates)
        .bind(insights)
        .bind(metadata)
        .fetch_one(self.pool.as_ref())
        .await?;

        Ok(row)
    }
}
