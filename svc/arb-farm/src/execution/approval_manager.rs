use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;
use chrono::Utc;

use crate::error::{AppError, AppResult};
use crate::events::{ArbEvent, AgentType, EventSource, approval as approval_topics};
use crate::models::{
    ApprovalStatus, ApprovalType, GlobalExecutionConfig, HecateRecommendation,
    PendingApproval, UpdateExecutionConfigRequest,
};

pub struct ApprovalManager {
    pending: Arc<RwLock<HashMap<Uuid, PendingApproval>>>,
    config: Arc<RwLock<GlobalExecutionConfig>>,
    event_tx: broadcast::Sender<ArbEvent>,
}

impl ApprovalManager {
    pub fn new(event_tx: broadcast::Sender<ArbEvent>) -> Self {
        Self {
            pending: Arc::new(RwLock::new(HashMap::new())),
            config: Arc::new(RwLock::new(GlobalExecutionConfig::default())),
            event_tx,
        }
    }

    pub async fn get_config(&self) -> GlobalExecutionConfig {
        self.config.read().await.clone()
    }

    pub async fn update_config(&self, request: UpdateExecutionConfigRequest) -> GlobalExecutionConfig {
        let mut config = self.config.write().await;

        if let Some(v) = request.auto_execution_enabled {
            config.auto_execution_enabled = v;
        }
        if let Some(v) = request.default_approval_timeout_secs {
            config.default_approval_timeout_secs = v;
        }
        if let Some(v) = request.notify_hecate_on_pending {
            config.notify_hecate_on_pending = v;
        }
        if let Some(v) = request.require_hecate_approval {
            config.require_hecate_approval = v;
        }
        if let Some(v) = request.max_pending_approvals {
            config.max_pending_approvals = v;
        }
        if let Some(v) = request.auto_approve_atomic {
            config.auto_approve_atomic = v;
        }
        if let Some(v) = request.auto_approve_min_profit_bps {
            config.auto_approve_min_profit_bps = Some(v);
        }
        if let Some(v) = request.auto_approve_max_risk_score {
            config.auto_approve_max_risk_score = Some(v);
        }
        if let Some(v) = request.emergency_exit_enabled {
            config.emergency_exit_enabled = v;
        }

        config.updated_at = Utc::now();

        let _ = self.event_tx.send(ArbEvent::new(
            "execution_config_updated",
            EventSource::Agent(AgentType::ApprovalManager),
            approval_topics::CONFIG_UPDATED,
            serde_json::json!({
                "auto_execution_enabled": config.auto_execution_enabled,
                "notify_hecate": config.notify_hecate_on_pending,
            }),
        ));

        config.clone()
    }

    pub async fn toggle_execution(&self, enabled: bool) -> GlobalExecutionConfig {
        let mut config = self.config.write().await;
        config.auto_execution_enabled = enabled;
        config.updated_at = Utc::now();

        let topic = if enabled {
            approval_topics::EXECUTION_ENABLED
        } else {
            approval_topics::EXECUTION_DISABLED
        };

        let _ = self.event_tx.send(ArbEvent::new(
            if enabled { "execution_enabled" } else { "execution_disabled" },
            EventSource::Agent(AgentType::ApprovalManager),
            topic,
            serde_json::json!({
                "enabled": enabled,
            }),
        ));

        config.clone()
    }

