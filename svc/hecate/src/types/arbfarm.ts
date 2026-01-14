// ArbFarm Types - Solana MEV Agent Swarm
// Maps to backend response types from arb-farm service

// ============================================================================
// Edge (Opportunity) Types
// ============================================================================

export type EdgeStatus =
  | 'detected'
  | 'pending_approval'
  | 'executing'
  | 'executed'
  | 'expired'
  | 'failed'
  | 'rejected';

export type EdgeType = 'dex_arb' | 'curve_arb' | 'liquidation' | 'backrun' | 'jit' | 'sandwich';

export type AtomicityLevel = 'fully_atomic' | 'partially_atomic' | 'non_atomic';

export type ExecutionMode = 'autonomous' | 'agent_directed' | 'hybrid';

export type VenueType = 'dex_amm' | 'bonding_curve' | 'lending' | 'orderbook';

export interface Edge {
  id: string;
  edge_type: EdgeType;
  venue_type: VenueType;
  execution_mode: ExecutionMode;
  status: EdgeStatus;
  atomicity: AtomicityLevel;
  simulated_profit_guaranteed: boolean;
  estimated_profit_lamports: number;
  estimated_profit_sol: number;
  risk_score: number;
  route_data: RouteData;
  rejection_reason?: string;
  executed_at?: string;
  actual_profit_lamports?: number;
  actual_gas_cost_lamports?: number;
  created_at: string;
  expires_at?: string;
}

export interface RouteData {
  input_token: string;
  output_token: string;
  input_amount: number;
  expected_output: number;
  venues: string[];
  route_signature: string;
}

export interface EdgeFilter {
  status?: EdgeStatus[];
  edge_type?: EdgeType[];
  venue_type?: VenueType[];
  execution_mode?: ExecutionMode[];
  atomicity?: AtomicityLevel[];
  min_profit_lamports?: number;
  max_risk_score?: number;
  limit?: number;
  offset?: number;
}

// ============================================================================
// Trade Types
// ============================================================================

export interface Trade {
  id: string;
  edge_id: string;
  strategy_id?: string;
  tx_signature?: string;
  bundle_id?: string;
  entry_price: number;
  exit_price?: number;
  profit_lamports: number;
  profit_sol: number;
  gas_cost_lamports: number;
  slippage_bps: number;
  executed_at: string;
}

export interface TradeStats {
  total_trades: number;
  successful_trades: number;
  failed_trades: number;
  win_rate: number;
  total_profit_lamports: number;
  total_profit_sol: number;
  total_gas_cost_lamports: number;
  net_profit_lamports: number;
  net_profit_sol: number;
  avg_profit_per_trade_lamports: number;
  best_trade_lamports: number;
  worst_trade_lamports: number;
  period_start?: string;
  period_end?: string;
}

export interface DailyStats {
  date: string;
  trades: number;
  profit_lamports: number;
  profit_sol: number;
  gas_cost_lamports: number;
  win_rate: number;
}

// ============================================================================
// Scanner Types
// ============================================================================

export interface ScannerStatus {
  is_running: boolean;
  venues_active: number;
  signals_detected_24h: number;
  last_signal_at?: string;
  uptime_secs: number;
  stats: ScannerStats;
  venues: VenueStatus[];
}

export interface ScannerStats {
  total_signals: number;
  signals_last_hour: number;
  signals_last_24h: number;
  avg_signal_quality: number;
  venues_scanned: number;
}

export interface VenueStatus {
  venue_type: VenueType;
  name: string;
  is_healthy: boolean;
  last_scan_at?: string;
  signals_count: number;
  error_message?: string;
}

export interface Signal {
  id: string;
  venue_type: VenueType;
  signal_type: string;
  confidence: number;
  estimated_profit_bps: number;
  token_pair?: string;
  detected_at: string;
  metadata: Record<string, unknown>;
}

// ============================================================================
// Swarm Health Types
// ============================================================================

export type AgentHealth = 'Healthy' | 'Degraded' | 'Unhealthy' | 'Dead';

export type CircuitState = 'Closed' | 'Open' | 'HalfOpen';

export interface SwarmHealth {
  total_agents: number;
  healthy_agents: number;
  degraded_agents: number;
  unhealthy_agents: number;
  dead_agents: number;
  overall_health: AgentHealth;
  is_paused: boolean;
}

export interface AgentStatus {
  agent_type: string;
  agent_id: string;
  health: AgentHealth;
  seconds_since_heartbeat: number;
  consecutive_failures: number;
  restart_count: number;
  started_at: string;
  error_message?: string;
}

export interface CircuitBreakerStatus {
  name: string;
  state: CircuitState;
}

