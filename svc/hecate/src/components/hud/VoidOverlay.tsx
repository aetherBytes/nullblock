import React, { useState, useEffect, useRef } from 'react';
import NullblockLogo from './NullblockLogo';
import styles from './VoidOverlay.module.scss';

interface VoidOverlayProps {
  onOpenSynapse: () => void;
  onTabSelect: (tab: 'crossroads' | 'memcache') => void;
  onDisconnect: () => void;
  onResetToVoid?: () => void;
  showWelcome?: boolean;
  onDismissWelcome?: () => void;
  hecatePanelOpen?: boolean;
  onHecateToggle?: (open: boolean) => void;
}

const VoidOverlay: React.FC<VoidOverlayProps> = ({
  onOpenSynapse,
  onTabSelect,
  onDisconnect,
  onResetToVoid,
  showWelcome = false,
  onDismissWelcome,
  hecatePanelOpen = false,
  onHecateToggle,
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
      {/* Top-right container: Settings */}
      <div className={styles.topRightContainer}>
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
      </div>

      {/* Pip-Boy style navigation */}
      <div className={styles.pipboyNav}>
        <div className={styles.pipboyFrame}>
          <div className={styles.pipboyHeader}>
            <NullblockLogo
              state="base"
              theme="dark"
              size="small"
              onClick={onResetToVoid}
              title="Return to Void"
            />
            <span className={styles.pipboyTitle} onClick={onResetToVoid}>
              NULLBLOCK
            </span>
            <span className={styles.pipboyVersion}>v2.0</span>
          </div>

          <div className={styles.pipboyScanline} />

          <div className={styles.pipboyMenu}>
            <button
              className={styles.pipboyItem}
              onClick={() => onTabSelect('memcache')}
            >
              <span className={styles.pipboyIndex}>[1]</span>
              <span className={styles.pipboyLabel}>MEM_CACHE</span>
            </button>

            <button
              className={styles.pipboyItem}
              onClick={() => onTabSelect('crossroads')}
            >
              <span className={styles.pipboyIndex}>[2]</span>
              <span className={styles.pipboyLabel}>CROSSROADS</span>
            </button>

            <button
              className={`${styles.pipboyItem} ${hecatePanelOpen ? styles.pipboyItemActive : ''}`}
              onClick={() => onHecateToggle?.(!hecatePanelOpen)}
            >
              <span className={styles.pipboyIndex}>[3]</span>
              <span className={styles.pipboyLabel}>STUDIO</span>
              {hecatePanelOpen && <span className={styles.pipboyActive}>●</span>}
            </button>
          </div>
        </div>
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
