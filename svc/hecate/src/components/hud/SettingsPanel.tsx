import React, { useEffect, useState } from 'react';
import type { UserProfile } from '../../types/user';
import ApiKeyManagement from './ApiKeyManagement';
import styles from './SettingsPanel.module.scss';

interface SettingsPanelProps {
  isOpen: boolean;
  onClose: () => void;
  userId: string | null;
  publicKey: string | null;
  isLoadingUser: boolean;
  userProfile?: UserProfile | null;
  onDisconnect?: () => void;
}

const SettingsPanel: React.FC<SettingsPanelProps> = ({
  isOpen,
  onClose,
  userId,
  publicKey,
  isLoadingUser,
  userProfile,
  onDisconnect,
}) => {
  const [isClosing, setIsClosing] = useState(false);

  // Helper functions for profile display
  const shortenAddress = (address: string): string => {
    if (address.length <= 12) {
      return address;
    }

    return `${address.slice(0, 6)}...${address.slice(-4)}`;
  };

  const formatAccountAge = (createdAt: string): string => {
    try {
      const date = new Date(createdAt);
      const month = date.toLocaleDateString('en-US', { month: 'short' });
      const year = date.getFullYear();

      return `Member since ${month} ${year}`;
    } catch (err) {
      return 'New User';
    }
  };

  const getUserTypeBadge = (userType: string): string => {
    switch (userType.toLowerCase()) {
      case 'external':
        return 'External User';
      case 'system':
        return 'System Agent';
      case 'agent':
        return 'Agent';
      case 'api':
        return 'API User';
      default:
        return userType;
    }
  };

  const isNewUser = (createdAt: string): boolean => {
    try {
      const createdDate = new Date(createdAt);
      const daysSinceCreation = Math.floor(
        (Date.now() - createdDate.getTime()) / (1000 * 60 * 60 * 24),
      );

      return daysSinceCreation < 7;
    } catch (err) {
      return false;
    }
  };

  const getBadgeText = (profile: UserProfile): string => {
    if (isNewUser(profile.created_at)) {
      return 'New User';
    }

    return getUserTypeBadge(profile.user_type);
  };

  const handleDisconnect = () => {
    onClose();
    onDisconnect?.();
  };

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
          {/* Profile Section */}
          {publicKey && (
            <div className={styles.settingsSection}>
              <h3 className={styles.sectionTitle}>
                <span className={styles.sectionIcon}>👤</span>
                Profile
              </h3>
              <div className={styles.sectionDivider} />
              <div className={styles.profileCard}>
                {isLoadingUser ? (
                  <div className={styles.profileLoading}>
                    <div className={styles.spinner}>🔄</div>
                    <span>Loading profile...</span>
                  </div>
                ) : (
                  <>
                    <div className={styles.profileInfo}>
                      <div className={styles.walletAddress}>
                        <span className={styles.addressLabel}>Wallet</span>
                        <span className={styles.addressValue}>{shortenAddress(publicKey)}</span>
                      </div>
                      {userProfile && (
                        <>
                          <div className={styles.accountAge}>
                            {formatAccountAge(userProfile.created_at)}
                          </div>
                          <span className={styles.userBadge}>{getBadgeText(userProfile)}</span>
                        </>
                      )}
                      {!userProfile && !isLoadingUser && (
                        <span className={styles.userBadge}>Guest</span>
                      )}
                    </div>
                    {onDisconnect && (
                      <button className={styles.disconnectButton} onClick={handleDisconnect}>
                        <span>🔌</span>
                        <span>Disconnect Wallet</span>
                      </button>
                    )}
                  </>
                )}
              </div>
            </div>
          )}

          {/* API Keys Section */}
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
                <p className={styles.errorHint}>
                  Your wallet is connected but profile didn't load.
                </p>
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
          <button className={styles.closeFooterButton} onClick={handleClose}>
            Close Settings
          </button>
        </div>
      </div>
    </div>
  );
};

export default SettingsPanel;
