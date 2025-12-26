import React from 'react';
import styles from './NullblockLogo.module.scss';

interface NullblockLogoProps {
  state?: 'base' | 'response' | 'question' | 'thinking' | 'alert' | 'error' | 'warning' | 'success' | 'processing' | 'idle';
  theme?: 'null' | 'light' | 'dark';
  onClick?: () => void;
  title?: string;
  size?: 'small' | 'medium' | 'large' | 'xlarge';
}

const NullblockLogo: React.FC<NullblockLogoProps> = ({
  state = 'base',
  theme = 'dark',
  onClick,
  title,
  size = 'medium'
}) => {
  const logoSrc = '/nb_logo_circle_color.png';

  return (
    <div
      className={`${styles.nullblockLogo} ${styles[state]} ${styles[size]}`}
      onClick={onClick}
      title={title}
    >
      <svg
        className={styles.crossroadsRings}
        viewBox="0 0 100 100"
        preserveAspectRatio="xMidYMid meet"
        style={{ overflow: 'visible' }}
      >
        <defs>
          <linearGradient id="ringGradient" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="rgba(255, 250, 245, 0.9)" />
            <stop offset="50%" stopColor="rgba(232, 238, 245, 0.8)" />
            <stop offset="100%" stopColor="rgba(255, 245, 240, 0.7)" />
          </linearGradient>
          <filter id="ringGlow" x="-50%" y="-50%" width="200%" height="200%">
            <feGaussianBlur stdDeviation="1.5" result="blur" />
            <feMerge>
              <feMergeNode in="blur" />
              <feMergeNode in="SourceGraphic" />
            </feMerge>
          </filter>
        </defs>

        {/* Outer halo ring around entire logo */}
        <circle
          cx="50"
          cy="50"
          r="46"
          fill="none"
          stroke="url(#ringGradient)"
          strokeWidth="1.5"
          filter="url(#ringGlow)"
          className={styles.outerRing}
        />
      </svg>
      <img
        src={logoSrc}
        alt="NullBlock"
        className={styles.logoImage}
      />
    </div>
  );
};

export default NullblockLogo;
