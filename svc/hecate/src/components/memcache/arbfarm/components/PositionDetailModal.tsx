import React, { useState, useEffect } from 'react';
import type { OpenPosition, ExitConfig, DetailedCurveMetrics } from '../../../../types/arbfarm';
import { arbFarmService } from '../../../../common/services/arbfarm-service';
import styles from '../arbfarm.module.scss';

type ChartProvider = 'dexscreener' | 'birdeye' | 'defined';

interface PositionDetailModalProps {
  position: OpenPosition;
  onClose: () => void;
  onQuickSell: (positionId: string, percent: number) => void;
  onUpdateExitConfig: (positionId: string, config: Partial<ExitConfig>) => void;
  onSuccess?: (message: string) => void;
  onError?: (message: string) => void;
}

const PositionDetailModal: React.FC<PositionDetailModalProps> = ({
  position,
  onClose,
  onQuickSell,
  onUpdateExitConfig,
}) => {
  const [showEditExit, setShowEditExit] = useState(false);
  const [exitConfig, setExitConfig] = useState<Partial<ExitConfig>>({
    stop_loss_percent: position.exit_config?.stop_loss_percent ?? 10,
    take_profit_percent: position.exit_config?.take_profit_percent ?? 50,
    trailing_stop_percent: position.exit_config?.trailing_stop_percent ?? 0,
  });
  const [copied, setCopied] = useState(false);
  const [metrics, setMetrics] = useState<DetailedCurveMetrics | null>(null);
  const [loadingMetrics, setLoadingMetrics] = useState(true);
  const [chartProvider, setChartProvider] = useState<ChartProvider>('dexscreener');

  useEffect(() => {
    const fetchMetrics = async () => {
      if (!position.token_mint || !position.venue) {
        setLoadingMetrics(false);
        return;
      }
      setLoadingMetrics(true);
      try {
        const res = await arbFarmService.getCurveMetrics(position.token_mint, position.venue);
        if (res.success && res.data) {
          setMetrics(res.data);
        }
      } catch (e) {
        console.error('Failed to fetch metrics:', e);
      } finally {
        setLoadingMetrics(false);
      }
    };
    fetchMetrics();
  }, [position.token_mint, position.venue]);

  const formatSol = (sol: number | undefined | null): string => {
    if (sol == null || sol === 0) return '—';
    if (sol >= 1000000) return `${(sol / 1000000).toFixed(2)}M`;
    if (sol >= 1000) return `${(sol / 1000).toFixed(1)}K`;
    return sol.toFixed(4);
  };

  const formatNumber = (num: number | undefined | null): string => {
    if (num == null || num === 0) return '—';
    if (num >= 1000000) return `${(num / 1000000).toFixed(2)}M`;
    if (num >= 1000) return `${(num / 1000).toFixed(1)}K`;
    return num.toLocaleString();
  };

  const formatPrice = (price: number | undefined | null): string => {
    if (price == null || price === 0) return '—';
    if (price >= 1) return price.toFixed(4);
    if (price >= 0.0001) return price.toFixed(6);
    return price.toExponential(4);
  };

  const formatPercent = (pct: number | undefined | null): string => {
    if (pct == null) return '—';
    return `${pct >= 0 ? '+' : ''}${pct.toFixed(2)}%`;
  };

  const getVenueUrl = (): string => {
    const venue = position.venue ?? 'unknown';
    const mint = position.token_mint ?? '';
    if (venue === 'pump_fun') {
      return `https://pump.fun/${mint}`;
    }
    if (venue === 'moonshot') {
      return `https://moonshot.cc/token/${mint}`;
    }
    return `https://solscan.io/token/${mint}`;
  };

  const getChartUrl = (): string => {
    const mint = position.token_mint ?? '';
    switch (chartProvider) {
      case 'dexscreener':
        return `https://dexscreener.com/solana/${mint}?embed=1&theme=dark&trades=0&info=0`;
      case 'birdeye':
        return `https://birdeye.so/token/${mint}?chain=solana&embed=1`;
      case 'defined':
        return `https://www.defined.fi/sol/${mint}?embedded=1&hideTxTable=1&hideChart=0&hideChartEmptyBars=1`;
      default:
        return `https://dexscreener.com/solana/${mint}?embed=1&theme=dark&trades=0&info=0`;
    }
  };

  const getJupiterUrl = (): string => {
    const SOL_MINT = 'So11111111111111111111111111111111111111112';
    const tokenMint = position.token_mint ?? '';
    return `https://jup.ag/swap?sell=${tokenMint}&buy=${SOL_MINT}`;
  };

  const copyMint = () => {
    navigator.clipboard.writeText(position.token_mint ?? '');
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleSaveExit = () => {
    onUpdateExitConfig(position.id, exitConfig);
    setShowEditExit(false);
  };

  const getPnlClass = (pnl: number | undefined | null): string => {
    const value = pnl ?? 0;
    if (value > 0) return styles.pnlPositive;
    if (value < 0) return styles.pnlNegative;
    return styles.pnlNeutral;
  };

  const getStatusBadge = (status: string | undefined): { text: string; className: string } => {
    switch (status) {
      case 'open':
        return { text: 'Open', className: styles.statusOpen };
      case 'pending_exit':
        return { text: 'Exiting', className: styles.statusPending };
      case 'closed':
        return { text: 'Closed', className: styles.statusClosed };
      default:
        return { text: status ?? 'Unknown', className: '' };
    }
  };

  const getVenueIcon = (venue: string | undefined): string => {
    if (venue === 'pump_fun') return '🎰';
    if (venue === 'moonshot') return '🌙';
    return '📈';
  };

  const isSnipe = position.signal_source === 'graduation_sniper';

  // Use direct values from position - these come from position_manager with real-time updates
  const entryAmount = position.entry_amount_sol ?? position.entry_amount_base ?? 0;
  const entryPrice = position.entry_price ?? 0;
  const currentPrice = position.current_price ?? 0;
  const currentValue = position.current_value_base ?? 0;
  const tokenAmount = position.entry_token_amount ?? position.remaining_token_amount ?? 0;
  const pnlPercent = position.unrealized_pnl_percent ?? 0;
  const pnlSol = position.unrealized_pnl ?? 0;
  const openedTime = position.opened_at ?? position.entry_time ?? new Date().toISOString();
  const holdTime = Math.floor((Date.now() - new Date(openedTime).getTime()) / 60000);
  const statusBadge = getStatusBadge(position.status);
  const symbol = position.token_symbol || (position.token_mint ?? '').slice(0, 6) || 'UNKN';

  return (
    <div className={styles.positionModal} onClick={onClose}>
      <div className={styles.positionModalContent} onClick={(e) => e.stopPropagation()}>
        <button className={styles.closeButton} onClick={onClose}>×</button>

        <div className={styles.positionModalLayout}>
          {/* Left Panel - Position Details */}
          <div className={styles.positionDetailsPanel}>
            {/* Header */}
            <div className={styles.positionHeader}>
              <div className={styles.tokenIdentity}>
                <span className={styles.venueIconLarge}>
                  {getVenueIcon(position.venue)}
                </span>
                <div>
                  <h2>{isSnipe && '🔫 '}${symbol}</h2>
                  <span className={styles.tokenFullName}>
                    {(position.venue ?? 'unknown').replace('_', '.')} • {holdTime < 60 ? `${holdTime}m` : `${Math.floor(holdTime / 60)}h ${holdTime % 60}m`} hold
                  </span>
                </div>
              </div>
              <div className={styles.headerBadges}>
                <span className={`${styles.statusBadge} ${statusBadge.className}`}>
                  {statusBadge.text}
                </span>
                <div className={`${styles.pnlBadgeLarge} ${getPnlClass(pnlPercent)}`}>
                  {formatPercent(pnlPercent)}
                </div>
              </div>
            </div>

            {/* Core Position Metrics */}
            <div className={styles.positionMetricsCompact}>
              <div className={styles.metricRow}>
                <span className={styles.metricLabel}>Entry</span>
                <span className={styles.metricValue}>{formatSol(entryAmount)} SOL @ {formatPrice(entryPrice)}</span>
              </div>
              <div className={styles.metricRow}>
                <span className={styles.metricLabel}>Current</span>
                <span className={styles.metricValue}>{formatSol(currentValue)} SOL @ {formatPrice(currentPrice)}</span>
              </div>
              <div className={styles.metricRow}>
                <span className={styles.metricLabel}>P&L</span>
                <span className={`${styles.metricValue} ${getPnlClass(pnlSol)}`}>
                  {pnlSol >= 0 ? '+' : ''}{formatSol(Math.abs(pnlSol))} SOL ({formatPercent(pnlPercent)})
                </span>
              </div>
              <div className={styles.metricRow}>
                <span className={styles.metricLabel}>Tokens</span>
                <span className={styles.metricValue}>{formatNumber(tokenAmount)}</span>
              </div>
              {position.high_water_mark != null && position.high_water_mark > 0 && (
                <div className={styles.metricRow}>
                  <span className={styles.metricLabel}>Peak Price</span>
                  <span className={styles.metricValue}>{formatPrice(position.high_water_mark)}</span>
                </div>
              )}
            </div>

            {/* Exit Rules */}
            <div className={styles.exitConfigCompact}>
              <div className={styles.exitHeader}>
                <span className={styles.exitLabel}>Exit Rules</span>
                <button
                  className={styles.editExitButtonSmall}
                  onClick={() => setShowEditExit(!showEditExit)}
                >
                  {showEditExit ? 'Cancel' : 'Edit'}
                </button>
              </div>
              <div className={styles.exitBadges}>
                {position.exit_config?.stop_loss_percent != null && (
                  <span className={styles.exitBadgeSL}>SL {position.exit_config.stop_loss_percent}%</span>
                )}
                {position.exit_config?.take_profit_percent != null && (
                  <span className={styles.exitBadgeTP}>TP {position.exit_config.take_profit_percent}%</span>
                )}
                {position.exit_config?.trailing_stop_percent != null && position.exit_config.trailing_stop_percent > 0 && (
                  <span className={styles.exitBadgeTS}>Trail {position.exit_config.trailing_stop_percent}%</span>
                )}
                {position.exit_config?.time_limit_minutes != null && (
                  <span className={styles.exitBadgeTL}>TTL {position.exit_config.time_limit_minutes}m</span>
                )}
              </div>

              {showEditExit && (
                <div className={styles.exitConfigEditor}>
                  <div className={styles.configRow}>
                    <label>Stop Loss %</label>
                    <input
                      type="number"
                      value={exitConfig.stop_loss_percent || ''}
                      placeholder="None"
                      onChange={(e) =>
                        setExitConfig({
                          ...exitConfig,
                          stop_loss_percent: e.target.value ? parseFloat(e.target.value) : undefined,
                        })
                      }
                    />
                  </div>
                  <div className={styles.configRow}>
                    <label>Take Profit %</label>
                    <input
                      type="number"
                      value={exitConfig.take_profit_percent || ''}
                      placeholder="None"
                      onChange={(e) =>
                        setExitConfig({
                          ...exitConfig,
                          take_profit_percent: e.target.value ? parseFloat(e.target.value) : undefined,
                        })
                      }
                    />
                  </div>
                  <div className={styles.configRow}>
                    <label>Trailing Stop %</label>
                    <input
                      type="number"
                      value={exitConfig.trailing_stop_percent || ''}
                      placeholder="None"
                      onChange={(e) =>
                        setExitConfig({
                          ...exitConfig,
                          trailing_stop_percent: e.target.value ? parseFloat(e.target.value) : undefined,
                        })
                      }
                    />
                  </div>
                  <button className={styles.saveButtonSmall} onClick={handleSaveExit}>
                    Save Changes
                  </button>
                </div>
              )}
            </div>

            {/* Market Metrics (from API) */}
            {!loadingMetrics && metrics && (
              <div className={styles.marketMetricsCompact}>
                <h4>Market Data</h4>
                <div className={styles.metricsGridCompact}>
                  {metrics.market_cap_sol != null && metrics.market_cap_sol > 0 && (
                    <div className={styles.metricItem}>
                      <span className={styles.metricLabel}>MCap</span>
                      <span className={styles.metricValue}>{formatSol(metrics.market_cap_sol)}</span>
                    </div>
                  )}
                  {metrics.volume_1h != null && metrics.volume_1h > 0 && (
                    <div className={styles.metricItem}>
                      <span className={styles.metricLabel}>Vol 1h</span>
                      <span className={styles.metricValue}>{formatSol(metrics.volume_1h)}</span>
                    </div>
                  )}
                  {metrics.holder_count != null && metrics.holder_count > 0 && (
                    <div className={styles.metricItem}>
                      <span className={styles.metricLabel}>Holders</span>
                      <span className={styles.metricValue}>{formatNumber(metrics.holder_count)}</span>
                    </div>
                  )}
                  {metrics.graduation_progress != null && metrics.graduation_progress > 0 && (
                    <div className={styles.metricItem}>
                      <span className={styles.metricLabel}>Grad %</span>
                      <span className={styles.metricValue}>{metrics.graduation_progress.toFixed(1)}%</span>
                    </div>
                  )}
                  {metrics.buy_sell_ratio_1h != null && metrics.buy_sell_ratio_1h > 0 && (
                    <div className={styles.metricItem}>
                      <span className={styles.metricLabel}>B/S Ratio</span>
                      <span className={`${styles.metricValue} ${metrics.buy_sell_ratio_1h >= 1 ? styles.positive : styles.negative}`}>
                        {metrics.buy_sell_ratio_1h.toFixed(2)}
                      </span>
                    </div>
                  )}
                  {metrics.top_10_concentration != null && metrics.top_10_concentration > 0 && (
                    <div className={styles.metricItem}>
                      <span className={styles.metricLabel}>Top10</span>
                      <span className={styles.metricValue}>{metrics.top_10_concentration.toFixed(0)}%</span>
                    </div>
                  )}
                </div>
              </div>
            )}

            {/* Contract Address */}
            <div className={styles.addressCompact}>
              <code className={styles.mintAddressSmall} onClick={copyMint} title="Click to copy">
                {position.token_mint ?? ''}
              </code>
              {copied && <span className={styles.copiedBadge}>Copied!</span>}
            </div>

            {/* Quick Links */}
            <div className={styles.linksCompact}>
              <a href={getVenueUrl()} target="_blank" rel="noopener noreferrer" className={styles.linkButtonSmall}>
                {position.venue === 'pump_fun' ? '🎰' : position.venue === 'moonshot' ? '🌙' : '📈'}
              </a>
              <a href={`https://solscan.io/token/${position.token_mint}`} target="_blank" rel="noopener noreferrer" className={styles.linkButtonSmall}>
                🔍
              </a>
              <a href={`https://birdeye.so/token/${position.token_mint}?chain=solana`} target="_blank" rel="noopener noreferrer" className={styles.linkButtonSmall}>
                🦅
              </a>
              <a href={`https://dexscreener.com/solana/${position.token_mint}`} target="_blank" rel="noopener noreferrer" className={styles.linkButtonSmall}>
                📊
              </a>
              <a href={getJupiterUrl()} target="_blank" rel="noopener noreferrer" className={styles.linkButtonSmall}>
                🪐
              </a>
            </div>

            {/* Sell Actions */}
            {position.status === 'open' && (
              <div className={styles.sellActionsCompact}>
                <button className={styles.sellButton25} onClick={() => onQuickSell(position.id, 25)}>
                  25%
                </button>
                <button className={styles.sellButton50} onClick={() => onQuickSell(position.id, 50)}>
                  50%
                </button>
                <button className={styles.sellButton75} onClick={() => onQuickSell(position.id, 75)}>
                  75%
                </button>
                <button className={styles.sellButtonAll} onClick={() => onQuickSell(position.id, 100)}>
                  SELL ALL
                </button>
              </div>
            )}
          </div>

          {/* Right Panel - Chart */}
          <div className={styles.positionChartPanel}>
            <div className={styles.chartProviderTabs}>
              <button
                className={`${styles.chartTab} ${chartProvider === 'dexscreener' ? styles.chartTabActive : ''}`}
                onClick={() => setChartProvider('dexscreener')}
              >
                DEX Screener
              </button>
              <button
                className={`${styles.chartTab} ${chartProvider === 'birdeye' ? styles.chartTabActive : ''}`}
                onClick={() => setChartProvider('birdeye')}
              >
                Birdeye
              </button>
              <button
                className={`${styles.chartTab} ${chartProvider === 'defined' ? styles.chartTabActive : ''}`}
                onClick={() => setChartProvider('defined')}
              >
                Defined
              </button>
            </div>
            <div className={styles.chartContainer}>
              <iframe
                src={getChartUrl()}
                title={`${symbol} Chart`}
                className={styles.chartIframe}
                frameBorder="0"
                allow="clipboard-write"
                sandbox="allow-scripts allow-same-origin allow-popups"
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default PositionDetailModal;