export interface SwarmStatus {
  health: SwarmHealth;
  agents: AgentStatus[];
  circuit_breakers: CircuitBreakerStatus[];
}

// ============================================================================
// Strategy Types
// ============================================================================

export interface Strategy {
  id: string;
  name: string;
  strategy_type: string;
  venue_types: VenueType[];
  execution_mode: ExecutionMode;
  risk_params: RiskParams;
  is_active: boolean;
  created_at: string;
  updated_at: string;
  stats?: StrategyStats;
}

export interface RiskParams {
  max_position_sol: number;
  min_profit_bps: number;
  max_slippage_bps: number;
  max_risk_score: number;
  daily_loss_limit_sol?: number;
}

export interface StrategyStats {
  total_trades: number;
  win_rate: number;
  total_profit_sol: number;
  avg_profit_bps: number;
  period_days: number;
}

// ============================================================================
// Threat Detection Types
// ============================================================================

export type ThreatCategory =
  | 'rug_pull'
  | 'honeypot'
  | 'scam_wallet'
  | 'wash_trader'
  | 'fake_token'
  | 'blacklist_function'
  | 'bundle_manipulation';

export type ThreatSeverity = 'low' | 'medium' | 'high' | 'critical';

export interface ThreatScore {
  token_mint: string;
  overall_score: number;
  factors: ThreatFactors;
  confidence: number;
  external_data: ExternalThreatData;
  created_at: string;
}

export interface ThreatFactors {
  has_mint_authority: boolean;
  has_freeze_authority: boolean;
  has_blacklist: boolean;
  upgradeable: boolean;
  top_10_concentration: number;
  creator_holdings: number;
  suspicious_holder_count: number;
  sell_pressure_score: number;
  wash_trade_likelihood: number;
  bundle_manipulation: boolean;
}

export interface ExternalThreatData {
  rugcheck_score?: number;
  goplus_honeypot?: boolean;
  birdeye_holder_count?: number;
}

export interface ThreatAlert {
  id: string;
  alert_type: string;
  severity: ThreatSeverity;
  entity_type: string;
  address: string;
  details: Record<string, unknown>;
  action_taken: string;
  created_at: string;
}

export interface BlockedEntity {
  id: string;
  entity_type: string;
  address: string;
  threat_category: ThreatCategory;
  threat_score: number;
  reason: string;
  evidence_url?: string;
  reported_by: string;
  is_active: boolean;
  created_at: string;
}

// ============================================================================
// KOL Tracking Types
// ============================================================================

export interface KOL {
  id: string;
  entity_type: string;
  identifier: string;
  display_name?: string;
  linked_wallet?: string;
  trust_score: number;
  total_trades_tracked: number;
  profitable_trades: number;
  avg_profit_percent: number;
  max_drawdown: number;
  copy_trading_enabled: boolean;
  copy_config: CopyConfig;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface CopyConfig {
  max_position_sol: number;
  delay_ms: number;
  min_trust_score: number;
  copy_percentage: number;
  token_whitelist?: string[];
  token_blacklist?: string[];
}

export interface KOLTrade {
  id: string;
  entity_id: string;
  tx_signature: string;
  trade_type: 'buy' | 'sell';
  token_mint: string;
  amount_sol: number;
  price_at_trade: number;
  detected_at: string;
}

export interface CopyTrade {
  id: string;
  entity_id: string;
  kol_trade_id: string;
  our_tx_signature?: string;
  copy_amount_sol: number;
  delay_ms: number;
  profit_loss_lamports: number;
  status: 'pending' | 'executing' | 'executed' | 'failed' | 'skipped';
  executed_at?: string;
  created_at: string;
}

export interface TrustBreakdown {
  kol_id: string;
  total_score: number;
  win_rate_component: number;
  profit_component: number;
  consistency_component: number;
  drawdown_penalty: number;
  sample_size_factor: number;
}

// ============================================================================
// Bonding Curve Types
// ============================================================================

export type CurveVenue = 'pump_fun' | 'moonshot';

export interface CurveToken {
  mint: string;
  name: string;
  symbol: string;
  venue: CurveVenue;
  creator: string;
  graduation_progress: number;
  current_price_sol: number;
  market_cap_sol: number;
  volume_24h_sol: number;
  holder_count: number;
  is_graduated: boolean;
  raydium_pool?: string;
  created_at: string;
  updated_at: string;
}

export interface GraduationCandidate {
  token: CurveToken;
  graduation_eta_minutes?: number;
  momentum_score: number;
  risk_score: number;
}

export interface CrossVenueArbitrage {
  token_mint: string;
  curve_price_sol: number;
  dex_price_sol: number;
  price_diff_percent: number;
  estimated_profit_sol: number;
  curve_venue: CurveVenue;
  dex_venue: string;
}

export interface CurveQuote {
  token_mint: string;
  venue: CurveVenue;
  direction: 'buy' | 'sell';
  input_amount: number;
  output_amount: number;
  price_impact_percent: number;
  fee_amount: number;
}

// ============================================================================
// Engram Types (Pattern Learning)
// ============================================================================

export type EngramType =
  | 'edge_pattern'
  | 'avoidance'
  | 'strategy'
  | 'threat_intel'
  | 'consensus_outcome'
  | 'trade_result'
  | 'market_condition';

export interface ArbEngram {
  id: string;
  key: string;
  engram_type: EngramType;
  content: Record<string, unknown>;
  confidence: number;
  tags: string[];
  source_edge_id?: string;
  expires_at?: string;
  created_at: string;
  updated_at: string;
}

export interface PatternMatch {
  engram: ArbEngram;
  match_score: number;
  matching_factors: string[];
}

export interface HarvesterStats {
  total_engrams: number;
  patterns_stored: number;
  avoidances_stored: number;
  strategies_stored: number;
  threat_intel_stored: number;
  avg_confidence: number;
}

// ============================================================================
// Event Types (for SSE)
// ============================================================================

export interface ArbEvent {
  id: string;
  event_type: string;
  source: EventSource;
  topic: string;
  payload: Record<string, unknown>;
  timestamp: string;
  correlation_id?: string;
}

export interface EventSource {
  type: 'Agent' | 'Tool' | 'External' | 'System';
  id: string;
}

// ============================================================================
// Dashboard Summary Types
// ============================================================================

export interface DashboardSummary {
  // P&L Metrics
  total_profit_sol: number;
  today_profit_sol: number;
  week_profit_sol: number;
  win_rate: number;

