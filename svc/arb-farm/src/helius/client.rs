use std::sync::Arc;
use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::events::{ArbEvent, EventBus, EventSource};
use super::types::{HeliusConfig, HeliusStatus};

pub struct HeliusClient {
    http_client: Client,
    api_key: Option<String>,
    rpc_url: String,
    sender_url: String,
    laserstream_url: String,
    config: Arc<RwLock<HeliusConfig>>,
    event_bus: Option<Arc<EventBus>>,
}

impl HeliusClient {
    pub fn new(config: &Config) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            http_client,
            api_key: config.helius_api_key.clone(),
            rpc_url: config.helius_api_url.clone(),
            sender_url: config.helius_sender_url.clone(),
            laserstream_url: config.helius_laserstream_url.clone(),
            config: Arc::new(RwLock::new(HeliusConfig::default())),
            event_bus: None,
        }
    }

    #[cfg(test)]
    pub fn new_mock() -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            http_client,
            api_key: None,
            rpc_url: "http://localhost:8899".to_string(),
            sender_url: "http://localhost:8899".to_string(),
            laserstream_url: "ws://localhost:8899".to_string(),
            config: Arc::new(RwLock::new(HeliusConfig::default())),
            event_bus: None,
        }
    }

    pub fn with_event_bus(mut self, event_bus: Arc<EventBus>) -> Self {
        self.event_bus = Some(event_bus);
        self
    }

    pub fn rpc_url_with_key(&self) -> String {
        match &self.api_key {
            Some(key) => format!("{}/?api-key={}", self.rpc_url, key),
            None => self.rpc_url.clone(),
        }
    }

    pub fn sender_url_with_key(&self) -> String {
        match &self.api_key {
            Some(key) => format!("{}/?api-key={}", self.sender_url, key),
            None => self.sender_url.clone(),
        }
    }

    pub fn api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }

    pub fn has_api_key(&self) -> bool {
        self.api_key.is_some()
    }

    pub fn is_configured(&self) -> bool {
        self.api_key.is_some()
    }

    pub async fn get_status(&self) -> HeliusStatus {
        let config = self.config.read().await;
        HeliusStatus {
            connected: self.has_api_key(),
            api_key_configured: self.has_api_key(),
            laserstream_enabled: config.laserstream_enabled,
            sender_enabled: config.use_helius_sender,
            rpc_url: self.rpc_url.clone(),
            sender_url: self.sender_url.clone(),
            laserstream_url: self.laserstream_url.clone(),
        }
    }

    pub async fn get_config(&self) -> HeliusConfig {
        self.config.read().await.clone()
    }

    pub async fn update_config(&self, new_config: HeliusConfig) {
        let mut config = self.config.write().await;
        *config = new_config;
    }

    pub async fn rpc_call<T: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> AppResult<T> {
        let url = self.rpc_url_with_key();

        let request_body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params
        });

        debug!("Helius RPC call: {} to {}", method, self.rpc_url);

        let response = self
            .http_client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Helius RPC request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Helius RPC error: status={}, body={}", status, body);
            return Err(AppError::ExternalApi(format!(
                "Helius RPC error: status={}, body={}",
                status, body
            )));
        }

        let rpc_response: RpcResponse<T> = response
            .json()
            .await
            .map_err(|e| AppError::Serialization(format!("Failed to parse Helius response: {}", e)))?;

        match rpc_response.result {
            Some(result) => Ok(result),
            None => {
                let error_msg = rpc_response
                    .error
                    .map(|e| format!("{}: {}", e.code, e.message))
                    .unwrap_or_else(|| "Unknown error".to_string());
                Err(AppError::ExternalApi(format!(
                    "Helius RPC error: {}",
                    error_msg
                )))
            }
        }
    }

    pub async fn emit_event(&self, topic: &str, event_type: &str, payload: serde_json::Value) {
        if let Some(bus) = &self.event_bus {
            let event = ArbEvent::new(
                event_type,
                EventSource::External("helius".to_string()),
                topic,
                payload,
            );
            if let Err(e) = bus.publish(event).await {
                warn!("Failed to emit Helius event: {}", e);
            }
        }
    }

    pub async fn test_connection(&self) -> AppResult<bool> {
        #[derive(Deserialize)]
        struct HealthResponse {
            result: Option<String>,
        }

        let result: String = self.rpc_call("getHealth", json!([])).await?;
        Ok(result == "ok")
    }

    pub async fn get_latest_blockhash(&self) -> AppResult<String> {
        #[derive(Deserialize)]
        struct BlockhashResponse {
            blockhash: String,
            #[serde(rename = "lastValidBlockHeight")]
            last_valid_block_height: u64,
        }

        #[derive(Deserialize)]
        struct ValueWrapper {
            value: BlockhashResponse,
        }

        let response: ValueWrapper = self
            .rpc_call("getLatestBlockhash", json!([{"commitment": "finalized"}]))
            .await?;

        Ok(response.value.blockhash)
    }

    pub async fn get_slot(&self) -> AppResult<u64> {
        let slot: u64 = self.rpc_call("getSlot", json!([])).await?;
        Ok(slot)
    }

    pub async fn get_token_largest_accounts(&self, mint: &str) -> AppResult<TokenLargestAccountsResponse> {
        let response: TokenLargestAccountsResponse = self
            .rpc_call("getTokenLargestAccounts", json!([mint]))
            .await?;
        Ok(response)
    }

    pub async fn get_signatures_for_address(
        &self,
        address: &str,
        limit: u8,
    ) -> AppResult<Vec<SignatureInfo>> {
        let result: Vec<SignatureInfo> = self
            .rpc_call(
                "getSignaturesForAddress",
                json!([
                    address,
                    {
                        "limit": limit,
                        "commitment": "confirmed"
                    }
                ]),
            )
            .await?;
        Ok(result)
    }

    pub async fn get_transaction(&self, signature: &str) -> AppResult<Option<TransactionResponse>> {
        let result: Option<TransactionResponse> = self
            .rpc_call(
                "getTransaction",
                json!([
                    signature,
                    {
                        "encoding": "json",
                        "maxSupportedTransactionVersion": 0,
                        "commitment": "confirmed"
                    }
                ]),
            )
            .await?;
        Ok(result)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenLargestAccountsResponse {
    pub context: RpcContext,
    pub value: Vec<TokenAccountBalance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcContext {
    pub slot: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAccountBalance {
    pub address: String,
    pub amount: String,
    pub decimals: u8,
    #[serde(rename = "uiAmount")]
    pub ui_amount: Option<f64>,
    #[serde(rename = "uiAmountString")]
    pub ui_amount_string: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureInfo {
    pub signature: String,
    pub slot: u64,
    #[serde(rename = "blockTime")]
    pub block_time: Option<i64>,
    pub err: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionMeta {
    pub fee: u64,
    #[serde(rename = "preBalances")]
    pub pre_balances: Vec<u64>,
    #[serde(rename = "postBalances")]
    pub post_balances: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionMessage {
    #[serde(rename = "accountKeys")]
    pub account_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInner {
    pub message: TransactionMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub meta: Option<TransactionMeta>,
    pub transaction: Option<TransactionInner>,
    pub slot: u64,
}

#[derive(Debug, Deserialize)]
struct RpcResponse<T> {
    jsonrpc: String,
    id: u64,
    result: Option<T>,
    error: Option<RpcError>,
}

#[derive(Debug, Deserialize)]
struct RpcError {
    code: i64,
    message: String,
}
