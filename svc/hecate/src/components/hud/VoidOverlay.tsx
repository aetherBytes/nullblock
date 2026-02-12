import React, { useState, useEffect, useRef, useMemo } from 'react';
import { useWalletTools } from '../../common/hooks/useWalletTools';
import { NULLBLOCK_SERVICE_COWS } from '../../constants/nullblock';
import type { MemCacheSection } from '../memcache';
import type { CrossroadsSection } from './hud';
import NullblockLogo from './NullblockLogo';
import styles from './VoidOverlay.module.scss';

const DEV_SHOW_ALL_COW_TABS = true;

const BASE_MEMCACHE_ITEMS: { id: MemCacheSection; icon: string; label: string }[] = [
  { id: 'engrams', icon: '◈', label: 'Engrams' },
  { id: 'stash', icon: '⬡', label: 'Stash' },
  { id: 'agents', icon: '◉', label: 'Agents' },
  { id: 'consensus', icon: '⚖', label: 'Consensus' },
  { id: 'model', icon: '◎', label: 'Model' },
];

const CROSSROADS_ITEMS: { id: CrossroadsSection; label: string }[] = [
  { id: 'hype', label: 'Hype' },
  { id: 'agents', label: 'Agents' },
  { id: 'tools', label: 'Tools' },
  { id: 'cows', label: 'COWs' },
];

interface VoidOverlayProps {
  onOpenSynapse: () => void;
  onTabSelect: (tab: 'crossroads' | 'memcache') => void;
  onDisconnect: () => void;
  onConnectWallet?: () => void;
  onResetToVoid?: () => void;
  showWelcome?: boolean;
  onDismissWelcome?: () => void;
  publicKey?: string | null;
  activeTab?: 'crossroads' | 'memcache' | null;
  memcacheSection?: MemCacheSection;
  onMemcacheSectionChange?: (section: MemCacheSection) => void;
  crossroadsSection?: CrossroadsSection;
  onCrossroadsSectionChange?: (section: CrossroadsSection) => void;
  onEnterCrossroads?: () => void;
  pendingCrossroadsTransition?: boolean;
}

