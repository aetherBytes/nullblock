import { useState, useEffect, useCallback, useRef } from 'react';
import type {
  Edge,
  EdgeFilter,
  EdgeStatus,
  Trade,
  TradeStats,
  DailyStats,
  ScannerStatus,
  Signal,
  SwarmStatus,
  SwarmHealth,
  Strategy,
  ThreatScore,
  ThreatAlert,
  ThreatCategory,
  ThreatSeverity,
  BlockedEntity,
  KOL,
  KOLTrade,
  CopyTrade,
  CurveToken,
  GraduationCandidate,
  ArbEngram,
  DashboardSummary,
  ArbFarmCow,
  ArbFarmCowSummary,
  ArbFarmCowFork,
  ArbFarmEarningsSummary,
  ArbFarmCowStats,
  CreateArbFarmCowRequest,
  ForkArbFarmCowRequest,
} from '../../types/arbfarm';
import { arbFarmService } from '../services/arbfarm-service';

interface UseArbFarmOptions {
  pollInterval?: number;
  autoFetchDashboard?: boolean;
  autoFetchScanner?: boolean;
  autoFetchSwarm?: boolean;
}

interface UseArbFarmResult {
  dashboard: {
    summary: DashboardSummary | null;
    isLoading: boolean;
    error: string | null;
    refresh: () => Promise<void>;
  };

  edges: {
    data: Edge[];
    isLoading: boolean;
    error: string | null;
    filter: EdgeFilter;
    setFilter: (filter: EdgeFilter) => void;
    refresh: () => Promise<void>;
    approve: (id: string) => Promise<boolean>;
    reject: (id: string, reason: string) => Promise<boolean>;
    execute: (id: string, maxSlippageBps?: number) => Promise<boolean>;
    executeAuto: (
      id: string,
      slippageBps?: number,
    ) => Promise<{
      success: boolean;
      txSignature?: string;
      bundleId?: string;
      profitLamports?: number;
      executionTimeMs?: number;
      error?: string;
    }>;
  };

  trades: {
    data: Trade[];
    stats: TradeStats | null;
    dailyStats: DailyStats[];
    isLoading: boolean;
    error: string | null;
    refresh: () => Promise<void>;
    refreshStats: (period?: 'day' | 'week' | 'month' | 'all') => Promise<void>;
  };

  scanner: {
    status: ScannerStatus | null;
    signals: Signal[];
    isLoading: boolean;
    error: string | null;
    refresh: () => Promise<void>;
    start: () => Promise<boolean>;
    stop: () => Promise<boolean>;
  };

  swarm: {
    status: SwarmStatus | null;
    health: SwarmHealth | null;
    isLoading: boolean;
    error: string | null;
    refresh: () => Promise<void>;
    pause: () => Promise<boolean>;
    resume: () => Promise<boolean>;
  };

  strategies: {
    data: Strategy[];
    isLoading: boolean;
    error: string | null;
    refresh: () => Promise<void>;
    toggle: (id: string, enabled: boolean) => Promise<boolean>;
    create: (strategy: Omit<Strategy, 'id' | 'created_at' | 'updated_at'>) => Promise<boolean>;
    update: (id: string, updates: Partial<Strategy>) => Promise<boolean>;
    delete: (id: string) => Promise<boolean>;
  };

  threats: {
    alerts: ThreatAlert[];
    blocked: BlockedEntity[];
    isLoading: boolean;
    error: string | null;
    refresh: () => Promise<void>;
    checkToken: (mint: string) => Promise<ThreatScore | null>;
    report: (
      entityType: string,
      address: string,
      category: ThreatCategory,
      reason: string,
      severity: ThreatSeverity,
      evidenceUrl?: string,
    ) => Promise<boolean>;
    block: (
      entityType: string,
      address: string,
      category: string,
      reason: string,
    ) => Promise<boolean>;
    whitelist: (entityType: string, address: string, reason: string) => Promise<boolean>;
  };

  kols: {
    data: KOL[];
    isLoading: boolean;
    error: string | null;
    refresh: () => Promise<void>;
    add: (wallet: string, name?: string, twitterHandle?: string) => Promise<boolean>;
    remove: (id: string) => Promise<boolean>;
    enableCopy: (
      id: string,
      config: { maxPositionSol: number; delayMs: number },
    ) => Promise<boolean>;
    disableCopy: (id: string) => Promise<boolean>;
    getTrades: (id: string) => Promise<KOLTrade[]>;
    getCopyTrades: (id: string) => Promise<CopyTrade[]>;
  };

