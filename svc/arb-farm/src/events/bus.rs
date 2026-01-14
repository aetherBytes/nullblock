use sqlx::PgPool;
use tokio::sync::broadcast;
use uuid::Uuid;

use super::{topics::matches_pattern, ArbEvent, EventSource};
use crate::error::{AppError, AppResult};

pub struct EventBus {
    tx: broadcast::Sender<ArbEvent>,
    db_pool: PgPool,
}

impl EventBus {
    pub fn new(tx: broadcast::Sender<ArbEvent>, db_pool: PgPool) -> Self {
        Self { tx, db_pool }
    }

    pub async fn publish(&self, event: ArbEvent) -> AppResult<()> {
        self.persist_event(&event).await?;

        if self.tx.receiver_count() > 0 {
            self.tx
                .send(event)
                .map_err(|e| AppError::EventBus(format!("Failed to broadcast event: {}", e)))?;
        }

        Ok(())
    }

    pub fn subscribe(&self) -> EventSubscription {
        EventSubscription {
            rx: self.tx.subscribe(),
            filters: Vec::new(),
        }
    }

    pub fn subscribe_to(&self, topics: Vec<String>) -> EventSubscription {
        EventSubscription {
            rx: self.tx.subscribe(),
            filters: topics,
        }
    }

    async fn persist_event(&self, event: &ArbEvent) -> AppResult<()> {
        let source_type = match &event.source {
            EventSource::Agent(_) => "agent",
            EventSource::Tool(_) => "tool",
            EventSource::External(_) => "external",
            EventSource::System => "system",
        };

        let source_id = match &event.source {
            EventSource::Agent(t) => t.to_string(),
            EventSource::Tool(s) => s.clone(),
            EventSource::External(s) => s.clone(),
            EventSource::System => "system".to_string(),
        };

        sqlx::query(
            r#"
            INSERT INTO arb_events (id, event_type, source_type, source_id, topic, payload, correlation_id, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(event.id)
        .bind(&event.event_type)
        .bind(source_type)
        .bind(&source_id)
        .bind(&event.topic)
        .bind(&event.payload)
        .bind(event.correlation_id)
        .bind(event.timestamp)
        .execute(&self.db_pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn get_events_by_topic(
        &self,
        topic_pattern: &str,
        limit: i64,
    ) -> AppResult<Vec<ArbEvent>> {
        let events: Vec<EventRow> = if topic_pattern.ends_with(".*") {
            let prefix = format!("{}%", &topic_pattern[..topic_pattern.len() - 2]);
            sqlx::query_as(
                r#"
                SELECT id, event_type, source_type, source_id, topic, payload, correlation_id, created_at
                FROM arb_events
                WHERE topic LIKE $1
                ORDER BY created_at DESC
                LIMIT $2
                "#,
            )
            .bind(&prefix)
            .bind(limit)
            .fetch_all(&self.db_pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
        } else {
            sqlx::query_as(
                r#"
                SELECT id, event_type, source_type, source_id, topic, payload, correlation_id, created_at
                FROM arb_events
                WHERE topic = $1
                ORDER BY created_at DESC
                LIMIT $2
                "#,
            )
            .bind(topic_pattern)
            .bind(limit)
            .fetch_all(&self.db_pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
        };

        Ok(events.into_iter().map(|r| r.into()).collect())
    }

    pub async fn get_events_since(
        &self,
        event_id: Uuid,
        topics: &[String],
        limit: i64,
    ) -> AppResult<Vec<ArbEvent>> {
        let events: Vec<EventRow> = sqlx::query_as(
            r#"
            SELECT id, event_type, source_type, source_id, topic, payload, correlation_id, created_at
            FROM arb_events
            WHERE created_at > (SELECT created_at FROM arb_events WHERE id = $1)
            ORDER BY created_at ASC
            LIMIT $2
            "#,
        )
        .bind(event_id)
        .bind(limit)
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let filtered: Vec<ArbEvent> = events
            .into_iter()
            .map(|r| r.into())
            .filter(|e: &ArbEvent| {
                if topics.is_empty() {
                    return true;
                }
                topics.iter().any(|t| matches_pattern(&e.topic, t))
            })
            .collect();

        Ok(filtered)
    }
}

pub struct EventSubscription {
    rx: broadcast::Receiver<ArbEvent>,
    filters: Vec<String>,
}

impl EventSubscription {
    pub async fn recv(&mut self) -> Option<ArbEvent> {
        loop {
            match self.rx.recv().await {
                Ok(event) => {
                    if self.filters.is_empty() {
                        return Some(event);
                    }
                    if self
                        .filters
                        .iter()
                        .any(|f| matches_pattern(&event.topic, f))
                    {
                        return Some(event);
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!("Event subscription lagged by {} events", n);
                    continue;
                }
                Err(broadcast::error::RecvError::Closed) => {
                    return None;
                }
            }
        }
    }

    pub fn with_filter(mut self, topic: impl Into<String>) -> Self {
        self.filters.push(topic.into());
        self
    }
}

#[derive(sqlx::FromRow)]
struct EventRow {
    id: Uuid,
    event_type: String,
    source_type: String,
    source_id: String,
    topic: String,
    payload: serde_json::Value,
    correlation_id: Option<Uuid>,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<EventRow> for ArbEvent {
    fn from(row: EventRow) -> Self {
        let source = match row.source_type.as_str() {
            "agent" => {
                let agent_type = match row.source_id.as_str() {
                    "scanner" => super::AgentType::Scanner,
                    "refiner" => super::AgentType::Refiner,
                    "mev_hunter" => super::AgentType::MevHunter,
                    "executor" => super::AgentType::Executor,
                    "strategy_engine" => super::AgentType::StrategyEngine,
                    "research_dd" => super::AgentType::ResearchDd,
                    "copy_trade" => super::AgentType::CopyTrade,
                    "threat_detector" => super::AgentType::ThreatDetector,
                    "engram_harvester" => super::AgentType::EngramHarvester,
                    "overseer" => super::AgentType::Overseer,
                    _ => super::AgentType::Scanner,
                };
                EventSource::Agent(agent_type)
            }
            "tool" => EventSource::Tool(row.source_id),
            "external" => EventSource::External(row.source_id),
            _ => EventSource::System,
        };

        ArbEvent {
            id: row.id,
            event_type: row.event_type,
            source,
            topic: row.topic,
            payload: row.payload,
            timestamp: row.created_at,
            correlation_id: row.correlation_id,
        }
    }
}
