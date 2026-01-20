import React from 'react';
import type { GraduationCandidate, DetailedCurveMetrics, OpportunityScore } from '../../../../types/arbfarm';
import GraduationProgressBar from './GraduationProgressBar';
import styles from '../arbfarm.module.scss';

interface CurveCandidateCardProps {
  candidate: GraduationCandidate;
  onQuickBuy: (mint: string, amount: number) => void;
  onTrack: (mint: string) => void;
  onUntrack?: (mint: string) => void;
  onViewMetrics?: (mint: string, venue: string, symbol: string) => void;
  onViewDetails?: (candidate: GraduationCandidate) => void;
  isTracking?: boolean;
  isTracked?: boolean;
  quickMetrics?: DetailedCurveMetrics;
  opportunityScore?: OpportunityScore;
}

const CurveCandidateCard: React.FC<CurveCandidateCardProps> = ({
  candidate,
  onQuickBuy,
  onTrack,
  onUntrack,
  onViewMetrics,
  onViewDetails,
  isTracking = false,
  isTracked = false,
  quickMetrics,
  opportunityScore,
}) => {
  const { token, graduation_eta_minutes, momentum_score, risk_score } = candidate;

  const formatSol = (sol: number): string => {
    if (sol >= 1000) return `${(sol / 1000).toFixed(1)}K`;
    return sol.toFixed(2);
  };

  const getRecommendation = (): { label: string; class: string } | null => {
    if (opportunityScore) {
      if (opportunityScore.recommendation === 'strong_buy') return { label: 'BUY', class: styles.recBuy };
      if (opportunityScore.recommendation === 'buy') return { label: 'BUY', class: styles.recBuy };
      if (opportunityScore.recommendation === 'hold') return { label: 'HOLD', class: styles.recHold };
      if (opportunityScore.recommendation === 'sell') return { label: 'SELL', class: styles.recSell };
      if (opportunityScore.recommendation === 'strong_sell') return { label: 'SELL', class: styles.recSell };
    }
    if (momentum_score >= 70 && risk_score <= 40) return { label: 'HOT', class: styles.recBuy };
    if (momentum_score <= 30 || risk_score >= 70) return { label: 'RISKY', class: styles.recSell };
    return null;
  };

  const formatVelocity = (velocity: number): string => {
    if (velocity > 0) return `+${velocity.toFixed(1)}/hr`;
    return `${velocity.toFixed(1)}/hr`;
  };

  const recommendation = getRecommendation();

  const getVenueIcon = (venue: string): string => {
    if (venue === 'pump_fun') return '🎰';
    if (venue === 'moonshot') return '🌙';
    return '📈';
  };

  const getMomentumClass = (score: number): string => {
    if (score >= 80) return styles.momentumHigh;
    if (score >= 50) return styles.momentumMedium;
    return styles.momentumLow;
  };

  const getRiskClass = (score: number): string => {
    if (score <= 30) return styles.riskLow;
    if (score <= 60) return styles.riskMedium;
    return styles.riskHigh;
  };

  const handleCardClick = (e: React.MouseEvent) => {
    const target = e.target as HTMLElement;
    if (target.tagName === 'BUTTON' || target.closest('button')) {
      return;
    }
    if (onViewDetails) {
      onViewDetails(candidate);
    }
  };

  return (
    <div
      className={`${styles.curveCandidateCard} ${onViewDetails ? styles.clickable : ''}`}
      onClick={handleCardClick}
    >
      <div className={styles.candidateHeader}>
        <div className={styles.candidateToken}>
          <span className={styles.venueIcon}>{getVenueIcon(token.venue)}</span>
          <div className={styles.tokenInfo}>
            <span className={styles.tokenSymbol}>
              ${token.symbol}
              {recommendation && (
                <span className={`${styles.recBadge} ${recommendation.class}`}>
                  {recommendation.label}
                </span>
              )}
            </span>
            <span className={styles.tokenName}>{token.name}</span>
          </div>
        </div>
        <div className={styles.candidateScores}>
          <span className={`${styles.scoreBadge} ${getMomentumClass(momentum_score)}`}>
            Mom: {momentum_score}
          </span>
          <span className={`${styles.scoreBadge} ${getRiskClass(risk_score)}`}>
            Risk: {risk_score}
          </span>
        </div>
      </div>

      <GraduationProgressBar progress={token.graduation_progress} size="medium" />

      <div className={styles.candidateMetrics}>
        <div className={styles.candidateMetric}>
          <span className={styles.metricLabel}>MC</span>
          <span className={styles.metricValue}>{formatSol(token.market_cap_sol)} SOL</span>
        </div>
        <div className={styles.candidateMetric}>
          <span className={styles.metricLabel}>Vol 24h</span>
          <span className={styles.metricValue}>{formatSol(token.volume_24h_sol)} SOL</span>
        </div>
        <div className={styles.candidateMetric}>
          <span className={styles.metricLabel}>Holders</span>
          <span className={styles.metricValue}>{token.holder_count}</span>
        </div>
        {graduation_eta_minutes && (
          <div className={styles.candidateMetric}>
            <span className={styles.metricLabel}>ETA</span>
            <span className={styles.metricValue}>{graduation_eta_minutes}m</span>
          </div>
        )}
      </div>

      {quickMetrics && (
        <div className={styles.quickMetricsRow}>
          <div className={styles.quickMetric}>
            <span className={styles.quickMetricLabel}>Vol Velocity</span>
            <span className={`${styles.quickMetricValue} ${quickMetrics.volume_velocity > 0 ? styles.positive : styles.negative}`}>
              {formatVelocity(quickMetrics.volume_velocity)}
            </span>
          </div>
          <div className={styles.quickMetric}>
            <span className={styles.quickMetricLabel}>Holder Growth</span>
            <span className={`${styles.quickMetricValue} ${quickMetrics.holder_growth_1h > 0 ? styles.positive : styles.negative}`}>
              {quickMetrics.holder_growth_1h > 0 ? '+' : ''}{quickMetrics.holder_growth_1h}
            </span>
          </div>
          {opportunityScore && (
            <div className={styles.quickMetric}>
              <span className={styles.quickMetricLabel}>Score</span>
              <span className={styles.quickMetricValue}>
                {opportunityScore.overall_score.toFixed(0)}/100
              </span>
            </div>
          )}
        </div>
      )}

      <div className={styles.candidateActions}>
        <div className={styles.actionButtonsRow}>
          <button
            className={`${styles.trackButton} ${isTracked ? styles.tracked : ''}`}
            onClick={() => isTracked && onUntrack ? onUntrack(token.mint) : onTrack(token.mint)}
            disabled={isTracking}
          >
            {isTracking ? 'Processing...' : isTracked ? 'Untrack' : 'Track'}
          </button>
          {onViewMetrics && (
            <button
              className={styles.metricsButton}
              onClick={() => onViewMetrics(token.mint, token.venue, token.symbol)}
            >
              Metrics
            </button>
          )}
        </div>
        <div className={styles.quickBuyButtons}>
          <button
            className={styles.quickBuyButton}
            onClick={() => onQuickBuy(token.mint, 0.1)}
          >
            0.1 SOL
          </button>
          <button
            className={styles.quickBuyButton}
            onClick={() => onQuickBuy(token.mint, 0.5)}
          >
            0.5 SOL
          </button>
          <button
            className={styles.quickBuyButton}
            onClick={() => onQuickBuy(token.mint, 1.0)}
          >
            1 SOL
          </button>
        </div>
      </div>

      <div className={styles.candidateFooter}>
        <span className={styles.mintAddress} title={token.mint}>
          {token.mint.slice(0, 8)}...{token.mint.slice(-6)}
        </span>
        <span className={styles.candidateVenue}>{token.venue.replace('_', '.')}</span>
      </div>
    </div>
  );
};

export default CurveCandidateCard;
