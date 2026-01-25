import React, { useState } from 'react';
import type { RecentTradeInfo } from '../../../../types/arbfarm';
import styles from '../arbfarm.module.scss';

type ChartProvider = 'dexscreener' | 'birdeye' | 'defined';

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

interface TradeDetailModalProps {
  trade: RecentTradeInfo | LiveTrade;
  isLive: boolean;
  onClose: () => void;
}

const TradeDetailModal: React.FC<TradeDetailModalProps> = ({
  trade,
  isLive,
  onClose,
}) => {
  const [chartProvider, setChartProvider] = useState<ChartProvider>('birdeye');
  const [copied, setCopied] = useState(false);

  const isRecentTrade = (t: RecentTradeInfo | LiveTrade): t is RecentTradeInfo => {
    return 'pnl' in t || 'exit_type' in t;
  };

  const getMint = (): string | null => {
    if (isRecentTrade(trade)) {
      return trade.mint ?? null;
    }
    return trade.token_mint ?? null;
  };

  const getSymbol = (): string => {
    if (isRecentTrade(trade)) {
      return trade.symbol ?? getMint()?.slice(0, 6) ?? 'UNKN';
    }
    return trade.token_symbol ?? getMint()?.slice(0, 6) ?? 'UNKN';
  };

  const getVenue = (): string => {
    if (isRecentTrade(trade)) {
      return trade.venue ?? 'unknown';
    }
    return trade.venue ?? 'unknown';
  };

  const mint = getMint();
  const symbol = getSymbol();
  const venue = getVenue();

  const formatSol = (sol: number | undefined | null): string => {
    if (sol == null || sol === 0) return '—';
    if (sol >= 1000) return `${(sol / 1000).toFixed(2)}K`;
    return sol.toFixed(4);
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

  const getChartUrl = (): string => {
    if (!mint) return '';
    switch (chartProvider) {
      case 'dexscreener':
        return `https://dexscreener.com/solana/${mint}?embed=1&theme=dark&trades=0&info=0`;
      case 'birdeye':
        return `https://birdeye.so/tv-widget/${mint}?chain=solana&viewMode=pair&chartInterval=15&chartType=candle&chartTimezone=America%2FLos_Angeles&chartLeftToolbar=show&theme=dark`;
      case 'defined':
        return `https://www.defined.fi/sol/${mint}?embedded=1&hideTxTable=1&hideChart=0&hideChartEmptyBars=1`;
      default:
        return `https://birdeye.so/tv-widget/${mint}?chain=solana&viewMode=pair&chartInterval=15&chartType=candle&chartTimezone=America%2FLos_Angeles&chartLeftToolbar=show&theme=dark`;
    }
  };

  const getVenueUrl = (): string => {
    if (!mint) return '#';
    if (venue === 'pump_fun') {
      return `https://pump.fun/${mint}`;
    }
    if (venue === 'moonshot') {
      return `https://moonshot.cc/token/${mint}`;
    }
    return `https://solscan.io/token/${mint}`;
  };

  const copyMint = () => {
    if (!mint) return;
    navigator.clipboard.writeText(mint);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const getPnlClass = (pnl: number | undefined | null): string => {
    const value = pnl ?? 0;
    if (value > 0) return styles.pnlPositive;
    if (value < 0) return styles.pnlNegative;
    return styles.pnlNeutral;
  };

  const getTokenImageUrl = (): string => {
    if (venue === 'pump_fun' && mint) {
      return `https://pump.fun/coin/${mint}/image/256x256`;
    }
    return '/nb-logo.svg';
  };

  const renderLiveTradeDetails = (t: LiveTrade) => (
    <>
      <div className={styles.positionMetricsCompact}>
        <div className={styles.metricRow}>
          <span className={styles.metricLabel}>Status</span>
          <span className={`${styles.metricValue} ${styles.statusLive}`}>Live Trade</span>
        </div>
        <div className={styles.metricRow}>
          <span className={styles.metricLabel}>Entry</span>
          <span className={styles.metricValue}>{formatSol(t.entry_price)} SOL</span>
        </div>
        <div className={styles.metricRow}>
          <span className={styles.metricLabel}>Time</span>
          <span className={styles.metricValue}>{new Date(t.executed_at).toLocaleString()}</span>
        </div>
      </div>

      {t.tx_signature && (
        <div className={styles.tradeTransactions}>
          <h4>Transaction</h4>
          <div className={styles.txRow}>
            <span className={styles.txLabel}>Entry TX</span>
            <a
              href={`https://solscan.io/tx/${t.tx_signature}`}
              target="_blank"
              rel="noopener noreferrer"
              className={styles.txLink}
            >
              {t.tx_signature.slice(0, 12)}...
            </a>
          </div>
        </div>
      )}
    </>
  );

  const renderClosedTradeDetails = (t: RecentTradeInfo) => {
    const pnl = t.pnl ?? 0;
    const pnlPercent = t.pnl_percent ?? 0;

    return (
      <>
        <div className={styles.tradePnlDisplay}>
          <div className={`${styles.tradePnlAmount} ${getPnlClass(pnl)}`}>
            {pnl >= 0 ? '+' : ''}{formatSol(Math.abs(pnl))} SOL
          </div>
          <div className={`${styles.tradePnlPercent} ${getPnlClass(pnlPercent)}`}>
            {formatPercent(pnlPercent)}
          </div>
        </div>

        <div className={styles.positionMetricsCompact}>
          <div className={styles.metricRow}>
            <span className={styles.metricLabel}>Exit Type</span>
            <span className={styles.metricValue}>{t.exit_type ?? 'Closed'}</span>
          </div>
          {t.entry_price != null && (
            <div className={styles.metricRow}>
              <span className={styles.metricLabel}>Entry Price</span>
              <span className={styles.metricValue}>{formatPrice(t.entry_price)}</span>
            </div>
          )}
          {t.exit_price != null && (
            <div className={styles.metricRow}>
              <span className={styles.metricLabel}>Exit Price</span>
              <span className={styles.metricValue}>{formatPrice(t.exit_price)}</span>
            </div>
          )}
          {t.entry_amount_sol != null && (
            <div className={styles.metricRow}>
              <span className={styles.metricLabel}>Entry Amount</span>
              <span className={styles.metricValue}>{formatSol(t.entry_amount_sol)} SOL</span>
            </div>
          )}
          {t.hold_duration_mins != null && (
            <div className={styles.metricRow}>
              <span className={styles.metricLabel}>Hold Time</span>
              <span className={styles.metricValue}>
                {t.hold_duration_mins < 60
                  ? `${t.hold_duration_mins}m`
                  : `${Math.floor(t.hold_duration_mins / 60)}h ${t.hold_duration_mins % 60}m`}
              </span>
            </div>
          )}
          <div className={styles.metricRow}>
            <span className={styles.metricLabel}>Closed</span>
            <span className={styles.metricValue}>{t.time_ago ?? '—'}</span>
          </div>
        </div>

        {(t.entry_tx || t.exit_tx) && (
          <div className={styles.tradeTransactions}>
            <h4>Transactions</h4>
            {t.entry_tx && (
              <div className={styles.txRow}>
                <span className={styles.txLabel}>Entry</span>
                <a
                  href={`https://solscan.io/tx/${t.entry_tx}`}
                  target="_blank"
                  rel="noopener noreferrer"
                  className={styles.txLink}
                >
                  {t.entry_tx.slice(0, 12)}...
                </a>
              </div>
            )}
            {t.exit_tx && (
              <div className={styles.txRow}>
                <span className={styles.txLabel}>Exit</span>
                <a
                  href={`https://solscan.io/tx/${t.exit_tx}`}
                  target="_blank"
                  rel="noopener noreferrer"
                  className={styles.txLink}
                >
                  {t.exit_tx.slice(0, 12)}...
                </a>
              </div>
            )}
          </div>
        )}
      </>
    );
  };

  return (
    <div className={styles.positionModal} onClick={onClose}>
      <div className={styles.positionModalContent} onClick={(e) => e.stopPropagation()}>
        <button className={styles.closeButton} onClick={onClose}>×</button>

        <div className={styles.positionModalLayout}>
          {/* Left Panel - Trade Details */}
          <div className={styles.positionDetailsPanel}>
            {/* Header */}
            <div className={styles.positionHeader}>
              <div className={styles.tokenIdentity}>
                <img
                  src={getTokenImageUrl()}
                  alt={symbol}
                  className={styles.tokenImage}
                  onError={(e) => {
                    (e.target as HTMLImageElement).src = '/nb-logo.svg';
                  }}
                />
                <div>
                  <h2>${symbol}</h2>
                  <span className={styles.tokenFullName}>
                    {venue.replace('_', '.')} • {isLive ? 'Live' : 'Closed'}
                  </span>
                </div>
              </div>
              <div className={styles.headerBadges}>
                <span className={`${styles.statusBadge} ${isLive ? styles.statusOpen : styles.statusClosed}`}>
                  {isLive ? 'Live' : 'Closed'}
                </span>
              </div>
            </div>

            {/* Trade Details */}
            {isLive
              ? renderLiveTradeDetails(trade as LiveTrade)
              : renderClosedTradeDetails(trade as RecentTradeInfo)}

            {/* Quick Links */}
            <div className={styles.quickLinksCompact}>
              {mint && (
                <>
                  <button className={styles.linkButton} onClick={copyMint}>
                    {copied ? 'Copied!' : 'Copy Mint'}
                  </button>
                  <a
                    href={getVenueUrl()}
                    target="_blank"
                    rel="noopener noreferrer"
                    className={styles.linkButton}
                  >
                    {venue.replace('_', '.')}
                  </a>
                  <a
                    href={`https://solscan.io/token/${mint}`}
                    target="_blank"
                    rel="noopener noreferrer"
                    className={styles.linkButton}
                  >
                    Solscan
                  </a>
                </>
              )}
            </div>
          </div>

          {/* Right Panel - Chart */}
          <div className={styles.positionChartPanel}>
            <div className={styles.chartProviderTabs}>
              {(['birdeye', 'dexscreener', 'defined'] as ChartProvider[]).map((p) => (
                <button
                  key={p}
                  className={`${styles.chartTab} ${chartProvider === p ? styles.activeTab : ''}`}
                  onClick={() => setChartProvider(p)}
                >
                  {p === 'dexscreener' ? 'DEX' : p === 'birdeye' ? 'Birdeye' : 'Defined'}
                </button>
              ))}
            </div>
            {mint ? (
              <iframe
                src={getChartUrl()}
                className={styles.chartIframe}
                title="Trade Chart"
                frameBorder="0"
                allowFullScreen
              />
            ) : (
              <div className={styles.noChartPlaceholder}>
                <span>No chart available</span>
                <span className={styles.noChartHint}>Mint address not available for this trade</span>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

export default TradeDetailModal;
