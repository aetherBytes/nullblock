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
  const [moreDropdownOpen, setMoreDropdownOpen] = useState(false);
  const settingsRef = useRef<HTMLDivElement>(null);
  const moreDropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    setWelcomeVisible(showWelcome);
  }, [showWelcome]);

  // Close settings dropdown when clicking outside
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

  // Close more dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (moreDropdownRef.current && !moreDropdownRef.current.contains(event.target as Node)) {
        setMoreDropdownOpen(false);
      }
    };

    if (moreDropdownOpen) {
      document.addEventListener('mousedown', handleClickOutside);
      return () => {
        document.removeEventListener('mousedown', handleClickOutside);
      };
    }
  }, [moreDropdownOpen]);

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
        {/* Settings Menu */}
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

      {/* Quick access buttons (top-left) */}
      <div className={styles.quickAccess}>
        {/* Top row: Logo and buttons */}
        <div className={styles.navRow}>
          {/* NULLBLOCK Logo */}
          <NullblockLogo
            state="base"
            theme="dark"
            size="medium"
            onClick={onResetToVoid}
            title="Return to Void"
          />
          {/* NULLBLOCK Text */}
          <div
            className={styles.nullblockTextLogo}
            onClick={onResetToVoid}
            title="Return to Void"
          >
            NULLBLOCK
          </div>

          {/* MEM CACHE - Standalone first button */}
          <button
            className={styles.quickButton}
            onClick={() => onTabSelect('memcache')}
            title="The Mem Cache - Your Engrams"
          >
            <span className={styles.buttonIcon}>◈</span>
            <span className={styles.buttonLabel}>Mem Cache</span>
          </button>

          {/* MORE - Dropdown with Crossroads and Studio */}
          <div className={styles.moreDropdownContainer} ref={moreDropdownRef}>
            <button
              className={`${styles.quickButton} ${styles.dropdownTrigger} ${moreDropdownOpen ? styles.quickButtonActive : ''}`}
              onClick={() => setMoreDropdownOpen(!moreDropdownOpen)}
              title="More options"
            >
              <span className={styles.buttonLabel}>More</span>
              <span className={styles.dropdownArrow}>{moreDropdownOpen ? '▴' : '▾'}</span>
            </button>

            {moreDropdownOpen && (
              <div className={styles.moreDropdown}>
                <button
                  className={styles.dropdownItem}
                  onClick={() => {
                    onTabSelect('crossroads');
                    setMoreDropdownOpen(false);
                  }}
                >
                  <span className={styles.dropdownIcon}>⬡</span>
                  <span>Crossroads</span>
                </button>
                <button
                  className={`${styles.dropdownItem} ${hecatePanelOpen ? styles.active : ''}`}
                  onClick={() => {
                    onHecateToggle?.(!hecatePanelOpen);
                    setMoreDropdownOpen(false);
                  }}
                >
                  <span className={styles.dropdownIcon}>{hecatePanelOpen ? '⬢' : '⬡'}</span>
                  <span>Studio</span>
                </button>
              </div>
            )}
          </div>

          {/* Description text - inline after buttons */}
          {!hecatePanelOpen && (
            <span className={styles.navDescriptionInline}>
              Remember in Mem Cache. Discover in Crossroads. Compose in Studio.
            </span>
          )}
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
