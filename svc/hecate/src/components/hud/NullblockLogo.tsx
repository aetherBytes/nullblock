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
      <img
        src={logoSrc}
        alt="NullBlock"
        className={styles.logoImage}
      />
    </div>
  );
};

export default NullblockLogo;
