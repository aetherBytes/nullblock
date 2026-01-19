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
// Signal Types
// ============================================================================

export type SignalType =
  | 'price_discrepancy'
  | 'volume_spike'
  | 'liquidity_change'
  | 'new_token'
  | 'curve_graduation'
  | 'large_order'
  | 'liquidation'
  | 'pool_imbalance'
  | 'dex_arb'
  | 'jit_liquidity'
  | 'backrun';

export type Significance = 'low' | 'medium' | 'high' | 'critical';

export interface Signal {
  id: string;
  signal_type: SignalType;
  venue_id: string;
  venue_type: VenueType;
  token_mint?: string;
  pool_address?: string;
  estimated_profit_bps: number;
  confidence: number;
  significance: Significance;
  metadata: Record<string, any>;
  detected_at: string;
  expires_at: string;
}

export interface SignalFilter {
  signal_type?: SignalType[];
  venue_type?: VenueType[];
  min_profit_bps?: number;
  min_confidence?: number;
  significance?: Significance[];
  limit?: number;
  offset?: number;
}

export interface SignalListResponse {
  signals: Signal[];
  total: number;
  limit: number;
  offset: number;
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
  daily_loss_limit_sol: number;
  min_profit_bps: number;
  max_slippage_bps: number;
  max_risk_score: number;
  require_simulation: boolean;
  auto_execute_atomic: boolean;
}

export interface CreateStrategyRequest {
  wallet_address: string;
  name: string;
  strategy_type: string;
  venue_types: VenueType[];
  execution_mode: ExecutionMode;
  risk_params: RiskParams;
}

export interface UpdateStrategyRequest {
  name?: string;
  venue_types?: VenueType[];
  execution_mode?: ExecutionMode;
  risk_params?: Partial<RiskParams>;
  is_active?: boolean;
}

export const DEFAULT_RISK_PARAMS: RiskParams = {
  max_position_sol: 1.0,
  daily_loss_limit_sol: 0.5,
  min_profit_bps: 50,
  max_slippage_bps: 100,
  max_risk_score: 50,
  require_simulation: true,
  auto_execute_atomic: true,
};

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
// KOL Discovery Types
// ============================================================================

export interface DiscoveredKol {
  wallet_address: string;
  display_name?: string;
  total_trades: number;
  winning_trades: number;
  win_rate: number;
  avg_profit_pct: number;
  total_volume_usd: number;
  consecutive_wins: number;
  trust_score: number;
  discovered_at: string;
  source: string;
}

export interface KolDiscoveryStatus {
  is_running: boolean;
  total_wallets_analyzed: number;
  total_kols_discovered: number;
  last_scan_at?: string;
  scan_interval_ms: number;
}

export interface DiscoveredKolsResponse {
  discovered: DiscoveredKol[];
  total: number;
}

