import React from 'react';
import styles from './HecateHologram.module.scss';

interface HecateHologramProps {
  agentState: 'idle' | 'thinking' | 'responding';
  agentName: string;
  currentModel: string;
  healthStatus: string;
  sessionMessageCount?: number;
}

const HecateHologram: React.FC<HecateHologramProps> = ({
  agentState,
  agentName,
  currentModel,
  healthStatus,
  sessionMessageCount = 0,
}) => {
  const formatModel = (model: string): string => {
    if (!model) return 'STANDBY';
    return model.split('/').pop()?.split(':')[0]?.toUpperCase() || 'MODEL';
  };

  return (
    <div className={styles.hologram}>
      <div className={styles.projectionBase} />
      <div className={styles.panel}>
        <div className={styles.avatarZone}>
          <div className={`${styles.sigil} ${styles[agentState]}`}>
            <div className={styles.ring1} />
            <div className={styles.ring2} />
            <div className={styles.ring3} />
            <div className={styles.core} />
          </div>
          <div className={styles.entityLabel}>{agentName.toUpperCase()}</div>
          <div className={`${styles.stateTag} ${styles[`state_${agentState}`]}`}>
            {agentState.toUpperCase()}
          </div>
        </div>

        <div className={styles.divider} />

        <div className={styles.readouts}>
          <div className={styles.readoutRow}>
            <span className={styles.label}>MODEL</span>
            <span className={styles.value}>{formatModel(currentModel)}</span>
          </div>
          <div className={styles.readoutRow}>
            <span className={styles.label}>HEALTH</span>
            <span className={`${styles.value} ${styles[`health_${healthStatus}`]}`}>
              {healthStatus.toUpperCase()}
            </span>
          </div>
          {sessionMessageCount > 0 && (
            <div className={styles.readoutRow}>
              <span className={styles.label}>MSGS</span>
              <span className={styles.value}>{sessionMessageCount}</span>
            </div>
          )}
        </div>
        <div className={styles.scanLines} />
      </div>
    </div>
  );
};

export default HecateHologram;
