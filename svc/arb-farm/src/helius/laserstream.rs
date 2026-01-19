use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

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

// Helius WebSocket JSON-RPC message types
#[derive(Debug, Serialize)]
struct HeliusSubscribeRequest {
    jsonrpc: &'static str,
    id: u64,
    method: &'static str,
    params: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct HeliusNotification {
    #[serde(default)]
    method: Option<String>,
    #[serde(default)]
    params: Option<HeliusNotificationParams>,
}

#[derive(Debug, Deserialize)]
struct HeliusNotificationParams {
    result: HeliusAccountResult,
    subscription: u64,
}

#[derive(Debug, Deserialize)]
struct HeliusAccountResult {
    context: HeliusContext,
    value: HeliusAccountValue,
}

#[derive(Debug, Deserialize)]
struct HeliusContext {
    slot: u64,
}

#[derive(Debug, Deserialize)]
struct HeliusAccountValue {
    lamports: u64,
    data: Vec<String>,
    owner: String,
    executable: bool,
    #[serde(rename = "rentEpoch")]
    rent_epoch: u64,
}

pub struct LaserStreamClient {
    endpoint: String,
    api_key: Option<String>,
    event_bus: Arc<EventBus>,
    status: Arc<RwLock<LaserStreamStatus>>,
    subscribed_accounts: Arc<RwLock<HashSet<String>>>,
    subscription_ids: Arc<RwLock<HashMap<String, u64>>>,
    command_tx: Arc<RwLock<Option<mpsc::Sender<WebSocketCommand>>>>,
    account_update_tx: broadcast::Sender<AccountUpdate>,
}

#[derive(Debug)]
enum WebSocketCommand {
    Subscribe(Vec<String>),
    Unsubscribe(Vec<String>),
    Disconnect,
}

impl LaserStreamClient {
    pub fn new(endpoint: String, api_key: Option<String>, event_bus: Arc<EventBus>) -> Self {
        let (account_update_tx, _) = broadcast::channel(1000);
        Self {
            endpoint,
            api_key,
            event_bus,
            status: Arc::new(RwLock::new(LaserStreamStatus::Disconnected)),
            subscribed_accounts: Arc::new(RwLock::new(HashSet::new())),
            subscription_ids: Arc::new(RwLock::new(HashMap::new())),
            command_tx: Arc::new(RwLock::new(None)),
            account_update_tx,
        }
    }

    pub fn is_configured(&self) -> bool {
        !self.endpoint.is_empty() && self.api_key.is_some()
    }

    pub async fn get_status(&self) -> LaserStreamStatus {
        *self.status.read().await
    }

    pub fn subscribe_account_updates(&self) -> broadcast::Receiver<AccountUpdate> {
        self.account_update_tx.subscribe()
    }

