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
  const [walletBalance, setWalletBalance] = useState<WalletBalanceResponse | null>(null);
  const [exposure, setExposure] = useState<PositionExposure | null>(null);
  const [monitorStatus, setMonitorStatus] = useState<MonitorStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [closingPosition, setClosingPosition] = useState<string | null>(null);
  const [selectedDetailItem, setSelectedDetailItem] = useState<LiveTrade | RecentTradeInfo | null>(null);
  const [selectedDetailType, setSelectedDetailType] = useState<DetailItemType>(null);
  const [selectedPosition, setSelectedPosition] = useState<OpenPosition | null>(null);
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
              title="Dumps all tokens to SOL (USDC excluded)"
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