    pub async fn create_approval(&self, approval: PendingApproval) -> AppResult<PendingApproval> {
        let config = self.config.read().await;

        {
            let pending = self.pending.read().await;
            if pending.len() >= config.max_pending_approvals {
                return Err(AppError::Validation(
                    "Maximum pending approvals reached".to_string(),
                ));
            }
        }

        let should_auto_approve = self.should_auto_approve(&approval, &config);

        let mut approval = approval;
        if should_auto_approve {
            approval.status = ApprovalStatus::AutoApproved;
            approval.user_decision = Some(true);
            approval.user_decided_at = Some(Utc::now());
        }

        let approval_id = approval.id;
        let approval_type = approval.approval_type;

        {
            let mut pending = self.pending.write().await;
            pending.insert(approval_id, approval.clone());
        }

        let topic = if should_auto_approve {
            approval_topics::AUTO_APPROVED
        } else {
            approval_topics::CREATED
        };

        let _ = self.event_tx.send(ArbEvent::new(
            if should_auto_approve { "approval_auto_approved" } else { "approval_created" },
            EventSource::Agent(AgentType::ApprovalManager),
            topic,
            serde_json::json!({
                "approval_id": approval_id,
                "approval_type": format!("{}", approval_type),
                "auto_approved": should_auto_approve,
                "expires_at": approval.expires_at.to_rfc3339(),
            }),
        ));

        if config.notify_hecate_on_pending && !should_auto_approve {
            let _ = self.event_tx.send(ArbEvent::new(
                "approval_pending_hecate_notification",
                EventSource::Agent(AgentType::ApprovalManager),
                approval_topics::HECATE_NOTIFIED,
                serde_json::json!({
                    "approval_id": approval_id,
                    "approval_type": format!("{}", approval_type),
                    "context": approval.context,
                }),
            ));
        }

        Ok(approval)
    }

    fn should_auto_approve(&self, approval: &PendingApproval, config: &GlobalExecutionConfig) -> bool {
        if !config.auto_execution_enabled {
            return false;
        }

        if config.auto_approve_atomic {
            if let Some(profit) = approval.estimated_profit_lamports {
                if profit > 0 {
                    if let Some(risk) = approval.risk_score {
                        if let Some(max_risk) = config.auto_approve_max_risk_score {
                            if risk <= max_risk {
                                return true;
                            }
                        }
                    }
                }
            }
        }

        if let Some(min_profit) = config.auto_approve_min_profit_bps {
            if let Some(profit_lamports) = approval.estimated_profit_lamports {
                let profit_bps = (profit_lamports as f64 / 1_000_000_000.0 * 10000.0) as u16;
                if profit_bps >= min_profit {
                    if let Some(max_risk) = config.auto_approve_max_risk_score {
                        if let Some(risk) = approval.risk_score {
                            if risk <= max_risk {
                                return true;
                            }
                        }
                    } else {
                        return true;
                    }
                }
            }
        }

        false
    }

    pub async fn approve(&self, approval_id: Uuid, notes: Option<String>) -> AppResult<PendingApproval> {
        let mut pending = self.pending.write().await;

        let approval = pending.get_mut(&approval_id)
            .ok_or_else(|| AppError::NotFound(format!("Approval {} not found", approval_id)))?;

        if approval.is_expired() {
            approval.status = ApprovalStatus::Expired;
            return Err(AppError::Validation("Approval has expired".to_string()));
        }

        if approval.status != ApprovalStatus::Pending {
            return Err(AppError::Validation(format!(
                "Approval is not pending (current: {})",
                approval.status
            )));
        }

        approval.status = ApprovalStatus::Approved;
        approval.user_decision = Some(true);
        approval.user_decided_at = Some(Utc::now());

        if let Some(notes) = notes {
            if let Some(obj) = approval.context.as_object_mut() {
                obj.insert("approval_notes".to_string(), serde_json::json!(notes));
            }
        }

        let _ = self.event_tx.send(ArbEvent::new(
            "approval_approved",
            EventSource::Agent(AgentType::ApprovalManager),
            approval_topics::APPROVED,
            serde_json::json!({
                "approval_id": approval_id,
                "edge_id": approval.edge_id,
                "position_id": approval.position_id,
            }),
        ));

        Ok(approval.clone())
    }

    pub async fn reject(&self, approval_id: Uuid, reason: String) -> AppResult<PendingApproval> {
        let mut pending = self.pending.write().await;

        let approval = pending.get_mut(&approval_id)
            .ok_or_else(|| AppError::NotFound(format!("Approval {} not found", approval_id)))?;

        if approval.status != ApprovalStatus::Pending {
            return Err(AppError::Validation(format!(
                "Approval is not pending (current: {})",
                approval.status
            )));
        }

        approval.status = ApprovalStatus::Rejected;
        approval.user_decision = Some(false);
        approval.user_decided_at = Some(Utc::now());

        if let Some(obj) = approval.context.as_object_mut() {
            obj.insert("rejection_reason".to_string(), serde_json::json!(reason.clone()));
        }

        let _ = self.event_tx.send(ArbEvent::new(
            "approval_rejected",
            EventSource::Agent(AgentType::ApprovalManager),
            approval_topics::REJECTED,
            serde_json::json!({
                "approval_id": approval_id,
                "edge_id": approval.edge_id,
                "position_id": approval.position_id,
                "reason": reason,
            }),
        ));

        Ok(approval.clone())
    }

