use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
    AutoApproved,
}

impl std::fmt::Display for ApprovalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApprovalStatus::Pending => write!(f, "pending"),
            ApprovalStatus::Approved => write!(f, "approved"),
            ApprovalStatus::Rejected => write!(f, "rejected"),
            ApprovalStatus::Expired => write!(f, "expired"),
            ApprovalStatus::AutoApproved => write!(f, "auto_approved"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalType {
    Entry,
    Exit,
    Emergency,
}

impl std::fmt::Display for ApprovalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApprovalType::Entry => write!(f, "entry"),
            ApprovalType::Exit => write!(f, "exit"),
            ApprovalType::Emergency => write!(f, "emergency"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingApproval {
    pub id: Uuid,
    pub edge_id: Option<Uuid>,
    pub position_id: Option<Uuid>,
    pub strategy_id: Option<Uuid>,
    pub approval_type: ApprovalType,
    pub status: ApprovalStatus,
    pub estimated_profit_lamports: Option<i64>,
    pub risk_score: Option<i32>,
    pub token_mint: Option<String>,
    pub token_symbol: Option<String>,
    pub amount_sol: Option<f64>,
    pub context: serde_json::Value,
    pub expires_at: DateTime<Utc>,
    pub hecate_decision: Option<bool>,
    pub hecate_reasoning: Option<String>,
    pub hecate_confidence: Option<f64>,
    pub user_decision: Option<bool>,
    pub user_decided_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl PendingApproval {
    pub fn new_entry(
        edge_id: Uuid,
        strategy_id: Option<Uuid>,
        estimated_profit_lamports: Option<i64>,
        risk_score: Option<i32>,
        token_mint: Option<String>,
        token_symbol: Option<String>,
        amount_sol: Option<f64>,
        context: serde_json::Value,
        timeout_secs: u64,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            edge_id: Some(edge_id),
            position_id: None,
            strategy_id,
            approval_type: ApprovalType::Entry,
            status: ApprovalStatus::Pending,
            estimated_profit_lamports,
            risk_score,
            token_mint,
            token_symbol,
            amount_sol,
            context,
            expires_at: Utc::now() + chrono::Duration::seconds(timeout_secs as i64),
            hecate_decision: None,
            hecate_reasoning: None,
            hecate_confidence: None,
            user_decision: None,
            user_decided_at: None,
            created_at: Utc::now(),
        }
    }

    pub fn new_exit(
        position_id: Uuid,
        strategy_id: Option<Uuid>,
        token_mint: Option<String>,
        token_symbol: Option<String>,
        amount_sol: Option<f64>,
        context: serde_json::Value,
        timeout_secs: u64,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            edge_id: None,
            position_id: Some(position_id),
            strategy_id,
            approval_type: ApprovalType::Exit,
            status: ApprovalStatus::Pending,
            estimated_profit_lamports: None,
            risk_score: None,
            token_mint,
            token_symbol,
            amount_sol,
            context,
            expires_at: Utc::now() + chrono::Duration::seconds(timeout_secs as i64),
            hecate_decision: None,
            hecate_reasoning: None,
            hecate_confidence: None,
            user_decision: None,
            user_decided_at: None,
            created_at: Utc::now(),
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn time_remaining_secs(&self) -> i64 {
        let remaining = self.expires_at - Utc::now();
        remaining.num_seconds().max(0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalExecutionConfig {
    pub auto_execution_enabled: bool,
    pub default_approval_timeout_secs: u64,
    pub notify_hecate_on_pending: bool,
    pub require_hecate_approval: bool,
    pub max_pending_approvals: usize,
    pub auto_approve_atomic: bool,
    pub auto_approve_min_profit_bps: Option<u16>,
    pub auto_approve_max_risk_score: Option<i32>,
    pub emergency_exit_enabled: bool,
    pub auto_min_confidence: f64,
    pub auto_max_position_sol: f64,
    pub require_simulation: bool,
    pub updated_at: DateTime<Utc>,
}

impl Default for GlobalExecutionConfig {
    fn default() -> Self {
        Self {
            auto_execution_enabled: false,
            default_approval_timeout_secs: 300,
            notify_hecate_on_pending: true,
            require_hecate_approval: false,
            max_pending_approvals: 10,
            auto_approve_atomic: true,
            auto_approve_min_profit_bps: Some(100),
            auto_approve_max_risk_score: Some(30),
            emergency_exit_enabled: true,
            auto_min_confidence: 0.8,
            auto_max_position_sol: 0.5,
            require_simulation: true,
            updated_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproveRequest {
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectRequest {
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateExecutionConfigRequest {
    pub auto_execution_enabled: Option<bool>,
    pub default_approval_timeout_secs: Option<u64>,
    pub notify_hecate_on_pending: Option<bool>,
    pub require_hecate_approval: Option<bool>,
    pub max_pending_approvals: Option<usize>,
    pub auto_approve_atomic: Option<bool>,
    pub auto_approve_min_profit_bps: Option<u16>,
    pub auto_approve_max_risk_score: Option<i32>,
    pub emergency_exit_enabled: Option<bool>,
    pub auto_min_confidence: Option<f64>,
    pub auto_max_position_sol: Option<f64>,
    pub require_simulation: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionToggleRequest {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HecateRecommendation {
    pub approval_id: Uuid,
    pub decision: bool,
    pub reasoning: String,
    pub confidence: f64,
}
