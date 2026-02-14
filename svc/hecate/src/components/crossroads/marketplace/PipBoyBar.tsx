import React from 'react';
import styles from '../crossroads.module.scss';
import type { ToolCategory, CategorySummary } from '../../../types/crossroads';
import { TOOL_CATEGORY_LABELS } from '../../../types/crossroads';

type ViewType = 'nexus' | 'marketplace';
type TabType = 'tools' | 'agents' | 'protocols';

interface PipBoyBarProps {
  activeView: ViewType;
  onViewChange: (view: ViewType) => void;
  activeTab: TabType;
  onTabChange: (tab: TabType) => void;
  searchQuery: string;
  onSearchChange: (query: string) => void;
  categories: CategorySummary[];
  selectedCategory: ToolCategory | 'All';
  onCategoryChange: (category: ToolCategory | 'All') => void;
  toolCount: number;
  agentCount: number;
  protocolCount: number;
  discoveryTime: number;
  footerVisible?: boolean;
}

const PipBoyBar: React.FC<PipBoyBarProps> = ({
  activeView,
  onViewChange,
  activeTab,
  onTabChange,
  searchQuery,
  onSearchChange,
  categories,
  selectedCategory,
  onCategoryChange,
  toolCount,
  agentCount,
  protocolCount,
  discoveryTime,
  footerVisible = false,
}) => {
  const showCategories = activeView === 'marketplace' && activeTab === 'tools' && categories.length > 0;
  const barClasses = `${styles.pipboyBar} ${footerVisible ? styles.pipboyBarWithFooter : ''}`;

  return (
    <div className={barClasses}>
      <div className={styles.pipboyRow}>
        <div className={styles.pipboyViewTabs}>
          <button
            className={`${styles.pipboyViewTab} ${activeView === 'nexus' ? styles.active : ''}`}
            onClick={() => onViewChange('nexus')}
          >
            THE NEXUS
          </button>
          <button
            className={`${styles.pipboyViewTab} ${activeView === 'marketplace' ? styles.active : ''}`}
            onClick={() => onViewChange('marketplace')}
          >
            MARKETPLACE
          </button>
        </div>
        {discoveryTime > 0 && (
          <span className={styles.pipboyDiscovery}>
            Discovery: {discoveryTime}ms
          </span>
        )}
      </div>

      <div className={styles.pipboyRow}>
        <div className={styles.pipboyAssetTabs}>
          <button
            className={`${styles.pipboyAssetTab} ${activeTab === 'tools' ? styles.active : ''}`}
            onClick={() => onTabChange('tools')}
          >
            TOOLS
            {toolCount > 0 && <span className={styles.pipboyCount}>{toolCount}</span>}
          </button>
          <button
            className={`${styles.pipboyAssetTab} ${activeTab === 'agents' ? styles.active : ''}`}
            onClick={() => onTabChange('agents')}
          >
            AGENTS
            {agentCount > 0 && <span className={styles.pipboyCount}>{agentCount}</span>}
          </button>
          <button
            className={`${styles.pipboyAssetTab} ${activeTab === 'protocols' ? styles.active : ''}`}
            onClick={() => onTabChange('protocols')}
          >
            PROTOCOLS
            {protocolCount > 0 && <span className={styles.pipboyCount}>{protocolCount}</span>}
          </button>
        </div>
        <div className={styles.pipboySearch}>
          <input
            type="text"
            placeholder={activeTab === 'tools' ? '> search tools...' : `> search ${activeTab}...`}
            value={searchQuery}
            onChange={(e) => onSearchChange(e.target.value)}
          />
        </div>
      </div>

      {showCategories && (
        <div className={styles.pipboyRow}>
          <div className={styles.pipboyCategoryChips}>
            <button
              className={`${styles.pipboyCategoryChip} ${selectedCategory === 'All' ? styles.active : ''}`}
              onClick={() => onCategoryChange('All')}
            >
              All ({toolCount})
            </button>
            {categories.map((cat) => (
              <button
                key={cat.category}
                className={`${styles.pipboyCategoryChip} ${selectedCategory === cat.category ? styles.active : ''}`}
                onClick={() => onCategoryChange(cat.category)}
              >
                {TOOL_CATEGORY_LABELS[cat.category]} ({cat.count})
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};

export default PipBoyBar;