    pub async fn add_hecate_recommendation(&self, recommendation: HecateRecommendation) -> AppResult<PendingApproval> {
        let mut pending = self.pending.write().await;

        let approval = pending.get_mut(&recommendation.approval_id)
            .ok_or_else(|| AppError::NotFound(format!("Approval {} not found", recommendation.approval_id)))?;

        approval.hecate_decision = Some(recommendation.decision);
        approval.hecate_reasoning = Some(recommendation.reasoning.clone());
        approval.hecate_confidence = Some(recommendation.confidence);

        let _ = self.event_tx.send(ArbEvent::new(
            "hecate_recommendation_received",
            EventSource::Agent(AgentType::ApprovalManager),
            approval_topics::HECATE_RECOMMENDED,
            serde_json::json!({
                "approval_id": recommendation.approval_id,
                "decision": recommendation.decision,
                "confidence": recommendation.confidence,
            }),
        ));

        Ok(approval.clone())
    }

    pub async fn get_approval(&self, approval_id: Uuid) -> Option<PendingApproval> {
        let pending = self.pending.read().await;
        pending.get(&approval_id).cloned()
    }

    pub async fn list_pending(&self) -> Vec<PendingApproval> {
        let pending = self.pending.read().await;
        pending
            .values()
            .filter(|a| a.status == ApprovalStatus::Pending)
            .cloned()
            .collect()
    }

    pub async fn list_all(&self) -> Vec<PendingApproval> {
        let pending = self.pending.read().await;
        pending.values().cloned().collect()
    }

    pub async fn cleanup_expired(&self) -> Vec<Uuid> {
        let mut pending = self.pending.write().await;
        let mut expired_ids = Vec::new();

        for (id, approval) in pending.iter_mut() {
            if approval.status == ApprovalStatus::Pending && approval.is_expired() {
                approval.status = ApprovalStatus::Expired;
                expired_ids.push(*id);
            }
        }

        for id in &expired_ids {
            let _ = self.event_tx.send(ArbEvent::new(
                "approval_expired",
                EventSource::Agent(AgentType::ApprovalManager),
                approval_topics::EXPIRED,
                serde_json::json!({
                    "approval_id": id,
                }),
            ));
        }

        expired_ids
    }

    pub async fn remove_completed(&self, max_age_secs: i64) {
        let mut pending = self.pending.write().await;
        let cutoff = Utc::now() - chrono::Duration::seconds(max_age_secs);

        pending.retain(|_, approval| {
            approval.status == ApprovalStatus::Pending
                || approval.user_decided_at.map(|t| t > cutoff).unwrap_or(true)
        });
    }

    pub async fn cancel_by_strategy(&self, strategy_id: Uuid) -> AppResult<Vec<Uuid>> {
        let mut pending = self.pending.write().await;
        let mut cancelled_ids = Vec::new();

        for (id, approval) in pending.iter_mut() {
            if approval.strategy_id == Some(strategy_id) && approval.status == ApprovalStatus::Pending {
                approval.status = ApprovalStatus::Rejected;
                approval.user_decision = Some(false);
                approval.user_decided_at = Some(Utc::now());

                if let Some(obj) = approval.context.as_object_mut() {
                    obj.insert(
                        "rejection_reason".to_string(),
                        serde_json::json!("Strategy killed - all operations cancelled"),
                    );
                }

                cancelled_ids.push(*id);

                let _ = self.event_tx.send(ArbEvent::new(
                    "approval_cancelled_by_kill",
                    EventSource::Agent(AgentType::ApprovalManager),
                    approval_topics::REJECTED,
                    serde_json::json!({
                        "approval_id": id,
                        "strategy_id": strategy_id,
                        "reason": "strategy_killed",
                    }),
                ));
            }
        }

        tracing::info!(
            strategy_id = %strategy_id,
            cancelled_count = cancelled_ids.len(),
            "Cancelled pending approvals for killed strategy"
        );

        Ok(cancelled_ids)
    }
}
