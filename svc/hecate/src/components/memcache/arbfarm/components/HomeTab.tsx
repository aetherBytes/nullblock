import React, { useEffect, useState, useCallback } from 'react';
import styles from '../arbfarm.module.scss';
import { arbFarmService } from '../../../../common/services/arbfarm-service';
import DashboardPositionCard from './DashboardPositionCard';
import TradeActivityCard from './TradeActivityCard';
import TokenDetailModal from './TokenDetailModal';
import CurveMetricsPanel from './CurveMetricsPanel';
import BehavioralStrategiesPanel from './behavioral-strategies-panel';
import ScannerSignalsPanel from './ScannerSignalsPanel';
import type {
  PnLSummary,
  OpenPosition,
  WalletBalanceResponse,
  PositionExposure,
  MonitorStatus,
  RecentTradeInfo,
  ScannerStatus,
  SniperStats,
  TopOpportunity,
  CurveToken,
} from '../../../../types/arbfarm';

interface LiveTrade {
  id: string;
  edge_id: string;
  tx_signature?: string;
  entry_price: number;
  executed_at: string;
  token_mint?: string;
  token_symbol?: string;
  venue?: string;
}

interface HomeTabProps {
  liveTrades?: LiveTrade[];
}

interface SelectedToken {
  mint: string;
  symbol?: string;
  venue?: string;
  closedTrade?: {
    pnl?: number;
    pnl_percent?: number;
    entry_price?: number;
    exit_price?: number;
    entry_amount_sol?: number;
    exit_type?: string;
    time_ago?: string;
  };
}

