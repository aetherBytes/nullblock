use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::error::{AppError, AppResult};
use crate::events::{topics, ArbEvent, EventBus, EventSource};
use super::client::HeliusClient;
use super::types::{SenderStats, SenderTxEvent, TxStatus};

pub struct HeliusSender {
    client: Arc<HeliusClient>,
    event_bus: Arc<EventBus>,
    stats: Arc<RwLock<SenderStats>>,
}

impl HeliusSender {
    pub fn new(client: Arc<HeliusClient>, event_bus: Arc<EventBus>) -> Self {
        Self {
            client,
            event_bus,
            stats: Arc::new(RwLock::new(SenderStats {
                total_sent: 0,
                total_confirmed: 0,
                total_failed: 0,
                success_rate: 0.0,
                avg_landing_ms: 0.0,
            })),
        }
    }

    pub async fn send_transaction(
        &self,
        transaction_base64: &str,
        skip_preflight: bool,
    ) -> AppResult<String> {
        let start = Instant::now();

        let params = json!([
            transaction_base64,
            {
                "encoding": "base64",
                "skipPreflight": skip_preflight,
                "maxRetries": 0,
                "preflightCommitment": "confirmed"
            }
        ]);

        let signature: String = self.client.rpc_call("sendTransaction", params).await?;

        let latency_ms = start.elapsed().as_millis() as u64;

        {
            let mut stats = self.stats.write().await;
            stats.total_sent += 1;
        }

        let event = SenderTxEvent {
            signature: signature.clone(),
            status: TxStatus::Sent,
            landing_slot: None,
            latency_ms,
            error: None,
        };

        self.emit_event(topics::helius::sender::TX_SENT, "tx_sent", &event)
            .await;

        info!(
            "Transaction sent via Helius Sender: {} ({}ms)",
            signature, latency_ms
        );

        Ok(signature)
    }

    pub async fn send_and_confirm(
        &self,
        transaction_base64: &str,
        timeout: Duration,
    ) -> AppResult<String> {
        let start = Instant::now();
        let signature = self.send_transaction(transaction_base64, true).await?;

        let confirmation_result = self
            .wait_for_confirmation(&signature, timeout)
            .await;

        let latency_ms = start.elapsed().as_millis() as u64;

        match confirmation_result {
            Ok(slot) => {
                {
                    let mut stats = self.stats.write().await;
                    stats.total_confirmed += 1;
                    stats.success_rate =
                        stats.total_confirmed as f64 / stats.total_sent as f64 * 100.0;
                    let total = stats.total_confirmed;
                    stats.avg_landing_ms =
                        (stats.avg_landing_ms * (total - 1) as f64 + latency_ms as f64) / total as f64;
                }

                let event = SenderTxEvent {
                    signature: signature.clone(),
                    status: TxStatus::Confirmed,
                    landing_slot: Some(slot),
                    latency_ms,
                    error: None,
                };

                self.emit_event(topics::helius::sender::TX_CONFIRMED, "tx_confirmed", &event)
                    .await;

                info!(
                    "Transaction confirmed: {} at slot {} ({}ms)",
                    signature, slot, latency_ms
                );

                Ok(signature)
            }
            Err(e) => {
                {
                    let mut stats = self.stats.write().await;
                    stats.total_failed += 1;
                    stats.success_rate =
                        stats.total_confirmed as f64 / stats.total_sent as f64 * 100.0;
                }

                let event = SenderTxEvent {
                    signature: signature.clone(),
                    status: TxStatus::Failed,
                    landing_slot: None,
                    latency_ms,
                    error: Some(e.to_string()),
                };

                self.emit_event(topics::helius::sender::TX_FAILED, "tx_failed", &event)
                    .await;

                error!("Transaction failed: {} - {}", signature, e);

                Err(e)
            }
        }
    }

    async fn wait_for_confirmation(&self, signature: &str, timeout: Duration) -> AppResult<u64> {
        let start = Instant::now();
        let mut interval = tokio::time::interval(Duration::from_millis(500));

        while start.elapsed() < timeout {
            interval.tick().await;

            #[derive(Debug, Deserialize)]
            struct SignatureStatus {
                slot: u64,
                confirmations: Option<u64>,
                err: Option<serde_json::Value>,
                confirmation_status: Option<String>,
            }

            #[derive(Debug, Deserialize)]
            struct ValueWrapper {
                value: Vec<Option<SignatureStatus>>,
            }

            let response: ValueWrapper = self
                .client
                .rpc_call(
                    "getSignatureStatuses",
                    json!([[signature], {"searchTransactionHistory": false}]),
                )
                .await?;

            if let Some(Some(status)) = response.value.first() {
                if let Some(ref err) = status.err {
                    return Err(AppError::Execution(format!(
                        "Transaction error: {:?}",
                        err
                    )));
                }

                if let Some(ref conf_status) = status.confirmation_status {
                    if conf_status == "confirmed" || conf_status == "finalized" {
                        return Ok(status.slot);
                    }
                }
            }
        }

        Err(AppError::Timeout(format!(
            "Transaction confirmation timeout after {:?}",
            timeout
        )))
    }

    pub async fn ping(&self) -> AppResult<u64> {
        let start = Instant::now();
        let _slot: u64 = self.client.rpc_call("getSlot", json!([])).await?;
        Ok(start.elapsed().as_millis() as u64)
    }

    pub async fn get_stats(&self) -> SenderStats {
        self.stats.read().await.clone()
    }

    async fn emit_event(&self, topic: &str, event_type: &str, payload: &SenderTxEvent) {
        let event = ArbEvent::new(
            event_type,
            EventSource::External("helius_sender".to_string()),
            topic,
            serde_json::to_value(payload).unwrap_or_default(),
        );

        if let Err(e) = self.event_bus.publish(event).await {
            warn!("Failed to emit sender event: {}", e);
        }
    }
}
