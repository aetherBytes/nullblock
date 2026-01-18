import React, { useEffect, useState, useCallback } from 'react';
import styles from '../arbfarm.module.scss';
import { arbFarmService } from '../../../../common/services/arbfarm-service';
import type {
  PnLSummary,
  LearningInsight,
  OpenPosition,
  PositionStats,
  WalletBalanceResponse,
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
}

const HomeTab: React.FC<HomeTabProps> = ({ recentEvents }) => {
  const [pnlSummary, setPnlSummary] = useState<PnLSummary | null>(null);
  const [insights, setInsights] = useState<LearningInsight[]>([]);
  const [realPositions, setRealPositions] = useState<OpenPosition[]>([]);
  const [positionStats, setPositionStats] = useState<PositionStats | null>(null);
  const [walletBalance, setWalletBalance] = useState<WalletBalanceResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [closingPosition, setClosingPosition] = useState<string | null>(null);
  const [editingExitConfig, setEditingExitConfig] = useState<string | null>(null);
  const [exitConfigForm, setExitConfigForm] = useState({
    stop_loss_percent: 10,
    take_profit_percent: 50,
    trailing_stop_percent: 0,
  });
  const [sellingAll, setSellingAll] = useState(false);

  const fetchData = useCallback(async () => {
    try {
      const [pnlRes, insightsRes, positionsRes, balanceRes] = await Promise.all([
        arbFarmService.getPnLSummary(),
        arbFarmService.getLearningInsights(),
        arbFarmService.getPositions(),
        arbFarmService.getWalletBalance(),
      ]);

      if (pnlRes.success && pnlRes.data) {
        setPnlSummary(pnlRes.data);
      }
      if (insightsRes.success && insightsRes.data) {
        setInsights(insightsRes.data.insights || []);
      }
      if (positionsRes.success && positionsRes.data) {
        setRealPositions(positionsRes.data.positions || []);
        setPositionStats(positionsRes.data.stats || null);
      }
      if (balanceRes.success && balanceRes.data) {
        setWalletBalance(balanceRes.data);
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

  const formatPnL = (value: number): string => {
    const formatted = value >= 0 ? `+${value.toFixed(4)}` : value.toFixed(4);
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
        <div className={styles.loadingSpinner}>Loading...</div>
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
              {walletBalance ? walletBalance.balance_sol.toFixed(4) : '0.0000'} SOL
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

        {/* P&L Summary Card */}
        <div className={styles.homeCard}>
          <h3 className={styles.cardTitle}>P&L Summary</h3>
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
          </div>
          <div className={styles.pnlStats}>
            <span>
              Win Rate: {(pnlSummary?.win_rate ?? 0).toFixed(0)}% ({pnlSummary?.wins ?? 0}/
              {pnlSummary?.total_trades ?? 0})
            </span>
            {pnlSummary?.avg_hold_minutes ? (
              <span>Avg Hold: {pnlSummary.avg_hold_minutes.toFixed(0)}min</span>
            ) : null}
          </div>
          {pnlSummary?.best_trade && (
            <div className={styles.tradeHighlight}>
              <span className={styles.profit}>Best: {pnlSummary.best_trade.symbol} +{pnlSummary.best_trade.pnl.toFixed(4)} SOL</span>
            </div>
          )}
          {pnlSummary?.worst_trade && (
            <div className={styles.tradeHighlight}>
              <span className={styles.loss}>Worst: {pnlSummary.worst_trade.symbol} {pnlSummary.worst_trade.pnl.toFixed(4)} SOL</span>
            </div>
          )}
        </div>

        {/* Position Stats Card */}
        <div className={styles.homeCard}>
          <h3 className={styles.cardTitle}>Position Stats</h3>
          {positionStats ? (
            <div className={styles.statsGrid}>
              <div className={styles.statItem}>
                <span className={styles.statLabel}>Active</span>
                <span className={styles.statValue}>{positionStats.active_positions}</span>
              </div>
              <div className={styles.statItem}>
                <span className={styles.statLabel}>Opened</span>
                <span className={styles.statValue}>{positionStats.total_positions_opened}</span>
              </div>
              <div className={styles.statItem}>
                <span className={styles.statLabel}>Closed</span>
                <span className={styles.statValue}>{positionStats.total_positions_closed}</span>
              </div>
              <div className={styles.statItem}>
                <span className={styles.statLabel}>Stop Losses</span>
                <span className={styles.statValue}>{positionStats.stop_losses_triggered}</span>
              </div>
              <div className={styles.statItem}>
                <span className={styles.statLabel}>Take Profits</span>
                <span className={styles.statValue}>{positionStats.take_profits_triggered}</span>
              </div>
              <div className={styles.statItem}>
                <span className={styles.statLabel}>Unrealized</span>
                <span className={`${styles.statValue} ${positionStats.total_unrealized_pnl >= 0 ? styles.profit : styles.loss}`}>
                  {formatPnL(positionStats.total_unrealized_pnl)}
                </span>
              </div>
            </div>
          ) : (
            <div className={styles.emptyState}>No stats available</div>
          )}
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
                    <div className={styles.positionDetails}>
                      <span>Entry: {position.entry_amount_sol.toFixed(4)} SOL</span>
                      <span>SL: {position.exit_config.stop_loss_percent}%</span>
                      <span>TP: {position.exit_config.take_profit_percent}%</span>
                      {position.exit_config.trailing_stop_percent && (
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
                              stop_loss_percent: position.exit_config.stop_loss_percent,
                              take_profit_percent: position.exit_config.take_profit_percent,
                              trailing_stop_percent: position.exit_config.trailing_stop_percent || 0,
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

        {/* Recent Trades Card */}
        {pnlSummary?.recent_trades && pnlSummary.recent_trades.length > 0 && (
          <div className={styles.homeCard}>
            <h3 className={styles.cardTitle}>Recent Trades</h3>
            <div className={styles.recentTradesList}>
              {pnlSummary.recent_trades.map((trade) => (
                <div key={trade.id} className={styles.recentTradeItem}>
                  <span className={styles.tradeSymbol}>{trade.symbol}</span>
                  <span className={`${styles.tradePnl} ${trade.pnl >= 0 ? styles.profit : styles.loss}`}>
                    {trade.pnl >= 0 ? '+' : ''}{trade.pnl.toFixed(4)} ({trade.pnl_percent.toFixed(1)}%)
                  </span>
                  <span className={styles.tradeExit}>{trade.exit_type}</span>
                  <span className={styles.tradeTime}>{trade.time_ago}</span>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Learning Insights Card */}
        <div className={styles.homeCard}>
          <h3 className={styles.cardTitle}>Learning Insights</h3>
          <div className={styles.insightsList}>
            {insights.length === 0 ? (
              <div className={styles.emptyState}>No insights yet</div>
            ) : (
              insights.slice(0, 5).map((insight, i) => (
                <div key={i} className={styles.insightItem}>
                  <span className={styles.insightBullet}>
                    {insight.insight_type === 'risk' ? '!' : insight.insight_type === 'performance' ? '*' : '-'}
                  </span>
                  <span className={styles.insightText}>{insight.text}</span>
                </div>
              ))
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
