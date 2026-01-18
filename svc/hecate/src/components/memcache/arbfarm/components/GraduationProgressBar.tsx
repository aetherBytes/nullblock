import React from 'react';
import styles from '../arbfarm.module.scss';

interface GraduationProgressBarProps {
  progress: number;
  showLabel?: boolean;
  size?: 'small' | 'medium' | 'large';
  velocity?: number;
}

const GraduationProgressBar: React.FC<GraduationProgressBarProps> = ({
  progress,
  showLabel = true,
  size = 'medium',
  velocity,
}) => {
  const getProgressColor = (p: number): string => {
    if (p >= 95) return '#22c55e';
    if (p >= 85) return '#f59e0b';
    if (p >= 70) return '#3b82f6';
    return '#6b7280';
  };

  const getStatusLabel = (p: number): string => {
    if (p >= 100) return 'Graduated';
    if (p >= 95) return 'Imminent';
    if (p >= 85) return 'Near';
    if (p >= 70) return 'Tracking';
    return 'Early';
  };

  const color = getProgressColor(progress);
  const sizeClass = size === 'small' ? styles.progressSmall : size === 'large' ? styles.progressLarge : styles.progressMedium;

  return (
    <div className={`${styles.graduationProgress} ${sizeClass}`}>
      {showLabel && (
        <div className={styles.progressHeader}>
          <span className={styles.progressLabel}>{progress.toFixed(1)}%</span>
          <span className={styles.progressStatus} style={{ color }}>
            {getStatusLabel(progress)}
          </span>
        </div>
      )}
      <div className={styles.progressTrack}>
        <div
          className={styles.progressFill}
          style={{
            width: `${Math.min(progress, 100)}%`,
            backgroundColor: color,
          }}
        />
        <div className={styles.progressMarkers}>
          <div className={styles.marker} style={{ left: '70%' }} title="Tracking threshold" />
          <div className={styles.marker} style={{ left: '85%' }} title="Near graduation" />
          <div className={styles.marker} style={{ left: '95%' }} title="Imminent" />
        </div>
      </div>
      {velocity !== undefined && velocity > 0 && (
        <div className={styles.progressVelocity}>
          +{velocity.toFixed(2)}%/min
        </div>
      )}
    </div>
  );
};

export default GraduationProgressBar;
