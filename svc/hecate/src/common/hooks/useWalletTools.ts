import { useState, useEffect, useCallback } from 'react';
import { NULLBLOCK_SERVICE_COWS, type NullBlockServiceCow } from '../../constants/nullblock';

interface OwnedTool {
  id: string;
  name: string;
  toolType: 'mcp' | 'agent' | 'strategy' | 'workflow';
  cowId?: string;
  isActive: boolean;
}

interface OwnedCow {
  id: string;
  name: string;
  creatorWallet: string;
  toolCount: number;
  ownedToolCount: number;
  isFullyOwned: boolean;
}

interface UnlockProgress {
  cowId: string;
  cowName: string;
  owned: number;
  required: number;
  percent: number;
  isNullBlockService: boolean;
}

interface UseWalletToolsResult {
  ownedCows: OwnedCow[];
  ownedTools: OwnedTool[];
  unlockedTabs: string[];
  unlockProgress: UnlockProgress[];
  isLoading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
  hasUnlockedTab: (tabId: string) => boolean;
  getVisibleMenuItems: () => NullBlockServiceCow[];
}

interface UseWalletToolsOptions {
  autoFetch?: boolean;
  pollInterval?: number;
}

export function useWalletTools(
  walletAddress: string | null,
  options: UseWalletToolsOptions = {}
): UseWalletToolsResult {
  const { autoFetch = true, pollInterval = 0 } = options;

  const [ownedCows, setOwnedCows] = useState<OwnedCow[]>([]);
  const [ownedTools, setOwnedTools] = useState<OwnedTool[]>([]);
  const [unlockedTabs, setUnlockedTabs] = useState<string[]>([]);
  const [unlockProgress, setUnlockProgress] = useState<UnlockProgress[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchStashData = useCallback(async () => {
    if (!walletAddress) {
      setOwnedCows([]);
      setOwnedTools([]);
      setUnlockedTabs([]);
      setUnlockProgress([]);

      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      const response = await fetch(`/api/marketplace/wallet/${walletAddress}/stash`);

      if (!response.ok) {
        throw new Error('Failed to fetch stash data');
      }

      const data = await response.json();

      setOwnedCows(data.owned_cows || []);
      setOwnedTools(data.owned_tools || []);
      setUnlockedTabs(data.unlocked_tabs || []);
      setUnlockProgress(data.unlock_progress || []);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setIsLoading(false);
    }
  }, [walletAddress]);

  useEffect(() => {
    if (autoFetch && walletAddress) {
      fetchStashData();
    }
  }, [autoFetch, walletAddress, fetchStashData]);

  useEffect(() => {
    if (pollInterval > 0 && walletAddress) {
      const intervalId = setInterval(fetchStashData, pollInterval);

      return () => clearInterval(intervalId);
    }
  }, [pollInterval, walletAddress, fetchStashData]);

  const hasUnlockedTab = useCallback(
    (tabId: string): boolean => {
      return unlockedTabs.includes(tabId);
    },
    [unlockedTabs]
  );

  const getVisibleMenuItems = useCallback((): NullBlockServiceCow[] => {
    return NULLBLOCK_SERVICE_COWS.filter((cow) => unlockedTabs.includes(cow.id));
  }, [unlockedTabs]);

  return {
    ownedCows,
    ownedTools,
    unlockedTabs,
    unlockProgress,
    isLoading,
    error,
    refresh: fetchStashData,
    hasUnlockedTab,
    getVisibleMenuItems,
  };
}

export default useWalletTools;
