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
} from '../../types/arbfarm';

export interface ArbFarmServiceResponse<T = unknown> {
  success: boolean;
  data?: T;
  error?: string;
  timestamp: Date;
}

class ArbFarmService {
  private baseUrl: string;
  private isConnected: boolean = false;
  private walletAddress: string | null = null;

  constructor(baseUrl: string = import.meta.env.VITE_ARBFARM_API_URL || 'http://localhost:9007') {
    this.baseUrl = baseUrl;
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

      const responseJson = await response.json();
      const actualData =
        response.ok && responseJson.data !== undefined ? responseJson.data : responseJson;

      return {
        success: response.ok,
        data: response.ok ? actualData : undefined,
        error: response.ok
          ? undefined
          : responseJson.message || responseJson.error || 'Request failed',
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
    // Aggregate multiple endpoints for dashboard view
    try {
      const [statsRes, swarmRes, edgesRes, tradesRes, alertsRes] = await Promise.all([
        this.getTradeStats(),
        this.getSwarmHealth(),
        this.listEdges({ status: ['detected', 'pending_approval'], limit: 5 }),
        this.listTrades(5),
        this.getThreatAlerts(5),
      ]);

      if (!statsRes.success || !swarmRes.success) {
        throw new Error('Failed to fetch dashboard data');
      }

      const stats = statsRes.data!;
      const summary: DashboardSummary = {
        total_profit_sol: stats.net_profit_sol,
        today_profit_sol: 0, // Would need daily endpoint
        week_profit_sol: 0,
        win_rate: stats.win_rate,
        active_opportunities: edgesRes.data?.filter((e) => e.status === 'detected').length || 0,
        pending_approvals:
          edgesRes.data?.filter((e) => e.status === 'pending_approval').length || 0,
        executed_today: stats.successful_trades,
        swarm_health: swarmRes.data!,
        top_opportunities: edgesRes.data || [],
        recent_trades: tradesRes.data || [],
        recent_alerts: alertsRes.data || [],
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

  async getSignals(limit?: number): Promise<ArbFarmServiceResponse<Signal[]>> {
    const params = limit ? `?limit=${limit}` : '';

    return this.makeRequest(`/scanner/signals${params}`);
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
    return `${this.baseUrl}/scanner/stream`;
  }

  getEdgesStreamUrl(): string {
    return `${this.baseUrl}/edges/stream`;
  }

  getEventsStreamUrl(): string {
    return `${this.baseUrl}/events/stream`;
  }

  getThreatStreamUrl(): string {
    return `${this.baseUrl}/threat/stream`;
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

  getRiskLevel(score: number): 'low' | 'medium' | 'high' | 'critical' {
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
}

export const arbFarmService = new ArbFarmService();

export default ArbFarmService;
