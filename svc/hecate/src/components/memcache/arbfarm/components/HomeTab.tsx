import React, { useEffect, useState, useCallback } from 'react';
import styles from '../arbfarm.module.scss';
import { arbFarmService } from '../../../../common/services/arbfarm-service';
import DashboardPositionCard from './DashboardPositionCard';
import TradeActivityCard from './TradeActivityCard';
import RecentActivityCard from './RecentActivityCard';
import ActivityDetailModal from './ActivityDetailModal';
import PositionDetailModal from './PositionDetailModal';
import CurveMetricsPanel from './CurveMetricsPanel';
import type { ActivityEvent } from './RecentActivityCard';
import type {
  PnLSummary,
  OpenPosition,
  PositionStats,
  WalletBalanceResponse,
  PositionExposure,
  MonitorStatus,
  RecentTradeInfo,
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
  positions: Array<{
    id: string;
    token_mint: string;
    token_symbol?: string;
    entry_sol_amount: number;
    unrealized_pnl?: number;
    status: string;
  }>;
  recentEvents: ActivityEvent[];
  liveTrades?: LiveTrade[];
}

type DetailItemType = 'live_trade' | 'completed_trade' | 'event' | null;

const HomeTab: React.FC<HomeTabProps> = ({ recentEvents, liveTrades }) => {
  const [pnlSummary, setPnlSummary] = useState<PnLSummary | null>(null);
  const [realPositions, setRealPositions] = useState<OpenPosition[]>([]);
  const [positionStats, setPositionStats] = useState<PositionStats | null>(null);
  const [walletBalance, setWalletBalance] = useState<WalletBalanceResponse | null>(null);
  const [exposure, setExposure] = useState<PositionExposure | null>(null);
  const [monitorStatus, setMonitorStatus] = useState<MonitorStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [closingPosition, setClosingPosition] = useState<string | null>(null);
  const [editingExitConfig, setEditingExitConfig] = useState<string | null>(null);
  const [selectedDetailItem, setSelectedDetailItem] = useState<LiveTrade | RecentTradeInfo | ActivityEvent | null>(null);
  const [selectedDetailType, setSelectedDetailType] = useState<DetailItemType>(null);
  const [selectedPosition, setSelectedPosition] = useState<OpenPosition | null>(null);
  const [exitConfigForm, setExitConfigForm] = useState({
    stop_loss_percent: 10,
    take_profit_percent: 50,
    trailing_stop_percent: 0,
  });
  const [sellingAll, setSellingAll] = useState(false);
  const [reconciling, setReconciling] = useState(false);
  const [lastPositionRefresh, setLastPositionRefresh] = useState<number>(0);
  const [selectedMetricsToken, setSelectedMetricsToken] = useState<{
    mint: string;
    venue: string;
    symbol: string;
  } | null>(null);

  const fetchData = useCallback(async () => {
    try {
      const [pnlRes, positionsRes, balanceRes, exposureRes, monitorRes] = await Promise.all([
        arbFarmService.getPnLSummary(),
        arbFarmService.getPositions(),
        arbFarmService.getWalletBalance(),
        arbFarmService.getPositionExposure(),
        arbFarmService.getMonitorStatus(),
      ]);

      if (pnlRes.success && pnlRes.data) {
        setPnlSummary(pnlRes.data);
      }
      if (positionsRes.success && positionsRes.data) {
        setRealPositions(positionsRes.data.positions || []);
        setPositionStats(positionsRes.data.stats || null);
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

  useEffect(() => {
    const positionEvents = recentEvents.filter(e =>
      e.event_type.includes('position') ||
      e.event_type.includes('edge_executed') ||
      e.event_type.includes('auto_execution')
    );
    if (positionEvents.length > 0) {
      const latestEventTime = Math.max(...positionEvents.map(e => new Date(e.timestamp).getTime()));
      if (latestEventTime > lastPositionRefresh) {
        setLastPositionRefresh(latestEventTime);
        fetchData();
      }
    }
  }, [recentEvents, fetchData, lastPositionRefresh]);

  const handleClosePosition = async (positionId: string, exitPercent: number = 100) => {
    setClosingPosition(positionId);
    try {
      const res = await arbFarmService.closePosition(positionId, exitPercent);
      if (res.success) {
        await fetchData();
      }
    } catch (error) {
      console.error('Failed to close position:', error);
    } finally {
      setClosingPosition(null);
    }
  };

  const handleUpdateExitConfig = async (positionId: string) => {
    try {
      await arbFarmService.updatePositionExitConfig(positionId, {
        stop_loss_percent: exitConfigForm.stop_loss_percent,
        take_profit_percent: exitConfigForm.take_profit_percent,
        trailing_stop_percent: exitConfigForm.trailing_stop_percent || undefined,
      });
      setEditingExitConfig(null);
      await fetchData();
    } catch (error) {
      console.error('Failed to update exit config:', error);
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

  const formatPnL = (value: number | null | undefined): string => {
    const safeValue = value ?? 0;
    const formatted = safeValue >= 0 ? `+${safeValue.toFixed(4)}` : safeValue.toFixed(4);
    return `${formatted} SOL`;
  };

  const formatTime = (timestamp: string): string => {
    const date = new Date(timestamp);
    return date.toLocaleTimeString('en-US', {
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const getEventDescription = (event: { event_type: string; payload?: Record<string, unknown> }): string => {
    const eventType = event.event_type.replace(/_/g, ' ');
    const mint = (event.payload?.mint as string)?.slice(0, 8) || '';
    const action = event.payload?.action as string || '';

    if (mint) {
      return `${eventType} - ${mint}...`;
    }
    if (action) {
      return `${eventType} - ${action}`;
    }
    return eventType;
  };

  if (loading) {
    return (
      <div className={styles.dashboardView}>
        <div className={styles.loadingContainer}>
          <div className={styles.loadingSpinner}>
            <div className={styles.spinnerRing}></div>
          </div>
          <span className={styles.loadingText}>Loading dashboard data...</span>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.dashboardView}>
      {/* Active Positions - Always at top */}
      <div className={styles.positionsHeader}>
        <div className={styles.sectionHeader}>
          <h3>📊 Active Positions ({realPositions.length})</h3>
          <div className={styles.positionsActions}>
            <button
              className={styles.actionButton}
              onClick={handleReconcile}
              disabled={reconciling}
            >
              {reconciling ? 'Syncing...' : 'Sync Wallet'}
            </button>
            <button
              className={`${styles.actionButton} ${styles.danger}`}
              onClick={handleSellAll}
              disabled={sellingAll || realPositions.length === 0}
            >
              {sellingAll ? 'Selling...' : 'Sell All'}
            </button>
          </div>
        </div>
        {realPositions.length === 0 ? (
          <div className={styles.emptyState}>No open positions</div>
        ) : (
          <div className={styles.openPositionsGrid}>
            {realPositions.map((position) => (
              <DashboardPositionCard
                key={position.id}
                position={position}
                onQuickSell={(positionId, percent) => handleClosePosition(positionId, percent)}
                onViewDetails={(pos) => setSelectedPosition(pos)}
                onViewMetrics={(mint, venue, symbol) => setSelectedMetricsToken({ mint, venue, symbol })}
              />
            ))}
          </div>
        )}
      </div>

      {/* Activity Cards - Wide layout */}
      <div className={styles.activityRow}>
        <TradeActivityCard
          liveTrades={liveTrades}
          recentTrades={pnlSummary?.recent_trades}
          onTradeClick={(trade, isLive) => {
            setSelectedDetailItem(trade);
            setSelectedDetailType(isLive ? 'live_trade' : 'completed_trade');
          }}
        />
        <RecentActivityCard
          events={recentEvents}
          onEventClick={(event) => {
            setSelectedDetailItem(event);
            setSelectedDetailType('event');
          }}
        />
      </div>

      {/* Status Cards Row - Full width centered */}
      <div className={styles.statusCardsRow}>
        {/* Wallet Balance Card */}
        <div className={styles.statusCard}>
          <h3 className={styles.cardTitle}>Wallet Balance</h3>
          <div className={styles.balanceDisplay}>
            <span className={styles.balanceAmount}>
              {(walletBalance?.balance_sol ?? 0).toFixed(4)} SOL
            </span>
          </div>
        </div>

        {/* Capital Exposure Card */}
        <div className={styles.statusCard}>
          <h3 className={styles.cardTitle}>Capital Exposure</h3>
          {exposure ? (
            <>
              <div className={styles.exposureGrid}>
                <div className={styles.exposureItem}>
                  <span className={styles.exposureLabel}>SOL</span>
                  <span className={styles.exposureValue}>{(exposure.sol_exposure ?? 0).toFixed(4)}</span>
                </div>
                <div className={styles.exposureItem}>
                  <span className={styles.exposureLabel}>USDC</span>
                  <span className={styles.exposureValue}>{(exposure.usdc_exposure ?? 0).toFixed(2)}</span>
                </div>
                <div className={styles.exposureItem}>
                  <span className={styles.exposureLabel}>USDT</span>
                  <span className={styles.exposureValue}>{(exposure.usdt_exposure ?? 0).toFixed(2)}</span>
                </div>
              </div>
              <div className={styles.exposureBreakdown}>
                <span className={styles.breakdownTitle}>Total (in SOL):</span>
                <span>{(exposure.total_exposure_sol ?? 0).toFixed(4)} SOL</span>
              </div>
            </>
          ) : (
            <div className={styles.emptyState}>No exposure data</div>
          )}
        </div>

        {/* Monitor Status Card */}
        <div className={styles.statusCard}>
          <h3 className={styles.cardTitle}>Position Monitor</h3>
          <div className={styles.monitorStatus}>
            <span className={`${styles.statusIndicator} ${monitorStatus?.monitoring_active ? styles.running : styles.stopped}`}>
              {monitorStatus?.monitoring_active ? '● Running' : '○ Stopped'}
            </span>
            <button
              className={`${styles.actionButton} ${monitorStatus?.monitoring_active ? styles.warning : styles.primary}`}
              onClick={handleToggleMonitor}
            >
              {monitorStatus?.monitoring_active ? 'Stop' : 'Start'}
            </button>
          </div>
          {monitorStatus && (
            <div className={styles.monitorStats}>
              <div className={styles.monitorStatItem}>
                <span className={styles.monitorStatLabel}>Active</span>
                <span className={styles.monitorStatValue}>{monitorStatus.active_positions}</span>
              </div>
              <div className={styles.monitorStatItem}>
                <span className={styles.monitorStatLabel}>Pending</span>
                <span className={styles.monitorStatValue}>{monitorStatus.pending_exit_signals}</span>
              </div>
              <div className={styles.monitorStatItem}>
                <span className={styles.monitorStatLabel}>Interval</span>
                <span className={styles.monitorStatValue}>{monitorStatus.price_check_interval_secs}s</span>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Performance Card */}
      <div className={styles.homeGrid}>
        <div className={styles.homeCard}>
          <h3 className={styles.cardTitle}>Performance</h3>
          <div className={styles.performanceGrid}>
            <div className={styles.pnlSection}>
              <div className={styles.pnlGrid}>
                <div className={styles.pnlItem}>
                  <span className={styles.pnlLabel}>Today</span>
                  <span
                    className={`${styles.pnlValue} ${
                      (pnlSummary?.today_sol ?? 0) >= 0 ? styles.profit : styles.loss
                    }`}
                  >
                    {formatPnL(pnlSummary?.today_sol ?? 0)}
                  </span>
                </div>
                <div className={styles.pnlItem}>
                  <span className={styles.pnlLabel}>Week</span>
                  <span
                    className={`${styles.pnlValue} ${
                      (pnlSummary?.week_sol ?? 0) >= 0 ? styles.profit : styles.loss
                    }`}
                  >
                    {formatPnL(pnlSummary?.week_sol ?? 0)}
                  </span>
                </div>
                <div className={styles.pnlItem}>
                  <span className={styles.pnlLabel}>Total</span>
                  <span
                    className={`${styles.pnlValue} ${
                      (pnlSummary?.total_sol ?? 0) >= 0 ? styles.profit : styles.loss
                    }`}
                  >
                    {formatPnL(pnlSummary?.total_sol ?? 0)}
                  </span>
                </div>
                {positionStats && (
                  <div className={styles.pnlItem}>
                    <span className={styles.pnlLabel}>Unrealized</span>
                    <span className={`${styles.pnlValue} ${positionStats.total_unrealized_pnl >= 0 ? styles.profit : styles.loss}`}>
                      {formatPnL(positionStats.total_unrealized_pnl)}
                    </span>
                  </div>
                )}
              </div>
              <div className={styles.pnlStats}>
                <span>
                  Win Rate: {(pnlSummary?.win_rate ?? 0).toFixed(0)}% ({pnlSummary?.wins ?? 0}/{pnlSummary?.total_trades ?? 0})
                </span>
                {pnlSummary?.avg_hold_minutes != null && (
                  <span>Avg Hold: {(pnlSummary.avg_hold_minutes ?? 0).toFixed(0)}min</span>
                )}
                {positionStats && (
                  <>
                    <span>SL: {positionStats.stop_losses_triggered}</span>
                    <span>TP: {positionStats.take_profits_triggered}</span>
                  </>
                )}
              </div>
            </div>
            {positionStats && (
              <div className={styles.positionStatsSection}>
                <div className={styles.miniStatsRow}>
                  <span className={styles.miniStat}>
                    <strong>{positionStats.active_positions}</strong> Active
                  </span>
                  <span className={styles.miniStat}>
                    <strong>{positionStats.total_positions_opened}</strong> Opened
                  </span>
                  <span className={styles.miniStat}>
                    <strong>{positionStats.total_positions_closed}</strong> Closed
                  </span>
                </div>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Activity Detail Modal */}
      <ActivityDetailModal
        item={selectedDetailItem}
        itemType={selectedDetailType}
        onClose={() => {
          setSelectedDetailItem(null);
          setSelectedDetailType(null);
        }}
      />

      {/* Position Detail Modal */}
      {selectedPosition && (
        <PositionDetailModal
          position={selectedPosition}
          onClose={() => setSelectedPosition(null)}
          onQuickSell={(positionId, percent) => {
            handleClosePosition(positionId, percent);
            setSelectedPosition(null);
          }}
          onUpdateExitConfig={async (positionId, config) => {
            try {
              await arbFarmService.updatePositionExitConfig(positionId, config);
              await fetchData();
            } catch (error) {
              console.error('Failed to update exit config:', error);
            }
          }}
        />
      )}

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
