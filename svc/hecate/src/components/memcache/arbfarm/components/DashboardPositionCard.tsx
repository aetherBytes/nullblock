import React, { useEffect, useState } from 'react';
import type { OpenPosition } from '../../../../types/arbfarm';
import { arbFarmService } from '../../../../common/services/arbfarm-service';
import styles from '../arbfarm.module.scss';

interface TokenMetadata {
  name: string;
  symbol: string;
  image?: string;
}

const metadataCache = new Map<string, TokenMetadata>();

interface DashboardPositionCardProps {
  position: OpenPosition;
  onQuickSell: (positionId: string, percent: number) => Promise<boolean>;
  onViewDetails: (position: OpenPosition) => void;
  onViewMetrics?: (mint: string, venue: string, symbol: string) => void;
  isSelling?: boolean;
}

const DashboardPositionCard: React.FC<DashboardPositionCardProps> = ({
  position,
  onQuickSell,
  onViewDetails,
  onViewMetrics,
  isSelling = false,
}) => {
  const [metadata, setMetadata] = useState<TokenMetadata | null>(null);
  const [isHovered, setIsHovered] = useState(false);
  const [localSelling, setLocalSelling] = useState<number | null>(null);
  const [autoExitEnabled, setAutoExitEnabled] = useState(position.auto_exit_enabled ?? true);
  const [togglingAutoExit, setTogglingAutoExit] = useState(false);

  useEffect(() => {
    const fetchMetadata = async () => {
      if (!position.token_mint) return;
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
        // Silently fail
      }
    };
    fetchMetadata();
  }, [position.token_mint]);

  const formatSol = (sol: number | undefined | null): string => {
    const value = sol ?? 0;
    if (value >= 1000) return `${(value / 1000).toFixed(1)}K`;
    if (value >= 1) return value.toFixed(3);
    return value.toFixed(4);
  };

  const isSnipe = position.signal_source === 'graduation_sniper';
  const isScanner = position.strategy_id && position.strategy_id !== '00000000-0000-0000-0000-000000000000' && !isSnipe;

  const getStrategyTag = (): { label: string; className: string } => {
    if (isSnipe) return { label: 'SNIPE', className: styles.logTagSnipe };
    if (isScanner) return { label: 'SCAN', className: styles.logTagScan };
    return { label: 'DISC', className: styles.logTagDisc };
  };

  const getStatusIndicator = (status: string): { symbol: string; className: string } => {
    switch (status) {
      case 'open':
        return { symbol: '●', className: styles.logStatusOpen };
      case 'pending_exit':
        return { symbol: '◐', className: styles.logStatusPending };
      case 'closed':
        return { symbol: '○', className: styles.logStatusClosed };
      default:
        return { symbol: '?', className: styles.logStatusUnknown };
    }
  };

  const getVenueTag = (venue: string): string => {
    if (venue === 'pump_fun') return 'pump';
    if (venue === 'moonshot') return 'moon';
    if (venue === 'raydium') return 'ray';
    return venue?.slice(0, 4) || '???';
  };

  const handleCardClick = (e: React.MouseEvent) => {
    const target = e.target as HTMLElement;
    if (target.tagName === 'BUTTON' || target.closest('button') || target.tagName === 'A' || target.closest('a')) {
      return;
    }
    onViewDetails(position);
  };

  const handleToggleAutoExit = async (e: React.MouseEvent) => {
    e.stopPropagation();
    if (!position.id || togglingAutoExit) return;
    setTogglingAutoExit(true);
    try {
      const newEnabled = !autoExitEnabled;
      const res = await arbFarmService.togglePositionAutoExit(position.id, newEnabled);
      if (res.success && res.data) {
        setAutoExitEnabled(res.data.auto_exit_enabled);
      }
    } catch (error) {
      console.error('Failed to toggle auto-exit:', error);
    } finally {
      setTogglingAutoExit(false);
    }
  };

  const openedTime = position.opened_at ?? position.entry_time ?? new Date().toISOString();
  const holdTimeMins = Math.floor((Date.now() - new Date(openedTime).getTime()) / 60000);
  const entryAmount = position.entry_amount_sol ?? position.entry_amount_base ?? 0;
  const entryPrice = position.entry_price ?? 1;
  const currentValue = position.current_value_base ?? (position.current_price
    ? (entryAmount / entryPrice) * position.current_price
    : entryAmount);
  const pnlPercent = position.unrealized_pnl_percent ?? 0;
  const pnlSol = currentValue - entryAmount;

  const formatTimestamp = (iso: string): string => {
    const d = new Date(iso);
    return d.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false });
  };

  const formatHoldTime = (mins: number): string => {
    if (mins < 60) return `${mins}m`;
    const h = Math.floor(mins / 60);
    const m = mins % 60;
    return `${h}h${m > 0 ? m + 'm' : ''}`;
  };

  const formatPrice = (price: number | undefined | null): string => {
    const p = price ?? 0;
    if (p === 0) return '???';
    if (p >= 1) return p.toFixed(4);
    if (p >= 0.0001) return p.toFixed(6);
    if (p >= 0.00000001) return p.toExponential(2);
    return p.toExponential(2);
  };

  const getSourceReason = (): string => {
    if (isSnipe) return 'Graduation detected at ~85% progress';
    if (isScanner) return 'Scanner signal: momentum + volume spike';
    return 'Discovered in wallet reconciliation';
  };

  const strategyTag = getStrategyTag();
  const statusIndicator = getStatusIndicator(position.status ?? 'unknown');
  const symbol = metadata?.symbol || position.token_symbol || (position.token_mint ?? '').slice(0, 6) || 'UNKN';
  const exitConfig = position.exit_config;
  const currentPrice = position.current_price ?? entryPrice;

  return (
    <div
      className={`${styles.logPositionCard} ${isHovered ? styles.logCardHovered : ''}`}
      onClick={handleCardClick}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
    >
      {/* Log Header Line */}
      <div className={styles.logHeader}>
        <span className={styles.logTimestamp}>{formatTimestamp(openedTime)}</span>
        <span className={`${styles.logTag} ${strategyTag.className}`}>{strategyTag.label}</span>
        <span className={`${styles.logStatus} ${statusIndicator.className}`}>{statusIndicator.symbol}</span>
        <span className={styles.logSymbol}>${symbol}</span>
        <span className={`${styles.logPnl} ${pnlPercent >= 0 ? styles.logPnlPositive : styles.logPnlNegative}`}>
          {pnlPercent >= 0 ? '+' : ''}{pnlPercent.toFixed(1)}%
        </span>
        <button
          className={`${styles.autoExitToggle} ${autoExitEnabled ? styles.autoExitOn : styles.autoExitOff}`}
          onClick={handleToggleAutoExit}
          disabled={togglingAutoExit}
          title={autoExitEnabled ? 'Auto-exit enabled - click to switch to manual' : 'Manual mode - click to enable auto-exit'}
        >
          {togglingAutoExit ? '...' : autoExitEnabled ? 'AUTO' : 'MANUAL'}
        </button>
      </div>

      {/* Log Body - Stats Line */}
      <div className={styles.logBody}>
        <div className={styles.logStats}>
          <span className={styles.logStat}>
            <span className={styles.logStatLabel}>in:</span>
            <span className={styles.logStatValue}>{formatSol(entryAmount)} SOL</span>
          </span>
          <span className={styles.logStatSep}>→</span>
          <span className={styles.logStat}>
            <span className={styles.logStatLabel}>now:</span>
            <span className={`${styles.logStatValue} ${pnlSol >= 0 ? styles.logValueUp : styles.logValueDown}`}>
              {formatSol(currentValue)} SOL
            </span>
          </span>
          <span className={styles.logStatDivider}>│</span>
          <span className={styles.logStat}>
            <span className={styles.logStatLabel}>hold:</span>
            <span className={styles.logStatValue}>{formatHoldTime(holdTimeMins)}</span>
          </span>
          <span className={styles.logStatDivider}>│</span>
          <span className={styles.logStat}>
            <span className={styles.logStatLabel}>venue:</span>
            <span className={styles.logStatValue}>{getVenueTag(position.venue ?? 'unknown')}</span>
          </span>
          <span className={styles.logStatDivider}>│</span>
          <span className={styles.logStat}>
            <span className={styles.logStatLabel}>mom:</span>
            <span className={`${styles.logStatValue} ${(position.momentum?.momentum_score ?? 0) >= 0 ? styles.logValueUp : styles.logValueDown}`}>
              {(position.momentum?.momentum_score ?? 0).toFixed(0)}
              {(position.momentum?.momentum_score ?? 0) >= 30 ? '🚀' : (position.momentum?.momentum_score ?? 0) >= 0 ? '📈' : (position.momentum?.momentum_score ?? 0) >= -30 ? '📉' : '💀'}
            </span>
          </span>
        </div>
        <div className={styles.logStats} style={{ marginTop: '0.25rem' }}>
          <span className={styles.logStat}>
            <span className={styles.logStatLabel}>entry@</span>
            <span className={styles.logStatValue}>{formatPrice(entryPrice)}</span>
          </span>
          <span className={styles.logStatSep}>→</span>
          <span className={styles.logStat}>
            <span className={styles.logStatLabel}>now@</span>
            <span className={`${styles.logStatValue} ${currentPrice >= entryPrice ? styles.logValueUp : styles.logValueDown}`}>
              {formatPrice(currentPrice)}
            </span>
          </span>
          {position.high_water_mark && position.high_water_mark > currentPrice && (
            <>
              <span className={styles.logStatDivider}>│</span>
              <span className={styles.logStat}>
                <span className={styles.logStatLabel}>high:</span>
                <span className={styles.logStatValue}>{formatPrice(position.high_water_mark)}</span>
              </span>
            </>
          )}
        </div>
        <div className={styles.logReason}>
          {getSourceReason()}
        </div>
      </div>

      {/* Log Config Line */}
      <div className={styles.logConfig}>
        <span className={styles.logConfigItem}>
          <span className={styles.logConfigLabel}>TP</span>
          <span className={styles.logConfigValue}>{exitConfig?.take_profit_percent ?? '?'}%</span>
        </span>
        <span className={styles.logConfigItem}>
          <span className={styles.logConfigLabel}>SL</span>
          <span className={styles.logConfigValue}>{exitConfig?.stop_loss_percent ?? '?'}%</span>
        </span>
        <span className={styles.logConfigItem}>
          <span className={styles.logConfigLabel}>TTL</span>
          <span className={styles.logConfigValue}>{exitConfig?.time_limit_minutes ?? '?'}m</span>
        </span>
        {exitConfig?.trailing_stop_percent && exitConfig.trailing_stop_percent > 0 && (
          <span className={styles.logConfigItem}>
            <span className={styles.logConfigLabel}>TS</span>
            <span className={styles.logConfigValue}>{exitConfig.trailing_stop_percent}%</span>
          </span>
        )}
        <span className={styles.logMint} title={position.token_mint ?? ''}>
          {(position.token_mint ?? '').slice(0, 6)}...{(position.token_mint ?? '').slice(-4)}
        </span>
      </div>

      {/* Log Actions - Show on hover or always for open positions */}
      {position.status === 'open' && position.id && (
        <div className={styles.logActions}>
          {(isSelling || localSelling !== null) ? (
            <span className={styles.logSelling}>
              <span className={styles.logSpinner}>◠</span>
              Selling {localSelling ?? ''}%...
            </span>
          ) : (
            <>
              <button
                className={styles.logActionBtn}
                onClick={async (e) => {
                  e.stopPropagation();
                  setLocalSelling(25);
                  const success = await onQuickSell(position.id, 25);
                  if (!success) setLocalSelling(null);
                }}
              >
                25%
              </button>
              <button
                className={styles.logActionBtn}
                onClick={async (e) => {
                  e.stopPropagation();
                  setLocalSelling(50);
                  const success = await onQuickSell(position.id, 50);
                  if (!success) setLocalSelling(null);
                }}
              >
                50%
              </button>
              <button
                className={`${styles.logActionBtn} ${styles.logActionDanger}`}
                onClick={async (e) => {
                  e.stopPropagation();
                  setLocalSelling(100);
                  const success = await onQuickSell(position.id, 100);
                  if (!success) setLocalSelling(null);
                }}
              >
                SELL
              </button>
              <button
                className={styles.logActionBtnSecondary}
                onClick={(e) => { e.stopPropagation(); onViewDetails(position); }}
              >
                ···
              </button>
              {onViewMetrics && position.token_mint && (
                <button
                  className={styles.logActionBtnSecondary}
                  onClick={(e) => {
                    e.stopPropagation();
                    onViewMetrics(position.token_mint!, position.venue || 'pump_fun', symbol);
                  }}
                >
                  📊
                </button>
              )}
            </>
          )}
        </div>
      )}
    </div>
  );
};

export default DashboardPositionCard;
