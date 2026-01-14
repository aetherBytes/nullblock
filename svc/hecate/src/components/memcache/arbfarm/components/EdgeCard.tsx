import React, { useState } from 'react';
import type { Edge } from '../../../../types/arbfarm';
import { EDGE_STATUS_COLORS, VENUE_TYPE_ICONS } from '../../../../types/arbfarm';
import styles from '../arbfarm.module.scss';

interface EdgeCardProps {
  edge: Edge;
  onApprove: () => void;
  onReject: (reason: string) => void;
  onExecute: () => void;
  compact?: boolean;
}

const EdgeCard: React.FC<EdgeCardProps> = ({
  edge,
  onApprove,
  onReject,
  onExecute,
  compact = false,
}) => {
  const [showRejectInput, setShowRejectInput] = useState(false);
  const [rejectReason, setRejectReason] = useState('');

  const statusColor = EDGE_STATUS_COLORS[edge.status];
  const venueIcon = VENUE_TYPE_ICONS[edge.venue_type];

  const formatProfit = (lamports: number): string => {
    const sol = lamports / 1_000_000_000;

    return `${sol >= 0 ? '+' : ''}${sol.toFixed(4)} SOL`;
  };

  const formatProfitBps = (lamports: number, inputLamports: number): string => {
    if (inputLamports === 0) {
      return '0 bps';
    }

    const bps = (lamports / inputLamports) * 10000;

    return `${bps.toFixed(0)} bps`;
  };

  const handleReject = () => {
    if (rejectReason.trim()) {
      onReject(rejectReason.trim());
      setShowRejectInput(false);
      setRejectReason('');
    }
  };

  const getPriorityClass = (): string => {
    if (edge.atomicity === 'fully_atomic' && edge.simulated_profit_guaranteed) {
      return styles.priorityCritical;
    }

    if (edge.estimated_profit_lamports > 100_000_000) {
      return styles.priorityHigh;
    }

    if (edge.risk_score < 30) {
      return styles.priorityMedium;
    }

    return styles.priorityLow;
  };

  if (compact) {
    return (
      <div className={`${styles.edgeCardCompact} ${getPriorityClass()}`}>
        <div className={styles.edgeHeader}>
          <span className={styles.edgeType}>
            {venueIcon} {edge.edge_type.replace('_', ' ')}
          </span>
          <span className={styles.edgeStatus} style={{ color: statusColor }}>
            {edge.status.replace('_', ' ')}
          </span>
        </div>
        <div className={styles.edgeRoute}>
          {edge.route_data.input_token.slice(0, 4)}... → {edge.route_data.output_token.slice(0, 4)}
          ...
        </div>
        <div className={styles.edgeProfit}>
          <span className={edge.estimated_profit_lamports >= 0 ? styles.positive : styles.negative}>
            {formatProfit(edge.estimated_profit_lamports)}
          </span>
          {edge.simulated_profit_guaranteed && (
            <span className={styles.guaranteedBadge}>Guaranteed</span>
          )}
        </div>
        {(edge.status === 'detected' || edge.status === 'pending_approval') && (
          <div className={styles.quickActions}>
            <button className={styles.approveBtn} onClick={onApprove}>
              ✓
            </button>
            <button className={styles.executeBtn} onClick={onExecute}>
              ▶
            </button>
          </div>
        )}
      </div>
    );
  }

  return (
    <div className={`${styles.edgeCard} ${getPriorityClass()}`}>
      <div className={styles.edgeHeader}>
        <div className={styles.edgeTypeInfo}>
          <span className={styles.edgeIcon}>{venueIcon}</span>
          <span className={styles.edgeTypeName}>{edge.edge_type.replace('_', ' ')}</span>
          <span className={styles.venueType}>{edge.venue_type.replace('_', ' ')}</span>
        </div>
        <span className={styles.edgeStatus} style={{ backgroundColor: statusColor }}>
          {edge.status.replace('_', ' ')}
        </span>
      </div>

      <div className={styles.edgeRoute}>
        <div className={styles.routeTokens}>
          <span className={styles.token}>{edge.route_data.input_token}</span>
          <span className={styles.routeArrow}>→</span>
          <span className={styles.token}>{edge.route_data.output_token}</span>
        </div>
        <div className={styles.routeVenues}>via {edge.route_data.venues.join(' → ')}</div>
      </div>

      <div className={styles.edgeMetrics}>
        <div className={styles.metric}>
          <span className={styles.metricLabel}>Est. Profit</span>
          <span
            className={`${styles.metricValue} ${edge.estimated_profit_lamports >= 0 ? styles.positive : styles.negative}`}
          >
            {formatProfit(edge.estimated_profit_lamports)}
          </span>
        </div>
        <div className={styles.metric}>
          <span className={styles.metricLabel}>Risk Score</span>
          <span className={styles.metricValue}>{edge.risk_score}/100</span>
        </div>
        <div className={styles.metric}>
          <span className={styles.metricLabel}>Atomicity</span>
          <span className={styles.metricValue}>{edge.atomicity.replace('_', ' ')}</span>
        </div>
        <div className={styles.metric}>
          <span className={styles.metricLabel}>Mode</span>
          <span className={styles.metricValue}>{edge.execution_mode.replace('_', ' ')}</span>
        </div>
      </div>

      {edge.simulated_profit_guaranteed && (
        <div className={styles.guaranteedBanner}>
          ✓ Simulated profit guaranteed - Zero capital risk
        </div>
      )}

      {edge.rejection_reason && (
        <div className={styles.rejectionReason}>Rejected: {edge.rejection_reason}</div>
      )}

      {(edge.status === 'detected' || edge.status === 'pending_approval') && (
        <div className={styles.edgeActions}>
          {showRejectInput ? (
            <div className={styles.rejectInput}>
              <input
                type="text"
                placeholder="Rejection reason..."
                value={rejectReason}
                onChange={(e) => setRejectReason(e.target.value)}
              />
              <button onClick={handleReject} disabled={!rejectReason.trim()}>
                Confirm
              </button>
              <button onClick={() => setShowRejectInput(false)}>Cancel</button>
            </div>
          ) : (
            <>
              <button className={styles.approveButton} onClick={onApprove}>
                ✓ Approve
              </button>
              <button className={styles.rejectButton} onClick={() => setShowRejectInput(true)}>
                ✗ Reject
              </button>
              <button className={styles.executeButton} onClick={onExecute}>
                ▶ Execute Now
              </button>
            </>
          )}
        </div>
      )}

      <div className={styles.edgeFooter}>
        <span className={styles.edgeId}>{edge.id.slice(0, 8)}...</span>
        <span className={styles.edgeTime}>{new Date(edge.created_at).toLocaleString()}</span>
        {edge.expires_at && (
          <span className={styles.edgeExpiry}>
            Expires: {new Date(edge.expires_at).toLocaleTimeString()}
          </span>
        )}
      </div>
    </div>
  );
};

export default EdgeCard;
