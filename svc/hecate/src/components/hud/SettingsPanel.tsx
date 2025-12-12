import React, { useEffect } from 'react';
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
  useEffect(() => {
    if (isOpen) {
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
        onClose();
      }
    };

    document.addEventListener('keydown', handleEscape);
    return () => {
      document.removeEventListener('keydown', handleEscape);
    };
  }, [isOpen, onClose]);

  if (!isOpen) {
    return null;
  }

  return (
    <div className={styles.settingsOverlay} onClick={onClose}>
      <div
        className={styles.settingsPanel}
        onClick={(e) => e.stopPropagation()}
      >
        <div className={styles.settingsHeader}>
          <div className={styles.headerContent}>
            <span className={styles.headerIcon}>⚙️</span>
            <h2 className={styles.headerTitle}>Settings</h2>
          </div>
          <button
            className={styles.closeButton}
            onClick={onClose}
            title="Close Settings"
          >
            ✕
          </button>
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
      </div>
    </div>
  );
};

export default SettingsPanel;
