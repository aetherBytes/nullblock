import React from 'react';
import type { SwarmHealth, ScannerStatus } from '../../../../types/arbfarm';
import { AGENT_HEALTH_COLORS } from '../../../../types/arbfarm';
import styles from '../arbfarm.module.scss';

interface SwarmStatusCardProps {
  health: SwarmHealth | null;
  scannerStatus: ScannerStatus | null;
  isLoading: boolean;
}

const SwarmStatusCard: React.FC<SwarmStatusCardProps> = ({ health, scannerStatus, isLoading }) => {
  if (isLoading) {
    return (
      <div className={styles.swarmStatusCard}>
        <div className={styles.loadingState}>Loading swarm status...</div>
      </div>
    );
  }

  if (!health) {
    return (
      <div className={styles.swarmStatusCard}>
        <div className={styles.emptyState}>Unable to fetch swarm status</div>
      </div>
    );
  }

  const healthColor = AGENT_HEALTH_COLORS[health.overall_health] || '#6b7280';

  return (
    <div className={styles.swarmStatusCard}>
      <div className={styles.swarmOverview}>
        <div className={styles.swarmIndicator} style={{ backgroundColor: healthColor }} />
        <div className={styles.swarmInfo}>
          <span className={styles.swarmLabel}>Overall Health</span>
          <span className={styles.swarmValue} style={{ color: healthColor }}>
            {health.overall_health}
          </span>
        </div>
        {health.is_paused && <span className={styles.pausedBadge}>PAUSED</span>}
      </div>

      <div className={styles.agentBreakdown}>
        <div className={styles.agentStat}>
          <span
            className={styles.agentDot}
            style={{ backgroundColor: AGENT_HEALTH_COLORS.Healthy }}
          />
          <span>Healthy: {health.healthy_agents}</span>
        </div>
        <div className={styles.agentStat}>
          <span
            className={styles.agentDot}
            style={{ backgroundColor: AGENT_HEALTH_COLORS.Degraded }}
          />
          <span>Degraded: {health.degraded_agents}</span>
        </div>
        <div className={styles.agentStat}>
          <span
            className={styles.agentDot}
            style={{ backgroundColor: AGENT_HEALTH_COLORS.Unhealthy }}
          />
          <span>Unhealthy: {health.unhealthy_agents}</span>
        </div>
        <div className={styles.agentStat}>
          <span className={styles.agentDot} style={{ backgroundColor: AGENT_HEALTH_COLORS.Dead }} />
          <span>Dead: {health.dead_agents}</span>
        </div>
      </div>

      {scannerStatus && (
        <div className={styles.scannerInfo}>
          <div className={styles.scannerHeader}>
            <span
              className={`${styles.scannerDot} ${scannerStatus.is_running ? styles.running : styles.stopped}`}
            />
            <span>Scanner {scannerStatus.is_running ? 'Active' : 'Stopped'}</span>
          </div>
          <div className={styles.scannerStats}>
            <span>Venues: {scannerStatus.venues_active}</span>
            <span>Signals (24h): {scannerStatus.signals_detected_24h}</span>
          </div>
        </div>
      )}
    </div>
  );
};

export default SwarmStatusCard;
