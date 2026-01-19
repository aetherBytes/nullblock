use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ToolCategory {
    Scanner,
    Edge,
    Strategy,
    Curve,
    Research,
    Kol,
    Threat,
    Event,
    Engram,
    Learning,
    Consensus,
    Swarm,
    Approval,
    Position,
    Utility,
    Integration,
    Analysis,
    Unknown,
}

impl ToolCategory {
    pub fn from_tool_name(name: &str) -> Self {
        let prefix = name.split('_').next().unwrap_or("");
        match prefix {
            "scanner" => Self::Scanner,
            "edge" => Self::Edge,
            "strategy" => Self::Strategy,
            "curve" => Self::Curve,
            "research" => Self::Research,
            "kol" => Self::Kol,
            "threat" => Self::Threat,
            "event" => Self::Event,
            "engram" => Self::Engram,
            "learning" => Self::Learning,
            "consensus" => Self::Consensus,
            "swarm" => Self::Swarm,
            "approval" => Self::Approval,
            "position" => Self::Position,
            _ => Self::Unknown,
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Scanner => "🔍",
            Self::Edge => "📊",
            Self::Strategy => "♟️",
            Self::Curve => "📈",
            Self::Research => "🔬",
            Self::Kol => "👥",
            Self::Threat => "🛡️",
            Self::Event => "📡",
            Self::Engram => "🧠",
            Self::Learning => "📚",
            Self::Consensus => "🔥",
            Self::Swarm => "🐝",
            Self::Approval => "✅",
            Self::Position => "💰",
            Self::Utility => "🔧",
            Self::Integration => "🔌",
            Self::Analysis => "📋",
            Self::Unknown => "❓",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub category: ToolCategory,
    pub is_hot: bool,
    pub provider: String,
    pub related_cow: Option<String>,
    pub endpoint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredAgent {
    pub name: String,
    pub agent_type: String,
    pub status: HealthStatus,
    pub capabilities: Vec<String>,
    pub endpoint: String,
    pub provider: String,
    pub description: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredProtocol {
    pub name: String,
    pub protocol_type: String,
    pub version: String,
    pub endpoint: String,
    pub provider: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHealth {
    pub provider: String,
    pub status: HealthStatus,
    pub latency_ms: Option<u64>,
    pub last_checked: chrono::DateTime<chrono::Utc>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryResponse {
    pub tools: Vec<DiscoveredTool>,
    pub agents: Vec<DiscoveredAgent>,
    pub protocols: Vec<DiscoveredProtocol>,
    pub hot: Vec<DiscoveredTool>,
    pub provider_health: Vec<ProviderHealth>,
    pub discovery_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsResponse {
    pub tools: Vec<DiscoveredTool>,
    pub total_count: usize,
    pub hot_count: usize,
    pub categories: Vec<CategorySummary>,
    pub discovery_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorySummary {
    pub category: ToolCategory,
    pub count: usize,
    pub icon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentsResponse {
    pub agents: Vec<DiscoveredAgent>,
    pub total_count: usize,
    pub healthy_count: usize,
    pub discovery_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolsResponse {
    pub protocols: Vec<DiscoveredProtocol>,
    pub total_count: usize,
    pub discovery_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotItemsResponse {
    pub tools: Vec<DiscoveredTool>,
    pub total_count: usize,
    pub discovery_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub providers: Vec<ProviderHealth>,
    pub overall_status: HealthStatus,
    pub checked_at: chrono::DateTime<chrono::Utc>,
}
