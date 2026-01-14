use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbEngram {
    pub id: Uuid,
    pub key: String,
    pub engram_type: EngramType,
    pub content: serde_json::Value,
    pub metadata: EngramMetadata,
    pub source: EngramSource,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl ArbEngram {
    pub fn new(
        key: impl Into<String>,
        engram_type: EngramType,
        content: serde_json::Value,
        source: EngramSource,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            key: key.into(),
            engram_type,
            content,
            metadata: EngramMetadata::default(),
            source,
            confidence: 0.5,
            created_at: now,
            updated_at: now,
            expires_at: None,
        }
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    pub fn with_metadata(mut self, metadata: EngramMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn with_expiry(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EngramType {
    EdgePattern,
    Avoidance,
    Strategy,
    ThreatIntel,
    ConsensusOutcome,
    TradeResult,
    MarketCondition,
}

impl std::fmt::Display for EngramType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngramType::EdgePattern => write!(f, "edge_pattern"),
            EngramType::Avoidance => write!(f, "avoidance"),
            EngramType::Strategy => write!(f, "strategy"),
            EngramType::ThreatIntel => write!(f, "threat_intel"),
            EngramType::ConsensusOutcome => write!(f, "consensus_outcome"),
            EngramType::TradeResult => write!(f, "trade_result"),
            EngramType::MarketCondition => write!(f, "market_condition"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngramMetadata {
    pub tags: Vec<String>,
    pub related_edges: Vec<Uuid>,
    pub related_tokens: Vec<String>,
    pub related_wallets: Vec<String>,
    pub access_count: u32,
    pub last_accessed_at: Option<DateTime<Utc>>,
    pub effectiveness_score: Option<f64>,
}

impl Default for EngramMetadata {
    fn default() -> Self {
        Self {
            tags: Vec::new(),
            related_edges: Vec::new(),
            related_tokens: Vec::new(),
            related_wallets: Vec::new(),
            access_count: 0,
            last_accessed_at: None,
            effectiveness_score: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "id")]
pub enum EngramSource {
    Agent(String),
    Trade(Uuid),
    Consensus(Uuid),
    ThreatDetection(String),
    Research(Uuid),
    Manual(String),
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgePatternContent {
    pub edge_type: String,
    pub venue_type: String,
    pub route_signature: String,
    pub avg_profit_bps: f64,
    pub success_rate: f64,
    pub sample_count: u32,
    pub optimal_conditions: Vec<String>,
    pub risk_factors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvoidanceContent {
    pub entity_type: String,
    pub address: String,
    pub reason: String,
    pub category: String,
    pub severity: AvoidanceSeverity,
    pub evidence: Vec<String>,
    pub reported_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AvoidanceSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyContent {
    pub name: String,
    pub strategy_type: String,
    pub entry_conditions: Vec<String>,
    pub exit_conditions: Vec<String>,
    pub risk_parameters: serde_json::Value,
    pub backtest_results: Option<BacktestSummary>,
    pub live_performance: Option<LivePerformance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestSummary {
    pub period_days: u32,
    pub total_trades: u32,
    pub win_rate: f64,
    pub total_return_percent: f64,
    pub max_drawdown_percent: f64,
    pub sharpe_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LivePerformance {
    pub start_date: DateTime<Utc>,
    pub total_trades: u32,
    pub win_rate: f64,
    pub total_profit_sol: f64,
    pub avg_profit_per_trade: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusOutcomeContent {
    pub edge_id: Uuid,
    pub models_queried: Vec<String>,
    pub model_votes: Vec<ModelVoteContent>,
    pub final_decision: bool,
    pub agreement_score: f64,
    pub reasoning_summary: String,
    pub trade_result: Option<TradeResultSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelVoteContent {
    pub model: String,
    pub approved: bool,
    pub confidence: f64,
    pub reasoning: String,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeResultSummary {
    pub executed: bool,
    pub profit_lamports: Option<i64>,
    pub outcome: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEngramRequest {
    pub key: String,
    pub engram_type: EngramType,
    pub content: serde_json::Value,
    pub tags: Option<Vec<String>>,
    pub confidence: Option<f64>,
    pub expires_in_hours: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngramQuery {
    pub engram_type: Option<EngramType>,
    pub key_prefix: Option<String>,
    pub tag: Option<String>,
    pub min_confidence: Option<f64>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngramSearchResult {
    pub engrams: Vec<ArbEngram>,
    pub total: u64,
    pub query: EngramQuery,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatchRequest {
    pub edge_type: String,
    pub venue_type: String,
    pub token_mint: Option<String>,
    pub min_similarity: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatch {
    pub engram: ArbEngram,
    pub similarity_score: f64,
    pub recommended_action: String,
}