  curves: {
    tokens: CurveToken[];
    candidates: GraduationCandidate[];
    isLoading: boolean;
    error: string | null;
    refresh: () => Promise<void>;
    refreshCandidates: () => Promise<void>;
  };

  engrams: {
    data: ArbEngram[];
    isLoading: boolean;
    error: string | null;
    refresh: () => Promise<void>;
    search: (query: string, limit?: number) => Promise<ArbEngram[]>;
  };

  cows: {
    data: ArbFarmCowSummary[];
    selectedCow: ArbFarmCow | null;
    earnings: ArbFarmEarningsSummary | null;
    stats: ArbFarmCowStats | null;
    isLoading: boolean;
    error: string | null;
    refresh: () => Promise<void>;
    getCow: (id: string) => Promise<ArbFarmCow | null>;
    create: (request: CreateArbFarmCowRequest) => Promise<ArbFarmCow | null>;
    fork: (parentId: string, request: ForkArbFarmCowRequest) => Promise<{ cow: ArbFarmCow; fork: ArbFarmCowFork } | null>;
    refreshEarnings: (wallet: string) => Promise<void>;
    refreshStats: () => Promise<void>;
  };

  sse: {
    isConnected: boolean;
    lastEvent: { topic: string; payload: unknown } | null;
    connect: (topics: string[]) => void;
    disconnect: () => void;
  };
}

const DEFAULT_OPTIONS: UseArbFarmOptions = {
  pollInterval: 30000,
  autoFetchDashboard: true,
  autoFetchScanner: true,
  autoFetchSwarm: true,
};

