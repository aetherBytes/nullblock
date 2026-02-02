/**
 * ArbFarm Service
 *
 * API client for the ArbFarm MEV Agent Swarm backend.
 * Handles all communication with the arb-farm service for:
 * - Edge (opportunity) management
 * - Trade execution and history
 * - Scanner status and signals
 * - Swarm health monitoring
 * - Threat detection
 * - KOL tracking and copy trading
 * - Strategy management
 */

import type {
  Edge,
  EdgeFilter,
  Trade,
  TradeStats,
  DailyStats,
  ScannerStatus,
  Signal,
  BehavioralStrategy,
  Contender,
  BehavioralStrategiesResponse,
  SwarmStatus,
  SwarmHealth,
  AgentStatus,
  CircuitBreakerStatus,
  Strategy,
  ThreatScore,
  ThreatAlert,
  BlockedEntity,
  KOL,
  KOLTrade,
  CopyTrade,
  TrustBreakdown,
  CurveToken,
  GraduationCandidate,
  CrossVenueArbitrage,
  CurveQuote,
  ArbEngram,
  PatternMatch,
  HarvesterStats,
  DashboardSummary,
  CurveVenue,
  ThreatCategory,
  ThreatSeverity,
  ArbFarmCow,
  ArbFarmCowSummary,
  ArbFarmCowFork,
  ArbFarmRevenue,
  ArbFarmEarningsSummary,
  ArbFarmCowStats,
  CreateArbFarmCowRequest,
  ForkArbFarmCowRequest,
  ArbFarmCowStrategy,
  WalletStatus,
  WalletSetupResponse,
  WalletBalanceResponse,
  DailyUsageResponse,
  UpdatePolicyRequest,
  AllSettingsResponse,
  RiskSettingsResponse,
  RiskConfig,
  VenueConfig,
  ApiKeyStatus,
  HeliusStatus,
  LaserStreamStatus,
  PriorityFees,
  SenderStats,
  TokenMetadata,
  HeliusConfig,
  IngestUrlResponse,
  ExtractedStrategy,
  DiscoveryListResponse,
  MonitoredSource,
  MonitorStats,
  SocialAlert,
  CapitalUsageResponse,
  SyncCapitalResponse,
  CurvePosition,
  CurveExitConfig,
  CurveStrategyParams,
  CurveStrategyStats,
  TrackedToken,
  SnipePosition,
  SniperStats,
  GraduationTrackerStats,
  OnChainCurveState,
  CurveBuildResult,
  SimulatedTrade,
  PnLSummary,
  LearningInsight,
  LearningInsightsResponse,
  RiskLevel,
  RiskLevelParams,
  SetRiskLevelResponse,
  OpenPosition,
  PositionsResponse,
  PositionExposure,
  PositionHistoryResponse,
  MonitorStatus,
  ReconcileResult,
  WhitelistedEntity,
  WatchedEntity,
  ThreatHistoryItem,
  ThreatStats,
} from '../../types/arbfarm';

export interface ArbFarmServiceResponse<T = unknown> {
  success: boolean;
  data?: T;
  error?: string;
  timestamp: Date;
}

class ArbFarmService {
  private baseUrl: string;
  private sseBaseUrl: string;
  private isConnected: boolean = false;
  private walletAddress: string | null = null;

  constructor(baseUrl: string = import.meta.env.VITE_ARBFARM_API_URL || `${import.meta.env.VITE_EREBUS_API_URL || 'http://localhost:3000'}/api/arb`) {
    this.baseUrl = baseUrl;
    this.sseBaseUrl = import.meta.env.VITE_ARBFARM_SSE_URL || 'http://localhost:9007';
  }

  setWalletContext(walletAddress: string | null) {
    this.walletAddress = walletAddress;
  }

  async connect(): Promise<boolean> {
    try {
      const response = await fetch(`${this.baseUrl}/health`);

      this.isConnected = response.ok;

      return this.isConnected;
    } catch (error) {
      console.error('Failed to connect to ArbFarm:', error);
      this.isConnected = false;

      return false;
    }
  }

