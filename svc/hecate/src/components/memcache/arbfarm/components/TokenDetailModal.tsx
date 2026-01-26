import React, { useState, useEffect } from 'react';
import type { OpenPosition, ExitConfig, DetailedCurveMetrics, MarketData } from '../../../../types/arbfarm';
import { arbFarmService } from '../../../../common/services/arbfarm-service';
import styles from '../arbfarm.module.scss';

type ChartProvider = 'dexscreener' | 'birdeye' | 'defined';

interface ClosedTradeData {
  pnl?: number;
  pnl_percent?: number;
  entry_price?: number;
  exit_price?: number;
  entry_amount_sol?: number;
  exit_type?: string;
  time_ago?: string;
}

interface TokenDetailModalProps {
  mint: string;
  symbol?: string;
  venue?: string;
  closedTrade?: ClosedTradeData;
  onClose: () => void;
  onRefresh?: () => void;
  asPanel?: boolean;
}

const TokenDetailModal: React.FC<TokenDetailModalProps> = ({
  mint,
  symbol: initialSymbol,
  venue: initialVenue,
  closedTrade,
  onClose,
  onRefresh,
  asPanel = false,
}) => {
  const [position, setPosition] = useState<OpenPosition | null>(null);
  const [loadingPosition, setLoadingPosition] = useState(true);
  const [showEditExit, setShowEditExit] = useState(false);
  const [exitConfig, setExitConfig] = useState<Partial<ExitConfig>>({
    stop_loss_percent: 15,
    take_profit_percent: 50,
    trailing_stop_percent: 0,
  });
  const [copied, setCopied] = useState(false);
  const [metrics, setMetrics] = useState<DetailedCurveMetrics | null>(null);
  const [marketData, setMarketData] = useState<MarketData | null>(null);
  const [loadingMetrics, setLoadingMetrics] = useState(false);
  const [chartProvider, setChartProvider] = useState<ChartProvider>('birdeye');
  const [imageError, setImageError] = useState(false);
  const [buyAmount, setBuyAmount] = useState('0.05');
  const [sellPercent, setSellPercent] = useState('100');
  const [walletBalance, setWalletBalance] = useState<number>(0);
  const [solPriceUsd, setSolPriceUsd] = useState<number>(0);
  const [isBuying, setIsBuying] = useState(false);
  const [isSelling, setIsSelling] = useState(false);
  const [savingConfig, setSavingConfig] = useState(false);

  const symbol = position?.token_symbol || initialSymbol || mint.slice(0, 6);
  const venue = position?.venue || initialVenue || 'pump_fun';

  useEffect(() => {
    const fetchPosition = async () => {
      setLoadingPosition(true);
      try {
        const res = await arbFarmService.getPositions();
        if (res.success && res.data?.positions) {
          const found = res.data.positions.find(p => p.token_mint === mint && p.status === 'open');
          setPosition(found || null);
          if (found?.exit_config) {
            setExitConfig({
              stop_loss_percent: found.exit_config.stop_loss_percent ?? 15,
              take_profit_percent: found.exit_config.take_profit_percent ?? 50,
              trailing_stop_percent: found.exit_config.trailing_stop_percent ?? 0,
            });
          }
        }
      } catch (e) {
        console.error('Failed to fetch position:', e);
      } finally {
        setLoadingPosition(false);
      }
    };
    fetchPosition();
  }, [mint]);

  useEffect(() => {
    const fetchMetrics = async () => {
      if (!mint) return;
      setLoadingMetrics(true);
      try {
        const res = await arbFarmService.getCurveMetrics(mint, venue);
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
  }, [mint, venue]);

  useEffect(() => {
    const fetchMarketData = async () => {
      if (!mint) return;
      try {
        const res = await arbFarmService.getMarketData(mint);
        if (res.success && res.data) {
          setMarketData(res.data);
        }
      } catch (e) {
        console.error('Failed to fetch market data:', e);
      }
    };
    fetchMarketData();
  }, [mint]);

  useEffect(() => {
    const fetchWalletBalance = async () => {
      try {
        const res = await arbFarmService.getWalletBalance();
        if (res.success && res.data) {
          setWalletBalance(res.data.balance_sol ?? 0);
        }
      } catch (e) {
        console.error('Failed to fetch wallet balance:', e);
      }
    };
    fetchWalletBalance();
  }, []);

  useEffect(() => {
    const fetchSolPrice = async () => {
      try {
        const res = await fetch('https://api.coingecko.com/api/v3/simple/price?ids=solana&vs_currencies=usd');
        const data = await res.json();
        if (data?.solana?.usd) {
          setSolPriceUsd(data.solana.usd);
        }
      } catch (e) {
        console.error('Failed to fetch SOL price:', e);
        setSolPriceUsd(150);
      }
    };
    fetchSolPrice();
  }, []);

  const buyAmountNum = parseFloat(buyAmount) || 0;
  // Use market data, then position price, then closed trade price as fallback for buy calculations
  const currentPriceNative = marketData?.price_native || position?.current_price || closedTrade?.entry_price || 0;
  const currentPriceUsd = marketData?.price_usd || 0;
  const estimatedTokens = currentPriceNative > 0 ? buyAmountNum / currentPriceNative : 0;
  const buyValueUsd = buyAmountNum * solPriceUsd;
  const liquidityUsd = marketData?.liquidity?.usd || 0;
  const liquiditySol = liquidityUsd / solPriceUsd;
  const priceImpact = liquiditySol > 0 ? (buyAmountNum / liquiditySol) * 100 : 0;
  const marketCapUsd = marketData?.market_cap || 0;
  const marketCapSol = marketCapUsd / solPriceUsd;
  const marketCapPercent = marketCapSol > 0 ? (buyAmountNum / marketCapSol) * 100 : 0;

  const formatSol = (sol: number | undefined | null): string => {
    if (sol == null || sol === 0) return '—';
    if (Math.abs(sol) >= 1000) return `${(sol / 1000).toFixed(2)}K`;
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

  const formatNumber = (num: number | undefined | null): string => {
    if (num == null || num === 0) return '—';
    if (num >= 1000000) return `${(num / 1000000).toFixed(2)}M`;
    if (num >= 1000) return `${(num / 1000).toFixed(1)}K`;
    return num.toLocaleString();
  };

  const getChartUrl = (): string => {
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
    if (venue === 'pump_fun') return `https://pump.fun/${mint}`;
    if (venue === 'moonshot') return `https://moonshot.cc/token/${mint}`;
    return `https://solscan.io/token/${mint}`;
  };

  const getJupiterUrl = (): string => {
    const SOL_MINT = 'So11111111111111111111111111111111111111112';
    return `https://jup.ag/swap/${mint}-${SOL_MINT}`;
  };

  const copyMint = () => {
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
    if (position?.token_image_uri) {
      return position.token_image_uri;
    }
    if (venue === 'pump_fun') {
      return `https://pump.fun/coin/${mint}/image/256x256`;
    }
    return '/nb-logo.svg';
  };

  const handleBuy = async () => {
    if (isBuying) return;
    const amount = parseFloat(buyAmount);
    if (isNaN(amount) || amount <= 0) return;

    setIsBuying(true);
    try {
      const res = await arbFarmService.buyCurveToken(mint, amount);
      if (res.success) {
        onRefresh?.();
        const posRes = await arbFarmService.getPositions();
        if (posRes.success && posRes.data?.positions) {
          const found = posRes.data.positions.find(p => p.token_mint === mint && p.status === 'open');
          setPosition(found || null);
        }
      } else {
        console.error('Buy failed:', res.error);
      }
    } catch (e) {
      console.error('Buy error:', e);
    } finally {
      setIsBuying(false);
    }
  };

  const handleSell = async () => {
    if (!position || isSelling) return;
    const percent = parseFloat(sellPercent);
    if (isNaN(percent) || percent <= 0 || percent > 100) return;

    setIsSelling(true);
    try {
      const res = await arbFarmService.closePosition(position.id, percent);
      if (res.success) {
        onRefresh?.();
        if (percent === 100) {
          setPosition(null);
        } else {
          const posRes = await arbFarmService.getPositions();
          if (posRes.success && posRes.data?.positions) {
            const found = posRes.data.positions.find(p => p.token_mint === mint && p.status === 'open');
            setPosition(found || null);
          }
        }
      } else {
        console.error('Sell failed:', res.error);
      }
    } catch (e) {
      console.error('Sell error:', e);
    } finally {
      setIsSelling(false);
    }
  };

  const handleSaveExitConfig = async () => {
    if (!position || savingConfig) return;
    setSavingConfig(true);
    try {
      await arbFarmService.updatePositionExitConfig(position.id, {
        stop_loss_percent: exitConfig.stop_loss_percent,
        take_profit_percent: exitConfig.take_profit_percent,
        trailing_stop_percent: exitConfig.trailing_stop_percent || undefined,
      });
      setShowEditExit(false);
      onRefresh?.();
    } catch (e) {
      console.error('Failed to save exit config:', e);
    } finally {
      setSavingConfig(false);
    }
  };

  const hasPosition = position !== null;
  const pnlSol = position?.unrealized_pnl ?? 0;
  const pnlPercent = position?.unrealized_pnl_percent ?? 0;
  const entryAmount = position?.entry_amount_sol ?? position?.entry_amount_base ?? 0;
  const currentValue = position?.current_value_base ?? 0;
  const holdTime = position
    ? Math.floor((Date.now() - new Date(position.opened_at ?? position.entry_time ?? Date.now()).getTime()) / 60000)
    : 0;

  const renderLeftPanel = () => (
    <div className={styles.positionDetailsPanel}>
      {/* Header */}
      <div className={styles.positionHeader}>
        <div className={styles.tokenIdentity}>
          <img
            src={imageError ? '/nb-logo.svg' : getTokenImageUrl()}
            alt={symbol}
            className={styles.tokenImage}
            onError={() => setImageError(true)}
          />
          <div>
            <h2>${symbol}</h2>
            <span className={styles.tokenFullName}>
              {venue.replace('_', '.')}
            </span>
          </div>
        </div>
        <div className={styles.headerBadges}>
          <span className={`${styles.statusBadge} ${hasPosition ? styles.statusOpen : styles.statusClosed}`}>
            {hasPosition ? 'Open Position' : closedTrade ? 'Closed' : 'No Position'}
          </span>
        </div>
      </div>

      {/* Position Info */}
      {loadingPosition ? (
        <div className={styles.assetMetrics}>
          <div className={styles.metricRow}>
            <span className={styles.metricLabel}>Loading position data...</span>
          </div>
        </div>
      ) : hasPosition ? (
        <>
          <div className={styles.assetPnlDisplay}>
            <div className={`${styles.assetPnlAmount} ${getPnlClass(pnlSol)}`}>
              {pnlSol >= 0 ? '+' : ''}{formatSol(pnlSol)} SOL
            </div>
            <div className={`${styles.assetPnlPercent} ${getPnlClass(pnlPercent)}`}>
              {formatPercent(pnlPercent)}
            </div>
          </div>

          <div className={styles.assetMetrics}>
            <div className={styles.metricRow}>
              <span className={styles.metricLabel}>Entry</span>
              <span className={styles.metricValue}>{formatSol(entryAmount)} SOL @ {formatPrice(position?.entry_price)}</span>
            </div>
            <div className={styles.metricRow}>
              <span className={styles.metricLabel}>Current</span>
              <span className={styles.metricValue}>{formatSol(currentValue)} SOL @ {formatPrice(position?.current_price)}</span>
            </div>
            <div className={styles.metricRow}>
              <span className={styles.metricLabel}>Tokens</span>
              <span className={styles.metricValue}>{formatNumber(position?.entry_token_amount ?? position?.remaining_token_amount)}</span>
            </div>
            <div className={styles.metricRow}>
              <span className={styles.metricLabel}>Hold Time</span>
              <span className={styles.metricValue}>
                {holdTime < 60 ? `${holdTime}m` : `${Math.floor(holdTime / 60)}h ${holdTime % 60}m`}
              </span>
            </div>
            {position?.high_water_mark != null && position.high_water_mark > 0 && (
              <div className={styles.metricRow}>
                <span className={styles.metricLabel}>Peak Price</span>
                <span className={styles.metricValue}>{formatPrice(position.high_water_mark)}</span>
              </div>
            )}
          </div>

          {position?.entry_tx_signature && (
            <div className={styles.assetTransactions}>
              <div className={styles.txRow}>
                <span className={styles.txLabel}>Entry TX</span>
                <a href={`https://solscan.io/tx/${position.entry_tx_signature}`} target="_blank" rel="noopener noreferrer" className={styles.txLink}>
                  {position.entry_tx_signature.slice(0, 10)}...
                </a>
              </div>
            </div>
          )}
        </>
      ) : closedTrade ? (
        <>
          <div className={styles.assetPnlDisplay}>
            <div className={`${styles.assetPnlAmount} ${getPnlClass(closedTrade.pnl)}`}>
              {(closedTrade.pnl ?? 0) >= 0 ? '+' : ''}{formatSol(closedTrade.pnl)} SOL
            </div>
            <div className={`${styles.assetPnlPercent} ${getPnlClass(closedTrade.pnl_percent)}`}>
              {formatPercent(closedTrade.pnl_percent)}
            </div>
          </div>

          <div className={styles.assetMetrics}>
            <div className={styles.metricRow}>
              <span className={styles.metricLabel}>Entry</span>
              <span className={styles.metricValue}>{formatSol(closedTrade.entry_amount_sol)} SOL @ {formatPrice(closedTrade.entry_price)}</span>
            </div>
            <div className={styles.metricRow}>
              <span className={styles.metricLabel}>Exit</span>
              <span className={styles.metricValue}>@ {formatPrice(closedTrade.exit_price)}</span>
            </div>
            <div className={styles.metricRow}>
              <span className={styles.metricLabel}>Exit Type</span>
              <span className={styles.metricValue}>{closedTrade.exit_type || 'Manual'}</span>
            </div>
            <div className={styles.metricRow}>
              <span className={styles.metricLabel}>Closed</span>
              <span className={styles.metricValue}>{closedTrade.time_ago || '—'}</span>
            </div>
          </div>
        </>
      ) : (
        <div className={styles.assetMetrics}>
          <div className={styles.metricRow}>
            <span className={styles.metricLabel}>Status</span>
            <span className={styles.metricValue}>No position data</span>
          </div>
        </div>
      )}

      {/* Market Metrics - Cached Backend Data */}
      {loadingMetrics && !marketData ? (
        <div className={styles.assetMarketData}>
          <h4>Market Data</h4>
          <div className={styles.loadingMetrics}>Loading market data...</div>
        </div>
      ) : marketData ? (
        <div className={styles.assetMarketData}>
          <h4>Market Data <span className={styles.dataSource}>cached {marketData.cache_ttl_secs}s</span></h4>

          {/* Primary Stats Row */}
          <div className={styles.marketStatsRow}>
            <div className={styles.marketStatCard}>
              <span className={styles.marketStatLabel}>Price</span>
              <span className={styles.marketStatValue}>${marketData.price_usd.toFixed(8)}</span>
            </div>
            <div className={styles.marketStatCard}>
              <span className={styles.marketStatLabel}>Market Cap</span>
              <span className={styles.marketStatValue}>${formatNumber(marketData.market_cap)}</span>
            </div>
            <div className={styles.marketStatCard}>
              <span className={styles.marketStatLabel}>Liquidity</span>
              <span className={styles.marketStatValue}>${formatNumber(marketData.liquidity.usd)}</span>
            </div>
          </div>

          {/* Volume Section */}
          <div className={styles.marketMetricsSection}>
            <span className={styles.marketSectionTitle}>Volume (USD)</span>
            <div className={styles.marketMetricsGrid}>
              <div className={styles.marketMetricItem}>
                <span className={styles.marketMetricLabel}>5m</span>
                <span className={styles.marketMetricValue}>${formatNumber(marketData.volume.m5)}</span>
              </div>
              <div className={styles.marketMetricItem}>
                <span className={styles.marketMetricLabel}>1h</span>
                <span className={styles.marketMetricValue}>${formatNumber(marketData.volume.h1)}</span>
              </div>
              <div className={styles.marketMetricItem}>
                <span className={styles.marketMetricLabel}>6h</span>
                <span className={styles.marketMetricValue}>${formatNumber(marketData.volume.h6)}</span>
              </div>
              <div className={styles.marketMetricItem}>
                <span className={styles.marketMetricLabel}>24h</span>
                <span className={styles.marketMetricValue}>${formatNumber(marketData.volume.h24)}</span>
              </div>
            </div>
          </div>

          {/* Price Change Section */}
          <div className={styles.marketMetricsSection}>
            <span className={styles.marketSectionTitle}>Price Change</span>
            <div className={styles.marketMetricsGrid}>
              <div className={styles.marketMetricItem}>
                <span className={styles.marketMetricLabel}>5m</span>
                <span className={`${styles.marketMetricValue} ${marketData.price_change.m5 >= 0 ? styles.positive : styles.negative}`}>
                  {marketData.price_change.m5 >= 0 ? '+' : ''}{marketData.price_change.m5.toFixed(2)}%
                </span>
              </div>
              <div className={styles.marketMetricItem}>
                <span className={styles.marketMetricLabel}>1h</span>
                <span className={`${styles.marketMetricValue} ${marketData.price_change.h1 >= 0 ? styles.positive : styles.negative}`}>
                  {marketData.price_change.h1 >= 0 ? '+' : ''}{marketData.price_change.h1.toFixed(2)}%
                </span>
              </div>
              <div className={styles.marketMetricItem}>
                <span className={styles.marketMetricLabel}>6h</span>
                <span className={`${styles.marketMetricValue} ${marketData.price_change.h6 >= 0 ? styles.positive : styles.negative}`}>
                  {marketData.price_change.h6 >= 0 ? '+' : ''}{marketData.price_change.h6.toFixed(2)}%
                </span>
              </div>
              <div className={styles.marketMetricItem}>
                <span className={styles.marketMetricLabel}>24h</span>
                <span className={`${styles.marketMetricValue} ${marketData.price_change.h24 >= 0 ? styles.positive : styles.negative}`}>
                  {marketData.price_change.h24 >= 0 ? '+' : ''}{marketData.price_change.h24.toFixed(2)}%
                </span>
              </div>
            </div>
          </div>

          {/* Transactions Section */}
          <div className={styles.marketMetricsSection}>
            <span className={styles.marketSectionTitle}>Transactions</span>
            <div className={styles.marketMetricsGrid}>
              <div className={styles.marketMetricItem}>
                <span className={styles.marketMetricLabel}>5m Buys</span>
                <span className={`${styles.marketMetricValue} ${styles.positive}`}>{marketData.txns.m5.buys}</span>
              </div>
              <div className={styles.marketMetricItem}>
                <span className={styles.marketMetricLabel}>5m Sells</span>
                <span className={`${styles.marketMetricValue} ${styles.negative}`}>{marketData.txns.m5.sells}</span>
              </div>
              <div className={styles.marketMetricItem}>
                <span className={styles.marketMetricLabel}>1h Buys</span>
                <span className={`${styles.marketMetricValue} ${styles.positive}`}>{marketData.txns.h1.buys}</span>
              </div>
              <div className={styles.marketMetricItem}>
                <span className={styles.marketMetricLabel}>1h Sells</span>
                <span className={`${styles.marketMetricValue} ${styles.negative}`}>{marketData.txns.h1.sells}</span>
              </div>
              <div className={styles.marketMetricItem}>
                <span className={styles.marketMetricLabel}>24h Buys</span>
                <span className={`${styles.marketMetricValue} ${styles.positive}`}>{formatNumber(marketData.txns.h24.buys)}</span>
              </div>
              <div className={styles.marketMetricItem}>
                <span className={styles.marketMetricLabel}>24h Sells</span>
                <span className={`${styles.marketMetricValue} ${styles.negative}`}>{formatNumber(marketData.txns.h24.sells)}</span>
              </div>
            </div>
          </div>

          {/* Buy/Sell Ratios */}
          <div className={styles.marketMetricsSection}>
            <span className={styles.marketSectionTitle}>Buy/Sell Ratio</span>
            <div className={styles.marketMetricsGrid}>
              <div className={styles.marketMetricItem}>
                <span className={styles.marketMetricLabel}>5m</span>
                <span className={`${styles.marketMetricValue} ${(marketData.txns.m5.buys / Math.max(1, marketData.txns.m5.sells)) >= 1 ? styles.positive : styles.negative}`}>
                  {(marketData.txns.m5.buys / Math.max(1, marketData.txns.m5.sells)).toFixed(2)}
                </span>
              </div>
              <div className={styles.marketMetricItem}>
                <span className={styles.marketMetricLabel}>1h</span>
                <span className={`${styles.marketMetricValue} ${(marketData.txns.h1.buys / Math.max(1, marketData.txns.h1.sells)) >= 1 ? styles.positive : styles.negative}`}>
                  {(marketData.txns.h1.buys / Math.max(1, marketData.txns.h1.sells)).toFixed(2)}
                </span>
              </div>
              <div className={styles.marketMetricItem}>
                <span className={styles.marketMetricLabel}>24h</span>
                <span className={`${styles.marketMetricValue} ${(marketData.txns.h24.buys / Math.max(1, marketData.txns.h24.sells)) >= 1 ? styles.positive : styles.negative}`}>
                  {(marketData.txns.h24.buys / Math.max(1, marketData.txns.h24.sells)).toFixed(2)}
                </span>
              </div>
            </div>
          </div>

          {/* Holder data from backend if available */}
          {metrics && metrics.holder_count > 0 && (
            <div className={styles.marketMetricsSection}>
              <span className={styles.marketSectionTitle}>Holders</span>
              <div className={styles.marketMetricsGrid}>
                <div className={styles.marketMetricItem}>
                  <span className={styles.marketMetricLabel}>Total</span>
                  <span className={styles.marketMetricValue}>{formatNumber(metrics.holder_count)}</span>
                </div>
                <div className={styles.marketMetricItem}>
                  <span className={styles.marketMetricLabel}>Top 10</span>
                  <span className={styles.marketMetricValue}>{(metrics.top_10_concentration ?? 0).toFixed(1)}%</span>
                </div>
                <div className={styles.marketMetricItem}>
                  <span className={styles.marketMetricLabel}>Creator</span>
                  <span className={`${styles.marketMetricValue} ${(metrics.creator_holdings_percent ?? 0) > 10 ? styles.warning : ''}`}>
                    {(metrics.creator_holdings_percent ?? 0).toFixed(1)}%
                  </span>
                </div>
              </div>
            </div>
          )}

          {/* Graduation progress from backend */}
          {metrics && metrics.graduation_progress > 0 && metrics.graduation_progress < 100 && (
            <div className={styles.marketMetricsSection}>
              <span className={styles.marketSectionTitle}>Bonding Curve</span>
              <div className={styles.graduationBar}>
                <div className={styles.graduationFill} style={{ width: `${metrics.graduation_progress}%` }} />
                <span className={styles.graduationText}>{metrics.graduation_progress.toFixed(1)}% to graduation</span>
              </div>
            </div>
          )}
        </div>
      ) : null}

      {/* Exit Config - only for positions */}
      {hasPosition && (
        <div className={styles.exitConfigSection}>
          <div className={styles.exitConfigHeader}>
            <h4>Auto Exit</h4>
            <button
              className={styles.editExitBtn}
              onClick={() => setShowEditExit(!showEditExit)}
            >
              {showEditExit ? 'Cancel' : 'Edit'}
            </button>
          </div>
          {!showEditExit ? (
            <div className={styles.exitBadgesRow}>
              <span className={styles.exitBadgeSL}>SL {exitConfig.stop_loss_percent ?? 15}%</span>
              <span className={styles.exitBadgeTP}>TP {exitConfig.take_profit_percent ?? 50}%</span>
              {(exitConfig.trailing_stop_percent ?? 0) > 0 && (
                <span className={styles.exitBadgeTS}>Trail {exitConfig.trailing_stop_percent}%</span>
              )}
            </div>
          ) : (
            <div className={styles.exitConfigForm}>
              <div className={styles.exitConfigRow}>
                <label>Stop Loss %</label>
                <input
                  type="number"
                  value={exitConfig.stop_loss_percent ?? ''}
                  onChange={(e) => setExitConfig({ ...exitConfig, stop_loss_percent: e.target.value ? parseFloat(e.target.value) : undefined })}
                />
              </div>
              <div className={styles.exitConfigRow}>
                <label>Take Profit %</label>
                <input
                  type="number"
                  value={exitConfig.take_profit_percent ?? ''}
                  onChange={(e) => setExitConfig({ ...exitConfig, take_profit_percent: e.target.value ? parseFloat(e.target.value) : undefined })}
                />
              </div>
              <div className={styles.exitConfigRow}>
                <label>Trailing Stop %</label>
                <input
                  type="number"
                  value={exitConfig.trailing_stop_percent ?? ''}
                  onChange={(e) => setExitConfig({ ...exitConfig, trailing_stop_percent: e.target.value ? parseFloat(e.target.value) : undefined })}
                />
              </div>
              <button className={styles.saveExitBtn} onClick={handleSaveExitConfig} disabled={savingConfig}>
                {savingConfig ? 'Saving...' : 'Save'}
              </button>
            </div>
          )}
        </div>
      )}

      {/* Contract Address */}
      <div className={styles.mintAddressRow}>
        <code className={styles.mintCode} onClick={copyMint}>{mint}</code>
        {copied && <span className={styles.copiedBadge}>Copied!</span>}
      </div>

      {/* Quick Links */}
      <div className={styles.quickLinksRow}>
        <a href={getVenueUrl()} target="_blank" rel="noopener noreferrer" className={styles.quickLinkBtn}>
          {venue.replace('_', '.')}
        </a>
        <a href={`https://solscan.io/token/${mint}`} target="_blank" rel="noopener noreferrer" className={styles.quickLinkBtn}>
          Solscan
        </a>
        <a href={`https://birdeye.so/token/${mint}?chain=solana`} target="_blank" rel="noopener noreferrer" className={styles.quickLinkBtn}>
          Birdeye
        </a>
        <a href={getJupiterUrl()} target="_blank" rel="noopener noreferrer" className={styles.quickLinkBtn}>
          Jupiter
        </a>
        <a
          href={`https://twitter.com/search?q=${encodeURIComponent(`$${symbol} OR ${mint}`)}&f=live`}
          target="_blank"
          rel="noopener noreferrer"
          className={styles.quickLinkBtn}
        >
          X/Twitter
        </a>
      </div>

      {/* Trading Actions */}
      <div className={styles.tradingActions}>
        {/* Buy Section - always shown */}
        <div className={styles.buySection}>
          <h4>Buy {walletBalance > 0 && <span className={styles.balanceHint}>({walletBalance.toFixed(3)} SOL)</span>}</h4>
          <div className={styles.buyInputRow}>
            <input
              type="number"
              value={buyAmount}
              onChange={(e) => setBuyAmount(e.target.value)}
              placeholder="0.05"
              step="0.01"
              min="0.001"
              className={styles.buyInput}
            />
            <span className={styles.buyInputLabel}>SOL</span>
          </div>
          <div className={styles.buyPresets}>
            {['0.01', '0.05', '0.1', '0.25'].map((amt) => (
              <button key={amt} className={styles.buyPresetBtn} onClick={() => setBuyAmount(amt)}>
                {amt}
              </button>
            ))}
            <button
              className={styles.buyPresetBtn}
              onClick={() => setBuyAmount((walletBalance * 0.5).toFixed(4))}
              disabled={walletBalance <= 0}
            >
              Half
            </button>
            <button
              className={styles.buyPresetBtn}
              onClick={() => setBuyAmount((walletBalance * 0.9).toFixed(4))}
              disabled={walletBalance <= 0}
            >
              Max
            </button>
          </div>

          {/* Buy Preview */}
          {buyAmountNum > 0 && (
            <div className={styles.buyPreview}>
              <div className={styles.previewRow}>
                <span className={styles.previewLabel}>You Pay</span>
                <span className={styles.previewValue}>
                  {buyAmountNum.toFixed(4)} SOL
                  {solPriceUsd > 0 && <span className={styles.previewUsd}>(${buyValueUsd.toFixed(2)})</span>}
                </span>
              </div>
              {estimatedTokens > 0 && (
                <div className={styles.previewRow}>
                  <span className={styles.previewLabel}>You Get (est.)</span>
                  <span className={styles.previewValue}>{formatNumber(estimatedTokens)} {symbol}</span>
                </div>
              )}
              {currentPriceNative > 0 && (
                <div className={styles.previewRow}>
                  <span className={styles.previewLabel}>Price</span>
                  <span className={styles.previewValue}>
                    {formatPrice(currentPriceNative)} SOL
                    {currentPriceUsd > 0 && <span className={styles.previewUsd}>(${currentPriceUsd.toFixed(8)})</span>}
                  </span>
                </div>
              )}
              {priceImpact > 0.01 && (
                <div className={styles.previewRow}>
                  <span className={styles.previewLabel}>Price Impact</span>
                  <span className={`${styles.previewValue} ${priceImpact > 5 ? styles.impactHigh : priceImpact > 1 ? styles.impactMedium : styles.impactLow}`}>
                    {priceImpact.toFixed(2)}%
                  </span>
                </div>
              )}
              {marketCapPercent > 0.01 && (
                <div className={styles.previewRow}>
                  <span className={styles.previewLabel}>% of MCap</span>
                  <span className={styles.previewValue}>{marketCapPercent.toFixed(3)}%</span>
                </div>
              )}
            </div>
          )}
          <button className={styles.buyBtn} onClick={handleBuy} disabled={isBuying || !buyAmount}>
            {isBuying ? 'Buying...' : `Buy ${symbol}`}
          </button>
        </div>

        {/* Sell Section - only for positions */}
        {hasPosition && position && (
          <div className={styles.sellSection}>
            <h4>Sell</h4>
            <div className={styles.sellInputRow}>
              <input
                type="number"
                value={sellPercent}
                onChange={(e) => setSellPercent(e.target.value)}
                placeholder="100"
                step="1"
                min="1"
                max="100"
                className={styles.sellInput}
              />
              <span className={styles.sellInputLabel}>%</span>
            </div>
            <div className={styles.sellPresets}>
              <button className={styles.sellPresetBtn} onClick={() => setSellPercent('25')}>25%</button>
              <button className={styles.sellPresetBtn} onClick={() => setSellPercent('50')}>Half</button>
              <button className={styles.sellPresetBtn} onClick={() => setSellPercent('75')}>75%</button>
              <button className={styles.sellPresetBtn} onClick={() => setSellPercent('100')}>Max</button>
            </div>

            {/* Sell Preview */}
            {parseFloat(sellPercent) > 0 && (
              <div className={styles.sellPreview}>
                <div className={styles.previewRow}>
                  <span className={styles.previewLabel}>You Sell</span>
                  <span className={styles.previewValue}>
                    {formatNumber((position.remaining_token_amount ?? position.entry_token_amount ?? 0) * (parseFloat(sellPercent) / 100))} {symbol}
                  </span>
                </div>
                <div className={styles.previewRow}>
                  <span className={styles.previewLabel}>You Receive (est.)</span>
                  <span className={styles.previewValue}>
                    {formatSol((position.current_value_base ?? 0) * (parseFloat(sellPercent) / 100))} SOL
                    {solPriceUsd > 0 && (
                      <span className={styles.previewUsd}>
                        (${((position.current_value_base ?? 0) * (parseFloat(sellPercent) / 100) * solPriceUsd).toFixed(2)})
                      </span>
                    )}
                  </span>
                </div>
                <div className={styles.previewRow}>
                  <span className={styles.previewLabel}>Current PnL</span>
                  <span className={`${styles.previewValue} ${(position.unrealized_pnl ?? 0) >= 0 ? styles.impactLow : styles.impactHigh}`}>
                    {(position.unrealized_pnl ?? 0) >= 0 ? '+' : ''}{formatSol(position.unrealized_pnl)} SOL ({formatPercent(position.unrealized_pnl_percent)})
                  </span>
                </div>
              </div>
            )}

            <button className={styles.sellBtn} onClick={handleSell} disabled={isSelling || !sellPercent}>
              {isSelling ? 'Selling...' : `Sell ${sellPercent}%`}
            </button>
          </div>
        )}
      </div>
    </div>
  );

  const renderRightPanel = () => (
    <div className={styles.positionChartPanel}>
      <div className={styles.chartProviderTabs}>
        {(['birdeye', 'dexscreener', 'defined'] as ChartProvider[]).map((p) => (
          <button
            key={p}
            className={`${styles.chartTab} ${chartProvider === p ? styles.chartTabActive : ''}`}
            onClick={() => setChartProvider(p)}
          >
            {p === 'dexscreener' ? 'DEX' : p === 'birdeye' ? 'Birdeye' : 'Defined'}
          </button>
        ))}
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
  );

  if (asPanel) {
    return (
      <div className={styles.tokenDetailPanelWrapper}>
        <div className={styles.tokenDetailPanelHeader}>
          <button className={styles.backButton} onClick={onClose}>
            <span className={styles.backArrow}>←</span>
            <span>Back</span>
          </button>
          <span className={styles.panelHeaderTitle}>${symbol} Details</span>
        </div>
        <div className={styles.tokenDetailPanelContent}>
          <div className={styles.tokenDetailPanelLayout}>
            {renderLeftPanel()}
            {renderRightPanel()}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.positionModal} onClick={onClose}>
      <div className={styles.positionModalContent} onClick={(e) => e.stopPropagation()}>
        <button className={styles.closeButton} onClick={onClose}>×</button>

        <div className={styles.positionModalLayout}>
          {renderLeftPanel()}
          {renderRightPanel()}
        </div>
      </div>
    </div>
  );
};

export default TokenDetailModal;
