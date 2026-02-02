import React, { useEffect, useState, useCallback } from 'react';
import styles from '../arbfarm.module.scss';
import { arbFarmService } from '../../../../common/services/arbfarm-service';
import DashboardPositionCard from './DashboardPositionCard';
import ScannerHUD from './ScannerHUD';
import TokenDetailModal from './TokenDetailModal';
import CurveMetricsPanel from './CurveMetricsPanel';
import type {
  PnLSummary,
  OpenPosition,
  WalletBalanceResponse,
  PositionExposure,
  MonitorStatus,
  RecentTradeInfo,
  ScannerStatus,
  SniperStats,
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
  lastSseEvent?: { topic: string; payload: unknown } | null;
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
    hold_duration_mins?: number;
    entry_time?: string;
    exit_time?: string;
    entry_tx_signature?: string;
    exit_tx_signature?: string;
    momentum_at_exit?: number;
  };
}

const HomeTab: React.FC<HomeTabProps> = ({ liveTrades, lastSseEvent }) => {
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

  const [scannerStatus, setScannerStatus] = useState<ScannerStatus | null>(null);
  const [sniperStats, setSniperStats] = useState<SniperStats | null>(null);
  const [executionEnabled, setExecutionEnabled] = useState<boolean | null>(null);
  const [togglingMonitor, setTogglingMonitor] = useState(false);
  const [togglingScanner, setTogglingScanner] = useState(false);
  const [togglingSniper, setTogglingSniper] = useState(false);
  const [togglingExecution, setTogglingExecution] = useState(false);

  const fetchData = useCallback(async () => {
    try {
      const [pnlRes, positionsRes, balanceRes, exposureRes, monitorRes, scannerRes, sniperRes, execRes] = await Promise.all([
        arbFarmService.getPnLSummary(),
        arbFarmService.getPositions(),
        arbFarmService.getWalletBalance(),
        arbFarmService.getPositionExposure(),
        arbFarmService.getMonitorStatus(),
        arbFarmService.getScannerStatus(),
        arbFarmService.getSniperStats(),
        arbFarmService.getExecutionConfig(),
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
    } catch (error) {
      console.error('Failed to fetch home tab data:', error);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    setLoading(true);
    fetchData();
  }, [fetchData]);

  useEffect(() => {
    if (!lastSseEvent) return;
    if (closingPosition || sellingAll || reconciling) return;
    const { topic } = lastSseEvent;
    if (topic?.startsWith('arb.position.') || topic?.startsWith('arb.trade')) {
      fetchData();
    }
  }, [lastSseEvent, fetchData, closingPosition, sellingAll, reconciling]);


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
    if (togglingMonitor) return;
    setTogglingMonitor(true);
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
    } finally {
      setTogglingMonitor(false);
    }
  };

  const handleToggleScanner = async () => {
    if (togglingScanner) return;
    setTogglingScanner(true);
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
    } finally {
      setTogglingScanner(false);
    }
  };

  const handleToggleSniper = async () => {
    if (togglingSniper) return;
    setTogglingSniper(true);
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
    } finally {
      setTogglingSniper(false);
    }
  };

  const handleToggleExecution = async () => {
    if (togglingExecution) return;
    setTogglingExecution(true);
    try {
      const newEnabled = !executionEnabled;
      const res = await arbFarmService.toggleExecution(newEnabled);
      if (res.success && res.data) {
        setExecutionEnabled(res.data.enabled);
      }
    } catch (error) {
      console.error('Failed to toggle execution:', error);
    } finally {
      setTogglingExecution(false);
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


  const totalSniped = sniperStats
    ? sniperStats.positions_sold + sniperStats.positions_waiting + sniperStats.positions_failed
    : 0;
  const sniperWinRate = sniperStats
    ? sniperStats.positions_sold > 0
      ? ((sniperStats.positions_sold / (totalSniped || 1)) * 100)
      : 0
    : 0;

  return (
    <div className={styles.hudDashboard}>
      <div className={styles.hudMainArea}>
        <div className={styles.hudPanel}>
          <div className={styles.hudPanelHeader}>
            <h3 className={styles.hudPanelTitle}>Activity Monitor</h3>
          </div>

          {sniperStats && (totalSniped > 0 || sniperStats.positions_waiting > 0) && (
            <div className={styles.sniperSummary}>
              <div className={styles.sniperStat}>
                <span className={styles.sniperStatLabel}>Watching</span>
                <span className={styles.sniperStatValue}>{sniperStats.positions_waiting}</span>
              </div>
              <div className={styles.sniperStat}>
                <span className={styles.sniperStatLabel}>Sniped</span>
                <span className={styles.sniperStatValue}>{totalSniped}</span>
              </div>
              <div className={styles.sniperStat}>
                <span className={styles.sniperStatLabel}>Sold</span>
                <span className={styles.sniperStatValue}>{sniperStats.positions_sold}</span>
              </div>
              <div className={styles.sniperStat}>
                <span className={styles.sniperStatLabel}>Win Rate</span>
                <span className={styles.sniperStatValue}>{sniperWinRate.toFixed(0)}%</span>
              </div>
              <div className={styles.sniperStat}>
                <span className={styles.sniperStatLabel}>PnL</span>
                <span className={`${styles.sniperStatValue} ${(sniperStats.total_pnl_sol ?? 0) >= 0 ? styles.profit : styles.loss}`}>
                  {(sniperStats.total_pnl_sol ?? 0) >= 0 ? '+' : ''}{(sniperStats.total_pnl_sol ?? 0).toFixed(3)}
                </span>
              </div>
            </div>
          )}

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
        </div>

        <ScannerHUD
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
                    hold_duration_mins: closedTrade.hold_duration_mins,
                    entry_time: closedTrade.entry_time,
                    exit_time: closedTrade.exit_time,
                    entry_tx_signature: closedTrade.entry_tx_signature,
                    exit_tx_signature: closedTrade.exit_tx_signature,
                    momentum_at_exit: closedTrade.momentum_at_exit,
                  },
                });
              }
            }
          }}
          onContenderClick={(mint, symbol) => {
            setSelectedToken({ mint, venue: 'pump_fun', symbol });
          }}
          onViewPosition={(tokenMint) => {
            setSelectedToken({ mint: tokenMint, venue: 'pump_fun', symbol: tokenMint.slice(0, 6) });
          }}
          onToggleMonitor={handleToggleMonitor}
          onSellAll={handleSellAll}
          onToggleExecution={handleToggleExecution}
          onReconcile={handleReconcile}
          monitorActive={monitorStatus?.monitoring_active ?? false}
          executionEnabled={executionEnabled ?? false}
          sellingAll={sellingAll}
          reconciling={reconciling}
          positionCount={realPositions.length}
          togglingMonitor={togglingMonitor}
          togglingExecution={togglingExecution}
        />
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
