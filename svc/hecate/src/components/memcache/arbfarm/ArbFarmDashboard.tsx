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
} from '../../../types/arbfarm';
import { arbFarmService } from '../../../common/services/arbfarm-service';
import styles from './arbfarm.module.scss';
import EdgeCard from './components/EdgeCard';
import MetricCard from './components/MetricCard';
import SwarmStatusCard from './components/SwarmStatusCard';
import ThreatAlertCard from './components/ThreatAlertCard';
import TradeHistoryCard from './components/TradeHistoryCard';

export type ArbFarmView =
  | 'dashboard'
  | 'opportunities'
  | 'strategies'
  | 'cows'
  | 'research'
  | 'kol-tracker'
  | 'threats'
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

  // Research & Consensus state
  const [consensusDecisions, setConsensusDecisions] = useState<any[]>([]);
  const [consensusStats, setConsensusStats] = useState<any>(null);
  const [availableModels, setAvailableModels] = useState<any[]>([]);
  const [selectedConsensusId, setSelectedConsensusId] = useState<string | null>(null);
  const [consensusDetail, setConsensusDetail] = useState<any>(null);
  const [researchLoading, setResearchLoading] = useState(false);
  const [testConsensusLoading, setTestConsensusLoading] = useState(false);
  const [testConsensusResult, setTestConsensusResult] = useState<any>(null);

  // Settings state
  const [walletStatus, setWalletStatus] = useState<WalletStatus | null>(null);
  const [riskConfig, setRiskConfig] = useState<RiskConfig | null>(null);
  const [riskPresets, setRiskPresets] = useState<RiskPreset[]>([]);
  const [venues, setVenues] = useState<VenueConfig[]>([]);
  const [apiKeys, setApiKeys] = useState<ApiKeyStatus[]>([]);
  const [settingsLoading, setSettingsLoading] = useState(false);
  const [walletSetupAddress, setWalletSetupAddress] = useState('');
  const [settingsTab, setSettingsTab] = useState<'wallet' | 'risk' | 'venues' | 'api'>('wallet');

  useEffect(() => {
    sse.connect(['arb.edge.*', 'arb.trade.*', 'arb.threat.*', 'arb.swarm.*']);

    return () => sse.disconnect();
  }, []);

  useEffect(() => {
    edges.refresh();
    trades.refresh();
    trades.refreshStats('week');
    strategies.refresh();
    threats.refresh();
  }, []);

  useEffect(() => {
    if (activeView === 'cows') {
      cows.refresh();
      cows.refreshStats();
    } else if (activeView === 'kol-tracker') {
      kols.refresh();
    } else if (activeView === 'research') {
      fetchConsensusData();
    }
  }, [activeView]);

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
    const fetchSettings = async () => {
      if (activeView === 'settings') {
        setSettingsLoading(true);
        try {
          const [settingsRes, walletRes] = await Promise.all([
            arbFarmService.getAllSettings(),
            arbFarmService.getWalletStatus(),
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
        } catch (err) {
          console.error('Failed to fetch settings:', err);
        } finally {
          setSettingsLoading(false);
        }
      }
    };

    fetchSettings();
  }, [activeView]);

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

  const renderDashboardView = () => {
    const { summary } = dashboard;
    const swarmHealth = swarm.health;
    const topEdges = edges.data
      .filter((e) => ['detected', 'pending_approval'].includes(e.status))
      .slice(0, 5);
    const recentTrades = trades.data.slice(0, 5);
    const recentAlerts = threats.alerts.slice(0, 3);

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
                edges.data.filter((e) => e.status === 'detected').length,
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
            ) : strategies.data.length === 0 ? (
              <div className={styles.emptyState}>No strategies configured</div>
            ) : (
              strategies.data
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
    const filteredEdges =
      opportunitiesFilter === 'all'
        ? edges.data
        : edges.data.filter((e) => e.status === opportunitiesFilter);

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
                    ? edges.data.length
                    : edges.data.filter((e) => e.status === status).length}
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
        <button className={styles.createButton}>+ New Strategy</button>
      </div>

      <div className={styles.strategiesGrid}>
        {strategies.isLoading ? (
          <div className={styles.loadingState}>Loading strategies...</div>
        ) : strategies.data.length === 0 ? (
          <div className={styles.emptyState}>
            <p>No strategies configured</p>
            <p className={styles.emptyHint}>Create a strategy to start automated trading</p>
          </div>
        ) : (
          strategies.data.map((strategy) => (
            <div key={strategy.id} className={styles.strategyCard}>
              <div className={styles.strategyHeader}>
                <span className={styles.strategyName}>{strategy.name}</span>
                <button
                  className={`${styles.toggleButton} ${strategy.is_active ? styles.active : ''}`}
                  onClick={() => strategies.toggle(strategy.id, !strategy.is_active)}
                >
                  {strategy.is_active ? 'ON' : 'OFF'}
                </button>
              </div>
              <div className={styles.strategyMeta}>
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
                </div>
              )}
            </div>
          ))
        )}
      </div>
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
          <h3>Recent Alerts ({threats.alerts.length})</h3>
          <div className={styles.alertsList}>
            {threats.isLoading ? (
              <div className={styles.loadingState}>Loading alerts...</div>
            ) : threats.alerts.length === 0 ? (
              <div className={styles.emptyState}>No threat alerts</div>
            ) : (
              threats.alerts.map((alert) => <ThreatAlertCard key={alert.id} alert={alert} />)
            )}
          </div>
        </div>

        <div className={styles.blockedSection}>
          <h3>Blocked Entities ({threats.blocked.length})</h3>
          <div className={styles.blockedList}>
            {threats.blocked.length === 0 ? (
              <div className={styles.emptyState}>No blocked entities</div>
            ) : (
              threats.blocked.slice(0, 10).map((entity) => (
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

  const getTrustScoreColor = (score: number): string => {
    if (score >= 70) return '#22c55e';
    if (score >= 40) return '#f59e0b';
    return '#ef4444';
  };

  const renderKOLTrackerView = () => {
    const selectedKol = kols.data.find((k) => k.id === selectedKolId);

    return (
      <div className={styles.kolView}>
        <div className={styles.viewHeader}>
          <button className={styles.backButton} onClick={() => onViewChange('dashboard')}>
            ← Back
          </button>
          <h2>KOL Tracker</h2>
          <button className={styles.createButton} onClick={() => setShowAddKolModal(true)}>
            + Track Wallet
          </button>
        </div>

        <div className={styles.kolLayout}>
          <div className={styles.kolListSection}>
            <h3>
              Tracked Wallets ({kols.data.length})
              <button className={styles.refreshButton} onClick={() => kols.refresh()}>
                🔄
              </button>
            </h3>

            <div className={styles.kolList}>
              {kols.isLoading ? (
                <div className={styles.loadingState}>Loading KOLs...</div>
              ) : kols.data.length === 0 ? (
                <div className={styles.emptyState}>
                  <p>No wallets tracked</p>
                  <p className={styles.emptyHint}>Add a wallet to start tracking</p>
                </div>
              ) : (
                kols.data.map((kol) => (
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

  const renderSettingsView = () => (
    <div className={styles.settingsView}>
      <div className={styles.viewHeader}>
        <button className={styles.backButton} onClick={() => onViewChange('dashboard')}>
          ← Back
        </button>
        <h2>Settings</h2>
      </div>

      <div className={styles.settingsTabs}>
        {(['wallet', 'risk', 'venues', 'api'] as const).map((tab) => (
          <button
            key={tab}
            className={`${styles.settingsTab} ${settingsTab === tab ? styles.active : ''}`}
            onClick={() => setSettingsTab(tab)}
          >
            {tab === 'wallet' && '💼 Wallet'}
            {tab === 'risk' && '⚠️ Risk'}
            {tab === 'venues' && '🏛️ Venues'}
            {tab === 'api' && '🔑 API Keys'}
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

  const renderResearchView = () => (
    <div className={styles.researchView}>
      <div className={styles.viewHeader}>
        <button className={styles.backButton} onClick={() => onViewChange('dashboard')}>
          ← Back
        </button>
        <h2>Research & Consensus</h2>
        <button className={styles.refreshButton} onClick={fetchConsensusData}>
          🔄
        </button>
      </div>

      {researchLoading ? (
        <div className={styles.loadingState}>Loading consensus data...</div>
      ) : (
        <>
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
        </>
      )}
    </div>
  );

  const formatAddress = (address: string): string => {
    if (!address || address.length < 12) return address;

    return `${address.slice(0, 6)}...${address.slice(-4)}`;
  };

  const renderCowsView = () => {
    const filteredCows = cows.data.filter((cow) => {
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

  switch (activeView) {
    case 'opportunities':
      return renderOpportunitiesView();
    case 'strategies':
      return renderStrategiesView();
    case 'cows':
      return renderCowsView();
    case 'threats':
      return renderThreatsView();
    case 'kol-tracker':
      return renderKOLTrackerView();
    case 'settings':
      return renderSettingsView();
    case 'research':
      return renderResearchView();
    default:
      return renderDashboardView();
  }
};

export default ArbFarmDashboard;
