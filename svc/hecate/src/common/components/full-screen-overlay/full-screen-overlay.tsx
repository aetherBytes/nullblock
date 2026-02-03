import styles from './fsoverlay.module.scss';

const FullScreenOverlay = ({ isVisible, onClose, children }: { isVisible: boolean; onClose: () => void; children: React.ReactNode }) => {
  if (!isVisible) {
    return null;
  }

  return (
    <div className={styles.overlay} onClick={onClose}>
      <div className={styles.content} onClick={(e) => e.stopPropagation()}>
        {children}
        <button className={styles.closeButton} onClick={onClose}>
          ✖
        </button>
      </div>
    </div>
  );
};

export default FullScreenOverlay;
