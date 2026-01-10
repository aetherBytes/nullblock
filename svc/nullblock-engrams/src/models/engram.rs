use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EngramType {
    Persona,
    Preference,
    Strategy,
    Knowledge,
    Compliance,
}

impl EngramType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EngramType::Persona => "persona",
            EngramType::Preference => "preference",
            EngramType::Strategy => "strategy",
            EngramType::Knowledge => "knowledge",
            EngramType::Compliance => "compliance",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "persona" => Some(EngramType::Persona),
            "preference" => Some(EngramType::Preference),
            "strategy" => Some(EngramType::Strategy),
            "knowledge" => Some(EngramType::Knowledge),
            "compliance" => Some(EngramType::Compliance),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Engram {
    pub id: Uuid,
    pub wallet_address: String,
    pub engram_type: String,
    pub key: String,
    pub tags: Vec<String>,
    pub content: serde_json::Value,
    pub summary: Option<String>,
    pub version: i32,
    pub parent_id: Option<Uuid>,
    pub lineage_root_id: Option<Uuid>,
    pub is_public: bool,
    pub is_mintable: bool,
    pub nft_token_id: Option<String>,
    pub price_mon: Option<rust_decimal::Decimal>,
    pub royalty_percent: Option<i32>,
    pub priority: i32,
    pub ttl_seconds: Option<i32>,
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub accessed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEngramRequest {
    pub wallet_address: String,
    pub engram_type: String,
    pub key: String,
    pub content: serde_json::Value,
    #[serde(default)]
    pub tags: Vec<String>,
    pub summary: Option<String>,
    #[serde(default)]
    pub priority: i32,
    pub ttl_seconds: Option<i32>,
    pub created_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEngramRequest {
    pub content: serde_json::Value,
    pub summary: Option<String>,
    pub reason: Option<String>,
    pub changed_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchEngramsRequest {
    pub wallet_address: Option<String>,
    pub engram_type: Option<String>,
    pub tags: Option<Vec<String>>,
    pub is_public: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForkEngramRequest {
    pub target_wallet: String,
    pub new_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EngramHistory {
    pub id: Uuid,
    pub engram_id: Uuid,
    pub version: i32,
    pub content: serde_json::Value,
    pub changed_by: Option<String>,
    pub change_reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngramResponse {
    pub success: bool,
    pub data: Option<Engram>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngramsListResponse {
    pub success: bool,
    pub data: Vec<Engram>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
