import React from 'react';
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
}) => {
  return (
    <>
      <div className={styles.logoContainer}>
        <NullblockLogo
          state="base"
          theme="dark"
          size="medium"
          onClick={onResetToVoid}
          title="Return to Void"
        />
      </div>
    </>
  );
};

export default VoidOverlay;
