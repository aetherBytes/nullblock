import React from 'react';
import styles from '../crossroads.module.scss';

interface NexusViewProps {
  toolCount: number;
  agentCount: number;
  protocolCount: number;
}

const NexusView: React.FC<NexusViewProps> = ({
  toolCount,
  agentCount,
  protocolCount,
}) => {
  return (
    <div className={styles.nexusView}>
      <div className={styles.nexusContent}>
        <h1 className={styles.nexusTitle}>THE NEXUS</h1>
        <p className={styles.nexusSubtitle}>where agentic flows connect, modify, and evolve</p>
        <div className={styles.nexusStats}>
          <div className={styles.nexusStat}>
            <span className={styles.nexusStatValue}>{toolCount}</span>
            <span className={styles.nexusStatLabel}>TOOLS</span>
          </div>
          <div className={styles.nexusStat}>
            <span className={styles.nexusStatValue}>{agentCount}</span>
            <span className={styles.nexusStatLabel}>AGENTS</span>
          </div>
          <div className={styles.nexusStat}>
            <span className={styles.nexusStatValue}>{protocolCount}</span>
            <span className={styles.nexusStatLabel}>PROTOCOLS</span>
          </div>
        </div>
        <div className={styles.nexusComingSoon}>
          <span>Leaderboards & Highlights Coming Soon</span>
        </div>
      </div>
    </div>
  );
};

export default NexusView;
