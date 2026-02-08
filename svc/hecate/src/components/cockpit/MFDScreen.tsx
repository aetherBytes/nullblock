import React from 'react';
import styles from './MFDScreen.module.scss';

interface MFDScreenProps {
  title: string;
  children: React.ReactNode;
  className?: string;
  statusColor?: 'green' | 'amber' | 'red' | 'cyan';
}

const MFDScreen: React.FC<MFDScreenProps> = ({
  title,
  children,
  className,
  statusColor = 'green',
}) => {
  return (
    <div className={`${styles.mfd} ${className || ''}`}>
      <div className={styles.bezel}>
        <div className={styles.titleBar}>
          <div className={`${styles.statusDot} ${styles[statusColor]}`} />
          <span className={styles.titleLabel}>{title}</span>
        </div>
        <div className={styles.screen}>
          <div className={styles.screenContent}>
            {children}
          </div>
          <div className={styles.scanLines} />
        </div>
        <div className={styles.buttonRow}>
          <div className={styles.btnDot} />
          <div className={styles.btnDot} />
          <div className={styles.btnDot} />
        </div>
      </div>
    </div>
  );
};

export default MFDScreen;
