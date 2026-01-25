import React, { useEffect, useState, useCallback } from 'react';
import styles from '../arbfarm.module.scss';
import { arbFarmService } from '../../../../common/services/arbfarm-service';
import DashboardPositionCard from './DashboardPositionCard';
import TradeActivityCard from './TradeActivityCard';
import ActivityDetailModal from './ActivityDetailModal';
import PositionDetailModal from './PositionDetailModal';
import CurveMetricsPanel from './CurveMetricsPanel';
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
  liveTrades?: LiveTrade[];
}

type DetailItemType = 'live_trade' | 'completed_trade' | null;

const HomeTab: React.FC<HomeTabProps> = ({ liveTrades }) => {
  const [pnlSummary, setPnlSummary] = useState<PnLSummary | null>(null);
  const [realPositions, setRealPositions] = useState<OpenPosition[]>([]);
  const [positionStats, setPositionStats] = useState<PositionStats | null>(null);
  const [walletBalance, setWalletBalance] = useState<WalletBalanceResponse | null>(null);
  const [exposure, setExposure] = useState<PositionExposure | null>(null);
  const [monitorStatus, setMonitorStatus] = useState<MonitorStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [closingPosition, setClosingPosition] = useState<string | null>(null);
  const [editingExitConfig, setEditingExitConfig] = useState<string | null>(null);
  const [selectedDetailItem, setSelectedDetailItem] = useState<LiveTrade | RecentTradeInfo | null>(null);
  const [selectedDetailType, setSelectedDetailType] = useState<DetailItemType>(null);
  const [selectedPosition, setSelectedPosition] = useState<OpenPosition | null>(null);
  const [exitConfigForm, setExitConfigForm] = useState({
    stop_loss_percent: 10,
    take_profit_percent: 50,
    trailing_stop_percent: 0,
  });
  const [sellingAll, setSellingAll] = useState(false);
  const [reconciling, setReconciling] = useState(false);
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
      {/* Row 1: Control Panel + Trade Activity */}
      <div className={styles.topRow}>
        {/* Control Panel Card */}
        <div className={styles.controlPanelCard}>
          <div className={styles.controlPanelHeader}>
            <div className={styles.userSection}>
              <div className={styles.userAvatar}>
                <img src="/nb-logo.svg" alt="User" className={styles.avatarImg} />
              </div>
              <div className={styles.userInfo}>
                <span className={styles.userName}>Operator</span>
                <span className={styles.userWallet}>
                  {walletBalance?.wallet_address
                    ? `${walletBalance.wallet_address.slice(0, 4)}...${walletBalance.wallet_address.slice(-4)}`
                    : 'Not connected'}
                </span>
              </div>
              <span className={`${styles.userBadge} ${styles.architectBadge}`}>Architect</span>
            </div>
          </div>

          <div className={styles.controlPanelStats}>
            <div className={styles.statItem}>
              <span className={styles.statValue}>{realPositions.length}</span>
              <span className={styles.statLabel}>Active</span>
            </div>
            <div className={styles.statItem}>
              <span className={styles.statValue}>{pnlSummary?.total_trades ?? 0}</span>
              <span className={styles.statLabel}>Trades</span>
            </div>
            <div className={styles.statItem}>
              <span className={`${styles.statValue} ${(pnlSummary?.win_rate ?? 0) >= 50 ? styles.profit : styles.loss}`}>
                {(pnlSummary?.win_rate ?? 0).toFixed(0)}%
              </span>
              <span className={styles.statLabel}>Win Rate</span>
            </div>
            <div className={styles.statItem}>
              <span className={`${styles.statValue} ${(pnlSummary?.total_sol ?? 0) >= 0 ? styles.profit : styles.loss}`}>
                {(pnlSummary?.total_sol ?? 0) >= 0 ? '+' : ''}{(pnlSummary?.total_sol ?? 0).toFixed(3)}
              </span>
              <span className={styles.statLabel}>Total PnL</span>
            </div>
          </div>

          <div className={styles.controlPanelQuickStats}>
            <div className={styles.quickStatRow}>
              <span className={styles.quickStatLabel}>Balance</span>
              <span className={styles.quickStatValue}>{(walletBalance?.balance_sol ?? 0).toFixed(4)} SOL</span>
            </div>
            <div className={styles.quickStatRow}>
              <span className={styles.quickStatLabel}>Exposure</span>
              <span className={styles.quickStatValue}>{(exposure?.total_exposure_sol ?? 0).toFixed(4)} SOL</span>
            </div>
            <div className={styles.quickStatRow}>
              <span className={styles.quickStatLabel}>Monitor</span>
              <span className={`${styles.quickStatValue} ${monitorStatus?.monitoring_active ? styles.running : styles.stopped}`}>
                {monitorStatus?.monitoring_active ? 'Running' : 'Stopped'}
              </span>
            </div>
          </div>

          <div className={styles.controlPanelActions}>
            <button
              className={`${styles.controlButton} ${styles.primary}`}
              onClick={handleToggleMonitor}
            >
              {monitorStatus?.monitoring_active ? 'Pause Monitor' : 'Start Monitor'}
            </button>
            <button
              className={styles.controlButton}
              onClick={handleReconcile}
              disabled={reconciling}
            >
              {reconciling ? 'Syncing...' : 'Sync Wallet'}
            </button>
            <button
              className={`${styles.controlButton} ${styles.danger}`}
              onClick={handleSellAll}
              disabled={sellingAll || realPositions.length === 0}
            >
              {sellingAll ? 'Selling...' : 'Sell All'}
            </button>
          </div>
        </div>

        {/* Trade Activity Card */}
        <TradeActivityCard
          liveTrades={liveTrades}
          recentTrades={pnlSummary?.recent_trades}
          onTradeClick={(trade, isLive) => {
            setSelectedDetailItem(trade);
            setSelectedDetailType(isLive ? 'live_trade' : 'completed_trade');
          }}
          onViewPosition={(tokenMint) => {
            const position = realPositions.find(p => p.token_mint === tokenMint);
            if (position) {
              setSelectedPosition(position);
            } else {
              setSelectedMetricsToken({ mint: tokenMint, venue: 'pump_fun', symbol: tokenMint.slice(0, 6) });
            }
          }}
        />
      </div>

      {/* Row 2: Active Positions */}
      <div className={styles.positionsSection}>
        <div className={styles.sectionHeader}>
          <h3>Open Positions ({realPositions.length})</h3>
        </div>
        {realPositions.length === 0 ? (
          <div className={styles.emptyState}>No open positions</div>
        ) : (
          <div className={styles.openPositionsGrid}>
            {realPositions.map((position) => (
              <DashboardPositionCard
                key={position.id}
                position={position}
                onQuickSell={handleClosePosition}
                onViewDetails={(pos) => setSelectedPosition(pos)}
                onViewMetrics={(mint, venue, symbol) => setSelectedMetricsToken({ mint, venue, symbol })}
                isSelling={closingPosition === position.id}
              />
            ))}
          </div>
        )}
      </div>

      {/* Row 3: Additional Stats */}
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