const HomeTab: React.FC<HomeTabProps> = ({ liveTrades }) => {
  const [pnlSummary, setPnlSummary] = useState<PnLSummary | null>(null);
  const [realPositions, setRealPositions] = useState<OpenPosition[]>([]);
  const [walletBalance, setWalletBalance] = useState<WalletBalanceResponse | null>(null);
  const [exposure, setExposure] = useState<PositionExposure | null>(null);
  const [monitorStatus, setMonitorStatus] = useState<MonitorStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [closingPosition, setClosingPosition] = useState<string | null>(null);
  const [selectedToken, setSelectedToken] = useState<SelectedToken | null>(null);
  const [sellingAll, setSellingAll] = useState(false);
  const [reconciling, setReconciling] = useState(false);
  const [selectedMetricsToken, setSelectedMetricsToken] = useState<{
    mint: string;
    venue: string;
    symbol: string;
  } | null>(null);

  // Automation state
  const [scannerStatus, setScannerStatus] = useState<ScannerStatus | null>(null);
  const [sniperStats, setSniperStats] = useState<SniperStats | null>(null);
  const [executionEnabled, setExecutionEnabled] = useState<boolean | null>(null);
  const [autoExitStats, setAutoExitStats] = useState<{ total_positions: number; auto_exit_enabled: number; manual_mode: number } | null>(null);

  // Command bar state
  const [topOpportunities, setTopOpportunities] = useState<TopOpportunity[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [searchResults, setSearchResults] = useState<CurveToken[]>([]);
  const [searchFocused, setSearchFocused] = useState(false);

  const fetchData = useCallback(async () => {
    try {
      const [pnlRes, positionsRes, balanceRes, exposureRes, monitorRes, scannerRes, sniperRes, execRes, autoExitRes, opportunitiesRes] = await Promise.all([
        arbFarmService.getPnLSummary(),
        arbFarmService.getPositions(),
        arbFarmService.getWalletBalance(),
        arbFarmService.getPositionExposure(),
        arbFarmService.getMonitorStatus(),
        arbFarmService.getScannerStatus(),
        arbFarmService.getSniperStats(),
        arbFarmService.getExecutionConfig(),
        arbFarmService.getAutoExitStats(),
        arbFarmService.getTopOpportunities(5),
      ]);

      if (pnlRes.success && pnlRes.data) {
        setPnlSummary(pnlRes.data);
      }
      if (positionsRes.success && positionsRes.data) {
        setRealPositions(positionsRes.data.positions || []);
      }
      if (balanceRes.success && balanceRes.data) {
        setWalletBalance(balanceRes.data);
      }
      if (exposureRes.success && exposureRes.data) {
        setExposure(exposureRes.data);
      }
      if (monitorRes.success && monitorRes.data) {
        setMonitorStatus(monitorRes.data);
      }
      if (scannerRes.success && scannerRes.data) {
        setScannerStatus(scannerRes.data);
      }
      if (sniperRes.success && sniperRes.data?.stats) {
        setSniperStats(sniperRes.data.stats);
      }
      if (execRes.success && execRes.data) {
        setExecutionEnabled(execRes.data.auto_execution_enabled);
      }
      if (autoExitRes.success && autoExitRes.data) {
        setAutoExitStats(autoExitRes.data);
      }
      if (opportunitiesRes.success && opportunitiesRes.data) {
        setTopOpportunities(opportunitiesRes.data.opportunities || []);
      }
    } catch (error) {
      console.error('Failed to fetch home tab data:', error);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    setLoading(true);
    fetchData();
    const interval = setInterval(fetchData, 15000);
    return () => clearInterval(interval);
  }, [fetchData]);

  // Search handler with debounce
  useEffect(() => {
    if (searchQuery.length < 2) {
      setSearchResults([]);
      return;
    }
    const timer = setTimeout(async () => {
      const res = await arbFarmService.listCurveTokens(undefined, 50);
      if (res.success && res.data) {
        const filtered = res.data.filter((t: CurveToken) =>
          t.symbol?.toLowerCase().includes(searchQuery.toLowerCase()) ||
          t.mint.toLowerCase().includes(searchQuery.toLowerCase())
        );
        setSearchResults(filtered.slice(0, 5));
      }
    }, 300);
    return () => clearTimeout(timer);
  }, [searchQuery]);


  const handleClosePosition = async (positionId: string, exitPercent: number = 100): Promise<boolean> => {
    setClosingPosition(positionId);
    try {
      const res = await arbFarmService.closePosition(positionId, exitPercent);
      if (res.success) {
        await fetchData();
        return true;
      }
      console.error('Sell failed:', res.error);
      return false;
    } catch (error) {
      console.error('Failed to close position:', error);
      return false;
    } finally {
      setClosingPosition(null);
    }
  };

  const handleSellAll = async () => {
    if (!confirm('Are you sure you want to sell ALL tokens in your wallet?')) return;
    setSellingAll(true);
    try {
      await arbFarmService.sellAllWalletTokens();
      await fetchData();
    } catch (error) {
      console.error('Failed to sell all:', error);
    } finally {
      setSellingAll(false);
    }
  };

  const handleReconcile = async () => {
    setReconciling(true);
    try {
      const res = await arbFarmService.reconcilePositions();
      if (res.success && res.data) {
        console.log('Reconciliation result:', res.data);
      }
      await fetchData();
    } catch (error) {
      console.error('Failed to reconcile:', error);
    } finally {
      setReconciling(false);
    }
  };

  const handleToggleMonitor = async () => {
    try {
      if (monitorStatus?.monitoring_active) {
        await arbFarmService.stopPositionMonitor();
      } else {
        await arbFarmService.startPositionMonitor();
      }
      const res = await arbFarmService.getMonitorStatus();
      if (res.success && res.data) {
        setMonitorStatus(res.data);
      }
    } catch (error) {
      console.error('Failed to toggle monitor:', error);
    }
  };

  const handleToggleScanner = async () => {
    try {
      if (scannerStatus?.is_running) {
        await arbFarmService.stopScanner();
      } else {
        await arbFarmService.startScanner();
      }
      const res = await arbFarmService.getScannerStatus();
      if (res.success && res.data) {
        setScannerStatus(res.data);
      }
    } catch (error) {
      console.error('Failed to toggle scanner:', error);
    }
  };

  const handleToggleSniper = async () => {
    try {
      if (sniperStats?.is_running) {
        await arbFarmService.stopSniper();
      } else {
        await arbFarmService.startSniper();
      }
      const res = await arbFarmService.getSniperStats();
      if (res.success && res.data?.stats) {
        setSniperStats(res.data.stats);
      }
    } catch (error) {
      console.error('Failed to toggle sniper:', error);
    }
  };

  const handleToggleExecution = async () => {
    try {
      const newEnabled = !executionEnabled;
      const res = await arbFarmService.toggleExecution(newEnabled);
      if (res.success && res.data) {
        setExecutionEnabled(res.data.enabled);
      }
    } catch (error) {
      console.error('Failed to toggle execution:', error);
    }
  };

  if (loading) {
    return (
      <div className={styles.hudDashboard}>
        <div className={styles.loadingContainer}>
          <div className={styles.loadingSpinner}>
            <div className={styles.spinnerRing}></div>
          </div>
          <span className={styles.loadingText}>Loading dashboard data...</span>
        </div>
      </div>
    );
  }

  if (selectedToken) {
    return (
      <div className={styles.hudDashboard}>
        <TokenDetailModal
          mint={selectedToken.mint}
          symbol={selectedToken.symbol}
          venue={selectedToken.venue}
          closedTrade={selectedToken.closedTrade}
          onClose={() => setSelectedToken(null)}
          onRefresh={fetchData}
          asPanel={true}
        />
      </div>
    );
  }

  return (
    <div className={styles.hudDashboard}>
      {/* Command Bar */}
      <div className={styles.hudCommandBar}>
        {/* Next Picks - Top 3 opportunities */}
        <div className={styles.nextPicksWidget}>
          <span className={styles.widgetLabel}>NEXT PICKS</span>
          <div className={styles.picksList}>
            {topOpportunities.slice(0, 3).map((opp, i) => (
              <div
                key={opp.mint}
                className={styles.pickItem}
                onClick={() => setSelectedToken({ mint: opp.mint, symbol: opp.symbol, venue: opp.venue })}
              >
                <span className={styles.pickRank}>#{i + 1}</span>
                <span className={styles.pickSymbol}>{opp.symbol}</span>
                <span className={styles.pickScore}>{opp.score.overall_score.toFixed(0)}</span>
                <div className={styles.pickMomentum} style={{ width: `${opp.metrics.graduation_progress}%` }} />
              </div>
            ))}
            {topOpportunities.length === 0 && (
              <span className={styles.emptyPicks}>No picks available</span>
            )}
          </div>
        </div>

        {/* Quick Search */}
        <div className={styles.searchWidget}>
          <input
            type="text"
            placeholder="Search token or mint..."
            className={styles.searchInput}
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            onFocus={() => setSearchFocused(true)}
            onBlur={() => setTimeout(() => setSearchFocused(false), 200)}
          />
          {searchFocused && searchResults.length > 0 && (
            <div className={styles.searchResults}>
              {searchResults.map((token) => (
                <div
                  key={token.mint}
                  className={styles.searchResultItem}
                  onClick={() => {
                    setSelectedToken({ mint: token.mint, symbol: token.symbol, venue: token.venue });
                    setSearchQuery('');
                  }}
                >
                  <span className={styles.resultSymbol}>{token.symbol}</span>
                  <span className={styles.resultMint}>{token.mint.slice(0, 8)}...</span>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Signal Counter */}
        <div className={styles.signalWidget}>
          <span className={styles.widgetLabel}>SIGNALS</span>
          <span className={styles.signalCount}>{scannerStatus?.stats?.total_signals ?? 0}</span>
          <span className={styles.signalSub}>detected</span>
        </div>

        {/* Best Trade Badge */}
        <div className={styles.bestTradeWidget}>
          <span className={styles.widgetLabel}>BEST TRADE</span>
          <span className={styles.bestSymbol}>{pnlSummary?.best_trade?.symbol || '-'}</span>
          <span className={`${styles.bestPnl} ${(pnlSummary?.best_trade?.pnl ?? 0) >= 0 ? styles.profit : ''}`}>
            {pnlSummary?.best_trade?.pnl ? `+${pnlSummary.best_trade.pnl.toFixed(3)} SOL` : '-'}
          </span>
        </div>

        {/* Quick Actions */}
        <div className={styles.quickActions}>
          <button onClick={handleToggleScanner} className={styles.quickBtn}>
            {scannerStatus?.is_running ? '\u23F8' : '\u25B6'} Scanner
          </button>
          <button onClick={handleToggleSniper} className={styles.quickBtn}>
            {sniperStats?.is_running ? '\u23F8' : '\u25B6'} Sniper
          </button>
        </div>
      </div>

      {/* Main Area: Positions + Activity */}
      <div className={styles.hudMainArea}>
        {/* Positions Panel */}
        <div className={styles.hudPanel}>
          <div className={styles.hudPanelHeader}>
            <h3 className={styles.hudPanelTitle}>Open Positions ({realPositions.length})</h3>
          </div>
          <div className={styles.hudAutomationRow}>
            <div className={styles.hudToggle} onClick={handleToggleExecution}>
              <span className={styles.hudToggleLabel}>Execution</span>
              <span className={`${styles.hudToggleSwitch} ${executionEnabled ? styles.on : styles.off}`}>
                {executionEnabled ? 'ON' : 'OFF'}
              </span>
            </div>
            <div className={styles.hudToggle} onClick={handleToggleScanner}>
              <span className={styles.hudToggleLabel}>Scanner</span>
              <span className={`${styles.hudToggleSwitch} ${scannerStatus?.is_running ? styles.on : styles.off}`}>
                {scannerStatus?.is_running ? 'ON' : 'OFF'}
              </span>
            </div>
            <div className={styles.hudToggle} onClick={handleToggleSniper}>
              <span className={styles.hudToggleLabel}>Sniper</span>
              <span className={`${styles.hudToggleSwitch} ${sniperStats?.is_running ? styles.on : styles.off}`}>
                {sniperStats?.is_running ? 'ON' : 'OFF'}
              </span>
            </div>
            <div className={styles.hudToggle}>
              <span className={styles.hudToggleLabel}>Auto-Exits</span>
              <span className={styles.hudToggleSwitch}>
                {autoExitStats ? `${autoExitStats.auto_exit_enabled}/${autoExitStats.total_positions}` : '-'}
              </span>
            </div>
          </div>
          <div className={styles.hudPanelContent}>
            {realPositions.length === 0 ? (
              <div className={styles.emptyState}>No open positions</div>
            ) : (
              <div className={styles.hudPositionsGrid}>
                {realPositions.map((position) => (
                  <DashboardPositionCard
                    key={position.id}
                    position={position}
                    onQuickSell={handleClosePosition}
                    onViewDetails={(pos) => {
                      setSelectedToken({ mint: pos.token_mint, symbol: pos.token_symbol, venue: pos.venue });
                    }}
                    onViewMetrics={(mint, venue, symbol) => setSelectedMetricsToken({ mint, venue, symbol })}
                    isSelling={closingPosition === position.id}
                  />
                ))}
              </div>
            )}
          </div>
          <div className={styles.hudControlsRow}>
            <button
              className={`${styles.hudControlBtn} ${styles.primary}`}
              onClick={handleToggleMonitor}
            >
              {monitorStatus?.monitoring_active ? 'Pause Monitor' : 'Start Monitor'}
            </button>
            <button
              className={styles.hudControlBtn}
              onClick={handleReconcile}
              disabled={reconciling}
            >
              {reconciling ? 'Syncing...' : 'Sync Wallet'}
            </button>
            <button
              className={`${styles.hudControlBtn} ${styles.danger}`}
              onClick={handleSellAll}
              disabled={sellingAll || realPositions.length === 0}
              title="Dumps all tokens to SOL (USDC excluded)"
            >
              {sellingAll ? 'Selling...' : 'Sell All'}
            </button>
          </div>
        </div>

        {/* Trade Activity Panel */}
        <TradeActivityCard
          liveTrades={liveTrades}
          recentTrades={pnlSummary?.recent_trades}
          onSignalClick={(signal) => {
            if (signal.token_mint) {
              setSelectedToken({ mint: signal.token_mint, venue: 'pump_fun', symbol: signal.token_mint.slice(0, 6) });
            }
          }}
          onTradeClick={(trade, isLive) => {
            const tokenMint = isLive
              ? (trade as LiveTrade).token_mint
              : (trade as RecentTradeInfo).mint;
            const tokenSymbol = isLive
              ? (trade as LiveTrade).token_symbol
              : (trade as RecentTradeInfo).symbol;
            const tokenVenue = isLive
              ? (trade as LiveTrade).venue
              : (trade as RecentTradeInfo).venue || 'pump_fun';
            if (tokenMint) {
              if (isLive) {
                setSelectedToken({ mint: tokenMint, symbol: tokenSymbol, venue: tokenVenue });
              } else {
                const closedTrade = trade as RecentTradeInfo;
                setSelectedToken({
                  mint: tokenMint,
                  symbol: tokenSymbol,
                  venue: tokenVenue,
                  closedTrade: {
                    pnl: closedTrade.pnl,
                    pnl_percent: closedTrade.pnl_percent,
                    entry_price: closedTrade.entry_price,
                    exit_price: closedTrade.exit_price,
                    entry_amount_sol: closedTrade.entry_amount_sol,
                    exit_type: closedTrade.exit_type,
                    time_ago: closedTrade.time_ago,
                  },
                });
              }
            }
          }}
          onViewPosition={(tokenMint) => {
            setSelectedToken({ mint: tokenMint, venue: 'pump_fun', symbol: tokenMint.slice(0, 6) });
          }}
        />

        {/* Scanner & Strategies Column */}
        <div className={styles.hudScannerColumn}>
          <ScannerSignalsPanel
            onSignalClick={(signal) => {
              if (signal.token_mint) {
                setSelectedToken({ mint: signal.token_mint, venue: 'pump_fun', symbol: signal.token_mint.slice(0, 6) });
              }
            }}
            maxSignals={10}
          />
          <BehavioralStrategiesPanel compact={true} />
        </div>
      </div>

      {/* Metrics Panel Overlay */}
      {selectedMetricsToken && (
        <div className={styles.metricsOverlay} onClick={() => setSelectedMetricsToken(null)}>
          <div className={styles.metricsDrawer} onClick={(e) => e.stopPropagation()}>
            <CurveMetricsPanel
              mint={selectedMetricsToken.mint}
              venue={selectedMetricsToken.venue}
              symbol={selectedMetricsToken.symbol}
              onClose={() => setSelectedMetricsToken(null)}
            />
          </div>
        </div>
      )}
    </div>
  );
};

export default HomeTab;