export interface KolScanResult {
  discovered: DiscoveredKol[];
  count: number;
  message: string;
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
// Curve Position & Strategy Types
// ============================================================================

export type CurvePositionStatus = 'open' | 'pending_exit' | 'closed';

export interface CurveExitConfig {
  sell_on_graduation: boolean;
  graduation_sell_delay_ms: number;
  stop_loss_percent?: number;
  take_profit_percent?: number;
}

export interface CurvePosition {
  id: string;
  token_mint: string;
  token_symbol: string;
  venue: CurveVenue;
  entry_sol: number;
  entry_tokens: number;
  entry_progress: number;
  current_progress: number;
  unrealized_pnl_percent: number;
  status: CurvePositionStatus;
  exit_config: CurveExitConfig;
  created_at: string;
  updated_at: string;
}

export type CurveStrategyMode = 'graduation_arbitrage' | 'fast_snipe' | 'scalp_on_curve';

export interface CurveStrategyParams {
  mode: CurveStrategyMode;
  min_graduation_progress: number;
  max_graduation_progress: number;
  min_volume_24h_sol: number;
  max_holder_concentration: number;
  min_holder_count: number;
  entry_sol_amount: number;
  exit_on_graduation: boolean;
  graduation_sell_delay_ms: number;
  venue_filter?: CurveVenue[];
  min_score?: number;
}

export interface CurveStrategyStats {
  strategy_id: string;
  total_entries: number;
  successful_exits: number;
  graduations_caught: number;
  total_pnl_sol: number;
  win_rate: number;
  avg_hold_time_seconds: number;
  best_trade_sol: number;
  worst_trade_sol: number;
}

export type TrackedState = 'monitoring' | 'near_graduation' | 'graduating' | 'graduated' | 'failed';

export interface TrackedToken {
  mint: string;
  name: string;
  symbol: string;
  venue: string;
  strategy_id: string;
  state: TrackedState;
  progress: number;
  progress_velocity: number;
  started_tracking_at: string;
  last_checked_at: string;
  entry_price_sol?: number;
  entry_tokens?: number;
  raydium_pool?: string;
}

export type SnipeStatus = 'waiting' | 'selling' | 'sold' | 'failed';

export interface SnipePosition {
  mint: string;
  symbol: string;
  strategy_id: string;
  entry_tokens: number;
  entry_price_sol: number;
  entry_time: string;
  status: SnipeStatus;
  sell_attempts: number;
  last_sell_attempt?: string;
  sell_tx_signature?: string;
  exit_sol?: number;
  pnl_sol?: number;
}

export interface SniperStats {
  positions_waiting: number;
  positions_sold: number;
  positions_failed: number;
  total_pnl_sol: number;
  is_running: boolean;
}

export interface GraduationTrackerStats {
  tokens_tracked: number;
  tokens_near_graduation: number;
  tokens_graduated: number;
  tokens_failed: number;
  total_checks: number;
  is_running: boolean;
}

export interface OnChainCurveState {
  mint: string;
  venue: CurveVenue;
  virtual_sol_reserves: number;
  virtual_token_reserves: number;
  real_sol_reserves: number;
  real_token_reserves: number;
  token_total_supply: number;
  complete: boolean;
  graduation_progress: number;
  current_price_sol: number;
  market_cap_sol: number;
}

export interface CurveBuyRequest {
  sol_amount: number;
  slippage_bps?: number;
  simulate_only?: boolean;
}

export interface CurveSellRequest {
  token_amount: number;
  slippage_bps?: number;
  simulate_only?: boolean;
}

export interface CurveBuildResult {
  transaction_base64: string;
  expected_tokens_out?: number;
  expected_sol_out?: number;
  price_impact_percent: number;
  fee_lamports: number;
}

export interface SimulatedTrade {
  input_amount: number;
  output_amount: number;
  price_impact_percent: number;
  fee_amount: number;
  effective_price: number;
}

export const CURVE_STRATEGY_MODE_LABELS: Record<CurveStrategyMode, string> = {
  graduation_arbitrage: 'Graduation Arbitrage',
  fast_snipe: 'Fast Snipe',
  scalp_on_curve: 'Scalp on Curve',
};

export const TRACKED_STATE_COLORS: Record<TrackedState, string> = {
  monitoring: '#3b82f6', // blue
  near_graduation: '#f59e0b', // amber
  graduating: '#8b5cf6', // purple
  graduated: '#22c55e', // green
  failed: '#ef4444', // red
};

export const SNIPE_STATUS_COLORS: Record<SnipeStatus, string> = {
  waiting: '#f59e0b', // amber
  selling: '#8b5cf6', // purple
  sold: '#22c55e', // green
  failed: '#ef4444', // red
};

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

// ============================================================================
// Wallet Types
// ============================================================================

export type DelegationStatus = 'not_configured' | 'pending' | 'active' | 'revoked' | 'error';

export interface WalletStatus {
  is_connected: boolean;
  wallet_address?: string;
  turnkey_wallet_id?: string;
  balance_lamports?: number;
  daily_usage: DailyUsage;
  policy: ArbFarmPolicy;
  delegation_status: DelegationStatus;
}

export interface DailyUsage {
  date: string;
  total_volume_lamports: number;
  transaction_count: number;
}

export interface ArbFarmPolicy {
  max_transaction_amount_lamports: number;
  daily_volume_limit_lamports: number;
  max_transactions_per_day: number;
  allowed_programs: string[];
  require_simulation: boolean;
  min_profit_threshold_lamports: number;
}

export interface WalletSetupRequest {
  user_wallet_address: string;
  wallet_name?: string;
}

export interface WalletSetupResponse {
  success: boolean;
  wallet_status?: WalletStatus;
  error?: string;
}

export interface UpdatePolicyRequest {
  max_transaction_amount_sol?: number;
  daily_volume_limit_sol?: number;
  max_transactions_per_day?: number;
  require_simulation?: boolean;
  min_profit_threshold_sol?: number;
}

export interface WalletBalanceResponse {
  balance_lamports: number;
  balance_sol: number;
}

export interface DailyUsageResponse {
  date: string;
  volume_lamports: number;
  volume_sol: number;
  transaction_count: number;
  remaining_volume_sol: number;
  remaining_transactions: number;
}

// ============================================================================
// Settings Types
// ============================================================================

export interface RiskConfig {
  max_position_sol: number;
  daily_loss_limit_sol: number;
  max_drawdown_percent: number;
  max_concurrent_positions: number;
  max_position_per_token_sol: number;
  cooldown_after_loss_ms: number;
  volatility_scaling_enabled: boolean;
  auto_pause_on_drawdown: boolean;
}

export interface RiskPreset {
  name: string;
  description: string;
  config: RiskConfig;
}

export interface RiskSettingsResponse {
  config: RiskConfig;
  presets: RiskPreset[];
}

export interface VenueConfig {
  name: string;
  venue_type: string;
  enabled: boolean;
  api_url: string;
  has_api_key: boolean;
}

export interface ApiKeyStatus {
  name: string;
  configured: boolean;
  required: boolean;
}

export interface AllSettingsResponse {
  risk: RiskConfig;
  risk_presets: RiskPreset[];
  venues: VenueConfig[];
  api_keys: ApiKeyStatus[];
  wallet_connected: boolean;
}

export const DELEGATION_STATUS_COLORS: Record<DelegationStatus, string> = {
  not_configured: '#6b7280',
  pending: '#f59e0b',
  active: '#22c55e',
  revoked: '#ef4444',
  error: '#ef4444',
};

// ============================================================================
// Capital Management Types
// ============================================================================

export interface StrategyAllocationInfo {
  strategy_id: string;
  max_allocation_percent: number;
  max_allocation_sol: number;
  current_reserved_sol: number;
  available_sol: number;
  active_positions: number;
  max_positions: number;
}

export interface CapitalReservationInfo {
  position_id: string;
  strategy_id: string;
  amount_sol: number;
  created_at: string;
}

export interface CapitalUsageResponse {
  total_balance_sol: number;
  global_reserved_sol: number;
  available_sol: number;
  strategy_allocations: StrategyAllocationInfo[];
  active_reservations: CapitalReservationInfo[];
}

export interface SyncCapitalResponse {
  success: boolean;
  balance_sol: number;
  available_sol: number;
  reserved_sol: number;
  error?: string;
}

export const DELEGATION_STATUS_LABELS: Record<DelegationStatus, string> = {
  not_configured: 'Not Connected',
  pending: 'Pending',
  active: 'Active',
  revoked: 'Revoked',
  error: 'Error',
};

// ============================================================================
// Helius Integration Types
// ============================================================================

export interface HeliusStatus {
  connected: boolean;
  api_key_configured: boolean;
  laserstream_enabled: boolean;
  sender_enabled: boolean;
  rpc_url: string;
  sender_url: string;
}

export interface LaserStreamStatus {
  connected: boolean;
  subscriptions: LaserStreamSubscription[];
  avg_latency_ms: number;
  events_per_second: number;
}

export interface LaserStreamSubscription {
  id: string;
  subscription_type: string;
  address?: string;
  events_received: number;
}

export interface PriorityFees {
  min: number;
  low: number;
  medium: number;
  high: number;
  very_high: number;
  unsafe_max: number;
  recommended: number;
}

export interface SenderStats {
  total_sent: number;
  total_confirmed: number;
  total_failed: number;
  success_rate: number;
  avg_landing_ms: number;
}

export interface TokenMetadata {
  mint: string;
  name: string;
  symbol: string;
  decimals: number;
  supply: number;
  creators: TokenCreator[];
  collection?: TokenCollection;
  attributes: Record<string, string>;
  image_uri?: string;
}

export interface TokenCreator {
  address: string;
  verified: boolean;
  share: number;
}

export interface TokenCollection {
  address: string;
  name?: string;
  verified: boolean;
}

export interface HeliusConfig {
  laserstream_enabled: boolean;
  default_priority_level: string;
  use_helius_sender: boolean;
}

export type PriorityLevel = 'min' | 'low' | 'medium' | 'high' | 'very_high' | 'unsafe_max';

export const PRIORITY_LEVEL_LABELS: Record<PriorityLevel, string> = {
  min: 'Min',
  low: 'Low',
  medium: 'Medium',
  high: 'High',
  very_high: 'Very High',
  unsafe_max: 'Unsafe Max',
};

export const PRIORITY_LEVEL_COLORS: Record<PriorityLevel, string> = {
  min: '#6b7280',
  low: '#3b82f6',
  medium: '#f59e0b',
  high: '#ef4444',
  very_high: '#dc2626',
  unsafe_max: '#991b1b',
};

// ============================================================================
// Research & Discovery Types
// ============================================================================

export type StrategyConfidence = 'High' | 'Medium' | 'Low';

export type ExtractedStrategyType =
  | 'DexArbitrage'
  | 'BondingCurve'
  | 'Momentum'
  | 'MeanReversion'
  | 'Breakout'
  | 'Scalping'
  | 'Swing'
  | 'CopyTrade'
  | 'Liquidation'
  | 'Unknown';

export type ConditionType =
  | 'PriceAbove'
  | 'PriceBelow'
  | 'VolumeSpike'
  | 'TimeWindow'
  | 'PercentageGain'
  | 'PercentageLoss'
  | 'CurveProgress'
  | 'HolderConcentration'
  | 'MarketCapThreshold'
  | 'Custom';

export interface Condition {
  description: string;
  condition_type: ConditionType;
  parameters: Record<string, unknown>;
}

export interface ExtractedRiskParams {
  max_position_sol?: number;
  stop_loss_percent?: number;
  take_profit_percent?: number;
  max_slippage_bps?: number;
  time_limit_minutes?: number;
}

export interface ExtractedStrategy {
  id: string;
  source_id: string;
  source_url: string;
  name: string;
  description: string;
  strategy_type: ExtractedStrategyType;
  entry_conditions: Condition[];
  exit_conditions: Condition[];
  risk_params: ExtractedRiskParams;
  tokens_mentioned: string[];
  confidence: StrategyConfidence;
  confidence_score: number;
  raw_extraction: string;
  extracted_at: string;
}

export interface IngestResult {
  id: string;
  url: string;
  content_type: string;
  title?: string;
  author?: string;
  published_at?: string;
  cleaned_content: string;
  extracted_tokens: string[];
  extracted_numbers: ExtractedNumber[];
  ingested_at: string;
}

export interface ExtractedNumber {
  value: number;
  context: string;
  original: string;
}

export interface IngestUrlRequest {
  url: string;
  context?: string;
  extract_strategy?: boolean;
}

export interface IngestUrlResponse {
  ingest_result: IngestResult;
  extracted_strategy?: ExtractedStrategy;
}

export interface DiscoveryListResponse {
  discoveries: ExtractedStrategy[];
  total: number;
}

export type SourceType = 'Twitter' | 'Discord' | 'Telegram' | 'Reddit' | 'Website';
export type TrackType = 'Alpha' | 'Threat' | 'General';

export interface MonitoredSource {
  id: string;
  source_type: SourceType;
  handle: string;
  display_name?: string;
  track_type: TrackType;
  keywords: string[];
  is_active: boolean;
  created_at: string;
  last_checked_at?: string;
}

export interface MonitorStats {
  total_sources: number;
  active_sources: number;
  total_alerts: number;
  alerts_last_24h: number;
  sources_by_type: Record<string, number>;
}

export interface SocialAlert {
  id: string;
  source_id: string;
  alert_type: string;
  content: string;
  tokens_mentioned: string[];
  sentiment: string;
  created_at: string;
}

export const STRATEGY_CONFIDENCE_COLORS: Record<StrategyConfidence, string> = {
  High: '#22c55e',
  Medium: '#f59e0b',
  Low: '#ef4444',
};

export const EXTRACTED_STRATEGY_TYPE_LABELS: Record<ExtractedStrategyType, string> = {
  DexArbitrage: 'DEX Arbitrage',
  BondingCurve: 'Bonding Curve',
  Momentum: 'Momentum',
  MeanReversion: 'Mean Reversion',
  Breakout: 'Breakout',
  Scalping: 'Scalping',
  Swing: 'Swing Trade',
  CopyTrade: 'Copy Trade',
  Liquidation: 'Liquidation',
  Unknown: 'Unknown',
};

// ============================================================================
// Curve Metrics & Scoring Types
// ============================================================================

export interface DetailedCurveMetrics {
  mint: string;
  venue: string;
  volume_1h: number;
  volume_24h: number;
  volume_velocity: number;
  volume_acceleration: number;
  trade_count_1h: number;
  trade_count_24h: number;
  unique_buyers_1h: number;
  unique_buyers_24h: number;
  holder_count: number;
  holder_growth_1h: number;
  holder_growth_24h: number;
  top_10_concentration: number;
  top_20_concentration: number;
  creator_holdings_percent: number;
  price_momentum_1h: number;
  price_momentum_24h: number;
  buy_sell_ratio_1h: number;
  avg_trade_size_sol: number;
  graduation_progress: number;
  market_cap_sol: number;
  liquidity_depth_sol: number;
  holder_quality_score: number;
  activity_score: number;
  momentum_score: number;
  last_updated: string;
}

export interface HolderInfo {
  address: string;
  balance: number;
  balance_percent: number;
  is_creator: boolean;
  is_suspicious: boolean;
}

export interface HolderAnalysis {
  mint: string;
  total_holders: number;
  total_supply: number;
  circulating_supply: number;
  top_10_holders: HolderInfo[];
  top_10_concentration: number;
  top_20_concentration: number;
  top_50_concentration: number;
  creator_address: string | null;
  creator_holdings_percent: number;
  gini_coefficient: number;
  unique_wallets_24h: number;
  new_holders_24h: number;
  wash_trade_likelihood: number;
  is_healthy: boolean;
  health_score: number;
  analyzed_at: string;
}

export type OpportunityRecommendation = 'strong_buy' | 'buy' | 'hold' | 'avoid';

export interface OpportunityScore {
  mint: string;
  venue: string;
  overall_score: number;
  graduation_factor: number;
  volume_factor: number;
  holder_factor: number;
  momentum_factor: number;
  risk_penalty: number;
  recommendation: OpportunityRecommendation;
  is_actionable: boolean;
  risk_warnings: string[];
  positive_signals: string[];
}

export interface ScoringWeights {
  graduation: number;
  volume: number;
  holders: number;
  momentum: number;
  risk: number;
}

export interface ScoringThresholds {
  min_graduation_progress: number;
  max_graduation_progress: number;
  min_volume_1h_sol: number;
  min_holder_count: number;
  max_top_10_concentration: number;
  max_creator_holdings: number;
  max_wash_trade_likelihood: number;
  min_unique_buyers_1h: number;
}

export interface ScoringConfig {
  weights: ScoringWeights;
  thresholds: ScoringThresholds;
}

export interface TopOpportunity {
  mint: string;
  venue: string;
  name: string;
  symbol: string;
  score: OpportunityScore;
  metrics: DetailedCurveMetrics;
}

export interface TopOpportunitiesResponse {
  opportunities: TopOpportunity[];
  total: number;
  scoring_weights: ScoringWeights;
}

export const RECOMMENDATION_COLORS: Record<OpportunityRecommendation, string> = {
  strong_buy: '#22c55e',
  buy: '#3b82f6',
  hold: '#f59e0b',
  avoid: '#ef4444',
};

export const RECOMMENDATION_LABELS: Record<OpportunityRecommendation, string> = {
  strong_buy: 'Strong Buy',
  buy: 'Buy',
  hold: 'Hold',
  avoid: 'Avoid',
};

// ============================================================================
// P&L Summary Types
// ============================================================================

export interface BestWorstTrade {
  symbol: string;
  pnl: number;
}

export interface RecentTradeInfo {
  id: string;
  symbol: string;
  pnl: number;
  pnl_percent: number;
  exit_type: string;
  time_ago: string;
}

export interface ActiveStrategy {
  id: string;
  name: string;
  position_count: number;
}

export interface PnLSummary {
  today_sol: number;
  week_sol: number;
  total_sol: number;
  wins: number;
  losses: number;
  win_rate: number;
  avg_hold_minutes: number;
  total_trades: number;
  active_positions: number;
  take_profits: number;
  take_profit_pnl: number;
  stop_losses: number;
  stop_loss_pnl: number;
  trailing_stops: number;
  trailing_stop_pnl: number;
  manual_exits: number;
  manual_pnl: number;
  best_trade?: BestWorstTrade;
  worst_trade?: BestWorstTrade;
  recent_trades: RecentTradeInfo[];
  active_strategies: ActiveStrategy[];
}

export interface ExitConfig {
  stop_loss_percent: number;
  take_profit_percent: number;
  trailing_stop_percent?: number;
  time_limit_minutes?: number;
}

export interface OpenPosition {
  id: string;
  edge_id?: string;
  token_mint: string;
  token_symbol?: string;
  venue: string;
  strategy_id?: string;
  entry_price: number;
  entry_amount_sol: number;
  entry_tx_signature?: string;
  current_price?: number;
  unrealized_pnl?: number;
  unrealized_pnl_percent?: number;
  exit_config: ExitConfig;
  status: string;
  opened_at: string;
}

export interface PositionStats {
  total_positions_opened: number;
  total_positions_closed: number;
  active_positions: number;
  total_realized_pnl: number;
  total_unrealized_pnl: number;
  stop_losses_triggered: number;
  take_profits_triggered: number;
  time_exits_triggered: number;
}

export interface PositionsResponse {
  positions: OpenPosition[];
  stats: PositionStats;
}

// ============================================================================
// Learning Insights Types
// ============================================================================

export interface LearningInsight {
  text: string;
  confidence: number;
  source: string;
  insight_type: string;
}

export interface LearningInsightsResponse {
  insights: LearningInsight[];
  total: number;
}

// ============================================================================
// Risk Level Types
// ============================================================================

export type RiskLevel = 'low' | 'medium' | 'high';

export interface RiskLevelParams {
  level: string;
  max_position_sol: number;
  max_concurrent_positions: number;
  max_liquidity_contribution_pct: number;
  stop_loss_pct: number;
  take_profit_pct: number;
  daily_loss_limit_sol: number;
}

export interface SetRiskLevelResponse {
  success: boolean;
  message: string;
  params: RiskLevelParams;
}

// ============================================================================
// View Types
// ============================================================================

export type ArbFarmView =
  | 'home'
  | 'curve-bonding'
  | 'dex-arb'
  | 'liquidations'
  | 'kol-tracker'
  | 'settings';

export const WIP_TABS: ArbFarmView[] = ['dex-arb', 'liquidations', 'kol-tracker'];

export const TAB_LABELS: Record<ArbFarmView, string> = {
  home: 'Home',
  'curve-bonding': 'Curve Bonding',
  'dex-arb': 'DEX Arb',
  liquidations: 'Liquidations',
  'kol-tracker': 'KOL Tracker',
  settings: 'Settings',
};

export const TAB_ICONS: Record<ArbFarmView, string> = {
  home: '🏠',
  'curve-bonding': '📈',
  'dex-arb': '🔄',
  liquidations: '⚡',
  'kol-tracker': '👥',
  settings: '⚙️',
};

// ============================================================================
// Position Exposure & Monitor Types
// ============================================================================

export interface PositionExposure {
  sol_exposure: number;
  usdc_exposure: number;
  usdt_exposure: number;
  total_exposure_sol: number;
}

export interface PositionHistoryItem {
  id: string;
  token_mint: string;
  token_symbol?: string;
  venue: string;
  entry_price: number;
  exit_price: number;
  entry_amount_sol: number;
  realized_pnl: number;
  realized_pnl_percent: number;
  exit_type: string;
  hold_duration_minutes: number;
  opened_at: string;
  closed_at: string;
}

export interface PositionHistoryResponse {
  positions: PositionHistoryItem[];
  total: number;
  page: number;
  page_size: number;
}

export interface MonitorStatus {
  monitoring_active: boolean;
  price_check_interval_secs: number;
  exit_slippage_bps: number;
  active_positions: number;
  pending_exit_signals: number;
}

export interface WalletTokenHolding {
  mint: string;
  balance: number;
  decimals: number;
  symbol?: string;
}

export interface ReconcileResult {
  tracked_positions: number;
  discovered_tokens: WalletTokenHolding[];
  orphaned_positions: string[];
  message: string;
}

// ============================================================================
// Threat Management Types
// ============================================================================

export interface WhitelistedEntity {
  id: string;
  entity_type: string;
  address: string;
  reason: string;
  added_by: string;
  added_at: string;
}

export interface WatchedEntity {
  id: string;
  entity_type: string;
  address: string;
  reason?: string;
  added_at: string;
  last_score?: number;
  last_checked_at?: string;
}

export interface ThreatHistoryItem {
  id: string;
  mint: string;
  score: number;
  factors: ThreatFactors;
  checked_at: string;
}

export interface ThreatStats {
  total_checks: number;
  checks_last_24h: number;
  blocked_count: number;
  whitelisted_count: number;
  watched_count: number;
  avg_score: number;
  high_risk_detected: number;
}