const VoidOverlay: React.FC<VoidOverlayProps> = ({
  onOpenSynapse,
  onTabSelect,
  onDisconnect,
  onConnectWallet,
  onResetToVoid,
  showWelcome = false,
  onDismissWelcome,
  publicKey,
  activeTab,
  memcacheSection = 'engrams',
  onMemcacheSectionChange,
  crossroadsSection = 'hype',
  onCrossroadsSectionChange,
  onEnterCrossroads,
  pendingCrossroadsTransition = false,
}) => {
  const [welcomeVisible, setWelcomeVisible] = useState(showWelcome);
  const [welcomeFading, setWelcomeFading] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const settingsRef = useRef<HTMLDivElement>(null);
  const memcacheRef = useRef<HTMLDivElement>(null);

  const { unlockedTabs } = useWalletTools(publicKey || null, { autoFetch: true });

  const MEMCACHE_ITEMS = useMemo(() => {
    const items = [...BASE_MEMCACHE_ITEMS];
    const insertIndex = 3;

    NULLBLOCK_SERVICE_COWS.forEach((cow) => {
      const isUnlocked = unlockedTabs.includes(cow.id) || DEV_SHOW_ALL_COW_TABS;

      if (isUnlocked) {
        items.splice(insertIndex, 0, {
          id: cow.id as MemCacheSection,
          icon: cow.menuIcon,
          label: cow.name,
        });
      }
    });

    return items;
  }, [unlockedTabs]);

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
          variant="color"
          onClick={onResetToVoid}
          title="Return to Void"
        />
        <div className={styles.nullblockTextLogo} onClick={onResetToVoid} title="Return to Void">
          NULLBLOCK
        </div>
        {!publicKey && (
          <>
            <span className={styles.navbarDivider} />
            <span className={styles.navbarTagline}>Picks and shovels for the new age.</span>
          </>
        )}
      </div>

      {/* Top-right container: Nav + Settings */}
      <div className={styles.topRightContainer}>
        {publicKey && (
          <div className={styles.navWrapper} ref={memcacheRef}>
            {activeTab === 'memcache' && (
              <div className={styles.submenuExtra}>
                {MEMCACHE_ITEMS.slice(2)
                  .reverse()
                  .map((item, index) => (
                    <React.Fragment key={item.id}>
                      <button
                        className={`${styles.submenuItemExtra} ${memcacheSection === item.id ? styles.submenuItemActive : ''}`}
                        onClick={() => onMemcacheSectionChange?.(item.id)}
                        style={{ animationDelay: `${(MEMCACHE_ITEMS.length - 2 - index) * 0.03}s` }}
                      >
                        {item.label}
                      </button>
                      {index < MEMCACHE_ITEMS.length - 3 && <span className={styles.navDivider} />}
                    </React.Fragment>
                  ))}
                <span className={styles.navDivider} />
              </div>
            )}

            {activeTab === 'crossroads' && (
              <div className={styles.submenuExtra}>
                {CROSSROADS_ITEMS.slice(2)
                  .reverse()
                  .map((item, index) => (
                    <React.Fragment key={item.id}>
                      <button
                        className={`${styles.submenuItemExtra} ${crossroadsSection === item.id ? styles.submenuItemActive : ''}`}
                        onClick={() => onCrossroadsSectionChange?.(item.id)}
                        style={{ animationDelay: `${(CROSSROADS_ITEMS.length - 2 - index) * 0.03}s` }}
                      >
                        {item.label}
                      </button>
                      {index < CROSSROADS_ITEMS.length - 3 && <span className={styles.navDivider} />}
                    </React.Fragment>
                  ))}
                <span className={styles.navDivider} />
              </div>
            )}

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

              {activeTab === 'memcache' && (
                <>
                  {MEMCACHE_ITEMS.slice(0, 2).map((item, index) => (
                    <React.Fragment key={item.id}>
                      <button
                        className={`${styles.submenuItem} ${memcacheSection === item.id ? styles.submenuItemActive : ''}`}
                        onClick={() => onMemcacheSectionChange?.(item.id)}
                        style={{ animationDelay: `${index * 0.03}s` }}
                      >
                        {item.label}
                      </button>
                      {index < 1 && <span className={styles.submenuDivider} />}
                    </React.Fragment>
                  ))}
                </>
              )}

              {activeTab === 'crossroads' && (
                <>
                  {CROSSROADS_ITEMS.slice(0, 2).map((item, index) => (
                    <React.Fragment key={item.id}>
                      <button
                        className={`${styles.submenuItem} ${crossroadsSection === item.id ? styles.submenuItemActive : ''}`}
                        onClick={() => onCrossroadsSectionChange?.(item.id)}
                        style={{ animationDelay: `${index * 0.03}s` }}
                      >
                        {item.label}
                      </button>
                      {index < 1 && <span className={styles.submenuDivider} />}
                    </React.Fragment>
                  ))}
                </>
              )}
            </nav>
          </div>
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
          <>
            <button
              className={`${styles.crossroadsNavButton} ${pendingCrossroadsTransition ? styles.transitioning : ''}`}
              onClick={onEnterCrossroads}
              disabled={pendingCrossroadsTransition}
            >
              {pendingCrossroadsTransition ? 'Aligning...' : 'Crossroads'}
            </button>
            <button className={styles.connectButton} onClick={onConnectWallet} title="Connect Wallet">
              Connect
            </button>
          </>
        )}
      </div>

      {/* Footer bar (pre-login) */}
      {!publicKey && (
        <div className={styles.footerBar}>
          <div className={styles.footerLeft}>
            <a
              href="https://aetherbytes.github.io/nullblock-sdk/"
              target="_blank"
              rel="noopener noreferrer"
              className={styles.footerLink}
            >
              📚 Documentation
            </a>
            <a
              href="https://x.com/Nullblock_io"
              target="_blank"
              rel="noopener noreferrer"
              className={styles.footerLink}
            >
              𝕏 Follow Updates
            </a>
          </div>
          <span className={styles.footerTagline}>
            Discover agents, tools, and workflows. Own the tools that own the future.
          </span>
        </div>
      )}

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
