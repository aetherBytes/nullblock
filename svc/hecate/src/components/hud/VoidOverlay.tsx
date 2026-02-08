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

type PageInfo = {
  title: string;
  subtitle: string;
};

const PAGE_INFO: Record<string, PageInfo> = {
  crossroads: {
    title: 'Picks and shovels for the new age.',
    subtitle: 'Agents are the new users. Own the tools that own the future.',
  },
  memcache: {
    title: 'MemCache',
    subtitle: 'Memory, context, and operational intelligence.',
  },
  canvas: {
    title: 'Canvas',
    subtitle: 'Open workspace.',
  },
  void: {
    title: 'Picks and shovels for the new age.',
    subtitle: 'Agents are the new users. Own the tools that own the future.',
  },
};

const VoidOverlay: React.FC<VoidOverlayProps> = ({
  onResetToVoid,
  activeTab,
}) => {
  const pageInfo = PAGE_INFO[activeTab || 'void'] || PAGE_INFO.void;

  return (
    <>
      <div className={styles.navbarBorder} />

      <div className={styles.pageTitleContainer} key={activeTab || 'void'}>
        <h1 className={styles.pageTitle}>{pageInfo.title}</h1>
        <p className={styles.pageSubtitle}>{pageInfo.subtitle}</p>
      </div>

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
