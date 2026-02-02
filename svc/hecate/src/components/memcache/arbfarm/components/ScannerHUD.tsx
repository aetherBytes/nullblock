import React, { useState, useEffect, useCallback, useRef } from 'react';
import styles from '../arbfarm.module.scss';
import { arbFarmService } from '../../../../common/services/arbfarm-service';
import type { RecentTradeInfo, Signal, BehavioralStrategy, ScannerStatus, Contender } from '../../../../types/arbfarm';

interface LiveTrade {
  id: string;
  edge_id?: string;
  tx_signature?: string;
  entry_price?: number;
  executed_at: string;
  token_mint?: string;
  token_symbol?: string;
  venue?: string;
}

type ActiveTab = 'trades' | 'scanner' | 'strategies' | 'control';

interface ScannerHUDProps {
  liveTrades?: LiveTrade[];
  recentTrades?: RecentTradeInfo[];
  onTradeClick?: (trade: LiveTrade | RecentTradeInfo, isLive: boolean) => void;
  onViewPosition?: (tokenMint: string) => void;
  onSignalClick?: (signal: Signal) => void;
  onContenderClick?: (mint: string, symbol: string) => void;
  onToggleMonitor?: () => void;
  onSellAll?: () => void;
  onToggleExecution?: () => void;
  onReconcile?: () => void;
  monitorActive?: boolean;
  executionEnabled?: boolean;
  sellingAll?: boolean;
  reconciling?: boolean;
  positionCount?: number;
  togglingMonitor?: boolean;
  togglingExecution?: boolean;
}

