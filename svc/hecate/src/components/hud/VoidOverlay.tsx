import React, { useState, useEffect, useRef } from 'react';
import NullblockLogo from './NullblockLogo';
import styles from './VoidOverlay.module.scss';

interface VoidOverlayProps {
  onOpenSynapse: () => void;
  onTabSelect: (tab: 'crossroads' | 'memcache') => void;
  onDisconnect: () => void;
  onConnectWallet?: () => void;
  onResetToVoid?: () => void;
  showWelcome?: boolean;
  onDismissWelcome?: () => void;
  hecatePanelOpen?: boolean;
  onHecateToggle?: (open: boolean) => void;
  publicKey?: string | null;
  activeTab?: 'crossroads' | 'memcache' | null;
}

const VoidOverlay: React.FC<VoidOverlayProps> = ({
  onOpenSynapse,
  onTabSelect,
  onDisconnect,
  onConnectWallet,
  onResetToVoid,
  showWelcome = false,
  onDismissWelcome,
  hecatePanelOpen = false,
  onHecateToggle,
  publicKey,
  activeTab,
}) => {
  const [welcomeVisible, setWelcomeVisible] = useState(showWelcome);
  const [welcomeFading, setWelcomeFading] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const settingsRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    setWelcomeVisible(showWelcome);
  }, [showWelcome]);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (settingsRef.current && !settingsRef.current.contains(event.target as Node)) {
        setSettingsOpen(false);
      }
    };

    if (settingsOpen) {
      document.addEventListener('mousedown', handleClickOutside);
      return () => {
        document.removeEventListener('mousedown', handleClickOutside);
      };
    }
  }, [settingsOpen]);

  const handleDismissWelcome = () => {
    setWelcomeFading(true);
    setTimeout(() => {
      setWelcomeVisible(false);
      setWelcomeFading(false);
      onDismissWelcome?.();
    }, 500);
  };

  const handleSettingsClick = () => {
    setSettingsOpen(false);
    onOpenSynapse();
  };

  const handleDisconnectClick = () => {
    setSettingsOpen(false);
    onDisconnect();
  };

  return (
    <>
      {/* Full-width navbar border */}
      <div className={styles.navbarBorder} />

      {/* Top-left: Logo and branding */}
      <div className={styles.logoContainer}>
        <NullblockLogo
          state="base"
          theme="dark"
          size="medium"
          onClick={onResetToVoid}
          title="Return to Void"
        />
        <div
          className={styles.nullblockTextLogo}
          onClick={onResetToVoid}
          title="Return to Void"
        >
          NULLBLOCK
        </div>
      </div>

      {/* Top-right container: Nav + Settings */}
      <div className={styles.topRightContainer}>
        {/* Navigation menu - only show when logged in */}
        {publicKey && (
          <nav className={styles.voidNav}>
            <button
              className={`${styles.navItem} ${activeTab === 'memcache' ? styles.navItemActive : ''}`}
              onClick={() => onTabSelect('memcache')}
            >
              Mem Cache
            </button>
            <span className={styles.navDivider} />
            <button
              className={`${styles.navItem} ${activeTab === 'crossroads' ? styles.navItemActive : ''}`}
              onClick={() => onTabSelect('crossroads')}
            >
              Crossroads
            </button>
            <span className={styles.navDivider} />
            <button
              className={`${styles.navItem} ${hecatePanelOpen ? styles.navItemActive : ''}`}
              onClick={() => onHecateToggle?.(!hecatePanelOpen)}
            >
              Studio
            </button>
          </nav>
        )}

        {/* Settings Menu or Connect Button */}
        {publicKey ? (
          <div className={styles.settingsContainer} ref={settingsRef}>
            <button
              className={styles.settingsButton}
              onClick={() => setSettingsOpen(!settingsOpen)}
              title="Settings"
              aria-label="Open settings menu"
            >
              <svg
                width="24"
                height="24"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="1.5"
              >
                <circle cx="12" cy="12" r="3" />
                <path d="M12 2v4M12 18v4M2 12h4M18 12h4" />
                <path d="M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83" />
              </svg>
            </button>

            {settingsOpen && (
              <div className={styles.settingsDropdown}>
                <button className={styles.settingsItem} onClick={handleSettingsClick}>
                  <span className={styles.settingsIcon}>⚙️</span>
                  <span>Settings</span>
                </button>
                <div className={styles.settingsDivider} />
                <button className={styles.settingsItem} onClick={handleDisconnectClick}>
                  <span className={styles.settingsIcon}>🔌</span>
                  <span>Disconnect</span>
                </button>
              </div>
            )}
          </div>
        ) : (
          <button
            className={styles.connectButton}
            onClick={onConnectWallet}
            title="Connect Wallet"
          >
            Connect
          </button>
        )}
      </div>

      {/* First-time Welcome Overlay */}
      {welcomeVisible && (
        <div
          className={`${styles.welcomeOverlay} ${welcomeFading ? styles.fading : ''}`}
          onClick={handleDismissWelcome}
          role="button"
          tabIndex={0}
          onKeyDown={(e) => e.key === 'Enter' && handleDismissWelcome()}
          aria-label="Dismiss welcome message"
        >
          <div className={styles.welcomeContent}>
            <p className={styles.welcomeText}>You have awakened.</p>
            <p className={styles.welcomeHint}>Touch the lights or speak.</p>
            <div className={styles.welcomeDismiss}>
              <span>Click anywhere to begin</span>
            </div>
          </div>
        </div>
      )}
    </>
  );
};

export default VoidOverlay;
