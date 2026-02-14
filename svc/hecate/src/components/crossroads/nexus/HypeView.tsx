import React from 'react';
import styles from '../crossroads.module.scss';

interface HypeViewProps {
  toolCount: number;
  agentCount: number;
  protocolCount: number;
}

const HypeView: React.FC<HypeViewProps> = ({
  toolCount,
  agentCount,
  protocolCount,
}) => {
  return (
    <div className={styles.hypeView}>
      <div className={styles.hypeContent}>
        <h1 className={styles.hypeTitle}>HYPE</h1>
        <p className={styles.hypeSubtitle}>where agentic flows connect, modify, and evolve</p>
        <div className={styles.hypeStats}>
          <div className={styles.hypeStat}>
            <span className={styles.hypeStatValue}>{toolCount}</span>
            <span className={styles.hypeStatLabel}>TOOLS</span>
          </div>
          <div className={styles.hypeStat}>
            <span className={styles.hypeStatValue}>{agentCount}</span>
            <span className={styles.hypeStatLabel}>AGENTS</span>
          </div>
          <div className={styles.hypeStat}>
            <span className={styles.hypeStatValue}>{protocolCount}</span>
            <span className={styles.hypeStatLabel}>PROTOCOLS</span>
          </div>
        </div>
        <div className={styles.hypeComingSoon}>
          <span>Leaderboards & Highlights Coming Soon</span>
        </div>
      </div>
    </div>
  );
};

export default HypeView;
