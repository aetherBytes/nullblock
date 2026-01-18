import React from 'react';
import type { OpportunityScore } from '../../../../types/arbfarm';
import { RECOMMENDATION_COLORS, RECOMMENDATION_LABELS } from '../../../../types/arbfarm';
import styles from '../arbfarm.module.scss';

interface OpportunityScoreCardProps {
  score: OpportunityScore;
  onViewDetails?: () => void;
  compact?: boolean;
}

const OpportunityScoreCard: React.FC<OpportunityScoreCardProps> = ({
  score,
  onViewDetails,
  compact = false,
}) => {
  const getScoreColor = (value: number): string => {
    if (value >= 80) return '#22c55e';
    if (value >= 60) return '#3b82f6';
    if (value >= 40) return '#f59e0b';
    return '#ef4444';
  };

  const formatPercent = (value: number): string => {
    return `${value.toFixed(1)}%`;
  };

  if (compact) {
    return (
      <div className={styles.scoreCardCompact}>
        <div className={styles.scoreGaugeCompact}>
          <div
            className={styles.scoreValue}
            style={{ color: getScoreColor(score.overall_score) }}
          >
            {score.overall_score.toFixed(0)}
          </div>
          <div className={styles.scoreLabel}>Score</div>
        </div>
        <div
          className={styles.recommendationBadge}
          style={{
            backgroundColor: `${RECOMMENDATION_COLORS[score.recommendation]}20`,
            color: RECOMMENDATION_COLORS[score.recommendation],
            borderColor: RECOMMENDATION_COLORS[score.recommendation],
          }}
        >
          {RECOMMENDATION_LABELS[score.recommendation]}
        </div>
      </div>
    );
  }

  return (
    <div className={styles.opportunityScoreCard}>
      <div className={styles.scoreHeader}>
        <h4>Opportunity Score</h4>
        <div
          className={styles.recommendationBadge}
          style={{
            backgroundColor: `${RECOMMENDATION_COLORS[score.recommendation]}20`,
            color: RECOMMENDATION_COLORS[score.recommendation],
            borderColor: RECOMMENDATION_COLORS[score.recommendation],
          }}
        >
          {RECOMMENDATION_LABELS[score.recommendation]}
        </div>
      </div>

      <div className={styles.scoreGauge}>
        <svg viewBox="0 0 100 50" className={styles.gaugeSvg}>
          <path
            d="M 10 45 A 40 40 0 0 1 90 45"
            fill="none"
            stroke="#2a2a3a"
            strokeWidth="8"
            strokeLinecap="round"
          />
          <path
            d="M 10 45 A 40 40 0 0 1 90 45"
            fill="none"
            stroke={getScoreColor(score.overall_score)}
            strokeWidth="8"
            strokeLinecap="round"
            strokeDasharray={`${(score.overall_score / 100) * 126} 126`}
          />
        </svg>
        <div className={styles.gaugeCenter}>
          <span
            className={styles.gaugeValue}
            style={{ color: getScoreColor(score.overall_score) }}
          >
            {score.overall_score.toFixed(0)}
          </span>
          <span className={styles.gaugeMax}>/100</span>
        </div>
      </div>

      <div className={styles.factorBreakdown}>
        <div className={styles.factorRow}>
          <span className={styles.factorLabel}>Graduation</span>
          <div className={styles.factorBar}>
            <div
              className={styles.factorFill}
              style={{
                width: formatPercent(score.graduation_factor),
                backgroundColor: '#8b5cf6',
              }}
            />
          </div>
          <span className={styles.factorValue}>{score.graduation_factor.toFixed(0)}</span>
        </div>
        <div className={styles.factorRow}>
          <span className={styles.factorLabel}>Volume</span>
          <div className={styles.factorBar}>
            <div
              className={styles.factorFill}
              style={{
                width: formatPercent(score.volume_factor),
                backgroundColor: '#3b82f6',
              }}
            />
          </div>
          <span className={styles.factorValue}>{score.volume_factor.toFixed(0)}</span>
        </div>
        <div className={styles.factorRow}>
          <span className={styles.factorLabel}>Holders</span>
          <div className={styles.factorBar}>
            <div
              className={styles.factorFill}
              style={{
                width: formatPercent(score.holder_factor),
                backgroundColor: '#22c55e',
              }}
            />
          </div>
          <span className={styles.factorValue}>{score.holder_factor.toFixed(0)}</span>
        </div>
        <div className={styles.factorRow}>
          <span className={styles.factorLabel}>Momentum</span>
          <div className={styles.factorBar}>
            <div
              className={styles.factorFill}
              style={{
                width: formatPercent(score.momentum_factor),
                backgroundColor: '#f59e0b',
              }}
            />
          </div>
          <span className={styles.factorValue}>{score.momentum_factor.toFixed(0)}</span>
        </div>
        {score.risk_penalty > 0 && (
          <div className={styles.factorRow}>
            <span className={styles.factorLabel}>Risk Penalty</span>
            <div className={styles.factorBar}>
              <div
                className={styles.factorFill}
                style={{
                  width: formatPercent(score.risk_penalty * 100),
                  backgroundColor: '#ef4444',
                }}
              />
            </div>
            <span className={styles.factorValue} style={{ color: '#ef4444' }}>
              -{(score.risk_penalty * 100).toFixed(0)}%
            </span>
          </div>
        )}
      </div>

      {score.risk_warnings.length > 0 && (
        <div className={styles.warningsSection}>
          <h5>Risk Warnings</h5>
          <ul className={styles.warningsList}>
            {score.risk_warnings.map((warning, idx) => (
              <li key={idx} className={styles.warningItem}>
                <span className={styles.warningIcon}>⚠️</span>
                {warning}
              </li>
            ))}
          </ul>
        </div>
      )}

      {score.positive_signals.length > 0 && (
        <div className={styles.signalsSection}>
          <h5>Positive Signals</h5>
          <ul className={styles.signalsList}>
            {score.positive_signals.map((signal, idx) => (
              <li key={idx} className={styles.signalItem}>
                <span className={styles.signalIcon}>✅</span>
                {signal}
              </li>
            ))}
          </ul>
        </div>
      )}

      {onViewDetails && (
        <button className={styles.viewDetailsButton} onClick={onViewDetails}>
          View Full Analysis
        </button>
      )}
    </div>
  );
};

export default OpportunityScoreCard;
