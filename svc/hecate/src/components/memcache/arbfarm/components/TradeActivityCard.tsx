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
}

const TradeActivityCard: React.FC<TradeActivityCardProps> = ({
  liveTrades = [],
  recentTrades = [],
  onTradeClick,
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

    if (diff < 60000) return 'Just now';
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
    return date.toLocaleDateString();
  };

  const getTradeTypeIcon = (trade: LiveTrade): string => {
    if (trade.venue === 'pump_fun') return '🎰';
    if (trade.venue === 'moonshot') return '🌙';
    return '⚡';
  };

  const filteredRecentTrades = recentTrades.filter(t =>
    t.exit_type !== 'Emergency' && t.exit_type !== 'AlreadySold'
  );
  const hasActivity = liveTrades.length > 0 || filteredRecentTrades.length > 0;

  return (
    <div className={styles.activityCard}>
      <div className={styles.activityCardHeader}>
        <h3 className={styles.activityCardTitle}>
          {liveTrades.length > 0 && (
            <span className={styles.liveIndicator} />
          )}
          Trade Activity
        </h3>
        {liveTrades.length > 0 && (
          <span className={styles.activityBadge}>{liveTrades.length} live</span>
        )}
      </div>

      <div className={styles.activityCardContent}>
        {!hasActivity ? (
          <div className={styles.activityEmptyState}>
            <span className={styles.activityEmptyIcon}>📊</span>
            <span>No trade activity yet</span>
            <span className={styles.activityEmptyHint}>Trades will appear here as they execute</span>
          </div>
        ) : (
          <>
            {liveTrades.length > 0 && (
              <div className={styles.activitySection}>
                <div className={styles.activitySectionHeader}>
                  <span className={styles.activitySectionLabel}>Live Trades</span>
                  <span className={styles.activitySectionCount}>{liveTrades.length}</span>
                </div>
                <div className={styles.activityItems}>
                  {liveTrades.slice(0, 5).map((trade) => (
                    <div
                      key={trade.id}
                      className={`${styles.activityItem} ${styles.liveItem} ${newTradeIds.has(trade.id) ? styles.newItem : ''}`}
                      onClick={() => onTradeClick?.(trade, true)}
                    >
                      <div className={styles.activityItemLeft}>
                        <span className={styles.activityIcon}>{getTradeTypeIcon(trade)}</span>
                        <div className={styles.activityItemInfo}>
                          <span className={styles.activityItemTitle}>
                            {trade.token_symbol || (trade.edge_id ? trade.edge_id.slice(0, 8) : trade.id.slice(0, 8))}
                          </span>
                          <span className={styles.activityItemMeta}>
                            {trade.tx_signature ? (
                              <a
                                href={`https://solscan.io/tx/${trade.tx_signature}`}
                                target="_blank"
                                rel="noopener noreferrer"
                                onClick={(e) => e.stopPropagation()}
                                className={styles.txLink}
                              >
                                {trade.tx_signature.slice(0, 8)}...
                              </a>
                            ) : (
                              'Pending confirmation'
                            )}
                          </span>
                        </div>
                      </div>
                      <div className={styles.activityItemRight}>
                        <span className={styles.activityItemValue}>
                          {(trade.entry_price ?? 0) > 0 ? `${(trade.entry_price ?? 0).toFixed(4)} SOL` : '--'}
                        </span>
                        <span className={styles.activityItemTime}>{formatTime(trade.executed_at)}</span>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {filteredRecentTrades.length > 0 && (
              <div className={styles.activitySection}>
                <div className={styles.activitySectionHeader}>
                  <span className={styles.activitySectionLabel}>Completed Trades</span>
                  <span className={styles.activitySectionCount}>{filteredRecentTrades.length}</span>
                </div>
                <div className={styles.activityItems}>
                  {filteredRecentTrades.slice(0, 5).map((trade, idx) => (
                    <div
                      key={trade.id || `trade-${idx}`}
                      className={`${styles.activityItem} ${styles.completedItem}`}
                      onClick={() => onTradeClick?.(trade, false)}
                    >
                      <div className={styles.activityItemLeft}>
                        <span className={`${styles.activityIcon} ${(trade.pnl ?? 0) >= 0 ? styles.profitIcon : styles.lossIcon}`}>
                          {(trade.pnl ?? 0) >= 0 ? '📈' : '📉'}
                        </span>
                        <div className={styles.activityItemInfo}>
                          <span className={styles.activityItemTitle}>{trade.symbol || 'Unknown'}</span>
                          <span className={styles.activityItemMeta}>
                            {trade.exit_type || 'Closed'} • {trade.time_ago || '--'}
                          </span>
                        </div>
                      </div>
                      <div className={styles.activityItemRight}>
                        <span className={`${styles.activityItemPnl} ${(trade.pnl ?? 0) >= 0 ? styles.profit : styles.loss}`}>
                          {(trade.pnl ?? 0) >= 0 ? '+' : ''}{(trade.pnl ?? 0).toFixed(4)} SOL
                        </span>
                        <span className={`${styles.activityItemPercent} ${(trade.pnl_percent ?? 0) >= 0 ? styles.profit : styles.loss}`}>
                          {(trade.pnl_percent ?? 0) >= 0 ? '+' : ''}{(trade.pnl_percent ?? 0).toFixed(1)}%
                        </span>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
};

export default TradeActivityCard;
