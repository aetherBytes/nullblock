import React from 'react';
import type { ThreatAlert } from '../../../../types/arbfarm';
import { THREAT_SEVERITY_COLORS } from '../../../../types/arbfarm';
import styles from '../arbfarm.module.scss';

interface ThreatAlertCardProps {
  alert: ThreatAlert;
  compact?: boolean;
}

const ThreatAlertCard: React.FC<ThreatAlertCardProps> = ({ alert, compact = false }) => {
  const severityColor = THREAT_SEVERITY_COLORS[alert.severity];

  const getSeverityIcon = (): string => {
    switch (alert.severity) {
      case 'critical':
        return '🔴';
      case 'high':
        return '🟠';
      case 'medium':
        return '🟡';
      case 'low':
        return '⚪';
      default:
        return '⚪';
    }
  };

  const formatAddress = (address: string): string =>
    `${address.slice(0, 6)}...${address.slice(-4)}`;

  const timeSince = (dateStr: string): string => {
    const date = new Date(dateStr);
    const now = new Date();
    const seconds = Math.floor((now.getTime() - date.getTime()) / 1000);

    if (seconds < 60) {
      return `${seconds}s ago`;
    }

    if (seconds < 3600) {
      return `${Math.floor(seconds / 60)}m ago`;
    }

    if (seconds < 86400) {
      return `${Math.floor(seconds / 3600)}h ago`;
    }

    return `${Math.floor(seconds / 86400)}d ago`;
  };

  if (compact) {
    return (
      <div className={`${styles.alertCardCompact} ${styles[alert.severity]}`}>
        <span className={styles.alertIcon}>{getSeverityIcon()}</span>
        <div className={styles.alertInfo}>
          <span className={styles.alertType}>{alert.alert_type.replace('_', ' ')}</span>
          <span className={styles.alertAddress}>{formatAddress(alert.address)}</span>
        </div>
        <span className={styles.alertTime}>{timeSince(alert.created_at)}</span>
      </div>
    );
  }

  return (
    <div
      className={`${styles.alertCard} ${styles[alert.severity]}`}
      style={{ borderColor: severityColor }}
    >
      <div className={styles.alertHeader}>
        <span className={styles.alertIcon}>{getSeverityIcon()}</span>
        <span className={styles.alertSeverity} style={{ color: severityColor }}>
          {alert.severity.toUpperCase()}
        </span>
        <span className={styles.alertType}>{alert.alert_type.replace(/_/g, ' ')}</span>
      </div>

      <div className={styles.alertBody}>
        <div className={styles.alertEntity}>
          <span className={styles.entityType}>{alert.entity_type}:</span>
          <span className={styles.entityAddress}>{alert.address}</span>
        </div>

        {alert.details && Object.keys(alert.details).length > 0 && (
          <div className={styles.alertDetails}>
            {Object.entries(alert.details)
              .slice(0, 3)
              .map(([key, value]) => (
                <span key={key} className={styles.detailItem}>
                  {key}: {String(value)}
                </span>
              ))}
          </div>
        )}
      </div>

      <div className={styles.alertFooter}>
        <span className={styles.alertAction}>Action: {alert.action_taken}</span>
        <span className={styles.alertTime}>{new Date(alert.created_at).toLocaleString()}</span>
      </div>
    </div>
  );
};

export default ThreatAlertCard;
