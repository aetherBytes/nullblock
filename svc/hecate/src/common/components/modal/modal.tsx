import styles from './modal.module.scss';

const Modal = ({ children, isVisible, onClose }: { children: React.ReactNode; isVisible: boolean; onClose: () => void }) => {
  if (!isVisible) {
    return null;
  }

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
