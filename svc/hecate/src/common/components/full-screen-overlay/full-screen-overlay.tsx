
import React from 'react';
import styles from './fsoverlay.module.scss'; // Path to your SCSS file

const FullScreenOverlay = ({ isVisible, onClose, children }) => {
  if (!isVisible) return null;

  return (
    <div className={styles.overlay} onClick={onClose}>
      <div className={styles.content} onClick={e => e.stopPropagation()}>
        {children}
        <button className={styles.closeButton} onClick={onClose}>✖</button>
      </div>
    </div>
  );
};

export default FullScreenOverlay;
