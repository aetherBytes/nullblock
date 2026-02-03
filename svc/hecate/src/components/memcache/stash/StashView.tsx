import React, { useEffect, useState } from 'react';
import {
  isNullBlockBranded,
  NULLBLOCK_SERVICE_COWS,
  type NullBlockServiceCow,
} from '../../../constants/nullblock';
import styles from './stash.module.scss';

interface OwnedTool {
  id: string;
  name: string;
  toolType: 'mcp' | 'agent' | 'strategy' | 'workflow';
  cowId?: string;
  cowName?: string;
  isActive: boolean;
  acquiredAt: string;
}

interface OwnedCow {
  id: string;
  name: string;
  description: string;
  creatorWallet: string;
  isForked: boolean;
  parentCowId?: string;
  toolCount: number;
  ownedToolCount: number;
  isFullyOwned: boolean;
  acquiredAt: string;
}

interface UnlockProgress {
  cowId: string;
  cowName: string;
  owned: number;
  required: number;
  percent: number;
  isNullBlockService: boolean;
  serviceCow?: NullBlockServiceCow;
}

interface StashViewProps {
  walletAddress: string | null;
}

type StashTab = 'overview' | 'cows' | 'tools' | 'unlocks';

const StashView: React.FC<StashViewProps> = ({ walletAddress }) => {
  const [activeTab, setActiveTab] = useState<StashTab>('overview');
  const [isLoading, setIsLoading] = useState(false);
  const [ownedCows, setOwnedCows] = useState<OwnedCow[]>([]);
  const [ownedTools, setOwnedTools] = useState<OwnedTool[]>([]);
  const [unlockProgress, setUnlockProgress] = useState<UnlockProgress[]>([]);
  const [unlockedTabs, setUnlockedTabs] = useState<string[]>([]);

  useEffect(() => {
    if (walletAddress) {
      fetchStashData();
    }
  }, [walletAddress]);

  const fetchStashData = async () => {
    if (!walletAddress) return;

    setIsLoading(true);

    try {
      const response = await fetch(`/api/marketplace/wallet/${walletAddress}/stash`);

      if (response.ok) {
        const data = await response.json();

        setOwnedCows(data.owned_cows || []);
        setOwnedTools(data.owned_tools || []);
        setUnlockProgress(data.unlock_progress || []);
        setUnlockedTabs(data.unlocked_tabs || []);
      }
    } catch (error) {
      console.error('Failed to fetch stash data:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const renderNullBlockBadge = (creatorWallet: string) => {
    if (!isNullBlockBranded(creatorWallet)) return null;

    return (
      <span className={styles.nullblockBadge} title="Official NullBlock COW">
        ⬢
      </span>
    );
  };

  const renderOverview = () => {
    const totalCows = ownedCows.length;
    const totalTools = ownedTools.length;
    const activeTools = ownedTools.filter((t) => t.isActive).length;
    const _fullyOwnedCows = ownedCows.filter((c) => c.isFullyOwned).length; void _fullyOwnedCows;
    const pendingUnlocks = unlockProgress.filter((u) => u.percent > 0 && u.percent < 100);

    return (
      <div className={styles.overviewView}>
        <div className={styles.statsGrid}>
          <div className={styles.statCard}>
            <div className={styles.statValue}>{totalCows}</div>
            <div className={styles.statLabel}>COWs Owned</div>
          </div>
          <div className={styles.statCard}>
            <div className={styles.statValue}>{totalTools}</div>
            <div className={styles.statLabel}>Tools</div>
          </div>
          <div className={styles.statCard}>
            <div className={styles.statValue}>{activeTools}</div>
            <div className={styles.statLabel}>Active</div>
          </div>
          <div className={styles.statCard}>
            <div className={styles.statValue}>{unlockedTabs.length}</div>
            <div className={styles.statLabel}>Unlocked Tabs</div>
          </div>
        </div>

        {pendingUnlocks.length > 0 && (
          <div className={styles.unlockProgressSection}>
            <h3>Unlock Progress</h3>
            <div className={styles.unlockList}>
              {pendingUnlocks.slice(0, 3).map((progress) => (
                <div key={progress.cowId} className={styles.unlockCard}>
                  <div className={styles.unlockHeader}>
                    {progress.isNullBlockService && (
                      <span className={styles.nullblockBadge}>⬢</span>
                    )}
                    <span className={styles.unlockName}>{progress.cowName}</span>
                    <span className={styles.unlockPercent}>{progress.percent}%</span>
                  </div>
                  <div className={styles.progressBar}>
                    <div
                      className={styles.progressFill}
                      style={{ width: `${progress.percent}%` }}
                    />
                  </div>
                  <div className={styles.unlockCount}>
                    {progress.owned} / {progress.required} tools
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {ownedCows.length > 0 && (
          <div className={styles.recentSection}>
            <h3>Your COWs</h3>
            <div className={styles.cowGrid}>
              {ownedCows.slice(0, 4).map((cow) => (
                <div key={cow.id} className={styles.cowCard}>
                  <div className={styles.cowHeader}>
                    {renderNullBlockBadge(cow.creatorWallet)}
                    <span className={styles.cowName}>{cow.name}</span>
                    {cow.isForked && <span className={styles.forkBadge}>Fork</span>}
                  </div>
                  <p className={styles.cowDescription}>{cow.description}</p>
                  <div className={styles.cowMeta}>
                    <span>
                      {cow.ownedToolCount}/{cow.toolCount} tools
                    </span>
                    {cow.isFullyOwned && <span className={styles.fullyOwnedBadge}>Complete</span>}
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {ownedTools.length > 0 && (
          <div className={styles.recentSection}>
            <h3>Recent Tools</h3>
            <div className={styles.toolList}>
              {ownedTools.slice(0, 5).map((tool) => (
                <div key={tool.id} className={styles.toolRow}>
                  <span className={styles.toolIcon}>
                    {tool.toolType === 'mcp' && '⚙'}
                    {tool.toolType === 'agent' && '◉'}
                    {tool.toolType === 'strategy' && '◈'}
                    {tool.toolType === 'workflow' && '⬡'}
                  </span>
                  <span className={styles.toolName}>{tool.name}</span>
                  {tool.cowName && <span className={styles.toolCow}>from {tool.cowName}</span>}
                  <span className={`${styles.toolStatus} ${tool.isActive ? styles.active : ''}`}>
                    {tool.isActive ? 'Active' : 'Inactive'}
                  </span>
                </div>
              ))}
            </div>
          </div>
        )}

        {!isLoading && ownedCows.length === 0 && ownedTools.length === 0 && (
          <div className={styles.emptyState}>
            <div className={styles.emptyIcon}>⬡</div>
            <h2>Your Stash is Empty</h2>
            <p>Browse the Crossroads to discover COWs and tools to add to your collection.</p>
          </div>
        )}
      </div>
    );
  };

  const renderCowsTab = () => (
    <div className={styles.cowsView}>
      <div className={styles.viewHeader}>
        <h2>Your COWs</h2>
        <span className={styles.count}>{ownedCows.length} total</span>
      </div>

      {ownedCows.length > 0 ? (
        <div className={styles.cowGrid}>
          {ownedCows.map((cow) => (
            <div
              key={cow.id}
              className={`${styles.cowCard} ${cow.isFullyOwned ? styles.fullyOwned : ''}`}
            >
              <div className={styles.cowHeader}>
                {renderNullBlockBadge(cow.creatorWallet)}
                <span className={styles.cowName}>{cow.name}</span>
                {cow.isForked && <span className={styles.forkBadge}>Fork</span>}
              </div>
              <p className={styles.cowDescription}>{cow.description}</p>
              <div className={styles.cowProgress}>
                <div className={styles.progressBar}>
                  <div
                    className={styles.progressFill}
                    style={{ width: `${(cow.ownedToolCount / cow.toolCount) * 100}%` }}
                  />
                </div>
                <span>
                  {cow.ownedToolCount}/{cow.toolCount} tools
                </span>
              </div>
              {cow.isFullyOwned && <div className={styles.fullyOwnedBadge}>Complete Set</div>}
            </div>
          ))}
        </div>
      ) : (
        <div className={styles.emptyState}>
          <p>No COWs in your Stash yet.</p>
        </div>
      )}
    </div>
  );

  const renderToolsTab = () => (
    <div className={styles.toolsView}>
      <div className={styles.viewHeader}>
        <h2>Your Tools</h2>
        <span className={styles.count}>{ownedTools.length} total</span>
      </div>

      {ownedTools.length > 0 ? (
        <div className={styles.toolList}>
          {ownedTools.map((tool) => (
            <div key={tool.id} className={styles.toolCard}>
              <div className={styles.toolIcon}>
                {tool.toolType === 'mcp' && '⚙'}
                {tool.toolType === 'agent' && '◉'}
                {tool.toolType === 'strategy' && '◈'}
                {tool.toolType === 'workflow' && '⬡'}
              </div>
              <div className={styles.toolInfo}>
                <span className={styles.toolName}>{tool.name}</span>
                <span className={styles.toolType}>{tool.toolType.toUpperCase()}</span>
                {tool.cowName && <span className={styles.toolCow}>Part of: {tool.cowName}</span>}
              </div>
              <div
                className={`${styles.toolStatus} ${tool.isActive ? styles.active : styles.inactive}`}
              >
                {tool.isActive ? 'Active' : 'Inactive'}
              </div>
            </div>
          ))}
        </div>
      ) : (
        <div className={styles.emptyState}>
          <p>No tools in your Stash yet.</p>
        </div>
      )}
    </div>
  );

  const renderUnlocksTab = () => {
    const nullblockUnlocks = NULLBLOCK_SERVICE_COWS.map((serviceCow) => {
      const progress = unlockProgress.find((p) => p.cowId === serviceCow.id);

      return {
        ...serviceCow,
        progress: progress || { owned: 0, required: 0, percent: 0 },
        isUnlocked: unlockedTabs.includes(serviceCow.id),
      };
    });

    return (
      <div className={styles.unlocksView}>
        <div className={styles.viewHeader}>
          <h2>COW Tab Unlocks</h2>
          <p className={styles.viewDescription}>
            Own all tools from a NullBlock COW to unlock its dedicated dashboard tab.
          </p>
        </div>

        <div className={styles.unlockGrid}>
          {nullblockUnlocks.map((unlock) => (
            <div
              key={unlock.id}
              className={`${styles.unlockCard} ${unlock.isUnlocked ? styles.unlocked : ''}`}
            >
              <div className={styles.unlockIcon}>{unlock.menuIcon}</div>
              <div className={styles.unlockInfo}>
                <h3>
                  <span className={styles.nullblockBadge}>⬢</span>
                  {unlock.name}
                </h3>
                <p>{unlock.description}</p>
              </div>
              {unlock.isUnlocked ? (
                <div className={styles.unlockedBadge}>✓ Unlocked</div>
              ) : (
                <div className={styles.progressSection}>
                  <div className={styles.progressBar}>
                    <div
                      className={styles.progressFill}
                      style={{ width: `${unlock.progress.percent}%` }}
                    />
                  </div>
                  <span className={styles.progressText}>
                    {unlock.progress.owned}/{unlock.progress.required} tools ({unlock.progress.percent}%)
                  </span>
                </div>
              )}
            </div>
          ))}
        </div>
      </div>
    );
  };

  const renderContent = () => {
    if (isLoading) {
      return (
        <div className={styles.loadingState}>
          <div className={styles.spinner} />
          <span>Loading your Stash...</span>
        </div>
      );
    }

    if (!walletAddress) {
      return (
        <div className={styles.emptyState}>
          <div className={styles.emptyIcon}>⬡</div>
          <h2>Connect Your Wallet</h2>
          <p>Connect a wallet to view your Stash.</p>
        </div>
      );
    }

    switch (activeTab) {
      case 'overview':
        return renderOverview();
      case 'cows':
        return renderCowsTab();
      case 'tools':
        return renderToolsTab();
      case 'unlocks':
        return renderUnlocksTab();
      default:
        return renderOverview();
    }
  };

  return (
    <div className={styles.stashContainer}>
      <div className={styles.stashHeader}>
        <h1>Stash</h1>
        <p>Your tools, COWs, and access rights from Crossroads</p>
      </div>

      <nav className={styles.tabNav}>
        {[
          { id: 'overview', label: 'Overview', icon: '◎' },
          { id: 'cows', label: 'COWs', icon: '⬡' },
          { id: 'tools', label: 'Tools', icon: '⚙' },
          { id: 'unlocks', label: 'Unlocks', icon: '⬢' },
        ].map((tab) => (
          <button
            key={tab.id}
            className={`${styles.tabButton} ${activeTab === tab.id ? styles.active : ''}`}
            onClick={() => setActiveTab(tab.id as StashTab)}
          >
            <span className={styles.tabIcon}>{tab.icon}</span>
            <span className={styles.tabLabel}>{tab.label}</span>
          </button>
        ))}
      </nav>

      <div className={styles.stashContent}>{renderContent()}</div>
    </div>
  );
};

export default StashView;
