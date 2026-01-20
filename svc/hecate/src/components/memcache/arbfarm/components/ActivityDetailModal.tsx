import React from 'react';
import type { RecentTradeInfo } from '../../../../types/arbfarm';
import type { ActivityEvent } from './RecentActivityCard';
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

type DetailItem = LiveTrade | RecentTradeInfo | ActivityEvent;

interface ActivityDetailModalProps {
  item: DetailItem | null;
  itemType: 'live_trade' | 'completed_trade' | 'event' | null;
  onClose: () => void;
}

const ActivityDetailModal: React.FC<ActivityDetailModalProps> = ({
  item,
  itemType,
  onClose,
}) => {
  if (!item || !itemType) return null;

  const formatTimestamp = (ts: string): string => {
    const date = new Date(ts);
    return date.toLocaleString();
  };

  const renderLiveTradeDetails = (trade: LiveTrade) => (
    <>
      <div className={styles.detailSection}>
        <h4>Trade Information</h4>
        <div className={styles.detailGrid}>
          <div className={styles.detailItem}>
            <span className={styles.detailLabel}>Token</span>
            <span className={styles.detailValue}>{trade.token_symbol || 'Unknown'}</span>
          </div>
          <div className={styles.detailItem}>
            <span className={styles.detailLabel}>Venue</span>
            <span className={styles.detailValue}>{trade.venue || 'Unknown'}</span>
          </div>
          <div className={styles.detailItem}>
            <span className={styles.detailLabel}>Entry Price</span>
            <span className={styles.detailValue}>{(trade.entry_price ?? 0).toFixed(6)} SOL</span>
          </div>
          <div className={styles.detailItem}>
            <span className={styles.detailLabel}>Executed At</span>
            <span className={styles.detailValue}>{formatTimestamp(trade.executed_at)}</span>
          </div>
        </div>
      </div>

      <div className={styles.detailSection}>
        <h4>Identifiers</h4>
        <div className={styles.detailGrid}>
          <div className={styles.detailItem}>
            <span className={styles.detailLabel}>Trade ID</span>
            <span className={styles.detailValueMono}>{trade.id}</span>
          </div>
          {trade.edge_id && (
            <div className={styles.detailItem}>
              <span className={styles.detailLabel}>Edge ID</span>
              <span className={styles.detailValueMono}>{trade.edge_id}</span>
            </div>
          )}
          {trade.token_mint && (
            <div className={styles.detailItem}>
              <span className={styles.detailLabel}>Token Mint</span>
              <span className={styles.detailValueMono}>{trade.token_mint}</span>
            </div>
          )}
        </div>
      </div>

      {trade.tx_signature && (
        <div className={styles.detailSection}>
          <h4>Transaction</h4>
          <div className={styles.detailGrid}>
            <div className={styles.detailItem}>
              <span className={styles.detailLabel}>Signature</span>
              <span className={styles.detailValueMono}>{trade.tx_signature}</span>
            </div>
          </div>
          <div className={styles.detailActions}>
            <a
              href={`https://solscan.io/tx/${trade.tx_signature}`}
              target="_blank"
              rel="noopener noreferrer"
              className={styles.detailLink}
            >
              View on Solscan →
            </a>
            <a
              href={`https://solana.fm/tx/${trade.tx_signature}`}
              target="_blank"
              rel="noopener noreferrer"
              className={styles.detailLink}
            >
              View on Solana FM →
            </a>
          </div>
        </div>
      )}
    </>
  );

  const renderCompletedTradeDetails = (trade: RecentTradeInfo) => (
    <>
      <div className={styles.detailSection}>
        <h4>Trade Summary</h4>
        <div className={styles.detailGrid}>
          <div className={styles.detailItem}>
            <span className={styles.detailLabel}>Token</span>
            <span className={styles.detailValue}>{trade.symbol || 'Unknown'}</span>
          </div>
          <div className={styles.detailItem}>
            <span className={styles.detailLabel}>Exit Type</span>
            <span className={styles.detailValue}>{trade.exit_type || 'Closed'}</span>
          </div>
          <div className={styles.detailItem}>
            <span className={styles.detailLabel}>Time</span>
            <span className={styles.detailValue}>{trade.time_ago || '--'}</span>
          </div>
        </div>
      </div>

      <div className={styles.detailSection}>
        <h4>Profit & Loss</h4>
        <div className={styles.pnlDisplay}>
          <div className={`${styles.pnlAmount} ${(trade.pnl ?? 0) >= 0 ? styles.profit : styles.loss}`}>
            {(trade.pnl ?? 0) >= 0 ? '+' : ''}{(trade.pnl ?? 0).toFixed(6)} SOL
          </div>
          <div className={`${styles.pnlPercent} ${(trade.pnl_percent ?? 0) >= 0 ? styles.profit : styles.loss}`}>
            {(trade.pnl_percent ?? 0) >= 0 ? '+' : ''}{(trade.pnl_percent ?? 0).toFixed(2)}%
          </div>
        </div>
      </div>

      <div className={styles.detailSection}>
        <h4>Identifiers</h4>
        <div className={styles.detailGrid}>
          <div className={styles.detailItem}>
            <span className={styles.detailLabel}>Trade ID</span>
            <span className={styles.detailValueMono}>{trade.id}</span>
          </div>
        </div>
      </div>
    </>
  );

  const renderEventDetails = (event: ActivityEvent) => (
    <>
      <div className={styles.detailSection}>
        <h4>Event Information</h4>
        <div className={styles.detailGrid}>
          <div className={styles.detailItem}>
            <span className={styles.detailLabel}>Event Type</span>
            <span className={styles.detailValue}>{event.event_type.replace(/_/g, ' ')}</span>
          </div>
          <div className={styles.detailItem}>
            <span className={styles.detailLabel}>Timestamp</span>
            <span className={styles.detailValue}>{formatTimestamp(event.timestamp)}</span>
          </div>
          {event.source && (
            <>
              <div className={styles.detailItem}>
                <span className={styles.detailLabel}>Source Type</span>
                <span className={styles.detailValue}>{event.source.type}</span>
              </div>
              <div className={styles.detailItem}>
                <span className={styles.detailLabel}>Source ID</span>
                <span className={styles.detailValueMono}>{event.source.id}</span>
              </div>
            </>
          )}
        </div>
      </div>

      {event.payload && Object.keys(event.payload).length > 0 && (
        <div className={styles.detailSection}>
          <h4>Event Payload</h4>
          <div className={styles.payloadDisplay}>
            {Object.entries(event.payload).map(([key, value]) => (
              <div key={key} className={styles.payloadItem}>
                <span className={styles.payloadKey}>{key.replace(/_/g, ' ')}</span>
                <span className={styles.payloadValue}>
                  {typeof value === 'object' ? JSON.stringify(value, null, 2) : String(value)}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      {event.payload?.tx_signature && (
        <div className={styles.detailActions}>
          <a
            href={`https://solscan.io/tx/${event.payload.tx_signature}`}
            target="_blank"
            rel="noopener noreferrer"
            className={styles.detailLink}
          >
            View Transaction on Solscan →
          </a>
        </div>
      )}

      {event.payload?.token_mint && (
        <div className={styles.detailActions}>
          <a
            href={`https://solscan.io/token/${event.payload.token_mint}`}
            target="_blank"
            rel="noopener noreferrer"
            className={styles.detailLink}
          >
            View Token on Solscan →
          </a>
          <a
            href={`https://dexscreener.com/solana/${event.payload.token_mint}`}
            target="_blank"
            rel="noopener noreferrer"
            className={styles.detailLink}
          >
            View on DexScreener →
          </a>
        </div>
      )}
    </>
  );

  const getTitle = (): string => {
    switch (itemType) {
      case 'live_trade':
        return 'Live Trade Details';
      case 'completed_trade':
        return 'Completed Trade Details';
      case 'event':
        return 'Event Details';
      default:
        return 'Details';
    }
  };

  const getIcon = (): string => {
    switch (itemType) {
      case 'live_trade':
        return '⚡';
      case 'completed_trade':
        return '📊';
      case 'event':
        return '📋';
      default:
        return '📄';
    }
  };

  return (
    <div className={styles.detailModalOverlay} onClick={onClose}>
      <div className={styles.detailModal} onClick={(e) => e.stopPropagation()}>
        <div className={styles.detailModalHeader}>
          <div className={styles.detailModalTitle}>
            <span className={styles.detailModalIcon}>{getIcon()}</span>
            <h3>{getTitle()}</h3>
          </div>
          <button className={styles.detailModalClose} onClick={onClose}>
            ×
          </button>
        </div>

        <div className={styles.detailModalContent}>
          {itemType === 'live_trade' && renderLiveTradeDetails(item as LiveTrade)}
          {itemType === 'completed_trade' && renderCompletedTradeDetails(item as RecentTradeInfo)}
          {itemType === 'event' && renderEventDetails(item as ActivityEvent)}
        </div>
      </div>
    </div>
  );
};

export default ActivityDetailModal;
