use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceStatus {
    #[serde(rename = "healthy")]
    Healthy,
    #[serde(rename = "unhealthy")]
    Unhealthy,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: ServiceStatus,
    pub service: String,
    pub timestamp: DateTime<Utc>,
    pub components: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ListingType {
    Agent,
    Workflow,
    Tool,
    McpServer,
    Dataset,
    Model,
    ArbFarm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ListingStatus {
    Active,
    Inactive,
    Pending,
    Rejected,
    Featured,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Listing {
    pub id: Uuid,
    pub listing_type: ListingType,
    pub title: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub tags: Vec<String>,
    pub status: ListingStatus,
    pub price: Option<f64>,
    pub is_free: bool,
    pub rating: Option<f32>,
    pub total_ratings: i32,
    pub download_count: i64,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateListingRequest {
    pub listing_type: ListingType,
    pub title: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub tags: Vec<String>,
    pub price: Option<f64>,
    pub metadata: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: Option<String>,
    pub listing_type: Option<ListingType>,
    pub tags: Option<Vec<String>>,
    pub author: Option<String>,
    pub is_free: Option<bool>,
    pub min_rating: Option<f32>,
    pub sort_by: Option<SortBy>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SortBy {
    CreatedAt,
    UpdatedAt,
    Rating,
    Downloads,
    Name,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub listings: Vec<Listing>,
    pub total_count: i64,
    pub page: i32,
    pub per_page: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpServerInfo {
    pub name: String,
    pub description: String,
    pub endpoint: String,
    pub capabilities: Vec<String>,
    pub protocol_version: String,
    pub health_check_url: Option<String>,
    pub documentation_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentInfo {
    pub name: String,
    pub description: String,
    pub capabilities: Vec<String>,
    pub supported_models: Vec<String>,
    pub endpoint: String,
    pub config_schema: Option<Value>,
    pub example_usage: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowInfo {
    pub name: String,
    pub description: String,
    pub steps: Vec<WorkflowStep>,
    pub input_schema: Value,
    pub output_schema: Value,
    pub estimated_cost: Option<f64>,
    pub estimated_duration: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub name: String,
    pub description: String,
    pub agent_type: Option<String>,
    pub tool_name: Option<String>,
    pub config: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscoveryStats {
    pub total_listings: i64,
    pub active_listings: i64,
    pub agents_count: i64,
    pub workflows_count: i64,
    pub tools_count: i64,
    pub mcp_servers_count: i64,
    pub featured_count: i64,
    pub last_updated: DateTime<Utc>,
}

// Blockchain and tokenization models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenizedAsset {
    pub id: Uuid,
    pub asset_type: ListingType,
    pub contract_address: String,
    pub token_id: String,
    pub chain_id: i32,
    pub owner_address: String,
    pub metadata_uri: String,
    pub royalty_percentage: Option<f64>,
    pub is_tradeable: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerRegistration {
    pub id: Uuid,
    pub server_name: String,
    pub endpoint_url: String,
    pub metadata: McpServerMetadata,
    pub owner_address: Option<String>,
    pub registration_type: RegistrationType,
    pub verification_status: VerificationStatus,
    pub auto_discovery_enabled: bool,
    pub sampling_enabled: bool,
    pub pricing_model: PricingModel,
    pub created_at: DateTime<Utc>,
    pub last_heartbeat: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerMetadata {
    pub protocol_version: String,
    pub capabilities: Vec<String>,
    pub resources: Vec<McpResource>,
    pub tools: Vec<McpTool>,
    pub prompts: Vec<McpPrompt>,
    pub sampling_config: Option<SamplingConfig>,
    pub contact_info: Option<ContactInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
    pub name: String,
    pub description: String,
    pub mime_type: Option<String>,
    pub schema: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub output_schema: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPrompt {
    pub name: String,
    pub description: String,
    pub arguments: Vec<PromptArgument>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptArgument {
    pub name: String,
    pub description: String,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingConfig {
    pub allows_outbound_sampling: bool,
    pub accepts_inbound_sampling: bool,
    pub rate_limits: Option<RateLimits>,
    pub authentication_required: bool,
    pub supported_protocols: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimits {
    pub requests_per_minute: Option<i32>,
    pub requests_per_hour: Option<i32>,
    pub requests_per_day: Option<i32>,
    pub concurrent_connections: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactInfo {
    pub name: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub documentation_url: Option<String>,
    pub support_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegistrationType {
    Automatic,
    Manual,
    Verified,
    Community,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationStatus {
    Pending,
    Verified,
    Rejected,
    Flagged,
    Trusted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PricingModel {
    Free,
    PayPerUse,
    Subscription,
    TokenStaking,
    RevenueShare,
}

// Trading and wealth distribution models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingPair {
    pub id: Uuid,
    pub asset_a: Uuid, // TokenizedAsset ID
    pub asset_b: Uuid, // TokenizedAsset ID or native token
    pub exchange_rate: f64,
    pub volume_24h: f64,
    pub price_change_24h: f64,
    pub liquidity: f64,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WealthDistribution {
    pub id: Uuid,
    pub pool_name: String,
    pub total_rewards: f64,
    pub distribution_rules: Vec<DistributionRule>,
    pub eligible_participants: Vec<String>, // wallet addresses
    pub distribution_schedule: DistributionSchedule,
    pub created_at: DateTime<Utc>,
    pub next_distribution: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionRule {
    pub criteria_type: DistributionCriteria,
    pub weight_percentage: f64,
    pub minimum_threshold: Option<f64>,
    pub maximum_cap: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DistributionCriteria {
    Usage,
    Staking,
    Liquidity,
    Contribution,
    Governance,
    Performance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DistributionSchedule {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    EventBased,
}

// Agent interoperability models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInterface {
    pub id: Uuid,
    pub agent_id: String,
    pub interface_version: String,
    pub mcp_compatible: bool,
    pub supported_protocols: Vec<String>,
    pub capability_matrix: CapabilityMatrix,
    pub integration_instructions: Option<String>,
    pub schema_definitions: Vec<SchemaDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityMatrix {
    pub can_receive_tasks: bool,
    pub can_delegate_tasks: bool,
    pub can_compose_workflows: bool,
    pub can_access_mcp_servers: bool,
    pub can_trade_assets: bool,
    pub can_participate_governance: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaDefinition {
    pub name: String,
    pub version: String,
    pub schema_type: SchemaType,
    pub definition: Value,
    pub examples: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchemaType {
    Task,
    Response,
    Configuration,
    Metadata,
    Event,
}

// =============================================================================
// ArbFarm COW (Constellation of Work) Models
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbFarmCow {
    pub id: Uuid,
    pub listing_id: Uuid,
    pub creator_wallet: String,
    pub name: String,
    pub description: String,
    pub strategies: Vec<ArbFarmStrategy>,
    pub venue_types: Vec<ArbFarmVenueType>,
    pub risk_profile: ArbFarmRiskProfile,
    pub parent_cow_id: Option<Uuid>,
    pub fork_count: i32,
    pub total_profit_generated_lamports: i64,
    pub total_trades: i32,
    pub win_rate: f32,
    pub creator_revenue_share_bps: i32,
    pub fork_revenue_share_bps: i32,
    pub is_public: bool,
    pub is_forkable: bool,
    pub inherited_engrams: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbFarmStrategy {
    pub id: Uuid,
    pub name: String,
    pub strategy_type: ArbFarmStrategyType,
    pub venue_types: Vec<ArbFarmVenueType>,
    pub execution_mode: ArbFarmExecutionMode,
    pub risk_params: ArbFarmRiskParams,
    pub is_active: bool,
    pub performance: Option<ArbFarmStrategyPerformance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArbFarmStrategyType {
    DexArb,
    CurveArb,
    Liquidation,
    JitLiquidity,
    Backrun,
    Sandwich,
    CopyTrade,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArbFarmVenueType {
    DexAmm,
    BondingCurve,
    Lending,
    Orderbook,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArbFarmExecutionMode {
    Autonomous,
    AgentDirected,
    Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbFarmRiskParams {
    pub max_position_sol: f64,
    pub min_profit_bps: i32,
    pub max_slippage_bps: i32,
    pub max_risk_score: i32,
    pub daily_loss_limit_sol: Option<f64>,
    pub require_consensus: bool,
    pub min_consensus_agreement: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbFarmRiskProfile {
    pub profile_type: ArbFarmRiskProfileType,
    pub max_position_sol: f64,
    pub daily_loss_limit_sol: f64,
    pub max_concurrent_positions: i32,
    pub allowed_venue_types: Vec<ArbFarmVenueType>,
    pub blocked_tokens: Vec<String>,
    pub custom_params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArbFarmRiskProfileType {
    Conservative,
    Balanced,
    Aggressive,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbFarmStrategyPerformance {
    pub total_trades: i32,
    pub successful_trades: i32,
    pub win_rate: f32,
    pub total_profit_lamports: i64,
    pub avg_profit_per_trade_lamports: i64,
    pub max_drawdown_lamports: i64,
    pub sharpe_ratio: Option<f32>,
    pub last_trade_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateArbFarmCowRequest {
    pub name: String,
    pub description: String,
    pub strategies: Vec<CreateArbFarmStrategyRequest>,
    pub venue_types: Vec<ArbFarmVenueType>,
    pub risk_profile: ArbFarmRiskProfile,
    pub price_sol: Option<f64>,
    pub creator_revenue_share_bps: Option<i32>,
    pub fork_revenue_share_bps: Option<i32>,
    pub is_public: bool,
    pub is_forkable: bool,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateArbFarmStrategyRequest {
    pub name: String,
    pub strategy_type: ArbFarmStrategyType,
    pub venue_types: Vec<ArbFarmVenueType>,
    pub execution_mode: ArbFarmExecutionMode,
    pub risk_params: ArbFarmRiskParams,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ForkArbFarmCowRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub risk_profile_overrides: Option<ArbFarmRiskProfile>,
    pub inherit_engrams: bool,
    pub engram_filters: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbFarmCowFork {
    pub id: Uuid,
    pub parent_cow_id: Uuid,
    pub forked_cow_id: Uuid,
    pub forker_wallet: String,
    pub inherited_strategies: Vec<Uuid>,
    pub inherited_engrams: Vec<String>,
    pub fork_price_paid_lamports: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbFarmRevenue {
    pub id: Uuid,
    pub cow_id: Uuid,
    pub wallet_address: String,
    pub revenue_type: ArbFarmRevenueType,
    pub amount_lamports: i64,
    pub source_trade_id: Option<Uuid>,
    pub source_fork_id: Option<Uuid>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub is_distributed: bool,
    pub distributed_at: Option<DateTime<Utc>>,
    pub tx_signature: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArbFarmRevenueType {
    TradingProfit,
    ForkFee,
    RevenueShare,
    CreatorRoyalty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbFarmEarningsSummary {
    pub wallet_address: String,
    pub total_earnings_lamports: i64,
    pub trading_profit_lamports: i64,
    pub fork_fees_lamports: i64,
    pub revenue_share_lamports: i64,
    pub creator_royalties_lamports: i64,
    pub pending_distribution_lamports: i64,
    pub cow_count: i32,
    pub fork_count: i32,
    pub period: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArbFarmCowListResponse {
    pub cows: Vec<ArbFarmCowSummary>,
    pub total_count: i64,
    pub page: i32,
    pub per_page: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbFarmCowSummary {
    pub id: Uuid,
    pub listing_id: Uuid,
    pub name: String,
    pub description: String,
    pub creator_wallet: String,
    pub strategy_count: i32,
    pub venue_types: Vec<ArbFarmVenueType>,
    pub risk_profile_type: ArbFarmRiskProfileType,
    pub fork_count: i32,
    pub total_profit_sol: f64,
    pub win_rate: f32,
    pub price_sol: Option<f64>,
    pub is_free: bool,
    pub is_forkable: bool,
    pub rating: Option<f32>,
    pub created_at: DateTime<Utc>,
}
