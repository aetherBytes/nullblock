export interface ConsensusConfig {
  enabled: boolean;
  models: ConsensusModelConfig[];
  min_consensus_threshold: number;
  auto_apply_recommendations: boolean;
  review_interval_hours: number;
  max_tokens_per_request: number;
  timeout_ms: number;
}

export interface ConsensusModelConfig {
  model_id: string;
  display_name: string;
  provider: string;
  weight: number;
  enabled: boolean;
  max_tokens: number;
}

export interface ConsensusConfigResponse {
  config: ConsensusConfig;
  is_dev_wallet: boolean;
  available_models: ConsensusModelConfig[];
}

export interface UpdateConsensusConfigRequest {
  enabled?: boolean;
  models?: ConsensusModelConfig[];
  min_consensus_threshold?: number;
  auto_apply_recommendations?: boolean;
  review_interval_hours?: number;
}

export interface ModelVote {
  model: string;
  approved: boolean;
  confidence: number;
  reasoning: string;
  latency_ms: number;
}

export interface ConsensusResult {
  approved: boolean;
  agreement_score: number;
  weighted_confidence: number;
  reasoning_summary: string;
  model_votes: ModelVote[];
  total_latency_ms: number;
}

export interface ConsensusHistoryEntry {
  id: string;
  edge_id: string;
  result: ConsensusResult;
  edge_context: string;
  created_at: string;
}

export interface ConsensusStatsResponse {
  total_decisions: number;
  approved_count: number;
  rejected_count: number;
  average_agreement: number;
  average_confidence: number;
  average_latency_ms: number;
  decisions_last_24h: number;
}

export interface ModelInfo {
  id: string;
  name: string;
  provider: string;
}

export interface AvailableModelsResponse {
  models: ModelInfo[];
  default_models: string[];
}

export type ConversationTopic =
  | 'trade_analysis'
  | 'risk_assessment'
  | 'strategy_review'
  | 'pattern_discovery'
  | 'market_conditions';

export type ConversationTrigger =
  | 'daily_review'
  | 'trade_failure'
  | 'high_profit_trade'
  | 'risk_alert'
  | 'user_request'
  | 'scheduled';

export interface ConversationContext {
  trigger: ConversationTrigger;
  trades_in_scope?: number;
  time_period?: string;
  additional_context?: Record<string, unknown>;
}

export interface ConversationMessage {
  role: string;
  content: string;
  timestamp: string;
  tokens_used?: number;
  latency_ms?: number;
}

export interface ConversationOutcome {
  consensus_reached: boolean;
  recommendations_generated: number;
  engram_refs: string[];
  summary?: string;
}

export interface ConversationLog {
  session_id: string;
  participants: string[];
  topic: ConversationTopic;
  context: ConversationContext;
  messages: ConversationMessage[];
  outcome: ConversationOutcome;
  created_at: string;
}

export interface ConversationListResponse {
  conversations: ConversationLog[];
  total: number;
}

export interface LearningSummary {
  total_recommendations: number;
  pending_recommendations: number;
  applied_recommendations: number;
  total_conversations: number;
  recent_conversations: ConversationLog[];
  recent_recommendations: Recommendation[];
}

export type RecommendationSource =
  | 'consensus_llm'
  | 'pattern_analysis'
  | 'risk_engine'
  | 'manual';

export type RecommendationCategory =
  | 'strategy'
  | 'risk'
  | 'timing'
  | 'venue'
  | 'position';

export type RecommendationStatus =
  | 'pending'
  | 'acknowledged'
  | 'applied'
  | 'rejected';

export type SuggestedActionType =
  | 'config_change'
  | 'strategy_toggle'
  | 'risk_adjustment'
  | 'venue_disable'
  | 'avoid_token';

export interface SuggestedAction {
  action_type: SuggestedActionType;
  target: string;
  current_value?: unknown;
  suggested_value: unknown;
  reasoning: string;
}

export interface SupportingData {
  trades_analyzed: number;
  time_period: string;
  relevant_engrams: string[];
  metrics?: Record<string, unknown>;
}

export interface Recommendation {
  recommendation_id: string;
  source: RecommendationSource;
  category: RecommendationCategory;
  title: string;
  description: string;
  suggested_action: SuggestedAction;
  confidence: number;
  supporting_data: SupportingData;
  status: RecommendationStatus;
  created_at: string;
  applied_at?: string;
}

export interface RecommendationListResponse {
  recommendations: Recommendation[];
  total: number;
}

export interface DiscoveredModel {
  id: string;
  name: string;
  provider?: string;
  context_length?: number;
  pricing?: {
    prompt: string;
    completion: string;
  };
  reasoning_capable?: boolean;
  image_capable?: boolean;
}

export interface TradeAnalysis {
  analysis_id: string;
  position_id: string;
  token_symbol: string;
  venue: string;
  pnl_sol: number;
  exit_reason: string;
  root_cause: string;
  config_issue?: string;
  pattern?: string;
  suggested_fix?: string;
  confidence: number;
  created_at: string;
}

export interface TradeAnalysisListResponse {
  analyses: TradeAnalysis[];
  total: number;
}

export interface PatternSummary {
  summary_id: string;
  losing_patterns: string[];
  winning_patterns: string[];
  config_recommendations: string[];
  trades_analyzed: number;
  time_period: string;
  created_at: string;
}

export interface PatternSummaryResponse {
  summary: PatternSummary | null;
  has_data: boolean;
}

export interface AnalysisSummaryResponse {
  trade_analyses_count: number;
  latest_pattern_summary: PatternSummary | null;
  recent_trade_analyses: TradeAnalysis[];
  config: ConsensusConfig;
  is_dev_wallet: boolean;
}