  private async makeRequest<T>(
    endpoint: string,
    options: RequestInit = {},
  ): Promise<ArbFarmServiceResponse<T>> {
    try {
      if (!this.isConnected) {
        await this.connect();
      }

      const headers: Record<string, string> = {
        'Content-Type': 'application/json',
        ...(options.headers as Record<string, string>),
      };

      if (this.walletAddress) {
        headers['x-wallet-address'] = this.walletAddress;
      }

      const url = `${this.baseUrl}${endpoint}`;
      const response = await fetch(url, {
        headers,
        ...options,
      });

      // Handle responses - always try parsing as JSON
      const text = await response.text();
      let responseJson: Record<string, unknown> | null = null;

      if (text.length > 0) {
        try {
          responseJson = JSON.parse(text);
        } catch {
          // Response wasn't valid JSON, treat as empty
        }
      }

      const actualData =
        response.ok && responseJson?.data !== undefined ? responseJson.data : responseJson;

      return {
        success: response.ok,
        data: response.ok ? actualData : undefined,
        error: response.ok
          ? undefined
          : (responseJson?.message as string) || (responseJson?.error as string) || 'Request failed',
        timestamp: new Date(),
      };
    } catch (error) {
      console.error('ArbFarm service request failed:', error);

      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date(),
      };
    }
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {},
  ): Promise<ArbFarmServiceResponse<T>> {
    return this.makeRequest<T>(endpoint, options);
  }

  // ============================================================================
  // Health & Status
  // ============================================================================

  async getHealth(): Promise<ArbFarmServiceResponse<{ status: string; service: string }>> {
    return this.makeRequest('/health');
  }

  // ============================================================================
  // Dashboard Summary
  // ============================================================================

  async getDashboardSummary(): Promise<ArbFarmServiceResponse<DashboardSummary>> {
    try {
      const [statsRes, swarmRes, edgesRes, tradesRes, alertsRes, pnlRes] = await Promise.all([
        this.getTradeStats().catch(() => ({ success: false, data: undefined, timestamp: new Date() }) as ArbFarmServiceResponse<TradeStats>),
        this.getSwarmHealth().catch(() => ({ success: false, data: undefined, timestamp: new Date() }) as ArbFarmServiceResponse<SwarmHealth>),
        this.listEdges({ status: ['detected', 'pending_approval'], limit: 5 }).catch(() => ({ success: false, data: undefined, timestamp: new Date() }) as ArbFarmServiceResponse<Edge[]>),
        this.listTrades(5).catch(() => ({ success: false, data: undefined, timestamp: new Date() }) as ArbFarmServiceResponse<Trade[]>),
        this.getThreatAlerts(5).catch(() => ({ success: false, data: undefined, timestamp: new Date() }) as ArbFarmServiceResponse<ThreatAlert[]>),
        this.getPnLSummary().catch(() => ({ success: false, data: undefined, timestamp: new Date() }) as ArbFarmServiceResponse<PnLSummary>),
      ]);

      const stats = statsRes.data;
      const pnl = pnlRes.data;
      const summary: DashboardSummary = {
        total_profit_sol: pnl?.total_sol ?? stats?.net_pnl_sol ?? 0,
        today_profit_sol: pnl?.today_sol ?? 0,
        week_profit_sol: pnl?.week_sol ?? 0,
        win_rate: pnl?.win_rate ?? stats?.win_rate ?? 0,
        active_opportunities: pnl?.active_positions ?? (Array.isArray(edgesRes.data) ? edgesRes.data : edgesRes.data?.edges)?.filter((e: Edge) => e.status === 'detected')?.length ?? 0,
        pending_approvals:
          (Array.isArray(edgesRes.data) ? edgesRes.data : edgesRes.data?.edges)?.filter((e: Edge) => e.status === 'pending_approval')?.length || 0,
        executed_today: pnl?.total_trades ?? stats?.total_trades ?? 0,
        swarm_health: swarmRes.data!,
        top_opportunities: (Array.isArray(edgesRes.data) ? edgesRes.data : edgesRes.data?.edges) || [],
        recent_trades: (Array.isArray(tradesRes.data) ? tradesRes.data : tradesRes.data?.trades) || [],
        recent_alerts: (Array.isArray(alertsRes.data) ? alertsRes.data : alertsRes.data?.alerts) || [],
        pnl_reset_at: pnl?.pnl_reset_at,
      };

      return {
        success: true,
        data: summary,
        timestamp: new Date(),
      };
    } catch (error) {
      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date(),
      };
    }
  }

  // ============================================================================
  // Edge (Opportunity) Operations
  // ============================================================================

  async listEdges(filter?: EdgeFilter): Promise<ArbFarmServiceResponse<Edge[]>> {
    const params = new URLSearchParams();

    if (filter?.status) {
      params.append('status', filter.status.join(','));
    }

    if (filter?.edge_type) {
      params.append('edge_type', filter.edge_type.join(','));
    }

    if (filter?.venue_type) {
      params.append('venue_type', filter.venue_type.join(','));
    }

    if (filter?.min_profit_lamports) {
      params.append('min_profit', filter.min_profit_lamports.toString());
    }

    if (filter?.max_risk_score) {
      params.append('max_risk', filter.max_risk_score.toString());
    }

    if (filter?.limit) {
      params.append('limit', filter.limit.toString());
    }

    if (filter?.offset) {
      params.append('offset', filter.offset.toString());
    }

    const queryString = params.toString();

    return this.makeRequest(`/edges${queryString ? `?${queryString}` : ''}`);
  }

  async listAtomicEdges(
    minProfit?: number,
    limit?: number,
  ): Promise<ArbFarmServiceResponse<Edge[]>> {
    const params = new URLSearchParams();

    if (minProfit) {
      params.append('min_profit', minProfit.toString());
    }

    if (limit) {
      params.append('limit', limit.toString());
    }

    const queryString = params.toString();

    return this.makeRequest(`/edges/atomic${queryString ? `?${queryString}` : ''}`);
  }

  async getEdge(id: string): Promise<ArbFarmServiceResponse<Edge>> {
    return this.makeRequest(`/edges/${id}`);
  }

  async approveEdge(id: string): Promise<ArbFarmServiceResponse<Edge>> {
    return this.makeRequest(`/edges/${id}/approve`, { method: 'POST' });
  }

  async rejectEdge(id: string, reason: string): Promise<ArbFarmServiceResponse<Edge>> {
    return this.makeRequest(`/edges/${id}/reject`, {
      method: 'POST',
      body: JSON.stringify({ reason }),
    });
  }

  async executeEdge(id: string, maxSlippageBps?: number): Promise<ArbFarmServiceResponse<Trade>> {
    return this.makeRequest(`/edges/${id}/execute`, {
      method: 'POST',
      body: JSON.stringify({ max_slippage_bps: maxSlippageBps }),
    });
  }

  async executeEdgeAuto(
    id: string,
    slippageBps?: number,
  ): Promise<
    ArbFarmServiceResponse<{
      edge_id: string;
      success: boolean;
      tx_signature?: string;
      bundle_id?: string;
      profit_lamports?: number;
      gas_cost_lamports?: number;
      execution_time_ms: number;
      error?: string;
      route_info?: {
        input_mint: string;
        output_mint: string;
        in_amount: number;
        out_amount: number;
        price_impact_bps: number;
      };
    }>
  > {
    return this.makeRequest(`/edges/${id}/execute-auto`, {
      method: 'POST',
      body: JSON.stringify({ slippage_bps: slippageBps }),
    });
  }

  async simulateEdge(id: string): Promise<
    ArbFarmServiceResponse<{
      success: boolean;
      estimated_profit_lamports: number;
      gas_estimate_lamports: number;
    }>
  > {
    return this.makeRequest(`/edges/${id}/simulate`, { method: 'POST' });
  }

  // ============================================================================
  // Trade Operations
  // ============================================================================

  async listTrades(limit?: number, offset?: number): Promise<ArbFarmServiceResponse<Trade[]>> {
    const params = new URLSearchParams();

    if (limit) {
      params.append('limit', limit.toString());
    }

    if (offset) {
      params.append('offset', offset.toString());
    }

    const queryString = params.toString();

    return this.makeRequest(`/trades${queryString ? `?${queryString}` : ''}`);
  }

  async getTrade(id: string): Promise<ArbFarmServiceResponse<Trade>> {
    return this.makeRequest(`/trades/${id}`);
  }

  async getTradeStats(period?: string): Promise<ArbFarmServiceResponse<TradeStats>> {
    const params = period ? `?period=${period}` : '';

    return this.makeRequest(`/trades/stats${params}`);
  }

  async getDailyStats(days?: number): Promise<ArbFarmServiceResponse<DailyStats[]>> {
    const params = days ? `?days=${days}` : '';

    return this.makeRequest(`/trades/daily${params}`);
  }

  // ============================================================================
  // Scanner Operations
  // ============================================================================

  async getScannerStatus(): Promise<ArbFarmServiceResponse<ScannerStatus>> {
    return this.makeRequest('/scanner/status');
  }

  async startScanner(): Promise<ArbFarmServiceResponse<{ success: boolean; message: string }>> {
    return this.makeRequest('/scanner/start', { method: 'POST' });
  }

  async stopScanner(): Promise<ArbFarmServiceResponse<{ success: boolean; message: string }>> {
    return this.makeRequest('/scanner/stop', { method: 'POST' });
  }

  async getSignals(options?: {
    limit?: number;
    signal_type?: string;
    venue_type?: string;
    min_profit_bps?: number;
    min_confidence?: number;
  }): Promise<ArbFarmServiceResponse<Signal[]>> {
    const params = new URLSearchParams();
    if (options?.limit) params.set('limit', options.limit.toString());
    if (options?.signal_type) params.set('signal_type', options.signal_type);
    if (options?.venue_type) params.set('venue_type', options.venue_type);
    if (options?.min_profit_bps) params.set('min_profit_bps', options.min_profit_bps.toString());
    if (options?.min_confidence) params.set('min_confidence', options.min_confidence.toString());
    const queryString = params.toString();
    return this.makeRequest(`/scanner/signals${queryString ? '?' + queryString : ''}`);
  }

  async getContenders(limit?: number): Promise<ArbFarmServiceResponse<{ contenders: Contender[]; count: number }>> {
    const params = limit ? `?limit=${limit}` : '';
    return this.makeRequest(`/scanner/contenders${params}`);
  }

  subscribeToSignals(onSignal: (signal: Signal) => void): () => void {
    const eventSource = new EventSource(`${this.sseBaseUrl}/scanner/stream`);
    eventSource.addEventListener('signal', (event) => {
      try {
        const data = JSON.parse(event.data);
        if (data.payload) {
          onSignal(data.payload as Signal);
        }
      } catch (err) {
        console.error('Failed to parse signal event:', err);
      }
    });
    eventSource.onerror = () => {
      console.warn('Scanner SSE connection error, will retry...');
    };
    return () => eventSource.close();
  }

  // ============================================================================
  // Behavioral Strategies (Scanner-driven)
  // ============================================================================

  async listBehavioralStrategies(): Promise<ArbFarmServiceResponse<BehavioralStrategiesResponse>> {
    return this.makeRequest('/scanner/strategies');
  }

  async getBehavioralStrategy(name: string): Promise<ArbFarmServiceResponse<BehavioralStrategy>> {
    return this.makeRequest(`/scanner/strategies/${encodeURIComponent(name)}`);
  }

  async toggleBehavioralStrategy(
    name: string,
    active: boolean,
  ): Promise<ArbFarmServiceResponse<{ success: boolean; name: string; is_active: boolean; message: string }>> {
    return this.makeRequest(`/scanner/strategies/${encodeURIComponent(name)}/toggle`, {
      method: 'POST',
      body: JSON.stringify({ active }),
    });
  }

  async toggleAllBehavioralStrategies(
    active: boolean,
  ): Promise<ArbFarmServiceResponse<{ success: boolean; toggled_count: number; is_active: boolean; message: string }>> {
    return this.makeRequest('/scanner/strategies/toggle-all', {
      method: 'POST',
      body: JSON.stringify({ active }),
    });
  }

  // ============================================================================
  // Swarm Operations
  // ============================================================================

  async getSwarmStatus(): Promise<ArbFarmServiceResponse<SwarmStatus>> {
    return this.makeRequest('/swarm/status');
  }

  async getSwarmHealth(): Promise<ArbFarmServiceResponse<SwarmHealth>> {
    return this.makeRequest('/swarm/health');
  }

  async listAgents(): Promise<ArbFarmServiceResponse<AgentStatus[]>> {
    return this.makeRequest('/swarm/agents');
  }

  async getAgentStatus(id: string): Promise<ArbFarmServiceResponse<AgentStatus>> {
    return this.makeRequest(`/swarm/agents/${id}`);
  }

  async pauseSwarm(): Promise<ArbFarmServiceResponse<{ success: boolean; message: string }>> {
    return this.makeRequest('/swarm/pause', { method: 'POST' });
  }

  async resumeSwarm(): Promise<ArbFarmServiceResponse<{ success: boolean; message: string }>> {
    return this.makeRequest('/swarm/resume', { method: 'POST' });
  }

  async listCircuitBreakers(): Promise<ArbFarmServiceResponse<CircuitBreakerStatus[]>> {
    return this.makeRequest('/swarm/circuit-breakers');
  }

  async resetCircuitBreaker(
    name: string,
  ): Promise<ArbFarmServiceResponse<{ success: boolean; message: string }>> {
    return this.makeRequest(`/swarm/circuit-breakers/${name}/reset`, { method: 'POST' });
  }

  async resetAllCircuitBreakers(): Promise<
    ArbFarmServiceResponse<{ success: boolean; message: string }>
  > {
    return this.makeRequest('/swarm/circuit-breakers/reset-all', { method: 'POST' });
  }

  // ============================================================================
  // Threat Detection Operations
  // ============================================================================

  async checkTokenThreat(mint: string): Promise<ArbFarmServiceResponse<ThreatScore>> {
    return this.makeRequest(`/threat/check/${mint}`);
  }

  async checkWalletThreat(address: string): Promise<
    ArbFarmServiceResponse<{
      is_suspicious: boolean;
      known_scams: number;
      associations: string[];
    }>
  > {
    return this.makeRequest(`/threat/wallet/${address}`);
  }

  async listBlockedEntities(
    category?: ThreatCategory,
    limit?: number,
  ): Promise<ArbFarmServiceResponse<BlockedEntity[]>> {
    const params = new URLSearchParams();

    if (category) {
      params.append('category', category);
    }

    if (limit) {
      params.append('limit', limit.toString());
    }

    const queryString = params.toString();

    return this.makeRequest(`/threat/blocked${queryString ? `?${queryString}` : ''}`);
  }

  async reportThreat(
    entityType: string,
    address: string,
    category: ThreatCategory,
    reason: string,
    severity: ThreatSeverity,
    evidenceUrl?: string,
  ): Promise<ArbFarmServiceResponse<BlockedEntity>> {
    return this.makeRequest('/threat/report', {
      method: 'POST',
      body: JSON.stringify({
        entity_type: entityType,
        address,
        category,
        reason,
        severity,
        evidence_url: evidenceUrl,
      }),
    });
  }

  async whitelistEntity(
    entityType: string,
    address: string,
    reason: string,
  ): Promise<ArbFarmServiceResponse<{ success: boolean; message: string }>> {
    return this.makeRequest('/threat/whitelist', {
      method: 'POST',
      body: JSON.stringify({ entity_type: entityType, address, reason }),
    });
  }

  async blockEntity(
    entityType: string,
    address: string,
    category: string,
    reason: string,
  ): Promise<ArbFarmServiceResponse<BlockedEntity>> {
    return this.makeRequest('/threat/block', {
      method: 'POST',
      body: JSON.stringify({ entity_type: entityType, address, threat_category: category, reason }),
    });
  }

  async getThreatAlerts(
    limit?: number,
    severity?: ThreatSeverity,
  ): Promise<ArbFarmServiceResponse<ThreatAlert[]>> {
    const params = new URLSearchParams();

    if (limit) {
      params.append('limit', limit.toString());
    }

    if (severity) {
      params.append('severity', severity);
    }

    const queryString = params.toString();

    return this.makeRequest(`/threat/alerts${queryString ? `?${queryString}` : ''}`);
  }

  // ============================================================================
  // KOL Tracking Operations
  // ============================================================================

  async listKOLs(
    activeOnly?: boolean,
    copyEnabled?: boolean,
  ): Promise<ArbFarmServiceResponse<KOL[]>> {
    const params = new URLSearchParams();

    if (activeOnly !== undefined) {
      params.append('is_active', activeOnly.toString());
    }

    if (copyEnabled !== undefined) {
      params.append('copy_enabled', copyEnabled.toString());
    }

    const queryString = params.toString();

    return this.makeRequest(`/kol${queryString ? `?${queryString}` : ''}`);
  }

  async addKOL(
    walletAddress?: string,
    twitterHandle?: string,
    displayName?: string,
  ): Promise<ArbFarmServiceResponse<KOL>> {
    return this.makeRequest('/kol', {
      method: 'POST',
      body: JSON.stringify({
        wallet_address: walletAddress,
        twitter_handle: twitterHandle,
        display_name: displayName,
      }),
    });
  }

  async getKOL(id: string): Promise<ArbFarmServiceResponse<KOL>> {
    return this.makeRequest(`/kol/${id}`);
  }

  async getKOLTrades(id: string, limit?: number): Promise<ArbFarmServiceResponse<KOLTrade[]>> {
    const params = limit ? `?limit=${limit}` : '';

    return this.makeRequest(`/kol/${id}/trades${params}`);
  }

  async getKOLStats(id: string): Promise<
    ArbFarmServiceResponse<{
      total_trades: number;
      win_rate: number;
      avg_profit_percent: number;
      total_profit_sol: number;
    }>
  > {
    return this.makeRequest(`/kol/${id}/stats`);
  }

  async getTrustBreakdown(id: string): Promise<ArbFarmServiceResponse<TrustBreakdown>> {
    return this.makeRequest(`/kol/${id}/trust`);
  }

  async enableCopyTrading(
    id: string,
    config: {
      max_position_sol?: number;
      delay_ms?: number;
      min_trust_score?: number;
      copy_percentage?: number;
    },
  ): Promise<ArbFarmServiceResponse<KOL>> {
    return this.makeRequest(`/kol/${id}/copy/enable`, {
      method: 'POST',
      body: JSON.stringify(config),
    });
  }

  async disableCopyTrading(id: string): Promise<ArbFarmServiceResponse<KOL>> {
    return this.makeRequest(`/kol/${id}/copy/disable`, { method: 'POST' });
  }

  async removeKOL(id: string): Promise<ArbFarmServiceResponse<void>> {
    return this.makeRequest(`/kol/${id}`, { method: 'DELETE' });
  }

  async listActiveCopies(): Promise<ArbFarmServiceResponse<CopyTrade[]>> {
    return this.makeRequest('/kol/copies/active');
  }

  async getCopyHistory(
    kolId: string,
    limit?: number,
  ): Promise<ArbFarmServiceResponse<CopyTrade[]>> {
    const params = limit ? `?limit=${limit}` : '';

    return this.makeRequest(`/kol/${kolId}/copy/history${params}`);
  }

  // ============================================================================
  // KOL Discovery Operations
  // ============================================================================

  async getKolDiscoveryStatus(): Promise<
    ArbFarmServiceResponse<{
      is_running: boolean;
      total_wallets_analyzed: number;
      total_kols_discovered: number;
      last_scan_at?: string;
      scan_interval_ms: number;
    }>
  > {
    return this.makeRequest('/kol/discovery/status');
  }

  async startKolDiscovery(): Promise<
    ArbFarmServiceResponse<{ status: string; message: string }>
  > {
    return this.makeRequest('/kol/discovery/start', { method: 'POST' });
  }

  async stopKolDiscovery(): Promise<
    ArbFarmServiceResponse<{ status: string; message: string }>
  > {
    return this.makeRequest('/kol/discovery/stop', { method: 'POST' });
  }

  async scanForKols(): Promise<
    ArbFarmServiceResponse<{
      discovered: Array<{
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
      }>;
      count: number;
      message: string;
    }>
  > {
    return this.makeRequest('/kol/discovery/scan', { method: 'POST' });
  }

  async listDiscoveredKols(options?: {
    min_trust_score?: number;
    min_win_rate?: number;
    source?: string;
    limit?: number;
  }): Promise<
    ArbFarmServiceResponse<{
      discovered: Array<{
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
      }>;
      total: number;
    }>
  > {
    const params = new URLSearchParams();
    if (options?.min_trust_score) params.append('min_trust_score', options.min_trust_score.toString());
    if (options?.min_win_rate) params.append('min_win_rate', options.min_win_rate.toString());
    if (options?.source) params.append('source', options.source);
    if (options?.limit) params.append('limit', options.limit.toString());
    const queryString = params.toString();
    return this.makeRequest(`/kol/discovery/discovered${queryString ? `?${queryString}` : ''}`);
  }

  async promoteDiscoveredKol(walletAddress: string): Promise<ArbFarmServiceResponse<KOL>> {
    return this.makeRequest(`/kol/discovery/promote/${walletAddress}`, { method: 'POST' });
  }

  // ============================================================================
  // Bonding Curve Operations
  // ============================================================================

  async listCurveTokens(
    venue?: CurveVenue,
    limit?: number,
  ): Promise<ArbFarmServiceResponse<CurveToken[]>> {
    const params = new URLSearchParams();

    if (venue) {
      params.append('venue', venue);
    }

    if (limit) {
      params.append('limit', limit.toString());
    }

    const queryString = params.toString();

    return this.makeRequest(`/curves/tokens${queryString ? `?${queryString}` : ''}`);
  }

  async getGraduationProgress(mint: string): Promise<
    ArbFarmServiceResponse<{
      token: CurveToken;
      graduation_progress: number;
      estimated_eta_minutes?: number;
    }>
  > {
    return this.makeRequest(`/curves/${mint}/progress`);
  }

  async getHolderStats(mint: string): Promise<
    ArbFarmServiceResponse<{
      total_holders: number;
      top_10_concentration: number;
      creator_holdings_percent: number;
      holders: { address: string; balance: number; percent: number }[];
    }>
  > {
    return this.makeRequest(`/curves/${mint}/holders`);
  }

  async getCurveQuote(
    mint: string,
    direction: 'buy' | 'sell',
    amount: number,
  ): Promise<ArbFarmServiceResponse<CurveQuote>> {
    return this.makeRequest(`/curves/${mint}/quote`, {
      method: 'POST',
      body: JSON.stringify({ direction, amount }),
    });
  }

  async listGraduationCandidates(
    minProgress?: number,
    maxProgress?: number,
    limit?: number,
  ): Promise<ArbFarmServiceResponse<GraduationCandidate[]>> {
    const params = new URLSearchParams();

    if (minProgress) {
      params.append('min_progress', minProgress.toString());
    }

    if (maxProgress) {
      params.append('max_progress', maxProgress.toString());
    }

    if (limit) {
      params.append('limit', limit.toString());
    }

    const queryString = params.toString();

    return this.makeRequest(`/curves/graduation-candidates${queryString ? `?${queryString}` : ''}`);
  }

  async detectCrossVenueArbitrage(
    minDiffPercent?: number,
  ): Promise<ArbFarmServiceResponse<CrossVenueArbitrage[]>> {
    const params = minDiffPercent ? `?min_diff=${minDiffPercent}` : '';

    return this.makeRequest(`/curves/cross-venue-arb${params}`);
  }

  // ============================================================================
  // Curve Execution Operations
  // ============================================================================

  async getCurveOnChainState(mint: string): Promise<ArbFarmServiceResponse<OnChainCurveState>> {
    return this.makeRequest(`/curves/${mint}/state`);
  }

  async buyCurveToken(
    mint: string,
    solAmount: number,
    options?: {
      slippage_bps?: number;
      simulate_only?: boolean;
    },
  ): Promise<ArbFarmServiceResponse<CurveBuildResult>> {
    return this.makeRequest(`/curves/${mint}/buy`, {
      method: 'POST',
      body: JSON.stringify({
        sol_amount: solAmount,
        slippage_bps: options?.slippage_bps ?? 300,
        simulate_only: options?.simulate_only ?? false,
      }),
    });
  }

  async sellCurveToken(
    mint: string,
    tokenAmount: number,
    options?: {
      slippage_bps?: number;
      simulate_only?: boolean;
    },
  ): Promise<ArbFarmServiceResponse<CurveBuildResult>> {
    return this.makeRequest(`/curves/${mint}/sell`, {
      method: 'POST',
      body: JSON.stringify({
        token_amount: tokenAmount,
        slippage_bps: options?.slippage_bps ?? 300,
        simulate_only: options?.simulate_only ?? false,
      }),
    });
  }

  async simulateCurveBuy(
    mint: string,
    solAmount: number,
  ): Promise<ArbFarmServiceResponse<SimulatedTrade>> {
    return this.makeRequest(`/curves/${mint}/simulate-buy`, {
      method: 'POST',
      body: JSON.stringify({ sol_amount: solAmount }),
    });
  }

  async simulateCurveSell(
    mint: string,
    tokenAmount: number,
  ): Promise<ArbFarmServiceResponse<SimulatedTrade>> {
    return this.makeRequest(`/curves/${mint}/simulate-sell`, {
      method: 'POST',
      body: JSON.stringify({ token_amount: tokenAmount }),
    });
  }

  async getPostGraduationPool(
    mint: string,
  ): Promise<ArbFarmServiceResponse<{ mint: string; raydium_pool?: string; dex_price_sol?: number }>> {
    return this.makeRequest(`/curves/${mint}/post-graduation-pool`);
  }

  async getCurveAddresses(
    mint: string,
  ): Promise<
    ArbFarmServiceResponse<{
      mint: string;
      bonding_curve: string;
      associated_bonding_curve: string;
      global_state: string;
      fee_recipient: string;
    }>
  > {
    return this.makeRequest(`/curves/${mint}/addresses`);
  }

  // ============================================================================
  // Curve Metrics & Scoring Operations
  // ============================================================================

  async getCurveMetrics(
    mint: string,
  ): Promise<ArbFarmServiceResponse<import('../../types/arbfarm').DetailedCurveMetrics>> {
    return this.makeRequest(`/curves/${mint}/metrics`);
  }

  async getHolderAnalysis(
    mint: string,
  ): Promise<ArbFarmServiceResponse<import('../../types/arbfarm').HolderAnalysis>> {
    return this.makeRequest(`/curves/${mint}/holder-analysis`);
  }

  async getOpportunityScore(
    mint: string,
  ): Promise<ArbFarmServiceResponse<import('../../types/arbfarm').OpportunityScore>> {
    return this.makeRequest(`/curves/${mint}/score`);
  }

  async getMarketData(
    mint: string,
  ): Promise<ArbFarmServiceResponse<import('../../types/arbfarm').MarketData>> {
    return this.makeRequest(`/curves/${mint}/market-data`);
  }

  async getTopOpportunities(
    limit?: number,
  ): Promise<ArbFarmServiceResponse<import('../../types/arbfarm').TopOpportunitiesResponse>> {
    const params = limit ? `?limit=${limit}` : '';
    return this.makeRequest(`/curves/top-opportunities${params}`);
  }

  async getScoringConfig(): Promise<ArbFarmServiceResponse<import('../../types/arbfarm').ScoringConfig>> {
    return this.makeRequest('/curves/scoring-config');
  }

  // ============================================================================
  // Curve Position Operations
  // ============================================================================

  async listCurvePositions(): Promise<ArbFarmServiceResponse<CurvePosition[]>> {
    return this.makeRequest('/positions');
  }

  async getCurvePosition(positionId: string): Promise<ArbFarmServiceResponse<CurvePosition>> {
    return this.makeRequest(`/curves/positions/${positionId}`);
  }

  async closeCurvePosition(
    positionId: string,
    percentToClose?: number,
  ): Promise<ArbFarmServiceResponse<{ success: boolean; tx_signature?: string; pnl_sol?: number }>> {
    return this.makeRequest(`/curves/positions/${positionId}/close`, {
      method: 'POST',
      body: JSON.stringify({ percent: percentToClose ?? 100 }),
    });
  }

  async updateCurvePositionExit(
    positionId: string,
    exitConfig: Partial<CurveExitConfig>,
  ): Promise<ArbFarmServiceResponse<CurvePosition>> {
    return this.makeRequest(`/curves/positions/${positionId}/exit-config`, {
      method: 'PUT',
      body: JSON.stringify(exitConfig),
    });
  }

  async emergencyExitAllPositions(): Promise<ArbFarmServiceResponse<{
    positions_exited: number;
    positions_failed: number;
    total_positions: number;
    message: string;
    results: Array<{
      position_id: string;
      token_mint: string;
      token_symbol?: string;
      success: boolean;
      error?: string;
    }>;
  }>> {
    return this.makeRequest('/positions/emergency-close', { method: 'POST' });
  }

  // ============================================================================
  // Graduation Tracker Operations
  // ============================================================================

  async getGraduationTrackerStats(): Promise<ArbFarmServiceResponse<GraduationTrackerStats>> {
    return this.makeRequest('/graduation/stats');
  }

  async listTrackedTokens(): Promise<ArbFarmServiceResponse<TrackedToken[]>> {
    return this.makeRequest('/graduation/tracked');
  }

  async trackToken(
    mint: string,
    strategyId: string,
    options?: {
      name?: string;
      symbol?: string;
      venue?: string;
    },
  ): Promise<ArbFarmServiceResponse<{ success: boolean; message: string; mint: string }>> {
    return this.makeRequest('/graduation/track', {
      method: 'POST',
      body: JSON.stringify({
        mint,
        strategy_id: strategyId || undefined,
        name: options?.name,
        symbol: options?.symbol,
        venue: options?.venue,
      }),
    });
  }

  async untrackToken(
    mint: string,
  ): Promise<ArbFarmServiceResponse<{ success: boolean; message: string }>> {
    return this.makeRequest(`/graduation/untrack/${mint}`, { method: 'POST' });
  }

  async isTokenTracked(
    mint: string,
  ): Promise<ArbFarmServiceResponse<{ mint: string; is_tracked: boolean; token: TrackedToken | null }>> {
    return this.makeRequest(`/graduation/tracked/${mint}`);
  }

  async clearAllTracked(): Promise<ArbFarmServiceResponse<{ success: boolean; message: string; cleared: number }>> {
    return this.makeRequest('/graduation/clear', { method: 'POST' });
  }

  async startGraduationTracker(): Promise<
    ArbFarmServiceResponse<{ success: boolean; message: string }>
  > {
    return this.makeRequest('/graduation/start', { method: 'POST' });
  }

  async stopGraduationTracker(): Promise<
    ArbFarmServiceResponse<{ success: boolean; message: string }>
  > {
    return this.makeRequest('/graduation/stop', { method: 'POST' });
  }

  // ============================================================================
  // Graduation Sniper Operations
  // ============================================================================

  async getSniperStats(): Promise<ArbFarmServiceResponse<{ stats: SniperStats }>> {
    return this.makeRequest('/sniper/stats');
  }

  async listSnipePositions(): Promise<ArbFarmServiceResponse<SnipePosition[]>> {
    return this.makeRequest('/sniper/positions');
  }

  async addSnipePosition(
    mint: string,
    symbol: string,
    strategyId: string,
    entryTokens: number,
    entryPriceSol: number,
  ): Promise<ArbFarmServiceResponse<{ success: boolean; message: string }>> {
    return this.makeRequest('/sniper/positions', {
      method: 'POST',
      body: JSON.stringify({
        mint,
        symbol,
        strategy_id: strategyId,
        entry_tokens: entryTokens,
        entry_price_sol: entryPriceSol,
      }),
    });
  }

  async removeSnipePosition(
    mint: string,
  ): Promise<ArbFarmServiceResponse<{ success: boolean; position?: SnipePosition }>> {
    return this.makeRequest(`/sniper/positions/${mint}`, { method: 'DELETE' });
  }

  async manualSell(
    mint: string,
  ): Promise<ArbFarmServiceResponse<{ success: boolean; message: string }>> {
    return this.makeRequest(`/sniper/positions/${mint}/sell`, { method: 'POST' });
  }

  async startSniper(): Promise<ArbFarmServiceResponse<{ success: boolean; message: string }>> {
    return this.makeRequest('/sniper/start', { method: 'POST' });
  }

  async stopSniper(): Promise<ArbFarmServiceResponse<{ success: boolean; message: string }>> {
    return this.makeRequest('/sniper/stop', { method: 'POST' });
  }

  // ============================================================================
  // Curve Strategy Operations
  // ============================================================================

  async createCurveStrategy(
    name: string,
    params: CurveStrategyParams,
    executionMode: 'autonomous' | 'agent_directed' | 'hybrid' = 'agent_directed',
    riskParams?: Partial<import('../../types/arbfarm').RiskParams>,
  ): Promise<ArbFarmServiceResponse<Strategy>> {
    return this.makeRequest('/strategies', {
      method: 'POST',
      body: JSON.stringify({
        name,
        strategy_type: 'curve_arb',
        venue_types: ['bonding_curve'],
        execution_mode: executionMode,
        curve_params: params,
        risk_params: riskParams,
      }),
    });
  }

  async getCurveStrategyStats(strategyId: string): Promise<ArbFarmServiceResponse<CurveStrategyStats>> {
    return this.makeRequest(`/strategies/${strategyId}/curve-stats`);
  }

  async updateCurveStrategyParams(
    strategyId: string,
    params: Partial<CurveStrategyParams>,
  ): Promise<ArbFarmServiceResponse<Strategy>> {
    return this.makeRequest(`/strategies/${strategyId}/curve-params`, {
      method: 'PUT',
      body: JSON.stringify(params),
    });
  }

  async setRiskProfile(
    strategyId: string,
    profile: 'conservative' | 'moderate' | 'aggressive',
  ): Promise<ArbFarmServiceResponse<{ success: boolean; profile: string; strategy: Strategy }>> {
    return this.makeRequest(`/strategies/${strategyId}/risk-profile`, {
      method: 'POST',
      body: JSON.stringify({ profile }),
    });
  }

  // ============================================================================
  // Engram Operations
  // ============================================================================

  async getEngram(key: string): Promise<ArbFarmServiceResponse<ArbEngram>> {
    return this.makeRequest(`/engram/${encodeURIComponent(key)}`);
  }

  async searchEngrams(params: {
    engram_type?: string;
    key_prefix?: string;
    tag?: string;
    min_confidence?: number;
    limit?: number;
  }): Promise<ArbFarmServiceResponse<ArbEngram[]>> {
    const searchParams = new URLSearchParams();

    if (params.engram_type) {
      searchParams.append('engram_type', params.engram_type);
    }

    if (params.key_prefix) {
      searchParams.append('key_prefix', params.key_prefix);
    }

    if (params.tag) {
      searchParams.append('tag', params.tag);
    }

    if (params.min_confidence) {
      searchParams.append('min_confidence', params.min_confidence.toString());
    }

    if (params.limit) {
      searchParams.append('limit', params.limit.toString());
    }

    const queryString = searchParams.toString();

    return this.makeRequest(`/engram/search${queryString ? `?${queryString}` : ''}`);
  }

  async findPatterns(
    edgeType: string,
    venueType: string,
  ): Promise<ArbFarmServiceResponse<PatternMatch[]>> {
    return this.makeRequest('/engram/patterns', {
      method: 'POST',
      body: JSON.stringify({ edge_type: edgeType, venue_type: venueType }),
    });
  }

  async getHarvesterStats(): Promise<ArbFarmServiceResponse<HarvesterStats>> {
    return this.makeRequest('/engram/stats');
  }

  // ============================================================================
  // Strategy Operations
  // ============================================================================

  async listStrategies(): Promise<ArbFarmServiceResponse<Strategy[]>> {
    return this.makeRequest('/strategies');
  }

  async createStrategy(
    strategy: Omit<Strategy, 'id' | 'created_at' | 'updated_at'>,
  ): Promise<ArbFarmServiceResponse<Strategy>> {
    return this.makeRequest('/strategies', {
      method: 'POST',
      body: JSON.stringify(strategy),
    });
  }

  async getStrategy(id: string): Promise<ArbFarmServiceResponse<Strategy>> {
    return this.makeRequest(`/strategies/${id}`);
  }

  async updateStrategy(
    id: string,
    updates: Partial<Strategy>,
  ): Promise<ArbFarmServiceResponse<Strategy>> {
    return this.makeRequest(`/strategies/${id}`, {
      method: 'PUT',
      body: JSON.stringify(updates),
    });
  }

  async toggleStrategy(id: string, enabled: boolean): Promise<ArbFarmServiceResponse<Strategy>> {
    return this.makeRequest(`/strategies/${id}/toggle`, {
      method: 'POST',
      body: JSON.stringify({ enabled }),
    });
  }

  async deleteStrategy(id: string): Promise<ArbFarmServiceResponse<void>> {
    return this.makeRequest(`/strategies/${id}`, { method: 'DELETE' });
  }

  async killStrategy(id: string): Promise<
    ArbFarmServiceResponse<{
      success: boolean;
      id: string;
      strategy_name: string;
      message: string;
      action: string;
    }>
  > {
    return this.makeRequest(`/strategies/${id}/kill`, { method: 'POST' });
  }

  async batchToggleStrategies(
    ids: string[],
    enabled: boolean,
  ): Promise<ArbFarmServiceResponse<{ results: Array<{ id: string; success: boolean; error?: string }>; enabled: boolean }>> {
    return this.makeRequest('/strategies/batch-toggle', {
      method: 'POST',
      body: JSON.stringify({ ids, enabled }),
    });
  }

  async saveStrategiesToEngrams(): Promise<ArbFarmServiceResponse<{ success: boolean; message: string; count: number }>> {
    return this.makeRequest('/strategies/save-to-engrams', { method: 'POST' });
  }

  async resetStrategyStats(id: string): Promise<ArbFarmServiceResponse<{ success: boolean; message: string }>> {
    return this.makeRequest(`/strategies/${id}/reset-stats`, { method: 'POST' });
  }

  // ============================================================================
  // COW (Constellation of Work) Operations - via Erebus/Crossroads
  // ============================================================================

  private get erebusBaseUrl(): string {
    return import.meta.env.VITE_API_URL || 'http://localhost:3000';
  }

  async listCows(
    limit?: number,
    offset?: number,
    isPublic?: boolean,
    isForkable?: boolean,
  ): Promise<ArbFarmServiceResponse<ArbFarmCowSummary[]>> {
    const params = new URLSearchParams();

    if (limit) {
      params.append('limit', limit.toString());
    }

    if (offset) {
      params.append('offset', offset.toString());
    }

    if (isPublic !== undefined) {
      params.append('is_public', isPublic.toString());
    }

    if (isForkable !== undefined) {
      params.append('is_forkable', isForkable.toString());
    }

    const queryString = params.toString();
    const url = `${this.erebusBaseUrl}/api/marketplace/arbfarm/cows${queryString ? `?${queryString}` : ''}`;

    try {
      const response = await fetch(url, {
        headers: this.walletAddress ? { 'x-wallet-address': this.walletAddress } : {},
      });
      const data = await response.json();

      return {
        success: response.ok,
        data: response.ok ? data.cows || data : undefined,
        error: response.ok ? undefined : data.message || 'Failed to list COWs',
        timestamp: new Date(),
      };
    } catch (error) {
      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date(),
      };
    }
  }

  async getCow(id: string): Promise<ArbFarmServiceResponse<ArbFarmCow>> {
    const url = `${this.erebusBaseUrl}/api/marketplace/arbfarm/cows/${id}`;

    try {
      const response = await fetch(url, {
        headers: this.walletAddress ? { 'x-wallet-address': this.walletAddress } : {},
      });
      const data = await response.json();

      return {
        success: response.ok,
        data: response.ok ? data : undefined,
        error: response.ok ? undefined : data.message || 'Failed to get COW',
        timestamp: new Date(),
      };
    } catch (error) {
      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date(),
      };
    }
  }

  async createCow(request: CreateArbFarmCowRequest): Promise<ArbFarmServiceResponse<ArbFarmCow>> {
    const url = `${this.erebusBaseUrl}/api/marketplace/arbfarm/cows`;

    try {
      const response = await fetch(url, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...(this.walletAddress ? { 'x-wallet-address': this.walletAddress } : {}),
        },
        body: JSON.stringify(request),
      });
      const data = await response.json();

      return {
        success: response.ok,
        data: response.ok ? data : undefined,
        error: response.ok ? undefined : data.message || 'Failed to create COW',
        timestamp: new Date(),
      };
    } catch (error) {
      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date(),
      };
    }
  }

  async forkCow(
    parentId: string,
    request: ForkArbFarmCowRequest,
  ): Promise<ArbFarmServiceResponse<{ cow: ArbFarmCow; fork: ArbFarmCowFork }>> {
    const url = `${this.erebusBaseUrl}/api/marketplace/arbfarm/cows/${parentId}/fork`;

    try {
      const response = await fetch(url, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...(this.walletAddress ? { 'x-wallet-address': this.walletAddress } : {}),
        },
        body: JSON.stringify(request),
      });
      const data = await response.json();

      return {
        success: response.ok,
        data: response.ok ? data : undefined,
        error: response.ok ? undefined : data.message || 'Failed to fork COW',
        timestamp: new Date(),
      };
    } catch (error) {
      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date(),
      };
    }
  }

  async getCowStrategies(cowId: string): Promise<ArbFarmServiceResponse<ArbFarmCowStrategy[]>> {
    const url = `${this.erebusBaseUrl}/api/marketplace/arbfarm/cows/${cowId}/strategies`;

    try {
      const response = await fetch(url, {
        headers: this.walletAddress ? { 'x-wallet-address': this.walletAddress } : {},
      });
      const data = await response.json();

      return {
        success: response.ok,
        data: response.ok ? data : undefined,
        error: response.ok ? undefined : data.message || 'Failed to get COW strategies',
        timestamp: new Date(),
      };
    } catch (error) {
      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date(),
      };
    }
  }

  async getCowForks(cowId: string): Promise<ArbFarmServiceResponse<ArbFarmCowFork[]>> {
    const url = `${this.erebusBaseUrl}/api/marketplace/arbfarm/cows/${cowId}/forks`;

    try {
      const response = await fetch(url, {
        headers: this.walletAddress ? { 'x-wallet-address': this.walletAddress } : {},
      });
      const data = await response.json();

      return {
        success: response.ok,
        data: response.ok ? data : undefined,
        error: response.ok ? undefined : data.message || 'Failed to get COW forks',
        timestamp: new Date(),
      };
    } catch (error) {
      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date(),
      };
    }
  }

  async getCowRevenue(cowId: string): Promise<ArbFarmServiceResponse<ArbFarmRevenue[]>> {
    const url = `${this.erebusBaseUrl}/api/marketplace/arbfarm/cows/${cowId}/revenue`;

    try {
      const response = await fetch(url, {
        headers: this.walletAddress ? { 'x-wallet-address': this.walletAddress } : {},
      });
      const data = await response.json();

      return {
        success: response.ok,
        data: response.ok ? data : undefined,
        error: response.ok ? undefined : data.message || 'Failed to get COW revenue',
        timestamp: new Date(),
      };
    } catch (error) {
      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date(),
      };
    }
  }

  async getEarnings(wallet: string): Promise<ArbFarmServiceResponse<ArbFarmEarningsSummary>> {
    const url = `${this.erebusBaseUrl}/api/marketplace/arbfarm/earnings/${wallet}`;

    try {
      const response = await fetch(url, {
        headers: this.walletAddress ? { 'x-wallet-address': this.walletAddress } : {},
      });
      const data = await response.json();

      return {
        success: response.ok,
        data: response.ok ? data : undefined,
        error: response.ok ? undefined : data.message || 'Failed to get earnings',
        timestamp: new Date(),
      };
    } catch (error) {
      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date(),
      };
    }
  }

  async getCowStats(): Promise<ArbFarmServiceResponse<ArbFarmCowStats>> {
    const url = `${this.erebusBaseUrl}/api/marketplace/arbfarm/stats`;

    try {
      const response = await fetch(url);
      const data = await response.json();

      return {
        success: response.ok,
        data: response.ok ? data : undefined,
        error: response.ok ? undefined : data.message || 'Failed to get COW stats',
        timestamp: new Date(),
      };
    } catch (error) {
      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date(),
      };
    }
  }

  // ============================================================================
  // Wallet Operations
  // ============================================================================

  async getWalletStatus(): Promise<ArbFarmServiceResponse<WalletStatus>> {
    return this.makeRequest('/wallet/status');
  }

  async setupWallet(
    userWalletAddress: string,
    walletName?: string,
  ): Promise<ArbFarmServiceResponse<WalletSetupResponse>> {
    return this.makeRequest('/wallet/setup', {
      method: 'POST',
      body: JSON.stringify({
        user_wallet_address: userWalletAddress,
        wallet_name: walletName,
      }),
    });
  }

  async updateWalletPolicy(
    policy: UpdatePolicyRequest,
  ): Promise<ArbFarmServiceResponse<{ success: boolean; policy: unknown }>> {
    return this.makeRequest('/wallet/policy', {
      method: 'POST',
      body: JSON.stringify(policy),
    });
  }

  async getWalletBalance(): Promise<ArbFarmServiceResponse<WalletBalanceResponse>> {
    return this.makeRequest('/wallet/balance');
  }

  async disconnectWallet(): Promise<ArbFarmServiceResponse<{ success: boolean; message: string }>> {
    return this.makeRequest('/wallet/disconnect', { method: 'POST' });
  }

  async getDailyUsage(): Promise<ArbFarmServiceResponse<DailyUsageResponse>> {
    return this.makeRequest('/wallet/usage');
  }

  async testSign(
    amountSol: number,
    description: string,
  ): Promise<ArbFarmServiceResponse<{ success: boolean; message?: string; violation?: unknown }>> {
    return this.makeRequest('/wallet/test-sign', {
      method: 'POST',
      body: JSON.stringify({
        amount_sol: amountSol,
        description,
      }),
    });
  }

  async getDevMode(): Promise<
    ArbFarmServiceResponse<{
      dev_mode_available: boolean;
      wallet_address: string | null;
      has_private_key: boolean;
    }>
  > {
    return this.makeRequest('/wallet/dev-mode');
  }

  async connectDevWallet(): Promise<
    ArbFarmServiceResponse<{
      success: boolean;
      message?: string;
      wallet_address?: string;
      status?: WalletStatus;
      error?: string;
    }>
  > {
    return this.makeRequest('/wallet/dev-connect', { method: 'POST' });
  }

  // ============================================================================
  // Capital Management Operations
  // ============================================================================

  async getCapitalUsage(): Promise<ArbFarmServiceResponse<CapitalUsageResponse>> {
    return this.makeRequest('/wallet/capital');
  }

  async syncCapitalBalance(): Promise<ArbFarmServiceResponse<SyncCapitalResponse>> {
    return this.makeRequest('/wallet/capital/sync', { method: 'POST' });
  }

  // ============================================================================
  // Settings Operations
  // ============================================================================

  async getAllSettings(): Promise<ArbFarmServiceResponse<AllSettingsResponse>> {
    return this.makeRequest('/settings');
  }

  async getRiskSettings(): Promise<ArbFarmServiceResponse<RiskSettingsResponse>> {
    return this.makeRequest('/settings/risk');
  }

  async updateRiskSettings(
    preset?: string,
    custom?: RiskConfig,
  ): Promise<ArbFarmServiceResponse<{ success: boolean; config?: RiskConfig; error?: string }>> {
    return this.makeRequest('/settings/risk', {
      method: 'POST',
      body: JSON.stringify({ preset, custom }),
    });
  }

  async getVenueSettings(): Promise<ArbFarmServiceResponse<{ venues: VenueConfig[] }>> {
    return this.makeRequest('/settings/venues');
  }

  async getApiKeyStatus(): Promise<ArbFarmServiceResponse<{ services: ApiKeyStatus[] }>> {
    return this.makeRequest('/settings/api-keys');
  }

  // ============================================================================
  // Approval Operations
  // ============================================================================

  async listApprovals(): Promise<
    ArbFarmServiceResponse<{
      approvals: Array<{
        id: string;
        edge_id?: string;
        strategy_id?: string;
        approval_type: string;
        status: string;
        expires_at?: string;
        hecate_decision?: boolean;
        hecate_reasoning?: string;
        user_decision?: boolean;
        user_decided_at?: string;
        created_at: string;
      }>;
      total: number;
    }>
  > {
    return this.makeRequest('/approvals');
  }

  async listPendingApprovals(): Promise<
    ArbFarmServiceResponse<{
      approvals: Array<{
        id: string;
        edge_id?: string;
        strategy_id?: string;
        approval_type: string;
        status: string;
        expires_at?: string;
        hecate_decision?: boolean;
        hecate_reasoning?: string;
        created_at: string;
      }>;
      total: number;
    }>
  > {
    return this.makeRequest('/approvals/pending');
  }

  async getApproval(id: string): Promise<
    ArbFarmServiceResponse<{
      id: string;
      edge_id?: string;
      strategy_id?: string;
      approval_type: string;
      status: string;
      expires_at?: string;
      hecate_decision?: boolean;
      hecate_reasoning?: string;
      user_decision?: boolean;
      user_decided_at?: string;
      created_at: string;
    }>
  > {
    return this.makeRequest(`/approvals/${id}`);
  }

  async approveApproval(
    id: string,
    notes?: string,
  ): Promise<ArbFarmServiceResponse<{ id: string; status: string }>> {
    return this.makeRequest(`/approvals/${id}/approve`, {
      method: 'POST',
      body: JSON.stringify({ notes }),
    });
  }

  async rejectApproval(
    id: string,
    reason: string,
  ): Promise<ArbFarmServiceResponse<{ id: string; status: string }>> {
    return this.makeRequest(`/approvals/${id}/reject`, {
      method: 'POST',
      body: JSON.stringify({ reason }),
    });
  }

  async getExecutionConfig(): Promise<
    ArbFarmServiceResponse<{
      auto_execution_enabled: boolean;
      default_approval_timeout_secs: number;
      notify_hecate_on_pending: boolean;
    }>
  > {
    return this.makeRequest('/execution/config');
  }

  async updateExecutionConfig(config: {
    auto_execution_enabled?: boolean;
    default_approval_timeout_secs?: number;
    notify_hecate_on_pending?: boolean;
  }): Promise<
    ArbFarmServiceResponse<{
      auto_execution_enabled: boolean;
      default_approval_timeout_secs: number;
      notify_hecate_on_pending: boolean;
    }>
  > {
    return this.makeRequest('/execution/config', {
      method: 'PUT',
      body: JSON.stringify(config),
    });
  }

  async toggleExecution(
    enabled: boolean,
  ): Promise<ArbFarmServiceResponse<{ enabled: boolean; message: string }>> {
    return this.makeRequest('/execution/toggle', {
      method: 'POST',
      body: JSON.stringify({ enabled }),
    });
  }

  async saveExecutionSettings(settings: {
    auto_execution_enabled?: boolean;
    auto_min_confidence?: number;
    auto_max_position_sol?: number;
    require_simulation?: boolean;
  }): Promise<ArbFarmServiceResponse<{
    auto_execution_enabled: boolean;
    auto_min_confidence: number;
    auto_max_position_sol: number;
    require_simulation: boolean;
  }>> {
    return this.makeRequest('/execution/config', {
      method: 'PUT',
      body: JSON.stringify(settings),
    });
  }

  // ============================================================================
  // Consensus Operations
  // ============================================================================

  async listConsensusHistory(
    limit?: number,
    offset?: number,
    approvedOnly?: boolean,
  ): Promise<
    ArbFarmServiceResponse<{
      decisions: Array<{
        id: string;
        edge_id: string;
        result: {
          approved: boolean;
          agreement_score: number;
          weighted_confidence: number;
          reasoning_summary: string;
          model_votes: Array<{
            model: string;
            approved: boolean;
            confidence: number;
            reasoning: string;
            latency_ms: number;
          }>;
        };
        edge_context: string;
        created_at: string;
      }>;
      total: number;
    }>
  > {
    const params = new URLSearchParams();
    if (limit) params.append('limit', limit.toString());
    if (offset) params.append('offset', offset.toString());
    if (approvedOnly !== undefined) params.append('approved_only', approvedOnly.toString());
    const queryString = params.toString();

    return this.makeRequest(`/consensus${queryString ? `?${queryString}` : ''}`);
  }

  async getConsensusDetail(consensusId: string): Promise<
    ArbFarmServiceResponse<{
      id: string;
      edge_id: string;
      approved: boolean;
      agreement_score: number;
      weighted_confidence: number;
      reasoning_summary: string;
      model_votes: Array<{
        model: string;
        approved: boolean;
        confidence: number;
        reasoning: string;
        latency_ms: number;
      }>;
      edge_context: string;
      total_latency_ms: number;
      created_at: string;
    }>
  > {
    return this.makeRequest(`/consensus/${consensusId}`);
  }

  async requestConsensus(params: {
    edge_type: string;
    venue: string;
    token_pair: string[];
    estimated_profit_lamports: number;
    risk_score: number;
    route_data: Record<string, unknown>;
    models?: string[];
  }): Promise<
    ArbFarmServiceResponse<{
      consensus_id: string;
      edge_id: string;
      approved: boolean;
      agreement_score: number;
      weighted_confidence: number;
      reasoning_summary: string;
      model_votes: Array<{
        model: string;
        approved: boolean;
        confidence: number;
        reasoning: string;
        latency_ms: number;
      }>;
      total_latency_ms: number;
    }>
  > {
    return this.makeRequest('/consensus/request', {
      method: 'POST',
      body: JSON.stringify(params),
    });
  }

  async getConsensusStats(): Promise<
    ArbFarmServiceResponse<{
      total_decisions: number;
      approved_count: number;
      rejected_count: number;
      average_agreement: number;
      average_confidence: number;
      average_latency_ms: number;
      decisions_last_24h: number;
    }>
  > {
    return this.makeRequest('/consensus/stats');
  }

  async listAvailableModels(): Promise<
    ArbFarmServiceResponse<{
      models: Array<{
        id: string;
        name: string;
        provider: string;
      }>;
      default_models: string[];
    }>
  > {
    return this.makeRequest('/consensus/models');
  }

  // ============================================================================
  // SSE Stream URLs (for direct EventSource connections)
  // ============================================================================

  getScannerStreamUrl(): string {
    return `${this.sseBaseUrl}/scanner/stream`;
  }

  getEdgesStreamUrl(): string {
    return `${this.sseBaseUrl}/edges/stream`;
  }

  getEventsStreamUrl(): string {
    return `${this.sseBaseUrl}/events/stream`;
  }

  getThreatStreamUrl(): string {
    return `${this.sseBaseUrl}/threat/stream`;
  }

  getHeliusStreamUrl(): string {
    return `${this.sseBaseUrl}/helius/stream`;
  }

  // ============================================================================
  // Helius Integration Operations
  // ============================================================================

  async getHeliusStatus(): Promise<ArbFarmServiceResponse<HeliusStatus>> {
    return this.makeRequest('/helius/status');
  }

  async getLaserStreamStatus(): Promise<ArbFarmServiceResponse<LaserStreamStatus>> {
    return this.makeRequest('/helius/laserstream');
  }

  async getPriorityFees(accountKeys?: string[]): Promise<ArbFarmServiceResponse<PriorityFees>> {
    const params = accountKeys ? `?account_keys=${accountKeys.join(',')}` : '';
    return this.makeRequest(`/helius/priority-fees${params}`);
  }

  async getCachedPriorityFees(): Promise<ArbFarmServiceResponse<PriorityFees | null>> {
    return this.makeRequest('/helius/priority-fees/cached');
  }

  async getSenderStats(): Promise<ArbFarmServiceResponse<SenderStats>> {
    return this.makeRequest('/helius/sender/stats');
  }

  async pingHeliusSender(): Promise<ArbFarmServiceResponse<{ latency_ms: number }>> {
    return this.makeRequest('/helius/sender/ping', { method: 'POST' });
  }

  async lookupTokenMetadata(mint: string): Promise<ArbFarmServiceResponse<TokenMetadata>> {
    return this.makeRequest('/helius/das/lookup', {
      method: 'POST',
      body: JSON.stringify({ mint }),
    });
  }

  async getAssetsByOwner(
    owner: string,
    page?: number,
    limit?: number,
  ): Promise<ArbFarmServiceResponse<{ assets: TokenMetadata[]; total: number }>> {
    const params = new URLSearchParams({ owner });
    if (page) params.append('page', page.toString());
    if (limit) params.append('limit', limit.toString());
    return this.makeRequest(`/helius/das/assets?${params.toString()}`);
  }

  async getHeliusConfig(): Promise<ArbFarmServiceResponse<HeliusConfig>> {
    return this.makeRequest('/helius/config');
  }

  async updateHeliusConfig(
    config: Partial<HeliusConfig>,
  ): Promise<ArbFarmServiceResponse<HeliusConfig>> {
    return this.makeRequest('/helius/config', {
      method: 'PUT',
      body: JSON.stringify(config),
    });
  }

  // ============================================================================
  // Research & Discovery Operations
  // ============================================================================

  async submitResearchUrl(
    url: string,
    context?: string,
    extractStrategy: boolean = true,
  ): Promise<ArbFarmServiceResponse<IngestUrlResponse>> {
    return this.makeRequest('/research/ingest', {
      method: 'POST',
      body: JSON.stringify({
        url,
        context,
        extract_strategy: extractStrategy,
      }),
    });
  }

  async listDiscoveries(
    status?: string,
    limit?: number,
    offset?: number,
  ): Promise<ArbFarmServiceResponse<DiscoveryListResponse>> {
    const params = new URLSearchParams();
    if (status) params.append('status', status);
    if (limit) params.append('limit', limit.toString());
    if (offset) params.append('offset', offset.toString());
    const queryString = params.toString();
    return this.makeRequest(`/research/discoveries${queryString ? `?${queryString}` : ''}`);
  }

  async getDiscovery(discoveryId: string): Promise<ArbFarmServiceResponse<ExtractedStrategy>> {
    return this.makeRequest(`/research/discoveries/${discoveryId}`);
  }

  async approveDiscovery(
    discoveryId: string,
    notes?: string,
  ): Promise<ArbFarmServiceResponse<{ message: string; status: string }>> {
    return this.makeRequest(`/research/discoveries/${discoveryId}/approve`, {
      method: 'POST',
      body: JSON.stringify({ notes }),
    });
  }

  async rejectDiscovery(
    discoveryId: string,
    reason: string,
  ): Promise<ArbFarmServiceResponse<{ message: string; status: string; reason: string }>> {
    return this.makeRequest(`/research/discoveries/${discoveryId}/reject`, {
      method: 'POST',
      body: JSON.stringify({ reason }),
    });
  }

  async runBacktest(
    strategyId: string,
    periodDays?: number,
    initialCapitalSol?: number,
    maxPositionSizeSol?: number,
  ): Promise<ArbFarmServiceResponse<{ message: string; strategy_id: string }>> {
    return this.makeRequest('/research/backtest', {
      method: 'POST',
      body: JSON.stringify({
        strategy_id: strategyId,
        period_days: periodDays,
        initial_capital_sol: initialCapitalSol,
        max_position_size_sol: maxPositionSizeSol,
      }),
    });
  }

  async getBacktestResult(
    backtestId: string,
  ): Promise<ArbFarmServiceResponse<{ status: string; result?: unknown }>> {
    return this.makeRequest(`/research/backtest/${backtestId}`);
  }

  async listSources(
    sourceType?: string,
    trackType?: string,
    activeOnly?: boolean,
  ): Promise<ArbFarmServiceResponse<{ sources: MonitoredSource[]; total: number }>> {
    const params = new URLSearchParams();
    if (sourceType) params.append('source_type', sourceType);
    if (trackType) params.append('track_type', trackType);
    if (activeOnly !== undefined) params.append('active_only', activeOnly.toString());
    const queryString = params.toString();
    return this.makeRequest(`/research/sources${queryString ? `?${queryString}` : ''}`);
  }

  async addSource(
    sourceType: string,
    handle: string,
    trackType: string,
    displayName?: string,
    keywords?: string[],
  ): Promise<ArbFarmServiceResponse<{ message: string; source: MonitoredSource }>> {
    return this.makeRequest('/research/sources', {
      method: 'POST',
      body: JSON.stringify({
        source_type: sourceType,
        handle,
        track_type: trackType,
        display_name: displayName,
        keywords,
      }),
    });
  }

  async deleteSource(sourceId: string): Promise<ArbFarmServiceResponse<{ message: string }>> {
    return this.makeRequest(`/research/sources/${sourceId}`, {
      method: 'DELETE',
    });
  }

  async toggleSource(
    sourceId: string,
    active: boolean,
  ): Promise<ArbFarmServiceResponse<{ message: string; active: boolean }>> {
    return this.makeRequest(`/research/sources/${sourceId}/toggle`, {
      method: 'POST',
      body: JSON.stringify({ active }),
    });
  }

  async listResearchAlerts(
    sourceId?: string,
    alertType?: string,
    limit?: number,
  ): Promise<ArbFarmServiceResponse<{ alerts: SocialAlert[]; total: number }>> {
    const params = new URLSearchParams();
    if (sourceId) params.append('source_id', sourceId);
    if (alertType) params.append('alert_type', alertType);
    if (limit) params.append('limit', limit.toString());
    const queryString = params.toString();
    return this.makeRequest(`/research/alerts${queryString ? `?${queryString}` : ''}`);
  }

  async getMonitorStats(): Promise<ArbFarmServiceResponse<MonitorStats>> {
    return this.makeRequest('/research/stats');
  }

  async monitorAccount(
    handle: string,
    trackType: string,
  ): Promise<ArbFarmServiceResponse<{ message: string; source: MonitoredSource }>> {
    return this.makeRequest('/research/monitor', {
      method: 'POST',
      body: JSON.stringify({ handle, track_type: trackType }),
    });
  }

  // ============================================================================
  // Utility Methods
  // ============================================================================

  formatProfitSol(lamports: number): string {
    const sol = lamports / 1_000_000_000;

    return sol.toFixed(4);
  }

  formatProfitDisplay(lamports: number): string {
    const sol = lamports / 1_000_000_000;
    const sign = sol >= 0 ? '+' : '';

    return `${sign}${sol.toFixed(4)} SOL`;
  }

  getRiskLevelFromScore(score: number): 'low' | 'medium' | 'high' | 'critical' {
    if (score <= 25) {
      return 'low';
    }

    if (score <= 50) {
      return 'medium';
    }

    if (score <= 75) {
      return 'high';
    }

    return 'critical';
  }

  getAtomicityLabel(level: string): string {
    switch (level) {
      case 'fully_atomic':
        return 'Guaranteed';
      case 'partially_atomic':
        return 'Partial';
      case 'non_atomic':
        return 'At Risk';
      default:
        return level;
    }
  }

  // ============================================================================
  // P&L Summary
  // ============================================================================

  async getPnLSummary(): Promise<ArbFarmServiceResponse<PnLSummary>> {
    return this.request<PnLSummary>('/positions/pnl-summary', {
      method: 'GET',
    });
  }

  async resetPnL(): Promise<ArbFarmServiceResponse<{ success: boolean; pnl_reset_at: string }>> {
    return this.request('/positions/pnl-reset', {
      method: 'POST',
    });
  }

  // ============================================================================
  // Learning Insights
  // ============================================================================

  async getLearningInsights(): Promise<ArbFarmServiceResponse<LearningInsightsResponse>> {
    return this.request<LearningInsightsResponse>('/engram/insights', {
      method: 'GET',
    });
  }

  // ============================================================================
  // Risk Level Configuration
  // ============================================================================

  async setRiskLevel(level: RiskLevel): Promise<ArbFarmServiceResponse<SetRiskLevelResponse>> {
    return this.request<SetRiskLevelResponse>('/config/risk', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ level }),
    });
  }

  async getRiskLevel(): Promise<ArbFarmServiceResponse<RiskLevelParams>> {
    return this.request<RiskLevelParams>('/config/risk', {
      method: 'GET',
    });
  }

  async setCustomRisk(config: {
    max_position_sol?: number;
    max_concurrent_positions?: number;
    daily_loss_limit_sol?: number;
    max_drawdown_percent?: number;
  }): Promise<ArbFarmServiceResponse<{ success: boolean; config?: RiskLevelParams; error?: string }>> {
    return this.request<{ success: boolean; config?: RiskLevelParams; error?: string }>('/config/risk/custom', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(config),
    });
  }

  // ============================================================================
  // Position Monitor Control
  // ============================================================================

  async startPositionMonitor(): Promise<ArbFarmServiceResponse<{ success: boolean; message: string }>> {
    return this.request<{ success: boolean; message: string }>('/positions/monitor/start', {
      method: 'POST',
    });
  }

  async stopPositionMonitor(): Promise<ArbFarmServiceResponse<{ success: boolean; message: string }>> {
    return this.request<{ success: boolean; message: string }>('/positions/monitor/stop', {
      method: 'POST',
    });
  }

  // ============================================================================
  // Position Management
  // ============================================================================

  async getPositions(): Promise<ArbFarmServiceResponse<PositionsResponse>> {
    return this.request<PositionsResponse>('/positions', {
      method: 'GET',
    });
  }

  async getPosition(id: string): Promise<ArbFarmServiceResponse<OpenPosition>> {
    return this.request<OpenPosition>(`/positions/${id}`, {
      method: 'GET',
    });
  }

  async closePosition(id: string, exitPercent?: number): Promise<ArbFarmServiceResponse<{ success: boolean; message: string; position_id: string }>> {
    return this.request<{ success: boolean; message: string; position_id: string }>(`/positions/${id}/close`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ exit_percent: exitPercent ?? 100 }),
    });
  }

  async updatePositionExitConfig(
    id: string,
    config: {
      stop_loss_percent?: number;
      take_profit_percent?: number;
      trailing_stop_percent?: number;
      time_limit_minutes?: number;
      preset?: string;
    }
  ): Promise<ArbFarmServiceResponse<{ success: boolean; position_id: string }>> {
    return this.request<{ success: boolean; position_id: string }>(`/positions/${id}/exit-config`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(config),
    });
  }

  async updateAllPositionsExitConfig(
    config: {
      stop_loss_percent?: number;
      take_profit_percent?: number;
      trailing_stop_percent?: number;
      time_limit_minutes?: number;
      preset?: string;
    }
  ): Promise<ArbFarmServiceResponse<{ updated_count: number }>> {
    return this.request<{ updated_count: number }>('/positions/exit-config', {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(config),
    });
  }

  async sellAllWalletTokens(): Promise<ArbFarmServiceResponse<{ success: boolean; results: Array<{ mint: string; success: boolean; error?: string }> }>> {
    return this.request<{ success: boolean; results: Array<{ mint: string; success: boolean; error?: string }> }>('/positions/sell-all', {
      method: 'POST',
    });
  }

  // ============================================================================
  // Position Exposure & History
  // ============================================================================

  async getPositionExposure(): Promise<ArbFarmServiceResponse<PositionExposure>> {
    return this.request<PositionExposure>('/positions/exposure', {
      method: 'GET',
    });
  }

  async getPositionHistory(
    page: number = 0,
    pageSize: number = 20
  ): Promise<ArbFarmServiceResponse<PositionHistoryResponse>> {
    return this.request<PositionHistoryResponse>(`/positions/history?page=${page}&page_size=${pageSize}`, {
      method: 'GET',
    });
  }

  async getMonitorStatus(): Promise<ArbFarmServiceResponse<MonitorStatus>> {
    return this.request<MonitorStatus>('/positions/monitor/status', {
      method: 'GET',
    });
  }

  async reconcilePositions(): Promise<ArbFarmServiceResponse<ReconcileResult>> {
    return this.request<ReconcileResult>('/positions/reconcile', {
      method: 'POST',
    });
  }

  // ============================================================================
  // Position Auto-Exit Controls
  // ============================================================================

  async togglePositionAutoExit(
    positionId: string,
    enabled: boolean
  ): Promise<ArbFarmServiceResponse<{ success: boolean; position_id: string; auto_exit_enabled: boolean; message: string }>> {
    return this.request(`/positions/${positionId}/auto-exit`, {
      method: 'PATCH',
      body: JSON.stringify({ enabled }),
    });
  }

  async getAutoExitStats(): Promise<ArbFarmServiceResponse<{ total_positions: number; auto_exit_enabled: number; manual_mode: number }>> {
    return this.request('/positions/auto-exit-stats', {
      method: 'GET',
    });
  }

  // ============================================================================
  // Extended Threat Management
  // ============================================================================

  async listWhitelisted(): Promise<ArbFarmServiceResponse<WhitelistedEntity[]>> {
    return this.request<WhitelistedEntity[]>('/threat/whitelist', {
      method: 'GET',
    });
  }

  async removeFromBlocklist(address: string): Promise<ArbFarmServiceResponse<{ success: boolean }>> {
    return this.request<{ success: boolean }>(`/threat/blocked/${address}`, {
      method: 'DELETE',
    });
  }

  async removeFromWhitelist(address: string): Promise<ArbFarmServiceResponse<{ success: boolean }>> {
    return this.request<{ success: boolean }>(`/threat/whitelist/${address}`, {
      method: 'DELETE',
    });
  }

  async listWatched(): Promise<ArbFarmServiceResponse<WatchedEntity[]>> {
    return this.request<WatchedEntity[]>('/threat/watch', {
      method: 'GET',
    });
  }

  async addWatch(mint: string, reason?: string): Promise<ArbFarmServiceResponse<WatchedEntity>> {
    return this.request<WatchedEntity>('/threat/watch', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ mint, reason }),
    });
  }

  async removeWatch(mint: string): Promise<ArbFarmServiceResponse<{ success: boolean }>> {
    return this.request<{ success: boolean }>(`/threat/watch/${mint}`, {
      method: 'DELETE',
    });
  }

  async getThreatHistory(mint: string): Promise<ArbFarmServiceResponse<ThreatHistoryItem[]>> {
    return this.request<ThreatHistoryItem[]>(`/threat/score/${mint}/history`, {
      method: 'GET',
    });
  }

  async getThreatStats(): Promise<ArbFarmServiceResponse<ThreatStats>> {
    return this.request<ThreatStats>('/threat/stats', {
      method: 'GET',
    });
  }

  // ============================================================================
  // Additional Curve Methods
  // ============================================================================

  async getCurveParameters(mint: string): Promise<ArbFarmServiceResponse<Record<string, unknown>>> {
    return this.request<Record<string, unknown>>(`/curves/${mint}/parameters`, {
      method: 'GET',
    });
  }

  // ============================================================================
  // Position Emergency Methods
  // ============================================================================

  async emergencyCloseAll(): Promise<ArbFarmServiceResponse<{ success: boolean; closed_count: number; errors: string[] }>> {
    return this.request<{ success: boolean; closed_count: number; errors: string[] }>('/positions/emergency-close', {
      method: 'POST',
    });
  }

  async cleanupExpiredApprovals(): Promise<ArbFarmServiceResponse<{ removed_count: number }>> {
    return this.request<{ removed_count: number }>('/approvals/cleanup', {
      method: 'POST',
    });
  }

  // ============================================================================
  // Autonomous Executor Methods
  // ============================================================================

  async getAutonomousExecutorStats(): Promise<ArbFarmServiceResponse<{
    is_running: boolean;
    total_executions: number;
    successful_executions: number;
    failed_executions: number;
    total_profit_sol: number;
    last_execution_at?: string;
  }>> {
    return this.request('/executor/stats', { method: 'GET' });
  }

  async listAutonomousExecutions(limit?: number): Promise<ArbFarmServiceResponse<Array<{
    id: string;
    edge_id: string;
    status: string;
    profit_sol?: number;
    executed_at: string;
  }>>> {
    const params = new URLSearchParams();
    if (limit) params.append('limit', limit.toString());
    const query = params.toString() ? `?${params.toString()}` : '';
    return this.request(`/executor/executions${query}`, { method: 'GET' });
  }

  async startAutonomousExecutor(): Promise<ArbFarmServiceResponse<{ success: boolean; message: string }>> {
    return this.request('/executor/start', { method: 'POST' });
  }

  async stopAutonomousExecutor(): Promise<ArbFarmServiceResponse<{ success: boolean; message: string }>> {
    return this.request('/executor/stop', { method: 'POST' });
  }

  async listBlocked(): Promise<ArbFarmServiceResponse<import('../../types/arbfarm').BlockedEntity[]>> {
    return this.request<import('../../types/arbfarm').BlockedEntity[]>('/threat/blocked', {
      method: 'GET',
    });
  }

  async isBlocked(address: string): Promise<ArbFarmServiceResponse<{ blocked: boolean }>> {
    return this.request<{ blocked: boolean }>(`/threat/blocked/${address}/status`, {
      method: 'GET',
    });
  }

  async isWhitelisted(address: string): Promise<ArbFarmServiceResponse<{ whitelisted: boolean }>> {
    return this.request<{ whitelisted: boolean }>(`/threat/whitelist/${address}/status`, {
      method: 'GET',
    });
  }

  async processSignals(signalIds: string[]): Promise<ArbFarmServiceResponse<{ processed: number; edges_created: number }>> {
    return this.request<{ processed: number; edges_created: number }>('/scanner/process', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ signal_ids: signalIds }),
    });
  }

  async setStrategyRiskProfile(id: string, profile: import('../../types/arbfarm').RiskParams): Promise<ArbFarmServiceResponse<import('../../types/arbfarm').Strategy>> {
    return this.request<import('../../types/arbfarm').Strategy>(`/strategies/${id}/risk-profile`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(profile),
    });
  }

  // ============================================================================
  // Consensus Config & Learning Methods
  // ============================================================================

  async getConsensusConfig(): Promise<ArbFarmServiceResponse<import('../../types/consensus').ConsensusConfigResponse>> {
    return this.request<import('../../types/consensus').ConsensusConfigResponse>('/consensus/config', {
      method: 'GET',
    });
  }

  async updateConsensusConfig(config: import('../../types/consensus').UpdateConsensusConfigRequest): Promise<ArbFarmServiceResponse<import('../../types/consensus').ConsensusConfigResponse>> {
    return this.request<import('../../types/consensus').ConsensusConfigResponse>('/consensus/config', {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(config),
    });
  }

  async resetConsensusConfig(): Promise<ArbFarmServiceResponse<import('../../types/consensus').ConsensusConfigResponse>> {
    return this.request<import('../../types/consensus').ConsensusConfigResponse>('/consensus/config/reset', {
      method: 'POST',
    });
  }

  async listConversations(limit?: number, topic?: string): Promise<ArbFarmServiceResponse<import('../../types/consensus').ConversationListResponse>> {
    const params = new URLSearchParams();
    if (limit) params.set('limit', limit.toString());
    if (topic) params.set('topic', topic);
    const query = params.toString() ? `?${params.toString()}` : '';
    return this.request<import('../../types/consensus').ConversationListResponse>(`/consensus/conversations${query}`, {
      method: 'GET',
    });
  }

  async getConversation(sessionId: string): Promise<ArbFarmServiceResponse<import('../../types/consensus').ConversationLog>> {
    return this.request<import('../../types/consensus').ConversationLog>(`/consensus/conversations/${sessionId}`, {
      method: 'GET',
    });
  }

  async listRecommendations(status?: string, limit?: number): Promise<ArbFarmServiceResponse<import('../../types/consensus').RecommendationListResponse>> {
    const params = new URLSearchParams();
    if (status) params.set('status', status);
    if (limit) params.set('limit', limit.toString());
    const query = params.toString() ? `?${params.toString()}` : '';
    return this.request<import('../../types/consensus').RecommendationListResponse>(`/consensus/recommendations${query}`, {
      method: 'GET',
    });
  }

  async updateRecommendationStatus(recommendationId: string, status: string): Promise<ArbFarmServiceResponse<{ success: boolean }>> {
    return this.request<{ success: boolean }>(`/consensus/recommendations/${recommendationId}/status`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ status }),
    });
  }

  async getLearningSummary(): Promise<ArbFarmServiceResponse<import('../../types/consensus').LearningSummary>> {
    return this.request<import('../../types/consensus').LearningSummary>('/consensus/learning', {
      method: 'GET',
    });
  }

  // ============================================================================
  // Engram Browser Methods
  // ============================================================================

  async listEngrams(filter?: import('../../types/engram').EngramBrowserFilter): Promise<ArbFarmServiceResponse<import('../../types/engram').EngramListResponse>> {
    const params = new URLSearchParams();
    if (filter?.engram_type) params.set('engram_type', filter.engram_type);
    if (filter?.tags && filter.tags.length > 0) params.set('tags', filter.tags.join(','));
    if (filter?.query) params.set('query', filter.query);
    if (filter?.limit) params.set('limit', filter.limit.toString());
    const query = params.toString() ? `?${params.toString()}` : '';
    return this.request<import('../../types/engram').EngramListResponse>(`/consensus/engrams${query}`, {
      method: 'GET',
    });
  }

  async getEngramByKey(key: string): Promise<ArbFarmServiceResponse<import('../../types/engram').Engram>> {
    return this.request<import('../../types/engram').Engram>(`/consensus/engrams/${encodeURIComponent(key)}`, {
      method: 'GET',
    });
  }

  async getAvailableModels(): Promise<ArbFarmServiceResponse<import('../../types/consensus').AvailableModelsResponse>> {
    return this.request<import('../../types/consensus').AvailableModelsResponse>('/consensus/models', {
      method: 'GET',
    });
  }

  async getConsensusHistory(options?: { limit?: number; offset?: number; approved_only?: boolean }): Promise<ArbFarmServiceResponse<import('../../types/consensus').ConsensusHistoryEntry[]>> {
    const params = new URLSearchParams();
    if (options?.limit) params.set('limit', options.limit.toString());
    if (options?.offset) params.set('offset', options.offset.toString());
    if (options?.approved_only !== undefined) params.set('approved_only', options.approved_only.toString());
    const query = params.toString() ? `?${params.toString()}` : '';
    return this.request<import('../../types/consensus').ConsensusHistoryEntry[]>(`/consensus${query}`, {
      method: 'GET',
    });
  }

  async getConsensusRecommendations(status?: string): Promise<ArbFarmServiceResponse<import('../../types/consensus').Recommendation[]>> {
    const params = new URLSearchParams();
    if (status) params.set('status', status);
    const query = params.toString() ? `?${params.toString()}` : '';
    return this.request<import('../../types/consensus').Recommendation[]>(`/consensus/recommendations${query}`, {
      method: 'GET',
    });
  }

  async getConsensusConversations(limit?: number): Promise<ArbFarmServiceResponse<import('../../types/consensus').ConversationLog[]>> {
    const params = new URLSearchParams();
    if (limit) params.set('limit', limit.toString());
    const query = params.toString() ? `?${params.toString()}` : '';
    return this.request<import('../../types/consensus').ConversationLog[]>(`/consensus/conversations${query}`, {
      method: 'GET',
    });
  }

  async getDiscoveredModels(): Promise<ArbFarmServiceResponse<import('../../types/consensus').DiscoveredModel[]>> {
    return this.request<import('../../types/consensus').DiscoveredModel[]>('/consensus/models/discovered', {
      method: 'GET',
    });
  }

  async refreshDiscoveredModels(): Promise<ArbFarmServiceResponse<{ success: boolean; count: number }>> {
    return this.request<{ success: boolean; count: number }>('/consensus/models/refresh', {
      method: 'POST',
    });
  }

  async getTradeAnalyses(limit?: number): Promise<ArbFarmServiceResponse<import('../../types/consensus').TradeAnalysisListResponse>> {
    const params = new URLSearchParams();
    if (limit) params.set('limit', limit.toString());
    const query = params.toString() ? `?${params.toString()}` : '';
    return this.request<import('../../types/consensus').TradeAnalysisListResponse>(`/consensus/trade-analyses${query}`, {
      method: 'GET',
    });
  }

  async getPatternSummary(): Promise<ArbFarmServiceResponse<import('../../types/consensus').PatternSummaryResponse>> {
    return this.request<import('../../types/consensus').PatternSummaryResponse>('/consensus/patterns', {
      method: 'GET',
    });
  }

  async getAnalysisSummary(): Promise<ArbFarmServiceResponse<import('../../types/consensus').AnalysisSummaryResponse>> {
    return this.request<import('../../types/consensus').AnalysisSummaryResponse>('/consensus/analysis-summary', {
      method: 'GET',
    });
  }

  async getConsensusSchedulerStatus(): Promise<ArbFarmServiceResponse<{ scheduler_enabled: boolean; last_queried: string | null }>> {
    return this.request<{ scheduler_enabled: boolean; last_queried: string | null }>('/consensus/scheduler', {
      method: 'GET',
    });
  }

  async toggleConsensusScheduler(enabled: boolean): Promise<ArbFarmServiceResponse<{ scheduler_enabled: boolean; last_queried: string | null }>> {
    return this.request<{ scheduler_enabled: boolean; last_queried: string | null }>('/consensus/scheduler', {
      method: 'POST',
      body: JSON.stringify({ enabled }),
    });
  }

  // Internet Research Methods
  async webSearch(query: string, options?: {
    numResults?: number;
    searchType?: 'search' | 'news';
    timeRange?: 'day' | 'week' | 'month' | 'year';
  }): Promise<ArbFarmServiceResponse<WebSearchResponse>> {
    return this.request<WebSearchResponse>('/mcp/call', {
      method: 'POST',
      body: JSON.stringify({
        name: 'web_search',
        arguments: {
          query,
          num_results: options?.numResults ?? 5,
          search_type: options?.searchType ?? 'search',
          ...(options?.timeRange && { time_range: options.timeRange }),
        },
      }),
    });
  }

  async webFetch(url: string, options?: {
    extractMode?: 'full' | 'article' | 'summary';
    maxLength?: number;
  }): Promise<ArbFarmServiceResponse<WebFetchResponse>> {
    return this.request<WebFetchResponse>('/mcp/call', {
      method: 'POST',
      body: JSON.stringify({
        name: 'web_fetch',
        arguments: {
          url,
          extract_mode: options?.extractMode ?? 'article',
          max_length: options?.maxLength ?? 10000,
        },
      }),
    });
  }

  async webSummarize(content: string, url: string, options?: {
    focus?: 'strategy' | 'alpha' | 'risk' | 'token_analysis' | 'general';
    saveAsEngram?: boolean;
  }): Promise<ArbFarmServiceResponse<WebSummarizeResponse>> {
    return this.request<WebSummarizeResponse>('/mcp/call', {
      method: 'POST',
      body: JSON.stringify({
        name: 'web_summarize',
        arguments: {
          content,
          url,
          focus: options?.focus ?? 'general',
          save_as_engram: options?.saveAsEngram ?? true,
        },
      }),
    });
  }

  async getWebResearch(limit?: number, focus?: string): Promise<ArbFarmServiceResponse<WebResearchListResponse>> {
    return this.request<WebResearchListResponse>('/mcp/call', {
      method: 'POST',
      body: JSON.stringify({
        name: 'web_research_list',
        arguments: {
          limit: limit ?? 20,
          ...(focus && { focus }),
        },
      }),
    });
  }
}