export const useArbFarm = (options: UseArbFarmOptions = {}): UseArbFarmResult => {
  const opts = { ...DEFAULT_OPTIONS, ...options };

  const [dashboardSummary, setDashboardSummary] = useState<DashboardSummary | null>(null);
  const [dashboardLoading, setDashboardLoading] = useState(false);
  const [dashboardError, setDashboardError] = useState<string | null>(null);

  const [edges, setEdges] = useState<Edge[]>([]);
  const [edgeFilter, setEdgeFilter] = useState<EdgeFilter>({});
  const [edgesLoading, setEdgesLoading] = useState(false);
  const [edgesError, setEdgesError] = useState<string | null>(null);

  const [trades, setTrades] = useState<Trade[]>([]);
  const [tradeStats, setTradeStats] = useState<TradeStats | null>(null);
  const [dailyStats, setDailyStats] = useState<DailyStats[]>([]);
  const [tradesLoading, setTradesLoading] = useState(false);
  const [tradesError, setTradesError] = useState<string | null>(null);

  const [scannerStatus, setScannerStatus] = useState<ScannerStatus | null>(null);
  const [signals, _setSignals] = useState<Signal[]>([]);
  const [scannerLoading, setScannerLoading] = useState(false);
  const [scannerError, setScannerError] = useState<string | null>(null);

  const [swarmStatus, setSwarmStatus] = useState<SwarmStatus | null>(null);
  const [swarmHealth, setSwarmHealth] = useState<SwarmHealth | null>(null);
  const [swarmLoading, setSwarmLoading] = useState(false);
  const [swarmError, setSwarmError] = useState<string | null>(null);

  const [strategies, setStrategies] = useState<Strategy[]>([]);
  const [strategiesLoading, setStrategiesLoading] = useState(false);
  const [strategiesError, setStrategiesError] = useState<string | null>(null);

  const [threatAlerts, setThreatAlerts] = useState<ThreatAlert[]>([]);
  const [blockedEntities, setBlockedEntities] = useState<BlockedEntity[]>([]);
  const [threatsLoading, setThreatsLoading] = useState(false);
  const [threatsError, setThreatsError] = useState<string | null>(null);

  const [kols, setKols] = useState<KOL[]>([]);
  const [kolsLoading, setKolsLoading] = useState(false);
  const [kolsError, setKolsError] = useState<string | null>(null);

  const [curveTokens, setCurveTokens] = useState<CurveToken[]>([]);
  const [graduationCandidates, setGraduationCandidates] = useState<GraduationCandidate[]>([]);
  const [curvesLoading, setCurvesLoading] = useState(false);
  const [curvesError, setCurvesError] = useState<string | null>(null);

  const [arbEngrams, setArbEngrams] = useState<ArbEngram[]>([]);
  const [engramsLoading, setEngramsLoading] = useState(false);
  const [engramsError, setEngramsError] = useState<string | null>(null);

  const [cowsList, setCowsList] = useState<ArbFarmCowSummary[]>([]);
  const [selectedCow, setSelectedCow] = useState<ArbFarmCow | null>(null);
  const [cowEarnings, setCowEarnings] = useState<ArbFarmEarningsSummary | null>(null);
  const [cowStats, setCowStats] = useState<ArbFarmCowStats | null>(null);
  const [cowsLoading, setCowsLoading] = useState(false);
  const [cowsError, setCowsError] = useState<string | null>(null);

  const [sseConnected, setSseConnected] = useState(false);
  const [lastSseEvent, setLastSseEvent] = useState<{ topic: string; payload: unknown } | null>(
    null,
  );
  const eventSourceRef = useRef<EventSource | null>(null);
  const sseReconnectAttemptRef = useRef(0);

  const fetchDashboard = useCallback(async () => {
    setDashboardLoading(true);
    setDashboardError(null);
    try {
      const response = await arbFarmService.getDashboardSummary();

      if (response.success && response.data) {
        setDashboardSummary(response.data);
      } else {
        setDashboardError(response.error || 'Failed to fetch dashboard');
      }
    } catch (err) {
      setDashboardError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setDashboardLoading(false);
    }
  }, []);

  const fetchEdges = useCallback(async () => {
    setEdgesLoading(true);
    setEdgesError(null);
    try {
      const response = await arbFarmService.listEdges(edgeFilter);

      if (response.success && response.data) {
        // Handle both array format and { edges: [], total: N } format from backend
        const edgesData = Array.isArray(response.data)
          ? response.data
          : (response.data as { edges?: Edge[] }).edges || [];
        setEdges(edgesData);
      } else {
        setEdgesError(response.error || 'Failed to fetch edges');
      }
    } catch (err) {
      setEdgesError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setEdgesLoading(false);
    }
  }, [edgeFilter]);

  const approveEdge = useCallback(async (id: string): Promise<boolean> => {
    try {
      const response = await arbFarmService.approveEdge(id);

      if (response.success) {
        setEdges((prev) =>
          prev.map((e) =>
            e.id === id ? { ...e, status: 'pending_approval' as EdgeStatus } : e,
          ),
        );

        return true;
      }

      return false;
    } catch {
      return false;
    }
  }, []);

  const rejectEdge = useCallback(async (id: string, reason: string): Promise<boolean> => {
    try {
      const response = await arbFarmService.rejectEdge(id, reason);

      if (response.success) {
        setEdges((prev) =>
          prev.map((e) =>
            e.id === id ? { ...e, status: 'rejected' as EdgeStatus, rejection_reason: reason } : e,
          ),
        );

        return true;
      }

      return false;
    } catch {
      return false;
    }
  }, []);

  const executeEdge = useCallback(async (id: string, maxSlippageBps?: number): Promise<boolean> => {
    try {
      const response = await arbFarmService.executeEdge(id, maxSlippageBps);

      if (response.success) {
        setEdges((prev) =>
          prev.map((e) => (e.id === id ? { ...e, status: 'executing' as EdgeStatus } : e)),
        );

        return true;
      }

      return false;
    } catch {
      return false;
    }
  }, []);

  const executeEdgeAuto = useCallback(
    async (
      id: string,
      slippageBps?: number,
    ): Promise<{
      success: boolean;
      txSignature?: string;
      bundleId?: string;
      profitLamports?: number;
      executionTimeMs?: number;
      error?: string;
    }> => {
      try {
        setEdges((prev) =>
          prev.map((e) => (e.id === id ? { ...e, status: 'executing' as EdgeStatus } : e)),
        );

        const response = await arbFarmService.executeEdgeAuto(id, slippageBps);

        if (response.success && response.data) {
          const status = response.data.success ? ('executed' as EdgeStatus) : ('failed' as EdgeStatus);
          setEdges((prev) => prev.map((e) => (e.id === id ? { ...e, status } : e)));

          return {
            success: response.data.success,
            txSignature: response.data.tx_signature,
            bundleId: response.data.bundle_id,
            profitLamports: response.data.profit_lamports,
            executionTimeMs: response.data.execution_time_ms,
            error: response.data.error,
          };
        }

        setEdges((prev) =>
          prev.map((e) => (e.id === id ? { ...e, status: 'failed' as EdgeStatus } : e)),
        );

        return {
          success: false,
          error: response.error || 'Execution failed',
        };
      } catch (err) {
        setEdges((prev) =>
          prev.map((e) => (e.id === id ? { ...e, status: 'failed' as EdgeStatus } : e)),
        );

        return {
          success: false,
          error: err instanceof Error ? err.message : 'Unknown error',
        };
      }
    },
    [],
  );

  const fetchTrades = useCallback(async () => {
    setTradesLoading(true);
    setTradesError(null);
    try {
      const response = await arbFarmService.listTrades();

      if (response.success && response.data) {
        // Handle both array format and { trades: [], total: N } format from backend
        const tradesData = Array.isArray(response.data)
          ? response.data
          : (response.data as { trades?: Trade[] }).trades || [];
        setTrades(tradesData);
      } else {
        setTradesError(response.error || 'Failed to fetch trades');
      }
    } catch (err) {
      setTradesError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setTradesLoading(false);
    }
  }, []);

  const fetchTradeStats = useCallback(async (period: 'day' | 'week' | 'month' | 'all' = 'week') => {
    try {
      const [statsResponse, dailyResponse] = await Promise.all([
        arbFarmService.getTradeStats(period),
        arbFarmService.getDailyStats(30),
      ]);

      if (statsResponse.success && statsResponse.data) {
        setTradeStats(statsResponse.data);
      }

      if (dailyResponse.success && dailyResponse.data) {
        setDailyStats(dailyResponse.data);
      }
    } catch (err) {
      setTradesError(err instanceof Error ? err.message : 'Unknown error');
    }
  }, []);

  const fetchScannerStatus = useCallback(async () => {
    setScannerLoading(true);
    setScannerError(null);
    try {
      const response = await arbFarmService.getScannerStatus();

      if (response.success && response.data) {
        setScannerStatus(response.data);
      } else {
        setScannerError(response.error || 'Failed to fetch scanner status');
      }
    } catch (err) {
      setScannerError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setScannerLoading(false);
    }
  }, []);

  const startScanner = useCallback(async (): Promise<boolean> => {
    try {
      const response = await arbFarmService.startScanner();

      if (response.success) {
        await fetchScannerStatus();

        return true;
      }

      return false;
    } catch {
      return false;
    }
  }, [fetchScannerStatus]);

  const stopScanner = useCallback(async (): Promise<boolean> => {
    try {
      const response = await arbFarmService.stopScanner();

      if (response.success) {
        await fetchScannerStatus();

        return true;
      }

      return false;
    } catch {
      return false;
    }
  }, [fetchScannerStatus]);

  const fetchSwarmStatus = useCallback(async () => {
    setSwarmLoading(true);
    setSwarmError(null);
    try {
      const response = await arbFarmService.getSwarmStatus();

      if (response.success && response.data) {
        setSwarmStatus(response.data);
        setSwarmHealth(response.data.health);
      } else {
        setSwarmError(response.error || 'Failed to fetch swarm status');
      }
    } catch (err) {
      setSwarmError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setSwarmLoading(false);
    }
  }, []);

  const pauseSwarm = useCallback(async (): Promise<boolean> => {
    try {
      const response = await arbFarmService.pauseSwarm();

      if (response.success) {
        await fetchSwarmStatus();

        return true;
      }

      return false;
    } catch {
      return false;
    }
  }, [fetchSwarmStatus]);

  const resumeSwarm = useCallback(async (): Promise<boolean> => {
    try {
      const response = await arbFarmService.resumeSwarm();

      if (response.success) {
        await fetchSwarmStatus();

        return true;
      }

      return false;
    } catch {
      return false;
    }
  }, [fetchSwarmStatus]);

  const fetchStrategies = useCallback(async () => {
    setStrategiesLoading(true);
    setStrategiesError(null);
    try {
      const response = await arbFarmService.listStrategies();

      if (response.success && response.data) {
        // Handle both array format and { strategies: [], total: N } format from backend
        const strategiesData = Array.isArray(response.data)
          ? response.data
          : (response.data as { strategies?: Strategy[] }).strategies || [];
        setStrategies(strategiesData);
      } else {
        setStrategiesError(response.error || 'Failed to fetch strategies');
      }
    } catch (err) {
      setStrategiesError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setStrategiesLoading(false);
    }
  }, []);

  const toggleStrategy = useCallback(async (id: string, enabled: boolean): Promise<boolean> => {
    try {
      const response = await arbFarmService.toggleStrategy(id, enabled);

      if (response.success) {
        setStrategies((prev) =>
          prev.map((s) => (s.id === id ? { ...s, is_active: enabled } : s)),
        );

        return true;
      }

      return false;
    } catch {
      return false;
    }
  }, []);

  const createStrategy = useCallback(
    async (strategy: Omit<Strategy, 'id' | 'created_at' | 'updated_at'>): Promise<boolean> => {
      try {
        const response = await arbFarmService.createStrategy(strategy);

        if (response.success && response.data) {
          setStrategies((prev) => [...prev, response.data!]);

          return true;
        }

        return false;
      } catch {
        return false;
      }
    },
    [],
  );

  const updateStrategy = useCallback(
    async (id: string, updates: Partial<Strategy>): Promise<boolean> => {
      try {
        const response = await arbFarmService.updateStrategy(id, updates);

        if (response.success && response.data) {
          setStrategies((prev) => prev.map((s) => (s.id === id ? response.data! : s)));

          return true;
        }

        return false;
      } catch {
        return false;
      }
    },
    [],
  );

  const deleteStrategy = useCallback(async (id: string): Promise<boolean> => {
    try {
      const response = await arbFarmService.deleteStrategy(id);

      if (response.success) {
        setStrategies((prev) => prev.filter((s) => s.id !== id));

        return true;
      }

      return false;
    } catch {
      return false;
    }
  }, []);

  const fetchThreats = useCallback(async () => {
    setThreatsLoading(true);
    setThreatsError(null);
    try {
      const [alertsResponse, blockedResponse] = await Promise.all([
        arbFarmService.getThreatAlerts(),
        arbFarmService.listBlockedEntities(),
      ]);

      if (alertsResponse.success && alertsResponse.data) {
        setThreatAlerts(alertsResponse.data);
      }

      if (blockedResponse.success && blockedResponse.data) {
        setBlockedEntities(blockedResponse.data);
      }
    } catch (err) {
      setThreatsError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setThreatsLoading(false);
    }
  }, []);

  const checkTokenThreat = useCallback(async (mint: string): Promise<ThreatScore | null> => {
    try {
      const response = await arbFarmService.checkTokenThreat(mint);

      if (response.success && response.data) {
        return response.data;
      }

      return null;
    } catch {
      return null;
    }
  }, []);

  const reportThreat = useCallback(
    async (
      entityType: string,
      address: string,
      category: ThreatCategory,
      reason: string,
      severity: ThreatSeverity,
      evidenceUrl?: string,
    ): Promise<boolean> => {
      try {
        const response = await arbFarmService.reportThreat(
          entityType,
          address,
          category,
          reason,
          severity,
          evidenceUrl,
        );

        return response.success;
      } catch {
        return false;
      }
    },
    [],
  );

  const blockEntity = useCallback(
    async (
      entityType: string,
      address: string,
      category: string,
      reason: string,
    ): Promise<boolean> => {
      try {
        const response = await arbFarmService.blockEntity(entityType, address, category, reason);

        if (response.success) {
          await fetchThreats();

          return true;
        }

        return false;
      } catch {
        return false;
      }
    },
    [fetchThreats],
  );

  const whitelistEntity = useCallback(
    async (entityType: string, address: string, reason: string): Promise<boolean> => {
      try {
        const response = await arbFarmService.whitelistEntity(entityType, address, reason);

        return response.success;
      } catch {
        return false;
      }
    },
    [],
  );

  const fetchKOLs = useCallback(async () => {
    setKolsLoading(true);
    setKolsError(null);
    try {
      const response = await arbFarmService.listKOLs();

      if (response.success && response.data) {
        // Handle both array format and { kols: [], total: N } format from backend
        const kolsData = Array.isArray(response.data)
          ? response.data
          : (response.data as { kols?: KOL[] }).kols || [];
        setKols(kolsData);
      } else {
        setKolsError(response.error || 'Failed to fetch KOLs');
      }
    } catch (err) {
      setKolsError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setKolsLoading(false);
    }
  }, []);

  const addKOL = useCallback(
    async (wallet: string, name?: string, twitterHandle?: string): Promise<boolean> => {
      try {
        const response = await arbFarmService.addKOL(wallet, name, twitterHandle);

        if (response.success && response.data) {
          setKols((prev) => [...prev, response.data!]);

          return true;
        }

        return false;
      } catch {
        return false;
      }
    },
    [],
  );

  const removeKOL = useCallback(async (id: string): Promise<boolean> => {
    try {
      const response = await arbFarmService.removeKOL(id);

      if (response.success) {
        setKols((prev) => prev.filter((k) => k.id !== id));

        return true;
      }

      return false;
    } catch {
      return false;
    }
  }, []);

  const enableCopyTrading = useCallback(
    async (id: string, config: { maxPositionSol: number; delayMs: number }): Promise<boolean> => {
      try {
        const response = await arbFarmService.enableCopyTrading(id, {
          max_position_sol: config.maxPositionSol,
          delay_ms: config.delayMs,
        });

        if (response.success) {
          setKols((prev) =>
            prev.map((k) => (k.id === id ? { ...k, copy_trading_enabled: true } : k)),
          );

          return true;
        }

        return false;
      } catch {
        return false;
      }
    },
    [],
  );

  const disableCopyTrading = useCallback(async (id: string): Promise<boolean> => {
    try {
      const response = await arbFarmService.disableCopyTrading(id);

      if (response.success) {
        setKols((prev) =>
          prev.map((k) => (k.id === id ? { ...k, copy_trading_enabled: false } : k)),
        );

        return true;
      }

      return false;
    } catch {
      return false;
    }
  }, []);

  const getKOLTrades = useCallback(async (id: string): Promise<KOLTrade[]> => {
    try {
      const response = await arbFarmService.getKOLTrades(id);

      if (response.success && response.data) {
        return response.data;
      }

      return [];
    } catch {
      return [];
    }
  }, []);

  const getKOLCopyTrades = useCallback(async (id: string): Promise<CopyTrade[]> => {
    try {
      const response = await arbFarmService.getCopyHistory(id);

      if (response.success && response.data) {
        return response.data;
      }

      return [];
    } catch {
      return [];
    }
  }, []);

  const fetchCurves = useCallback(async () => {
    setCurvesLoading(true);
    setCurvesError(null);
    try {
      const response = await arbFarmService.listCurveTokens();

      if (response.success && response.data) {
        // Handle both array format and { tokens: [], total: N } format from backend
        const tokensData = Array.isArray(response.data)
          ? response.data
          : (response.data as { tokens?: CurveToken[] }).tokens || [];
        setCurveTokens(tokensData);
      } else {
        setCurvesError(response.error || 'Failed to fetch curve tokens');
      }
    } catch (err) {
      setCurvesError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setCurvesLoading(false);
    }
  }, []);

  const fetchGraduationCandidates = useCallback(async () => {
    try {
      const response = await arbFarmService.listGraduationCandidates();

      if (response.success && response.data) {
        setGraduationCandidates(response.data);
      }
    } catch (err) {
      console.error('Failed to fetch graduation candidates:', err);
    }
  }, []);

  const fetchEngrams = useCallback(async () => {
    setEngramsLoading(true);
    setEngramsError(null);
    try {
      const response = await arbFarmService.searchEngrams({});

      if (response.success && response.data) {
        setArbEngrams(response.data);
      } else {
        setEngramsError(response.error || 'Failed to fetch engrams');
      }
    } catch (err) {
      setEngramsError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setEngramsLoading(false);
    }
  }, []);

  const searchEngrams = useCallback(async (query: string, limit = 10): Promise<ArbEngram[]> => {
    try {
      const response = await arbFarmService.searchEngrams({ key_prefix: query, limit });

      if (response.success && response.data) {
        return response.data;
      }

      return [];
    } catch {
      return [];
    }
  }, []);

  const fetchCows = useCallback(async () => {
    setCowsLoading(true);
    setCowsError(null);
    try {
      const response = await arbFarmService.listCows(50, 0, true);

      if (response.success && response.data) {
        // Handle both array format and { cows: [], total_count: N } format from backend
        const cowsData = Array.isArray(response.data)
          ? response.data
          : (response.data as { cows?: ArbFarmCowSummary[] }).cows || [];
        setCowsList(cowsData);
      } else {
        setCowsError(response.error || 'Failed to fetch COWs');
      }
    } catch (err) {
      setCowsError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setCowsLoading(false);
    }
  }, []);

  const getCow = useCallback(async (id: string): Promise<ArbFarmCow | null> => {
    try {
      const response = await arbFarmService.getCow(id);

      if (response.success && response.data) {
        setSelectedCow(response.data);

        return response.data;
      }

      return null;
    } catch {
      return null;
    }
  }, []);

  const createCow = useCallback(
    async (request: CreateArbFarmCowRequest): Promise<ArbFarmCow | null> => {
      try {
        const response = await arbFarmService.createCow(request);

        if (response.success && response.data) {
          setCowsList((prev) => [
            {
              id: response.data!.id,
              listing_id: response.data!.listing_id,
              name: response.data!.name,
              description: response.data!.description,
              creator_wallet: response.data!.creator_wallet,
              strategy_count: response.data!.strategies.length,
              venue_types: response.data!.venue_types,
              risk_profile_type: response.data!.risk_profile.profile_type,
              fork_count: 0,
              total_profit_sol: 0,
              win_rate: 0,
              price_sol: request.price_sol,
              is_free: !request.price_sol,
              is_forkable: request.is_forkable,
              created_at: response.data!.created_at,
            },
            ...prev,
          ]);

          return response.data;
        }

        return null;
      } catch {
        return null;
      }
    },
    [],
  );

  const forkCow = useCallback(
    async (
      parentId: string,
      request: ForkArbFarmCowRequest,
    ): Promise<{ cow: ArbFarmCow; fork: ArbFarmCowFork } | null> => {
      try {
        const response = await arbFarmService.forkCow(parentId, request);

        if (response.success && response.data) {
          await fetchCows();

          return response.data;
        }

        return null;
      } catch {
        return null;
      }
    },
    [fetchCows],
  );

  const fetchCowEarnings = useCallback(async (wallet: string) => {
    try {
      const response = await arbFarmService.getEarnings(wallet);

      if (response.success && response.data) {
        setCowEarnings(response.data);
      }
    } catch (err) {
      console.error('Failed to fetch COW earnings:', err);
    }
  }, []);

  const fetchCowStats = useCallback(async () => {
    try {
      const response = await arbFarmService.getCowStats();

      if (response.success && response.data) {
        setCowStats(response.data);
      }
    } catch (err) {
      console.error('Failed to fetch COW stats:', err);
    }
  }, []);

  const connectSSE = useCallback(
    (topics: string[]) => {
      if (eventSourceRef.current) {
        eventSourceRef.current.close();
      }

      const url = arbFarmService.getEventsStreamUrl();
      const eventSource = new EventSource(url);

      eventSource.onopen = () => {
        setSseConnected(true);
        sseReconnectAttemptRef.current = 0;
      };

      eventSource.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);

          setLastSseEvent({ topic: data.topic, payload: data.payload });

          if (data.topic?.startsWith('arb.edge')) {
            const payload = data.payload as Record<string, unknown>;

            if (data.topic === 'arb.edge.executed' || data.topic === 'arb.edge.failed') {
              const solSpent = (payload.sol_spent as number) || 0;
              const newTrade: Trade = {
                id: payload.edge_id as string || `trade-${Date.now()}`,
                edge_id: payload.edge_id as string,
                strategy_id: payload.strategy_id as string,
                tx_signature: payload.signature as string,
                bundle_id: undefined,
                entry_price: solSpent,
                exit_price: undefined,
                profit_lamports: 0,
                profit_sol: 0,
                gas_cost_lamports: 0,
                slippage_bps: 0,
                executed_at: new Date().toISOString(),
              };
              // Synthetic trade is added optimistically; fetchTrades() (triggered by arb.trade.* event below)
              // does a full setTrades() replacement from DB, which automatically purges synthetics.
              setTrades((prev) => [newTrade, ...prev.filter(t => t.edge_id !== payload.edge_id)].slice(0, 100));
              fetchTradeStats();
            }

            fetchEdges();
          } else if (data.topic?.startsWith('arb.trade')) {
            fetchTrades();
            fetchTradeStats();
          } else if (data.topic?.startsWith('arb.scanner')) {
            fetchScannerStatus();
          } else if (data.topic?.startsWith('arb.swarm')) {
            fetchSwarmStatus();
          } else if (data.topic?.startsWith('arb.threat')) {
            fetchThreats();
          } else if (data.topic?.startsWith('arb.position')) {
            fetchDashboard();
            fetchTrades();
            fetchTradeStats();
          } else if (data.topic?.startsWith('arb.approval')) {
            fetchDashboard();
            fetchEdges();
          } else if (data.topic?.startsWith('arb.kol')) {
            fetchKOLs();
          }
        } catch (err) {
          console.error('Failed to parse SSE event:', err);
        }
      };

      eventSource.onerror = () => {
        setSseConnected(false);
        const attempt = sseReconnectAttemptRef.current;
        const delay = Math.min(2000 * Math.pow(2, attempt), 30000);
        sseReconnectAttemptRef.current = attempt + 1;
        setTimeout(() => {
          if (eventSourceRef.current === eventSource) {
            connectSSE(topics);
          }
        }, delay);
      };

      eventSourceRef.current = eventSource;
    },
    [fetchDashboard, fetchEdges, fetchTrades, fetchTradeStats, fetchScannerStatus, fetchSwarmStatus, fetchThreats, fetchKOLs],
  );

  const disconnectSSE = useCallback(() => {
    if (eventSourceRef.current) {
      eventSourceRef.current.close();
      eventSourceRef.current = null;
      setSseConnected(false);
    }
  }, []);

  useEffect(() => {
    if (opts.autoFetchDashboard) {
      fetchDashboard();
    }

    if (opts.autoFetchScanner) {
      fetchScannerStatus();
    }

    if (opts.autoFetchSwarm) {
      fetchSwarmStatus();
    }
  }, [
    opts.autoFetchDashboard,
    opts.autoFetchScanner,
    opts.autoFetchSwarm,
    fetchDashboard,
    fetchScannerStatus,
    fetchSwarmStatus,
  ]);

  useEffect(() => {
    if (!opts.pollInterval) {return;}

    const interval = setInterval(() => {
      if (opts.autoFetchDashboard) {fetchDashboard();}
      if (opts.autoFetchScanner) {fetchScannerStatus();}
      if (opts.autoFetchSwarm) {fetchSwarmStatus();}
    }, opts.pollInterval);

    return () => clearInterval(interval);
  }, [
    opts.pollInterval,
    opts.autoFetchDashboard,
    opts.autoFetchScanner,
    opts.autoFetchSwarm,
    fetchDashboard,
    fetchScannerStatus,
    fetchSwarmStatus,
  ]);

  useEffect(() => () => {
      disconnectSSE();
    }, [disconnectSSE]);

  return {
    dashboard: {
      summary: dashboardSummary,
      isLoading: dashboardLoading,
      error: dashboardError,
      refresh: fetchDashboard,
    },

    edges: {
      data: edges,
      isLoading: edgesLoading,
      error: edgesError,
      filter: edgeFilter,
      setFilter: setEdgeFilter,
      refresh: fetchEdges,
      approve: approveEdge,
      reject: rejectEdge,
      execute: executeEdge,
      executeAuto: executeEdgeAuto,
    },

    trades: {
      data: trades,
      stats: tradeStats,
      dailyStats,
      isLoading: tradesLoading,
      error: tradesError,
      refresh: fetchTrades,
      refreshStats: fetchTradeStats,
    },

    scanner: {
      status: scannerStatus,
      signals,
      isLoading: scannerLoading,
      error: scannerError,
      refresh: fetchScannerStatus,
      start: startScanner,
      stop: stopScanner,
    },

    swarm: {
      status: swarmStatus,
      health: swarmHealth,
      isLoading: swarmLoading,
      error: swarmError,
      refresh: fetchSwarmStatus,
      pause: pauseSwarm,
      resume: resumeSwarm,
    },

    strategies: {
      data: strategies,
      isLoading: strategiesLoading,
      error: strategiesError,
      refresh: fetchStrategies,
      toggle: toggleStrategy,
      create: createStrategy,
      update: updateStrategy,
      delete: deleteStrategy,
    },

    threats: {
      alerts: threatAlerts,
      blocked: blockedEntities,
      isLoading: threatsLoading,
      error: threatsError,
      refresh: fetchThreats,
      checkToken: checkTokenThreat,
      report: reportThreat,
      block: blockEntity,
      whitelist: whitelistEntity,
    },

    kols: {
      data: kols,
      isLoading: kolsLoading,
      error: kolsError,
      refresh: fetchKOLs,
      add: addKOL,
      remove: removeKOL,
      enableCopy: enableCopyTrading,
      disableCopy: disableCopyTrading,
      getTrades: getKOLTrades,
      getCopyTrades: getKOLCopyTrades,
    },

    curves: {
      tokens: curveTokens,
      candidates: graduationCandidates,
      isLoading: curvesLoading,
      error: curvesError,
      refresh: fetchCurves,
      refreshCandidates: fetchGraduationCandidates,
    },

    engrams: {
      data: arbEngrams,
      isLoading: engramsLoading,
      error: engramsError,
      refresh: fetchEngrams,
      search: searchEngrams,
    },

    cows: {
      data: cowsList,
      selectedCow,
      earnings: cowEarnings,
      stats: cowStats,
      isLoading: cowsLoading,
      error: cowsError,
      refresh: fetchCows,
      getCow,
      create: createCow,
      fork: forkCow,
      refreshEarnings: fetchCowEarnings,
      refreshStats: fetchCowStats,
    },

    sse: {
      isConnected: sseConnected,
      lastEvent: lastSseEvent,
      connect: connectSSE,
      disconnect: disconnectSSE,
    },
  };
};

export default useArbFarm;
