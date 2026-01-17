use std::sync::Arc;
use std::collections::HashSet;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::events::{ArbEvent, EventBus, EventSource};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountUpdate {
    pub pubkey: String,
    pub slot: u64,
    pub lamports: u64,
    pub owner: String,
    pub executable: bool,
    pub rent_epoch: u64,
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionUpdate {
    pub signature: String,
    pub slot: u64,
    pub is_vote: bool,
    pub err: Option<String>,
    pub accounts: Vec<String>,
    pub log_messages: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LaserStreamEvent {
    AccountUpdate(AccountUpdate),
    TransactionUpdate(TransactionUpdate),
    SlotUpdate { slot: u64 },
    Ping,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaserStreamStatus {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
}

pub struct LaserStreamClient {
    endpoint: String,
    api_key: Option<String>,
    event_bus: Arc<EventBus>,
    status: Arc<RwLock<LaserStreamStatus>>,
    subscribed_accounts: Arc<RwLock<HashSet<String>>>,
}

impl LaserStreamClient {
    pub fn new(endpoint: String, api_key: Option<String>, event_bus: Arc<EventBus>) -> Self {
        Self {
            endpoint,
            api_key,
            event_bus,
            status: Arc::new(RwLock::new(LaserStreamStatus::Disconnected)),
            subscribed_accounts: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    pub fn is_configured(&self) -> bool {
        !self.endpoint.is_empty() && self.api_key.is_some()
    }

    pub async fn get_status(&self) -> LaserStreamStatus {
        *self.status.read().await
    }

    pub async fn connect(&self) -> Result<(), String> {
        if !self.is_configured() {
            return Err("LaserStream not configured - missing endpoint or API key".to_string());
        }

        {
            let mut status = self.status.write().await;
            *status = LaserStreamStatus::Connecting;
        }

        info!("🔌 Connecting to LaserStream: {}", self.endpoint);

        // NOTE: Full WebSocket implementation requires additional dependencies
        // For now, we'll log and set status to indicate this is a stub
        warn!("⚠️ LaserStream WebSocket connection not yet implemented");
        warn!("⚠️ Full implementation requires: tokio-tungstenite or similar WS client");

        // In production, this would:
        // 1. Establish WebSocket connection to endpoint
        // 2. Send authentication with API key
        // 3. Start receiving stream of events
        // 4. Emit events to event_bus

        {
            let mut status = self.status.write().await;
            *status = LaserStreamStatus::Disconnected;
        }

        Ok(())
    }

    pub async fn disconnect(&self) {
        info!("🔌 Disconnecting from LaserStream");
        let mut status = self.status.write().await;
        *status = LaserStreamStatus::Disconnected;
    }

    pub async fn subscribe_accounts(&self, addresses: Vec<String>) -> Result<(), String> {
        if !self.is_configured() {
            return Err("LaserStream not configured".to_string());
        }

        info!("📡 Subscribing to {} accounts via LaserStream", addresses.len());

        let mut subscribed = self.subscribed_accounts.write().await;
        for addr in addresses {
            subscribed.insert(addr);
        }

        // In production, this would send subscription message to WebSocket
        debug!("Total subscribed accounts: {}", subscribed.len());

        Ok(())
    }

    pub async fn unsubscribe_accounts(&self, addresses: Vec<String>) -> Result<(), String> {
        let count = addresses.len();
        let mut subscribed = self.subscribed_accounts.write().await;
        for addr in addresses {
            subscribed.remove(&addr);
        }

        debug!("Removed {} accounts from subscription", count);

        Ok(())
    }

    pub async fn get_subscribed_accounts(&self) -> Vec<String> {
        let subscribed = self.subscribed_accounts.read().await;
        subscribed.iter().cloned().collect()
    }

    #[allow(dead_code)]
    async fn emit_account_update(&self, update: AccountUpdate) {
        let event_data = serde_json::to_value(&update).unwrap_or_default();
        let event = ArbEvent::new(
            "account_update",
            EventSource::External("laserstream".to_string()),
            "arb.helius.laserstream.account",
            event_data,
        );
        if let Err(e) = self.event_bus.publish(event).await {
            warn!("Failed to publish account update: {}", e);
        }
    }

    #[allow(dead_code)]
    async fn emit_transaction_update(&self, update: TransactionUpdate) {
        let event_data = serde_json::to_value(&update).unwrap_or_default();
        let event = ArbEvent::new(
            "transaction_update",
            EventSource::External("laserstream".to_string()),
            "arb.helius.laserstream.transaction",
            event_data,
        );
        if let Err(e) = self.event_bus.publish(event).await {
            warn!("Failed to publish transaction update: {}", e);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaserStreamInfo {
    pub status: String,
    pub endpoint: String,
    pub subscribed_accounts: usize,
    pub is_configured: bool,
}

impl LaserStreamClient {
    pub async fn get_info(&self) -> LaserStreamInfo {
        let status = self.get_status().await;
        let subscribed = self.subscribed_accounts.read().await;

        LaserStreamInfo {
            status: format!("{:?}", status),
            endpoint: if self.is_configured() {
                self.endpoint.clone()
            } else {
                "Not configured".to_string()
            },
            subscribed_accounts: subscribed.len(),
            is_configured: self.is_configured(),
        }
    }
}
