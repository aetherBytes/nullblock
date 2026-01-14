use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use crate::error::AppResult;
use crate::events::{ArbEvent, AgentType, AtomicityLevel, EventSource, edge as edge_topics, strategy as strategy_topics};
use crate::models::{Edge, EdgeStatus, RiskParams, Signal, Strategy};

pub struct StrategyEngine {
    id: Uuid,
    strategies: Arc<RwLock<HashMap<Uuid, Strategy>>>,
    event_tx: broadcast::Sender<ArbEvent>,
}

#[derive(Debug, Clone)]
pub struct MatchResult {
    pub signal_id: Uuid,
    pub strategy_id: Uuid,
    pub execution_mode: String,
    pub approved: bool,
    pub reason: Option<String>,
    pub created_edge: Option<Edge>,
}

impl StrategyEngine {
    pub fn new(event_tx: broadcast::Sender<ArbEvent>) -> Self {
        Self {
            id: Uuid::new_v4(),
            strategies: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub async fn add_strategy(&self, strategy: Strategy) {
        let strategy_id = strategy.id;
        let mut strategies = self.strategies.write().await;
        strategies.insert(strategy_id, strategy.clone());

        let _ = self.event_tx.send(ArbEvent::new(
            "strategy_created",
            EventSource::Agent(AgentType::StrategyEngine),
            strategy_topics::CREATED,
            serde_json::json!({
                "strategy_id": strategy_id,
                "name": strategy.name,
                "execution_mode": format!("{:?}", strategy.execution_mode),
            }),
        ));
    }

    pub async fn remove_strategy(&self, strategy_id: Uuid) -> bool {
        let mut strategies = self.strategies.write().await;
        if strategies.remove(&strategy_id).is_some() {
            let _ = self.event_tx.send(ArbEvent::new(
                "strategy_deleted",
                EventSource::Agent(AgentType::StrategyEngine),
                strategy_topics::DELETED,
                serde_json::json!({
                    "strategy_id": strategy_id,
                }),
            ));
            true
        } else {
            false
        }
    }

    pub async fn toggle_strategy(&self, strategy_id: Uuid, enabled: bool) -> AppResult<()> {
        let mut strategies = self.strategies.write().await;
        if let Some(strategy) = strategies.get_mut(&strategy_id) {
            strategy.is_active = enabled;
            let topic = if enabled { strategy_topics::ENABLED } else { strategy_topics::DISABLED };
            let _ = self.event_tx.send(ArbEvent::new(
                if enabled { "strategy_enabled" } else { "strategy_disabled" },
                EventSource::Agent(AgentType::StrategyEngine),
                topic,
                serde_json::json!({
                    "strategy_id": strategy_id,
                    "enabled": enabled,
                }),
            ));
            Ok(())
        } else {
            Err(crate::error::AppError::NotFound(format!("Strategy {} not found", strategy_id)))
        }
    }

    pub async fn get_strategy(&self, strategy_id: Uuid) -> Option<Strategy> {
        let strategies = self.strategies.read().await;
        strategies.get(&strategy_id).cloned()
    }

    pub async fn list_strategies(&self) -> Vec<Strategy> {
        let strategies = self.strategies.read().await;
        strategies.values().cloned().collect()
    }

    pub async fn match_signal(&self, signal: &Signal) -> Option<MatchResult> {
        let strategies = self.strategies.read().await;

        for strategy in strategies.values() {
            if !strategy.is_active {
                continue;
            }

            if !self.signal_matches_strategy(signal, strategy) {
                continue;
            }

            let risk_params = self.get_risk_params(strategy);

            if !self.check_risk_params(signal, &risk_params) {
                return Some(MatchResult {
                    signal_id: signal.id,
                    strategy_id: strategy.id,
                    execution_mode: strategy.execution_mode.clone(),
                    approved: false,
                    reason: Some("Signal exceeds risk parameters".to_string()),
                    created_edge: None,
                });
            }

            let edge = self.create_edge_from_signal(signal, strategy);

            let _ = self.event_tx.send(ArbEvent::new(
                "edge_detected",
                EventSource::Agent(AgentType::StrategyEngine),
                edge_topics::DETECTED,
                serde_json::json!({
                    "edge_id": edge.id,
                    "signal_id": signal.id,
                    "strategy_id": strategy.id,
                    "execution_mode": format!("{:?}", strategy.execution_mode),
                    "estimated_profit_bps": signal.estimated_profit_bps,
                }),
            ));

            return Some(MatchResult {
                signal_id: signal.id,
                strategy_id: strategy.id,
                execution_mode: strategy.execution_mode.clone(),
                approved: true,
                reason: None,
                created_edge: Some(edge),
            });
        }

        None
    }

    fn get_risk_params(&self, strategy: &Strategy) -> RiskParams {
        strategy.risk_params.clone()
    }

    fn signal_matches_strategy(&self, signal: &Signal, strategy: &Strategy) -> bool {
        let signal_venue_str = format!("{:?}", signal.venue_type).to_lowercase();
        strategy.venue_types.iter().any(|vt| {
            vt.to_lowercase() == signal_venue_str ||
            vt.to_lowercase().contains(&signal_venue_str)
        })
    }

    fn check_risk_params(&self, signal: &Signal, risk_params: &RiskParams) -> bool {
        if signal.estimated_profit_bps < risk_params.min_profit_bps as i32 {
            return false;
        }

        if signal.confidence < 0.1 {
            return false;
        }

        true
    }

    fn create_edge_from_signal(&self, signal: &Signal, strategy: &Strategy) -> Edge {
        Edge {
            id: Uuid::new_v4(),
            strategy_id: Some(strategy.id),
            edge_type: format!("{:?}", signal.signal_type),
            execution_mode: strategy.execution_mode.clone(),
            atomicity: AtomicityLevel::NonAtomic,
            simulated_profit_guaranteed: false,
            estimated_profit_lamports: Some((signal.estimated_profit_bps as i64) * 10000),
            risk_score: Some(((1.0 - signal.confidence) * 100.0) as i32),
            route_data: signal.metadata.clone(),
            status: EdgeStatus::Detected,
            token_mint: signal.token_mint.clone(),
            created_at: chrono::Utc::now(),
            expires_at: Some(signal.expires_at),
        }
    }

    pub async fn process_signals(&self, signals: Vec<Signal>) -> Vec<MatchResult> {
        let mut results = Vec::new();

        for signal in signals {
            if let Some(result) = self.match_signal(&signal).await {
                results.push(result);
            }
        }

        results
    }
}
