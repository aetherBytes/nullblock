import React from 'react';
import type { GraduationCandidate } from '../../../../types/arbfarm';
import GraduationProgressBar from './GraduationProgressBar';
import styles from '../arbfarm.module.scss';

interface CurveCandidateCardProps {
  candidate: GraduationCandidate;
  onQuickBuy: (mint: string, amount: number) => void;
  onTrack: (mint: string) => void;
  onViewMetrics?: (mint: string, venue: string, symbol: string) => void;
  onViewDetails?: (candidate: GraduationCandidate) => void;
  isTracking?: boolean;
}

const CurveCandidateCard: React.FC<CurveCandidateCardProps> = ({
  candidate,
  onQuickBuy,
  onTrack,
  onViewMetrics,
  onViewDetails,
  isTracking = false,
}) => {
  const { token, graduation_eta_minutes, momentum_score, risk_score } = candidate;

  const formatSol = (sol: number): string => {
    if (sol >= 1000) return `${(sol / 1000).toFixed(1)}K`;
    return sol.toFixed(2);
  };

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
            <span className={styles.tokenSymbol}>${token.symbol}</span>
            <span className={styles.tokenName}>{token.name}</span>
          </div>
        </div>
        <div className={styles.candidateScores}>
          <span className={`${styles.scoreBadge} ${getMomentumClass(momentum_score)}`}>
            Momentum: {momentum_score}
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

      <div className={styles.candidateActions}>
        <div className={styles.actionButtonsRow}>
          <button
            className={styles.trackButton}
            onClick={() => onTrack(token.mint)}
            disabled={isTracking}
          >
            {isTracking ? 'Tracking...' : 'Track'}
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
