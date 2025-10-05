use axum::{
    response::{Sse, sse::Event},
    extract::{State, Path},
    http::StatusCode,
};
use futures::stream::{Stream, StreamExt};
use rdkafka::{
    consumer::{Consumer, StreamConsumer},
    config::ClientConfig,
    message::Message,
};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use tokio_stream::wrappers::BroadcastStream;
use tokio::sync::broadcast;
use tracing::{info, warn, error};

use crate::server::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskLifecycleEvent {
    pub task_id: String,
    pub event_type: String,
    pub state: String,
    pub message: Option<String>,
    pub timestamp: String,
}

pub struct KafkaSSEBridge {
    tx: broadcast::Sender<TaskLifecycleEvent>,
    consumer: Arc<StreamConsumer>,
}

impl KafkaSSEBridge {
    pub fn new(bootstrap_servers: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", bootstrap_servers)
            .set("group.id", "nullblock-protocols-sse")
            .set("enable.auto.commit", "true")
            .set("auto.offset.reset", "latest")
            .create()?;

        consumer.subscribe(&["task.lifecycle"])?;

        let (tx, _rx) = broadcast::channel(100);

        Ok(Self {
            tx,
            consumer: Arc::new(consumer),
        })
    }

    pub async fn start_forwarding(&self) {
        let consumer = Arc::clone(&self.consumer);
        let tx = self.tx.clone();

        tokio::spawn(async move {
            info!("🔄 Starting Kafka → SSE bridge for task.lifecycle topic");

            loop {
                match consumer.recv().await {
                    Ok(message) => {
                        if let Some(payload) = message.payload() {
                            match serde_json::from_slice::<TaskLifecycleEvent>(payload) {
                                Ok(event) => {
                                    info!("📨 Forwarding task event: {} → {}", event.task_id, event.state);
                                    if let Err(e) = tx.send(event) {
                                        warn!("⚠️ No active SSE subscribers: {}", e);
                                    }
                                }
                                Err(e) => {
                                    error!("❌ Failed to parse task event: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("❌ Kafka consumer error: {}", e);
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                }
            }
        });
    }

    pub fn subscribe(&self) -> broadcast::Receiver<TaskLifecycleEvent> {
        self.tx.subscribe()
    }

    pub fn subscribe_to_task(&self, _task_id: String) -> broadcast::Receiver<TaskLifecycleEvent> {
        let rx = self.tx.subscribe();
        rx
    }
}

pub async fn task_subscribe_handler(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, StatusCode> {
    info!("🔌 SSE client subscribed to task: {}", task_id);

    let bridge = state.kafka_bridge.as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let rx = bridge.subscribe_to_task(task_id.clone());
    let stream = BroadcastStream::new(rx);

    let task_id_filter = task_id.clone();
    let filtered_stream = stream.filter_map(move |result| {
        let task_id = task_id_filter.clone();
        async move {
            match result {
                Ok(event) if event.task_id == task_id => {
                    match serde_json::to_string(&event) {
                        Ok(json) => Some(Ok(Event::default().data(json))),
                        Err(e) => {
                            error!("❌ Failed to serialize event: {}", e);
                            None
                        }
                    }
                }
                Ok(_) => None,
                Err(e) => {
                    warn!("⚠️ Broadcast stream error: {}", e);
                    None
                }
            }
        }
    });

    let keep_alive_stream = tokio_stream::StreamExt::timeout(
        filtered_stream,
        Duration::from_secs(30),
    )
    .map(|result| match result {
        Ok(event) => event,
        Err(_) => Ok(Event::default().comment("keep-alive")),
    });

    Ok(Sse::new(keep_alive_stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(30))
            .text("keep-alive"),
    ))
}

pub async fn message_stream_handler(
    State(state): State<AppState>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, StatusCode> {
    info!("🔌 SSE client subscribed to message stream");

    let bridge = state.kafka_bridge.as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let rx = bridge.subscribe();
    let stream = BroadcastStream::new(rx);

    let event_stream = stream.filter_map(|result| async move {
        match result {
            Ok(event) => {
                match serde_json::to_string(&event) {
                    Ok(json) => Some(Ok(Event::default().data(json))),
                    Err(e) => {
                        error!("❌ Failed to serialize event: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                warn!("⚠️ Broadcast stream error: {}", e);
                None
            }
        }
    });

    let keep_alive_stream = tokio_stream::StreamExt::timeout(
        event_stream,
        Duration::from_secs(30),
    )
    .map(|result| match result {
        Ok(event) => event,
        Err(_) => Ok(Event::default().comment("keep-alive")),
    });

    Ok(Sse::new(keep_alive_stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(30))
            .text("keep-alive"),
    ))
}
