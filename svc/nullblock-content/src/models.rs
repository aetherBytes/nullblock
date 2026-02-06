use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ContentQueue {
    pub id: Uuid,
    pub theme: String,
    pub text: String,
    #[sqlx(default)]
    pub tags: Vec<String>,
    pub image_prompt: Option<String>,
    pub image_path: Option<String>,
    pub status: ContentStatus,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub posted_at: Option<DateTime<Utc>>,
    pub tweet_url: Option<String>,
    #[sqlx(json)]
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "content_status", rename_all = "lowercase")]
pub enum ContentStatus {
    Pending,
    Approved,
    Posted,
    Failed,
    Deleted,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateContentRequest {
    pub theme: String,
    pub include_image: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateContentResponse {
    pub id: Uuid,
    pub theme: String,
    pub text: String,
    pub tags: Vec<String>,
    pub image_prompt: Option<String>,
    pub status: ContentStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ContentMetrics {
    pub id: Uuid,
    pub content_id: Uuid,
    pub likes: i32,
    pub retweets: i32,
    pub replies: i32,
    pub impressions: i64,
    pub engagement_rate: Option<f64>,
    pub fetched_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ContentTemplate {
    pub id: Uuid,
    pub theme: String,
    pub name: String,
    pub description: Option<String>,
    #[sqlx(json)]
    pub templates: Value,
    #[sqlx(json)]
    pub insights: Value,
    #[sqlx(json)]
    pub metadata: Value,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostContentRequest {
    pub content_id: Uuid,
    pub platform: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostContentResponse {
    pub success: bool,
    pub content_id: Uuid,
    pub platform: String,
    pub url: Option<String>,
    pub posted_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContentQueueQuery {
    pub status: Option<ContentStatus>,
    pub theme: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContentQueueResponse {
    pub items: Vec<ContentQueue>,
    pub total: i64,
}
