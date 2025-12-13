import React, { useEffect, useState } from 'react';
import ApiKeyManagement from './ApiKeyManagement';
import styles from './SettingsPanel.module.scss';

interface SettingsPanelProps {
  isOpen: boolean;
  onClose: () => void;
  userId: string | null;
  publicKey: string | null;
  isLoadingUser: boolean;
}

const SettingsPanel: React.FC<SettingsPanelProps> = ({
  isOpen,
  onClose,
  userId,
  publicKey,
  isLoadingUser,
}) => {
  const [isClosing, setIsClosing] = useState(false);

  useEffect(() => {
    if (isOpen) {
      setIsClosing(false);
      document.body.style.overflow = 'hidden';
    } else {
      document.body.style.overflow = 'unset';
    }

    return () => {
      document.body.style.overflow = 'unset';
    };
  }, [isOpen]);

  useEffect(() => {
    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape' && isOpen) {
        handleClose();
      }
    };

    document.addEventListener('keydown', handleEscape);
    return () => {
      document.removeEventListener('keydown', handleEscape);
    };
  }, [isOpen, onClose]);

  const handleClose = () => {
    setIsClosing(true);
    setTimeout(() => {
      onClose();
      setIsClosing(false);
    }, 300);
  };

  if (!isOpen && !isClosing) {
    return null;
  }

  return (
    <div
      className={`${styles.settingsOverlay} ${isClosing ? styles.closing : ''}`}
      onClick={handleClose}
    >
      <div
        className={`${styles.settingsPanel} ${isClosing ? styles.closing : ''}`}
        onClick={(e) => e.stopPropagation()}
      >
        <div className={styles.settingsHeader}>
          <div className={styles.headerContent}>
            <span className={styles.headerIcon}>⚙️</span>
            <h2 className={styles.headerTitle}>Settings</h2>
          </div>
        </div>

        <div className={styles.settingsContent}>
          <div className={styles.settingsSection}>
            <h3 className={styles.sectionTitle}>
              <span className={styles.sectionIcon}>🔑</span>
              API Keys
            </h3>
            <div className={styles.sectionDivider} />
            {isLoadingUser ? (
              <div className={styles.loadingState}>
                <div className={styles.spinner}>🔄</div>
                <p>Loading user profile...</p>
              </div>
            ) : userId ? (
              <ApiKeyManagement userId={userId} />
            ) : publicKey ? (
              <div className={styles.errorState}>
                <p className={styles.errorIcon}>⚠️</p>
                <p>User profile not found</p>
                <p className={styles.errorHint}>Your wallet is connected but profile didn't load.</p>
                <button
                  className={styles.refreshButton}
                  onClick={() => {
                    localStorage.removeItem('userProfile');
                    localStorage.removeItem('userProfileTimestamp');
                    window.location.reload();
                  }}
                >
                  🔄 Refresh Page
                </button>
              </div>
            ) : (
              <div className={styles.noUser}>
                <p>Please connect your wallet to manage API keys</p>
              </div>
            )}
          </div>
        </div>

        <div className={styles.settingsFooter}>
          <button
            className={styles.closeFooterButton}
            onClick={handleClose}
          >
            Close Settings
          </button>
        </div>
      </div>
    </div>
  );
};

export default SettingsPanel;
