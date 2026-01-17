use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaserStreamAccountEvent {
    pub address: String,
    pub slot: u64,
    pub lamports: u64,
    pub data_hash: String,
    pub owner: String,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaserStreamTransactionEvent {
    pub signature: String,
    pub slot: u64,
    pub accounts: Vec<String>,
    pub program_ids: Vec<String>,
    pub is_swap: bool,
    pub token_transfers: Vec<TokenTransfer>,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenTransfer {
    pub mint: String,
    pub from_address: String,
    pub to_address: String,
    pub amount: u64,
    pub decimals: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityFeeEvent {
    pub levels: HashMap<String, u64>,
    pub recommended: u64,
    pub percentile_50: u64,
    pub percentile_75: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SenderTxEvent {
    pub signature: String,
    pub status: TxStatus,
    pub landing_slot: Option<u64>,
    pub latency_ms: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TxStatus {
    Sent,
    Confirmed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeliusStatus {
    pub connected: bool,
    pub api_key_configured: bool,
    pub laserstream_enabled: bool,
    pub sender_enabled: bool,
    pub rpc_url: String,
    pub sender_url: String,
    pub laserstream_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaserStreamStatus {
    pub connected: bool,
    pub subscriptions: Vec<LaserStreamSubscription>,
    pub avg_latency_ms: f64,
    pub events_per_second: f64,
    pub total_events_received: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaserStreamSubscription {
    pub id: String,
    pub subscription_type: String,
    pub address: Option<String>,
    pub events_received: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SenderStats {
    pub total_sent: u64,
    pub total_confirmed: u64,
    pub total_failed: u64,
    pub success_rate: f64,
    pub avg_landing_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeliusConfig {
    pub laserstream_enabled: bool,
    pub default_priority_level: String,
    pub use_helius_sender: bool,
    pub priority_fee_poll_interval_secs: u64,
}

impl Default for HeliusConfig {
    fn default() -> Self {
        Self {
            laserstream_enabled: true,
            default_priority_level: "medium".to_string(),
            use_helius_sender: true,
            priority_fee_poll_interval_secs: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMetadata {
    pub mint: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub supply: u64,
    pub creators: Vec<Creator>,
    pub collection: Option<Collection>,
    pub attributes: HashMap<String, String>,
    pub image_uri: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Creator {
    pub address: String,
    pub verified: bool,
    pub share: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub address: String,
    pub name: Option<String>,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedTransaction {
    pub signature: String,
    pub slot: u64,
    pub timestamp: DateTime<Utc>,
    pub fee: u64,
    pub fee_payer: String,
    pub native_transfers: Vec<NativeTransfer>,
    pub token_transfers: Vec<EnhancedTokenTransfer>,
    pub instructions: Vec<ParsedInstruction>,
    #[serde(rename = "type")]
    pub transaction_type: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeTransfer {
    pub from_user_account: String,
    pub to_user_account: String,
    pub amount: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedTokenTransfer {
    pub from_user_account: Option<String>,
    pub to_user_account: Option<String>,
    pub from_token_account: Option<String>,
    pub to_token_account: Option<String>,
    pub mint: String,
    pub token_amount: f64,
    pub token_standard: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedInstruction {
    pub program_id: String,
    #[serde(rename = "type")]
    pub instruction_type: Option<String>,
    pub data: Option<serde_json::Value>,
}
