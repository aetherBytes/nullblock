import React, { useState, useEffect } from 'react';
import type { MemCacheSection } from '../memcache';
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
  publicKey?: string | null;
  activeTab?: 'crossroads' | 'memcache' | null;
  memcacheSection?: MemCacheSection;
  onMemcacheSectionChange?: (section: MemCacheSection) => void;
}

const VoidOverlay: React.FC<VoidOverlayProps> = ({
  onResetToVoid,
  showWelcome = false,
  onDismissWelcome,
}) => {
  const [welcomeVisible, setWelcomeVisible] = useState(showWelcome);
  const [welcomeFading, setWelcomeFading] = useState(false);

  useEffect(() => {
    setWelcomeVisible(showWelcome);
  }, [showWelcome]);

  const handleDismissWelcome = () => {
    setWelcomeFading(true);
    setTimeout(() => {
      setWelcomeVisible(false);
      setWelcomeFading(false);
      onDismissWelcome?.();
    }, 500);
  };

  return (
    <>
      <div className={styles.navbarBorder} />

      <div className={styles.logoContainer}>
        <NullblockLogo
          state="base"
          theme="dark"
          size="medium"
          onClick={onResetToVoid}
          title="Return to Void"
        />
        <div className={styles.nullblockTextLogo} onClick={onResetToVoid} title="Return to Void">
          NULLBLOCK
        </div>
      </div>

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
