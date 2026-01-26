import React, { useState, useEffect, useRef } from 'react';
import type { RecentTradeInfo } from '../../../../types/arbfarm';
import styles from '../arbfarm.module.scss';

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

interface TradeActivityCardProps {
  liveTrades?: LiveTrade[];
  recentTrades?: RecentTradeInfo[];
  onTradeClick?: (trade: LiveTrade | RecentTradeInfo, isLive: boolean) => void;
  onViewPosition?: (tokenMint: string) => void;
}

const TradeActivityCard: React.FC<TradeActivityCardProps> = ({
  liveTrades = [],
  recentTrades = [],
  onTradeClick,
  onViewPosition,
}) => {
  const [newTradeIds, setNewTradeIds] = useState<Set<string>>(new Set());
  const prevTradesRef = useRef<string[]>([]);

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

  const formatTime = (timestamp: string): string => {
    const date = new Date(timestamp);
    const now = new Date();
    const diff = now.getTime() - date.getTime();

    if (diff < 60000) return 'now';
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m`;
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}h`;
    return date.toLocaleDateString();
  };

  const filteredRecentTrades = recentTrades.filter(t =>
    t.exit_type !== 'Emergency' && t.exit_type !== 'AlreadySold'
  );
  const hasActivity = liveTrades.length > 0 || filteredRecentTrades.length > 0;

  return (
    <div className={styles.activityCard}>
      <div className={styles.activityCardHeader}>
        <div className={styles.activityCardTitle}>
          {liveTrades.length > 0 && <span className={styles.liveIndicator} />}
          <span>Trades</span>
        </div>
        <div className={styles.activityHeaderStats}>
          {liveTrades.length > 0 && (
            <span className={styles.activityBadge}>{liveTrades.length} active</span>
          )}
          {filteredRecentTrades.length > 0 && (
            <span className={styles.activityCount}>{filteredRecentTrades.length} closed</span>
          )}
        </div>
      </div>

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
              const momEmoji = mom >= 30 ? '🚀' : mom >= 0 ? '📈' : mom >= -30 ? '📉' : '💀';
              return (
                <div
                  key={trade.id || `trade-${idx}`}
                  className={`${styles.tradeRow} ${styles.closedRow}`}
                  onClick={() => onTradeClick?.(trade, false)}
                >
                  <div className={styles.tradeLeft}>
                    <span className={styles.tradeSymbol}>{trade.symbol || '???'}</span>
                    <span className={styles.tradeMeta}>
                      {trade.exit_type || 'closed'} · {trade.time_ago || '--'}
                      {trade.momentum_at_exit !== undefined && trade.momentum_at_exit !== null && (
                        <span className={styles.tradeMomentum} title={`Momentum at exit: ${mom.toFixed(0)}`}>
                          {' '}· {mom.toFixed(0)}{momEmoji}
                        </span>
                      )}
                    </span>
                  </div>
                  <div className={styles.tradeRight}>
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
    </div>
  );
};

export default TradeActivityCard;
