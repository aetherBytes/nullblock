import React, { useEffect, useState } from 'react';
import { useArbFarm } from '../../../common/hooks/useArbFarm';
import {
  EDGE_STATUS_COLORS,
  AGENT_HEALTH_COLORS,
  STRATEGY_TYPE_LABELS,
  RISK_PROFILE_LABELS,
  RISK_PROFILE_COLORS,
  VENUE_TYPE_ICONS,
  DELEGATION_STATUS_COLORS,
  DELEGATION_STATUS_LABELS,
} from '../../../types/arbfarm';
import type {
  Edge,
  ThreatAlert,
  ArbFarmCowSummary,
  WalletStatus,
  RiskConfig,
  RiskPreset,
  VenueConfig,
  ApiKeyStatus,
  HeliusStatus,
  LaserStreamStatus,
  PriorityFees,
  SenderStats,
  TokenMetadata,
  HeliusConfig,
  DiscoveredKol,
  KolDiscoveryStatus,
  CapitalUsageResponse,
} from '../../../types/arbfarm';
import {
  PRIORITY_LEVEL_LABELS,
  PRIORITY_LEVEL_COLORS,
} from '../../../types/arbfarm';
import { arbFarmService } from '../../../common/services/arbfarm-service';
import styles from './arbfarm.module.scss';
import EdgeCard from './components/EdgeCard';
import MetricCard from './components/MetricCard';
import SwarmStatusCard from './components/SwarmStatusCard';
import ThreatAlertCard from './components/ThreatAlertCard';
import TradeHistoryCard from './components/TradeHistoryCard';
import CurvePanel from './components/CurvePanel';
import HomeTab from './components/HomeTab';
import WipTab from './components/WipTab';

export type ArbFarmView =
  | 'dashboard'
  | 'opportunities'
  | 'signals'
  | 'strategies'
  | 'curves'
  | 'cows'
  | 'research'
  | 'kol-tracker'
  | 'threats'
  | 'helius'
  | 'settings';

interface ArbFarmDashboardProps {
  activeView: ArbFarmView;
  onViewChange: (view: ArbFarmView) => void;
}

