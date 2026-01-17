use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::error::{AppError, AppResult};
use crate::events::{topics, ArbEvent, EventBus, EventSource};
use super::client::HeliusClient;
use super::types::PriorityFeeEvent;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PriorityLevel {
    Min,
    Low,
    Medium,
    High,
    VeryHigh,
    UnsafeMax,
}

impl PriorityLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            PriorityLevel::Min => "min",
            PriorityLevel::Low => "low",
            PriorityLevel::Medium => "medium",
            PriorityLevel::High => "high",
            PriorityLevel::VeryHigh => "veryHigh",
            PriorityLevel::UnsafeMax => "unsafeMax",
        }
    }

    pub fn all() -> Vec<PriorityLevel> {
        vec![
            PriorityLevel::Min,
            PriorityLevel::Low,
            PriorityLevel::Medium,
            PriorityLevel::High,
            PriorityLevel::VeryHigh,
            PriorityLevel::UnsafeMax,
        ]
    }
}

impl std::fmt::Display for PriorityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityFeeEstimate {
    pub priority_level: PriorityLevel,
    pub fee_estimate: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityFeeResponse {
    pub min: u64,
    pub low: u64,
    pub medium: u64,
    pub high: u64,
    pub very_high: u64,
    pub unsafe_max: u64,
    pub recommended: u64,
}

impl PriorityFeeResponse {
    pub fn get(&self, level: PriorityLevel) -> u64 {
        match level {
            PriorityLevel::Min => self.min,
            PriorityLevel::Low => self.low,
            PriorityLevel::Medium => self.medium,
            PriorityLevel::High => self.high,
            PriorityLevel::VeryHigh => self.very_high,
            PriorityLevel::UnsafeMax => self.unsafe_max,
        }
    }

    pub fn to_hashmap(&self) -> HashMap<String, u64> {
        let mut map = HashMap::new();
        map.insert("min".to_string(), self.min);
        map.insert("low".to_string(), self.low);
        map.insert("medium".to_string(), self.medium);
        map.insert("high".to_string(), self.high);
        map.insert("very_high".to_string(), self.very_high);
        map.insert("unsafe_max".to_string(), self.unsafe_max);
        map
    }
}

pub struct PriorityFeeMonitor {
    client: Arc<HeliusClient>,
    event_bus: Arc<EventBus>,
    cached_fees: Arc<RwLock<Option<PriorityFeeResponse>>>,
    poll_interval: Duration,
}

impl PriorityFeeMonitor {
    pub fn new(client: Arc<HeliusClient>, event_bus: Arc<EventBus>) -> Self {
        Self {
            client,
            event_bus,
            cached_fees: Arc::new(RwLock::new(None)),
            poll_interval: Duration::from_secs(10),
        }
    }

    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    pub async fn get_priority_fee_estimate(
        &self,
        transaction_base64: Option<&str>,
        account_keys: Option<&[String]>,
    ) -> AppResult<PriorityFeeResponse> {
        let params = if let Some(tx) = transaction_base64 {
            json!([{
                "transaction": tx,
                "options": {
                    "recommended": true,
                    "includeAllPriorityFeeLevels": true
                }
            }])
        } else if let Some(keys) = account_keys {
            json!([{
                "accountKeys": keys,
                "options": {
                    "recommended": true,
                    "includeAllPriorityFeeLevels": true
                }
            }])
        } else {
            json!([{
                "options": {
                    "recommended": true,
                    "includeAllPriorityFeeLevels": true
                }
            }])
        };

        #[derive(Debug, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct HeliusPriorityFeeResponse {
            priority_fee_levels: Option<PriorityFeeLevels>,
            priority_fee_estimate: Option<u64>,
        }

        #[derive(Debug, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct PriorityFeeLevels {
            min: u64,
            low: u64,
            medium: u64,
            high: u64,
            very_high: u64,
            unsafe_max: u64,
        }

        let response: HeliusPriorityFeeResponse = self
            .client
            .rpc_call("getPriorityFeeEstimate", params)
            .await?;

        let levels = response.priority_fee_levels.ok_or_else(|| {
            AppError::ExternalApi("No priority fee levels in response".to_string())
        })?;

        let fee_response = PriorityFeeResponse {
            min: levels.min,
            low: levels.low,
            medium: levels.medium,
            high: levels.high,
            very_high: levels.very_high,
            unsafe_max: levels.unsafe_max,
            recommended: response.priority_fee_estimate.unwrap_or(levels.medium),
        };

        let mut cached = self.cached_fees.write().await;
        *cached = Some(fee_response.clone());

        Ok(fee_response)
    }

    pub async fn get_cached_fees(&self) -> Option<PriorityFeeResponse> {
        self.cached_fees.read().await.clone()
    }

    pub async fn start_polling(&self) {
        info!(
            "Starting priority fee polling with interval: {:?}",
            self.poll_interval
        );

        let mut interval = tokio::time::interval(self.poll_interval);

        loop {
            interval.tick().await;

            match self.get_priority_fee_estimate(None, None).await {
                Ok(fees) => {
                    debug!("Priority fees updated: recommended={}", fees.recommended);

                    let event_payload = PriorityFeeEvent {
                        levels: fees.to_hashmap(),
                        recommended: fees.recommended,
                        percentile_50: fees.medium,
                        percentile_75: fees.high,
                        timestamp: Utc::now(),
                    };

                    let event = ArbEvent::new(
                        "priority_fee_updated",
                        EventSource::External("helius_priority_fee".to_string()),
                        topics::helius::priority_fee::UPDATED,
                        serde_json::to_value(&event_payload).unwrap_or_default(),
                    );

                    if let Err(e) = self.event_bus.publish(event).await {
                        warn!("Failed to publish priority fee event: {}", e);
                    }
                }
                Err(e) => {
                    error!("Failed to fetch priority fees: {}", e);
                }
            }
        }
    }
}

pub fn select_priority_level_for_profit(
    estimated_profit_lamports: i64,
    fees: &PriorityFeeResponse,
) -> (PriorityLevel, u64) {
    let profit_sol = estimated_profit_lamports as f64 / 1_000_000_000.0;

    if profit_sol >= 1.0 {
        (PriorityLevel::VeryHigh, fees.very_high)
    } else if profit_sol >= 0.5 {
        (PriorityLevel::High, fees.high)
    } else if profit_sol >= 0.1 {
        (PriorityLevel::Medium, fees.medium)
    } else if profit_sol >= 0.01 {
        (PriorityLevel::Low, fees.low)
    } else {
        (PriorityLevel::Min, fees.min)
    }
}
