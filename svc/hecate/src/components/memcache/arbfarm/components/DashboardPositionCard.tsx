import React, { useEffect, useState } from 'react';
import type { OpenPosition } from '../../../../types/arbfarm';
import { arbFarmService } from '../../../../common/services/arbfarm-service';
import styles from '../arbfarm.module.scss';

interface TokenMetadata {
  name: string;
  symbol: string;
  image?: string;
}

// Simple cache for token metadata to avoid repeated fetches
const metadataCache = new Map<string, TokenMetadata>();

interface DashboardPositionCardProps {
  position: OpenPosition;
  onQuickSell: (positionId: string, percent: number) => void;
  onViewDetails: (position: OpenPosition) => void;
  onViewMetrics?: (mint: string, venue: string, symbol: string) => void;
}

const DashboardPositionCard: React.FC<DashboardPositionCardProps> = ({
  position,
  onQuickSell,
  onViewDetails,
  onViewMetrics,
}) => {
  const [metadata, setMetadata] = useState<TokenMetadata | null>(null);

  useEffect(() => {
    const fetchMetadata = async () => {
      if (!position.token_mint) return;

      // Check cache first
      const cached = metadataCache.get(position.token_mint);
      if (cached) {
        setMetadata(cached);
        return;
      }

      try {
        const res = await arbFarmService.lookupTokenMetadata(position.token_mint);
        if (res.success && res.data) {
          const meta = {
            name: res.data.name || 'Unknown',
            symbol: res.data.symbol || position.token_mint.slice(0, 6),
            image: res.data.image_uri,
          };
          metadataCache.set(position.token_mint, meta);
          setMetadata(meta);
        }
      } catch (e) {
        // Silently fail - we'll just show the mint address
      }
    };

    fetchMetadata();
  }, [position.token_mint]);

  const formatSol = (sol: number | undefined | null): string => {
    const value = sol ?? 0;
    if (value >= 1000) return `${(value / 1000).toFixed(1)}K`;
    return value.toFixed(4);
  };

  const getPnlBadge = (): { label: string; class: string } => {
    const pnl = position.unrealized_pnl_percent ?? 0;
    if (pnl >= 100) return { label: `+${pnl.toFixed(0)}%`, class: styles.recBuy };
    if (pnl >= 20) return { label: `+${pnl.toFixed(0)}%`, class: styles.recBuy };
    if (pnl >= 0) return { label: `+${pnl.toFixed(1)}%`, class: styles.recHold };
    if (pnl >= -20) return { label: `${pnl.toFixed(1)}%`, class: styles.recSell };
    return { label: `${pnl.toFixed(0)}%`, class: styles.recSell };
  };

  const getVenueIcon = (venue: string): string => {
    if (venue === 'pump_fun') return '🎰';
    if (venue === 'moonshot') return '🌙';
    return '📈';
  };

  const isSnipe = position.signal_source === 'graduation_sniper';

  const getStatusClass = (status: string): string => {
    switch (status) {
      case 'open': return styles.momentumHigh;
      case 'pending_exit': return styles.momentumMedium;
      case 'closed': return styles.momentumLow;
      default: return '';
    }
  };

  const handleCardClick = (e: React.MouseEvent) => {
    const target = e.target as HTMLElement;
    if (target.tagName === 'BUTTON' || target.closest('button') || target.tagName === 'A' || target.closest('a')) {
      return;
    }
    onViewDetails(position);
  };

  const pnlBadge = getPnlBadge();
  const openedTime = position.opened_at ?? position.entry_time ?? new Date().toISOString();
  const holdTime = Math.floor((Date.now() - new Date(openedTime).getTime()) / 60000);
  const entryAmount = position.entry_amount_sol ?? position.entry_amount_base ?? 0;
  const entryPrice = position.entry_price ?? 1;
  const currentValue = position.current_value_base ?? (position.current_price
    ? (entryAmount / entryPrice) * position.current_price
    : entryAmount);

  return (
    <div
      className={`${styles.dashboardPositionCard} ${styles.clickable}`}
      onClick={handleCardClick}
    >
      <div className={styles.candidateHeader}>
        <div className={styles.candidateToken}>
          {metadata?.image ? (
            <img
              src={metadata.image}
              alt={metadata.symbol}
              className={styles.tokenImage}
              onError={(e) => {
                (e.target as HTMLImageElement).style.display = 'none';
                (e.target as HTMLImageElement).nextElementSibling?.classList.remove(styles.hidden);
              }}
            />
          ) : null}
          <span className={`${styles.venueIcon} ${metadata?.image ? styles.hidden : ''}`}>
            {getVenueIcon(position.venue ?? 'unknown')}
          </span>
          <div className={styles.tokenInfo}>
            <span className={styles.tokenSymbol}>
              {isSnipe && '🔫 '}${metadata?.symbol || position.token_symbol || (position.token_mint ?? '').slice(0, 6) || 'UNKN'}
              <span className={`${styles.recBadge} ${pnlBadge.class}`}>
                {pnlBadge.label}
              </span>
            </span>
            <span className={styles.tokenName}>
              {metadata?.name || (position.venue ?? 'unknown').replace('_', '.')} • {holdTime < 60 ? `${holdTime}m` : `${Math.floor(holdTime / 60)}h ${holdTime % 60}m`}
            </span>
          </div>
        </div>
        <div className={styles.candidateScores}>
          <span className={`${styles.scoreBadge} ${getStatusClass(position.status ?? 'unknown')}`}>
            {position.status === 'open' ? 'OPEN' : (position.status ?? 'UNKNOWN').toUpperCase()}
          </span>
        </div>
      </div>

      <div className={styles.candidateMetrics}>
        <div className={styles.candidateMetric}>
          <span className={styles.metricLabel}>Entry</span>
          <span className={styles.metricValue}>{formatSol(entryAmount)} SOL</span>
        </div>
        <div className={styles.candidateMetric}>
          <span className={styles.metricLabel}>Current</span>
          <span className={styles.metricValue}>{formatSol(currentValue)} SOL</span>
        </div>
        <div className={styles.candidateMetric}>
          <span className={styles.metricLabel}>P&L</span>
          <span className={`${styles.metricValue} ${(position.unrealized_pnl ?? 0) >= 0 ? styles.positive : styles.negative}`}>
            {formatSol(position.unrealized_pnl ?? 0)} SOL
          </span>
        </div>
        <div className={styles.candidateMetric}>
          <span className={styles.metricLabel}>Entry Price</span>
          <span className={styles.metricValue}>{entryPrice.toFixed(6)}</span>
        </div>
      </div>


      <div className={styles.candidateActions}>
        <div className={styles.actionButtonsRow}>
          <button
            className={styles.trackButton}
            onClick={(e) => {
              e.stopPropagation();
              onViewDetails(position);
            }}
          >
            Details
          </button>
          {onViewMetrics && position.token_mint && (
            <button
              className={styles.metricsButton}
              onClick={(e) => {
                e.stopPropagation();
                const symbol = metadata?.symbol || position.token_symbol || position.token_mint!.slice(0, 6);
                const venue = position.venue || 'pump_fun';
                onViewMetrics(position.token_mint!, venue, symbol);
              }}
            >
              Metrics
            </button>
          )}
        </div>
        {position.status === 'open' && position.id && (
          <div className={styles.quickBuyButtons}>
            <button
              className={`${styles.quickBuyButton} ${styles.sellButton}`}
              onClick={(e) => {
                e.stopPropagation();
                onQuickSell(position.id, 25);
              }}
            >
              Sell 25%
            </button>
            <button
              className={`${styles.quickBuyButton} ${styles.sellButton}`}
              onClick={(e) => {
                e.stopPropagation();
                onQuickSell(position.id, 50);
              }}
            >
              Sell 50%
            </button>
            <button
              className={`${styles.quickBuyButton} ${styles.sellButtonAll}`}
              onClick={(e) => {
                e.stopPropagation();
                onQuickSell(position.id, 100);
              }}
            >
              Sell All
            </button>
          </div>
        )}
      </div>

      <div className={styles.candidateFooter}>
        <span className={styles.mintAddress} title={position.token_mint ?? ''}>
          {(position.token_mint ?? '').slice(0, 8) || '???'}...{(position.token_mint ?? '').slice(-6) || '???'}
        </span>
        <span className={styles.candidateVenue}>
          {new Date(openedTime).toLocaleTimeString()}
        </span>
      </div>
    </div>
  );
};

export default DashboardPositionCard;
