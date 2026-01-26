import React, { useState, useEffect, useCallback } from 'react';
import styles from '../arbfarm.module.scss';
import { arbFarmService } from '../../../../common/services/arbfarm-service';

interface TradeRecord {
  id: string;
  edge_id: string;
  strategy_id: string;
  tx_signature?: string;
  profit_lamports?: number;
  gas_cost_lamports?: number;
  executed_at: string;
}

interface TradeHistoryPanelProps {
  onTradeClick?: (trade: TradeRecord) => void;
  maxTrades?: number;
}

const TradeHistoryPanel: React.FC<TradeHistoryPanelProps> = ({
  onTradeClick,
  maxTrades = 10,
}) => {
  const [trades, setTrades] = useState<TradeRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchTrades = useCallback(async () => {
    try {
      const res = await arbFarmService.listTrades(maxTrades);
      if (res.success && res.data) {
        setTrades(res.data as unknown as TradeRecord[]);
        setError(null);
      }
    } catch (e) {
      setError('Failed to load trades');
      console.error('Trade fetch error:', e);
    } finally {
      setLoading(false);
    }
  }, [maxTrades]);

  useEffect(() => {
    fetchTrades();
    const interval = setInterval(fetchTrades, 30000);
    return () => clearInterval(interval);
  }, [fetchTrades]);

  const formatTime = (timestamp: string): string => {
    const date = new Date(timestamp);
    const now = new Date();
    const diff = now.getTime() - date.getTime();

    if (diff < 60000) return 'Just now';
    if (diff < 3600000) {
      const mins = Math.floor(diff / 60000);
      return `${mins}m ago`;
    }
    if (diff < 86400000) {
      const hrs = Math.floor(diff / 3600000);
      return `${hrs}h ago`;
    }
    return date.toLocaleDateString();
  };

  const formatPnL = (lamports: number | undefined | null): { text: string; isPositive: boolean } => {
    if (lamports === undefined || lamports === null) {
      return { text: '???', isPositive: false };
    }
    const sol = lamports / 1e9;
    const isPositive = sol >= 0;
    const sign = isPositive ? '+' : '';
    return {
      text: `${sign}${sol.toFixed(4)}`,
      isPositive,
    };
  };

  if (loading) {
    return (
      <div className={styles.activityCard}>
        <div className={styles.activityCardHeader}>
          <h3 className={styles.activityCardTitle}>Trade History</h3>
        </div>
        <div className={styles.activityCardContent}>
          <div className={styles.activityEmptyState}>
            <span className={styles.logSpinner}>◠</span>
            <span>Loading trades...</span>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.activityCard}>
      <div className={styles.activityCardHeader}>
        <h3 className={styles.activityCardTitle}>Trade History</h3>
        {trades.length > 0 && (
          <span className={styles.activityBadge}>{trades.length} trades</span>
        )}
      </div>

      <div className={styles.activityCardContent}>
        {error ? (
          <div className={styles.activityEmptyState}>
            <span className={styles.activityEmptyIcon}>⚠️</span>
            <span>{error}</span>
          </div>
        ) : trades.length === 0 ? (
          <div className={styles.activityEmptyState}>
            <span className={styles.activityEmptyIcon}>📊</span>
            <span>No trades yet</span>
            <span className={styles.activityEmptyHint}>Completed trades will appear here</span>
          </div>
        ) : (
          <div className={styles.tradeHistoryStream}>
            {trades.map((trade) => {
              const pnl = formatPnL(trade.profit_lamports);
              const isWin = pnl.isPositive;

              return (
                <div
                  key={trade.id}
                  className={styles.tradeHistoryItem}
                  onClick={() => onTradeClick?.(trade)}
                >
                  <div className={styles.tradeHistoryIcon}>
                    <span>{isWin ? '✓' : '✗'}</span>
                  </div>
                  <div className={styles.tradeHistoryContent}>
                    <div className={styles.tradeHistoryHeader}>
                      <span className={styles.tradeHistorySymbol}>
                        Trade #{trade.id.slice(0, 6)}
                      </span>
                      <span className={styles.tradeHistoryTime}>{formatTime(trade.executed_at)}</span>
                    </div>
                    {trade.tx_signature && (
                      <a
                        href={`https://solscan.io/tx/${trade.tx_signature}`}
                        target="_blank"
                        rel="noopener noreferrer"
                        className={styles.tradeHistoryTxLink}
                        onClick={(e) => e.stopPropagation()}
                      >
                        View tx →
                      </a>
                    )}
                  </div>
                  <div className={`${styles.tradeHistoryPnl} ${isWin ? styles.pnlPositive : styles.pnlNegative}`}>
                    {pnl.text} SOL
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
};

export default TradeHistoryPanel;
