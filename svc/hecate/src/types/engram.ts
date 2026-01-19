export type EngramType =
  | 'persona'
  | 'preference'
  | 'strategy'
  | 'knowledge'
  | 'compliance';

export interface Engram {
  id: string;
  wallet_address: string;
  engram_type: EngramType;
  key: string;
  content: string;
  metadata?: Record<string, unknown>;
  tags: string[];
  is_public: boolean;
  created_at: string;
  updated_at: string;
  summary?: string;
  version?: number;
  parent_id?: string;
  lineage_root_id?: string;
  is_mintable?: boolean;
  nft_token_id?: string;
  price_mon?: string;
  royalty_percent?: number;
  priority?: number;
  ttl_seconds?: number;
  created_by?: string;
  accessed_at?: string;
}

export interface EngramListResponse {
  engrams: Engram[];
  total: number;
}

export interface EngramSearchRequest {
  wallet_address?: string;
  engram_type?: EngramType;
  query?: string;
  tags?: string[];
  limit?: number;
  offset?: number;
}

export interface CreateEngramRequest {
  wallet_address: string;
  engram_type: EngramType;
  key: string;
  content: string;
  metadata?: Record<string, unknown>;
  tags?: string[];
  is_public?: boolean;
}

export type TransactionAction = 'buy' | 'sell';

export interface TransactionMetadata {
  graduation_progress?: number;
  holder_count?: number;
  volume_24h_sol?: number;
  market_cap_sol?: number;
  bonding_curve_percent?: number;
}

export interface TransactionSummary {
  tx_signature: string;
  action: TransactionAction;
  token_mint: string;
  token_symbol?: string;
  venue: string;
  entry_sol: number;
  exit_sol?: number;
  pnl_sol?: number;
  pnl_percent?: number;
  slippage_bps: number;
  execution_time_ms: number;
  strategy_id?: string;
  timestamp: string;
  metadata: TransactionMetadata;
}

export type ExecutionErrorType =
  | 'rpc_timeout'
  | 'slippage_exceeded'
  | 'insufficient_funds'
  | 'tx_failed'
  | 'simulation_failed'
  | 'signing_failed'
  | 'network_error'
  | 'invalid_params'
  | 'rate_limited'
  | 'unknown';

export interface ErrorContext {
  action?: string;
  token_mint?: string;
  attempted_amount_sol?: number;
  venue?: string;
  strategy_id?: string;
  edge_id?: string;
}

export interface ExecutionError {
  error_type: ExecutionErrorType;
  message: string;
  context: ErrorContext;
  stack_trace?: string;
  recoverable: boolean;
  timestamp: string;
}

export interface TradeHighlight {
  token: string;
  pnl_sol: number;
  tx_signature?: string;
}

export interface VenueMetrics {
  trades: number;
  pnl_sol: number;
  win_rate: number;
}

export interface StrategyMetrics {
  trades: number;
  pnl_sol: number;
  win_rate: number;
}

export interface DailyMetrics {
  period: string;
  total_trades: number;
  winning_trades: number;
  win_rate: number;
  total_pnl_sol: number;
  avg_trade_pnl: number;
  max_drawdown_percent: number;
  best_trade?: TradeHighlight;
  worst_trade?: TradeHighlight;
  by_venue: Record<string, VenueMetrics>;
  by_strategy: Record<string, StrategyMetrics>;
}

export const A2A_TAG_LEARNING = 'arbFarm.learning';

export interface EngramBrowserFilter {
  engram_type?: EngramType;
  tags?: string[];
  query?: string;
  limit?: number;
}
