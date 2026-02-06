use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum ContentEvent {
    #[serde(rename = "content.generated")]
    Generated {
        content_id: Uuid,
        theme: String,
        status: String,
        metadata: serde_json::Value,
        timestamp: DateTime<Utc>,
    },
    #[serde(rename = "content.posted")]
    Posted {
        content_id: Uuid,
        platform: String,
        url: String,
        timestamp: DateTime<Utc>,
    },
    #[serde(rename = "content.failed")]
    Failed {
        content_id: Uuid,
        error: String,
        timestamp: DateTime<Utc>,
    },
}

#[async_trait::async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish(&self, event: ContentEvent) -> Result<(), String>;
}

pub struct NoOpPublisher;

#[async_trait::async_trait]
impl EventPublisher for NoOpPublisher {
    async fn publish(&self, event: ContentEvent) -> Result<(), String> {
        tracing::info!("Event published (no-op): {:?}", event);
        Ok(())
    }
}

pub struct HttpEventPublisher {
    client: reqwest::Client,
    endpoint: String,
}

impl HttpEventPublisher {
    pub fn new(endpoint: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            endpoint,
        }
    }
}

#[async_trait::async_trait]
impl EventPublisher for HttpEventPublisher {
    async fn publish(&self, event: ContentEvent) -> Result<(), String> {
        self.client
            .post(&self.endpoint)
            .json(&event)
            .send()
            .await
            .map_err(|e| format!("Failed to publish event: {}", e))?;

        tracing::info!("Event published to {}: {:?}", self.endpoint, event);
        Ok(())
    }
}
