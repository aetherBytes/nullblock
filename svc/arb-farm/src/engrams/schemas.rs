use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSummary {
    pub tx_signature: String,
    pub action: TransactionAction,
    pub token_mint: String,
    pub token_symbol: Option<String>,
    pub venue: String,
    pub entry_sol: f64,
    pub exit_sol: Option<f64>,
    pub pnl_sol: Option<f64>,
    pub pnl_percent: Option<f64>,
    pub slippage_bps: i32,
    pub execution_time_ms: u64,
    pub strategy_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
    #[serde(default)]
    pub metadata: TransactionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionAction {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransactionMetadata {
    pub graduation_progress: Option<f64>,
    pub holder_count: Option<u32>,
    pub volume_24h_sol: Option<f64>,
    pub market_cap_sol: Option<f64>,
    pub bonding_curve_percent: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionError {
    pub error_type: ExecutionErrorType,
    pub message: String,
    pub context: ErrorContext,
    pub stack_trace: Option<String>,
    pub recoverable: bool,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionErrorType {
    RpcTimeout,
    SlippageExceeded,
    InsufficientFunds,
    TxFailed,
    SimulationFailed,
    SigningFailed,
    NetworkError,
    InvalidParams,
    RateLimited,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub action: Option<String>,
    pub token_mint: Option<String>,
    pub attempted_amount_sol: Option<f64>,
    pub venue: Option<String>,
    pub strategy_id: Option<Uuid>,
    pub edge_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyMetrics {
    pub period: String, // YYYY-MM-DD
    pub total_trades: u32,
    pub winning_trades: u32,
    pub win_rate: f64,
    pub total_pnl_sol: f64,
    pub avg_trade_pnl: f64,
    pub max_drawdown_percent: f64,
    pub best_trade: Option<TradeHighlight>,
    pub worst_trade: Option<TradeHighlight>,
    pub by_venue: std::collections::HashMap<String, VenueMetrics>,
    pub by_strategy: std::collections::HashMap<String, StrategyMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeHighlight {
    pub token: String,
    pub pnl_sol: f64,
    pub tx_signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VenueMetrics {
    pub trades: u32,
    pub pnl_sol: f64,
    pub win_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyMetrics {
    pub trades: u32,
    pub pnl_sol: f64,
    pub win_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub recommendation_id: Uuid,
    pub source: RecommendationSource,
    pub category: RecommendationCategory,
    pub title: String,
    pub description: String,
    pub suggested_action: SuggestedAction,
    pub confidence: f64,
    pub supporting_data: SupportingData,
    pub status: RecommendationStatus,
    pub created_at: DateTime<Utc>,
    pub applied_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationSource {
    ConsensusLlm,
    PatternAnalysis,
    RiskEngine,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationCategory {
    Strategy,
    Risk,
    Timing,
    Venue,
    Position,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedAction {
    pub action_type: SuggestedActionType,
    pub target: String,
    pub current_value: Option<Value>,
    pub suggested_value: Value,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuggestedActionType {
    ConfigChange,
    StrategyToggle,
    RiskAdjustment,
    VenueDisable,
    AvoidToken,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportingData {
    pub trades_analyzed: u32,
    pub time_period: String,
    pub relevant_engrams: Vec<String>,
    pub metrics: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationStatus {
    Pending,
    Acknowledged,
    Applied,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationLog {
    pub session_id: Uuid,
    pub participants: Vec<String>,
    pub topic: ConversationTopic,
    pub context: ConversationContext,
    pub messages: Vec<ConversationMessage>,
    pub outcome: ConversationOutcome,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConversationTopic {
    TradeAnalysis,
    RiskAssessment,
    StrategyReview,
    PatternDiscovery,
    MarketConditions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationContext {
    pub trigger: ConversationTrigger,
    pub trades_in_scope: Option<u32>,
    pub time_period: Option<String>,
    pub additional_context: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConversationTrigger {
    DailyReview,
    TradeFailure,
    HighProfitTrade,
    RiskAlert,
    UserRequest,
    Scheduled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub role: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub tokens_used: Option<u32>,
    pub latency_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationOutcome {
    pub consensus_reached: bool,
    pub recommendations_generated: u32,
    pub engram_refs: Vec<String>,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusAnalysis {
    pub analysis_id: Uuid,
    pub analysis_type: ConsensusAnalysisType,
    pub time_period: String,
    pub total_trades_analyzed: u32,
    pub overall_assessment: String,
    pub risk_alerts: Vec<String>,
    pub recommendations_count: u32,
    pub recommendation_ids: Vec<Uuid>,
    pub avg_confidence: f64,
    pub models_queried: Vec<String>,
    pub total_latency_ms: u64,
    pub context_summary: AnalysisContextSummary,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsensusAnalysisType {
    TradeReview,
    RiskAssessment,
    StrategyOptimization,
    PatternDiscovery,
    Scheduled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisContextSummary {
    pub win_rate: f64,
    pub total_pnl_sol: f64,
    pub top_venue: Option<String>,
    pub error_count: u32,
}

pub fn generate_consensus_analysis_key(analysis_id: &Uuid) -> String {
    format!("arb.learning.analysis.{}", analysis_id)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusDecision {
    pub decision_id: Uuid,
    pub edge_id: Uuid,
    pub strategy_id: Option<Uuid>,
    pub approved: bool,
    pub agreement_score: f64,
    pub weighted_confidence: f64,
    pub model_votes: Vec<String>,
    pub reasoning_summary: String,
    pub edge_context: String,
    pub total_latency_ms: u64,
    pub created_at: DateTime<Utc>,
}

pub fn generate_consensus_decision_key(decision_id: &Uuid) -> String {
    format!("arb.consensus.decision.{}", decision_id)
}

pub const A2A_TAG_LEARNING: &str = "arbFarm.learning";
pub const WATCHLIST_TAG: &str = "watchlist";
pub const TRADE_ANALYSIS_TAG: &str = "arbFarm.tradeAnalysis";
pub const PATTERN_SUMMARY_TAG: &str = "arbFarm.patternSummary";
pub const RECOMMENDATION_TAG: &str = "arbFarm.recommendation";
pub const WEB_RESEARCH_TAG: &str = "arbFarm.webResearch";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebResearchEngram {
    pub research_id: Uuid,
    pub source_url: String,
    pub source_type: WebSourceType,
    pub title: Option<String>,
    pub author: Option<String>,
    pub content_summary: String,
    pub key_insights: Vec<String>,
    pub extracted_strategies: Vec<ExtractedStrategyInsight>,
    pub extracted_tokens: Vec<String>,
    pub analysis_focus: AnalysisFocus,
    pub confidence: f64,
    pub fetched_at: DateTime<Utc>,
    pub analyzed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebSourceType {
    SearchResult,
    DirectUrl,
    Tweet,
    Article,
    Documentation,
    News,
    Forum,
    Blog,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisFocus {
    Strategy,
    Alpha,
    Risk,
    TokenAnalysis,
    General,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedStrategyInsight {
    pub strategy_name: Option<String>,
    pub description: String,
    pub entry_criteria: Vec<String>,
    pub exit_criteria: Vec<String>,
    pub risk_notes: Vec<String>,
    pub confidence: f64,
}

pub fn generate_web_research_key(research_id: &Uuid) -> String {
    format!("arb.research.web.{}", research_id)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeAnalysis {
    pub analysis_id: Uuid,
    pub position_id: Uuid,
    pub token_symbol: String,
    pub venue: String,
    pub pnl_sol: f64,
    pub exit_reason: String,
    pub root_cause: String,
    pub config_issue: Option<String>,
    pub pattern: Option<String>,
    pub suggested_fix: Option<String>,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
}

pub fn generate_trade_analysis_key(analysis_id: &Uuid) -> String {
    format!("arb.learning.trade_analysis.{}", analysis_id)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredPatternSummary {
    pub summary_id: Uuid,
    pub losing_patterns: Vec<String>,
    pub winning_patterns: Vec<String>,
    pub config_recommendations: Vec<String>,
    pub trades_analyzed: u32,
    pub time_period: String,
    pub created_at: DateTime<Utc>,
}

pub fn generate_pattern_summary_key(summary_id: &Uuid) -> String {
    format!("arb.learning.pattern_summary.{}", summary_id)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeAnalysisEngramWrapper {
    pub engram_id: String,
    pub engram_key: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub analysis: TradeAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternSummaryEngramWrapper {
    pub engram_id: String,
    pub engram_key: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub summary: StoredPatternSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchlistToken {
    pub mint: String,
    pub name: String,
    pub symbol: String,
    pub venue: String,
    pub tracked_at: DateTime<Utc>,
    pub notes: Option<String>,
    pub last_progress: Option<f64>,
}

pub fn generate_watchlist_key(mint: &str) -> String {
    format!("arb.watchlist.{}", mint)
}

pub fn generate_transaction_key(tx_signature: &str) -> String {
    format!("arb.trade.summary.{}", tx_signature)
}

pub fn generate_error_key(error_type: &ExecutionErrorType, timestamp: &DateTime<Utc>) -> String {
    let error_type_str = serde_json::to_string(error_type)
        .unwrap_or_else(|_| "unknown".to_string())
        .trim_matches('"')
        .to_string();
    format!("arb.error.{}.{}", error_type_str, timestamp.timestamp())
}

pub fn generate_metrics_key(period: &str) -> String {
    format!("arb.metrics.daily.{}", period)
}

pub fn generate_recommendation_key(id: &Uuid) -> String {
    format!("arb.learning.recommendation.{}", id)
}

pub fn generate_conversation_key(session_id: &Uuid) -> String {
    format!("arb.learning.conversation.{}", session_id)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngramWrapper<T> {
    pub engram_id: String,
    pub engram_key: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub content: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeEngramWrapper {
    pub engram_id: String,
    pub engram_key: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub trade: TransactionSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationEngramWrapper {
    pub engram_id: String,
    pub engram_key: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub recommendation: Recommendation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEngramWrapper {
    pub engram_id: String,
    pub engram_key: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub error: ExecutionError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationEngramWrapper {
    pub engram_id: String,
    pub engram_key: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub conversation: ConversationLog,
}