  // Activity Metrics
  active_opportunities: number;
  pending_approvals: number;
  executed_today: number;

  // Swarm Health
  swarm_health: SwarmHealth;

  // Top Opportunities
  top_opportunities: Edge[];

  // Recent Trades
  recent_trades: Trade[];

  // Alerts
  recent_alerts: ThreatAlert[];
}

// ============================================================================
// Color Constants
// ============================================================================

export const EDGE_STATUS_COLORS: Record<EdgeStatus, string> = {
  detected: '#3b82f6', // blue
  pending_approval: '#f59e0b', // amber
  executing: '#8b5cf6', // purple
  executed: '#22c55e', // green
  expired: '#6b7280', // gray
  failed: '#ef4444', // red
  rejected: '#f97316', // orange
};

export const AGENT_HEALTH_COLORS: Record<AgentHealth, string> = {
  Healthy: '#22c55e', // green
  Degraded: '#f59e0b', // amber
  Unhealthy: '#f97316', // orange
  Dead: '#ef4444', // red
};

export const THREAT_SEVERITY_COLORS: Record<ThreatSeverity, string> = {
  low: '#6b7280', // gray
  medium: '#f59e0b', // amber
  high: '#f97316', // orange
  critical: '#ef4444', // red
};

export const VENUE_TYPE_ICONS: Record<VenueType, string> = {
  dex_amm: '🔄',
  bonding_curve: '📈',
  lending: '🏦',
  orderbook: '📊',
};

// ============================================================================
// COW (Constellation of Work) Types
// ============================================================================

export type ArbFarmStrategyType =
  | 'dex_arb'
  | 'curve_arb'
  | 'liquidation'
  | 'jit_liquidity'
  | 'backrun'
  | 'sandwich'
  | 'copy_trade'
  | 'custom';

export type ArbFarmRiskProfileType = 'conservative' | 'balanced' | 'aggressive' | 'custom';

export type ArbFarmRevenueType = 'trading_profit' | 'fork_fee' | 'revenue_share' | 'creator_royalty';

export interface ArbFarmCow {
  id: string;
  listing_id: string;
  creator_wallet: string;
  name: string;
  description: string;
  strategies: ArbFarmCowStrategy[];
  venue_types: VenueType[];
  risk_profile: ArbFarmRiskProfile;
  parent_cow_id?: string;
  fork_count: number;
  total_profit_generated_lamports: number;
  total_trades: number;
  win_rate: number;
  creator_revenue_share_bps: number;
  fork_revenue_share_bps: number;
  is_public: boolean;
  is_forkable: boolean;
  inherited_engrams: string[];
  created_at: string;
  updated_at: string;
}

export interface ArbFarmCowStrategy {
  id: string;
  name: string;
  strategy_type: ArbFarmStrategyType;
  venue_types: VenueType[];
  execution_mode: ExecutionMode;
  risk_params: CowRiskParams;
  is_active: boolean;
  performance?: ArbFarmStrategyPerformance;
}

export interface CowRiskParams {
  max_position_sol: number;
  min_profit_bps: number;
  max_slippage_bps: number;
  max_risk_score: number;
  daily_loss_limit_sol?: number;
  require_consensus: boolean;
  min_consensus_agreement?: number;
}

export interface ArbFarmRiskProfile {
  profile_type: ArbFarmRiskProfileType;
  max_position_sol: number;
  daily_loss_limit_sol: number;
  max_concurrent_positions: number;
  allowed_venue_types: VenueType[];
  blocked_tokens: string[];
  custom_params?: Record<string, unknown>;
}

export interface ArbFarmStrategyPerformance {
  total_trades: number;
  successful_trades: number;
  win_rate: number;
  total_profit_lamports: number;
  avg_profit_per_trade_lamports: number;
  max_drawdown_lamports: number;
  sharpe_ratio?: number;
  last_trade_at?: string;
}

export interface CreateArbFarmCowRequest {
  name: string;
  description: string;
  strategies: CreateArbFarmStrategyRequest[];
  venue_types: VenueType[];
  risk_profile: ArbFarmRiskProfile;
  price_sol?: number;
  creator_revenue_share_bps?: number;
  fork_revenue_share_bps?: number;
  is_public: boolean;
  is_forkable: boolean;
  tags: string[];
}

export interface CreateArbFarmStrategyRequest {
  name: string;
  strategy_type: ArbFarmStrategyType;
  venue_types: VenueType[];
  execution_mode: ExecutionMode;
  risk_params: CowRiskParams;
  is_active: boolean;
}

export interface ForkArbFarmCowRequest {
  name?: string;
  description?: string;
  risk_profile_overrides?: ArbFarmRiskProfile;
  inherit_engrams: boolean;
  engram_filters?: string[];
}

export interface ArbFarmCowFork {
  id: string;
  parent_cow_id: string;
  forked_cow_id: string;
  forker_wallet: string;
  inherited_strategies: string[];
  inherited_engrams: string[];
  fork_price_paid_lamports: number;
  created_at: string;
}

export interface ArbFarmRevenue {
  id: string;
  cow_id: string;
  wallet_address: string;
  revenue_type: ArbFarmRevenueType;
  amount_lamports: number;
  source_trade_id?: string;
  source_fork_id?: string;
  period_start: string;
  period_end: string;
  is_distributed: boolean;
  distributed_at?: string;
  tx_signature?: string;
  created_at: string;
}

export interface ArbFarmEarningsSummary {
  wallet_address: string;
  total_earnings_lamports: number;
  trading_profit_lamports: number;
  fork_fees_lamports: number;
  revenue_share_lamports: number;
  creator_royalties_lamports: number;
  pending_distribution_lamports: number;
  cow_count: number;
  fork_count: number;
  period: string;
}

export interface ArbFarmCowSummary {
  id: string;
  listing_id: string;
  name: string;
  description: string;
  creator_wallet: string;
  strategy_count: number;
  venue_types: VenueType[];
  risk_profile_type: ArbFarmRiskProfileType;
  fork_count: number;
  total_profit_sol: number;
  win_rate: number;
  price_sol?: number;
  is_free: boolean;
  is_forkable: boolean;
  rating?: number;
  created_at: string;
}

export interface ArbFarmCowStats {
  total_cows: number;
  total_forks: number;
  total_revenue_lamports: number;
  total_trades: number;
  avg_win_rate: number;
}

export const STRATEGY_TYPE_LABELS: Record<ArbFarmStrategyType, string> = {
  dex_arb: 'DEX Arbitrage',
  curve_arb: 'Curve Arbitrage',
  liquidation: 'Liquidation',
  jit_liquidity: 'JIT Liquidity',
  backrun: 'Backrun',
  sandwich: 'Sandwich',
  copy_trade: 'Copy Trade',
  custom: 'Custom',
};

export const RISK_PROFILE_LABELS: Record<ArbFarmRiskProfileType, string> = {
  conservative: 'Conservative',
  balanced: 'Balanced',
  aggressive: 'Aggressive',
  custom: 'Custom',
};

export const RISK_PROFILE_COLORS: Record<ArbFarmRiskProfileType, string> = {
  conservative: '#22c55e',
  balanced: '#3b82f6',
  aggressive: '#f59e0b',
  custom: '#8b5cf6',
};
