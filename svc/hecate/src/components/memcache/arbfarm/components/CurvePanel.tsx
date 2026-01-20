import React, { useState, useEffect, useCallback } from 'react';
import type {
  GraduationCandidate,
  CurvePosition,
  TrackedToken,
  SnipePosition,
  Strategy,
  CurveStrategyParams,
  CurveStrategyStats,
  SniperStats,
  GraduationTrackerStats,
  CurveExitConfig,
} from '../../../../types/arbfarm';
import { arbFarmService } from '../../../../common/services/arbfarm-service';
import CurveCandidateCard from './CurveCandidateCard';
import CurvePositionCard from './CurvePositionCard';
import CurveStrategyCard from './CurveStrategyCard';
import CurveMetricsPanel from './CurveMetricsPanel';
import GraduationProgressBar from './GraduationProgressBar';
import OpportunityDetailModal from './OpportunityDetailModal';
import CurveStrategyConfigModal from './CurveStrategyConfigModal';
import styles from '../arbfarm.module.scss';

type CurveTab = 'candidates' | 'positions' | 'tracked' | 'strategies' | 'sniper';

interface CurvePanelProps {
  onError: (message: string) => void;
  onSuccess: (message: string) => void;
}

const CurvePanel: React.FC<CurvePanelProps> = ({ onError, onSuccess }) => {
  const [activeTab, setActiveTab] = useState<CurveTab>('candidates');
  const [loading, setLoading] = useState(false);

  const [candidates, setCandidates] = useState<GraduationCandidate[]>([]);
  const [positions, setPositions] = useState<CurvePosition[]>([]);
  const [trackedTokens, setTrackedTokens] = useState<TrackedToken[]>([]);
  const [snipePositions, setSnipePositions] = useState<SnipePosition[]>([]);
  const [strategies, setStrategies] = useState<Strategy[]>([]);
  const [strategyStats, setStrategyStats] = useState<Record<string, CurveStrategyStats>>({});

  const [trackerStats, setTrackerStats] = useState<GraduationTrackerStats | null>(null);
  const [sniperStats, setSniperStats] = useState<SniperStats | null>(null);

  const [trackedMints, setTrackedMints] = useState<Set<string>>(new Set());
  const [customMint, setCustomMint] = useState('');

  const [showCreateStrategy, setShowCreateStrategy] = useState(false);
  const [newStrategy, setNewStrategy] = useState<{
    name: string;
    mode: string;
    execution_mode: 'autonomous' | 'agent_directed' | 'hybrid';
    min_progress: number;
    max_progress: number;
    entry_sol: number;
    exit_on_graduation: boolean;
    sell_delay_ms: number;
  }>({
    name: '',
    mode: 'graduation_arbitrage',
    execution_mode: 'agent_directed',
    min_progress: 70,
    max_progress: 98,
    entry_sol: 0.1,
    exit_on_graduation: true,
    sell_delay_ms: 500,
  });

  const [candidatesFilter, setCandidatesFilter] = useState({
    min_progress: 70,
    max_progress: 99,
    limit: 20,
  });

  const [selectedMetricsToken, setSelectedMetricsToken] = useState<{
    mint: string;
    venue: string;
    symbol: string;
  } | null>(null);

  const [selectedCandidate, setSelectedCandidate] = useState<GraduationCandidate | null>(null);

  const [configStrategy, setConfigStrategy] = useState<Strategy | null>(null);
  const [strategyCurveParams, setStrategyCurveParams] = useState<Record<string, CurveStrategyParams>>({});

  const fetchCandidates = useCallback(async () => {
    setLoading(true);
    try {
      const res = await arbFarmService.listGraduationCandidates(
        candidatesFilter.min_progress,
        candidatesFilter.max_progress,
        candidatesFilter.limit
      );
      if (res.success && res.data) {
        const data = res.data as { candidates?: GraduationCandidate[] } | GraduationCandidate[];
        const candidatesArray = Array.isArray(data) ? data : (data.candidates || []);
        setCandidates(candidatesArray);
      }
    } catch (e) {
      onError('Failed to fetch candidates');
    } finally {
      setLoading(false);
    }
  }, [candidatesFilter, onError]);

  const fetchPositions = useCallback(async () => {
    try {
      const res = await arbFarmService.listCurvePositions();
      if (res.success && res.data) {
        // Handle both array and {positions: [...]} response formats
        const data = res.data as CurvePosition[] | { positions?: CurvePosition[] };
        const positions = Array.isArray(data) ? data : (data.positions || []);
        setPositions(positions);
      }
    } catch (e) {
      onError('Failed to fetch positions');
    }
  }, [onError]);

  const fetchTrackedTokens = useCallback(async () => {
    try {
      const [tokensRes, statsRes] = await Promise.all([
        arbFarmService.listTrackedTokens(),
        arbFarmService.getGraduationTrackerStats(),
      ]);
      if (tokensRes.success && tokensRes.data) {
        // Handle both array and {tokens: [...]} response formats
        const data = tokensRes.data as TrackedToken[] | { tokens?: TrackedToken[] };
        const tokens = Array.isArray(data) ? data : (data.tokens || []);
        setTrackedTokens(tokens);
        setTrackedMints(new Set(tokens.map(t => t.mint)));
      }
      if (statsRes.success && statsRes.data) {
        setTrackerStats(statsRes.data);
      }
    } catch (e) {
      onError('Failed to fetch tracked tokens');
    }
  }, [onError]);

  const fetchSnipePositions = useCallback(async () => {
    try {
      const [posRes, statsRes] = await Promise.all([
        arbFarmService.listSnipePositions(),
        arbFarmService.getSniperStats(),
      ]);
      if (posRes.success && posRes.data) {
        // Handle both array and {positions: [...]} response formats
        const data = posRes.data as SnipePosition[] | { positions?: SnipePosition[] };
        const positions = Array.isArray(data) ? data : (data.positions || []);
        setSnipePositions(positions);
      }
      if (statsRes.success && statsRes.data) {
        setSniperStats(statsRes.data);
      }
    } catch (e) {
      onError('Failed to fetch snipe positions');
    }
  }, [onError]);

  const fetchStrategies = useCallback(async () => {
    try {
      const res = await arbFarmService.listStrategies();
      if (res.success && res.data) {
        // Handle both array and {strategies: [...]} response formats
        const data = res.data as Strategy[] | { strategies?: Strategy[] };
        const allStrategies = Array.isArray(data) ? data : (data.strategies || []);
        const curveStrategies = allStrategies.filter(
          (s: Strategy) => s.strategy_type === 'curve_arb'
        );
        setStrategies(curveStrategies);
      }
    } catch (e) {
      onError('Failed to fetch strategies');
    }
  }, [onError]);

  useEffect(() => {
    if (activeTab === 'candidates') fetchCandidates();
    else if (activeTab === 'positions') fetchPositions();
    else if (activeTab === 'tracked') fetchTrackedTokens();
    else if (activeTab === 'sniper') fetchSnipePositions();
    else if (activeTab === 'strategies') fetchStrategies();
  }, [activeTab, fetchCandidates, fetchPositions, fetchTrackedTokens, fetchSnipePositions, fetchStrategies]);

  const handleQuickBuy = async (mint: string, amount: number) => {
    setLoading(true);
    try {
      const res = await arbFarmService.buyCurveToken(mint, amount, { simulate_only: false });
      if (res.success) {
        onSuccess(`Buy order placed for ${amount} SOL`);
        fetchPositions();
      } else {
        onError(res.error || 'Buy failed');
      }
    } catch (e) {
      onError('Buy failed');
    } finally {
      setLoading(false);
    }
  };

  const handleTrackToken = async (mint: string) => {
    try {
      const candidate = candidates.find(c => c.token.mint === mint);
      const res = await arbFarmService.trackToken(mint, '', {
        name: candidate?.token.name,
        symbol: candidate?.token.symbol,
        venue: candidate?.token.venue,
      });
      if (res.success) {
        onSuccess('Token tracked');
        setTrackedMints(prev => new Set([...prev, mint]));
        fetchTrackedTokens();
      } else {
        onError(res.error || 'Failed to track');
      }
    } catch (e) {
      onError('Failed to track token');
    }
  };

  const handleUntrackToken = async (mint: string) => {
    try {
      const res = await arbFarmService.untrackToken(mint);
      if (res.success) {
        onSuccess('Token untracked');
        setTrackedMints(prev => {
          const next = new Set(prev);
          next.delete(mint);
          return next;
        });
        fetchTrackedTokens();
      } else {
        onError(res.error || 'Failed to untrack');
      }
    } catch (e) {
      onError('Failed to untrack token');
    }
  };

  const handleClearAllTracked = async () => {
    const confirmed = window.confirm(
      '⚠️ CLEAR ALL TRACKED TOKENS\n\n' +
      `This will remove all ${trackedTokens.length} tracked tokens from your watchlist.\n` +
      'This action cannot be undone.\n\n' +
      'Are you sure you want to proceed?'
    );

    if (!confirmed) return;

    try {
      const res = await arbFarmService.clearAllTracked();
      if (res.success && res.data) {
        onSuccess(`Cleared ${res.data.cleared} tracked tokens`);
        setTrackedMints(new Set());
        fetchTrackedTokens();
      } else {
        onError(res.error || 'Failed to clear tracked tokens');
      }
    } catch (e) {
      onError('Failed to clear tracked tokens');
    }
  };

  const handleTrackCustomToken = async () => {
    const mint = customMint.trim();
    if (!mint) {
      onError('Please enter a contract address');
      return;
    }

    if (mint.length < 32 || mint.length > 64) {
      onError('Invalid contract address format');
      return;
    }

    try {
      const res = await arbFarmService.trackToken(mint, '', {
        name: 'Custom Token',
        symbol: mint.slice(0, 4).toUpperCase(),
        venue: 'pump_fun',
      });
      if (res.success) {
        onSuccess('Custom token tracked');
        setCustomMint('');
        setTrackedMints(prev => new Set([...prev, mint]));
        fetchTrackedTokens();
      } else {
        onError(res.error || 'Failed to track custom token');
      }
    } catch (e) {
      onError('Failed to track custom token');
    }
  };

  const handleClosePosition = async (positionId: string, percent?: number) => {
    setLoading(true);
    try {
      const res = await arbFarmService.closeCurvePosition(positionId, percent);
      if (res.success) {
        onSuccess(`Position closed${res.data?.pnl_sol != null ? ` - PnL: ${res.data.pnl_sol.toFixed(4)} SOL` : ''}`);
        fetchPositions();
      } else {
        onError(res.error || 'Close failed');
      }
    } catch (e) {
      onError('Close failed');
    } finally {
      setLoading(false);
    }
  };

  const handleUpdateExitConfig = async (positionId: string, config: Partial<CurveExitConfig>) => {
    try {
      const res = await arbFarmService.updateCurvePositionExit(positionId, config);
      if (res.success) {
        onSuccess('Exit config updated');
        fetchPositions();
      } else {
        onError(res.error || 'Update failed');
      }
    } catch (e) {
      onError('Update failed');
    }
  };

  const handleEmergencyExitAll = async () => {
    const confirmed = window.confirm(
      '🚨 EMERGENCY EXIT ALL POSITIONS\n\n' +
      'This will IMMEDIATELY sell ALL open positions at market price.\n' +
      'This action cannot be undone.\n\n' +
      'Are you sure you want to proceed?'
    );

    if (!confirmed) return;

    setLoading(true);
    try {
      const res = await arbFarmService.emergencyExitAllPositions();
      if (res.success && res.data) {
        const { positions_exited, positions_failed, message } = res.data;
        if (positions_failed > 0) {
          onError(`${message} - Check console for details`);
          console.error('Emergency exit results:', res.data.results);
        } else {
          onSuccess(message);
        }
        fetchPositions();
      } else {
        onError(res.error || 'Emergency exit failed');
      }
    } catch (e) {
      onError('Emergency exit failed - check console');
      console.error('Emergency exit error:', e);
    } finally {
      setLoading(false);
    }
  };

  const handleToggleStrategy = async (id: string, enabled: boolean) => {
    try {
      const res = await arbFarmService.toggleStrategy(id, enabled);
      if (res.success) {
        onSuccess(`Strategy ${enabled ? 'enabled' : 'disabled'}`);
        fetchStrategies();
      } else {
        onError(res.error || 'Toggle failed');
      }
    } catch (e) {
      onError('Toggle failed');
    }
  };

  const handleChangeExecutionMode = async (
    id: string,
    mode: 'autonomous' | 'agent_directed' | 'hybrid'
  ) => {
    try {
      const res = await arbFarmService.updateStrategy(id, { execution_mode: mode });
      if (res.success) {
        onSuccess(`Execution mode changed to ${mode === 'autonomous' ? 'automatic' : 'manual approval'}`);
        fetchStrategies();
      } else {
        onError(res.error || 'Failed to update execution mode');
      }
    } catch (e) {
      onError('Failed to update execution mode');
    }
  };

  const handleChangeRiskProfile = async (
    id: string,
    profile: 'conservative' | 'moderate' | 'aggressive'
  ) => {
    try {
      const res = await arbFarmService.setRiskProfile(id, profile);
      if (res.success) {
        const labels = {
          conservative: 'Conservative (low risk)',
          moderate: 'Moderate (balanced)',
          aggressive: 'Aggressive (high risk)',
        };
        onSuccess(`Risk profile changed to ${labels[profile]}`);
        fetchStrategies();
      } else {
        onError(res.error || 'Failed to update risk profile');
      }
    } catch (e) {
      onError('Failed to update risk profile');
    }
  };

  const getDefaultCurveParams = (strategyId: string): CurveStrategyParams => {
    if (strategyCurveParams[strategyId]) {
      return strategyCurveParams[strategyId];
    }
    return {
      mode: 'graduation_sniper',
      min_graduation_progress: 70,
      max_graduation_progress: 98,
      min_volume_24h_sol: 10,
      max_holder_concentration: 50,
      min_holder_count: 50,
      entry_sol_amount: 0.1,
      exit_on_graduation: true,
      graduation_sell_delay_ms: 500,
    };
  };

  const handleConfigureStrategy = (strategy: Strategy) => {
    setConfigStrategy(strategy);
  };

  const handleCloseConfigModal = () => {
    setConfigStrategy(null);
  };

  const handleConfigUpdate = () => {
    fetchStrategies();
  };

  const handleCreateStrategy = async () => {
    setLoading(true);
    try {
      const params: CurveStrategyParams = {
        mode: newStrategy.mode as any,
        min_graduation_progress: newStrategy.min_progress,
        max_graduation_progress: newStrategy.max_progress,
        min_volume_24h_sol: 10,
        max_holder_concentration: 50,
        min_holder_count: 50,
        entry_sol_amount: newStrategy.entry_sol,
        exit_on_graduation: newStrategy.exit_on_graduation,
        graduation_sell_delay_ms: newStrategy.sell_delay_ms,
      };

      const res = await arbFarmService.createCurveStrategy(
        newStrategy.name,
        params,
        newStrategy.execution_mode
      );
      if (res.success) {
        onSuccess('Strategy created');
        setShowCreateStrategy(false);
        setNewStrategy({
          name: '',
          mode: 'graduation_arbitrage',
          execution_mode: 'agent_directed',
          min_progress: 70,
          max_progress: 98,
          entry_sol: 0.1,
          exit_on_graduation: true,
          sell_delay_ms: 500,
        });
        fetchStrategies();
      } else {
        onError(res.error || 'Create failed');
      }
    } catch (e) {
      onError('Create failed');
    } finally {
      setLoading(false);
    }
  };

  const handleStartTracker = async () => {
    const res = await arbFarmService.startGraduationTracker();
    if (res.success) {
      onSuccess('Tracker started');
      fetchTrackedTokens();
    } else {
      onError(res.error || 'Failed to start tracker');
    }
  };

  const handleStopTracker = async () => {
    const res = await arbFarmService.stopGraduationTracker();
    if (res.success) {
      onSuccess('Tracker stopped');
      fetchTrackedTokens();
    } else {
      onError(res.error || 'Failed to stop tracker');
    }
  };

  const handleStartSniper = async () => {
    const res = await arbFarmService.startSniper();
    if (res.success) {
      onSuccess('Sniper started');
      fetchSnipePositions();
    } else {
      onError(res.error || 'Failed to start sniper');
    }
  };

  const handleStopSniper = async () => {
    const res = await arbFarmService.stopSniper();
    if (res.success) {
      onSuccess('Sniper stopped');
      fetchSnipePositions();
    } else {
      onError(res.error || 'Failed to stop sniper');
    }
  };

  const handleManualSell = async (mint: string) => {
    const res = await arbFarmService.manualSell(mint);
    if (res.success) {
      onSuccess('Manual sell triggered');
      fetchSnipePositions();
    } else {
      onError(res.error || 'Manual sell failed');
    }
  };

  const handleViewMetrics = (mint: string, venue: string, symbol: string) => {
    setSelectedMetricsToken({ mint, venue, symbol });
  };

  const handleCloseMetrics = () => {
    setSelectedMetricsToken(null);
  };

  const handleViewDetails = (candidate: GraduationCandidate) => {
    setSelectedCandidate(candidate);
  };

  const handleCloseDetails = () => {
    setSelectedCandidate(null);
  };

  return (
    <div className={styles.curvePanel}>
      <div className={styles.curvePanelHeader}>
        <h2>Bonding Curves</h2>
        <div className={styles.curveTabs}>
          <button
            className={`${styles.curveTab} ${activeTab === 'candidates' ? styles.active : ''}`}
            onClick={() => setActiveTab('candidates')}
          >
            Candidates
          </button>
          <button
            className={`${styles.curveTab} ${activeTab === 'positions' ? styles.active : ''}`}
            onClick={() => setActiveTab('positions')}
          >
            Positions
          </button>
          <button
            className={`${styles.curveTab} ${activeTab === 'tracked' ? styles.active : ''}`}
            onClick={() => setActiveTab('tracked')}
          >
            Tracked
          </button>
          <button
            className={`${styles.curveTab} ${activeTab === 'sniper' ? styles.active : ''}`}
            onClick={() => setActiveTab('sniper')}
          >
            Sniper
          </button>
          <button
            className={`${styles.curveTab} ${activeTab === 'strategies' ? styles.active : ''}`}
            onClick={() => setActiveTab('strategies')}
          >
            Strategies
          </button>
        </div>
      </div>

      {loading && <div className={styles.loadingOverlay}>Loading...</div>}

      {activeTab === 'candidates' && (
        <div className={styles.candidatesView}>
          <div className={styles.candidatesFilters}>
            <label>
              Min Progress:
              <input
                type="number"
                value={candidatesFilter.min_progress}
                onChange={(e) =>
                  setCandidatesFilter({
                    ...candidatesFilter,
                    min_progress: parseInt(e.target.value),
                  })
                }
              />
            </label>
            <label>
              Max Progress:
              <input
                type="number"
                value={candidatesFilter.max_progress}
                onChange={(e) =>
                  setCandidatesFilter({
                    ...candidatesFilter,
                    max_progress: parseInt(e.target.value),
                  })
                }
              />
            </label>
            <button onClick={fetchCandidates}>Refresh</button>
          </div>

          <div className={styles.candidatesGrid}>
            {candidates.map((candidate) => (
              <CurveCandidateCard
                key={candidate.token.mint}
                candidate={candidate}
                onQuickBuy={handleQuickBuy}
                onTrack={handleTrackToken}
                onUntrack={handleUntrackToken}
                onViewMetrics={handleViewMetrics}
                onViewDetails={handleViewDetails}
                isTracked={trackedMints.has(candidate.token.mint)}
              />
            ))}
            {candidates.length === 0 && !loading && (
              <div className={styles.emptyState}>
                No graduation candidates found matching criteria
              </div>
            )}
          </div>
        </div>
      )}

      {activeTab === 'positions' && (
        <div className={styles.positionsView}>
          <div className={styles.viewHeader}>
            <h3>Open Positions</h3>
            <div className={styles.positionControls}>
              <button onClick={fetchPositions}>Refresh</button>
              {positions.length > 0 && (
                <button
                  className={styles.emergencyExitButton}
                  onClick={handleEmergencyExitAll}
                  title="Force sell ALL positions at market - EMERGENCY USE ONLY"
                >
                  🚨 EXIT ALL
                </button>
              )}
            </div>
          </div>

          <div className={styles.positionsGrid}>
            {positions.map((position) => (
              <CurvePositionCard
                key={position.id}
                position={position}
                onClose={handleClosePosition}
                onUpdateExit={handleUpdateExitConfig}
              />
            ))}
            {positions.length === 0 && !loading && (
              <div className={styles.emptyState}>No open positions</div>
            )}
          </div>
        </div>
      )}

      {activeTab === 'tracked' && (
        <div className={styles.trackedView}>
          <div className={styles.viewHeader}>
            <h3>Graduation Tracker</h3>
            <div className={styles.trackerControls}>
              {trackerStats && (
                <span className={styles.trackerStatus}>
                  {trackerStats.is_running ? '🟢 Running' : '🔴 Stopped'}
                </span>
              )}
              <button onClick={trackerStats?.is_running ? handleStopTracker : handleStartTracker}>
                {trackerStats?.is_running ? 'Stop' : 'Start'}
              </button>
              <button onClick={fetchTrackedTokens}>Refresh</button>
              {trackedTokens.length > 0 && (
                <button
                  className={styles.dangerButton}
                  onClick={handleClearAllTracked}
                  title="Remove all tracked tokens"
                >
                  Clear All
                </button>
              )}
            </div>
          </div>

          {trackerStats && (
            <div className={styles.trackerStatsBar}>
              <span>Tracked: {trackerStats.tokens_tracked}</span>
              <span>Near Grad: {trackerStats.tokens_near_graduation}</span>
              <span>Graduated: {trackerStats.tokens_graduated}</span>
              <span>Checks: {trackerStats.total_checks}</span>
            </div>
          )}

          <div className={styles.customTokenInput}>
            <input
              type="text"
              placeholder="Enter contract address to track..."
              value={customMint}
              onChange={(e) => setCustomMint(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleTrackCustomToken()}
            />
            <button onClick={handleTrackCustomToken}>Track</button>
          </div>

          <div className={styles.trackedGrid}>
            {trackedTokens.map((token) => (
              <div key={token.mint} className={styles.trackedTokenCard}>
                <div className={styles.trackedHeader}>
                  <span className={styles.tokenSymbol}>${token.symbol}</span>
                  <span className={styles.trackedState}>{token.state}</span>
                </div>
                <GraduationProgressBar
                  progress={token.progress}
                  velocity={token.progress_velocity}
                />
                <div className={styles.trackedInfo}>
                  <span>Venue: {token.venue}</span>
                  {token.entry_price_sol != null && (
                    <span>Entry: {token.entry_price_sol.toFixed(4)} SOL</span>
                  )}
                </div>
                <div className={styles.trackedActions}>
                  <button
                    className={styles.untrackButton}
                    onClick={() => handleUntrackToken(token.mint)}
                  >
                    Untrack
                  </button>
                  <span className={styles.mintAddress} title={token.mint}>
                    {token.mint.slice(0, 8)}...{token.mint.slice(-6)}
                  </span>
                </div>
              </div>
            ))}
            {trackedTokens.length === 0 && !loading && (
              <div className={styles.emptyState}>No tokens being tracked</div>
            )}
          </div>
        </div>
      )}

      {activeTab === 'sniper' && (
        <div className={styles.sniperView}>
          <div className={styles.viewHeader}>
            <h3>Graduation Sniper</h3>
            <div className={styles.sniperControls}>
              {sniperStats && (
                <span className={styles.sniperStatus}>
                  {sniperStats.is_running ? '🎯 Armed' : '⏸️ Paused'}
                </span>
              )}
              <button onClick={sniperStats?.is_running ? handleStopSniper : handleStartSniper}>
                {sniperStats?.is_running ? 'Stop' : 'Start'}
              </button>
              <button onClick={fetchSnipePositions}>Refresh</button>
            </div>
          </div>

          {sniperStats && (
            <div className={styles.sniperStatsBar}>
              <span>Waiting: {sniperStats.positions_waiting ?? 0}</span>
              <span>Sold: {sniperStats.positions_sold ?? 0}</span>
              <span>Failed: {sniperStats.positions_failed ?? 0}</span>
              <span
                className={(sniperStats.total_pnl_sol ?? 0) >= 0 ? styles.positive : styles.negative}
              >
                P&L: {(sniperStats.total_pnl_sol ?? 0).toFixed(4)} SOL
              </span>
            </div>
          )}

          <div className={styles.snipePositionsGrid}>
            {snipePositions.map((pos) => (
              <div key={pos.mint} className={styles.snipePositionCard}>
                <div className={styles.snipeHeader}>
                  <span className={styles.tokenSymbol}>${pos.symbol}</span>
                  <span className={`${styles.snipeStatus} ${styles[pos.status]}`}>
                    {pos.status}
                  </span>
                </div>
                <div className={styles.snipeInfo}>
                  <div>Entry: {(pos.entry_price_sol ?? 0).toFixed(4)} SOL</div>
                  <div>Tokens: {(pos.entry_tokens ?? 0).toLocaleString()}</div>
                  {pos.pnl_sol !== undefined && pos.pnl_sol !== null && (
                    <div className={pos.pnl_sol >= 0 ? styles.positive : styles.negative}>
                      P&L: {pos.pnl_sol.toFixed(4)} SOL
                    </div>
                  )}
                </div>
                {pos.status === 'waiting' && (
                  <button
                    className={styles.manualSellButton}
                    onClick={() => handleManualSell(pos.mint)}
                  >
                    Manual Sell
                  </button>
                )}
              </div>
            ))}
            {snipePositions.length === 0 && !loading && (
              <div className={styles.emptyState}>No snipe positions</div>
            )}
          </div>
        </div>
      )}

      {activeTab === 'strategies' && (
        <div className={styles.strategiesView}>
          <div className={styles.viewHeader}>
            <h3>Curve Strategies</h3>
            <button
              className={styles.createButton}
              onClick={() => setShowCreateStrategy(!showCreateStrategy)}
            >
              + New Strategy
            </button>
          </div>

          {showCreateStrategy && (
            <div className={styles.createStrategyForm}>
              <h4>Create Curve Strategy</h4>
              <div className={styles.formGrid}>
                <div className={styles.formField}>
                  <label>Name</label>
                  <input
                    type="text"
                    value={newStrategy.name}
                    onChange={(e) => setNewStrategy({ ...newStrategy, name: e.target.value })}
                    placeholder="Strategy name"
                  />
                </div>
                <div className={styles.formField}>
                  <label>Mode</label>
                  <select
                    value={newStrategy.mode}
                    onChange={(e) => setNewStrategy({ ...newStrategy, mode: e.target.value })}
                  >
                    <option value="graduation_arbitrage">Graduation Arbitrage</option>
                    <option value="fast_snipe">Fast Snipe</option>
                    <option value="scalp_on_curve">Scalp on Curve</option>
                  </select>
                </div>
                <div className={styles.formField}>
                  <label>Execution</label>
                  <select
                    value={newStrategy.execution_mode}
                    onChange={(e) =>
                      setNewStrategy({
                        ...newStrategy,
                        execution_mode: e.target.value as 'autonomous' | 'agent_directed' | 'hybrid',
                      })
                    }
                  >
                    <option value="agent_directed">Agent Directed (approval required)</option>
                    <option value="hybrid">Hybrid (auto small, approval large)</option>
                    <option value="autonomous">Autonomous (full auto)</option>
                  </select>
                </div>
                <div className={styles.formField}>
                  <label>Min Progress %</label>
                  <input
                    type="number"
                    value={newStrategy.min_progress}
                    onChange={(e) =>
                      setNewStrategy({ ...newStrategy, min_progress: parseInt(e.target.value) })
                    }
                  />
                </div>
                <div className={styles.formField}>
                  <label>Max Progress %</label>
                  <input
                    type="number"
                    value={newStrategy.max_progress}
                    onChange={(e) =>
                      setNewStrategy({ ...newStrategy, max_progress: parseInt(e.target.value) })
                    }
                  />
                </div>
                <div className={styles.formField}>
                  <label>Entry SOL</label>
                  <input
                    type="number"
                    step="0.1"
                    value={newStrategy.entry_sol}
                    onChange={(e) =>
                      setNewStrategy({ ...newStrategy, entry_sol: parseFloat(e.target.value) })
                    }
                  />
                </div>
                <div className={styles.formField}>
                  <label>Sell Delay (ms)</label>
                  <input
                    type="number"
                    value={newStrategy.sell_delay_ms}
                    onChange={(e) =>
                      setNewStrategy({ ...newStrategy, sell_delay_ms: parseInt(e.target.value) })
                    }
                  />
                </div>
                <div className={styles.formFieldCheckbox}>
                  <label>
                    <input
                      type="checkbox"
                      checked={newStrategy.exit_on_graduation}
                      onChange={(e) =>
                        setNewStrategy({ ...newStrategy, exit_on_graduation: e.target.checked })
                      }
                    />
                    Exit on Graduation
                  </label>
                </div>
              </div>
              <div className={styles.formActions}>
                <button className={styles.createButton} onClick={handleCreateStrategy}>
                  Create
                </button>
                <button
                  className={styles.cancelButton}
                  onClick={() => setShowCreateStrategy(false)}
                >
                  Cancel
                </button>
              </div>
            </div>
          )}

          <div className={styles.strategiesGrid}>
            {strategies.map((strategy) => (
              <CurveStrategyCard
                key={strategy.id}
                strategy={strategy}
                curveParams={getDefaultCurveParams(strategy.id)}
                stats={strategyStats[strategy.id]}
                onToggle={handleToggleStrategy}
                onChangeExecutionMode={handleChangeExecutionMode}
                onChangeRiskProfile={handleChangeRiskProfile}
                onConfigure={handleConfigureStrategy}
                onViewStats={async (id) => {
                  const res = await arbFarmService.getCurveStrategyStats(id);
                  if (res.success && res.data) {
                    setStrategyStats((prev) => ({ ...prev, [id]: res.data! }));
                  }
                }}
              />
            ))}
            {strategies.length === 0 && !loading && (
              <div className={styles.emptyState}>No curve strategies configured</div>
            )}
          </div>
        </div>
      )}

      {selectedMetricsToken && (
        <div className={styles.metricsOverlay} onClick={handleCloseMetrics}>
          <div className={styles.metricsDrawer} onClick={(e) => e.stopPropagation()}>
            <CurveMetricsPanel
              mint={selectedMetricsToken.mint}
              venue={selectedMetricsToken.venue}
              tokenSymbol={selectedMetricsToken.symbol}
              onClose={handleCloseMetrics}
            />
          </div>
        </div>
      )}

      {selectedCandidate && (
        <OpportunityDetailModal
          candidate={selectedCandidate}
          onClose={handleCloseDetails}
          onQuickBuy={(mint, amount) => {
            handleQuickBuy(mint, amount);
            handleCloseDetails();
          }}
          onTrack={(mint) => {
            handleTrackToken(mint);
            handleCloseDetails();
          }}
          onSuccess={onSuccess}
          onError={onError}
        />
      )}

      {configStrategy && (
        <CurveStrategyConfigModal
          strategy={configStrategy}
          curveParams={getDefaultCurveParams(configStrategy.id)}
          onClose={handleCloseConfigModal}
          onSuccess={onSuccess}
          onError={onError}
          onUpdate={handleConfigUpdate}
        />
      )}
    </div>
  );
};

export default CurvePanel;
