import React, { useEffect, useState } from 'react';
import { useArbFarm } from '../../../common/hooks/useArbFarm';
import {
  EDGE_STATUS_COLORS,
  AGENT_HEALTH_COLORS,
  STRATEGY_TYPE_LABELS,
  RISK_PROFILE_LABELS,
  RISK_PROFILE_COLORS,
  VENUE_TYPE_ICONS,
} from '../../../types/arbfarm';
import type { Edge, ThreatAlert, ArbFarmCowSummary } from '../../../types/arbfarm';
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
  const { dashboard, edges, trades, scanner, swarm, strategies, threats, cows, sse } = useArbFarm({
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
    }
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

  const renderKOLTrackerView = () => (
    <div className={styles.kolView}>
      <div className={styles.viewHeader}>
        <button className={styles.backButton} onClick={() => onViewChange('dashboard')}>
          ← Back
        </button>
        <h2>KOL Tracker</h2>
        <button className={styles.createButton}>+ Track Wallet</button>
      </div>
      <div className={styles.emptyState}>
        <p>KOL tracking coming soon</p>
        <p className={styles.emptyHint}>Track wallets and enable copy trading</p>
      </div>
    </div>
  );

  const renderSettingsView = () => (
    <div className={styles.settingsView}>
      <div className={styles.viewHeader}>
        <button className={styles.backButton} onClick={() => onViewChange('dashboard')}>
          ← Back
        </button>
        <h2>Settings</h2>
      </div>
      <div className={styles.emptyState}>
        <p>Settings panel coming soon</p>
        <p className={styles.emptyHint}>Configure risk parameters and execution settings</p>
      </div>
    </div>
  );

  const renderResearchView = () => (
    <div className={styles.researchView}>
      <div className={styles.viewHeader}>
        <button className={styles.backButton} onClick={() => onViewChange('dashboard')}>
          ← Back
        </button>
        <h2>Research & Discovery</h2>
      </div>
      <div className={styles.emptyState}>
        <p>Research module coming soon</p>
        <p className={styles.emptyHint}>Ingest URLs and discover strategies</p>
      </div>
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
