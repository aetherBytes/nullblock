import React, { useState, useEffect } from 'react';
import type { OpenPosition, ExitConfig, DetailedCurveMetrics } from '../../../../types/arbfarm';
import { arbFarmService } from '../../../../common/services/arbfarm-service';
import styles from '../arbfarm.module.scss';

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
    if (sol == null) return '0.0000';
    if (sol >= 1000000) return `${(sol / 1000000).toFixed(2)}M`;
    if (sol >= 1000) return `${(sol / 1000).toFixed(1)}K`;
    return sol.toFixed(4);
  };

  const formatNumber = (num: number | undefined | null): string => {
    if (num == null) return '0';
    if (num >= 1000000) return `${(num / 1000000).toFixed(2)}M`;
    if (num >= 1000) return `${(num / 1000).toFixed(1)}K`;
    return num.toLocaleString();
  };

  const formatPrice = (price: number | undefined | null): string => {
    if (price == null) return '0.000000';
    if (price >= 1) return price.toFixed(4);
    if (price >= 0.0001) return price.toFixed(6);
    return price.toExponential(4);
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

  const getExplorerUrl = (): string => {
    return `https://solscan.io/token/${position.token_mint ?? ''}`;
  };

  const getJupiterUrl = (): string => {
    const SOL_MINT = 'So11111111111111111111111111111111111111112';
    const tokenMint = position.token_mint ?? '';
    return `https://jup.ag/swap?inputMint=${SOL_MINT}&outputMint=${tokenMint}`;
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

  const entryAmount = position.entry_amount_sol ?? position.entry_amount_base ?? 0;
  const entryPrice = position.entry_price ?? 1;
  const currentPrice = position.current_price ?? entryPrice;
  const currentValue = position.current_value_base ?? (position.current_price
    ? (entryAmount / entryPrice) * position.current_price
    : entryAmount);
  const tokenAmount = position.entry_token_amount ?? position.remaining_token_amount ?? (entryAmount / entryPrice);
  const pnlPercent = position.unrealized_pnl_percent ?? 0;
  const pnlSol = position.unrealized_pnl ?? 0;
  const openedTime = position.opened_at ?? position.entry_time ?? new Date().toISOString();
  const holdTime = Math.floor((Date.now() - new Date(openedTime).getTime()) / 60000);
  const statusBadge = getStatusBadge(position.status);

  return (
    <div className={styles.opportunityModal} onClick={onClose}>
      <div className={styles.opportunityModalContent} onClick={(e) => e.stopPropagation()}>
        <button className={styles.closeButton} onClick={onClose}>×</button>

        <div className={styles.opportunityHeader}>
          <div className={styles.tokenIdentity}>
            <span className={styles.venueIconLarge}>
              {getVenueIcon(position.venue)}
            </span>
            <div>
              <h2>{isSnipe && '🔫 '}${position.token_symbol || (position.token_mint ?? '').slice(0, 6) || 'UNKN'}</h2>
              <span className={styles.tokenFullName}>
                {isSnipe ? '🔫 Snipe • ' : ''}{(position.venue ?? 'unknown').replace('_', '.')} • {holdTime < 60 ? `${holdTime}m` : `${Math.floor(holdTime / 60)}h ${holdTime % 60}m`} hold
              </span>
            </div>
          </div>

          <div className={styles.headerBadges}>
            <span className={`${styles.statusBadge} ${statusBadge.className}`}>
              {statusBadge.text}
            </span>
            <div className={`${styles.recommendationBadge} ${getPnlClass(pnlPercent)}`}>
              {pnlPercent >= 0 ? '+' : ''}{pnlPercent.toFixed(2)}%
            </div>
          </div>
        </div>

        <div className={styles.metricsSection}>
          <h3>Position Details</h3>
          <div className={styles.metricsGrid}>
            <div className={styles.metricItem}>
              <span className={styles.metricLabel}>Entry Amount</span>
              <span className={styles.metricValue}>{formatSol(entryAmount)} SOL</span>
            </div>
            <div className={styles.metricItem}>
              <span className={styles.metricLabel}>Current Value</span>
              <span className={styles.metricValue}>{formatSol(currentValue)} SOL</span>
            </div>
            <div className={styles.metricItem}>
              <span className={styles.metricLabel}>P&L (SOL)</span>
              <span className={`${styles.metricValue} ${getPnlClass(pnlSol)}`}>
                {pnlSol >= 0 ? '+' : ''}{formatSol(pnlSol)} SOL
              </span>
            </div>
            <div className={styles.metricItem}>
              <span className={styles.metricLabel}>P&L (%)</span>
              <span className={`${styles.metricValue} ${getPnlClass(pnlPercent)}`}>
                {pnlPercent >= 0 ? '+' : ''}{pnlPercent.toFixed(2)}%
              </span>
            </div>
            <div className={styles.metricItem}>
              <span className={styles.metricLabel}>Entry Price</span>
              <span className={styles.metricValue}>{formatPrice(entryPrice)}</span>
            </div>
            <div className={styles.metricItem}>
              <span className={styles.metricLabel}>Current Price</span>
              <span className={styles.metricValue}>{formatPrice(currentPrice)}</span>
            </div>
            <div className={styles.metricItem}>
              <span className={styles.metricLabel}>Tokens Held</span>
              <span className={styles.metricValue}>{tokenAmount.toLocaleString()}</span>
            </div>
            <div className={styles.metricItem}>
              <span className={styles.metricLabel}>Opened</span>
              <span className={styles.metricValue}>{new Date(openedTime).toLocaleString()}</span>
            </div>
            {position.high_water_mark && (
              <div className={styles.metricItem}>
                <span className={styles.metricLabel}>High Water Mark</span>
                <span className={styles.metricValue}>{formatPrice(position.high_water_mark)}</span>
              </div>
            )}
            {position.strategy_id && (
              <div className={styles.metricItem}>
                <span className={styles.metricLabel}>Strategy</span>
                <span className={styles.metricValue}>{position.strategy_id.slice(0, 8)}...</span>
              </div>
            )}
          </div>
        </div>

        <div className={styles.metricsSection}>
          <h3>Market Metrics</h3>
          {loadingMetrics ? (
            <div className={styles.loadingState}>Loading metrics...</div>
          ) : metrics ? (
            <div className={styles.metricsGrid}>
              {metrics.market_cap_sol != null && (
                <div className={styles.metricItem}>
                  <span className={styles.metricLabel}>Market Cap</span>
                  <span className={styles.metricValue}>{formatSol(metrics.market_cap_sol)} SOL</span>
                </div>
              )}
              {metrics.volume_24h != null && (
                <div className={styles.metricItem}>
                  <span className={styles.metricLabel}>Volume (24h)</span>
                  <span className={styles.metricValue}>{formatSol(metrics.volume_24h)} SOL</span>
                </div>
              )}
              {metrics.holder_count != null && (
                <div className={styles.metricItem}>
                  <span className={styles.metricLabel}>Holders</span>
                  <span className={styles.metricValue}>{formatNumber(metrics.holder_count)}</span>
                </div>
              )}
              {metrics.volume_1h != null && (
                <div className={styles.metricItem}>
                  <span className={styles.metricLabel}>Volume (1h)</span>
                  <span className={styles.metricValue}>{formatSol(metrics.volume_1h)} SOL</span>
                </div>
              )}
              {metrics.trade_count_1h != null && (
                <div className={styles.metricItem}>
                  <span className={styles.metricLabel}>Trades (1h)</span>
                  <span className={styles.metricValue}>{metrics.trade_count_1h}</span>
                </div>
              )}
              {metrics.unique_buyers_1h != null && (
                <div className={styles.metricItem}>
                  <span className={styles.metricLabel}>Unique Buyers (1h)</span>
                  <span className={styles.metricValue}>{metrics.unique_buyers_1h}</span>
                </div>
              )}
              {metrics.holder_growth_1h != null && (
                <div className={styles.metricItem}>
                  <span className={styles.metricLabel}>Holder Growth (1h)</span>
                  <span className={`${styles.metricValue} ${metrics.holder_growth_1h >= 0 ? styles.positive : styles.negative}`}>
                    {metrics.holder_growth_1h >= 0 ? '+' : ''}{metrics.holder_growth_1h}
                  </span>
                </div>
              )}
              {metrics.top_10_concentration != null && (
                <div className={styles.metricItem}>
                  <span className={styles.metricLabel}>Top 10 Concentration</span>
                  <span className={styles.metricValue}>{metrics.top_10_concentration.toFixed(1)}%</span>
                </div>
              )}
              {metrics.creator_holdings_percent != null && (
                <div className={styles.metricItem}>
                  <span className={styles.metricLabel}>Creator Holdings</span>
                  <span className={styles.metricValue}>{metrics.creator_holdings_percent.toFixed(1)}%</span>
                </div>
              )}
              {metrics.volume_velocity != null && (
                <div className={styles.metricItem}>
                  <span className={styles.metricLabel}>Volume Velocity</span>
                  <span className={`${styles.metricValue} ${metrics.volume_velocity >= 0 ? styles.positive : styles.negative}`}>
                    {metrics.volume_velocity >= 0 ? '+' : ''}{metrics.volume_velocity.toFixed(2)}
                  </span>
                </div>
              )}
              {metrics.price_momentum_1h != null && (
                <div className={styles.metricItem}>
                  <span className={styles.metricLabel}>Price Momentum</span>
                  <span className={`${styles.metricValue} ${metrics.price_momentum_1h >= 0 ? styles.positive : styles.negative}`}>
                    {metrics.price_momentum_1h >= 0 ? '+' : ''}{metrics.price_momentum_1h.toFixed(2)}%
                  </span>
                </div>
              )}
              {metrics.graduation_progress != null && (
                <div className={styles.metricItem}>
                  <span className={styles.metricLabel}>Graduation Progress</span>
                  <span className={styles.metricValue}>{metrics.graduation_progress.toFixed(1)}%</span>
                </div>
              )}
              {metrics.liquidity_depth_sol != null && (
                <div className={styles.metricItem}>
                  <span className={styles.metricLabel}>Liquidity Depth</span>
                  <span className={styles.metricValue}>{formatSol(metrics.liquidity_depth_sol)} SOL</span>
                </div>
              )}
              {metrics.buy_sell_ratio_1h != null && (
                <div className={styles.metricItem}>
                  <span className={styles.metricLabel}>Buy/Sell Ratio (1h)</span>
                  <span className={`${styles.metricValue} ${metrics.buy_sell_ratio_1h >= 1 ? styles.positive : styles.negative}`}>
                    {metrics.buy_sell_ratio_1h.toFixed(2)}
                  </span>
                </div>
              )}
            </div>
          ) : (
            <div className={styles.emptyState}>No market data available</div>
          )}
        </div>

        <div className={styles.exitConfigDisplay}>
          <span className={styles.exitLabel}>Exit Rules:</span>
          {position.exit_config?.stop_loss_percent && (
            <span className={styles.exitBadge}>SL {position.exit_config.stop_loss_percent}%</span>
          )}
          {position.exit_config?.take_profit_percent && (
            <span className={styles.exitBadge}>TP {position.exit_config.take_profit_percent}%</span>
          )}
          {position.exit_config?.trailing_stop_percent && position.exit_config.trailing_stop_percent > 0 && (
            <span className={styles.exitBadge}>Trail {position.exit_config.trailing_stop_percent}%</span>
          )}
          <button
            className={styles.editExitButton}
            onClick={() => setShowEditExit(!showEditExit)}
          >
            Edit
          </button>
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
            <div className={styles.configActions}>
              <button className={styles.saveButton} onClick={handleSaveExit}>
                Save
              </button>
              <button className={styles.cancelButton} onClick={() => setShowEditExit(false)}>
                Cancel
              </button>
            </div>
          </div>
        )}

        <div className={styles.addressSection}>
          <span className={styles.addressLabel}>Contract Address</span>
          <div className={styles.addressRow}>
            <code className={styles.mintAddress}>{position.token_mint ?? ''}</code>
            <button className={styles.copyButton} onClick={copyMint}>
              {copied ? '✓ Copied' : 'Copy'}
            </button>
          </div>
        </div>

        {position.entry_tx_signature && (
          <div className={styles.txSection}>
            <span className={styles.addressLabel}>Entry Transaction</span>
            <div className={styles.addressRow}>
              <a
                href={`https://solscan.io/tx/${position.entry_tx_signature}`}
                target="_blank"
                rel="noopener noreferrer"
                className={styles.flowLink}
              >
                {position.entry_tx_signature.slice(0, 16)}...{position.entry_tx_signature.slice(-8)} →
              </a>
            </div>
          </div>
        )}

        <div className={styles.linksSection}>
          <a href={getVenueUrl()} target="_blank" rel="noopener noreferrer" className={styles.linkButton}>
            {position.venue === 'pump_fun' ? '🎰 pump.fun' : position.venue === 'moonshot' ? '🌙 moonshot' : '📈 Trade'}
          </a>
          <a href={getExplorerUrl()} target="_blank" rel="noopener noreferrer" className={styles.linkButton}>
            🔍 Solscan
          </a>
          <a href={`https://birdeye.so/token/${position.token_mint}?chain=solana`} target="_blank" rel="noopener noreferrer" className={styles.linkButton}>
            🦅 Birdeye
          </a>
          <a href={`https://dexscreener.com/solana/${position.token_mint}`} target="_blank" rel="noopener noreferrer" className={styles.linkButton}>
            📊 DEX Screener
          </a>
          <a href={getJupiterUrl()} target="_blank" rel="noopener noreferrer" className={styles.linkButton}>
            🪐 Jupiter
          </a>
        </div>

        {position.status === 'open' && (
          <div className={styles.actionsSection}>
            <div className={styles.buySection}>
              <h4>Quick Sell</h4>
              <div className={styles.quickBuyRow}>
                <button className={styles.sellPartialButton} onClick={() => onQuickSell(position.id, 25)}>
                  Sell 25%
                </button>
                <button className={styles.sellPartialButton} onClick={() => onQuickSell(position.id, 50)}>
                  Sell 50%
                </button>
                <button className={styles.sellPartialButton} onClick={() => onQuickSell(position.id, 75)}>
                  Sell 75%
                </button>
                <button className={styles.sellAllButton} onClick={() => onQuickSell(position.id, 100)}>
                  Sell All
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default PositionDetailModal;
