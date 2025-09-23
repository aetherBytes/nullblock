use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::consumer::{StreamConsumer, Consumer};
use rdkafka::Message;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::time::Duration;

pub mod events;

pub use events::*;

#[derive(Clone)]
pub struct KafkaConfig {
    pub bootstrap_servers: String,
    pub consumer_group_id: String,
}

impl Default for KafkaConfig {
    fn default() -> Self {
        Self {
            bootstrap_servers: std::env::var("KAFKA_BOOTSTRAP_SERVERS")
                .unwrap_or_else(|_| "localhost:9092".to_string()),
            consumer_group_id: "nullblock-agents".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct KafkaProducer {
    producer: FutureProducer,
}

impl KafkaProducer {
    pub fn new(config: &KafkaConfig) -> Result<Self> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", &config.bootstrap_servers)
            .set("message.timeout.ms", "5000")
            .set("acks", "all")
            .set("retries", "3")
            .create()?;

        Ok(Self { producer })
    }

    pub async fn publish_event<T: Serialize>(&self, topic: &str, key: &str, event: &T) -> Result<()> {
        let payload = serde_json::to_string(event)?;

        let record = FutureRecord::to(topic)
            .key(key)
            .payload(&payload);

        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(e, _)| anyhow::anyhow!("Failed to send message: {}", e))?;

        tracing::debug!("📤 Published event to topic {}: {}", topic, key);
        Ok(())
    }

    pub async fn publish_task_event(&self, event: TaskLifecycleEvent) -> Result<()> {
        self.publish_event("task.lifecycle", &event.task_id, &event).await
    }

    pub async fn publish_agent_event(&self, event: AgentStatusEvent) -> Result<()> {
        self.publish_event("agent.status", &event.agent_id.to_string(), &event).await
    }
}

pub struct KafkaConsumer {
    consumer: StreamConsumer,
}

impl KafkaConsumer {
    pub fn new(config: &KafkaConfig) -> Result<Self> {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", &config.bootstrap_servers)
            .set("group.id", &config.consumer_group_id)
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "true")
            .set("auto.offset.reset", "latest")
            .create()?;

        Ok(Self { consumer })
    }

    pub async fn subscribe_to_user_events(&self) -> Result<()> {
        self.consumer.subscribe(&["user.created", "user.updated", "user.deleted"])?;
        tracing::info!("📥 Subscribed to user events from Erebus");
        Ok(())
    }

    pub async fn start_consuming<F>(&self, mut handler: F) -> Result<()>
    where
        F: FnMut(UserEvent) -> Result<()> + Send,
    {
        use rdkafka::consumer::StreamConsumer;
        use futures::StreamExt;

        let mut stream = self.consumer.stream();

        while let Some(message) = stream.next().await {
            match message {
                Ok(m) => {
                    if let Some(payload) = m.payload() {
                        let payload_str = std::str::from_utf8(payload)?;

                        match serde_json::from_str::<UserEvent>(payload_str) {
                            Ok(user_event) => {
                                if let Err(e) = handler(user_event) {
                                    tracing::error!("Failed to handle user event: {}", e);
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to deserialize user event: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to receive message: {}", e);
                }
            }
        }

        Ok(())
    }
}