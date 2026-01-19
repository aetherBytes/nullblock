import React, { useEffect, useState, useCallback } from 'react';
import styles from '../arbfarm.module.scss';
import { arbFarmService } from '../../../../common/services/arbfarm-service';
import type {
  PnLSummary,
  OpenPosition,
  PositionStats,
  WalletBalanceResponse,
  PositionExposure,
  MonitorStatus,
} from '../../../../types/arbfarm';

interface HomeTabProps {
  positions: Array<{
    id: string;
    token_mint: string;
    token_symbol?: string;
    entry_sol_amount: number;
    unrealized_pnl?: number;
    status: string;
  }>;
  recentEvents: Array<{
    id?: string;
    event_type: string;
    timestamp: string;
    payload?: Record<string, unknown>;
  }>;
  liveTrades?: Array<{
    id: string;
    edge_id: string;
    tx_signature?: string;
    entry_price: number;
    executed_at: string;
  }>;
}

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
  const [exitConfigForm, setExitConfigForm] = useState({
    stop_loss_percent: 10,
    take_profit_percent: 50,
    trailing_stop_percent: 0,
  });
  const [sellingAll, setSellingAll] = useState(false);
  const [reconciling, setReconciling] = useState(false);
  const [lastPositionRefresh, setLastPositionRefresh] = useState<number>(0);

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
      <div className={styles.homeGrid}>
        {/* Wallet Balance Card */}
        <div className={styles.homeCard}>
          <h3 className={styles.cardTitle}>Wallet Balance</h3>
          <div className={styles.balanceDisplay}>
            <span className={styles.balanceAmount}>
              {(walletBalance?.balance_sol ?? 0).toFixed(4)} SOL
            </span>
          </div>
          <div className={styles.quickActions}>
            <button
              className={`${styles.actionButton} ${styles.danger}`}
              onClick={handleSellAll}
              disabled={sellingAll || realPositions.length === 0}
            >
              {sellingAll ? 'Selling...' : 'Sell All Tokens'}
            </button>
          </div>
        </div>

        {/* Capital Exposure Card */}
        <div className={styles.homeCard}>
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
                <span className={styles.breakdownTitle}>Total Exposure (in SOL):</span>
                <div className={styles.breakdownRow}>
                  <span>Combined</span>
                  <span>{(exposure.total_exposure_sol ?? 0).toFixed(4)} SOL</span>
                </div>
              </div>
            </>
          ) : (
            <div className={styles.emptyState}>No exposure data</div>
          )}
        </div>

        {/* Monitor Status Card */}
        <div className={styles.homeCard}>
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
                <span className={styles.monitorStatLabel}>Active Positions</span>
                <span className={styles.monitorStatValue}>{monitorStatus.active_positions}</span>
              </div>
              <div className={styles.monitorStatItem}>
                <span className={styles.monitorStatLabel}>Pending Exits</span>
                <span className={styles.monitorStatValue}>{monitorStatus.pending_exit_signals}</span>
              </div>
              <div className={styles.monitorStatItem}>
                <span className={styles.monitorStatLabel}>Check Interval</span>
                <span className={styles.monitorStatValue}>{monitorStatus.price_check_interval_secs}s</span>
              </div>
              <div className={styles.monitorStatItem}>
                <span className={styles.monitorStatLabel}>Exit Slippage</span>
                <span className={styles.monitorStatValue}>{monitorStatus.exit_slippage_bps} bps</span>
              </div>
            </div>
          )}
          <div className={styles.quickActions}>
            <button
              className={styles.actionButton}
              onClick={handleReconcile}
              disabled={reconciling}
            >
              {reconciling ? 'Syncing...' : 'Sync Wallet'}
            </button>
          </div>
        </div>

        {/* Performance Card (merged P&L + Position Stats) */}
        <div className={`${styles.homeCard} ${styles.wideCard}`}>
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

        {/* Active Positions Card */}
        <div className={`${styles.homeCard} ${styles.wideCard}`}>
          <h3 className={styles.cardTitle}>Active Positions ({realPositions.length})</h3>
          <div className={styles.positionsList}>
            {realPositions.length === 0 ? (
              <div className={styles.emptyState}>No open positions</div>
            ) : (
              realPositions.map((position) => {
                const pnlPct = position.unrealized_pnl_percent ?? 0;
                const isEditing = editingExitConfig === position.id;
                return (
                  <div key={position.id} className={styles.positionCard}>
                    <div className={styles.positionHeader}>
                      <span className={styles.positionSymbol}>
                        {position.token_symbol || position.token_mint.slice(0, 8)}...
                      </span>
                      <span className={styles.positionVenue}>{position.venue}</span>
                      <span
                        className={`${styles.positionPnl} ${
                          pnlPct >= 0 ? styles.profit : styles.loss
                        }`}
                      >
                        {pnlPct >= 0 ? '+' : ''}
                        {pnlPct.toFixed(1)}%
                      </span>
                    </div>
                    <div className={styles.positionFlow}>
                      <span className={styles.flowItem} title={position.strategy_id}>
                        Strategy: {position.strategy_id?.slice(0, 6) || 'N/A'}
                      </span>
                      <span className={styles.flowArrow}>→</span>
                      <span className={styles.flowItem} title={position.edge_id}>
                        Edge: {position.edge_id?.slice(0, 6) || 'N/A'}
                      </span>
                      {position.entry_tx_signature && (
                        <>
                          <span className={styles.flowArrow}>→</span>
                          <a
                            href={`https://solscan.io/tx/${position.entry_tx_signature}`}
                            target="_blank"
                            rel="noopener noreferrer"
                            className={styles.flowLink}
                            title={position.entry_tx_signature}
                          >
                            TX: {position.entry_tx_signature.slice(0, 6)}...
                          </a>
                        </>
                      )}
                    </div>
                    <div className={styles.positionDetails}>
                      <span>Entry: {(position.entry_amount_sol ?? 0).toFixed(4)} SOL</span>
                      <span>SL: {position.exit_config?.stop_loss_percent ?? 10}%</span>
                      <span>TP: {position.exit_config?.take_profit_percent ?? 50}%</span>
                      {position.exit_config?.trailing_stop_percent && (
                        <span>Trail: {position.exit_config.trailing_stop_percent}%</span>
                      )}
                    </div>
                    {isEditing ? (
                      <div className={styles.exitConfigForm}>
                        <div className={styles.formRow}>
                          <label>Stop Loss %</label>
                          <input
                            type="number"
                            value={exitConfigForm.stop_loss_percent}
                            onChange={(e) => setExitConfigForm(prev => ({
                              ...prev,
                              stop_loss_percent: Number(e.target.value)
                            }))}
                          />
                        </div>
                        <div className={styles.formRow}>
                          <label>Take Profit %</label>
                          <input
                            type="number"
                            value={exitConfigForm.take_profit_percent}
                            onChange={(e) => setExitConfigForm(prev => ({
                              ...prev,
                              take_profit_percent: Number(e.target.value)
                            }))}
                          />
                        </div>
                        <div className={styles.formRow}>
                          <label>Trailing Stop %</label>
                          <input
                            type="number"
                            value={exitConfigForm.trailing_stop_percent}
                            onChange={(e) => setExitConfigForm(prev => ({
                              ...prev,
                              trailing_stop_percent: Number(e.target.value)
                            }))}
                          />
                        </div>
                        <div className={styles.formActions}>
                          <button onClick={() => handleUpdateExitConfig(position.id)}>Save</button>
                          <button onClick={() => setEditingExitConfig(null)}>Cancel</button>
                        </div>
                      </div>
                    ) : (
                      <div className={styles.positionActions}>
                        <button
                          className={styles.editButton}
                          onClick={() => {
                            setExitConfigForm({
                              stop_loss_percent: position.exit_config?.stop_loss_percent ?? 10,
                              take_profit_percent: position.exit_config?.take_profit_percent ?? 50,
                              trailing_stop_percent: position.exit_config?.trailing_stop_percent ?? 0,
                            });
                            setEditingExitConfig(position.id);
                          }}
                        >
                          Edit Exit
                        </button>
                        <button
                          className={styles.partialSellButton}
                          onClick={() => handleClosePosition(position.id, 50)}
                          disabled={closingPosition === position.id}
                        >
                          Sell 50%
                        </button>
                        <button
                          className={styles.sellButton}
                          onClick={() => handleClosePosition(position.id, 100)}
                          disabled={closingPosition === position.id}
                        >
                          {closingPosition === position.id ? 'Closing...' : 'Sell All'}
                        </button>
                      </div>
                    )}
                  </div>
                );
              })
            )}
          </div>
        </div>

        {/* Trade Activity Card (merged Recent Trades + Live Feed) */}
        <div className={styles.homeCard}>
          <h3 className={styles.cardTitle}>
            {liveTrades && liveTrades.length > 0 && (
              <span style={{ display: 'inline-block', width: '8px', height: '8px', borderRadius: '50%', background: '#22c55e', marginRight: '8px', animation: 'pulse 2s infinite' }} />
            )}
            Trade Activity
          </h3>
          <div className={styles.tradeActivityList}>
            {liveTrades && liveTrades.length > 0 && (
              <div className={styles.liveTradesSection}>
                <span className={styles.sectionLabel}>Live</span>
                {liveTrades.slice(0, 3).map((trade, i) => (
                  <div key={trade.id || `live-${i}`} className={styles.activityItem}>
                    <span className={styles.activityTime}>
                      {formatTime(trade.executed_at)}
                    </span>
                    <span className={styles.activityText}>
                      {trade.tx_signature ? (
                        <a
                          href={`https://solscan.io/tx/${trade.tx_signature}`}
                          target="_blank"
                          rel="noopener noreferrer"
                          style={{ color: '#8b5cf6' }}
                        >
                          {trade.tx_signature.slice(0, 12)}...
                        </a>
                      ) : (
                        trade.edge_id.slice(0, 8)
                      )}
                      {' - '}
                      {trade.entry_price > 0 ? `${trade.entry_price.toFixed(4)} SOL` : 'Pending'}
                    </span>
                  </div>
                ))}
              </div>
            )}
            {pnlSummary?.recent_trades && pnlSummary.recent_trades
              .filter(trade => trade.exit_type !== 'Emergency')
              .length > 0 && (
              <div className={styles.recentTradesSection}>
                <span className={styles.sectionLabel}>Recent</span>
                <div className={styles.recentTradesList}>
                  {pnlSummary.recent_trades
                    .filter(trade => trade.exit_type !== 'Emergency')
                    .slice(0, 5)
                    .map((trade, idx) => (
                    <div key={trade.id || `trade-${idx}`} className={styles.recentTradeItem}>
                      <span className={styles.tradeSymbol}>{trade.symbol}</span>
                      <span className={`${styles.tradePnl} ${(trade.pnl ?? 0) >= 0 ? styles.profit : styles.loss}`}>
                        {(trade.pnl ?? 0) >= 0 ? '+' : ''}{(trade.pnl ?? 0).toFixed(4)} ({(trade.pnl_percent ?? 0).toFixed(1)}%)
                      </span>
                      <span className={styles.tradeExit}>{trade.exit_type}</span>
                    </div>
                  ))}
                </div>
              </div>
            )}
            {(!liveTrades || liveTrades.length === 0) &&
             (!pnlSummary?.recent_trades || pnlSummary.recent_trades.filter(t => t.exit_type !== 'Emergency').length === 0) && (
              <div className={styles.emptyState}>No trade activity yet</div>
            )}
          </div>
        </div>

        {/* Recent Activity Card */}
        <div className={styles.homeCard}>
          <h3 className={styles.cardTitle}>Recent Activity</h3>
          <div className={styles.activityList}>
            {recentEvents.length === 0 ? (
              <div className={styles.emptyState}>No recent activity</div>
            ) : (
              recentEvents.slice(0, 6).map((event, i) => (
                <div key={event.id || i} className={styles.activityItem}>
                  <span className={styles.activityTime}>
                    {formatTime(event.timestamp)}
                  </span>
                  <span className={styles.activityText}>
                    {getEventDescription(event)}
                  </span>
                </div>
              ))
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

export default HomeTab;
