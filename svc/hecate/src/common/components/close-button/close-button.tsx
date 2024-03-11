// CloseButton.jsx
import React from 'react';
import styles from './styles.module.scss'; // Import styles for the close button

const CloseButton = ({ onClose }) => {
  return (
    <button className={styles.closeButton} onClick={onClose}>
      Close
    </button>
  );
};

export default CloseButton;
