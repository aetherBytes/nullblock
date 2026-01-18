import React, { useState, useEffect, useCallback } from 'react';
import type {
  DetailedCurveMetrics,
  HolderAnalysis,
  OpportunityScore,
} from '../../../../types/arbfarm';
import { arbFarmService } from '../../../../common/services/arbfarm-service';
import OpportunityScoreCard from './OpportunityScoreCard';
import styles from '../arbfarm.module.scss';

interface CurveMetricsPanelProps {
  mint: string;
  venue: string;
  tokenSymbol?: string;
  onClose?: () => void;
}

type MetricsTab = 'overview' | 'holders' | 'activity';

const CurveMetricsPanel: React.FC<CurveMetricsPanelProps> = ({
  mint,
  venue,
  tokenSymbol,
  onClose,
}) => {
  const [activeTab, setActiveTab] = useState<MetricsTab>('overview');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const [metrics, setMetrics] = useState<DetailedCurveMetrics | null>(null);
  const [holders, setHolders] = useState<HolderAnalysis | null>(null);
  const [score, setScore] = useState<OpportunityScore | null>(null);

  const fetchData = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      const [metricsRes, holdersRes, scoreRes] = await Promise.all([
        arbFarmService.getCurveMetrics(mint),
        arbFarmService.getHolderAnalysis(mint),
        arbFarmService.getOpportunityScore(mint),
      ]);

      if (metricsRes.success && metricsRes.data) {
        setMetrics(metricsRes.data);
      }
      if (holdersRes.success && holdersRes.data) {
        setHolders(holdersRes.data);
      }
      if (scoreRes.success && scoreRes.data) {
        setScore(scoreRes.data);
      }
    } catch (e) {
      setError('Failed to fetch metrics data');
    } finally {
      setLoading(false);
    }
  }, [mint]);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  const formatSol = (sol: number): string => {
    if (sol >= 1000000) return `${(sol / 1000000).toFixed(2)}M`;
    if (sol >= 1000) return `${(sol / 1000).toFixed(1)}K`;
    return sol.toFixed(2);
  };

  const formatPercent = (value: number): string => {
    return `${value.toFixed(1)}%`;
  };

  const formatNumber = (num: number): string => {
    if (num >= 1000000) return `${(num / 1000000).toFixed(1)}M`;
    if (num >= 1000) return `${(num / 1000).toFixed(1)}K`;
    return num.toString();
  };

  const getHealthColor = (healthy: boolean, score: number): string => {
    if (healthy) return '#22c55e';
    if (score >= 50) return '#f59e0b';
    return '#ef4444';
  };

  const getMomentumColor = (momentum: number): string => {
    if (momentum > 10) return '#22c55e';
    if (momentum > 0) return '#3b82f6';
    if (momentum > -10) return '#f59e0b';
    return '#ef4444';
  };

  if (loading) {
    return (
      <div className={styles.metricsPanel}>
        <div className={styles.metricsPanelHeader}>
          <h3>{tokenSymbol ? `$${tokenSymbol}` : 'Loading...'} Metrics</h3>
          {onClose && (
            <button className={styles.closeButton} onClick={onClose}>
              ×
            </button>
          )}
        </div>
        <div className={styles.loadingState}>Loading metrics...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className={styles.metricsPanel}>
        <div className={styles.metricsPanelHeader}>
          <h3>Metrics</h3>
          {onClose && (
            <button className={styles.closeButton} onClick={onClose}>
              ×
            </button>
          )}
        </div>
        <div className={styles.errorState}>{error}</div>
      </div>
    );
  }

  return (
    <div className={styles.metricsPanel}>
      <div className={styles.metricsPanelHeader}>
        <h3>{tokenSymbol ? `$${tokenSymbol}` : mint.slice(0, 8)} Metrics</h3>
        <span className={styles.venueBadge}>{venue.replace('_', '.')}</span>
        {onClose && (
          <button className={styles.closeButton} onClick={onClose}>
            ×
          </button>
        )}
      </div>

      <div className={styles.metricsTabs}>
        <button
          className={`${styles.metricsTab} ${activeTab === 'overview' ? styles.active : ''}`}
          onClick={() => setActiveTab('overview')}
        >
          Overview
        </button>
        <button
          className={`${styles.metricsTab} ${activeTab === 'holders' ? styles.active : ''}`}
          onClick={() => setActiveTab('holders')}
        >
          Holders
        </button>
        <button
          className={`${styles.metricsTab} ${activeTab === 'activity' ? styles.active : ''}`}
          onClick={() => setActiveTab('activity')}
        >
          Activity
        </button>
      </div>

      {activeTab === 'overview' && (
        <div className={styles.metricsContent}>
          {score && <OpportunityScoreCard score={score} />}

          {metrics && (
            <div className={styles.metricsGrid}>
              <div className={styles.metricBox}>
                <span className={styles.metricBoxLabel}>Graduation Progress</span>
                <span className={styles.metricBoxValue}>
                  {formatPercent(metrics.graduation_progress)}
                </span>
                <div className={styles.progressBar}>
                  <div
                    className={styles.progressFill}
                    style={{
                      width: `${metrics.graduation_progress}%`,
                      backgroundColor:
                        metrics.graduation_progress > 90
                          ? '#22c55e'
                          : metrics.graduation_progress > 70
                            ? '#f59e0b'
                            : '#3b82f6',
                    }}
                  />
                </div>
              </div>

              <div className={styles.metricBox}>
                <span className={styles.metricBoxLabel}>Market Cap</span>
                <span className={styles.metricBoxValue}>{formatSol(metrics.market_cap_sol)} SOL</span>
              </div>

              <div className={styles.metricBox}>
                <span className={styles.metricBoxLabel}>Liquidity Depth</span>
                <span className={styles.metricBoxValue}>
                  {formatSol(metrics.liquidity_depth_sol)} SOL
                </span>
              </div>

              <div className={styles.metricBox}>
                <span className={styles.metricBoxLabel}>Price Momentum (1h)</span>
                <span
                  className={styles.metricBoxValue}
                  style={{ color: getMomentumColor(metrics.price_momentum_1h) }}
                >
                  {metrics.price_momentum_1h > 0 ? '+' : ''}
                  {formatPercent(metrics.price_momentum_1h)}
                </span>
              </div>
            </div>
          )}

          {metrics && (
            <div className={styles.qualityScores}>
              <h4>Quality Scores</h4>
              <div className={styles.scoreRow}>
                <span>Holder Quality</span>
                <div className={styles.scoreBar}>
                  <div
                    className={styles.scoreFill}
                    style={{
                      width: `${metrics.holder_quality_score}%`,
                      backgroundColor: '#22c55e',
                    }}
                  />
                </div>
                <span>{metrics.holder_quality_score.toFixed(0)}/100</span>
              </div>
              <div className={styles.scoreRow}>
                <span>Activity</span>
                <div className={styles.scoreBar}>
                  <div
                    className={styles.scoreFill}
                    style={{
                      width: `${metrics.activity_score}%`,
                      backgroundColor: '#3b82f6',
                    }}
                  />
                </div>
                <span>{metrics.activity_score.toFixed(0)}/100</span>
              </div>
              <div className={styles.scoreRow}>
                <span>Momentum</span>
                <div className={styles.scoreBar}>
                  <div
                    className={styles.scoreFill}
                    style={{
                      width: `${metrics.momentum_score}%`,
                      backgroundColor: '#f59e0b',
                    }}
                  />
                </div>
                <span>{metrics.momentum_score.toFixed(0)}/100</span>
              </div>
            </div>
          )}
        </div>
      )}

      {activeTab === 'holders' && holders && (
        <div className={styles.metricsContent}>
          <div className={styles.holdersSummary}>
            <div className={styles.holdersStat}>
              <span className={styles.holdersStatValue}>{formatNumber(holders.total_holders)}</span>
              <span className={styles.holdersStatLabel}>Total Holders</span>
            </div>
            <div className={styles.holdersStat}>
              <span
                className={styles.holdersStatValue}
                style={{ color: getHealthColor(holders.is_healthy, holders.health_score) }}
              >
                {holders.health_score.toFixed(0)}
              </span>
              <span className={styles.holdersStatLabel}>Health Score</span>
            </div>
            <div className={styles.holdersStat}>
              <span className={styles.holdersStatValue}>
                {(holders.gini_coefficient * 100).toFixed(1)}%
              </span>
              <span className={styles.holdersStatLabel}>Gini Coefficient</span>
            </div>
          </div>

          <div className={styles.concentrationSection}>
            <h4>Concentration</h4>
            <div className={styles.concentrationRow}>
              <span>Top 10 Holders</span>
              <span
                style={{
                  color: holders.top_10_concentration > 70 ? '#ef4444' : '#e8e8e8',
                }}
              >
                {formatPercent(holders.top_10_concentration)}
              </span>
            </div>
            <div className={styles.concentrationRow}>
              <span>Top 20 Holders</span>
              <span>{formatPercent(holders.top_20_concentration)}</span>
            </div>
            {holders.creator_address && (
              <div className={styles.concentrationRow}>
                <span>Creator Holdings</span>
                <span
                  style={{
                    color: holders.creator_holdings_percent > 10 ? '#f59e0b' : '#e8e8e8',
                  }}
                >
                  {formatPercent(holders.creator_holdings_percent)}
                </span>
              </div>
            )}
            {holders.wash_trade_likelihood > 0 && (
              <div className={styles.concentrationRow}>
                <span>Wash Trade Likelihood</span>
                <span
                  style={{
                    color: holders.wash_trade_likelihood > 0.5 ? '#ef4444' : '#f59e0b',
                  }}
                >
                  {formatPercent(holders.wash_trade_likelihood * 100)}
                </span>
              </div>
            )}
          </div>

          <div className={styles.topHoldersSection}>
            <h4>Top 10 Holders</h4>
            <div className={styles.holdersList}>
              {holders.top_10_holders.map((holder, idx) => (
                <div key={holder.address} className={styles.holderRow}>
                  <span className={styles.holderRank}>#{idx + 1}</span>
                  <span className={styles.holderAddress}>
                    {holder.address.slice(0, 6)}...{holder.address.slice(-4)}
                    {holder.is_creator && (
                      <span className={styles.creatorBadge}>Creator</span>
                    )}
                    {holder.is_suspicious && (
                      <span className={styles.suspiciousBadge}>⚠️</span>
                    )}
                  </span>
                  <span className={styles.holderPercent}>
                    {formatPercent(holder.balance_percent)}
                  </span>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}

      {activeTab === 'activity' && metrics && (
        <div className={styles.metricsContent}>
          <div className={styles.activityGrid}>
            <div className={styles.activitySection}>
              <h4>Volume</h4>
              <div className={styles.activityRow}>
                <span>1h Volume</span>
                <span>{formatSol(metrics.volume_1h)} SOL</span>
              </div>
              <div className={styles.activityRow}>
                <span>24h Volume</span>
                <span>{formatSol(metrics.volume_24h)} SOL</span>
              </div>
              <div className={styles.activityRow}>
                <span>Volume Velocity</span>
                <span
                  style={{
                    color: metrics.volume_velocity > 0 ? '#22c55e' : '#ef4444',
                  }}
                >
                  {metrics.volume_velocity > 0 ? '+' : ''}
                  {formatSol(metrics.volume_velocity)}/hr
                </span>
              </div>
              <div className={styles.activityRow}>
                <span>Avg Trade Size</span>
                <span>{formatSol(metrics.avg_trade_size_sol)} SOL</span>
              </div>
            </div>

            <div className={styles.activitySection}>
              <h4>Trading Activity</h4>
              <div className={styles.activityRow}>
                <span>Trades (1h)</span>
                <span>{formatNumber(metrics.trade_count_1h)}</span>
              </div>
              <div className={styles.activityRow}>
                <span>Trades (24h)</span>
                <span>{formatNumber(metrics.trade_count_24h)}</span>
              </div>
              <div className={styles.activityRow}>
                <span>Buy/Sell Ratio (1h)</span>
                <span
                  style={{
                    color:
                      metrics.buy_sell_ratio_1h > 1
                        ? '#22c55e'
                        : metrics.buy_sell_ratio_1h < 1
                          ? '#ef4444'
                          : '#e8e8e8',
                  }}
                >
                  {metrics.buy_sell_ratio_1h.toFixed(2)}
                </span>
              </div>
            </div>

            <div className={styles.activitySection}>
              <h4>Holder Activity</h4>
              <div className={styles.activityRow}>
                <span>Unique Buyers (1h)</span>
                <span>{formatNumber(metrics.unique_buyers_1h)}</span>
              </div>
              <div className={styles.activityRow}>
                <span>Unique Buyers (24h)</span>
                <span>{formatNumber(metrics.unique_buyers_24h)}</span>
              </div>
              <div className={styles.activityRow}>
                <span>Holder Growth (1h)</span>
                <span
                  style={{
                    color: metrics.holder_growth_1h > 0 ? '#22c55e' : '#ef4444',
                  }}
                >
                  {metrics.holder_growth_1h > 0 ? '+' : ''}
                  {formatNumber(metrics.holder_growth_1h)}
                </span>
              </div>
              <div className={styles.activityRow}>
                <span>Holder Growth (24h)</span>
                <span
                  style={{
                    color: metrics.holder_growth_24h > 0 ? '#22c55e' : '#ef4444',
                  }}
                >
                  {metrics.holder_growth_24h > 0 ? '+' : ''}
                  {formatNumber(metrics.holder_growth_24h)}
                </span>
              </div>
            </div>

            <div className={styles.activitySection}>
              <h4>Price Movement</h4>
              <div className={styles.activityRow}>
                <span>1h Momentum</span>
                <span style={{ color: getMomentumColor(metrics.price_momentum_1h) }}>
                  {metrics.price_momentum_1h > 0 ? '+' : ''}
                  {formatPercent(metrics.price_momentum_1h)}
                </span>
              </div>
              <div className={styles.activityRow}>
                <span>24h Momentum</span>
                <span style={{ color: getMomentumColor(metrics.price_momentum_24h) }}>
                  {metrics.price_momentum_24h > 0 ? '+' : ''}
                  {formatPercent(metrics.price_momentum_24h)}
                </span>
              </div>
            </div>
          </div>

          <div className={styles.lastUpdated}>
            Last updated: {new Date(metrics.last_updated).toLocaleTimeString()}
          </div>
        </div>
      )}

      <div className={styles.metricsPanelFooter}>
        <span className={styles.mintAddress} title={mint}>
          {mint.slice(0, 12)}...{mint.slice(-8)}
        </span>
        <button className={styles.refreshButton} onClick={fetchData}>
          Refresh
        </button>
      </div>
    </div>
  );
};

export default CurveMetricsPanel;
