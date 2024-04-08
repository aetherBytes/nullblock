import React from 'react';
import styles from './modal.module.scss'; // Assume you have corresponding CSS

const Modal = ({ children, isVisible, onClose }) => {
  if (!isVisible) return null;

  return (
    <div className={styles.modalOverlay}>
      <div className={styles.modalContent}>
        <button className={styles.closeButton} onClick={onClose}>
          Close
        </button>
        {children}
      </div>
    </div>
  );
};

export default Modal;
