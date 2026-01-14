use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::events::AtomicityLevel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: Uuid,
    pub strategy_id: Option<Uuid>,
    pub edge_type: String,
    pub execution_mode: String,
    pub atomicity: AtomicityLevel,
    pub simulated_profit_guaranteed: bool,
    pub estimated_profit_lamports: Option<i64>,
    pub risk_score: Option<i32>,
    pub route_data: serde_json::Value,
    pub status: EdgeStatus,
    pub token_mint: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum EdgeStatus {
    Detected,
    PendingApproval,
    Executing,
    Executed,
    Expired,
    Failed,
    Rejected,
}

impl std::fmt::Display for EdgeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EdgeStatus::Detected => write!(f, "detected"),
            EdgeStatus::PendingApproval => write!(f, "pending_approval"),
            EdgeStatus::Executing => write!(f, "executing"),
            EdgeStatus::Executed => write!(f, "executed"),
            EdgeStatus::Expired => write!(f, "expired"),
            EdgeStatus::Failed => write!(f, "failed"),
            EdgeStatus::Rejected => write!(f, "rejected"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    Autonomous,
    AgentDirected,
    Hybrid,
}

impl std::fmt::Display for ExecutionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionMode::Autonomous => write!(f, "autonomous"),
            ExecutionMode::AgentDirected => write!(f, "agent_directed"),
            ExecutionMode::Hybrid => write!(f, "hybrid"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EdgeType {
    DexArb,
    CurveArb,
    Liquidation,
    Backrun,
    Jit,
    Sandwich,
    FlashLoan,
}

impl std::fmt::Display for EdgeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EdgeType::DexArb => write!(f, "dex_arb"),
            EdgeType::CurveArb => write!(f, "curve_arb"),
            EdgeType::Liquidation => write!(f, "liquidation"),
            EdgeType::Backrun => write!(f, "backrun"),
            EdgeType::Jit => write!(f, "jit"),
            EdgeType::Sandwich => write!(f, "sandwich"),
            EdgeType::FlashLoan => write!(f, "flash_loan"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEdgeRequest {
    pub strategy_id: Option<Uuid>,
    pub edge_type: String,
    pub execution_mode: ExecutionMode,
    pub atomicity: AtomicityLevel,
    pub estimated_profit_lamports: Option<i64>,
    pub risk_score: Option<i32>,
    pub route_data: serde_json::Value,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeFilter {
    pub status: Option<EdgeStatus>,
    pub edge_type: Option<String>,
    pub execution_mode: Option<ExecutionMode>,
    pub atomicity: Option<AtomicityLevel>,
    pub min_profit_lamports: Option<i64>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl Default for EdgeFilter {
    fn default() -> Self {
        Self {
            status: None,
            edge_type: None,
            execution_mode: None,
            atomicity: None,
            min_profit_lamports: None,
            limit: Some(50),
            offset: Some(0),
        }
    }
}