// Web Research Types
export interface WebSearchResult {
  title: string;
  url: string;
  snippet: string;
  position: number;
}

export interface WebSearchResponse {
  query: string;
  search_type: string;
  total_results: number;
  search_time_ms: number;
  results: WebSearchResult[];
}

export interface WebFetchResponse {
  id: string;
  url: string;
  title?: string;
  author?: string;
  content_type: string;
  word_count: number;
  extracted_tokens: string[];
  extracted_addresses: string[];
  content: string;
  fetched_at: string;
}

export interface ExtractedStrategyInsight {
  strategy_name?: string;
  description: string;
  entry_criteria: string[];
  exit_criteria: string[];
  risk_notes: string[];
  confidence: number;
}

export interface WebSummarizeResponse {
  research_id: string;
  url: string;
  source_type: string;
  analysis_focus: string;
  summary: string;
  key_insights: string[];
  extracted_strategies: ExtractedStrategyInsight[];
  extracted_tokens: string[];
  confidence: number;
  engram_saved: boolean;
  analyzed_at: string;
}

export interface WebResearchItem {
  research_id: string;
  source_url: string;
  source_type: string;
  title?: string;
  summary: string;
  analysis_focus: string;
  confidence: number;
  insights_count: number;
  strategies_count: number;
  tokens: string[];
  analyzed_at: string;
}

export interface WebResearchListResponse {
  wallet: string;
  total_count: number;
  research: WebResearchItem[];
}

export const arbFarmService = new ArbFarmService();

export default ArbFarmService;
