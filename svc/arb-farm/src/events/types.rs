use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbEvent {
    pub id: Uuid,
    pub event_type: String,
    pub source: EventSource,
    pub topic: String,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Option<Uuid>,
}

impl ArbEvent {
    pub fn new(
        event_type: impl Into<String>,
        source: EventSource,
        topic: impl Into<String>,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type: event_type.into(),
            source,
            topic: topic.into(),
            payload,
            timestamp: Utc::now(),
            correlation_id: None,
        }
    }

    pub fn with_correlation(mut self, correlation_id: Uuid) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "id")]
pub enum EventSource {
    Agent(AgentType),
    Tool(String),
    External(String),
    System,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentType {
    Scanner,
    Refiner,
    MevHunter,
    Executor,
    StrategyEngine,
    ResearchDd,
    CopyTrade,
    ThreatDetector,
    EngramHarvester,
    Overseer,
    ApprovalManager,
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentType::Scanner => write!(f, "scanner"),
            AgentType::Refiner => write!(f, "refiner"),
            AgentType::MevHunter => write!(f, "mev_hunter"),
            AgentType::Executor => write!(f, "executor"),
            AgentType::StrategyEngine => write!(f, "strategy_engine"),
            AgentType::ResearchDd => write!(f, "research_dd"),
            AgentType::CopyTrade => write!(f, "copy_trade"),
            AgentType::ThreatDetector => write!(f, "threat_detector"),
            AgentType::EngramHarvester => write!(f, "engram_harvester"),
            AgentType::Overseer => write!(f, "overseer"),
            AgentType::ApprovalManager => write!(f, "approval_manager"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Significance {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AtomicityLevel {
    FullyAtomic,
    PartiallyAtomic,
    NonAtomic,
}
