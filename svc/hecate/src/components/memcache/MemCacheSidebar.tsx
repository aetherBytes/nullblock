import React from 'react';
import styles from './memcache.module.scss';

export type MemCacheSection =
  | 'engrams'
  | 'stash'
  | 'tasks'
  | 'listings'
  | 'earnings'
  | 'connections'
  | 'bookmarks'
  | 'arbfarm';

interface MemCacheSidebarProps {
  activeSection: MemCacheSection;
  onSectionChange: (section: MemCacheSection) => void;
  isOpen?: boolean;
  onClose?: () => void;
}

const SIDEBAR_ITEMS: { id: MemCacheSection; icon: string; label: string }[] = [
  { id: 'engrams', icon: '◈', label: 'Engrams' },
  { id: 'stash', icon: '⬡', label: 'Stash' },
  { id: 'tasks', icon: '▣', label: 'Active Tasks' },
  { id: 'arbfarm', icon: '⚡', label: 'ArbFarm' },
  { id: 'listings', icon: '◇', label: 'Listings' },
  { id: 'earnings', icon: '◆', label: 'Earnings' },
  { id: 'connections', icon: '○', label: 'Connections' },
  { id: 'bookmarks', icon: '☆', label: 'Bookmarks' },
];

const MemCacheSidebar: React.FC<MemCacheSidebarProps> = ({
  activeSection,
  onSectionChange,
  isOpen = true,
  onClose,
}) => {
  const handleItemClick = (section: MemCacheSection) => {
    onSectionChange(section);

    if (onClose) {
      onClose();
    }
  };

  return (
    <>
      {isOpen && <div className={styles.sidebarOverlay} onClick={onClose} />}
      <aside className={`${styles.memcacheSidebar} ${isOpen ? styles.sidebarOpen : ''}`}>
        <nav className={styles.sidebarNav}>
          {SIDEBAR_ITEMS.map((item) => (
            <button
              key={item.id}
              className={`${styles.sidebarItem} ${activeSection === item.id ? styles.sidebarItemActive : ''}`}
              onClick={() => handleItemClick(item.id)}
            >
              <span className={styles.sidebarIcon}>{item.icon}</span>
              <span className={styles.sidebarLabel}>{item.label}</span>
            </button>
          ))}
        </nav>
      </aside>
    </>
  );
};

export default MemCacheSidebar;
