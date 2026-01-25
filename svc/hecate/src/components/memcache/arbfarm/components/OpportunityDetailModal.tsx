import React, { useState, useEffect } from 'react';
import type { GraduationCandidate, DetailedCurveMetrics, OpportunityScore } from '../../../../types/arbfarm';
import { arbFarmService } from '../../../../common/services/arbfarm-service';
import GraduationProgressBar from './GraduationProgressBar';
import styles from '../arbfarm.module.scss';

interface OpportunityDetailModalProps {
  candidate: GraduationCandidate;
  onClose: () => void;
  onQuickBuy: (mint: string, amount: number) => void;
  onTrack: (mint: string) => void;
  onSuccess: (message: string) => void;
  onError: (message: string) => void;
}

const OpportunityDetailModal: React.FC<OpportunityDetailModalProps> = ({
  candidate,
  onClose,
  onQuickBuy,
  onTrack,
  onSuccess,
  onError,
}) => {
  const [metrics, setMetrics] = useState<DetailedCurveMetrics | null>(null);
  const [score, setScore] = useState<OpportunityScore | null>(null);
  const [loading, setLoading] = useState(true);
  const [customAmount, setCustomAmount] = useState('0.5');
  const [copied, setCopied] = useState(false);

  const { token, graduation_eta_minutes, momentum_score, risk_score } = candidate;

  useEffect(() => {
    const fetchDetails = async () => {
      setLoading(true);
      try {
        const [metricsRes, scoreRes] = await Promise.all([
          arbFarmService.getCurveMetrics(token.mint, token.venue),
          arbFarmService.getOpportunityScore(token.mint),
        ]);
        if (metricsRes.success && metricsRes.data) {
          setMetrics(metricsRes.data);
        }
        if (scoreRes.success && scoreRes.data) {
          setScore(scoreRes.data);
        }
      } catch (e) {
        console.error('Failed to fetch details:', e);
      } finally {
        setLoading(false);
      }
    };
    fetchDetails();
  }, [token.mint, token.venue]);

  const formatSol = (sol: number | undefined | null): string => {
    if (sol == null) return '0.00';
    if (sol >= 1000000) return `${(sol / 1000000).toFixed(2)}M`;
    if (sol >= 1000) return `${(sol / 1000).toFixed(1)}K`;
    return sol.toFixed(2);
  };

  const formatNumber = (num: number | undefined | null): string => {
    if (num == null) return '0';
    if (num >= 1000000) return `${(num / 1000000).toFixed(2)}M`;
    if (num >= 1000) return `${(num / 1000).toFixed(1)}K`;
    return num.toLocaleString();
  };

  const getVenueUrl = (): string => {
    if (token.venue === 'pump_fun') {
      return `https://pump.fun/${token.mint}`;
    }
    if (token.venue === 'moonshot') {
      return `https://moonshot.cc/token/${token.mint}`;
    }
    return `https://solscan.io/token/${token.mint}`;
  };

  const getExplorerUrl = (): string => {
    return `https://solscan.io/token/${token.mint}`;
  };

  const copyMint = () => {
    navigator.clipboard.writeText(token.mint);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const getRecommendationColor = (rec: string): string => {
    switch (rec.toLowerCase()) {
      case 'strong_buy': return '#00ff88';
      case 'buy': return '#88ff00';
      case 'hold': return '#ffaa00';
      case 'avoid': return '#ff4444';
      default: return '#888';
    }
  };

  const getRecommendationLabel = (rec: string): string => {
    switch (rec.toLowerCase()) {
      case 'strong_buy': return 'STRONG BUY';
      case 'buy': return 'BUY';
      case 'hold': return 'HOLD';
      case 'avoid': return 'AVOID';
      default: return rec.toUpperCase();
    }
  };

  const handleBuy = () => {
    const amount = parseFloat(customAmount);
    if (isNaN(amount) || amount <= 0) {
      onError('Invalid amount');
      return;
    }
    onQuickBuy(token.mint, amount);
  };

  return (
    <div className={styles.opportunityModal} onClick={onClose}>
      <div className={styles.opportunityModalContent} onClick={(e) => e.stopPropagation()}>
        <button className={styles.closeButton} onClick={onClose}>×</button>

        <div className={styles.opportunityHeader}>
          <div className={styles.tokenIdentity}>
            <span className={styles.venueIconLarge}>
              {token.venue === 'pump_fun' ? '🎰' : token.venue === 'moonshot' ? '🌙' : '📈'}
            </span>
            <div>
              <h2>${token.symbol}</h2>
              <span className={styles.tokenFullName}>{token.name}</span>
            </div>
          </div>

          {score && (
            <div
              className={styles.recommendationBadge}
              style={{ backgroundColor: getRecommendationColor(score.recommendation) }}
            >
              {getRecommendationLabel(score.recommendation)}
            </div>
          )}
        </div>

        <div className={styles.progressSection}>
          <GraduationProgressBar progress={token.graduation_progress ?? 0} size="large" />
          <div className={styles.progressDetails}>
            <span>{(token.graduation_progress ?? 0).toFixed(1)}% to graduation</span>
            {graduation_eta_minutes && graduation_eta_minutes > 0 && (
              <span className={styles.eta}>ETA: {graduation_eta_minutes} min</span>
            )}
          </div>
        </div>

        <div className={styles.scoreGrid}>
          <div className={styles.scoreCard}>
            <span className={styles.scoreLabel}>Overall Score</span>
            <span className={styles.scoreValue}>{(score?.overall ?? momentum_score ?? 0).toFixed(0)}/100</span>
          </div>
          <div className={styles.scoreCard}>
            <span className={styles.scoreLabel}>Momentum</span>
            <span className={styles.scoreValue}>{momentum_score ?? 0}</span>
          </div>
          <div className={styles.scoreCard}>
            <span className={styles.scoreLabel}>Risk</span>
            <span className={styles.scoreValue}>{risk_score ?? 0}</span>
          </div>
          {score && (
            <>
              <div className={styles.scoreCard}>
                <span className={styles.scoreLabel}>Grad Factor</span>
                <span className={styles.scoreValue}>{(score.graduation_factor ?? 0).toFixed(0)}</span>
              </div>
              <div className={styles.scoreCard}>
                <span className={styles.scoreLabel}>Volume Factor</span>
                <span className={styles.scoreValue}>{(score.volume_factor ?? 0).toFixed(0)}</span>
              </div>
              <div className={styles.scoreCard}>
                <span className={styles.scoreLabel}>Holder Factor</span>
                <span className={styles.scoreValue}>{(score.holder_factor ?? 0).toFixed(0)}</span>
              </div>
            </>
          )}
        </div>

        <div className={styles.metricsSection}>
          <h3>Market Metrics</h3>
          {loading ? (
            <div className={styles.loadingState}>Loading metrics...</div>
          ) : (
            <div className={styles.metricsGrid}>
              <div className={styles.metricItem}>
                <span className={styles.metricLabel}>Market Cap</span>
                <span className={styles.metricValue}>{formatSol(token.market_cap_sol)} SOL</span>
              </div>
              <div className={styles.metricItem}>
                <span className={styles.metricLabel}>Volume (24h)</span>
                <span className={styles.metricValue}>{formatSol(token.volume_24h_sol)} SOL</span>
              </div>
              <div className={styles.metricItem}>
                <span className={styles.metricLabel}>Holders</span>
                <span className={styles.metricValue}>{formatNumber(token.holder_count)}</span>
              </div>
              {metrics && (
                <>
                  <div className={styles.metricItem}>
                    <span className={styles.metricLabel}>Volume (1h)</span>
                    <span className={styles.metricValue}>{formatSol(metrics.volume_1h ?? 0)} SOL</span>
                  </div>
                  <div className={styles.metricItem}>
                    <span className={styles.metricLabel}>Trades (1h)</span>
                    <span className={styles.metricValue}>{metrics.trade_count_1h ?? 0}</span>
                  </div>
                  <div className={styles.metricItem}>
                    <span className={styles.metricLabel}>Unique Buyers (1h)</span>
                    <span className={styles.metricValue}>{metrics.unique_buyers_1h ?? 0}</span>
                  </div>
                  <div className={styles.metricItem}>
                    <span className={styles.metricLabel}>Holder Growth (1h)</span>
                    <span className={`${styles.metricValue} ${(metrics.holder_growth_1h ?? 0) >= 0 ? styles.positive : styles.negative}`}>
                      {(metrics.holder_growth_1h ?? 0) >= 0 ? '+' : ''}{metrics.holder_growth_1h ?? 0}
                    </span>
                  </div>
                  <div className={styles.metricItem}>
                    <span className={styles.metricLabel}>Top 10 Concentration</span>
                    <span className={styles.metricValue}>{(metrics.top_10_concentration ?? 0).toFixed(1)}%</span>
                  </div>
                  <div className={styles.metricItem}>
                    <span className={styles.metricLabel}>Creator Holdings</span>
                    <span className={styles.metricValue}>{(metrics.creator_holdings_percent ?? 0).toFixed(1)}%</span>
                  </div>
                  <div className={styles.metricItem}>
                    <span className={styles.metricLabel}>Volume Velocity</span>
                    <span className={`${styles.metricValue} ${(metrics.volume_velocity ?? 0) >= 0 ? styles.positive : styles.negative}`}>
                      {(metrics.volume_velocity ?? 0) >= 0 ? '+' : ''}{(metrics.volume_velocity ?? 0).toFixed(2)}
                    </span>
                  </div>
                  <div className={styles.metricItem}>
                    <span className={styles.metricLabel}>Price Momentum</span>
                    <span className={`${styles.metricValue} ${(metrics.price_momentum_1h ?? 0) >= 0 ? styles.positive : styles.negative}`}>
                      {(metrics.price_momentum_1h ?? 0) >= 0 ? '+' : ''}{(metrics.price_momentum_1h ?? 0).toFixed(2)}%
                    </span>
                  </div>
                </>
              )}
            </div>
          )}
        </div>

        <div className={styles.addressSection}>
          <span className={styles.addressLabel}>Contract Address</span>
          <div className={styles.addressRow}>
            <code className={styles.mintAddress}>{token.mint}</code>
            <button className={styles.copyButton} onClick={copyMint}>
              {copied ? '✓ Copied' : 'Copy'}
            </button>
          </div>
        </div>

        <div className={styles.linksSection}>
          <a href={getVenueUrl()} target="_blank" rel="noopener noreferrer" className={styles.linkButton}>
            {token.venue === 'pump_fun' ? '🎰 pump.fun' : token.venue === 'moonshot' ? '🌙 moonshot' : '📈 Trade'}
          </a>
          <a href={getExplorerUrl()} target="_blank" rel="noopener noreferrer" className={styles.linkButton}>
            🔍 Solscan
          </a>
          <a href={`https://birdeye.so/token/${token.mint}?chain=solana`} target="_blank" rel="noopener noreferrer" className={styles.linkButton}>
            🦅 Birdeye
          </a>
          <a href={`https://dexscreener.com/solana/${token.mint}`} target="_blank" rel="noopener noreferrer" className={styles.linkButton}>
            📊 DEX Screener
          </a>
          <a href={`https://jup.ag/swap?sell=${token.mint}&buy=So11111111111111111111111111111111111111112`} target="_blank" rel="noopener noreferrer" className={styles.linkButton}>
            🪐 Jupiter
          </a>
        </div>

        <div className={styles.actionsSection}>
          <div className={styles.buySection}>
            <h4>Quick Buy</h4>
            <div className={styles.quickBuyRow}>
              <button className={styles.quickBuyBtn} onClick={() => onQuickBuy(token.mint, 0.1)}>
                0.1 SOL
              </button>
              <button className={styles.quickBuyBtn} onClick={() => onQuickBuy(token.mint, 0.25)}>
                0.25 SOL
              </button>
              <button className={styles.quickBuyBtn} onClick={() => onQuickBuy(token.mint, 0.5)}>
                0.5 SOL
              </button>
              <button className={styles.quickBuyBtn} onClick={() => onQuickBuy(token.mint, 1.0)}>
                1 SOL
              </button>
            </div>
            <div className={styles.customBuyRow}>
              <input
                type="number"
                step="0.1"
                min="0.01"
                value={customAmount}
                onChange={(e) => setCustomAmount(e.target.value)}
                placeholder="Custom amount"
              />
              <button className={styles.buyButton} onClick={handleBuy}>
                Buy {customAmount} SOL
              </button>
            </div>
          </div>

          <div className={styles.otherActions}>
            <button className={styles.trackButton} onClick={() => onTrack(token.mint)}>
              📍 Track for Graduation
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default OpportunityDetailModal;
