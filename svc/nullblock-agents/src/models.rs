use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ==================== Core Agent Types ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub id: Uuid,
    pub content: String,
    pub role: String, // "user", "assistant", "system"
    pub timestamp: DateTime<Utc>,
    pub model_used: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl ConversationMessage {
    pub fn new(content: String, role: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            content,
            role,
            timestamp: Utc::now(),
            model_used: None,
            metadata: None,
        }
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model_used = Some(model);
        self
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    pub model_used: String,
    pub latency_ms: f64,
    pub confidence_score: f64,
    pub metadata: HashMap<String, serde_json::Value>,
}

// ==================== User Reference Types ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserReference {
    pub id: Uuid,
    pub source_identifier: String,
    pub chain: String,
    pub source_type: serde_json::Value,
    pub wallet_type: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserReferenceResponse {
    pub success: bool,
    pub data: Option<UserReference>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserReferenceListResponse {
    pub success: bool,
    pub data: Option<Vec<UserReference>>,
    pub total: usize,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

// ==================== API Request/Response Types ====================

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub user_context: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
pub struct ModelSelectionRequest {
    pub model_name: String,
}

#[derive(Debug, Deserialize)]
pub struct PersonalityRequest {
    pub personality: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessageResponse {
    pub id: String,
    pub timestamp: String,
    pub role: String,
    pub content: String,
    pub model_used: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl From<ConversationMessage> for ChatMessageResponse {
    fn from(msg: ConversationMessage) -> Self {
        Self {
            id: msg.id.to_string(),
            timestamp: msg.timestamp.to_rfc3339(),
            role: msg.role,
            content: msg.content,
            model_used: msg.model_used,
            metadata: msg.metadata,
        }
    }
}

// ==================== LLM Service Types ====================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ModelProvider {
    OpenAI,
    Anthropic,
    Groq,
    Ollama,
    HuggingFace,
    OpenRouter,
}

impl ModelProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OpenAI => "openai",
            Self::Anthropic => "anthropic", 
            Self::Groq => "groq",
            Self::Ollama => "ollama",
            Self::HuggingFace => "huggingface",
            Self::OpenRouter => "openrouter",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ModelTier {
    Free,
    Fast,
    Standard,
    Premium,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ModelCapability {
    Conversation,
    Reasoning,
    Creative,
    Vision,
    FunctionCalling,
    ReasoningTokens,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetrics {
    pub avg_latency_ms: f64,
    pub tokens_per_second: f64,
    pub cost_per_1k_tokens: f64,
    pub context_window: u32,
    pub max_output_tokens: u32,
    pub quality_score: f64,
    pub reliability_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub name: String,
    pub display_name: String,
    pub icon: String,
    pub provider: ModelProvider,
    pub tier: ModelTier,
    pub capabilities: Vec<ModelCapability>,
    pub metrics: ModelMetrics,
    pub api_endpoint: String,
    pub api_key_env: Option<String>,
    pub description: String,
    pub enabled: bool,
    pub supports_reasoning: bool,
    pub is_popular: bool,
    pub created: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMRequest {
    pub prompt: String,
    pub system_prompt: Option<String>,
    pub messages: Option<Vec<HashMap<String, String>>>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub stop_sequences: Option<Vec<String>>,
    pub tools: Option<Vec<serde_json::Value>>,
    pub model_override: Option<String>,
    pub concise: bool,
    pub max_chars: Option<u32>,
    pub reasoning: Option<ReasoningConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningConfig {
    pub enabled: bool,
    pub effort: Option<String>, // "high", "medium", "low"
    pub max_tokens: Option<u32>,
    pub exclude: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    pub content: String,
    pub model_used: String,
    pub usage: HashMap<String, u32>,
    pub latency_ms: f64,
    pub cost_estimate: f64,
    pub finish_reason: String,
    pub tool_calls: Option<Vec<serde_json::Value>>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub reasoning: Option<String>,
    pub reasoning_details: Option<Vec<serde_json::Value>>,
}

// ==================== Arbitrage Types ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageOpportunity {
    pub token_pair: String,
    pub buy_dex: String,
    pub sell_dex: String,
    pub buy_price: f64,
    pub sell_price: f64,
    pub profit_percentage: f64,
    pub profit_amount: f64,
    pub trade_amount: f64,
    pub gas_cost: f64,
    pub net_profit: f64,
    pub confidence: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageOpportunityResponse {
    pub token_pair: String,
    pub buy_dex: String,
    pub sell_dex: String,
    pub buy_price: f64,
    pub sell_price: f64,
    pub profit_percentage: f64,
    pub profit_amount: f64,
    pub trade_amount: f64,
    pub gas_cost: f64,
    pub net_profit: f64,
    pub confidence: f64,
    pub timestamp: String,
}

impl From<ArbitrageOpportunity> for ArbitrageOpportunityResponse {
    fn from(opp: ArbitrageOpportunity) -> Self {
        Self {
            token_pair: opp.token_pair,
            buy_dex: opp.buy_dex,
            sell_dex: opp.sell_dex,
            buy_price: opp.buy_price,
            sell_price: opp.sell_price,
            profit_percentage: opp.profit_percentage,
            profit_amount: opp.profit_amount,
            trade_amount: opp.trade_amount,
            gas_cost: opp.gas_cost,
            net_profit: opp.net_profit,
            confidence: opp.confidence,
            timestamp: opp.timestamp.to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSummary {
    pub pairs_monitored: u32,
    pub dexes_monitored: u32,
    pub last_update: DateTime<Utc>,
    pub opportunities_found: u32,
    pub avg_profit: f64,
    pub best_opportunity: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSummaryResponse {
    pub pairs_monitored: u32,
    pub dexes_monitored: u32,
    pub last_update: String,
    pub opportunities_found: u32,
    pub avg_profit: f64,
    pub best_opportunity: Option<HashMap<String, serde_json::Value>>,
}

impl From<MarketSummary> for MarketSummaryResponse {
    fn from(summary: MarketSummary) -> Self {
        Self {
            pairs_monitored: summary.pairs_monitored,
            dexes_monitored: summary.dexes_monitored,
            last_update: summary.last_update.to_rfc3339(),
            opportunities_found: summary.opportunities_found,
            avg_profit: summary.avg_profit,
            best_opportunity: summary.best_opportunity,
        }
    }
}

// ==================== Health Check Types ====================

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub version: String,
    pub timestamp: String,
    pub components: Option<HashMap<String, serde_json::Value>>,
}

// ==================== Error Response Types ====================

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub timestamp: String,
    pub request_id: Option<String>,
}

impl ErrorResponse {
    pub fn new(error: String, message: String) -> Self {
        Self {
            error,
            message,
            timestamp: Utc::now().to_rfc3339(),
            request_id: None,
        }
    }

    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }
}

// ==================== Task Management Types ====================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum TaskState {
    Submitted,
    Working,
    InputRequired,
    Completed,
    Canceled,
    Failed,
    Rejected,
    AuthRequired,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatus {
    pub state: TaskState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Urgent,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    Arbitrage,
    Social,
    Portfolio,
    Mcp,
    System,
    UserAssigned,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskCategory {
    Autonomous,
    UserAssigned,
    SystemGenerated,
    EventTriggered,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub description: String,
    pub task_type: TaskType,
    pub category: TaskCategory,

    // A2A Protocol required fields
    #[serde(rename = "contextId")]
    pub context_id: String,
    pub kind: String,
    pub status: TaskStatus,

    // A2A Protocol optional fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifacts: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,

    pub priority: TaskPriority,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub progress: u8,
    pub estimated_duration: Option<u64>,
    pub actual_duration: Option<u64>,
    pub sub_tasks: Vec<String>,
    pub dependencies: Vec<String>,
    pub context: HashMap<String, serde_json::Value>,
    pub parameters: HashMap<String, serde_json::Value>,
    pub outcome: Option<TaskOutcome>,
    pub logs: Vec<String>,
    pub triggers: Vec<String>,
    pub assigned_agent: Option<String>,
    pub auto_retry: bool,
    pub max_retries: u32,
    pub current_retries: u32,
    pub required_capabilities: Vec<String>,
    pub user_approval_required: bool,
    pub user_notifications: bool,

    // Action tracking fields
    pub actioned_at: Option<DateTime<Utc>>,
    pub action_result: Option<String>,
    pub action_metadata: HashMap<String, serde_json::Value>,
    pub action_duration: Option<u64>,

    // Source tracking fields
    pub source_identifier: Option<String>,
    pub source_metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskOutcome {
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub metrics: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub name: String,
    pub description: String,
    pub task_type: TaskType,
    pub category: Option<TaskCategory>,
    pub priority: Option<TaskPriority>,
    pub parameters: Option<HashMap<String, serde_json::Value>>,
    pub dependencies: Option<Vec<String>>,
    pub auto_start: Option<bool>,
    pub user_approval_required: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTaskRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<TaskState>,
    pub priority: Option<TaskPriority>,
    pub progress: Option<u8>,
    pub parameters: Option<HashMap<String, serde_json::Value>>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub outcome: Option<TaskOutcome>,
}

#[derive(Debug, Serialize)]
pub struct TaskResponse {
    pub success: bool,
    pub data: Option<Task>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct TaskListResponse {
    pub success: bool,
    pub data: Option<Vec<Task>>,
    pub total: usize,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}