const ScannerHUD: React.FC<ScannerHUDProps> = ({
  liveTrades = [],
  recentTrades = [],
  onTradeClick,
  onViewPosition,
  onSignalClick,
  onContenderClick,
  onToggleMonitor,
  onSellAll,
  onToggleExecution,
  onReconcile,
  monitorActive = false,
  executionEnabled = false,
  sellingAll = false,
  reconciling = false,
  positionCount = 0,
  togglingMonitor = false,
  togglingExecution = false,
}) => {
  const [activeTab, setActiveTab] = useState<ActiveTab>('control');
  const [newTradeIds, setNewTradeIds] = useState<Set<string>>(new Set());
  const prevTradesRef = useRef<string[]>([]);

  const [signals, setSignals] = useState<Signal[]>([]);
  const [contenders, setContenders] = useState<Contender[]>([]);
  const [strategies, setStrategies] = useState<BehavioralStrategy[]>([]);
  const [scannerStatus, setScannerStatus] = useState<ScannerStatus | null>(null);
  const [loading, setLoading] = useState(true);

  const [toggling, setToggling] = useState<string | null>(null);
  const [isScannerToggling, setIsScannerToggling] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [consensusEnabled, setConsensusEnabled] = useState<boolean>(true);
  const [consensusLastQueried, setConsensusLastQueried] = useState<string | null>(null);
  const [isConsensusToggling, setIsConsensusToggling] = useState(false);

  const fetchScannerData = useCallback(async () => {
    try {
      const [signalsRes, statusRes, strategiesRes, contendersRes, consensusRes] = await Promise.all([
        arbFarmService.getSignals({ limit: 15 }),
        arbFarmService.getScannerStatus(),
        arbFarmService.listBehavioralStrategies(),
        arbFarmService.getContenders(10),
        arbFarmService.getConsensusSchedulerStatus(),
      ]);

      if (signalsRes.success && signalsRes.data) {
        const signalData = signalsRes.data as { signals?: Signal[]; count?: number };
        setSignals(signalData.signals || []);
      }
      if (statusRes.success && statusRes.data) {
        setScannerStatus(statusRes.data);
      }
      if (strategiesRes.success && strategiesRes.data) {
        setStrategies(strategiesRes.data.strategies || []);
      }
      if (contendersRes.success && contendersRes.data) {
        setContenders(contendersRes.data.contenders || []);
      }
      if (consensusRes.success && consensusRes.data) {
        setConsensusEnabled(consensusRes.data.scheduler_enabled);
        setConsensusLastQueried(consensusRes.data.last_queried);
      }
      setError(null);
    } catch (e) {
      console.error('Scanner HUD data fetch error:', e);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchScannerData();
    const interval = setInterval(fetchScannerData, 10000);
    return () => clearInterval(interval);
  }, [fetchScannerData]);

  useEffect(() => {
    const currentIds = liveTrades.map(t => t.id);
    const newIds = currentIds.filter(id => !prevTradesRef.current.includes(id));

    if (newIds.length > 0) {
      setNewTradeIds(prev => new Set([...prev, ...newIds]));
      setTimeout(() => {
        setNewTradeIds(prev => {
          const next = new Set(prev);
          newIds.forEach(id => next.delete(id));
          return next;
        });
      }, 3000);
    }

    prevTradesRef.current = currentIds;
  }, [liveTrades]);

  const handleToggleScanner = async () => {
    setIsScannerToggling(true);
    try {
      const isRunning = scannerStatus?.is_running ?? false;
      const res = isRunning
        ? await arbFarmService.stopScanner()
        : await arbFarmService.startScanner();
      if (res.success) {
        setScannerStatus(prev => prev ? { ...prev, is_running: !isRunning } : { is_running: !isRunning, stats: {} } as ScannerStatus);
      } else {
        setError(res.error || 'Failed to toggle scanner');
      }
    } catch {
      setError('Failed to toggle scanner');
    } finally {
      setIsScannerToggling(false);
    }
  };

  const handleToggleConsensus = async () => {
    setIsConsensusToggling(true);
    try {
      const res = await arbFarmService.toggleConsensusScheduler(!consensusEnabled);
      if (res.success && res.data) {
        setConsensusEnabled(res.data.scheduler_enabled);
        setConsensusLastQueried(res.data.last_queried);
      } else {
        setError('Failed to toggle consensus scheduler');
      }
    } catch {
      setError('Failed to toggle consensus scheduler');
    } finally {
      setIsConsensusToggling(false);
    }
  };

  const formatLastQueried = (iso: string | null): string => {
    if (!iso) return 'Never';
    const date = new Date(iso);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    if (diffMins < 1) return 'Just now';
    if (diffMins < 60) return `${diffMins}m ago`;
    const diffHours = Math.floor(diffMins / 60);
    if (diffHours < 24) return `${diffHours}h ago`;
    return date.toLocaleDateString();
  };

  const handleToggleStrategy = async (name: string, currentActive: boolean) => {
    setToggling(name);
    try {
      const res = await arbFarmService.toggleBehavioralStrategy(name, !currentActive);
      if (res.success) {
        setStrategies(prev => prev.map(s => s.name === name ? { ...s, is_active: !currentActive } : s));
      } else {
        setError(res.error || 'Failed to toggle strategy');
      }
    } catch {
      setError('Failed to toggle strategy');
    } finally {
      setToggling(null);
    }
  };

  const handleToggleAll = async (active: boolean) => {
    setToggling('all');
    try {
      const res = await arbFarmService.toggleAllBehavioralStrategies(active);
      if (res.success) {
        setStrategies(prev => prev.map(s => ({ ...s, is_active: active })));
      } else {
        setError(res.error || 'Failed to toggle all strategies');
      }
    } catch {
      setError('Failed to toggle all strategies');
    } finally {
      setToggling(null);
    }
  };

  const formatTime = (timestamp: string): string => {
    const date = new Date(timestamp);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    if (diff < 60000) return 'now';
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m`;
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}h`;
    return date.toLocaleDateString();
  };

  const formatSignalTime = (timestamp: string): string => {
    const date = new Date(timestamp);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    if (diff < 60000) return 'Just now';
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
    return date.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit' });
  };

  const getSignalIcon = (signal: Signal): string => {
    const source = (signal.metadata as Record<string, unknown>)?.signal_source as string | undefined;
    if (source) {
      if (source.includes('graduation_sniper')) return '\uD83C\uDF93';
    }
    const type = signal.signal_type.toLowerCase();
    if (type.includes('graduation')) return '\uD83C\uDF93';
    if (type.includes('volume')) return '\uD83D\uDCCA';
    if (type.includes('momentum')) return '\uD83D\uDE80';
    if (type.includes('price')) return '\uD83D\uDCB0';
    if (type.includes('liquidity')) return '\uD83D\uDCA7';
    if (type.includes('new_token')) return '\u2728';
    return '\uD83D\uDCE1';
  };

  const getSignalSource = (signal: Signal): string | null => {
    const source = (signal.metadata as Record<string, unknown>)?.signal_source as string | undefined;
    if (!source) return null;
    return source.replace(/_/g, ' ');
  };

  const getSignificanceColor = (confidence: number): string => {
    if (confidence >= 80) return styles.sigCritical;
    if (confidence >= 60) return styles.sigHigh;
    if (confidence >= 40) return styles.sigMedium;
    return styles.sigLow;
  };

  const getStrategyIcon = (strategyType: string): string => {
    switch (strategyType) {
      case 'copy_trade': return '\uD83D\uDC65';
      case 'graduation_sniper': return '\uD83C\uDFAF';
      default: return '\u26A1';
    }
  };

  const getVenueLabel = (venue: string): string => {
    if (venue.includes('BondingCurve')) return 'Curves';
    if (venue.includes('DexAmm')) return 'DEX';
    return venue;
  };

  const filteredRecentTrades = recentTrades.filter(t =>
    t.exit_type !== 'Emergency' && t.exit_type !== 'AlreadySold'
  );
  const hasActivity = liveTrades.length > 0 || filteredRecentTrades.length > 0;
  const activeCount = strategies.filter(s => s.is_active).length;
  const totalCount = strategies.length;
  const isActive = scannerStatus?.is_running;

  const renderTradesContent = () => (
    <div className={styles.activityCardContent}>
      {!hasActivity ? (
        <div className={styles.activityEmptyState}>
          <span className={styles.activityEmptyText}>No trades yet</span>
        </div>
      ) : (
        <div className={styles.activityList}>
          {liveTrades.slice(0, 15).map((trade) => (
            <div
              key={trade.id}
              className={`${styles.tradeRow} ${styles.liveRow} ${newTradeIds.has(trade.id) ? styles.newRow : ''}`}
              onClick={() => onTradeClick?.(trade, true)}
            >
              <div className={styles.tradeLeft}>
                <span className={styles.tradeSymbol}>
                  {trade.token_symbol || trade.id.slice(0, 6)}
                </span>
                <span className={styles.tradeMeta}>
                  {trade.tx_signature ? (
                    <a
                      href={`https://solscan.io/tx/${trade.tx_signature}`}
                      target="_blank"
                      rel="noopener noreferrer"
                      onClick={(e) => e.stopPropagation()}
                      className={styles.txLink}
                    >
                      {trade.tx_signature.slice(0, 6)}...
                    </a>
                  ) : (
                    'pending'
                  )}
                </span>
              </div>
              <div className={styles.tradeRight}>
                <span className={styles.tradeAmount}>
                  {(trade.entry_price ?? 0) > 0 ? `${(trade.entry_price ?? 0).toFixed(4)}` : '--'}
                </span>
                <span className={styles.tradeTime}>{formatTime(trade.executed_at)}</span>
              </div>
            </div>
          ))}

          {filteredRecentTrades.slice(0, 30).map((trade, idx) => {
            const mom = trade.momentum_at_exit ?? 0;
            const momEmoji = mom >= 30 ? '\uD83D\uDE80' : mom >= 0 ? '\uD83D\uDCC8' : mom >= -30 ? '\uD83D\uDCC9' : '\uD83D\uDC80';
            const holdMins = trade.hold_duration_mins;
            const holdStr = holdMins != null
              ? (holdMins < 60 ? `${holdMins}m` : `${Math.floor(holdMins / 60)}h${holdMins % 60 > 0 ? `${holdMins % 60}m` : ''}`)
              : null;
            return (
              <div
                key={trade.id || `trade-${idx}`}
                className={`${styles.tradeRow} ${styles.closedRow}`}
                onClick={() => onTradeClick?.(trade, false)}
              >
                <div className={styles.tradeLeft}>
                  <span className={styles.tradeSymbol}>{trade.symbol || '???'}</span>
                  <span className={styles.tradeMeta}>
                    {trade.exit_type || 'closed'}
                    {holdStr && ` · ${holdStr}`}
                    {' '}· {trade.time_ago || '--'}
                    {trade.momentum_at_exit !== undefined && trade.momentum_at_exit !== null && (
                      <span className={styles.tradeMomentum} title={`Momentum at exit: ${mom.toFixed(0)}`}>
                        {' '}· {mom.toFixed(0)}{momEmoji}
                      </span>
                    )}
                  </span>
                </div>
                <div className={styles.tradeRight}>
                  <span className={styles.tradeEntrySol}>
                    {(trade.entry_amount_sol ?? 0).toFixed(2)}
                  </span>
                  <span className={`${styles.tradePnl} ${(trade.pnl ?? 0) >= 0 ? styles.profit : styles.loss}`}>
                    {(trade.pnl ?? 0) >= 0 ? '+' : ''}{(trade.pnl ?? 0).toFixed(4)}
                  </span>
                  <span className={`${styles.tradePercent} ${(trade.pnl_percent ?? 0) >= 0 ? styles.profit : styles.loss}`}>
                    {(trade.pnl_percent ?? 0) >= 0 ? '+' : ''}{(trade.pnl_percent ?? 0).toFixed(1)}%
                  </span>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );

  const renderScannerContent = () => {
    const signaledMints = new Set(signals.map(s => s.token_mint).filter(Boolean));
    const nonContenderSignals = signals.filter(s => !s.token_mint || !contenders.some(c => c.mint === s.token_mint));

    if (!isActive && signals.length === 0 && contenders.length === 0) {
      return (
        <div className={styles.activityCardContent}>
          <div className={styles.activityEmptyState}>
            <span className={styles.activityEmptyIcon}>{'\uD83D\uDCE1'}</span>
            <span>Scanner is paused</span>
            <span className={styles.activityEmptyHint}>Start scanner to detect opportunities</span>
          </div>
        </div>
      );
    }

    return (
      <div className={styles.activityCardContent}>
        {contenders.length > 0 && (
          <div className={styles.gradGrid}>
            {contenders.map((token) => {
              const vsrLamports = (token.metadata as Record<string, number>)?.vsr_lamports ?? 0;
              const vsrSol = vsrLamports / 1e9;
              const gradSol = 85;
              const isHot = token.graduation_progress >= 85;
              const isWarm = token.graduation_progress >= 70;
              const hasSignal = signaledMints.has(token.mint);
              return (
                <div
                  key={token.mint}
                  className={`${styles.gradCard} ${isHot ? styles.gradCardHot : isWarm ? styles.gradCardWarm : ''} ${hasSignal ? styles.gradCardSignaled : ''}`}
                  onClick={() => onContenderClick?.(token.mint, token.symbol || token.name)}
                >
                  <div className={styles.gradCardTop}>
                    <span className={styles.gradSymbol}>
                      {hasSignal && <span className={styles.gradSignalBadge}>SIGNAL</span>}
                      {token.symbol || token.name}
                    </span>
                    <span className={styles.gradPercent}>{token.graduation_progress.toFixed(1)}%</span>
                  </div>
                  <div className={styles.gradBarTrack}>
                    <div
                      className={`${styles.gradBarFill} ${isHot ? styles.gradBarHot : isWarm ? styles.gradBarWarm : ''}`}
                      style={{ width: `${Math.min(token.graduation_progress, 100)}%` }}
                    />
                  </div>
                  <div className={styles.gradSolProgress}>
                    {vsrSol > 0 ? `${vsrSol.toFixed(1)}` : '?'} / {gradSol} SOL
                  </div>
                  <div className={styles.gradStats}>
                    <span>{token.market_cap_sol.toFixed(1)} SOL mcap</span>
                    <span>
                      {token.volume_24h_sol > 0
                        ? `${token.volume_24h_sol.toFixed(1)} 24h`
                        : 'No vol'}
                    </span>
                    {token.volume_1h_sol != null && token.volume_1h_sol > 0 && (
                      <span>{token.volume_1h_sol.toFixed(2)} 1h</span>
                    )}
                    <span className={styles.gradMint}>{token.mint.slice(0, 6)}...</span>
                  </div>
                </div>
              );
            })}
          </div>
        )}

        {nonContenderSignals.length > 0 && (
          <>
            {contenders.length > 0 && <div className={styles.scannerDivider}>Other Signals</div>}
            <div className={styles.signalsStream}>
              {nonContenderSignals.map((signal) => (
                <div
                  key={signal.id}
                  className={styles.signalItem}
                  onClick={() => onSignalClick?.(signal)}
                >
                  <div className={styles.signalIcon}>
                    <span>{getSignalIcon(signal)}</span>
                  </div>
                  <div className={styles.signalContent}>
                    <div className={styles.signalHeader}>
                      <span className={styles.signalType}>
                        {signal.signal_type.replace(/_/g, ' ')}
                      </span>
                      {getSignalSource(signal) && (
                        <span className={styles.signalSourceBadge}>
                          {getSignalSource(signal)}
                        </span>
                      )}
                      <span className={styles.signalTime}>{formatSignalTime(signal.detected_at)}</span>
                    </div>
                    <div className={styles.signalDetails}>
                      {signal.token_mint && (
                        <span className={styles.signalMint}>
                          {signal.token_mint.slice(0, 6)}...
                        </span>
                      )}
                      <span className={`${styles.signalConfidence} ${getSignificanceColor(signal.confidence)}`}>
                        {signal.confidence}% conf
                      </span>
                      {signal.estimated_profit_bps > 0 && (
                        <span className={styles.signalProfit}>
                          +{(signal.estimated_profit_bps / 100).toFixed(1)}%
                        </span>
                      )}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </>
        )}

        {contenders.length === 0 && nonContenderSignals.length === 0 && (
          <div className={styles.activityEmptyState}>
            <span className={styles.activityEmptyIcon}>{'\uD83D\uDD0D'}</span>
            <span>No signals detected</span>
            <span className={styles.activityEmptyHint}>Scanner is active and monitoring</span>
          </div>
        )}
      </div>
    );
  };

  const renderStrategiesContent = () => (
    <div className={styles.activityCardContent}>
      <div className={styles.scannerControl}>
        <div className={styles.scannerInfo}>
          <span className={scannerStatus?.is_running ? styles.statusActive : styles.statusInactive}>
            {scannerStatus?.is_running ? 'Scanner Running' : 'Scanner Stopped'}
          </span>
          <span className={styles.scannerStats}>
            {scannerStatus?.stats?.total_scans || 0} scans · {scannerStatus?.stats?.total_signals || 0} signals
          </span>
        </div>
        <div className={styles.scannerInfo}>
          <span className={monitorActive ? styles.statusActive : styles.statusInactive}>
            {monitorActive ? 'Monitor ON' : 'Monitor OFF'}
          </span>
          <span className={executionEnabled ? styles.statusActive : styles.statusInactive}>
            {executionEnabled ? 'Executor ON' : 'Executor OFF'}
          </span>
        </div>
      </div>

      <div className={styles.strategyList}>
        {strategies.length === 0 ? (
          <div className={styles.emptyState}>No behavioral strategies registered</div>
        ) : (
          strategies.map((strategy) => (
            <div
              key={strategy.name}
              className={`${styles.strategyItem} ${strategy.is_active ? styles.active : styles.inactive}`}
            >
              <div className={styles.strategyInfo}>
                <span className={styles.strategyIcon}>
                  {getStrategyIcon(strategy.strategy_type)}
                </span>
                <div className={styles.strategyDetails}>
                  <span className={styles.strategyName}>{strategy.name}</span>
                  <span className={styles.strategyMeta}>
                    {strategy.strategy_type} · {strategy.supported_venues.map(getVenueLabel).join(', ')}
                  </span>
                  {strategy.status_detail && (
                    <span className={styles.strategyMeta} style={{
                      color: strategy.status_detail.includes('connected') ? '#22c55e' : '#f59e0b',
                      fontSize: '0.7rem',
                    }}>
                      {strategy.status_detail}
                    </span>
                  )}
                </div>
              </div>
              <span className={strategy.is_active ? styles.statusBadgeActive : styles.statusBadgeInactive}>
                {strategy.is_active ? 'Active' : 'Inactive'}
              </span>
            </div>
          ))
        )}
      </div>
    </div>
  );


  const getStrategyTooltip = (strategyType: string): string => {
    switch (strategyType) {
      case 'graduation_sniper':
        return 'Buys tokens at 85-100% graduation progress on the bonding curve before they migrate to Raydium';
      case 'raydium_snipe':
        return 'Detects tokens graduating to Raydium and buys via Jupiter swap immediately after migration';
      case 'copy_trade':
        return 'Mirrors trades from tracked KOL wallets via Helius webhooks';
      default:
        return `${strategyType} strategy`;
    }
  };

  const renderControlContent = () => (
    <div className={styles.activityCardContent}>
      <div className={styles.controlPanel}>
        <div className={styles.controlSection}>
          <div className={styles.controlSectionTitle}>System Controls</div>
          <div className={styles.controlRow}>
            <button
              type="button"
              className={`${styles.controlBtn} ${scannerStatus?.is_running ? styles.controlBtnActive : ''}`}
              onClick={() => void handleToggleScanner()}
              disabled={isScannerToggling}
            >
              <span className={`${styles.statusDot} ${scannerStatus?.is_running ? styles.statusDotOn : ''}`} />
              {isScannerToggling ? 'Processing...' : 'Enable Scanner'}
              <span className={styles.controlHelpIcon} onClick={(e) => e.stopPropagation()}>?
                <span className={styles.controlTooltip}>Polls venues for token data and feeds strategies. No signals are generated when off.</span>
              </span>
            </button>
            <button
              type="button"
              className={`${styles.controlBtn} ${monitorActive ? styles.controlBtnActive : ''}`}
              onClick={onToggleMonitor}
              disabled={togglingMonitor}
            >
              <span className={`${styles.statusDot} ${monitorActive ? styles.statusDotOn : ''}`} />
              {togglingMonitor ? 'Processing...' : 'Enable Monitor'}
              <span className={styles.controlHelpIcon} onClick={(e) => e.stopPropagation()}>?
                <span className={styles.controlTooltip}>Tracks open positions and detects exit conditions (stop loss, take profit, trailing stop, time limit). Queues sell transactions for the executor.</span>
              </span>
            </button>
            <button
              type="button"
              className={`${styles.controlBtn} ${executionEnabled ? styles.controlBtnActive : ''}`}
              onClick={onToggleExecution}
              disabled={togglingExecution}
            >
              <span className={`${styles.statusDot} ${executionEnabled ? styles.statusDotOn : ''}`} />
              {togglingExecution ? 'Processing...' : 'Enable Executor'}
              <span className={styles.controlHelpIcon} onClick={(e) => e.stopPropagation()}>?
                <span className={styles.controlTooltip}>Executes buy and sell transactions from signals and exit conditions. When off, signals are generated but no trades are placed.</span>
              </span>
            </button>
          </div>
        </div>

        <div className={styles.controlSection}>
          <div className={styles.controlSectionTitle}>Danger Zone</div>
          <div className={styles.controlRow}>
            <button
              type="button"
              className={styles.controlBtnDanger}
              onClick={onSellAll}
              disabled={sellingAll || positionCount === 0}
            >
              {sellingAll ? 'Selling...' : 'Sell All Tokens'}
              <span className={styles.controlHelpIcon} onClick={(e) => e.stopPropagation()}>?
                <span className={styles.controlTooltip}>Market sells all open token positions back to SOL. Requires confirmation.</span>
              </span>
            </button>
          </div>
        </div>

        <div className={styles.controlSection}>
          <div className={styles.controlSectionTitle}>Consensus Engine</div>
          <div className={styles.controlRow}>
            <button
              type="button"
              className={`${styles.controlBtn} ${consensusEnabled ? styles.controlBtnActive : ''}`}
              onClick={() => void handleToggleConsensus()}
              disabled={isConsensusToggling}
            >
              <span className={`${styles.statusDot} ${consensusEnabled ? styles.statusDotOn : ''}`} />
              {isConsensusToggling ? 'Processing...' : 'Enable Consensus'}
              <span className={styles.controlHelpIcon} onClick={(e) => e.stopPropagation()}>?
                <span className={`${styles.controlTooltip} ${styles.controlTooltipUp}`}>Runs periodic LLM analysis every 5 minutes — reviews trade performance, generates strategy recommendations, and saves pattern summaries. Disabling stops the scheduled queries but keeps the consensus API available for manual requests.</span>
              </span>
            </button>
            <span className={styles.controlMeta} title={consensusLastQueried ? new Date(consensusLastQueried).toLocaleString() : 'Never queried'}>
              Last queried: {formatLastQueried(consensusLastQueried)}
            </span>
          </div>
        </div>

        <div className={styles.controlSection}>
          <div className={styles.controlSectionTitle}>Wallet</div>
          <div className={styles.controlRow}>
            <button
              type="button"
              className={styles.controlBtn}
              onClick={onReconcile}
              disabled={reconciling}
            >
              {reconciling ? 'Reconciling...' : 'Reconcile Positions'}
              <span className={styles.controlHelpIcon} onClick={(e) => e.stopPropagation()}>?
                <span className={`${styles.controlTooltip} ${styles.controlTooltipUp}`}>Syncs wallet balances from the blockchain and fixes any position tracking discrepancies</span>
              </span>
            </button>
          </div>
        </div>

        <div className={styles.controlSection}>
          <div className={styles.controlSectionTitle}>Strategies</div>
          <div className={styles.controlRow}>
            {strategies.map((strategy) => (
              <button
                key={strategy.name}
                type="button"
                className={`${styles.controlBtn} ${strategy.is_active ? styles.controlBtnActive : ''}`}
                onClick={() => void handleToggleStrategy(strategy.name, strategy.is_active)}
                disabled={toggling === strategy.name}
              >
                <span className={`${styles.statusDot} ${strategy.is_active ? styles.statusDotOn : ''}`} />
                {toggling === strategy.name ? '...' : strategy.name}
                <span className={styles.controlHelpIcon} onClick={(e) => e.stopPropagation()}>?
                  <span className={`${styles.controlTooltip} ${styles.controlTooltipUp}`}>{getStrategyTooltip(strategy.strategy_type)}</span>
                </span>
              </button>
            ))}
          </div>
        </div>
      </div>
    </div>
  );

  if (loading) {
    return (
      <div className={styles.activityCard}>
        <div className={styles.activityCardContent}>
          <div className={styles.activityEmptyState}>
            <span className={styles.logSpinner}>{'\u25E0'}</span>
            <span>Loading scanner data...</span>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.activityCard}>
      <div className={styles.scannerHudHeader}>
        <span className={styles.scannerHudTitle}>Quick Actions</span>
        <div className={styles.scannerHudBadges}>
          <span className={`${styles.activityBadge} ${activeCount > 0 ? styles.badgeActive : styles.badgeInactive}`}>
            {activeCount}/{totalCount} active
          </span>
          <span
            className={styles.scannerHudToggle}
            title={isActive ? 'Scanner Running' : 'Scanner Stopped'}
          >
            <span className={`${styles.scannerStatusDot} ${isActive ? styles.dotActive : styles.dotInactive}`} />
          </span>
        </div>
      </div>

      <div className={styles.activityTabHeader}>
        <button
          className={`${styles.activityTab} ${activeTab === 'trades' ? styles.activeTab : ''}`}
          onClick={() => setActiveTab('trades')}
        >
          {liveTrades.length > 0 && <span className={styles.liveIndicator} />}
          Trades
          {(liveTrades.length + filteredRecentTrades.length) > 0 && (
            <span className={styles.tabBadge}>{liveTrades.length + filteredRecentTrades.length}</span>
          )}
        </button>
        <button
          className={`${styles.activityTab} ${activeTab === 'scanner' ? styles.activeTab : ''}`}
          onClick={() => setActiveTab('scanner')}
        >
          Scanner
          {(contenders.length + signals.length) > 0 && (
            <span className={styles.tabBadge}>{contenders.length + signals.length}</span>
          )}
        </button>
        <button
          className={`${styles.activityTab} ${activeTab === 'strategies' ? styles.activeTab : ''}`}
          onClick={() => setActiveTab('strategies')}
        >
          Strategies
        </button>
        <button
          className={`${styles.activityTab} ${activeTab === 'control' ? styles.activeTab : ''}`}
          onClick={() => setActiveTab('control')}
        >
          Control
        </button>
      </div>

      {activeTab === 'trades' && renderTradesContent()}
      {activeTab === 'scanner' && renderScannerContent()}
      {activeTab === 'strategies' && renderStrategiesContent()}
      {activeTab === 'control' && renderControlContent()}
    </div>
  );
};

export default ScannerHUD;