const ArbFarmDashboard: React.FC<ArbFarmDashboardProps> = ({ activeView, onViewChange }) => {
  const { dashboard, edges, trades, scanner, swarm, strategies, threats, cows, kols, sse } = useArbFarm({
    pollInterval: 10000,
    autoFetchDashboard: true,
    autoFetchScanner: true,
    autoFetchSwarm: true,
  });

  const [opportunitiesFilter, setOpportunitiesFilter] = useState<string>('all');
  const [threatTokenInput, setThreatTokenInput] = useState('');
  const [threatCheckResult, setThreatCheckResult] = useState<any>(null);
  const [threatChecking, setThreatChecking] = useState(false);
  const [cowsFilter, setCowsFilter] = useState<'all' | 'mine' | 'forkable'>('all');
  const [selectedCowForFork, setSelectedCowForFork] = useState<ArbFarmCowSummary | null>(null);

  // KOL Tracker state
  const [showAddKolModal, setShowAddKolModal] = useState(false);
  const [newKolWallet, setNewKolWallet] = useState('');
  const [newKolName, setNewKolName] = useState('');
  const [newKolTwitter, setNewKolTwitter] = useState('');
  const [selectedKolId, setSelectedKolId] = useState<string | null>(null);
  const [kolTrades, setKolTrades] = useState<any[]>([]);
  const [kolTradesLoading, setKolTradesLoading] = useState(false);
  const [copyConfig, setCopyConfig] = useState<{ maxPosition: number; delay: number }>({
    maxPosition: 0.5,
    delay: 500,
  });

  // KOL Discovery state
  const [kolViewTab, setKolViewTab] = useState<'tracked' | 'discovery'>('tracked');
  const [discoveredKols, setDiscoveredKols] = useState<DiscoveredKol[]>([]);
  const [discoveryStatus, setDiscoveryStatus] = useState<KolDiscoveryStatus | null>(null);
  const [discoveryLoading, setDiscoveryLoading] = useState(false);
  const [scanningKols, setScanningKols] = useState(false);
  const [promotingWallet, setPromotingWallet] = useState<string | null>(null);

  // Research & Consensus state
  const [consensusDecisions, setConsensusDecisions] = useState<any[]>([]);
  const [consensusStats, setConsensusStats] = useState<any>(null);
  const [availableModels, setAvailableModels] = useState<any[]>([]);
  const [selectedConsensusId, setSelectedConsensusId] = useState<string | null>(null);
  const [consensusDetail, setConsensusDetail] = useState<any>(null);
  const [researchLoading, setResearchLoading] = useState(false);
  const [testConsensusLoading, setTestConsensusLoading] = useState(false);
  const [testConsensusResult, setTestConsensusResult] = useState<any>(null);

  // Research URL Injection state
  const [researchUrl, setResearchUrl] = useState('');
  const [researchUrlSubmitting, setResearchUrlSubmitting] = useState(false);
  const [researchResult, setResearchResult] = useState<any>(null);
  const [discoveries, setDiscoveries] = useState<any[]>([]);
  const [monitorSources, setMonitorSources] = useState<any[]>([]);
  const [monitorStats, setMonitorStats] = useState<any>(null);
  const [researchTab, setResearchTab] = useState<'inject' | 'discoveries' | 'sources' | 'consensus'>('inject');

  // Signals state
  const [signals, setSignals] = useState<any[]>([]);
  const [signalsLoading, setSignalsLoading] = useState(false);
  const [signalFilter, setSignalFilter] = useState<{
    signal_type?: string;
    venue_type?: string;
    min_profit_bps?: number;
    min_confidence?: number;
  }>({});
  const [liveSignals, setLiveSignals] = useState<any[]>([]);
  const [signalsLiveMode, setSignalsLiveMode] = useState(false);

  // Strategy creation state
  const [showCreateStrategy, setShowCreateStrategy] = useState(false);
  const [newStrategy, setNewStrategy] = useState({
    name: '',
    strategy_type: 'dex_arb',
    venue_types: ['dex_amm'] as string[],
    execution_mode: 'agent_directed' as 'autonomous' | 'agent_directed' | 'hybrid',
    risk_params: {
      max_position_sol: 1.0,
      daily_loss_limit_sol: 0.5,
      min_profit_bps: 50,
      max_slippage_bps: 100,
      max_risk_score: 50,
      require_simulation: true,
      auto_execute_atomic: true,
    },
  });
  const [creatingStrategy, setCreatingStrategy] = useState(false);

  // Strategy edit state
  const [showEditStrategy, setShowEditStrategy] = useState(false);
  const [editingStrategy, setEditingStrategy] = useState<any>(null);
  const [selectedStrategies, setSelectedStrategies] = useState<Set<string>>(new Set());
  const [savingToEngrams, setSavingToEngrams] = useState(false);
  const [resettingStats, setResettingStats] = useState<string | null>(null);

  // Settings state
  const [walletStatus, setWalletStatus] = useState<WalletStatus | null>(null);
  const [riskConfig, setRiskConfig] = useState<RiskConfig | null>(null);
  const [riskPresets, setRiskPresets] = useState<RiskPreset[]>([]);
  const [venues, setVenues] = useState<VenueConfig[]>([]);
  const [apiKeys, setApiKeys] = useState<ApiKeyStatus[]>([]);
  const [settingsLoading, setSettingsLoading] = useState(false);
  const [walletSetupAddress, setWalletSetupAddress] = useState('');
  const [settingsTab, setSettingsTab] = useState<'wallet' | 'risk' | 'venues' | 'api' | 'execution'>('wallet');
  const [customRiskEdit, setCustomRiskEdit] = useState<{
    max_position_sol: string;
    max_concurrent_positions: string;
    daily_loss_limit_sol: string;
  }>({ max_position_sol: '', max_concurrent_positions: '', daily_loss_limit_sol: '' });
  const [savingCustomRisk, setSavingCustomRisk] = useState(false);
  const [executionSettings, setExecutionSettings] = useState({
    auto_execute_enabled: false,
    auto_min_confidence: 0.8,
    auto_max_position_sol: 0.5,
    require_simulation: true,
  });

  // Dev wallet state
  const [devModeAvailable, setDevModeAvailable] = useState(false);
  const [devWalletAddress, setDevWalletAddress] = useState<string | null>(null);
  const [hasPrivateKey, setHasPrivateKey] = useState(false);
  const [connectingDevWallet, setConnectingDevWallet] = useState(false);

  // Capital management state
  const [capitalUsage, setCapitalUsage] = useState<CapitalUsageResponse | null>(null);
  const [capitalLoading, setCapitalLoading] = useState(false);
  const [syncingCapital, setSyncingCapital] = useState(false);

  // Helius state
  const [heliusStatus, setHeliusStatus] = useState<HeliusStatus | null>(null);
  const [laserStreamStatus, setLaserStreamStatus] = useState<LaserStreamStatus | null>(null);
  const [priorityFees, setPriorityFees] = useState<PriorityFees | null>(null);
  const [senderStats, setSenderStats] = useState<SenderStats | null>(null);
  const [heliusConfig, setHeliusConfig] = useState<HeliusConfig | null>(null);
  const [heliusLoading, setHeliusLoading] = useState(false);
  const [dasMintInput, setDasMintInput] = useState('');
  const [dasResult, setDasResult] = useState<TokenMetadata | null>(null);
  const [dasLoading, setDasLoading] = useState(false);
  const [pingResult, setPingResult] = useState<number | null>(null);
  const [pingLoading, setPingLoading] = useState(false);

  // Pending Approvals state
  const [pendingApprovals, setPendingApprovals] = useState<Array<{
    id: string;
    edge_id?: string;
    strategy_id?: string;
    approval_type: string;
    status: string;
    expires_at?: string;
    hecate_decision?: boolean;
    hecate_reasoning?: string;
    created_at: string;
  }>>([]);
  const [approvalsLoading, setApprovalsLoading] = useState(false);
  const [approvingId, setApprovingId] = useState<string | null>(null);
  const [rejectingId, setRejectingId] = useState<string | null>(null);

  // Execution Config state (from backend)
  const [executionConfig, setExecutionConfig] = useState<{
    auto_execution_enabled: boolean;
    default_approval_timeout_secs: number;
    notify_hecate_on_pending: boolean;
  } | null>(null);
  const [togglingExecution, setTogglingExecution] = useState(false);

  useEffect(() => {
    sse.connect(['arb.edge.*', 'arb.trade.*', 'arb.threat.*', 'arb.swarm.*', 'arb.helius.*', 'arb.approval.*']);

    return () => sse.disconnect();
  }, []);

  useEffect(() => {
    edges.refresh();
    trades.refresh();
    trades.refreshStats('week');
    strategies.refresh();
    threats.refresh();
    fetchPendingApprovals();
    fetchExecutionConfig();
  }, []);

  useEffect(() => {
    if (activeView === 'cows') {
      cows.refresh();
      cows.refreshStats();
    } else if (activeView === 'kol-tracker') {
      kols.refresh();
    } else if (activeView === 'research') {
      fetchConsensusData();
    } else if (activeView === 'helius') {
      fetchHeliusData();
    }
  }, [activeView]);

  const fetchHeliusData = async () => {
    setHeliusLoading(true);
    try {
      const [statusRes, laserRes, feesRes, senderRes, configRes] = await Promise.all([
        arbFarmService.getHeliusStatus(),
        arbFarmService.getLaserStreamStatus(),
        arbFarmService.getPriorityFees(),
        arbFarmService.getSenderStats(),
        arbFarmService.getHeliusConfig(),
      ]);

      if (statusRes.success && statusRes.data) {
        setHeliusStatus(statusRes.data);
      }
      if (laserRes.success && laserRes.data) {
        setLaserStreamStatus(laserRes.data);
      }
      if (feesRes.success && feesRes.data) {
        setPriorityFees(feesRes.data);
      }
      if (senderRes.success && senderRes.data) {
        setSenderStats(senderRes.data);
      }
      if (configRes.success && configRes.data) {
        setHeliusConfig(configRes.data);
      }
    } catch (err) {
      console.error('Failed to fetch Helius data:', err);
    } finally {
      setHeliusLoading(false);
    }
  };

  const handleDasLookup = async () => {
    if (!dasMintInput.trim()) return;
    setDasLoading(true);
    setDasResult(null);
    try {
      const res = await arbFarmService.lookupTokenMetadata(dasMintInput.trim());
      if (res.success && res.data) {
        setDasResult(res.data);
      }
    } catch (err) {
      console.error('DAS lookup failed:', err);
    } finally {
      setDasLoading(false);
    }
  };

  const handlePingSender = async () => {
    setPingLoading(true);
    setPingResult(null);
    try {
      const res = await arbFarmService.pingHeliusSender();
      if (res.success && res.data) {
        setPingResult(res.data.latency_ms);
      }
    } catch (err) {
      console.error('Ping failed:', err);
    } finally {
      setPingLoading(false);
    }
  };

  const fetchPendingApprovals = async () => {
    setApprovalsLoading(true);
    try {
      const res = await arbFarmService.listPendingApprovals();
      if (res.success && res.data) {
        setPendingApprovals(res.data.approvals || []);
      }
    } catch (err) {
      console.error('Failed to fetch pending approvals:', err);
    } finally {
      setApprovalsLoading(false);
    }
  };

  const fetchExecutionConfig = async () => {
    try {
      const res = await arbFarmService.getExecutionConfig();
      if (res.success && res.data) {
        setExecutionConfig(res.data);
        setExecutionSettings((prev) => ({
          ...prev,
          auto_execute_enabled: res.data?.auto_execution_enabled || false,
        }));
      }
    } catch (err) {
      console.error('Failed to fetch execution config:', err);
    }
  };

  const handleApproveApproval = async (id: string) => {
    setApprovingId(id);
    try {
      const res = await arbFarmService.approveApproval(id);
      if (res.success) {
        fetchPendingApprovals();
        edges.refresh();
      }
    } catch (err) {
      console.error('Failed to approve:', err);
    } finally {
      setApprovingId(null);
    }
  };

  const handleRejectApproval = async (id: string, reason: string) => {
    setRejectingId(id);
    try {
      const res = await arbFarmService.rejectApproval(id, reason);
      if (res.success) {
        fetchPendingApprovals();
        edges.refresh();
      }
    } catch (err) {
      console.error('Failed to reject:', err);
    } finally {
      setRejectingId(null);
    }
  };

  const handleToggleExecution = async (enabled: boolean) => {
    setTogglingExecution(true);
    try {
      const res = await arbFarmService.toggleExecution(enabled);
      if (res.success) {
        setExecutionConfig((prev) => prev ? { ...prev, auto_execution_enabled: enabled } : null);
        setExecutionSettings((prev) => ({ ...prev, auto_execute_enabled: enabled }));
      }
    } catch (err) {
      console.error('Failed to toggle execution:', err);
    } finally {
      setTogglingExecution(false);
    }
  };

  const handleUpdateHeliusConfig = async (update: Partial<HeliusConfig>) => {
    try {
      const res = await arbFarmService.updateHeliusConfig(update);
      if (res.success && res.data) {
        setHeliusConfig(res.data);
      }
    } catch (err) {
      console.error('Failed to update Helius config:', err);
    }
  };

  const fetchConsensusData = async () => {
    setResearchLoading(true);
    try {
      const [historyRes, statsRes, modelsRes] = await Promise.all([
        arbFarmService.listConsensusHistory(20),
        arbFarmService.getConsensusStats(),
        arbFarmService.listAvailableModels(),
      ]);

      if (historyRes.success && historyRes.data) {
        setConsensusDecisions(historyRes.data.decisions || []);
      }
      if (statsRes.success && statsRes.data) {
        setConsensusStats(statsRes.data);
      }
      if (modelsRes.success && modelsRes.data) {
        setAvailableModels(modelsRes.data.models || []);
      }
    } catch (err) {
      console.error('Failed to fetch consensus data:', err);
    } finally {
      setResearchLoading(false);
    }
  };

  const handleTestConsensus = async () => {
    setTestConsensusLoading(true);
    setTestConsensusResult(null);
    try {
      const res = await arbFarmService.requestConsensus({
        edge_type: 'Arbitrage',
        venue: 'jupiter',
        token_pair: ['SOL', 'USDC'],
        estimated_profit_lamports: 50000000,
        risk_score: 25,
        route_data: { test: true },
      });
      if (res.success && res.data) {
        setTestConsensusResult(res.data);
        fetchConsensusData();
      }
    } catch (err) {
      console.error('Consensus test failed:', err);
    } finally {
      setTestConsensusLoading(false);
    }
  };

  const handleViewConsensusDetail = async (consensusId: string) => {
    setSelectedConsensusId(consensusId);
    try {
      const res = await arbFarmService.getConsensusDetail(consensusId);
      if (res.success && res.data) {
        setConsensusDetail(res.data);
      }
    } catch (err) {
      console.error('Failed to fetch consensus detail:', err);
    }
  };

  const handleSubmitResearchUrl = async () => {
    if (!researchUrl.trim()) return;
    setResearchUrlSubmitting(true);
    setResearchResult(null);
    try {
      const res = await arbFarmService.submitResearchUrl(researchUrl.trim());
      if (res.success && res.data) {
        setResearchResult(res.data);
        if (res.data.extracted_strategy) {
          fetchDiscoveries();
        }
      }
    } catch (err) {
      console.error('Research URL submission failed:', err);
    } finally {
      setResearchUrlSubmitting(false);
    }
  };

  const fetchDiscoveries = async () => {
    try {
      const res = await arbFarmService.listDiscoveries();
      if (res.success && res.data) {
        setDiscoveries(res.data.discoveries || []);
      }
    } catch (err) {
      console.error('Failed to fetch discoveries:', err);
    }
  };

  const fetchSignals = async () => {
    setSignalsLoading(true);
    try {
      const res = await arbFarmService.getSignals({
        limit: 50,
        ...signalFilter,
      });
      if (res.success && res.data) {
        setSignals(res.data);
      }
    } catch (err) {
      console.error('Failed to fetch signals:', err);
    } finally {
      setSignalsLoading(false);
    }
  };

  const toggleSignalsLiveMode = () => {
    setSignalsLiveMode(!signalsLiveMode);
    if (!signalsLiveMode) {
      setLiveSignals([]);
    }
  };

  useEffect(() => {
    if (activeView === 'signals') {
      fetchSignals();
    }
  }, [activeView, signalFilter]);

  useEffect(() => {
    if (activeView === 'signals' && signalsLiveMode) {
      const unsubscribe = arbFarmService.subscribeToSignals((signal) => {
        setLiveSignals((prev) => [signal, ...prev].slice(0, 100));
      });
      return () => unsubscribe();
    }
  }, [activeView, signalsLiveMode]);

  const fetchResearchSources = async () => {
    try {
      const [sourcesRes, statsRes] = await Promise.all([
        arbFarmService.listSources(),
        arbFarmService.getMonitorStats(),
      ]);
      if (sourcesRes.success && sourcesRes.data) {
        setMonitorSources(sourcesRes.data.sources || []);
      }
      if (statsRes.success && statsRes.data) {
        setMonitorStats(statsRes.data);
      }
    } catch (err) {
      console.error('Failed to fetch research sources:', err);
    }
  };

  const handleCreateStrategyFromResearch = async (extracted: any) => {
    setNewStrategy({
      name: extracted.name || 'Extracted Strategy',
      strategy_type: extracted.strategy_type?.toLowerCase() || 'custom',
      venue_types: ['dex_amm'],
      execution_mode: 'agent_directed',
      risk_params: {
        max_position_sol: extracted.risk_params?.max_position_sol || 1.0,
        daily_loss_limit_sol: extracted.risk_params?.daily_loss_limit_sol || 0.5,
        min_profit_bps: extracted.risk_params?.min_profit_bps || 50,
        max_slippage_bps: extracted.risk_params?.max_slippage_bps || 100,
        max_risk_score: 50,
        require_simulation: true,
        auto_execute_atomic: true,
      },
    });
    setShowCreateStrategy(true);
  };

  const handleRunBacktest = async (extracted: any) => {
    try {
      const res = await arbFarmService.runBacktest(extracted.id);
      if (res.success) {
        console.log('Backtest queued:', res.data);
      }
    } catch (err) {
      console.error('Failed to queue backtest:', err);
    }
  };

  const handleCreateStrategy = async () => {
    if (!newStrategy.name.trim()) return;
    setCreatingStrategy(true);
    try {
      const res = await arbFarmService.createStrategy({
        ...newStrategy,
        is_active: false,
      });
      if (res.success) {
        setShowCreateStrategy(false);
        setNewStrategy({
          name: '',
          strategy_type: 'dex_arb',
          venue_types: ['dex_amm'],
          execution_mode: 'agent_directed',
          risk_params: {
            max_position_sol: 1.0,
            daily_loss_limit_sol: 0.5,
            min_profit_bps: 50,
            max_slippage_bps: 100,
            max_risk_score: 50,
            require_simulation: true,
            auto_execute_atomic: true,
          },
        });
        strategies.refetch();
      }
    } catch (err) {
      console.error('Failed to create strategy:', err);
    } finally {
      setCreatingStrategy(false);
    }
  };

  const handleVenueToggle = (venue: string) => {
    setNewStrategy((prev) => ({
      ...prev,
      venue_types: prev.venue_types.includes(venue)
        ? prev.venue_types.filter((v) => v !== venue)
        : [...prev.venue_types, venue],
    }));
  };

  const handleEditStrategy = (strategy: any) => {
    setEditingStrategy({ ...strategy });
    setShowEditStrategy(true);
  };

  const handleDeleteStrategy = async (strategyId: string, strategyName: string) => {
    if (!window.confirm(`Are you sure you want to PERMANENTLY DELETE strategy "${strategyName}"? This removes all data and cannot be undone.`)) {
      return;
    }
    try {
      await arbFarmService.deleteStrategy(strategyId);
      strategies.refresh();
    } catch (err) {
      console.error('Failed to delete strategy:', err);
    }
  };

  const handleKillStrategy = async (strategyId: string, strategyName: string) => {
    if (!window.confirm(`Kill strategy "${strategyName}"? This will immediately stop all running operations but keep the strategy in your list.`)) {
      return;
    }
    try {
      const res = await arbFarmService.killStrategy(strategyId);
      if (res.success) {
        strategies.refresh();
        fetchPendingApprovals();
      }
    } catch (err) {
      console.error('Failed to kill strategy:', err);
    }
  };

  const handleUpdateStrategy = async () => {
    if (!editingStrategy) return;
    try {
      const res = await arbFarmService.updateStrategy(editingStrategy.id, {
        name: editingStrategy.name,
        venue_types: editingStrategy.venue_types,
        execution_mode: editingStrategy.execution_mode,
        risk_params: editingStrategy.risk_params,
      });
      if (res.success) {
        setShowEditStrategy(false);
        setEditingStrategy(null);
        strategies.refetch();
      }
    } catch (err) {
      console.error('Failed to update strategy:', err);
    }
  };

  const handleEditVenueToggle = (venue: string) => {
    if (!editingStrategy) return;
    setEditingStrategy((prev: any) => ({
      ...prev,
      venue_types: prev.venue_types.includes(venue)
        ? prev.venue_types.filter((v: string) => v !== venue)
        : [...prev.venue_types, venue],
    }));
  };

  const handleStrategySelection = (id: string) => {
    setSelectedStrategies((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  };

  const handleSelectAllStrategies = () => {
    if (selectedStrategies.size === (strategies.data || []).length) {
      setSelectedStrategies(new Set());
    } else {
      setSelectedStrategies(new Set((strategies.data || []).map((s) => s.id)));
    }
  };

  const handleBatchToggle = async (enabled: boolean) => {
    if (selectedStrategies.size === 0) return;
    try {
      const res = await arbFarmService.batchToggleStrategies(
        Array.from(selectedStrategies),
        enabled,
      );
      if (res.success) {
        setSelectedStrategies(new Set());
        strategies.refetch();
      }
    } catch (err) {
      console.error('Batch toggle failed:', err);
    }
  };

  const handleSaveToEngrams = async () => {
    setSavingToEngrams(true);
    try {
      const res = await arbFarmService.saveStrategiesToEngrams();
      if (res.success) {
        console.log('Strategies saved to engrams:', res.data);
      }
    } catch (err) {
      console.error('Failed to save strategies to engrams:', err);
    } finally {
      setSavingToEngrams(false);
    }
  };

  const handleResetStrategyStats = async (id: string) => {
    setResettingStats(id);
    try {
      const res = await arbFarmService.resetStrategyStats(id);
      if (res.success) {
        strategies.refetch();
      }
    } catch (err) {
      console.error('Failed to reset strategy stats:', err);
    } finally {
      setResettingStats(null);
    }
  };

  useEffect(() => {
    if (activeView === 'research') {
      fetchDiscoveries();
      fetchResearchSources();
    }
  }, [activeView]);

  useEffect(() => {
    const fetchKolTrades = async () => {
      if (selectedKolId) {
        setKolTradesLoading(true);
        try {
          const trades = await kols.getTrades(selectedKolId);
          setKolTrades(trades || []);
        } catch (err) {
          console.error('Failed to fetch KOL trades:', err);
        } finally {
          setKolTradesLoading(false);
        }
      }
    };
    fetchKolTrades();
  }, [selectedKolId]);

  useEffect(() => {
    const fetchDiscoveryData = async () => {
      if (activeView === 'kol-tracker' && kolViewTab === 'discovery') {
        await Promise.all([fetchDiscoveryStatus(), fetchDiscoveredKols()]);
      }
    };
    fetchDiscoveryData();
  }, [activeView, kolViewTab]);

  useEffect(() => {
    const fetchSettings = async () => {
      if (activeView === 'settings') {
        setSettingsLoading(true);
        try {
          const [settingsRes, walletRes, devModeRes] = await Promise.all([
            arbFarmService.getAllSettings(),
            arbFarmService.getWalletStatus(),
            arbFarmService.getDevMode(),
          ]);

          if (settingsRes.success && settingsRes.data) {
            setRiskConfig(settingsRes.data.risk);
            setRiskPresets(settingsRes.data.risk_presets);
            setVenues(settingsRes.data.venues);
            setApiKeys(settingsRes.data.api_keys);
          }

          if (walletRes.success && walletRes.data) {
            setWalletStatus(walletRes.data);
          }

          if (devModeRes.success && devModeRes.data) {
            setDevModeAvailable(devModeRes.data.dev_mode_available);
            setDevWalletAddress(devModeRes.data.wallet_address);
            setHasPrivateKey(devModeRes.data.has_private_key);
          }
        } catch (err) {
          console.error('Failed to fetch settings:', err);
        } finally {
          setSettingsLoading(false);
        }
      }
    };

    fetchSettings();
  }, [activeView]);

  useEffect(() => {
    const fetchCapital = async () => {
      if (activeView === 'dashboard') {
        setCapitalLoading(true);
        try {
          const res = await arbFarmService.getCapitalUsage();
          if (res.success && res.data) {
            setCapitalUsage(res.data);
          }
        } catch (err) {
          console.error('Failed to fetch capital usage:', err);
        } finally {
          setCapitalLoading(false);
        }
      }
    };

    fetchCapital();
  }, [activeView]);

  const handleSyncCapital = async () => {
    setSyncingCapital(true);
    try {
      const res = await arbFarmService.syncCapitalBalance();
      if (res.success && res.data) {
        const capitalRes = await arbFarmService.getCapitalUsage();
        if (capitalRes.success && capitalRes.data) {
          setCapitalUsage(capitalRes.data);
        }
      }
    } catch (err) {
      console.error('Failed to sync capital:', err);
    } finally {
      setSyncingCapital(false);
    }
  };

  const formatSol = (lamports: number): string => {
    const sol = lamports / 1_000_000_000;

    return sol.toFixed(4);
  };

  const formatProfit = (sol: number): string => {
    const prefix = sol >= 0 ? '+' : '';

    return `${prefix}${sol.toFixed(4)} SOL`;
  };

  const getHealthColor = (health: string): string =>
    AGENT_HEALTH_COLORS[health as keyof typeof AGENT_HEALTH_COLORS] || '#6b7280';

  const getSignalTypeLabel = (type: string): string => {
    const labels: Record<string, string> = {
      price_discrepancy: 'Price Gap',
      volume_spike: 'Volume Spike',
      liquidity_change: 'Liquidity Change',
      new_token: 'New Token',
      curve_graduation: 'Curve Graduation',
      large_order: 'Large Order',
      liquidation: 'Liquidation',
      pool_imbalance: 'Pool Imbalance',
      dex_arb: 'DEX Arb',
      jit_liquidity: 'JIT Liquidity',
      backrun: 'Backrun',
    };
    return labels[type] || type;
  };

  const getSignalTypeIcon = (type: string): string => {
    const icons: Record<string, string> = {
      price_discrepancy: '📊',
      volume_spike: '📈',
      liquidity_change: '💧',
      new_token: '✨',
      curve_graduation: '🎓',
      large_order: '🐋',
      liquidation: '⚡',
      pool_imbalance: '⚖️',
      dex_arb: '💹',
      jit_liquidity: '🏃',
      backrun: '🔄',
    };
    return icons[type] || '📡';
  };

  const formatTimeAgo = (dateStr: string): string => {
    const date = new Date(dateStr);
    const now = new Date();
    const seconds = Math.floor((now.getTime() - date.getTime()) / 1000);
    if (seconds < 60) return `${seconds}s ago`;
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
    if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
    return `${Math.floor(seconds / 86400)}d ago`;
  };

  const renderSignalsView = () => {
    const displaySignals = signalsLiveMode ? liveSignals : signals;

    return (
      <div className={styles.signalsView}>
        <div className={styles.viewHeader}>
          <button className={styles.backButton} onClick={() => onViewChange('dashboard')}>
            ← Back
          </button>
          <h2>Signal Feed</h2>
          <div className={styles.signalsHeaderActions}>
            <button
              className={`${styles.liveModeButton} ${signalsLiveMode ? styles.active : ''}`}
              onClick={toggleSignalsLiveMode}
            >
              {signalsLiveMode ? '🔴 Live' : '⏸️ Paused'}
            </button>
            <button className={styles.refreshButton} onClick={fetchSignals}>
              🔄
            </button>
          </div>
        </div>

        <div className={styles.signalFilters}>
          <div className={styles.filterGroup}>
            <label>Signal Type</label>
            <select
              value={signalFilter.signal_type || ''}
              onChange={(e) => setSignalFilter({ ...signalFilter, signal_type: e.target.value || undefined })}
            >
              <option value="">All Types</option>
              <option value="price_discrepancy">Price Gap</option>
              <option value="volume_spike">Volume Spike</option>
              <option value="liquidity_change">Liquidity Change</option>
              <option value="new_token">New Token</option>
              <option value="curve_graduation">Curve Graduation</option>
              <option value="large_order">Large Order</option>
              <option value="liquidation">Liquidation</option>
              <option value="pool_imbalance">Pool Imbalance</option>
              <option value="dex_arb">DEX Arb</option>
              <option value="jit_liquidity">JIT Liquidity</option>
              <option value="backrun">Backrun</option>
            </select>
          </div>
          <div className={styles.filterGroup}>
            <label>Venue</label>
            <select
              value={signalFilter.venue_type || ''}
              onChange={(e) => setSignalFilter({ ...signalFilter, venue_type: e.target.value || undefined })}
            >
              <option value="">All Venues</option>
              <option value="dex_amm">DEX AMM</option>
              <option value="bonding_curve">Bonding Curve</option>
              <option value="lending">Lending</option>
              <option value="orderbook">Orderbook</option>
            </select>
          </div>
          <div className={styles.filterGroup}>
            <label>Min Profit (bps)</label>
            <input
              type="number"
              placeholder="0"
              value={signalFilter.min_profit_bps || ''}
              onChange={(e) => setSignalFilter({ ...signalFilter, min_profit_bps: parseInt(e.target.value) || undefined })}
            />
          </div>
          <div className={styles.filterGroup}>
            <label>Min Confidence</label>
            <input
              type="number"
              placeholder="0.5"
              step="0.1"
              min="0"
              max="1"
              value={signalFilter.min_confidence || ''}
              onChange={(e) => setSignalFilter({ ...signalFilter, min_confidence: parseFloat(e.target.value) || undefined })}
            />
          </div>
        </div>

        {signalsLoading && !signalsLiveMode ? (
          <div className={styles.loadingState}>Loading signals...</div>
        ) : displaySignals.length === 0 ? (
          <div className={styles.emptyState}>
            <p>{signalsLiveMode ? 'Waiting for live signals...' : 'No signals detected'}</p>
            <p className={styles.emptyHint}>
              {signalsLiveMode
                ? 'Signals will appear here as they are detected'
                : 'Start the scanner to detect trading opportunities'}
            </p>
          </div>
        ) : (
          <div className={styles.signalsList}>
            {displaySignals.map((signal) => (
              <div
                key={signal.id}
                className={`${styles.signalCard} ${styles[signal.significance]}`}
              >
                <div className={styles.signalCardHeader}>
                  <span className={styles.signalIcon}>{getSignalTypeIcon(signal.signal_type)}</span>
                  <span className={styles.signalType}>{getSignalTypeLabel(signal.signal_type)}</span>
                  <span className={`${styles.significanceBadge} ${styles[signal.significance]}`}>
                    {signal.significance}
                  </span>
                  <span className={styles.signalTime}>{formatTimeAgo(signal.detected_at)}</span>
                </div>
                <div className={styles.signalCardBody}>
                  <div className={styles.signalMetrics}>
                    <div className={styles.signalMetric}>
                      <span className={styles.metricLabel}>Est. Profit</span>
                      <span className={`${styles.metricValue} ${signal.estimated_profit_bps > 0 ? styles.positive : ''}`}>
                        {signal.estimated_profit_bps} bps
                      </span>
                    </div>
                    <div className={styles.signalMetric}>
                      <span className={styles.metricLabel}>Confidence</span>
                      <span className={styles.metricValue}>
                        {(signal.confidence * 100).toFixed(1)}%
                      </span>
                    </div>
                    <div className={styles.signalMetric}>
                      <span className={styles.metricLabel}>Venue</span>
                      <span className={styles.metricValue}>{signal.venue_type}</span>
                    </div>
                    <div className={styles.signalMetric}>
                      <span className={styles.metricLabel}>Expires</span>
                      <span className={styles.metricValue}>
                        {formatTimeAgo(signal.expires_at)}
                      </span>
                    </div>
                  </div>
                  {signal.token_mint && (
                    <div className={styles.signalToken}>
                      <span className={styles.tokenLabel}>Token:</span>
                      <span className={styles.tokenMint}>
                        {signal.token_mint.slice(0, 8)}...{signal.token_mint.slice(-4)}
                      </span>
                    </div>
                  )}
                </div>
                <div className={styles.signalCardActions}>
                  <button className={styles.investigateButton}>
                    Investigate
                  </button>
                  <button className={styles.createEdgeButton}>
                    Create Edge
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}

        <div className={styles.signalFlowDiagram}>
          <h3>Signal to Edge Flow</h3>
          <div className={styles.flowSteps}>
            <div className={`${styles.flowStep} ${styles.active}`}>
              <span className={styles.flowIcon}>📡</span>
              <span className={styles.flowLabel}>Signal Detected</span>
            </div>
            <div className={styles.flowArrow}>→</div>
            <div className={styles.flowStep}>
              <span className={styles.flowIcon}>🔍</span>
              <span className={styles.flowLabel}>Strategy Match</span>
            </div>
            <div className={styles.flowArrow}>→</div>
            <div className={styles.flowStep}>
              <span className={styles.flowIcon}>📋</span>
              <span className={styles.flowLabel}>Edge Created</span>
            </div>
            <div className={styles.flowArrow}>→</div>
            <div className={styles.flowStep}>
              <span className={styles.flowIcon}>✅</span>
              <span className={styles.flowLabel}>Approval</span>
            </div>
            <div className={styles.flowArrow}>→</div>
            <div className={styles.flowStep}>
              <span className={styles.flowIcon}>⚡</span>
              <span className={styles.flowLabel}>Execution</span>
            </div>
          </div>
        </div>
      </div>
    );
  };

  const renderDashboardView = () => {
    const { summary } = dashboard;
    const swarmHealth = swarm.health;
    const topEdges = (edges.data || [])
      .filter((e) => ['detected', 'pending_approval'].includes(e.status))
      .slice(0, 5);
    const recentTrades = (trades.data || []).slice(0, 5);
    const recentAlerts = (threats.alerts || []).slice(0, 3);

    return (
      <div className={styles.dashboardView}>
        <div className={styles.dashboardHeader}>
          <h1>ArbFarm Dashboard</h1>
          <div className={styles.headerActions}>
            <button
              className={`${styles.actionButton} ${swarm.health?.is_paused ? styles.resumeBtn : styles.pauseBtn}`}
              onClick={() => (swarm.health?.is_paused ? swarm.resume() : swarm.pause())}
              disabled={swarm.isLoading}
            >
              {swarm.health?.is_paused ? '▶ Resume' : '⏸ Pause'}
            </button>
            <button className={styles.settingsButton} onClick={() => onViewChange('settings')}>
              ⚙️
            </button>
          </div>
        </div>

        <div className={styles.metricsGrid}>
          <MetricCard
            label="Total P&L"
            value={formatProfit(summary?.total_profit_sol || 0)}
            trend={
              summary?.week_profit_sol ? (summary.week_profit_sol > 0 ? 'up' : 'down') : undefined
            }
            trendValue={
              summary?.week_profit_sol
                ? `${summary.week_profit_sol > 0 ? '+' : ''}${summary.week_profit_sol.toFixed(2)} this week`
                : undefined
            }
            color={
              summary?.total_profit_sol && summary.total_profit_sol >= 0 ? '#22c55e' : '#ef4444'
            }
          />
          <MetricCard
            label="Today's P&L"
            value={formatProfit(summary?.today_profit_sol || 0)}
            color={
              summary?.today_profit_sol && summary.today_profit_sol >= 0 ? '#22c55e' : '#ef4444'
            }
          />
          <MetricCard
            label="Win Rate"
            value={`${((summary?.win_rate || 0) * 100).toFixed(1)}%`}
            subValue={
              trades.stats
                ? `${trades.stats.successful_trades}/${trades.stats.total_trades} trades`
                : undefined
            }
            color="#3b82f6"
          />
          <MetricCard
            label="Active Opportunities"
            value={String(
              summary?.active_opportunities ||
                (edges.data || []).filter((e) => e.status === 'detected').length,
            )}
            subValue={`${summary?.pending_approvals || 0} pending approval`}
            color="#f59e0b"
            onClick={() => onViewChange('opportunities')}
          />
        </div>

        <div className={styles.dashboardGrid}>
          <div className={styles.gridSection}>
            <div className={styles.sectionHeader}>
              <h3>Swarm Status</h3>
              <span
                className={`${styles.healthBadge} ${styles[swarmHealth?.overall_health?.toLowerCase() || 'unknown']}`}
              >
                {swarmHealth?.overall_health || 'Unknown'}
              </span>
            </div>
            <SwarmStatusCard
              health={swarmHealth}
              scannerStatus={scanner.status}
              isLoading={swarm.isLoading || scanner.isLoading}
            />
          </div>

          <div className={styles.gridSection}>
            <div className={styles.sectionHeader}>
              <h3>Top Opportunities</h3>
              <button
                className={styles.viewAllButton}
                onClick={() => onViewChange('opportunities')}
              >
                View All →
              </button>
            </div>
            <div className={styles.opportunitiesList}>
              {edges.isLoading ? (
                <div className={styles.loadingState}>Loading...</div>
              ) : topEdges.length === 0 ? (
                <div className={styles.emptyState}>No active opportunities</div>
              ) : (
                topEdges.map((edge) => (
                  <EdgeCard
                    key={edge.id}
                    edge={edge}
                    onApprove={() => edges.approve(edge.id)}
                    onReject={(reason) => edges.reject(edge.id, reason)}
                    onExecute={() => edges.execute(edge.id)}
                    compact
                  />
                ))
              )}
            </div>
          </div>

          <div className={styles.gridSection}>
            <div className={styles.sectionHeader}>
              <h3>⏳ Pending Approvals</h3>
              <div className={styles.headerBadges}>
                <span className={`${styles.executionBadge} ${executionConfig?.auto_execution_enabled ? styles.enabled : styles.disabled}`}>
                  Auto: {executionConfig?.auto_execution_enabled ? 'ON' : 'OFF'}
                </span>
                <button
                  className={styles.refreshButton}
                  onClick={fetchPendingApprovals}
                  disabled={approvalsLoading}
                >
                  🔄
                </button>
              </div>
            </div>
            <div className={styles.pendingApprovalsList}>
              {approvalsLoading ? (
                <div className={styles.loadingState}>Loading approvals...</div>
              ) : pendingApprovals.length === 0 ? (
                <div className={styles.emptyState}>
                  <p>No pending approvals</p>
                  <p className={styles.emptyHint}>
                    {executionConfig?.auto_execution_enabled
                      ? 'Auto-execution is enabled - trades execute automatically'
                      : 'Enable auto-execution in Settings → Execution, or manually approve edges'}
                  </p>
                </div>
              ) : (
                pendingApprovals.slice(0, 5).map((approval) => (
                  <div key={approval.id} className={styles.approvalCard}>
                    <div className={styles.approvalHeader}>
                      <span className={styles.approvalType}>{approval.approval_type}</span>
                      <span className={styles.approvalStatus}>{approval.status}</span>
                      {approval.expires_at && (
                        <span className={styles.expiresAt}>
                          Expires: {new Date(approval.expires_at).toLocaleTimeString()}
                        </span>
                      )}
                    </div>
                    {approval.hecate_decision !== undefined && (
                      <div className={styles.hecateRecommendation}>
                        <span className={styles.hecateLabel}>🤖 Hecate:</span>
                        <span className={approval.hecate_decision ? styles.recommend : styles.reject}>
                          {approval.hecate_decision ? 'Recommends Approve' : 'Recommends Reject'}
                        </span>
                        {approval.hecate_reasoning && (
                          <p className={styles.hecateReasoning}>{approval.hecate_reasoning}</p>
                        )}
                      </div>
                    )}
                    <div className={styles.approvalActions}>
                      <button
                        className={styles.approveButton}
                        onClick={() => handleApproveApproval(approval.id)}
                        disabled={approvingId === approval.id}
                      >
                        {approvingId === approval.id ? '...' : '✓ Approve'}
                      </button>
                      <button
                        className={styles.rejectButton}
                        onClick={() => handleRejectApproval(approval.id, 'Manual rejection')}
                        disabled={rejectingId === approval.id}
                      >
                        {rejectingId === approval.id ? '...' : '✗ Reject'}
                      </button>
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>

          <div className={styles.gridSection}>
            <div className={styles.sectionHeader}>
              <h3>Live Signals</h3>
              <button
                className={styles.viewAllButton}
                onClick={() => onViewChange('signals')}
              >
                View Feed →
              </button>
            </div>
            <div className={styles.signalsPreview}>
              <div className={styles.signalsSummary}>
                <div className={styles.signalStat}>
                  <span className={styles.signalStatValue}>
                    {scanner.status?.signals_detected_24h || 0}
                  </span>
                  <span className={styles.signalStatLabel}>Signals (24h)</span>
                </div>
                <div className={styles.signalStat}>
                  <span className={styles.signalStatValue}>
                    {scanner.status?.is_running ? '🟢' : '⏸️'}
                  </span>
                  <span className={styles.signalStatLabel}>Scanner</span>
                </div>
              </div>
              <p className={styles.signalsHint}>
                Monitor real-time trading signals from all venues
              </p>
            </div>
          </div>

          <div className={styles.gridSection}>
            <div className={styles.sectionHeader}>
              <h3>Recent Trades</h3>
              <span className={styles.statsLabel}>{trades.stats?.total_trades || 0} total</span>
            </div>
            <div className={styles.tradesList}>
              {trades.isLoading ? (
                <div className={styles.loadingState}>Loading...</div>
              ) : recentTrades.length === 0 ? (
                <div className={styles.emptyState}>No trades yet</div>
              ) : (
                recentTrades.map((trade) => (
                  <TradeHistoryCard key={trade.id} trade={trade} compact />
                ))
              )}
            </div>
          </div>

          <div className={styles.gridSection}>
            <div className={styles.sectionHeader}>
              <h3>Threat Alerts</h3>
              <button className={styles.viewAllButton} onClick={() => onViewChange('threats')}>
                View All →
              </button>
            </div>
            <div className={styles.alertsList}>
              {threats.isLoading ? (
                <div className={styles.loadingState}>Loading...</div>
              ) : recentAlerts.length === 0 ? (
                <div className={styles.emptyState}>No recent alerts</div>
              ) : (
                recentAlerts.map((alert) => (
                  <ThreatAlertCard key={alert.id} alert={alert} compact />
                ))
              )}
            </div>
          </div>
        </div>

        <div className={styles.capitalOverview}>
          <div className={styles.sectionHeader}>
            <h3>💰 Capital Allocation</h3>
            <button
              className={styles.syncButton}
              onClick={handleSyncCapital}
              disabled={syncingCapital}
            >
              {syncingCapital ? '⟳ Syncing...' : '🔄 Sync'}
            </button>
          </div>
          {capitalLoading ? (
            <div className={styles.loadingState}>Loading capital data...</div>
          ) : capitalUsage ? (
            <div className={styles.capitalContent}>
              <div className={styles.capitalSummary}>
                <div className={styles.capitalStat}>
                  <span className={styles.capitalValue}>{capitalUsage.total_balance_sol.toFixed(4)}</span>
                  <span className={styles.capitalLabel}>Total Balance (SOL)</span>
                </div>
                <div className={styles.capitalStat}>
                  <span className={styles.capitalValue} style={{ color: '#22c55e' }}>
                    {capitalUsage.available_sol.toFixed(4)}
                  </span>
                  <span className={styles.capitalLabel}>Available</span>
                </div>
                <div className={styles.capitalStat}>
                  <span className={styles.capitalValue} style={{ color: '#f59e0b' }}>
                    {capitalUsage.global_reserved_sol.toFixed(4)}
                  </span>
                  <span className={styles.capitalLabel}>Reserved</span>
                </div>
              </div>
              {capitalUsage.strategy_allocations.length > 0 && (
                <div className={styles.allocationsList}>
                  <div className={styles.allocationsHeader}>Strategy Allocations</div>
                  {capitalUsage.strategy_allocations.map((alloc) => {
                    const strategy = (strategies.data || []).find((s) => s.id === alloc.strategy_id);
                    const usagePercent = alloc.max_allocation_sol > 0
                      ? (alloc.current_reserved_sol / alloc.max_allocation_sol) * 100
                      : 0;
                    return (
                      <div key={alloc.strategy_id} className={styles.allocationItem}>
                        <div className={styles.allocationInfo}>
                          <span className={styles.allocationName}>
                            {strategy?.name || alloc.strategy_id.slice(0, 8)}
                          </span>
                          <span className={styles.allocationPercent}>
                            {alloc.max_allocation_percent}% max
                          </span>
                        </div>
                        <div className={styles.allocationBar}>
                          <div
                            className={styles.allocationUsed}
                            style={{ width: `${Math.min(usagePercent, 100)}%` }}
                          />
                        </div>
                        <div className={styles.allocationDetails}>
                          <span>{alloc.current_reserved_sol.toFixed(3)} / {alloc.max_allocation_sol.toFixed(3)} SOL</span>
                          <span className={styles.positionCount}>
                            {alloc.active_positions}/{alloc.max_positions} positions
                          </span>
                        </div>
                      </div>
                    );
                  })}
                </div>
              )}
              {capitalUsage.active_reservations.length > 0 && (
                <div className={styles.reservationsList}>
                  <div className={styles.reservationsHeader}>
                    Active Reservations ({capitalUsage.active_reservations.length})
                  </div>
                  {capitalUsage.active_reservations.map((res) => (
                    <div key={res.position_id} className={styles.reservationItem}>
                      <span className={styles.reservationAmount}>{res.amount_sol.toFixed(4)} SOL</span>
                      <span className={styles.reservationId}>Position: {res.position_id.slice(0, 8)}...</span>
                    </div>
                  ))}
                </div>
              )}
            </div>
          ) : (
            <div className={styles.emptyState}>
              <p>No capital data available</p>
              <button className={styles.syncButton} onClick={handleSyncCapital}>
                Sync Wallet Balance
              </button>
            </div>
          )}
        </div>

        <div className={styles.strategiesOverview}>
          <div className={styles.sectionHeader}>
            <h3>Active Strategies</h3>
            <button className={styles.viewAllButton} onClick={() => onViewChange('strategies')}>
              Manage →
            </button>
          </div>
          <div className={styles.strategiesList}>
            {strategies.isLoading ? (
              <div className={styles.loadingState}>Loading...</div>
            ) : (strategies.data || []).length === 0 ? (
              <div className={styles.emptyState}>No strategies configured</div>
            ) : (
              (strategies.data || [])
                .filter((s) => s.is_active)
                .slice(0, 4)
                .map((strategy) => (
                  <div key={strategy.id} className={styles.strategyChip}>
                    <span
                      className={styles.strategyIndicator}
                      style={{ backgroundColor: '#22c55e' }}
                    />
                    <span className={styles.strategyName}>{strategy.name}</span>
                    <span className={styles.strategyType}>{strategy.strategy_type}</span>
                  </div>
                ))
            )}
          </div>
        </div>

        <div className={styles.cowsOverview}>
          <div className={styles.sectionHeader}>
            <h3>COW Marketplace</h3>
            <button className={styles.viewAllButton} onClick={() => onViewChange('cows')}>
              Browse COWs →
            </button>
          </div>
          <div className={styles.cowsSummary}>
            {cows.isLoading ? (
              <div className={styles.loadingState}>Loading...</div>
            ) : cows.stats ? (
              <div className={styles.cowsStats}>
                <div className={styles.cowStat}>
                  <span className={styles.cowStatValue}>{cows.stats.total_cows}</span>
                  <span className={styles.cowStatLabel}>Total COWs</span>
                </div>
                <div className={styles.cowStat}>
                  <span className={styles.cowStatValue}>{cows.stats.total_forks}</span>
                  <span className={styles.cowStatLabel}>Forks</span>
                </div>
                <div className={styles.cowStat}>
                  <span className={styles.cowStatValue}>{cows.stats.forkable_cows}</span>
                  <span className={styles.cowStatLabel}>Forkable</span>
                </div>
              </div>
            ) : (
              <div className={styles.emptyState}>Mint or fork MEV strategies as NFTs</div>
            )}
          </div>
        </div>

        <div className={styles.heliusOverview}>
          <div className={styles.sectionHeader}>
            <h3>⚡ Helius Integration</h3>
            <button className={styles.viewAllButton} onClick={() => onViewChange('helius')}>
              Manage →
            </button>
          </div>
          <div className={styles.heliusSummary}>
            {heliusLoading ? (
              <div className={styles.loadingState}>Loading...</div>
            ) : heliusStatus ? (
              <div className={styles.heliusStats}>
                <div className={styles.heliusStat}>
                  <span className={`${styles.heliusStatValue} ${heliusStatus.connected ? styles.connected : styles.disconnected}`}>
                    {heliusStatus.connected ? '●' : '○'}
                  </span>
                  <span className={styles.heliusStatLabel}>
                    {heliusStatus.connected ? 'Connected' : 'Disconnected'}
                  </span>
                </div>
                <div className={styles.heliusStat}>
                  <span className={`${styles.heliusStatValue} ${laserStreamStatus?.connected ? styles.connected : styles.disconnected}`}>
                    {laserStreamStatus?.connected ? '●' : '○'}
                  </span>
                  <span className={styles.heliusStatLabel}>LaserStream</span>
                </div>
                <div className={styles.heliusStat}>
                  <span className={styles.heliusStatValue}>
                    {priorityFees?.recommended ? `${(priorityFees.recommended / 1000).toFixed(1)}K` : '-'}
                  </span>
                  <span className={styles.heliusStatLabel}>Priority Fee</span>
                </div>
                <div className={styles.heliusStat}>
                  <span className={styles.heliusStatValue}>
                    {senderStats?.total_sent || 0}
                  </span>
                  <span className={styles.heliusStatLabel}>TXs Sent</span>
                </div>
              </div>
            ) : (
              <div className={styles.emptyState}>Configure Helius for enhanced MEV capabilities</div>
            )}
          </div>
        </div>

        {sse.isConnected && (
          <div className={styles.sseStatus}>
            <span className={styles.sseIndicator} />
            Live updates active
          </div>
        )}
      </div>
    );
  };

  const renderOpportunitiesView = () => {
    const edgesData = edges.data || [];
    const filteredEdges =
      opportunitiesFilter === 'all'
        ? edgesData
        : edgesData.filter((e) => e.status === opportunitiesFilter);

    return (
      <div className={styles.opportunitiesView}>
        <div className={styles.viewHeader}>
          <button className={styles.backButton} onClick={() => onViewChange('dashboard')}>
            ← Back
          </button>
          <h2>Live Opportunities</h2>
          <button className={styles.refreshButton} onClick={() => edges.refresh()}>
            🔄
          </button>
        </div>

        <div className={styles.filterBar}>
          {['all', 'detected', 'pending_approval', 'executing', 'executed', 'failed'].map(
            (status) => (
              <button
                key={status}
                className={`${styles.filterChip} ${opportunitiesFilter === status ? styles.active : ''}`}
                onClick={() => setOpportunitiesFilter(status)}
                style={
                  status !== 'all'
                    ? { borderColor: EDGE_STATUS_COLORS[status as keyof typeof EDGE_STATUS_COLORS] }
                    : undefined
                }
              >
                {status === 'all' ? 'All' : status.replace('_', ' ')}
                <span className={styles.count}>
                  {status === 'all'
                    ? edgesData.length
                    : edgesData.filter((e) => e.status === status).length}
                </span>
              </button>
            ),
          )}
        </div>

        <div className={styles.edgesList}>
          {edges.isLoading ? (
            <div className={styles.loadingState}>Loading opportunities...</div>
          ) : filteredEdges.length === 0 ? (
            <div className={styles.emptyState}>
              <p>No opportunities found</p>
              <p className={styles.emptyHint}>
                {scanner.status?.is_running
                  ? 'Scanner is actively searching...'
                  : 'Start the scanner to detect opportunities'}
              </p>
            </div>
          ) : (
            filteredEdges.map((edge) => (
              <EdgeCard
                key={edge.id}
                edge={edge}
                onApprove={() => edges.approve(edge.id)}
                onReject={(reason) => edges.reject(edge.id, reason)}
                onExecute={() => edges.execute(edge.id)}
              />
            ))
          )}
        </div>
      </div>
    );
  };

  const renderStrategiesView = () => (
    <div className={styles.strategiesView}>
      <div className={styles.viewHeader}>
        <button className={styles.backButton} onClick={() => onViewChange('dashboard')}>
          ← Back
        </button>
        <h2>Strategies</h2>
        <div className={styles.headerActions}>
          <button
            className={styles.saveEngramsButton}
            onClick={handleSaveToEngrams}
            disabled={savingToEngrams}
          >
            {savingToEngrams ? 'Saving...' : '💾 Save to Engrams'}
          </button>
          <button className={styles.createButton} onClick={() => setShowCreateStrategy(true)}>
            + New Strategy
          </button>
        </div>
      </div>

      {selectedStrategies.size > 0 && (
        <div className={styles.batchActionBar}>
          <span className={styles.selectedCount}>{selectedStrategies.size} selected</span>
          <button className={styles.batchEnableButton} onClick={() => handleBatchToggle(true)}>
            Enable All
          </button>
          <button className={styles.batchDisableButton} onClick={() => handleBatchToggle(false)}>
            Disable All
          </button>
          <button className={styles.clearSelectionButton} onClick={() => setSelectedStrategies(new Set())}>
            Clear Selection
          </button>
        </div>
      )}

      <div className={styles.strategiesGrid}>
        {strategies.isLoading ? (
          <div className={styles.loadingState}>Loading strategies...</div>
        ) : (strategies.data || []).length === 0 ? (
          <div className={styles.emptyState}>
            <p>No strategies configured</p>
            <p className={styles.emptyHint}>Create a strategy to start automated trading</p>
          </div>
        ) : (
          <>
            {(strategies.data || []).length > 1 && (
              <div className={styles.selectAllRow}>
                <label className={styles.checkboxLabel}>
                  <input
                    type="checkbox"
                    checked={selectedStrategies.size === (strategies.data || []).length}
                    onChange={handleSelectAllStrategies}
                  />
                  Select All
                </label>
              </div>
            )}
            {(strategies.data || []).map((strategy) => (
              <div key={strategy.id} className={`${styles.strategyCard} ${selectedStrategies.has(strategy.id) ? styles.selected : ''}`}>
                <div className={styles.strategyHeader}>
                  <label className={styles.strategyCheckbox}>
                    <input
                      type="checkbox"
                      checked={selectedStrategies.has(strategy.id)}
                      onChange={() => handleStrategySelection(strategy.id)}
                    />
                  </label>
                  <span className={styles.strategyName}>{strategy.name}</span>
                  <div className={styles.strategyActions}>
                    <button
                      className={styles.editButton}
                      onClick={() => handleEditStrategy(strategy)}
                      title="Edit strategy parameters"
                    >
                      ✏️
                    </button>
                    {strategy.is_active && (
                      <button
                        className={styles.killButton}
                        onClick={() => handleKillStrategy(strategy.id, strategy.name)}
                        title="KILL: Emergency stop all running operations"
                      >
                        ⛔
                      </button>
                    )}
                    <button
                      className={styles.deleteButton}
                      onClick={() => handleDeleteStrategy(strategy.id, strategy.name)}
                      title="DELETE: Permanently remove strategy"
                    >
                      🗑️
                    </button>
                    <button
                      className={`${styles.toggleButton} ${strategy.is_active ? styles.active : ''}`}
                      onClick={() => strategies.toggle(strategy.id, !strategy.is_active)}
                      title={strategy.is_active ? 'Toggle OFF (pause)' : 'Toggle ON (resume)'}
                    >
                      {strategy.is_active ? '🟢 ON' : '⏸️ OFF'}
                    </button>
                  </div>
                </div>
                <div className={styles.strategyMeta}>
                  <span className={styles.strategyType}>{strategy.strategy_type}</span>
                  <span>Mode: {strategy.execution_mode}</span>
                  <span>Venues: {strategy.venue_types.join(', ')}</span>
                </div>
                <div className={styles.strategyRisk}>
                  <span>Max Position: {strategy.risk_params.max_position_sol} SOL</span>
                  <span>Min Profit: {strategy.risk_params.min_profit_bps} bps</span>
                  <span>Max Slippage: {strategy.risk_params.max_slippage_bps} bps</span>
                </div>
                {strategy.stats && (
                  <div className={styles.strategyStats}>
                    <span>Trades: {strategy.stats.total_trades}</span>
                    <span>Win Rate: {(strategy.stats.win_rate * 100).toFixed(1)}%</span>
                    <span
                      className={
                        strategy.stats.total_profit_sol >= 0 ? styles.positive : styles.negative
                      }
                    >
                      P&L: {strategy.stats.total_profit_sol >= 0 ? '+' : ''}
                      {strategy.stats.total_profit_sol.toFixed(4)} SOL
                    </span>
                    {devModeAvailable && (
                      <button
                        className={styles.resetStatsButton}
                        onClick={() => handleResetStrategyStats(strategy.id)}
                        disabled={resettingStats === strategy.id}
                        title="Reset stats (dev only)"
                      >
                        {resettingStats === strategy.id ? '...' : '🔄'}
                      </button>
                    )}
                  </div>
                )}
              </div>
            ))}
          </>
        )}
      </div>

      {showCreateStrategy && (
        <div className={styles.modalOverlay} onClick={() => setShowCreateStrategy(false)}>
          <div className={styles.createStrategyModal} onClick={(e) => e.stopPropagation()}>
            <div className={styles.modalHeader}>
              <h3>Create New Strategy</h3>
              <button className={styles.closeButton} onClick={() => setShowCreateStrategy(false)}>
                ×
              </button>
            </div>

            <div className={styles.modalBody}>
              <div className={styles.formGroup}>
                <label>Strategy Name</label>
                <input
                  type="text"
                  placeholder="e.g., Jupiter-Raydium Arb"
                  value={newStrategy.name}
                  onChange={(e) => setNewStrategy((prev) => ({ ...prev, name: e.target.value }))}
                />
              </div>

              <div className={styles.formGroup}>
                <label>Strategy Type</label>
                <select
                  value={newStrategy.strategy_type}
                  onChange={(e) =>
                    setNewStrategy((prev) => ({ ...prev, strategy_type: e.target.value }))
                  }
                >
                  <option value="dex_arb">DEX Arbitrage</option>
                  <option value="curve_graduation">Curve Graduation</option>
                  <option value="liquidation">Liquidation Hunting</option>
                  <option value="kol_copy">KOL Copy Trading</option>
                  <option value="momentum">Momentum</option>
                  <option value="mean_reversion">Mean Reversion</option>
                  {devModeAvailable && (
                    <>
                      <option value="jit_liquidity">JIT Liquidity (Dev)</option>
                      <option value="backrun">Backrun (Dev)</option>
                      <option value="sandwich">Sandwich (Dev)</option>
                    </>
                  )}
                </select>
              </div>

              <div className={styles.formGroup}>
                <label>Venue Types</label>
                <div className={styles.venueCheckboxes}>
                  {[
                    { value: 'dex_amm', label: 'DEX (Jupiter, Raydium)' },
                    { value: 'bonding_curve', label: 'Bonding Curves (pump.fun)' },
                    { value: 'lending', label: 'Lending (Kamino, Marginfi)' },
                    { value: 'perps', label: 'Perps (Drift)' },
                    { value: 'orderbook', label: 'Orderbook (Phoenix)' },
                  ].map((venue) => (
                    <label key={venue.value} className={styles.checkboxLabel}>
                      <input
                        type="checkbox"
                        checked={newStrategy.venue_types.includes(venue.value)}
                        onChange={() => handleVenueToggle(venue.value)}
                      />
                      {venue.label}
                    </label>
                  ))}
                </div>
              </div>

              <div className={styles.formGroup}>
                <label>Execution Mode</label>
                <div className={styles.executionModeOptions}>
                  {[
                    {
                      value: 'agent_directed',
                      label: 'Agent Directed',
                      desc: 'Multi-LLM consensus for every trade',
                    },
                    {
                      value: 'hybrid',
                      label: 'Hybrid',
                      desc: 'Auto for small, consensus for large',
                    },
                    {
                      value: 'autonomous',
                      label: 'Autonomous',
                      desc: 'Full auto-execution (use with caution)',
                    },
                  ].map((mode) => (
                    <label
                      key={mode.value}
                      className={`${styles.modeOption} ${
                        newStrategy.execution_mode === mode.value ? styles.selected : ''
                      }`}
                    >
                      <input
                        type="radio"
                        name="executionMode"
                        value={mode.value}
                        checked={newStrategy.execution_mode === mode.value}
                        onChange={(e) =>
                          setNewStrategy((prev) => ({
                            ...prev,
                            execution_mode: e.target.value as
                              | 'autonomous'
                              | 'agent_directed'
                              | 'hybrid',
                          }))
                        }
                      />
                      <div className={styles.modeContent}>
                        <span className={styles.modeLabel}>{mode.label}</span>
                        <span className={styles.modeDesc}>{mode.desc}</span>
                      </div>
                    </label>
                  ))}
                </div>
              </div>

              <div className={styles.formSection}>
                <h4>Risk Parameters</h4>
                <div className={styles.riskParamsGrid}>
                  <div className={styles.paramGroup}>
                    <label>Max Position (SOL)</label>
                    <input
                      type="number"
                      step="0.1"
                      min="0.01"
                      value={newStrategy.risk_params.max_position_sol}
                      onChange={(e) =>
                        setNewStrategy((prev) => ({
                          ...prev,
                          risk_params: {
                            ...prev.risk_params,
                            max_position_sol: parseFloat(e.target.value) || 0,
                          },
                        }))
                      }
                    />
                  </div>
                  <div className={styles.paramGroup}>
                    <label>Daily Loss Limit (SOL)</label>
                    <input
                      type="number"
                      step="0.1"
                      min="0.01"
                      value={newStrategy.risk_params.daily_loss_limit_sol}
                      onChange={(e) =>
                        setNewStrategy((prev) => ({
                          ...prev,
                          risk_params: {
                            ...prev.risk_params,
                            daily_loss_limit_sol: parseFloat(e.target.value) || 0,
                          },
                        }))
                      }
                    />
                  </div>
                  <div className={styles.paramGroup}>
                    <label>Min Profit (bps)</label>
                    <input
                      type="number"
                      step="5"
                      min="1"
                      value={newStrategy.risk_params.min_profit_bps}
                      onChange={(e) =>
                        setNewStrategy((prev) => ({
                          ...prev,
                          risk_params: {
                            ...prev.risk_params,
                            min_profit_bps: parseInt(e.target.value) || 0,
                          },
                        }))
                      }
                    />
                  </div>
                  <div className={styles.paramGroup}>
                    <label>Max Slippage (bps)</label>
                    <input
                      type="number"
                      step="5"
                      min="1"
                      value={newStrategy.risk_params.max_slippage_bps}
                      onChange={(e) =>
                        setNewStrategy((prev) => ({
                          ...prev,
                          risk_params: {
                            ...prev.risk_params,
                            max_slippage_bps: parseInt(e.target.value) || 0,
                          },
                        }))
                      }
                    />
                  </div>
                  <div className={styles.paramGroup}>
                    <label>Max Risk Score</label>
                    <input
                      type="number"
                      step="5"
                      min="0"
                      max="100"
                      value={newStrategy.risk_params.max_risk_score}
                      onChange={(e) =>
                        setNewStrategy((prev) => ({
                          ...prev,
                          risk_params: {
                            ...prev.risk_params,
                            max_risk_score: parseInt(e.target.value) || 0,
                          },
                        }))
                      }
                    />
                  </div>
                </div>

                <div className={styles.riskToggles}>
                  <label className={styles.toggleLabel}>
                    <input
                      type="checkbox"
                      checked={newStrategy.risk_params.require_simulation}
                      onChange={(e) =>
                        setNewStrategy((prev) => ({
                          ...prev,
                          risk_params: {
                            ...prev.risk_params,
                            require_simulation: e.target.checked,
                          },
                        }))
                      }
                    />
                    Require simulation before execution
                  </label>
                  <label className={styles.toggleLabel}>
                    <input
                      type="checkbox"
                      checked={newStrategy.risk_params.auto_execute_atomic}
                      onChange={(e) =>
                        setNewStrategy((prev) => ({
                          ...prev,
                          risk_params: {
                            ...prev.risk_params,
                            auto_execute_atomic: e.target.checked,
                          },
                        }))
                      }
                    />
                    Auto-execute fully atomic trades (zero capital risk)
                  </label>
                </div>
              </div>
            </div>

            <div className={styles.modalFooter}>
              <button
                className={styles.cancelButton}
                onClick={() => setShowCreateStrategy(false)}
              >
                Cancel
              </button>
              <button
                className={styles.submitButton}
                onClick={handleCreateStrategy}
                disabled={creatingStrategy || !newStrategy.name.trim()}
              >
                {creatingStrategy ? 'Creating...' : 'Create Strategy'}
              </button>
            </div>
          </div>
        </div>
      )}

      {showEditStrategy && editingStrategy && (
        <div className={styles.modalOverlay} onClick={() => setShowEditStrategy(false)}>
          <div className={styles.createStrategyModal} onClick={(e) => e.stopPropagation()}>
            <div className={styles.modalHeader}>
              <h3>Edit Strategy</h3>
              <button className={styles.closeButton} onClick={() => setShowEditStrategy(false)}>
                ×
              </button>
            </div>

            <div className={styles.modalBody}>
              <div className={styles.formGroup}>
                <label>Strategy Name</label>
                <input
                  type="text"
                  placeholder="e.g., Jupiter-Raydium Arb"
                  value={editingStrategy.name}
                  onChange={(e) => setEditingStrategy((prev: any) => ({ ...prev, name: e.target.value }))}
                />
              </div>

              <div className={styles.formGroup}>
                <label>Venue Types</label>
                <div className={styles.venueCheckboxes}>
                  {[
                    { value: 'dex_amm', label: 'DEX (Jupiter, Raydium)' },
                    { value: 'bonding_curve', label: 'Bonding Curves (pump.fun)' },
                    { value: 'lending', label: 'Lending (Kamino, Marginfi)' },
                    { value: 'perps', label: 'Perps (Drift)' },
                    { value: 'orderbook', label: 'Orderbook (Phoenix)' },
                  ].map((venue) => (
                    <label key={venue.value} className={styles.checkboxLabel}>
                      <input
                        type="checkbox"
                        checked={editingStrategy.venue_types?.includes(venue.value)}
                        onChange={() => handleEditVenueToggle(venue.value)}
                      />
                      {venue.label}
                    </label>
                  ))}
                </div>
              </div>

              <div className={styles.formGroup}>
                <label>Execution Mode</label>
                <div className={styles.executionModeOptions}>
                  {[
                    { value: 'agent_directed', label: 'Agent Directed', desc: 'Multi-LLM consensus for every trade' },
                    { value: 'hybrid', label: 'Hybrid', desc: 'Auto for small, consensus for large' },
                    { value: 'autonomous', label: 'Autonomous', desc: 'Full auto-execution (use with caution)' },
                  ].map((mode) => (
                    <label
                      key={mode.value}
                      className={`${styles.modeOption} ${editingStrategy.execution_mode === mode.value ? styles.selected : ''}`}
                    >
                      <input
                        type="radio"
                        name="editExecutionMode"
                        value={mode.value}
                        checked={editingStrategy.execution_mode === mode.value}
                        onChange={(e) => setEditingStrategy((prev: any) => ({ ...prev, execution_mode: e.target.value }))}
                      />
                      <div className={styles.modeContent}>
                        <span className={styles.modeLabel}>{mode.label}</span>
                        <span className={styles.modeDesc}>{mode.desc}</span>
                      </div>
                    </label>
                  ))}
                </div>
              </div>

              <div className={styles.formSection}>
                <h4>Risk Parameters</h4>
                <div className={styles.riskParamsGrid}>
                  <div className={styles.paramGroup}>
                    <label>Max Position (SOL)</label>
                    <input
                      type="number"
                      step="0.1"
                      min="0.01"
                      value={editingStrategy.risk_params?.max_position_sol || 1}
                      onChange={(e) => setEditingStrategy((prev: any) => ({
                        ...prev,
                        risk_params: { ...prev.risk_params, max_position_sol: parseFloat(e.target.value) || 0 },
                      }))}
                    />
                  </div>
                  <div className={styles.paramGroup}>
                    <label>Daily Loss Limit (SOL)</label>
                    <input
                      type="number"
                      step="0.1"
                      min="0.01"
                      value={editingStrategy.risk_params?.daily_loss_limit_sol || 0.5}
                      onChange={(e) => setEditingStrategy((prev: any) => ({
                        ...prev,
                        risk_params: { ...prev.risk_params, daily_loss_limit_sol: parseFloat(e.target.value) || 0 },
                      }))}
                    />
                  </div>
                  <div className={styles.paramGroup}>
                    <label>Min Profit (bps)</label>
                    <input
                      type="number"
                      step="5"
                      min="1"
                      value={editingStrategy.risk_params?.min_profit_bps || 50}
                      onChange={(e) => setEditingStrategy((prev: any) => ({
                        ...prev,
                        risk_params: { ...prev.risk_params, min_profit_bps: parseInt(e.target.value) || 0 },
                      }))}
                    />
                  </div>
                  <div className={styles.paramGroup}>
                    <label>Max Slippage (bps)</label>
                    <input
                      type="number"
                      step="5"
                      min="1"
                      value={editingStrategy.risk_params?.max_slippage_bps || 100}
                      onChange={(e) => setEditingStrategy((prev: any) => ({
                        ...prev,
                        risk_params: { ...prev.risk_params, max_slippage_bps: parseInt(e.target.value) || 0 },
                      }))}
                    />
                  </div>
                </div>
              </div>

              {devModeAvailable && (
                <div className={styles.devSection}>
                  <h4>🔧 Dev Options</h4>
                  <details className={styles.jsonView}>
                    <summary>Raw JSON</summary>
                    <pre>{JSON.stringify(editingStrategy, null, 2)}</pre>
                  </details>
                </div>
              )}
            </div>

            <div className={styles.modalFooter}>
              <button className={styles.cancelButton} onClick={() => setShowEditStrategy(false)}>
                Cancel
              </button>
              <button
                className={styles.submitButton}
                onClick={handleUpdateStrategy}
                disabled={!editingStrategy.name?.trim()}
              >
                Save Changes
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );

  const handleThreatCheck = async () => {
    if (!threatTokenInput.trim()) {
      return;
    }

    setThreatChecking(true);
    const result = await threats.checkToken(threatTokenInput.trim());

    setThreatCheckResult(result);
    setThreatChecking(false);
  };

  const renderThreatsView = () => {

    return (
      <div className={styles.threatsView}>
        <div className={styles.viewHeader}>
          <button className={styles.backButton} onClick={() => onViewChange('dashboard')}>
            ← Back
          </button>
          <h2>Threat Detection</h2>
        </div>

        <div className={styles.tokenChecker}>
          <h3>Quick Token Check</h3>
          <div className={styles.checkerInput}>
            <input
              type="text"
              placeholder="Enter token mint address..."
              value={threatTokenInput}
              onChange={(e) => setThreatTokenInput(e.target.value)}
            />
            <button onClick={handleThreatCheck} disabled={threatChecking || !threatTokenInput.trim()}>
              {threatChecking ? 'Checking...' : 'Check'}
            </button>
          </div>
          {threatCheckResult && (
            <div className={styles.checkResult}>
              <div
                className={`${styles.threatScore} ${threatCheckResult.overall_score > 0.7 ? styles.critical : threatCheckResult.overall_score > 0.3 ? styles.warning : styles.safe}`}
              >
                Score: {(threatCheckResult.overall_score * 100).toFixed(0)}%
              </div>
              <div className={styles.threatFactors}>
                {threatCheckResult.factors.has_mint_authority && (
                  <span className={styles.factorBad}>Mint Authority</span>
                )}
                {threatCheckResult.factors.has_freeze_authority && (
                  <span className={styles.factorBad}>Freeze Authority</span>
                )}
                {threatCheckResult.factors.has_blacklist && (
                  <span className={styles.factorBad}>Has Blacklist</span>
                )}
                {threatCheckResult.factors.upgradeable && (
                  <span className={styles.factorWarning}>Upgradeable</span>
                )}
                {threatCheckResult.factors.top_10_concentration > 0.5 && (
                  <span className={styles.factorWarning}>High Concentration</span>
                )}
              </div>
            </div>
          )}
        </div>

        <div className={styles.alertsSection}>
          <h3>Recent Alerts ({(threats.alerts || []).length})</h3>
          <div className={styles.alertsList}>
            {threats.isLoading ? (
              <div className={styles.loadingState}>Loading alerts...</div>
            ) : (threats.alerts || []).length === 0 ? (
              <div className={styles.emptyState}>No threat alerts</div>
            ) : (
              (threats.alerts || []).map((alert) => <ThreatAlertCard key={alert.id} alert={alert} />)
            )}
          </div>
        </div>

        <div className={styles.blockedSection}>
          <h3>Blocked Entities ({(threats.blocked || []).length})</h3>
          <div className={styles.blockedList}>
            {(threats.blocked || []).length === 0 ? (
              <div className={styles.emptyState}>No blocked entities</div>
            ) : (
              (threats.blocked || []).slice(0, 10).map((entity) => (
                <div key={entity.id} className={styles.blockedItem}>
                  <span className={styles.blockedType}>{entity.entity_type}</span>
                  <span className={styles.blockedAddress}>
                    {entity.address.slice(0, 8)}...{entity.address.slice(-4)}
                  </span>
                  <span className={styles.blockedCategory}>{entity.threat_category}</span>
                </div>
              ))
            )}
          </div>
        </div>
      </div>
    );
  };

  const handleAddKol = async () => {
    if (!newKolWallet && !newKolTwitter) return;

    const success = await kols.add(
      newKolWallet || '',
      newKolName || undefined,
      newKolTwitter || undefined,
    );

    if (success) {
      setShowAddKolModal(false);
      setNewKolWallet('');
      setNewKolName('');
      setNewKolTwitter('');
      kols.refresh();
    }
  };

  const handleEnableCopyTrading = async (kolId: string) => {
    const success = await kols.enableCopy(kolId, {
      maxPositionSol: copyConfig.maxPosition,
      delayMs: copyConfig.delay,
    });
    if (success) {
      kols.refresh();
    }
  };

  const handleDisableCopyTrading = async (kolId: string) => {
    const success = await kols.disableCopy(kolId);
    if (success) {
      kols.refresh();
    }
  };

  const handleRemoveKol = async (kolId: string) => {
    const success = await kols.remove(kolId);
    if (success) {
      if (selectedKolId === kolId) {
        setSelectedKolId(null);
        setKolTrades([]);
      }
      kols.refresh();
    }
  };

  // KOL Discovery functions
  const fetchDiscoveryStatus = async () => {
    setDiscoveryLoading(true);
    try {
      const res = await arbFarmService.getKolDiscoveryStatus();
      if (res.success && res.data) {
        setDiscoveryStatus(res.data);
      }
    } catch (err) {
      console.error('Failed to fetch discovery status:', err);
    } finally {
      setDiscoveryLoading(false);
    }
  };

  const fetchDiscoveredKols = async () => {
    setDiscoveryLoading(true);
    try {
      const res = await arbFarmService.listDiscoveredKols({ limit: 50 });
      if (res.success && res.data) {
        setDiscoveredKols(res.data.kols || []);
      }
    } catch (err) {
      console.error('Failed to fetch discovered KOLs:', err);
    } finally {
      setDiscoveryLoading(false);
    }
  };

  const handleStartDiscovery = async () => {
    try {
      await arbFarmService.startKolDiscovery();
      await fetchDiscoveryStatus();
    } catch (err) {
      console.error('Failed to start discovery:', err);
    }
  };

  const handleStopDiscovery = async () => {
    try {
      await arbFarmService.stopKolDiscovery();
      await fetchDiscoveryStatus();
    } catch (err) {
      console.error('Failed to stop discovery:', err);
    }
  };

  const handleScanNow = async () => {
    setScanningKols(true);
    try {
      const res = await arbFarmService.scanForKols();
      if (res.success) {
        await fetchDiscoveredKols();
        await fetchDiscoveryStatus();
      }
    } catch (err) {
      console.error('Failed to scan for KOLs:', err);
    } finally {
      setScanningKols(false);
    }
  };

  const handlePromoteKol = async (walletAddress: string) => {
    setPromotingWallet(walletAddress);
    try {
      const res = await arbFarmService.promoteDiscoveredKol(walletAddress);
      if (res.success) {
        await fetchDiscoveredKols();
        kols.refresh();
      }
    } catch (err) {
      console.error('Failed to promote KOL:', err);
    } finally {
      setPromotingWallet(null);
    }
  };

  const getTrustScoreColor = (score: number): string => {
    if (score >= 70) return '#22c55e';
    if (score >= 40) return '#f59e0b';
    return '#ef4444';
  };

  const renderKOLTrackerView = () => {
    const selectedKol = (kols.data || []).find((k) => k.id === selectedKolId);

    return (
      <div className={styles.kolView}>
        <div className={styles.viewHeader}>
          <button className={styles.backButton} onClick={() => onViewChange('dashboard')}>
            ← Back
          </button>
          <h2>KOL Tracker</h2>
          {kolViewTab === 'tracked' && (
            <button className={styles.createButton} onClick={() => setShowAddKolModal(true)}>
              + Track Wallet
            </button>
          )}
        </div>

        <div className={styles.kolTabs}>
          <button
            className={`${styles.kolTab} ${kolViewTab === 'tracked' ? styles.active : ''}`}
            onClick={() => setKolViewTab('tracked')}
          >
            Tracked Wallets ({(kols.data || []).length})
          </button>
          <button
            className={`${styles.kolTab} ${kolViewTab === 'discovery' ? styles.active : ''}`}
            onClick={() => setKolViewTab('discovery')}
          >
            Discovery ({discoveredKols.length})
          </button>
        </div>

        {kolViewTab === 'tracked' ? (
        <div className={styles.kolLayout}>
          <div className={styles.kolListSection}>
            <h3>
              Tracked Wallets ({(kols.data || []).length})
              <button className={styles.refreshButton} onClick={() => kols.refresh()}>
                🔄
              </button>
            </h3>

            <div className={styles.kolList}>
              {kols.isLoading ? (
                <div className={styles.loadingState}>Loading KOLs...</div>
              ) : (kols.data || []).length === 0 ? (
                <div className={styles.emptyState}>
                  <p>No wallets tracked</p>
                  <p className={styles.emptyHint}>Add a wallet to start tracking</p>
                </div>
              ) : (
                (kols.data || []).map((kol) => (
                  <div
                    key={kol.id}
                    className={`${styles.kolCard} ${selectedKolId === kol.id ? styles.selected : ''}`}
                    onClick={() => setSelectedKolId(kol.id)}
                  >
                    <div className={styles.kolHeader}>
                      <div className={styles.kolIdentity}>
                        <span className={styles.kolName}>
                          {kol.display_name || kol.identifier.slice(0, 8)}...
                        </span>
                        <span className={styles.kolType}>
                          {kol.entity_type === 'wallet' ? '💼' : '🐦'}
                        </span>
                      </div>
                      <div
                        className={styles.trustScore}
                        style={{ backgroundColor: getTrustScoreColor(kol.trust_score) }}
                      >
                        {kol.trust_score.toFixed(0)}
                      </div>
                    </div>

                    <div className={styles.kolStats}>
                      <span>Trades: {kol.total_trades_tracked}</span>
                      <span>Win: {kol.profitable_trades}/{kol.total_trades_tracked}</span>
                      {kol.avg_profit_percent && (
                        <span className={kol.avg_profit_percent >= 0 ? styles.positive : styles.negative}>
                          {kol.avg_profit_percent >= 0 ? '+' : ''}
                          {kol.avg_profit_percent.toFixed(1)}%
                        </span>
                      )}
                    </div>

                    <div className={styles.kolActions}>
                      {kol.copy_trading_enabled ? (
                        <span className={styles.copyEnabled}>✓ Copy Trading</span>
                      ) : (
                        <span className={styles.copyDisabled}>Copy Off</span>
                      )}
                      <span className={kol.is_active ? styles.active : styles.inactive}>
                        {kol.is_active ? '● Active' : '○ Inactive'}
                      </span>
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>

          <div className={styles.kolDetailSection}>
            {selectedKol ? (
              <>
                <div className={styles.kolDetailHeader}>
                  <h3>{selectedKol.display_name || 'KOL Details'}</h3>
                  <div className={styles.kolDetailActions}>
                    <button
                      className={styles.removeButton}
                      onClick={() => handleRemoveKol(selectedKol.id)}
                    >
                      Remove
                    </button>
                  </div>
                </div>

                <div className={styles.kolDetailInfo}>
                  <div className={styles.infoRow}>
                    <span className={styles.label}>Identifier</span>
                    <span className={styles.value}>{selectedKol.identifier}</span>
                  </div>
                  {selectedKol.linked_wallet && (
                    <div className={styles.infoRow}>
                      <span className={styles.label}>Wallet</span>
                      <span className={styles.value}>
                        {selectedKol.linked_wallet.slice(0, 8)}...{selectedKol.linked_wallet.slice(-6)}
                      </span>
                    </div>
                  )}
                  <div className={styles.infoRow}>
                    <span className={styles.label}>Trust Score</span>
                    <span
                      className={styles.value}
                      style={{ color: getTrustScoreColor(selectedKol.trust_score) }}
                    >
                      {selectedKol.trust_score.toFixed(1)}
                    </span>
                  </div>
                  <div className={styles.infoRow}>
                    <span className={styles.label}>Total Trades</span>
                    <span className={styles.value}>{selectedKol.total_trades_tracked}</span>
                  </div>
                  <div className={styles.infoRow}>
                    <span className={styles.label}>Win Rate</span>
                    <span className={styles.value}>
                      {selectedKol.total_trades_tracked > 0
                        ? ((selectedKol.profitable_trades / selectedKol.total_trades_tracked) * 100).toFixed(1)
                        : 0}
                      %
                    </span>
                  </div>
                  {selectedKol.max_drawdown && (
                    <div className={styles.infoRow}>
                      <span className={styles.label}>Max Drawdown</span>
                      <span className={styles.value} style={{ color: '#ef4444' }}>
                        -{selectedKol.max_drawdown.toFixed(1)}%
                      </span>
                    </div>
                  )}
                </div>

                <div className={styles.copyTradingSection}>
                  <h4>Copy Trading</h4>
                  {selectedKol.copy_trading_enabled ? (
                    <div className={styles.copyEnabled}>
                      <div className={styles.copyConfig}>
                        <div className={styles.configRow}>
                          <span>Max Position</span>
                          <span>{selectedKol.copy_config.max_position_sol} SOL</span>
                        </div>
                        <div className={styles.configRow}>
                          <span>Delay</span>
                          <span>{selectedKol.copy_config.delay_ms}ms</span>
                        </div>
                        <div className={styles.configRow}>
                          <span>Min Trust</span>
                          <span>{selectedKol.copy_config.min_trust_score}</span>
                        </div>
                      </div>
                      <button
                        className={styles.disableButton}
                        onClick={() => handleDisableCopyTrading(selectedKol.id)}
                      >
                        Disable Copy Trading
                      </button>
                    </div>
                  ) : (
                    <div className={styles.copySetup}>
                      <div className={styles.configInput}>
                        <label>Max Position (SOL)</label>
                        <input
                          type="number"
                          value={copyConfig.maxPosition}
                          onChange={(e) =>
                            setCopyConfig((c) => ({ ...c, maxPosition: parseFloat(e.target.value) || 0 }))
                          }
                          step="0.1"
                          min="0.01"
                          max="5"
                        />
                      </div>
                      <div className={styles.configInput}>
                        <label>Delay (ms)</label>
                        <input
                          type="number"
                          value={copyConfig.delay}
                          onChange={(e) =>
                            setCopyConfig((c) => ({ ...c, delay: parseInt(e.target.value) || 0 }))
                          }
                          step="100"
                          min="0"
                          max="5000"
                        />
                      </div>
                      <button
                        className={styles.enableButton}
                        onClick={() => handleEnableCopyTrading(selectedKol.id)}
                      >
                        Enable Copy Trading
                      </button>
                    </div>
                  )}
                </div>

                <div className={styles.kolTradesSection}>
                  <h4>Recent Trades</h4>
                  <div className={styles.tradesList}>
                    {kolTradesLoading ? (
                      <div className={styles.loadingState}>Loading trades...</div>
                    ) : kolTrades.length === 0 ? (
                      <div className={styles.emptyState}>No trades recorded</div>
                    ) : (
                      kolTrades.slice(0, 10).map((trade: any) => (
                        <div key={trade.id} className={styles.kolTradeItem}>
                          <div className={styles.tradeHeader}>
                            <span className={`${styles.tradeType} ${styles[trade.trade_type]}`}>
                              {trade.trade_type.toUpperCase()}
                            </span>
                            <span className={styles.tradeAmount}>{trade.amount_sol} SOL</span>
                          </div>
                          <div className={styles.tradeMeta}>
                            <span className={styles.tokenMint}>
                              {trade.token_mint?.slice(0, 8)}...
                            </span>
                            <span className={styles.tradeTime}>
                              {new Date(trade.detected_at).toLocaleTimeString()}
                            </span>
                          </div>
                        </div>
                      ))
                    )}
                  </div>
                </div>
              </>
            ) : (
              <div className={styles.emptyDetail}>
                <p>Select a KOL to view details</p>
              </div>
            )}
          </div>
        </div>
        ) : (
          <div className={styles.discoveryView}>
            <div className={styles.discoveryHeader}>
              <div className={styles.discoveryStatus}>
                {discoveryStatus ? (
                  <>
                    <span className={`${styles.statusDot} ${discoveryStatus.is_running ? styles.running : ''}`} />
                    <span>{discoveryStatus.is_running ? 'Scanning...' : 'Idle'}</span>
                    <span className={styles.statusStat}>
                      Wallets Analyzed: {discoveryStatus.total_wallets_analyzed}
                    </span>
                    <span className={styles.statusStat}>
                      KOLs Found: {discoveryStatus.total_kols_discovered}
                    </span>
                  </>
                ) : (
                  <span>Loading status...</span>
                )}
              </div>
              <div className={styles.discoveryActions}>
                {discoveryStatus?.is_running ? (
                  <button
                    className={styles.stopButton}
                    onClick={handleStopDiscovery}
                  >
                    Stop Discovery
                  </button>
                ) : (
                  <button
                    className={styles.startButton}
                    onClick={handleStartDiscovery}
                  >
                    Start Auto-Discovery
                  </button>
                )}
                <button
                  className={styles.scanButton}
                  onClick={handleScanNow}
                  disabled={scanningKols}
                >
                  {scanningKols ? 'Scanning...' : 'Scan Now'}
                </button>
              </div>
            </div>

            <div className={styles.discoveryInfo}>
              <p>
                Discovery scans pump.fun and DexScreener for wallets with high win rates (65%+),
                minimum 3 trades, and 20%+ average profit. Discovered KOLs can be promoted to your tracked list.
              </p>
            </div>

            <div className={styles.discoveredKolsList}>
              {discoveryLoading ? (
                <div className={styles.loadingState}>Loading discovered KOLs...</div>
              ) : discoveredKols.length === 0 ? (
                <div className={styles.emptyState}>
                  <p>No KOLs discovered yet</p>
                  <p className={styles.emptyHint}>Click "Scan Now" to search for successful traders</p>
                </div>
              ) : (
                discoveredKols.map((kol) => (
                  <div key={kol.wallet_address} className={styles.discoveredKolCard}>
                    <div className={styles.discoveredKolHeader}>
                      <div className={styles.kolIdentity}>
                        <span className={styles.kolName}>
                          {kol.display_name || `${kol.wallet_address.slice(0, 8)}...${kol.wallet_address.slice(-4)}`}
                        </span>
                        <span className={styles.kolSource}>{kol.source}</span>
                      </div>
                      <div
                        className={styles.trustScore}
                        style={{ backgroundColor: getTrustScoreColor(kol.trust_score) }}
                      >
                        {kol.trust_score.toFixed(0)}
                      </div>
                    </div>

                    <div className={styles.discoveredKolStats}>
                      <div className={styles.statItem}>
                        <span className={styles.statLabel}>Win Rate</span>
                        <span className={styles.statValue} style={{ color: '#22c55e' }}>
                          {(kol.win_rate * 100).toFixed(1)}%
                        </span>
                      </div>
                      <div className={styles.statItem}>
                        <span className={styles.statLabel}>Avg Profit</span>
                        <span className={styles.statValue} style={{ color: '#22c55e' }}>
                          +{kol.avg_profit_pct.toFixed(1)}%
                        </span>
                      </div>
                      <div className={styles.statItem}>
                        <span className={styles.statLabel}>Trades</span>
                        <span className={styles.statValue}>{kol.total_trades}</span>
                      </div>
                      <div className={styles.statItem}>
                        <span className={styles.statLabel}>Win Streak</span>
                        <span className={styles.statValue}>{kol.consecutive_wins}</span>
                      </div>
                      <div className={styles.statItem}>
                        <span className={styles.statLabel}>Volume</span>
                        <span className={styles.statValue}>
                          ${kol.total_volume_usd.toLocaleString()}
                        </span>
                      </div>
                    </div>

                    <div className={styles.discoveredKolActions}>
                      <button
                        className={styles.promoteButton}
                        onClick={() => handlePromoteKol(kol.wallet_address)}
                        disabled={promotingWallet === kol.wallet_address}
                      >
                        {promotingWallet === kol.wallet_address ? 'Promoting...' : 'Promote to Tracked'}
                      </button>
                      <span className={styles.discoveredAt}>
                        Found: {new Date(kol.discovered_at).toLocaleString()}
                      </span>
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>
        )}

        {showAddKolModal && (
          <div className={styles.modalOverlay} onClick={() => setShowAddKolModal(false)}>
            <div className={styles.modal} onClick={(e) => e.stopPropagation()}>
              <div className={styles.modalHeader}>
                <h3>Track New Wallet</h3>
                <button className={styles.closeButton} onClick={() => setShowAddKolModal(false)}>
                  ×
                </button>
              </div>
              <div className={styles.modalBody}>
                <div className={styles.formGroup}>
                  <label>Wallet Address</label>
                  <input
                    type="text"
                    placeholder="Enter Solana wallet address..."
                    value={newKolWallet}
                    onChange={(e) => setNewKolWallet(e.target.value)}
                  />
                </div>
                <div className={styles.formGroup}>
                  <label>Display Name (optional)</label>
                  <input
                    type="text"
                    placeholder="e.g., Whale Alpha"
                    value={newKolName}
                    onChange={(e) => setNewKolName(e.target.value)}
                  />
                </div>
                <div className={styles.formGroup}>
                  <label>Twitter Handle (optional)</label>
                  <input
                    type="text"
                    placeholder="@handle"
                    value={newKolTwitter}
                    onChange={(e) => setNewKolTwitter(e.target.value)}
                  />
                </div>
                <p className={styles.modalHint}>
                  When you add a wallet, ArbFarm will register a Helius webhook to track transactions
                  in real-time.
                </p>
              </div>
              <div className={styles.modalActions}>
                <button className={styles.cancelButton} onClick={() => setShowAddKolModal(false)}>
                  Cancel
                </button>
                <button
                  className={styles.confirmButton}
                  onClick={handleAddKol}
                  disabled={!newKolWallet && !newKolTwitter}
                >
                  Track Wallet
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    );
  };

  const handleWalletSetup = async () => {
    if (!walletSetupAddress.trim()) return;

    setSettingsLoading(true);
    try {
      const res = await arbFarmService.setupWallet(walletSetupAddress.trim());
      if (res.success && res.data?.wallet_status) {
        setWalletStatus(res.data.wallet_status);
        setWalletSetupAddress('');
      }
    } catch (err) {
      console.error('Wallet setup failed:', err);
    } finally {
      setSettingsLoading(false);
    }
  };

  const handleDisconnectWallet = async () => {
    setSettingsLoading(true);
    try {
      const res = await arbFarmService.disconnectWallet();
      if (res.success) {
        const walletRes = await arbFarmService.getWalletStatus();
        if (walletRes.success && walletRes.data) {
          setWalletStatus(walletRes.data);
        }
      }
    } catch (err) {
      console.error('Disconnect failed:', err);
    } finally {
      setSettingsLoading(false);
    }
  };

  const handleConnectDevWallet = async () => {
    if (!devModeAvailable) return;

    setConnectingDevWallet(true);
    try {
      const res = await arbFarmService.connectDevWallet();
      if (res.success && res.data?.status) {
        setWalletStatus(res.data.status);
      } else {
        console.error('Dev wallet connect failed:', res.data?.error);
      }
    } catch (err) {
      console.error('Dev wallet connect failed:', err);
    } finally {
      setConnectingDevWallet(false);
    }
  };

  const handleRiskPresetChange = async (presetName: string) => {
    setSettingsLoading(true);
    try {
      const res = await arbFarmService.updateRiskSettings(presetName);
      if (res.success && res.data?.config) {
        setRiskConfig(res.data.config);
      }
    } catch (err) {
      console.error('Failed to update risk settings:', err);
    } finally {
      setSettingsLoading(false);
    }
  };

  const handleSaveCustomRisk = async () => {
    setSavingCustomRisk(true);
    try {
      const config: { max_position_sol?: number; max_concurrent_positions?: number; daily_loss_limit_sol?: number } = {};
      if (customRiskEdit.max_position_sol) {
        config.max_position_sol = parseFloat(customRiskEdit.max_position_sol);
      }
      if (customRiskEdit.max_concurrent_positions) {
        config.max_concurrent_positions = parseInt(customRiskEdit.max_concurrent_positions, 10);
      }
      if (customRiskEdit.daily_loss_limit_sol) {
        config.daily_loss_limit_sol = parseFloat(customRiskEdit.daily_loss_limit_sol);
      }
      const res = await arbFarmService.setCustomRisk(config);
      if (res.success && res.data?.config) {
        setRiskConfig(res.data.config as RiskConfig);
        setCustomRiskEdit({ max_position_sol: '', max_concurrent_positions: '', daily_loss_limit_sol: '' });
      }
    } catch (err) {
      console.error('Failed to save custom risk:', err);
    } finally {
      setSavingCustomRisk(false);
    }
  };

  const renderSettingsView = () => (
    <div className={styles.settingsView}>
      <div className={styles.viewHeader}>
        <button className={styles.backButton} onClick={() => onViewChange('dashboard')}>
          ← Back
        </button>
        <h2>Settings</h2>
      </div>

      <div className={styles.settingsTabs}>
        {(['wallet', 'risk', 'venues', 'api', 'execution'] as const).map((tab) => (
          <button
            key={tab}
            className={`${styles.settingsTab} ${settingsTab === tab ? styles.active : ''}`}
            onClick={() => setSettingsTab(tab)}
          >
            {tab === 'wallet' && '💼 Wallet'}
            {tab === 'risk' && '⚠️ Risk'}
            {tab === 'venues' && '🏛️ Venues'}
            {tab === 'api' && '🔑 API Keys'}
            {tab === 'execution' && '⚡ Execution'}
          </button>
        ))}
      </div>

      {settingsLoading ? (
        <div className={styles.loadingState}>Loading settings...</div>
      ) : (
        <div className={styles.settingsContent}>
          {settingsTab === 'wallet' && (
            <div className={styles.settingsSection}>
              <h3>Turnkey Wallet Delegation</h3>
              <p className={styles.sectionDescription}>
                Connect your wallet to enable autonomous trading. ArbFarm uses Turnkey delegation
                to sign transactions without storing your private keys.
              </p>

              {walletStatus?.is_connected ? (
                <div className={styles.walletConnected}>
                  <div className={styles.walletStatusCard}>
                    <div className={styles.walletHeader}>
                      <span
                        className={styles.statusIndicator}
                        style={{
                          backgroundColor: DELEGATION_STATUS_COLORS[walletStatus.delegation_status],
                        }}
                      />
                      <span className={styles.statusLabel}>
                        {DELEGATION_STATUS_LABELS[walletStatus.delegation_status]}
                      </span>
                    </div>

                    <div className={styles.walletInfo}>
                      <div className={styles.infoRow}>
                        <span className={styles.label}>Wallet Address</span>
                        <span className={styles.value}>
                          {walletStatus.wallet_address?.slice(0, 8)}...
                          {walletStatus.wallet_address?.slice(-6)}
                        </span>
                      </div>
                      <div className={styles.infoRow}>
                        <span className={styles.label}>Balance</span>
                        <span className={styles.value}>
                          {((walletStatus.balance_lamports || 0) / 1e9).toFixed(4)} SOL
                        </span>
                      </div>
                      <div className={styles.infoRow}>
                        <span className={styles.label}>Daily Volume Used</span>
                        <span className={styles.value}>
                          {(walletStatus.daily_usage.total_volume_lamports / 1e9).toFixed(4)} SOL
                        </span>
                      </div>
                      <div className={styles.infoRow}>
                        <span className={styles.label}>Transactions Today</span>
                        <span className={styles.value}>
                          {walletStatus.daily_usage.transaction_count}
                        </span>
                      </div>
                    </div>

                    <div className={styles.policySection}>
                      <h4>Policy Limits</h4>
                      <div className={styles.policyGrid}>
                        <div className={styles.policyItem}>
                          <span className={styles.policyLabel}>Max Transaction</span>
                          <span className={styles.policyValue}>
                            {(walletStatus.policy.max_transaction_amount_lamports / 1e9).toFixed(1)}{' '}
                            SOL
                          </span>
                        </div>
                        <div className={styles.policyItem}>
                          <span className={styles.policyLabel}>Daily Limit</span>
                          <span className={styles.policyValue}>
                            {(walletStatus.policy.daily_volume_limit_lamports / 1e9).toFixed(1)} SOL
                          </span>
                        </div>
                        <div className={styles.policyItem}>
                          <span className={styles.policyLabel}>Max Txs/Day</span>
                          <span className={styles.policyValue}>
                            {walletStatus.policy.max_transactions_per_day}
                          </span>
                        </div>
                        <div className={styles.policyItem}>
                          <span className={styles.policyLabel}>Simulation</span>
                          <span className={styles.policyValue}>
                            {walletStatus.policy.require_simulation ? 'Required' : 'Optional'}
                          </span>
                        </div>
                      </div>
                    </div>

                    <button className={styles.disconnectButton} onClick={handleDisconnectWallet}>
                      Disconnect Wallet
                    </button>
                  </div>
                </div>
              ) : (
                <div className={styles.walletSetup}>
                  {devModeAvailable && (
                    <div className={styles.devWalletSection}>
                      <h4>Dev Wallet Available</h4>
                      <p className={styles.devWalletInfo}>
                        A dev wallet is configured in your environment (ARB_FARM_WALLET_ADDRESS).
                        {hasPrivateKey && ' Private key is available for signing.'}
                      </p>
                      <div className={styles.devWalletAddress}>
                        {devWalletAddress?.slice(0, 8)}...{devWalletAddress?.slice(-6)}
                      </div>
                      <button
                        className={styles.connectDevButton}
                        onClick={handleConnectDevWallet}
                        disabled={connectingDevWallet}
                      >
                        {connectingDevWallet ? 'Connecting...' : 'Connect Dev Wallet'}
                      </button>
                    </div>
                  )}

                  {!devModeAvailable && (
                    <>
                      <div className={styles.setupForm}>
                        <input
                          type="text"
                          placeholder="Enter your Solana wallet address..."
                          value={walletSetupAddress}
                          onChange={(e) => setWalletSetupAddress(e.target.value)}
                          className={styles.walletInput}
                        />
                        <button
                          className={styles.connectButton}
                          onClick={handleWalletSetup}
                          disabled={!walletSetupAddress.trim()}
                        >
                          Connect Wallet
                        </button>
                      </div>
                      <div className={styles.setupInfo}>
                        <h4>How it works:</h4>
                        <ul>
                          <li>Your private keys never leave your wallet</li>
                          <li>Turnkey creates a delegated signing key</li>
                          <li>Policy limits enforce safe trading bounds</li>
                          <li>You can disconnect at any time</li>
                        </ul>
                      </div>
                    </>
                  )}
                </div>
              )}
            </div>
          )}

          {settingsTab === 'risk' && (
            <div className={styles.settingsSection}>
              <h3>Risk Configuration</h3>
              <p className={styles.sectionDescription}>
                Configure position limits, loss thresholds, and trading behavior.
              </p>

              <div className={styles.presetSelector}>
                <h4>Risk Presets</h4>
                <div className={styles.presetGrid}>
                  {riskPresets.map((preset) => (
                    <button
                      key={preset.name}
                      className={`${styles.presetCard} ${
                        riskConfig?.max_position_sol === preset.config.max_position_sol
                          ? styles.active
                          : ''
                      }`}
                      onClick={() => handleRiskPresetChange(preset.name)}
                    >
                      <span className={styles.presetName}>{preset.name}</span>
                      <span className={styles.presetDescription}>{preset.description}</span>
                      <div className={styles.presetStats}>
                        <span>Max: {preset.config.max_position_sol} SOL</span>
                        <span>Loss: {preset.config.daily_loss_limit_sol} SOL</span>
                      </div>
                    </button>
                  ))}
                </div>
              </div>

              {riskConfig && (
                <div className={styles.currentConfig}>
                  <h4>Current Configuration</h4>
                  <div className={styles.configGrid}>
                    <div className={styles.configItem}>
                      <span className={styles.configLabel}>Max Position</span>
                      <span className={styles.configValue}>{riskConfig.max_position_sol} SOL</span>
                    </div>
                    <div className={styles.configItem}>
                      <span className={styles.configLabel}>Daily Loss Limit</span>
                      <span className={styles.configValue}>
                        {riskConfig.daily_loss_limit_sol} SOL
                      </span>
                    </div>
                    <div className={styles.configItem}>
                      <span className={styles.configLabel}>Max Drawdown</span>
                      <span className={styles.configValue}>
                        {riskConfig.max_drawdown_percent}%
                      </span>
                    </div>
                    <div className={styles.configItem}>
                      <span className={styles.configLabel}>Concurrent Positions</span>
                      <span className={styles.configValue}>
                        {riskConfig.max_concurrent_positions}
                      </span>
                    </div>
                    <div className={styles.configItem}>
                      <span className={styles.configLabel}>Per-Token Max</span>
                      <span className={styles.configValue}>
                        {riskConfig.max_position_per_token_sol} SOL
                      </span>
                    </div>
                    <div className={styles.configItem}>
                      <span className={styles.configLabel}>Loss Cooldown</span>
                      <span className={styles.configValue}>
                        {(riskConfig.cooldown_after_loss_ms / 1000).toFixed(0)}s
                      </span>
                    </div>
                    <div className={styles.configItem}>
                      <span className={styles.configLabel}>Volatility Scaling</span>
                      <span className={styles.configValue}>
                        {riskConfig.volatility_scaling_enabled ? 'Enabled' : 'Disabled'}
                      </span>
                    </div>
                    <div className={styles.configItem}>
                      <span className={styles.configLabel}>Auto-Pause</span>
                      <span className={styles.configValue}>
                        {riskConfig.auto_pause_on_drawdown ? 'Enabled' : 'Disabled'}
                      </span>
                    </div>
                  </div>
                </div>
              )}

              <div className={styles.customRiskSection}>
                <h4>Custom Configuration</h4>
                <p className={styles.sectionDescription}>
                  Override specific risk parameters. Leave blank to keep current value.
                </p>
                <div className={styles.customRiskForm}>
                  <div className={styles.inputGroup}>
                    <label>Max Position (SOL)</label>
                    <input
                      type="number"
                      step="0.01"
                      min="0.001"
                      max="10"
                      placeholder={riskConfig?.max_position_sol?.toString() || '0.02'}
                      value={customRiskEdit.max_position_sol}
                      onChange={(e) => setCustomRiskEdit(prev => ({ ...prev, max_position_sol: e.target.value }))}
                    />
                  </div>
                  <div className={styles.inputGroup}>
                    <label>Max Concurrent</label>
                    <input
                      type="number"
                      step="1"
                      min="1"
                      max="50"
                      placeholder={riskConfig?.max_concurrent_positions?.toString() || '2'}
                      value={customRiskEdit.max_concurrent_positions}
                      onChange={(e) => setCustomRiskEdit(prev => ({ ...prev, max_concurrent_positions: e.target.value }))}
                    />
                  </div>
                  <div className={styles.inputGroup}>
                    <label>Daily Loss Limit (SOL)</label>
                    <input
                      type="number"
                      step="0.01"
                      min="0.01"
                      max="100"
                      placeholder={riskConfig?.daily_loss_limit_sol?.toString() || '0.1'}
                      value={customRiskEdit.daily_loss_limit_sol}
                      onChange={(e) => setCustomRiskEdit(prev => ({ ...prev, daily_loss_limit_sol: e.target.value }))}
                    />
                  </div>
                  <button
                    className={styles.saveButton}
                    onClick={handleSaveCustomRisk}
                    disabled={savingCustomRisk || (!customRiskEdit.max_position_sol && !customRiskEdit.max_concurrent_positions && !customRiskEdit.daily_loss_limit_sol)}
                  >
                    {savingCustomRisk ? 'Saving...' : 'Save Custom Config'}
                  </button>
                </div>
              </div>
            </div>
          )}

          {settingsTab === 'venues' && (
            <div className={styles.settingsSection}>
              <h3>Venue Configuration</h3>
              <p className={styles.sectionDescription}>
                Enable or disable trading venues for opportunity scanning.
              </p>

              <div className={styles.venueList}>
                {venues.map((venue) => (
                  <div key={venue.name} className={styles.venueCard}>
                    <div className={styles.venueHeader}>
                      <span className={styles.venueName}>{venue.name}</span>
                      <span
                        className={`${styles.venueStatus} ${venue.enabled ? styles.enabled : styles.disabled}`}
                      >
                        {venue.enabled ? 'Active' : 'Inactive'}
                      </span>
                    </div>
                    <div className={styles.venueInfo}>
                      <span className={styles.venueType}>{venue.venue_type}</span>
                      <span className={styles.venueUrl}>{venue.api_url}</span>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {settingsTab === 'api' && (
            <div className={styles.settingsSection}>
              <h3>API Key Status</h3>
              <p className={styles.sectionDescription}>
                Status of external API integrations. Configure keys in environment variables.
              </p>

              <div className={styles.apiKeyList}>
                {apiKeys.map((key) => (
                  <div key={key.name} className={styles.apiKeyCard}>
                    <div className={styles.apiKeyHeader}>
                      <span className={styles.apiKeyName}>{key.name}</span>
                      <span
                        className={`${styles.apiKeyStatus} ${key.configured ? styles.configured : styles.missing}`}
                      >
                        {key.configured ? '✓ Configured' : '✗ Missing'}
                      </span>
                    </div>
                    {key.required && !key.configured && (
                      <span className={styles.requiredBadge}>Required</span>
                    )}
                  </div>
                ))}
              </div>
            </div>
          )}

          {settingsTab === 'execution' && (
            <div className={styles.settingsSection}>
              <h3>Auto-Execution Settings</h3>
              <p className={styles.sectionDescription}>
                Configure global auto-execution behavior. These settings apply across all strategies
                and can override individual strategy settings.
              </p>

              <div className={styles.executionSettingsCard}>
                <div className={styles.masterToggle}>
                  <div className={styles.toggleHeader}>
                    <label className={styles.toggleSwitch}>
                      <input
                        type="checkbox"
                        checked={executionConfig?.auto_execution_enabled || executionSettings.auto_execute_enabled}
                        onChange={(e) => handleToggleExecution(e.target.checked)}
                        disabled={togglingExecution}
                      />
                      <span className={styles.slider}></span>
                    </label>
                    <div className={styles.toggleInfo}>
                      <span className={styles.toggleLabel}>Auto-Execute Mode</span>
                      <span className={styles.toggleStatus}>
                        {togglingExecution ? (
                          <span className={styles.loading}>Updating...</span>
                        ) : (executionConfig?.auto_execution_enabled || executionSettings.auto_execute_enabled) ? (
                          <span className={styles.enabled}>ENABLED</span>
                        ) : (
                          <span className={styles.disabled}>DISABLED</span>
                        )}
                      </span>
                    </div>
                  </div>
                  <p className={styles.warningText}>
                    When enabled, the agent will automatically execute trades without asking for
                    permission. Ensure your risk parameters are properly configured.
                  </p>
                  {executionConfig && (
                    <div className={styles.configInfo}>
                      <span>Approval timeout: {executionConfig.default_approval_timeout_secs}s</span>
                      <span>Notify Hecate: {executionConfig.notify_hecate_on_pending ? 'Yes' : 'No'}</span>
                    </div>
                  )}
                </div>

                <div className={styles.thresholdSettings}>
                  <h4>Auto-Execute Thresholds</h4>
                  <p className={styles.thresholdDescription}>
                    These thresholds determine when auto-execution is allowed, even in autonomous mode.
                  </p>

                  <div className={styles.thresholdGrid}>
                    <div className={styles.thresholdItem}>
                      <label>Minimum Confidence</label>
                      <div className={styles.inputWithUnit}>
                        <input
                          type="number"
                          step="0.05"
                          min="0"
                          max="1"
                          value={executionSettings.auto_min_confidence}
                          onChange={(e) =>
                            setExecutionSettings((prev) => ({
                              ...prev,
                              auto_min_confidence: parseFloat(e.target.value) || 0,
                            }))
                          }
                        />
                        <span className={styles.percentage}>
                          ({(executionSettings.auto_min_confidence * 100).toFixed(0)}%)
                        </span>
                      </div>
                      <span className={styles.hint}>
                        Only auto-execute when strategy confidence exceeds this threshold
                      </span>
                    </div>

                    <div className={styles.thresholdItem}>
                      <label>Maximum Position for Auto</label>
                      <div className={styles.inputWithUnit}>
                        <input
                          type="number"
                          step="0.1"
                          min="0.01"
                          value={executionSettings.auto_max_position_sol}
                          onChange={(e) =>
                            setExecutionSettings((prev) => ({
                              ...prev,
                              auto_max_position_sol: parseFloat(e.target.value) || 0,
                            }))
                          }
                        />
                        <span className={styles.unit}>SOL</span>
                      </div>
                      <span className={styles.hint}>
                        Positions larger than this always require manual approval
                      </span>
                    </div>
                  </div>
                </div>

                <div className={styles.safetySettings}>
                  <h4>Safety Requirements</h4>
                  <div className={styles.safetyToggles}>
                    <label className={styles.safetyToggle}>
                      <input
                        type="checkbox"
                        checked={executionSettings.require_simulation}
                        onChange={(e) =>
                          setExecutionSettings((prev) => ({
                            ...prev,
                            require_simulation: e.target.checked,
                          }))
                        }
                      />
                      <div className={styles.safetyInfo}>
                        <span className={styles.safetyLabel}>Require Simulation</span>
                        <span className={styles.safetyDesc}>
                          Always simulate trades before execution (recommended)
                        </span>
                      </div>
                    </label>
                  </div>
                </div>

                <div className={styles.executionModeInfo}>
                  <h4>Execution Mode Reference</h4>
                  <div className={styles.modeInfoGrid}>
                    <div className={styles.modeInfoItem}>
                      <span className={styles.modeName}>Agent Directed</span>
                      <span className={styles.modeDescription}>
                        Multi-LLM consensus required for every trade. Most conservative.
                      </span>
                    </div>
                    <div className={styles.modeInfoItem}>
                      <span className={styles.modeName}>Hybrid</span>
                      <span className={styles.modeDescription}>
                        Auto-execute small positions below threshold, consensus for larger ones.
                      </span>
                    </div>
                    <div className={styles.modeInfoItem}>
                      <span className={styles.modeName}>Autonomous</span>
                      <span className={styles.modeDescription}>
                        Full auto-execution based on strategy rules. Use with caution.
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );

  const getModelShortName = (modelId: string): string => {
    const parts = modelId.split('/');
    return parts.length > 1 ? parts[1] : modelId;
  };

  const getProviderIcon = (provider: string): string => {
    switch (provider.toLowerCase()) {
      case 'anthropic': return '🔮';
      case 'openai': return '🧠';
      case 'meta-llama': return '🦙';
      case 'google': return '🌐';
      case 'mistralai': return '🌪️';
      default: return '🤖';
    }
  };

  const renderResearchView = () => {
    const renderInjectTab = () => (
      <div className={styles.injectTab}>
        <div className={styles.urlSubmitSection}>
          <h3>Submit URL for Analysis</h3>
          <p className={styles.sectionDescription}>
            Paste a URL containing trading alpha (Twitter threads, blog posts, research papers).
            The LLM will extract strategy parameters automatically.
          </p>
          <div className={styles.urlInputRow}>
            <input
              type="text"
              className={styles.urlInput}
              placeholder="https://twitter.com/... or any URL with trading alpha"
              value={researchUrl}
              onChange={(e) => setResearchUrl(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleSubmitResearchUrl()}
            />
            <button
              className={styles.analyzeButton}
              onClick={handleSubmitResearchUrl}
              disabled={researchUrlSubmitting || !researchUrl.trim()}
            >
              {researchUrlSubmitting ? 'Analyzing...' : 'Analyze'}
            </button>
          </div>
        </div>

        {researchResult && (
          <div className={styles.extractedStrategySection}>
            <h3>Extraction Result</h3>
            <div className={styles.ingestResultCard}>
              <div className={styles.ingestHeader}>
                <span className={`${styles.statusBadge} ${styles[researchResult.ingest_result?.status?.toLowerCase() || 'pending']}`}>
                  {researchResult.ingest_result?.status || 'Unknown'}
                </span>
                <span className={styles.sourceUrl}>
                  {researchResult.ingest_result?.url?.slice(0, 50)}...
                </span>
              </div>
              {researchResult.ingest_result?.content_preview && (
                <p className={styles.contentPreview}>
                  {researchResult.ingest_result.content_preview}
                </p>
              )}
            </div>

            {researchResult.extracted_strategy && (
              <div className={styles.strategyCard}>
                <div className={styles.strategyHeader}>
                  <h4>{researchResult.extracted_strategy.name}</h4>
                  <div className={styles.strategyBadges}>
                    <span className={`${styles.typeBadge} ${styles[researchResult.extracted_strategy.strategy_type?.toLowerCase()]}`}>
                      {researchResult.extracted_strategy.strategy_type}
                    </span>
                    <span className={`${styles.confidenceBadge} ${styles[researchResult.extracted_strategy.confidence?.toLowerCase()]}`}>
                      {researchResult.extracted_strategy.confidence} Confidence ({(researchResult.extracted_strategy.confidence_score * 100).toFixed(0)}%)
                    </span>
                  </div>
                </div>
                <p className={styles.strategyDescription}>
                  {researchResult.extracted_strategy.description}
                </p>
                <div className={styles.conditionsGrid}>
                  <div className={styles.conditionSection}>
                    <h5>Entry Conditions</h5>
                    <ul className={styles.conditionList}>
                      {researchResult.extracted_strategy.entry_conditions?.map((cond: any, i: number) => (
                        <li key={i}>
                          <span className={styles.conditionField}>{cond.field}</span>
                          <span className={styles.conditionOp}>{cond.operator}</span>
                          <span className={styles.conditionValue}>{cond.value}</span>
                        </li>
                      ))}
                    </ul>
                  </div>
                  <div className={styles.conditionSection}>
                    <h5>Exit Conditions</h5>
                    <ul className={styles.conditionList}>
                      {researchResult.extracted_strategy.exit_conditions?.map((cond: any, i: number) => (
                        <li key={i}>
                          <span className={styles.conditionField}>{cond.field}</span>
                          <span className={styles.conditionOp}>{cond.operator}</span>
                          <span className={styles.conditionValue}>{cond.value}</span>
                        </li>
                      ))}
                    </ul>
                  </div>
                </div>
                {researchResult.extracted_strategy.risk_params && (
                  <div className={styles.riskParamsSection}>
                    <h5>Risk Parameters</h5>
                    <div className={styles.riskParamsGrid}>
                      <div className={styles.riskParam}>
                        <span className={styles.paramLabel}>Max Position</span>
                        <span className={styles.paramValue}>
                          {researchResult.extracted_strategy.risk_params.max_position_sol} SOL
                        </span>
                      </div>
                      <div className={styles.riskParam}>
                        <span className={styles.paramLabel}>Stop Loss</span>
                        <span className={styles.paramValue}>
                          {researchResult.extracted_strategy.risk_params.stop_loss_percent}%
                        </span>
                      </div>
                      <div className={styles.riskParam}>
                        <span className={styles.paramLabel}>Take Profit</span>
                        <span className={styles.paramValue}>
                          {researchResult.extracted_strategy.risk_params.take_profit_percent}%
                        </span>
                      </div>
                    </div>
                  </div>
                )}
                {researchResult.extracted_strategy.tokens_mentioned?.length > 0 && (
                  <div className={styles.tokensMentioned}>
                    <h5>Tokens Mentioned</h5>
                    <div className={styles.tokenChips}>
                      {researchResult.extracted_strategy.tokens_mentioned.map((token: string, i: number) => (
                        <span key={i} className={styles.tokenChip}>{token}</span>
                      ))}
                    </div>
                  </div>
                )}
                <div className={styles.strategyActions}>
                  <button
                    className={styles.createStrategyButton}
                    onClick={() => handleCreateStrategyFromResearch(researchResult.extracted_strategy)}
                  >
                    Create Strategy from This
                  </button>
                  <button
                    className={styles.backtestButton}
                    onClick={() => handleRunBacktest(researchResult.extracted_strategy)}
                  >
                    Backtest First
                  </button>
                </div>
              </div>
            )}
          </div>
        )}
      </div>
    );

    const renderDiscoveriesTab = () => (
      <div className={styles.discoveriesTab}>
        <div className={styles.discoveriesHeader}>
          <h3>Extracted Strategies</h3>
          <button className={styles.refreshButton} onClick={fetchDiscoveries}>🔄</button>
        </div>
        {discoveries.length === 0 ? (
          <div className={styles.emptyState}>
            <p>No discoveries yet</p>
            <p className={styles.emptyHint}>
              Submit URLs in the Inject tab to extract trading strategies
            </p>
          </div>
        ) : (
          <div className={styles.discoveriesList}>
            {discoveries.map((discovery: any) => (
              <div key={discovery.id} className={styles.discoveryCard}>
                <div className={styles.discoveryHeader}>
                  <h4>{discovery.name || 'Unnamed Strategy'}</h4>
                  <span className={`${styles.statusBadge} ${styles[discovery.status?.toLowerCase()]}`}>
                    {discovery.status}
                  </span>
                </div>
                <p className={styles.discoveryDescription}>{discovery.description}</p>
                <div className={styles.discoveryMeta}>
                  <span className={styles.strategyType}>{discovery.strategy_type}</span>
                  <span className={styles.confidence}>
                    {discovery.confidence} ({(discovery.confidence_score * 100).toFixed(0)}%)
                  </span>
                  <span className={styles.timestamp}>
                    {new Date(discovery.extracted_at).toLocaleDateString()}
                  </span>
                </div>
                <div className={styles.discoveryActions}>
                  <button
                    className={styles.approveButton}
                    onClick={() => handleApproveDiscovery(discovery.id)}
                  >
                    ✓ Approve
                  </button>
                  <button
                    className={styles.rejectButton}
                    onClick={() => handleRejectDiscovery(discovery.id)}
                  >
                    ✗ Reject
                  </button>
                  <button
                    className={styles.backtestButton}
                    onClick={() => handleRunBacktest(discovery)}
                  >
                    Backtest
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    );

    const renderSourcesTab = () => (
      <div className={styles.sourcesTab}>
        <div className={styles.sourcesHeader}>
          <h3>Monitored Sources</h3>
          <button className={styles.refreshButton} onClick={fetchMonitorSources}>🔄</button>
        </div>
        {monitorStats && (
          <div className={styles.monitorStatsBar}>
            <div className={styles.statItem}>
              <span className={styles.statLabel}>Active Sources</span>
              <span className={styles.statValue}>{monitorStats.active_sources || 0}</span>
            </div>
            <div className={styles.statItem}>
              <span className={styles.statLabel}>Alerts Today</span>
              <span className={styles.statValue}>{monitorStats.alerts_today || 0}</span>
            </div>
            <div className={styles.statItem}>
              <span className={styles.statLabel}>Last Check</span>
              <span className={styles.statValue}>
                {monitorStats.last_check_at ? new Date(monitorStats.last_check_at).toLocaleTimeString() : 'N/A'}
              </span>
            </div>
          </div>
        )}
        {monitorSources.length === 0 ? (
          <div className={styles.emptyState}>
            <p>No monitored sources</p>
            <p className={styles.emptyHint}>
              Add Twitter accounts or other sources to monitor for alpha
            </p>
          </div>
        ) : (
          <div className={styles.sourcesList}>
            {monitorSources.map((source: any) => (
              <div key={source.id} className={styles.sourceCard}>
                <div className={styles.sourceHeader}>
                  <span className={styles.sourceIcon}>
                    {source.source_type === 'twitter' ? '𝕏' : '🔗'}
                  </span>
                  <span className={styles.sourceName}>{source.handle || source.url}</span>
                  <span className={`${styles.statusDot} ${source.is_active ? styles.active : styles.inactive}`} />
                </div>
                <div className={styles.sourceMeta}>
                  <span className={styles.trackType}>{source.track_type}</span>
                  <span className={styles.lastChecked}>
                    Last: {source.last_checked_at ? new Date(source.last_checked_at).toLocaleTimeString() : 'Never'}
                  </span>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    );

    const renderConsensusTab = () => (
      <div className={styles.consensusTab}>
        {consensusStats && (
          <div className={styles.consensusStatsBar}>
            <div className={styles.statItem}>
              <span className={styles.statLabel}>Total Decisions</span>
              <span className={styles.statValue}>{consensusStats.total_decisions}</span>
            </div>
            <div className={styles.statItem}>
              <span className={styles.statLabel}>Approved</span>
              <span className={`${styles.statValue} ${styles.positive}`}>
                {consensusStats.approved_count}
              </span>
            </div>
            <div className={styles.statItem}>
              <span className={styles.statLabel}>Rejected</span>
              <span className={`${styles.statValue} ${styles.negative}`}>
                {consensusStats.rejected_count}
              </span>
            </div>
            <div className={styles.statItem}>
              <span className={styles.statLabel}>Avg Agreement</span>
              <span className={styles.statValue}>
                {(consensusStats.average_agreement * 100).toFixed(1)}%
              </span>
            </div>
            <div className={styles.statItem}>
              <span className={styles.statLabel}>Avg Latency</span>
              <span className={styles.statValue}>
                {consensusStats.average_latency_ms.toFixed(0)}ms
              </span>
            </div>
            <div className={styles.statItem}>
              <span className={styles.statLabel}>Last 24h</span>
              <span className={styles.statValue}>{consensusStats.decisions_last_24h}</span>
            </div>
          </div>
        )}

        <div className={styles.researchGrid}>
          <div className={styles.gridSection}>
            <div className={styles.sectionHeader}>
              <h3>Test Consensus</h3>
            </div>
            <div className={styles.testConsensusSection}>
              <p className={styles.sectionDescription}>
                Test the multi-LLM consensus system with a sample trade decision.
              </p>
              <button
                className={styles.testButton}
                onClick={handleTestConsensus}
                disabled={testConsensusLoading}
              >
                {testConsensusLoading ? 'Requesting...' : 'Request Test Consensus'}
              </button>

              {testConsensusResult && (
                <div className={styles.testResult}>
                  <div className={styles.testResultHeader}>
                    <span
                      className={`${styles.decisionBadge} ${testConsensusResult.approved ? styles.approved : styles.rejected}`}
                    >
                      {testConsensusResult.approved ? '✓ APPROVED' : '✗ REJECTED'}
                    </span>
                    <span className={styles.agreementScore}>
                      {(testConsensusResult.agreement_score * 100).toFixed(1)}% agreement
                    </span>
                  </div>
                  <div className={styles.votesPreview}>
                    {testConsensusResult.model_votes?.slice(0, 3).map((vote: any) => (
                      <div
                        key={vote.model}
                        className={`${styles.voteChip} ${vote.approved ? styles.approve : styles.reject}`}
                      >
                        {getModelShortName(vote.model)}:{' '}
                        {vote.approved ? '✓' : '✗'}
                      </div>
                    ))}
                  </div>
                  <p className={styles.reasoningSummary}>{testConsensusResult.reasoning_summary}</p>
                  <span className={styles.latency}>
                    Total latency: {testConsensusResult.total_latency_ms}ms
                  </span>
                </div>
              )}
            </div>

            <div className={styles.modelsSection}>
              <h4>Available Models</h4>
              <div className={styles.modelsList}>
                {availableModels.map((model) => (
                  <div key={model.id} className={styles.modelCard}>
                    <span className={styles.modelIcon}>{getProviderIcon(model.provider)}</span>
                    <div className={styles.modelInfo}>
                      <span className={styles.modelName}>{model.name}</span>
                      <span className={styles.modelProvider}>{model.provider}</span>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>

          <div className={styles.gridSection}>
            <div className={styles.sectionHeader}>
              <h3>Consensus History</h3>
              <span className={styles.statsLabel}>{consensusDecisions.length} decisions</span>
            </div>
            <div className={styles.consensusHistory}>
              {consensusDecisions.length === 0 ? (
                <div className={styles.emptyState}>
                  <p>No consensus decisions yet</p>
                  <p className={styles.emptyHint}>
                    Agent-directed edges will use multi-LLM consensus for approval
                  </p>
                </div>
              ) : (
                consensusDecisions.map((decision) => (
                  <div
                    key={decision.id}
                    className={`${styles.consensusCard} ${selectedConsensusId === decision.id ? styles.selected : ''}`}
                    onClick={() => handleViewConsensusDetail(decision.id)}
                  >
                    <div className={styles.consensusHeader}>
                      <span
                        className={`${styles.decisionBadge} ${decision.result.approved ? styles.approved : styles.rejected}`}
                      >
                        {decision.result.approved ? '✓' : '✗'}
                      </span>
                      <span className={styles.edgeId}>
                        Edge: {decision.edge_id.slice(0, 8)}...
                      </span>
                      <span className={styles.timestamp}>
                        {new Date(decision.created_at).toLocaleTimeString()}
                      </span>
                    </div>
                    <div className={styles.consensusStats}>
                      <span>
                        Agreement: {(decision.result.agreement_score * 100).toFixed(0)}%
                      </span>
                      <span>
                        Confidence: {(decision.result.weighted_confidence * 100).toFixed(0)}%
                      </span>
                      <span>
                        Votes: {decision.result.model_votes?.length || 0}
                      </span>
                    </div>
                    <p className={styles.reasoningPreview}>
                      {decision.result.reasoning_summary?.slice(0, 100)}...
                    </p>
                  </div>
                ))
              )}
            </div>
          </div>
        </div>

        {consensusDetail && selectedConsensusId && (
          <div className={styles.modalOverlay} onClick={() => setSelectedConsensusId(null)}>
            <div className={styles.modal} onClick={(e) => e.stopPropagation()}>
              <div className={styles.modalHeader}>
                <h3>Consensus Decision Details</h3>
                <button
                  className={styles.closeButton}
                  onClick={() => setSelectedConsensusId(null)}
                >
                  ×
                </button>
              </div>
              <div className={styles.modalBody}>
                <div className={styles.consensusDetailHeader}>
                  <span
                    className={`${styles.decisionBadge} ${styles.large} ${consensusDetail.approved ? styles.approved : styles.rejected}`}
                  >
                    {consensusDetail.approved ? '✓ APPROVED' : '✗ REJECTED'}
                  </span>
                  <div className={styles.detailStats}>
                    <div>
                      <span className={styles.label}>Agreement Score</span>
                      <span className={styles.value}>
                        {(consensusDetail.agreement_score * 100).toFixed(1)}%
                      </span>
                    </div>
                    <div>
                      <span className={styles.label}>Weighted Confidence</span>
                      <span className={styles.value}>
                        {(consensusDetail.weighted_confidence * 100).toFixed(1)}%
                      </span>
                    </div>
                    <div>
                      <span className={styles.label}>Total Latency</span>
                      <span className={styles.value}>{consensusDetail.total_latency_ms}ms</span>
                    </div>
                  </div>
                </div>

                <div className={styles.reasoningSection}>
                  <h4>Reasoning Summary</h4>
                  <p>{consensusDetail.reasoning_summary}</p>
                </div>

                <div className={styles.votesSection}>
                  <h4>Model Votes</h4>
                  <div className={styles.votesList}>
                    {consensusDetail.model_votes?.map((vote: any, index: number) => (
                      <div
                        key={index}
                        className={`${styles.voteCard} ${vote.approved ? styles.approve : styles.reject}`}
                      >
                        <div className={styles.voteHeader}>
                          <span className={styles.modelName}>
                            {getProviderIcon(vote.model.split('/')[0])} {getModelShortName(vote.model)}
                          </span>
                          <span
                            className={`${styles.voteBadge} ${vote.approved ? styles.approve : styles.reject}`}
                          >
                            {vote.approved ? '✓ Approve' : '✗ Reject'}
                          </span>
                        </div>
                        <div className={styles.voteStats}>
                          <span>Confidence: {(vote.confidence * 100).toFixed(0)}%</span>
                          <span>Latency: {vote.latency_ms}ms</span>
                        </div>
                        <p className={styles.voteReasoning}>{vote.reasoning}</p>
                      </div>
                    ))}
                  </div>
                </div>

                <div className={styles.contextSection}>
                  <h4>Edge Context</h4>
                  <pre className={styles.contextPre}>{consensusDetail.edge_context}</pre>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    );

    return (
      <div className={styles.researchView}>
        <div className={styles.viewHeader}>
          <button className={styles.backButton} onClick={() => onViewChange('dashboard')}>
            ← Back
          </button>
          <h2>Research & Discovery</h2>
          <button className={styles.refreshButton} onClick={() => {
            if (researchTab === 'consensus') fetchConsensusData();
            else if (researchTab === 'discoveries') fetchDiscoveries();
            else if (researchTab === 'sources') fetchMonitorSources();
          }}>
            🔄
          </button>
        </div>

        <div className={styles.researchTabs}>
          <button
            className={`${styles.tabButton} ${researchTab === 'inject' ? styles.active : ''}`}
            onClick={() => setResearchTab('inject')}
          >
            🔗 Inject URL
          </button>
          <button
            className={`${styles.tabButton} ${researchTab === 'discoveries' ? styles.active : ''}`}
            onClick={() => { setResearchTab('discoveries'); fetchDiscoveries(); }}
          >
            📋 Discoveries
          </button>
          <button
            className={`${styles.tabButton} ${researchTab === 'sources' ? styles.active : ''}`}
            onClick={() => { setResearchTab('sources'); fetchMonitorSources(); }}
          >
            📡 Sources
          </button>
          <button
            className={`${styles.tabButton} ${researchTab === 'consensus' ? styles.active : ''}`}
            onClick={() => { setResearchTab('consensus'); fetchConsensusData(); }}
          >
            🤝 Consensus
          </button>
        </div>

        {researchLoading ? (
          <div className={styles.loadingState}>Loading...</div>
        ) : (
          <div className={styles.researchTabContent}>
            {researchTab === 'inject' && renderInjectTab()}
            {researchTab === 'discoveries' && renderDiscoveriesTab()}
            {researchTab === 'sources' && renderSourcesTab()}
            {researchTab === 'consensus' && renderConsensusTab()}
          </div>
        )}
      </div>
    );
  };

  const formatAddress = (address: string): string => {
    if (!address || address.length < 12) return address;

    return `${address.slice(0, 6)}...${address.slice(-4)}`;
  };

  const renderCowsView = () => {
    const filteredCows = (cows.data || []).filter((cow) => {
      if (cowsFilter === 'forkable') return cow.is_forkable;

      return true;
    });

    return (
      <div className={styles.cowsView}>
        <div className={styles.viewHeader}>
          <button className={styles.backButton} onClick={() => onViewChange('dashboard')}>
            ← Back
          </button>
          <h2>Strategy COWs</h2>
          <button className={styles.createButton} onClick={() => {}}>
            + Mint COW
          </button>
        </div>

        {cows.stats && (
          <div className={styles.cowsStatsBar}>
            <div className={styles.statItem}>
              <span className={styles.statLabel}>Total COWs</span>
              <span className={styles.statValue}>{cows.stats.total_cows}</span>
            </div>
            <div className={styles.statItem}>
              <span className={styles.statLabel}>Total Forks</span>
              <span className={styles.statValue}>{cows.stats.total_forks}</span>
            </div>
            <div className={styles.statItem}>
              <span className={styles.statLabel}>Total Revenue</span>
              <span className={styles.statValue}>
                {(cows.stats.total_revenue_lamports / 1_000_000_000).toFixed(2)} SOL
              </span>
            </div>
            <div className={styles.statItem}>
              <span className={styles.statLabel}>Avg Win Rate</span>
              <span className={styles.statValue}>
                {(cows.stats.avg_win_rate * 100).toFixed(1)}%
              </span>
            </div>
          </div>
        )}

        <div className={styles.filterBar}>
          {(['all', 'mine', 'forkable'] as const).map((filter) => (
            <button
              key={filter}
              className={`${styles.filterChip} ${cowsFilter === filter ? styles.active : ''}`}
              onClick={() => setCowsFilter(filter)}
            >
              {filter === 'all' && 'All COWs'}
              {filter === 'mine' && 'My COWs'}
              {filter === 'forkable' && 'Forkable'}
            </button>
          ))}
        </div>

        <div className={styles.cowsGrid}>
          {cows.isLoading ? (
            <div className={styles.loadingState}>Loading COWs...</div>
          ) : filteredCows.length === 0 ? (
            <div className={styles.emptyState}>
              <p>No COWs found</p>
              <p className={styles.emptyHint}>
                Create your first COW to package and sell your strategies
              </p>
            </div>
          ) : (
            filteredCows.map((cow) => (
              <div key={cow.id} className={styles.cowCard}>
                <div className={styles.cowHeader}>
                  <div className={styles.cowTitle}>
                    <h3>{cow.name}</h3>
                    <span
                      className={styles.riskBadge}
                      style={{ backgroundColor: RISK_PROFILE_COLORS[cow.risk_profile_type] }}
                    >
                      {RISK_PROFILE_LABELS[cow.risk_profile_type]}
                    </span>
                  </div>
                  <div className={styles.cowPrice}>
                    {cow.is_free ? (
                      <span className={styles.freeTag}>Free</span>
                    ) : (
                      <span className={styles.priceTag}>{cow.price_sol} SOL</span>
                    )}
                  </div>
                </div>

                <p className={styles.cowDescription}>{cow.description}</p>

                <div className={styles.cowMeta}>
                  <span className={styles.creator}>
                    By {formatAddress(cow.creator_wallet)}
                  </span>
                  <div className={styles.venueIcons}>
                    {cow.venue_types.map((venue) => (
                      <span key={venue} title={venue}>
                        {VENUE_TYPE_ICONS[venue]}
                      </span>
                    ))}
                  </div>
                </div>

                <div className={styles.cowStats}>
                  <div className={styles.cowStat}>
                    <span className={styles.statLabel}>Strategies</span>
                    <span className={styles.statValue}>{cow.strategy_count}</span>
                  </div>
                  <div className={styles.cowStat}>
                    <span className={styles.statLabel}>Forks</span>
                    <span className={styles.statValue}>{cow.fork_count}</span>
                  </div>
                  <div className={styles.cowStat}>
                    <span className={styles.statLabel}>Profit</span>
                    <span
                      className={`${styles.statValue} ${cow.total_profit_sol >= 0 ? styles.positive : styles.negative}`}
                    >
                      {cow.total_profit_sol >= 0 ? '+' : ''}
                      {cow.total_profit_sol.toFixed(2)} SOL
                    </span>
                  </div>
                  <div className={styles.cowStat}>
                    <span className={styles.statLabel}>Win Rate</span>
                    <span className={styles.statValue}>
                      {(cow.win_rate * 100).toFixed(0)}%
                    </span>
                  </div>
                </div>

                {cow.rating && (
                  <div className={styles.cowRating}>
                    {'★'.repeat(Math.round(cow.rating))}
                    {'☆'.repeat(5 - Math.round(cow.rating))}
                    <span className={styles.ratingValue}>{cow.rating.toFixed(1)}</span>
                  </div>
                )}

                <div className={styles.cowActions}>
                  <button
                    className={styles.viewButton}
                    onClick={() => cows.getCow(cow.id)}
                  >
                    View Details
                  </button>
                  {cow.is_forkable && (
                    <button
                      className={styles.forkButton}
                      onClick={() => setSelectedCowForFork(cow)}
                    >
                      Fork
                    </button>
                  )}
                </div>
              </div>
            ))
          )}
        </div>

        {selectedCowForFork && (
          <div className={styles.modalOverlay} onClick={() => setSelectedCowForFork(null)}>
            <div className={styles.modal} onClick={(e) => e.stopPropagation()}>
              <div className={styles.modalHeader}>
                <h3>Fork: {selectedCowForFork.name}</h3>
                <button
                  className={styles.closeButton}
                  onClick={() => setSelectedCowForFork(null)}
                >
                  ×
                </button>
              </div>
              <div className={styles.modalBody}>
                <p>
                  You are about to fork this COW. This will create a copy of all strategies
                  with optional engram inheritance.
                </p>
                {!selectedCowForFork.is_free && (
                  <p className={styles.forkCost}>
                    Fork cost: <strong>{selectedCowForFork.price_sol} SOL</strong>
                  </p>
                )}
                <div className={styles.forkOptions}>
                  <label className={styles.checkbox}>
                    <input type="checkbox" defaultChecked />
                    <span>Inherit engrams (learned patterns)</span>
                  </label>
                </div>
              </div>
              <div className={styles.modalActions}>
                <button
                  className={styles.cancelButton}
                  onClick={() => setSelectedCowForFork(null)}
                >
                  Cancel
                </button>
                <button
                  className={styles.confirmButton}
                  onClick={async () => {
                    await cows.fork(selectedCowForFork.id, {
                      inherit_engrams: true,
                    });
                    setSelectedCowForFork(null);
                  }}
                >
                  Fork COW
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    );
  };

  const renderHeliusView = () => {
    return (
      <div className={styles.heliusView}>
        <div className={styles.viewHeader}>
          <button className={styles.backButton} onClick={() => onViewChange('dashboard')}>
            ← Back
          </button>
          <h2>⚡ Helius Integration</h2>
          <span className={styles.apiKeyStatus}>
            API Key: {heliusStatus?.api_key_configured ? '✅ Configured' : '❌ Not Configured'}
          </span>
        </div>

        {heliusLoading ? (
          <div className={styles.loadingState}>Loading Helius data...</div>
        ) : (
          <div className={styles.heliusSections}>
            {/* LaserStream Section */}
            <div className={styles.heliusSection}>
              <h3>⚡ LaserStream (Real-Time)</h3>
              <div className={styles.sectionContent}>
                <div className={styles.statusRow}>
                  <span className={styles.statusLabel}>Connection Status:</span>
                  <span className={laserStreamStatus?.connected ? styles.statusConnected : styles.statusDisconnected}>
                    {laserStreamStatus?.connected ? '🟢 Connected' : '🔴 Disconnected'}
                  </span>
                </div>
                <div className={styles.metricsRow}>
                  <div className={styles.metric}>
                    <span className={styles.metricLabel}>Avg Latency</span>
                    <span className={styles.metricValue}>{laserStreamStatus?.avg_latency_ms.toFixed(1) || 0}ms</span>
                  </div>
                  <div className={styles.metric}>
                    <span className={styles.metricLabel}>Events/sec</span>
                    <span className={styles.metricValue}>{laserStreamStatus?.events_per_second.toFixed(1) || 0}</span>
                  </div>
                </div>
                {laserStreamStatus?.subscriptions && laserStreamStatus.subscriptions.length > 0 && (
                  <div className={styles.subscriptionsList}>
                    <h4>Active Subscriptions</h4>
                    {laserStreamStatus.subscriptions.map((sub) => (
                      <div key={sub.id} className={styles.subscriptionItem}>
                        <span className={styles.subType}>{sub.subscription_type}</span>
                        {sub.address && <span className={styles.subAddress}>{sub.address.slice(0, 8)}...</span>}
                        <span className={styles.subEvents}>{sub.events_received} events</span>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </div>

            {/* Priority Fee Section */}
            <div className={styles.heliusSection}>
              <h3>💰 Priority Fee Estimator</h3>
              <div className={styles.sectionContent}>
                {priorityFees ? (
                  <>
                    <div className={styles.priorityFeeGrid}>
                      {(['min', 'low', 'medium', 'high', 'very_high', 'unsafe_max'] as const).map((level) => (
                        <div
                          key={level}
                          className={`${styles.feeLevel} ${priorityFees.recommended === priorityFees[level] ? styles.recommended : ''}`}
                        >
                          <span className={styles.levelName}>{PRIORITY_LEVEL_LABELS[level]}</span>
                          <span className={styles.feeAmount} style={{ color: PRIORITY_LEVEL_COLORS[level] }}>
                            {priorityFees[level].toLocaleString()} lamports
                          </span>
                        </div>
                      ))}
                    </div>
                    <div className={styles.recommendedFee}>
                      <span>Recommended:</span>
                      <strong>{priorityFees.recommended.toLocaleString()} lamports</strong>
                    </div>
                  </>
                ) : (
                  <div className={styles.emptyState}>No priority fee data available</div>
                )}
              </div>
            </div>

            {/* Helius Sender Section */}
            <div className={styles.heliusSection}>
              <h3>🚀 Helius Sender (Fast TX)</h3>
              <div className={styles.sectionContent}>
                <div className={styles.senderMetrics}>
                  <div className={styles.metric}>
                    <span className={styles.metricLabel}>TXs Sent</span>
                    <span className={styles.metricValue}>{senderStats?.total_sent || 0}</span>
                  </div>
                  <div className={styles.metric}>
                    <span className={styles.metricLabel}>Confirmed</span>
                    <span className={styles.metricValue}>{senderStats?.total_confirmed || 0}</span>
                  </div>
                  <div className={styles.metric}>
                    <span className={styles.metricLabel}>Success Rate</span>
                    <span className={styles.metricValue}>{(senderStats?.success_rate || 0).toFixed(1)}%</span>
                  </div>
                  <div className={styles.metric}>
                    <span className={styles.metricLabel}>Avg Landing</span>
                    <span className={styles.metricValue}>{(senderStats?.avg_landing_ms || 0).toFixed(0)}ms</span>
                  </div>
                </div>
                <div className={styles.pingSection}>
                  <button
                    className={styles.pingButton}
                    onClick={handlePingSender}
                    disabled={pingLoading}
                  >
                    {pingLoading ? 'Pinging...' : '🏓 Ping Sender'}
                  </button>
                  {pingResult !== null && (
                    <span className={styles.pingResult}>Latency: {pingResult}ms</span>
                  )}
                </div>
              </div>
            </div>

            {/* DAS API Section */}
            <div className={styles.heliusSection}>
              <h3>🔍 DAS API (Token Metadata)</h3>
              <div className={styles.sectionContent}>
                <div className={styles.dasLookup}>
                  <input
                    type="text"
                    placeholder="Enter mint address..."
                    value={dasMintInput}
                    onChange={(e) => setDasMintInput(e.target.value)}
                    className={styles.dasInput}
                  />
                  <button
                    className={styles.dasButton}
                    onClick={handleDasLookup}
                    disabled={dasLoading || !dasMintInput.trim()}
                  >
                    {dasLoading ? 'Looking up...' : 'Lookup'}
                  </button>
                </div>
                {dasResult && (
                  <div className={styles.tokenMetadata}>
                    <div className={styles.tokenHeader}>
                      {dasResult.image_uri && (
                        <img src={dasResult.image_uri} alt={dasResult.name} className={styles.tokenImage} />
                      )}
                      <div className={styles.tokenInfo}>
                        <h4>{dasResult.name} ({dasResult.symbol})</h4>
                        <span className={styles.mintAddress}>{dasResult.mint}</span>
                      </div>
                    </div>
                    <div className={styles.tokenDetails}>
                      <div className={styles.detailRow}>
                        <span>Decimals:</span>
                        <span>{dasResult.decimals}</span>
                      </div>
                      <div className={styles.detailRow}>
                        <span>Supply:</span>
                        <span>{dasResult.supply.toLocaleString()}</span>
                      </div>
                      {dasResult.creators && dasResult.creators.length > 0 && (
                        <div className={styles.detailRow}>
                          <span>Creators:</span>
                          <span>{dasResult.creators.length}</span>
                        </div>
                      )}
                      {dasResult.collection && (
                        <div className={styles.detailRow}>
                          <span>Collection:</span>
                          <span>{dasResult.collection.name || dasResult.collection.address.slice(0, 8) + '...'}</span>
                        </div>
                      )}
                    </div>
                  </div>
                )}
              </div>
            </div>

            {/* Configuration Section */}
            <div className={styles.heliusSection}>
              <h3>⚙️ Helius Configuration</h3>
              <div className={styles.sectionContent}>
                {heliusConfig && (
                  <div className={styles.configOptions}>
                    <div className={styles.configRow}>
                      <label>
                        <input
                          type="checkbox"
                          checked={heliusConfig.laserstream_enabled}
                          onChange={(e) => handleUpdateHeliusConfig({ laserstream_enabled: e.target.checked })}
                        />
                        LaserStream Enabled
                      </label>
                    </div>
                    <div className={styles.configRow}>
                      <label>
                        <input
                          type="checkbox"
                          checked={heliusConfig.use_helius_sender}
                          onChange={(e) => handleUpdateHeliusConfig({ use_helius_sender: e.target.checked })}
                        />
                        Use Helius Sender for TX
                      </label>
                    </div>
                    <div className={styles.configRow}>
                      <label>Default Priority Level:</label>
                      <select
                        value={heliusConfig.default_priority_level}
                        onChange={(e) => handleUpdateHeliusConfig({ default_priority_level: e.target.value })}
                        className={styles.configSelect}
                      >
                        <option value="low">Low</option>
                        <option value="medium">Medium</option>
                        <option value="high">High</option>
                        <option value="very_high">Very High</option>
                      </select>
                    </div>
                  </div>
                )}
              </div>
            </div>
          </div>
        )}
      </div>
    );
  };

  const renderNavigationTabs = () => (
    <div className={styles.mainNavTabs}>
      <button
        className={`${styles.navTab} ${activeView === 'dashboard' ? styles.active : ''}`}
        onClick={() => onViewChange('dashboard')}
      >
        Dashboard
      </button>
      <button
        className={`${styles.navTab} ${activeView === 'curves' ? styles.active : ''}`}
        onClick={() => onViewChange('curves')}
      >
        Curve Bonding
      </button>
      <button
        className={`${styles.navTab} ${activeView === 'opportunities' ? styles.active : ''}`}
        onClick={() => onViewChange('opportunities')}
      >
        Opportunities
      </button>
      <button
        className={`${styles.navTab} ${activeView === 'kol-tracker' ? styles.active : ''}`}
        onClick={() => onViewChange('kol-tracker')}
      >
        KOL Tracker
      </button>
      <button
        className={`${styles.navTab} ${activeView === 'settings' ? styles.active : ''}`}
        onClick={() => onViewChange('settings')}
      >
        Settings
      </button>
    </div>
  );

  switch (activeView) {
    case 'dashboard':
      return (
        <div className={styles.viewContainer}>
          {renderNavigationTabs()}
          <HomeTab
            positions={edges.data?.filter(e => e.status === 'executed').map(e => ({
              id: e.id,
              token_mint: e.route_data?.input_token || '',
              token_symbol: undefined,
              entry_sol_amount: (e.estimated_profit_lamports || 0) / 1e9,
              unrealized_pnl: 0,
              status: 'open',
            })) || []}
            recentEvents={[]}
          />
        </div>
      );
    case 'opportunities':
      return renderOpportunitiesView();
    case 'signals':
      return renderSignalsView();
    case 'strategies':
      return renderStrategiesView();
    case 'curves':
      return (
        <div className={styles.viewContainer}>
          {renderNavigationTabs()}
          <CurvePanel
            onError={(msg) => console.error('Curve error:', msg)}
            onSuccess={(msg) => console.log('Curve success:', msg)}
          />
        </div>
      );
    case 'cows':
      return renderCowsView();
    case 'threats':
      return renderThreatsView();
    case 'kol-tracker':
      return renderKOLTrackerView();
    case 'helius':
      return renderHeliusView();
    case 'settings':
      return renderSettingsView();
    case 'research':
      return renderResearchView();
    default:
      return (
        <div className={styles.viewContainer}>
          {renderNavigationTabs()}
          <HomeTab
            positions={edges.data?.filter(e => e.status === 'executed').map(e => ({
              id: e.id,
              token_mint: e.route_data?.input_token || '',
              token_symbol: undefined,
              entry_sol_amount: (e.estimated_profit_lamports || 0) / 1e9,
              unrealized_pnl: 0,
              status: 'open',
            })) || []}
            recentEvents={[]}
          />
        </div>
      );
  }
};

export default ArbFarmDashboard;
