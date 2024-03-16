// CloseButton.tsx
import React from 'react';
import styles from './styles.module.scss';

interface CloseButtonProps {
  onClose: () => void;
}

const CloseButton: React.FC<CloseButtonProps> = ({ onClose }) => (
  <button type="button" className={styles.closeButton} onClick={onClose}>
    Close
  </button>
);

export default CloseButton;