    pub async fn connect(&self) -> Result<(), String> {
        if !self.is_configured() {
            return Err("LaserStream not configured - missing endpoint or API key".to_string());
        }

        {
            let mut status = self.status.write().await;
            if *status == LaserStreamStatus::Connected || *status == LaserStreamStatus::Connecting {
                return Ok(());
            }
            *status = LaserStreamStatus::Connecting;
        }

        let ws_url = format!("{}/?api-key={}", self.endpoint.trim_end_matches('/'), self.api_key.as_ref().unwrap());
        info!("🔌 Connecting to Helius WebSocket: {}...", &ws_url[..ws_url.len().min(50)]);

        let (ws_stream, _) = connect_async(&ws_url)
            .await
            .map_err(|e| format!("WebSocket connection failed: {}", e))?;

        info!("✅ Connected to Helius WebSocket");

        {
            let mut status = self.status.write().await;
            *status = LaserStreamStatus::Connected;
        }

        let (mut write, mut read) = ws_stream.split();
        let (cmd_tx, mut cmd_rx) = mpsc::channel::<WebSocketCommand>(100);

        {
            let mut command_tx = self.command_tx.write().await;
            *command_tx = Some(cmd_tx);
        }

        let status = self.status.clone();
        let subscribed_accounts = self.subscribed_accounts.clone();
        let subscription_ids = self.subscription_ids.clone();
        let account_update_tx = self.account_update_tx.clone();
        let event_bus = self.event_bus.clone();

        // Spawn message handler
        tokio::spawn(async move {
            let mut request_id: u64 = 1;
            let mut pending_subscriptions: HashMap<u64, String> = HashMap::new();

            loop {
                tokio::select! {
                    // Handle incoming messages
                    msg = read.next() => {
                        match msg {
                            Some(Ok(Message::Text(text))) => {
                                debug!("📨 WS message received: {}", &text[..text.len().min(200)]);

                                // Parse as generic JSON first to route correctly
                                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                                    // Check if this is a subscription confirmation (has "id" and "result" as integer)
                                    if let (Some(id), Some(result)) = (
                                        json.get("id").and_then(|v| v.as_u64()),
                                        json.get("result").and_then(|v| v.as_u64()),
                                    ) {
                                        debug!("📥 Subscription response: id={}, result={}, pending={:?}", id, result, pending_subscriptions.keys().collect::<Vec<_>>());
                                        if let Some(pubkey) = pending_subscriptions.remove(&id) {
                                            let mut sub_ids = subscription_ids.write().await;
                                            sub_ids.insert(pubkey.clone(), result);
                                            info!(pubkey = %pubkey, subscription_id = result, "✅ Subscribed to account");
                                        } else {
                                            debug!("⚠️ No pending subscription for id={}", id);
                                        }
                                    }
                                    // Check if this is an account notification
                                    else if json.get("method").and_then(|v| v.as_str()) == Some("accountNotification") {
                                        if let Ok(notification) = serde_json::from_str::<HeliusNotification>(&text) {
                                            if let Some(params) = notification.params {
                                                // Find which account this is for
                                                let sub_ids = subscription_ids.read().await;
                                                debug!("📡 accountNotification for subscription={}, known subs={:?}", params.subscription, sub_ids.values().collect::<Vec<_>>());
                                                if let Some((pubkey, _)) = sub_ids.iter().find(|(_, &id)| id == params.subscription) {
                                                    let update = AccountUpdate {
                                                        pubkey: pubkey.clone(),
                                                        slot: params.result.context.slot,
                                                        lamports: params.result.value.lamports,
                                                        owner: params.result.value.owner,
                                                        executable: params.result.value.executable,
                                                        rent_epoch: params.result.value.rent_epoch,
                                                        data: params.result.value.data.first().cloned().unwrap_or_default(),
                                                    };

                                                    debug!(
                                                        pubkey = %update.pubkey,
                                                        slot = update.slot,
                                                        "📡 Account update received"
                                                    );

                                                    // Broadcast to subscribers
                                                    let _ = account_update_tx.send(update.clone());

                                                    // Also emit to event bus
                                                    let event_data = serde_json::to_value(&update).unwrap_or_default();
                                                    let event = ArbEvent::new(
                                                        "account_update",
                                                        EventSource::External("helius_ws".to_string()),
                                                        "arb.helius.account.update",
                                                        event_data,
                                                    );
                                                    let _ = event_bus.publish(event).await;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            Some(Ok(Message::Ping(data))) => {
                                if let Err(e) = write.send(Message::Pong(data)).await {
                                    error!("Failed to send pong: {}", e);
                                }
                            }
                            Some(Ok(Message::Close(_))) => {
                                warn!("WebSocket closed by server");
                                break;
                            }
                            Some(Err(e)) => {
                                error!("WebSocket error: {}", e);
                                break;
                            }
                            None => {
                                warn!("WebSocket stream ended");
                                break;
                            }
                            _ => {}
                        }
                    }

                    // Handle commands
                    Some(cmd) = cmd_rx.recv() => {
                        match cmd {
                            WebSocketCommand::Subscribe(accounts) => {
                                for pubkey in accounts {
                                    let request = HeliusSubscribeRequest {
                                        jsonrpc: "2.0",
                                        id: request_id,
                                        method: "accountSubscribe",
                                        params: vec![
                                            serde_json::Value::String(pubkey.clone()),
                                            serde_json::json!({
                                                "encoding": "base64",
                                                "commitment": "confirmed"
                                            }),
                                        ],
                                    };

                                    if let Ok(msg) = serde_json::to_string(&request) {
                                        if let Err(e) = write.send(Message::Text(msg)).await {
                                            error!("Failed to send subscribe: {}", e);
                                        } else {
                                            pending_subscriptions.insert(request_id, pubkey.clone());
                                            let mut subs = subscribed_accounts.write().await;
                                            subs.insert(pubkey);
                                        }
                                    }
                                    request_id += 1;
                                }
                            }
                            WebSocketCommand::Unsubscribe(accounts) => {
                                let sub_ids = subscription_ids.read().await;
                                for pubkey in &accounts {
                                    if let Some(&sub_id) = sub_ids.get(pubkey) {
                                        let request = serde_json::json!({
                                            "jsonrpc": "2.0",
                                            "id": request_id,
                                            "method": "accountUnsubscribe",
                                            "params": [sub_id]
                                        });

                                        if let Ok(msg) = serde_json::to_string(&request) {
                                            let _ = write.send(Message::Text(msg)).await;
                                        }
                                        request_id += 1;
                                    }
                                }
                                drop(sub_ids);

                                let mut sub_ids = subscription_ids.write().await;
                                let mut subs = subscribed_accounts.write().await;
                                for pubkey in accounts {
                                    sub_ids.remove(&pubkey);
                                    subs.remove(&pubkey);
                                }
                            }
                            WebSocketCommand::Disconnect => {
                                info!("🔌 Disconnecting WebSocket...");
                                let _ = write.close().await;
                                break;
                            }
                        }
                    }
                }
            }

            // Update status on disconnect
            {
                let mut s = status.write().await;
                *s = LaserStreamStatus::Disconnected;
            }
            info!("🔌 WebSocket disconnected");

            // Auto-reconnect after delay
            let accounts_to_resubscribe: Vec<String> = subscribed_accounts.read().await.iter().cloned().collect();
            if !accounts_to_resubscribe.is_empty() {
                warn!("🔄 Will attempt to reconnect in 5 seconds (had {} subscriptions)", accounts_to_resubscribe.len());
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

                // Signal that we need reconnection - the main loop should handle this
                {
                    let mut s = status.write().await;
                    *s = LaserStreamStatus::Reconnecting;
                }
            }
        });

        Ok(())
    }

    pub async fn disconnect(&self) {
        if let Some(tx) = self.command_tx.read().await.as_ref() {
            let _ = tx.send(WebSocketCommand::Disconnect).await;
        }
    }

    /// Start a background task that monitors connection status and auto-reconnects
    pub fn start_reconnect_monitor(self: &Arc<Self>) {
        let client = Arc::clone(self);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

                let status = client.get_status().await;
                if status == LaserStreamStatus::Reconnecting {
                    info!("🔄 Attempting WebSocket reconnection...");

                    // Get accounts to resubscribe
                    let accounts: Vec<String> = client.subscribed_accounts.read().await.iter().cloned().collect();

                    // Clear old subscription IDs
                    {
                        let mut sub_ids = client.subscription_ids.write().await;
                        sub_ids.clear();
                    }

                    // Try to reconnect
                    match client.connect().await {
                        Ok(_) => {
                            info!("✅ WebSocket reconnected successfully");
                            // Resubscribe to accounts
                            if !accounts.is_empty() {
                                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                                if let Err(e) = client.subscribe_accounts(accounts.clone()).await {
                                    warn!("⚠️ Failed to resubscribe to accounts: {}", e);
                                } else {
                                    info!("✅ Resubscribed to {} accounts", accounts.len());
                                }
                            }
                        }
                        Err(e) => {
                            error!("❌ Reconnection failed: {}", e);
                            // Will retry on next loop iteration
                            let mut s = client.status.write().await;
                            *s = LaserStreamStatus::Reconnecting;
                        }
                    }
                }
            }
        });
    }

    pub async fn subscribe_accounts(&self, addresses: Vec<String>) -> Result<(), String> {
        if addresses.is_empty() {
            return Ok(());
        }

        let status = self.get_status().await;
        if status != LaserStreamStatus::Connected {
            return Err("WebSocket not connected".to_string());
        }

        info!("📡 Subscribing to {} accounts via WebSocket", addresses.len());

        if let Some(tx) = self.command_tx.read().await.as_ref() {
            tx.send(WebSocketCommand::Subscribe(addresses))
                .await
                .map_err(|e| format!("Failed to send subscribe command: {}", e))?;
        }

        Ok(())
    }

    pub async fn unsubscribe_accounts(&self, addresses: Vec<String>) -> Result<(), String> {
        if addresses.is_empty() {
            return Ok(());
        }

        if let Some(tx) = self.command_tx.read().await.as_ref() {
            tx.send(WebSocketCommand::Unsubscribe(addresses))
                .await
                .map_err(|e| format!("Failed to send unsubscribe command: {}", e))?;
        }

        Ok(())
    }

    pub async fn get_subscribed_accounts(&self) -> Vec<String> {
        self.subscribed_accounts.read().await.iter().cloned().collect()
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
