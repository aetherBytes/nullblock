import React from 'react';
import styles from './HessiTooltip.module.scss';

interface HessiTooltipProps {
  isCharging?: boolean;
  isProcessing?: boolean;
  isReceiving?: boolean;
}

/**
 * HessiTooltip - Hover HUD for the HESSI-RHESSI Communicator
 *
 * Named after NASA's HESSI (High Energy Solar Spectroscopic Imager),
 * later renamed RHESSI (Reuven Ramaty High Energy Solar Spectroscopic Imager).
 * The satellite studied solar flares and gamma rays from 2002-2018.
 *
 * In the NullBlock universe, HESSI serves as the biometric-to-digital
 * frequency converter - translating user input into AI-readable signals.
 */
const HessiTooltip: React.FC<HessiTooltipProps> = ({
  isCharging = false,
  isProcessing = false,
  isReceiving = false,
}) => {
  // Determine current status
  const getStatus = () => {
    if (isReceiving) return { text: 'RECEIVING', color: 'receiving' };
    if (isCharging) return { text: 'CHARGING', color: 'charging' };
    if (isProcessing) return { text: 'PROCESSING', color: 'processing' };
    return { text: 'STANDBY', color: 'standby' };
  };

  const status = getStatus();

  return (
    <div className={styles.tooltipContainer}>
      <div className={styles.tooltip}>
        {/* Header */}
        <div className={styles.header}>
          <div className={styles.iconBox}>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
              <circle cx="12" cy="12" r="3" />
              <path d="M12 2v4M12 18v4M2 12h4M18 12h4" />
              <path d="M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83" />
            </svg>
          </div>
          <div className={styles.titleGroup}>
            <h3 className={styles.title}>HESSI COMS</h3>
            <span className={styles.subtitle}>Frequency Converter</span>
          </div>
        </div>

        {/* Status Badge */}
        <div className={`${styles.statusBadge} ${styles[status.color]}`}>
          <span className={styles.statusDot} />
          <span className={styles.statusText}>{status.text}</span>
        </div>

        {/* Description */}
        <div className={styles.description}>
          <p className={styles.descText}>
            Biometric-to-digital signal translator. Converts organic frequency patterns
            into quantum-encoded transmissions for vessel AI communication.
          </p>
        </div>

        {/* NASA Reference */}
        <div className={styles.reference}>
          <span className={styles.refIcon}>🛰️</span>
          <span className={styles.refText}>
            Inspired by NASA RHESSI — High Energy Solar Spectroscopic Imager (2002-2018)
          </span>
        </div>
      </div>
    </div>
  );
};

export default HessiTooltip;
