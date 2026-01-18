import React from 'react';
import styles from '../arbfarm.module.scss';

interface WipTabProps {
  title: string;
  description: string;
  icon?: string;
}

const WipTab: React.FC<WipTabProps> = ({ title, description, icon = '' }) => {
  return (
    <div className={styles.dashboardView}>
      <div className={styles.wipContainer}>
        <div className={styles.wipBadge}>Work in Progress</div>
        <div className={styles.wipIcon}>{icon}</div>
        <h2 className={styles.wipTitle}>{title}</h2>
        <p className={styles.wipDescription}>{description}</p>
        <p className={styles.wipNote}>
          This feature is coming soon. Currently focusing on Curve Bonding strategy.
        </p>
      </div>
    </div>
  );
};

export default WipTab;
