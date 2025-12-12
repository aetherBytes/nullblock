import React, { useState, useRef, useEffect } from 'react';
import { UserProfile } from '../../types/user';
import styles from './MemoriesMenu.module.scss';

interface MemoriesMenuProps {
  publicKey: string | null;
  userProfile: UserProfile | null;
  isLoadingUser: boolean;
  onDisconnect: () => void;
  onOpenSettings: () => void;
  isMobile?: boolean;
}

const MemoriesMenu: React.FC<MemoriesMenuProps> = ({
  publicKey,
  userProfile,
  isLoadingUser,
  onDisconnect,
  onOpenSettings,
  isMobile = false,
}) => {
  const [isOpen, setIsOpen] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

  const shortenAddress = (address: string): string => {
    if (address.length <= 12) return address;
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
      const daysSinceCreation = Math.floor((Date.now() - createdDate.getTime()) / (1000 * 60 * 60 * 24));
      return daysSinceCreation < 7;
    } catch (err) {
      return false;
    }
  };

  const getBadgeText = (userProfile: UserProfile): string => {
    if (isNewUser(userProfile.created_at)) {
      return 'New User';
    }
    return getUserTypeBadge(userProfile.user_type);
  };

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    };

    if (isOpen && !isMobile) {
      document.addEventListener('mousedown', handleClickOutside);
      return () => {
        document.removeEventListener('mousedown', handleClickOutside);
      };
    }
  }, [isOpen, isMobile]);

  const handleToggle = () => {
    setIsOpen(!isOpen);
  };

  const handleSettingsClick = () => {
    setIsOpen(false);
    onOpenSettings();
  };

  const handleDisconnectClick = () => {
    setIsOpen(false);
    onDisconnect();
  };

  if (!publicKey) {
    return null;
  }

  if (isMobile) {
    return (
      <div className={styles.mobileMemoriesSection}>
        <button
          className={styles.mobileMemoriesHeader}
          onClick={handleToggle}
        >
          <span>🧠 Memories</span>
          <span className={styles.chevron}>{isOpen ? '▲' : '▼'}</span>
        </button>

        {isOpen && (
          <div className={styles.mobileMemoriesContent}>
            {isLoadingUser ? (
              <div className={styles.skeleton}>
                <div className={styles.skeletonLine} />
                <div className={styles.skeletonLine} />
              </div>
            ) : (
              <div className={styles.userInfo}>
                <p className={styles.walletAddress}>{shortenAddress(publicKey)}</p>
                {userProfile && (
                  <>
                    <p className={styles.accountAge}>{formatAccountAge(userProfile.created_at)}</p>
                    <span className={styles.userBadge}>{getBadgeText(userProfile)}</span>
                  </>
                )}
                {!userProfile && !isLoadingUser && (
                  <>
                    <p className={styles.accountAge}>Profile not loaded. Try refreshing.</p>
                    <span className={styles.userBadge}>Guest</span>
                  </>
                )}
                {!userProfile && isLoadingUser && (
                  <>
                    <p className={styles.accountAge}>Loading profile...</p>
                    <span className={styles.userBadge}>Please wait</span>
                  </>
                )}
              </div>
            )}
            <div className={styles.mobileActions}>
              <button className={styles.mobileActionBtn} onClick={handleSettingsClick}>
                ⚙️ Settings
              </button>
              <button className={styles.mobileActionBtn} onClick={handleDisconnectClick}>
                🔌 Disconnect
              </button>
            </div>
          </div>
        )}
      </div>
    );
  }

  return (
    <div className={styles.memoriesMenuContainer} ref={menuRef}>
      <button
        className={styles.memoriesButton}
        onClick={handleToggle}
        title="Memories"
      >
        <span className={styles.icon}>🧠</span>
        <span className={styles.label}>Memories</span>
        <span className={styles.chevron}>{isOpen ? '▲' : '▼'}</span>
      </button>

      {isOpen && (
        <div className={styles.memoriesDropdown}>
          {isLoadingUser ? (
            <div className={styles.skeleton}>
              <div className={styles.skeletonLine} />
              <div className={styles.skeletonLine} />
              <div className={styles.skeletonLine} />
            </div>
          ) : (
            <>
              <div className={styles.userInfo}>
                <p className={styles.walletAddress}>{shortenAddress(publicKey)}</p>
                {userProfile && (
                  <>
                    <p className={styles.accountAge}>{formatAccountAge(userProfile.created_at)}</p>
                    <span className={styles.userBadge}>{getBadgeText(userProfile)}</span>
                  </>
                )}
                {!userProfile && !isLoadingUser && (
                  <>
                    <p className={styles.accountAge}>Profile not loaded. Try refreshing.</p>
                    <span className={styles.userBadge}>Guest</span>
                  </>
                )}
                {!userProfile && isLoadingUser && (
                  <>
                    <p className={styles.accountAge}>Loading profile...</p>
                    <span className={styles.userBadge}>Please wait</span>
                  </>
                )}
              </div>

              <div className={styles.separator} />

              <button className={styles.menuItem} onClick={handleSettingsClick}>
                <span className={styles.menuIcon}>⚙️</span>
                <span className={styles.menuLabel}>Settings</span>
              </button>

              <button className={styles.menuItem} onClick={handleDisconnectClick}>
                <span className={styles.menuIcon}>🔌</span>
                <span className={styles.menuLabel}>Disconnect</span>
              </button>
            </>
          )}
        </div>
      )}
    </div>
  );
};

export default MemoriesMenu;
