import React, { useState, useEffect, useRef, useMemo } from 'react';
import { useApiKeyCheck } from '../../hooks/useApiKeyCheck';
import { useAuthentication as _useAuthentication } from '../../hooks/useAuthentication';
import { useChat } from '../../hooks/useChat';
import { useEventSystem } from '../../hooks/useEventSystem';
import { useLogs } from '../../hooks/useLogs';
import { useModelManagement } from '../../hooks/useModelManagement';
import { useTaskManagement } from '../../hooks/useTaskManagement';
import { useUserProfile } from '../../hooks/useUserProfile';
import type { Task as _Task, TaskCreationRequest as _TaskCreationRequest } from '../../types/tasks';
import type { UserProfile as _UserProfile } from '../../types/user';
import Crossroads from '../crossroads/Crossroads';
import type { MemCacheSection } from '../memcache';
import { MemCache } from '../memcache';

export type CrossroadsSection = 'hype' | 'marketplace' | 'agents' | 'tools' | 'cows';
import SettingsPanel from './SettingsPanel';
import VoidOverlay from './VoidOverlay';
import styles from './hud.module.scss';

type Theme = 'null' | 'light' | 'dark';

interface SystemStatus {
  hud: boolean;
  mcp: boolean;
  orchestration: boolean;
  agents: boolean;
  portfolio: boolean;
  defi: boolean;
  social: boolean;
  arbitrage: boolean;
  hecate: boolean;
  erebus: boolean;
}

// Login animation phases from home/index.tsx
type LoginAnimationPhase = 'idle' | 'black' | 'stars' | 'background' | 'navbar' | 'complete';

interface HUDProps {
  publicKey: string | null;
  onDisconnect: () => void;
  onConnectWallet: (walletType?: 'phantom' | 'metamask') => void;
  theme?: Theme;
  onClose: () => void;
  onThemeChange: (theme: 'null' | 'light' | 'dark') => void;
  systemStatus: SystemStatus;
  initialTab?: 'crossroads' | 'memcache' | 'tasks' | 'agents' | 'logs' | 'canvas' | null;
  onToggleMobileMenu?: () => void;
  loginAnimationPhase?: LoginAnimationPhase;
  onActiveTabChange?: (
    tab: 'crossroads' | 'memcache' | 'tasks' | 'agents' | 'logs' | 'canvas' | null,
  ) => void;
  onEnterCrossroads?: () => void;
  pendingCrossroadsTransition?: boolean;
}


const HUD: React.FC<HUDProps> = ({
  publicKey,
  onDisconnect,
  onConnectWallet,
  theme = 'light',
  onThemeChange: _onThemeChange,
  initialTab = null,
  onToggleMobileMenu,
  loginAnimationPhase = 'idle',
  onActiveTabChange,
  onEnterCrossroads,
  pendingCrossroadsTransition = false,
}) => {
  const [nullviewState, setNulleyeState] = useState<
    | 'base'
    | 'response'
    | 'question'
    | 'thinking'
    | 'alert'
    | 'error'
    | 'warning'
    | 'success'
    | 'processing'
    | 'idle'
  >('base');
  const [mainHudActiveTab, setMainHudActiveTab] = useState<
    'crossroads' | 'memcache' | 'tasks' | 'agents' | 'logs' | 'canvas' | null
  >(initialTab);
  const [showMobileMenu, setShowMobileMenu] = useState(false);
  const [resetCrossroadsToLanding, setResetCrossroadsToLanding] = useState(false);
  const [showSettingsPanel, setShowSettingsPanel] = useState(false);
  const [memcacheSection, setMemcacheSection] = useState<MemCacheSection>('arbfarm');
  const [crossroadsSection, setCrossroadsSection] = useState<CrossroadsSection>('hype');

  // Void mode state
  const [hasSeenVoidWelcome, setHasSeenVoidWelcome] = useState(() => {
    if (typeof window !== 'undefined') {
      return localStorage.getItem('nullblock_void_welcome_seen') === 'true';
    }

    return false;
  });

  // Detect if we're in void mode (logged in but no tab selected)
  // Note: We check for void mode during ALL animation phases to prevent old HUD from flickering in
  const inVoidMode = Boolean(publicKey) && mainHudActiveTab === null;

  // User profile state
  const { userProfile, isLoading: isLoadingUser } = useUserProfile(publicKey);
  const { hasApiKeys } = useApiKeyCheck(userProfile?.id || null);

  // Tab functionality state
  const [autoScroll, _setAutoScroll] = useState(true);
  const [_searchTerm, _setSearchTerm] = useState('');
  const [_logFilter, _setLogFilter] = useState<
    'all' | 'info' | 'warning' | 'error' | 'success' | 'debug'
  >('all');
  const logsEndRef = useRef<HTMLDivElement>(null);

  // Model info and UI state
  const [categoryModels, setCategoryModels] = useState<any[]>([]);
  const [isLoadingCategory, setIsLoadingCategory] = useState(false);
  const [showModelDropdown, setShowModelDropdown] = useState(false);
  const [modelSearchQuery, setModelSearchQuery] = useState('');
  const [_isSearchingModels, setIsSearchingModels] = useState(false);
  const [_searchResults, setSearchResults] = useState<any[]>([]);
  const [_searchSubmitted, setSearchSubmitted] = useState(false);
  const [showSearchDropdown, setShowSearchDropdown] = useState(false);
  const modelDropdownRef = useRef<HTMLDivElement>(null);
  const searchDropdownRef = useRef<HTMLDivElement>(null);

  const [_showFullDescription, _setShowFullDescription] = useState(false);
  const [showModelSelection, setShowModelSelection] = useState(false);
  const [activeQuickAction, _setActiveQuickAction] = useState<string | null>(null);

  // Use custom hooks
  const chat = useChat(publicKey);
  const modelManagement = useModelManagement(publicKey, chat.activeAgent);
  const taskManagement = useTaskManagement(publicKey, {}, true, chat.addTaskNotification);
  const eventSystem = useEventSystem(true, 3000);
  const logsHook = useLogs({ autoConnect: Boolean(publicKey), maxLogs: 500 });

  // Create task management interface for MemCache
  const taskManagementInterface = useMemo(
    () => ({
      tasks: taskManagement.tasks,
      isLoading: taskManagement.isLoading,
      createTask: taskManagement.createTask,
      startTask: taskManagement.startTask,
      pauseTask: taskManagement.pauseTask,
      resumeTask: taskManagement.resumeTask,
      cancelTask: taskManagement.cancelTask,
      retryTask: taskManagement.retryTask,
      processTask: taskManagement.processTask,
      deleteTask: taskManagement.deleteTask,
    }),
    [taskManagement],
  );

  useEffect(() => {
    const hasImages = chat.chatMessages.some(
      (msg) => msg.content?.imageIds && msg.content.imageIds.length > 0,
    );

    if (hasImages && !eventSystem.isPerformanceMode) {
      eventSystem.setPerformanceMode(true);
    } else if (!hasImages && eventSystem.isPerformanceMode) {
      eventSystem.setPerformanceMode(false);
    }
  }, [chat.chatMessages, eventSystem]);

  useEffect(() => {
    if (initialTab !== undefined && initialTab !== mainHudActiveTab) {
      setMainHudActiveTab(initialTab);

      if (initialTab === 'crossroads' && onToggleMobileMenu) {
        setShowMobileMenu(true);
      }
    }
  }, [initialTab, onToggleMobileMenu]);

  // Reset the resetCrossroadsToLanding flag after it's been used
  useEffect(() => {
    if (resetCrossroadsToLanding) {
      setResetCrossroadsToLanding(false);
    }
  }, [resetCrossroadsToLanding]);

  // Notify parent of active tab changes
  useEffect(() => {
    onActiveTabChange?.(mainHudActiveTab);
  }, [mainHudActiveTab, onActiveTabChange]);

  // MCP initialization is now handled by useAuthentication hook
  useEffect(() => {
    // Wallet connection state synced via publicKey prop
  }, [publicKey]);

  // Initialize Hecate specific functionality
  useEffect(() => {
    if (!publicKey) {
      chat.setChatMessages([]);
      modelManagement.setAvailableModels([]);
      modelManagement.setCurrentSelectedModel(null);
      modelManagement.setDefaultModelLoaded(false);
      modelManagement.setDefaultModelReady(false);
      modelManagement.setIsLoadingModels(false);
      modelManagement.setLastStatusMessageModel(null);
      modelManagement.isLoadingModelsRef.current = false;
      modelManagement.setModelsCached(false);
      setNulleyeState('base');

      return;
    }

    chat.setChatMessages([]);

    if (!modelManagement.defaultModelReady && !modelManagement.currentSelectedModel) {
      setNulleyeState('thinking');
    } else {
      setNulleyeState('base');
    }

    modelManagement.loadDefaultModel();

    if (!modelManagement.modelsCached) {
      setTimeout(() => {
        modelManagement.loadAvailableModels();
      }, 500);
    }

    // Auto-scroll disabled to prevent forced scrolling
    // setTimeout(() => {
    //   if (chat.chatEndRef.current) {
    //     chat.chatEndRef.current.scrollIntoView({ behavior: 'smooth' });
    //   }
    // }, 100);

    return () => {
      if (chat.userScrollTimeoutRef.current) {
        clearTimeout(chat.userScrollTimeoutRef.current);
      }
    };
  }, [publicKey, modelManagement.modelsCached]);

  // Auto-close settings panel when navigating away or disconnecting
  useEffect(() => {
    if (showSettingsPanel) {
      setShowSettingsPanel(false);
    }
  }, [mainHudActiveTab, publicKey]);

  // Auto-close mobile menu when settings panel opens
  useEffect(() => {
    if (showSettingsPanel && showMobileMenu) {
      setShowMobileMenu(false);
    }
  }, [showSettingsPanel]);

  // Click outside handler for model dropdown
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (modelDropdownRef.current && !modelDropdownRef.current.contains(event.target as Node)) {
        setShowModelDropdown(false);
        setModelSearchQuery('');
        setSearchResults([]);
      }
    };

    if (showModelDropdown) {
      document.addEventListener('mousedown', handleClickOutside);
    }

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [showModelDropdown]);

  // Debounced search effect for autocomplete
  useEffect(() => {
    const timeoutId = setTimeout(() => {
      if (modelSearchQuery.trim()) {
        searchModels(modelSearchQuery);
        setShowSearchDropdown(true);
      } else {
        setSearchResults([]);
        setShowSearchDropdown(false);
        setSearchSubmitted(false);
      }
    }, 300);

    return () => clearTimeout(timeoutId);
  }, [modelSearchQuery]);

  // Click outside handler for search dropdown
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (searchDropdownRef.current && !searchDropdownRef.current.contains(event.target as Node)) {
        setShowSearchDropdown(false);
      }
    };

    if (showSearchDropdown) {
      document.addEventListener('mousedown', handleClickOutside);
    }

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [showSearchDropdown]);

  // Safety effect to ensure NullEye returns to base state when ready
  useEffect(() => {
    if (
      publicKey &&
      modelManagement.defaultModelReady &&
      modelManagement.currentSelectedModel &&
      !modelManagement.isModelChanging &&
      !modelManagement.isLoadingModels &&
      !modelManagement.defaultModelLoadingRef.current &&
      !chat.isProcessingChat &&
      nullviewState === 'thinking'
    ) {
      // Delay slightly to avoid race conditions
      const timer = setTimeout(() => {
        setNulleyeState('base');
      }, 500);

      return () => clearTimeout(timer);
    }
  }, [
    publicKey,
    modelManagement.defaultModelReady,
    modelManagement.currentSelectedModel,
    modelManagement.isModelChanging,
    modelManagement.isLoadingModels,
    chat.isProcessingChat,
    nullviewState,
  ]);

  // Auto-load Latest models when model selection opens
  useEffect(() => {
    if (
      showModelSelection &&
      activeQuickAction === 'latest' &&
      categoryModels.length === 0 &&
      !isLoadingCategory
    ) {
      loadCategoryModels('latest');
    }
  }, [showModelSelection, activeQuickAction, categoryModels.length, isLoadingCategory]);

  // Transform logs from useLogs hook to match existing log structure
  const transformedLogs = logsHook.logs.map((log) => ({
    id: `${log.timestamp}-${Math.random()}`,
    timestamp: new Date(log.timestamp),
    level: log.level,
    source: log.source,
    message: log.message,
    data: Object.keys(log.metadata).length > 0 ? log.metadata : undefined,
  }));

  // Integrate chat system with event system
  useEffect(() => {
    // Publish user interaction events when chat messages are sent
    if (chat.chatMessages.length > 0) {
      const lastMessage = chat.chatMessages[chat.chatMessages.length - 1];

      if (lastMessage.sender === 'user') {
        eventSystem.publishUserInteraction('chat_message', {
          message: lastMessage.message,
          timestamp: lastMessage.timestamp,
          conversationLength: chat.chatMessages.length,
        });
      }
    }
  }, [chat.chatMessages, eventSystem.publishUserInteraction]);

  // Auto-scroll effect for logs
  useEffect(() => {
    if (autoScroll && logsEndRef.current) {
      logsEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [transformedLogs, autoScroll]);

  // Auto-scroll effect for chat messages - DISABLED to prevent forced scrolling
  // Users can manually scroll to bottom using the scroll button
  // useEffect(() => {
  //   if (chat.chatAutoScroll && !chat.isUserScrolling && chat.chatEndRef.current) {
  //     chat.chatEndRef.current.scrollIntoView({ behavior: 'smooth' });
  //   }
  // }, [chat.chatMessages, chat.chatAutoScroll, chat.isUserScrolling]);


  const fuzzyMatch = (text: string, query: string): { matches: boolean; score: number } => {
    const textLower = text.toLowerCase();
    const queryLower = query.toLowerCase();

    if (textLower.includes(queryLower)) {
      return { matches: true, score: 100 - textLower.indexOf(queryLower) };
    }

    let queryIndex = 0;
    let lastMatchIndex = -1;
    let score = 0;

    for (let i = 0; i < textLower.length && queryIndex < queryLower.length; i++) {
      if (textLower[i] === queryLower[queryIndex]) {
        if (lastMatchIndex === -1 || i === lastMatchIndex + 1) {
          score += 10;
        } else {
          score += 5;
        }

        lastMatchIndex = i;
        queryIndex++;
      }
    }

    const allMatched = queryIndex === queryLower.length;

    return { matches: allMatched, score: allMatched ? score : 0 };
  };

  const searchModels = async (query: string) => {
    if (!query.trim()) {
      setSearchResults([]);

      return;
    }

    try {
      setIsSearchingModels(true);

      let searchableModels = modelManagement.availableModels;

      if (searchableModels.length === 0) {
        await modelManagement.loadAvailableModels();
        searchableModels = modelManagement.availableModels;
      }

      const results = searchableModels
        .map((model: any) => {
          const nameMatch = fuzzyMatch(model.name || '', query);
          const displayNameMatch = fuzzyMatch(model.display_name || '', query);
          const descriptionMatch = fuzzyMatch(model.description || '', query);
          const providerMatch = fuzzyMatch(model.provider || '', query);

          const maxScore = Math.max(
            nameMatch.score,
            displayNameMatch.score,
            descriptionMatch.score,
            providerMatch.score,
          );

          const matches =
            nameMatch.matches ||
            displayNameMatch.matches ||
            descriptionMatch.matches ||
            providerMatch.matches;

          return { model, score: maxScore, matches };
        })
        .filter((result: any) => result.matches)
        .sort((a: any, b: any) => b.score - a.score)
        .map((result: any) => result.model)
        .slice(0, 20);

      setSearchResults(results);
    } catch (error) {
      setSearchResults([]);
    } finally {
      setIsSearchingModels(false);
    }
  };

  const loadCategoryModels = async (category: string) => {
    if (isLoadingCategory) {
      return;
    }

    try {
      setIsLoadingCategory(true);

      let allModels = modelManagement.availableModels;

      if (allModels.length === 0) {
        await modelManagement.loadAvailableModels();
        allModels = modelManagement.availableModels;
      }

      let filteredModels: any[] = [];

      switch (category) {
        case 'latest':
          filteredModels = allModels
            .filter((model) => {
              if (!model || !model.available) {
                return false;
              }

              const hasCreatedAt = model.created_at !== undefined && model.created_at !== null;
              const hasCreated =
                model.created !== undefined && model.created !== null && model.created !== 0;

              return hasCreatedAt || hasCreated;
            })
            .sort((a, b) => {
              let aCreated = a.created_at || a.created;
              let bCreated = b.created_at || b.created;

              if (typeof aCreated === 'string') {
                aCreated = new Date(aCreated).getTime();
              }

              if (typeof bCreated === 'string') {
                bCreated = new Date(bCreated).getTime();
              }

              if (isNaN(aCreated) || isNaN(bCreated)) {
                return 0;
              }

              return bCreated - aCreated;
            })
            .slice(0, 15);
          break;

        case 'free':
          filteredModels = allModels
            .filter(
              (model) =>
                model &&
                model.available &&
                (model.tier === 'economical' || model.cost_per_1k_tokens === 0),
            )
            .sort((a, b) => (a.display_name || a.name).localeCompare(b.display_name || b.name))
            .slice(0, 15);
          break;

        case 'premium':
          filteredModels = allModels
            .filter((model) => model && model.available && model.tier === 'premium')
            .sort((a, b) => (a.display_name || a.name).localeCompare(b.display_name || b.name))
            .slice(0, 15);
          break;

        case 'fast':
          filteredModels = allModels
            .filter((model) => model && model.available && model.tier === 'fast')
            .sort((a, b) => (a.display_name || a.name).localeCompare(b.display_name || b.name))
            .slice(0, 15);
          break;

        case 'thinkers':
          filteredModels = allModels
            .filter((model) => {
              if (!model || !model.available) {
                return false;
              }

              const name = (model.display_name || model.name).toLowerCase();

              return (
                (model.capabilities &&
                  (model.capabilities.includes('reasoning') ||
                    model.capabilities.includes('reasoning_tokens'))) ||
                name.includes('reasoning') ||
                name.includes('think') ||
                name.includes('r1') ||
                name.includes('o1')
              );
            })
            .sort((a, b) => (a.display_name || a.name).localeCompare(b.display_name || b.name))
            .slice(0, 15);
          break;

        case 'instruct':
          filteredModels = allModels
            .filter((model) => {
              if (!model || !model.available) {
                return false;
              }

              const name = (model.display_name || model.name).toLowerCase();

              return name.includes('instruct') || name.includes('it') || name.includes('chat');
            })
            .sort((a, b) => (a.display_name || a.name).localeCompare(b.display_name || b.name))
            .slice(0, 15);
          break;

        default:
          filteredModels = allModels.filter((model) => model && model.available).slice(0, 15);
      }

      setCategoryModels(filteredModels);
    } catch (error) {
      setCategoryModels([]);
    } finally {
      setIsLoadingCategory(false);
    }
  };


  const getFreeModels = (models: any[], limit: number = 10) =>
    models
      .filter(
        (model) =>
          model.available && (model.tier === 'economical' || model.cost_per_1k_tokens === 0),
      )
      .sort((a, b) => (a.display_name || a.name).localeCompare(b.display_name || b.name))
      .slice(0, limit);

  const getFastModels = (models: any[], limit: number = 10) =>
    models
      .filter((model) => model.available && model.tier === 'fast')
      .sort((a, b) => (a.display_name || a.name).localeCompare(b.display_name || b.name))
      .slice(0, limit);

  const getThinkerModels = (models: any[], limit: number = 10) =>
    models
      .filter((model) => {
        if (!model.available) {
          return false;
        }

        const name = (model.display_name || model.name).toLowerCase();

        return (
          (model.capabilities &&
            (model.capabilities.includes('reasoning') ||
              model.capabilities.includes('reasoning_tokens'))) ||
          name.includes('reasoning') ||
          name.includes('think') ||
          name.includes('r1') ||
          name.includes('o1')
        );
      })
      .sort((a, b) => (a.display_name || a.name).localeCompare(b.display_name || b.name))
      .slice(0, limit);


  const getImageModels = (models: any[], limit: number = 10) =>
    models
      .filter((model) => {
        if (!model.available) {
          return false;
        }

        return (
          model.architecture?.output_modalities?.includes('image') ||
          (model.capabilities && model.capabilities.includes('image_generation'))
        );
      })
      .sort((a, b) => (a.display_name || a.name).localeCompare(b.display_name || b.name))
      .slice(0, limit);

  // Create model management interface for MemCache
  const modelManagementInterface = useMemo(
    () => ({
      isLoadingModelInfo: modelManagement.isLoadingModels,
      currentSelectedModel: modelManagement.currentSelectedModel,
      availableModels: modelManagement.availableModels,
      showModelSelection,
      setShowModelSelection,
      handleModelSelection: modelManagement.handleModelSelection,
      loadAvailableModels: modelManagement.loadAvailableModels,
      getFreeModels,
      getFastModels,
      getThinkerModels,
      getImageModels,
    }),
    [
      modelManagement.isLoadingModels,
      modelManagement.currentSelectedModel,
      modelManagement.availableModels,
      showModelSelection,
      modelManagement.handleModelSelection,
      modelManagement.loadAvailableModels,
    ],
  );

  const renderTabContent = () => {
    if (!publicKey) {
      return (
        <>
          <div
            className={`${styles.tabWrapper} ${mainHudActiveTab === 'crossroads' ? '' : styles.hidden}`}
          >
            <Crossroads
              publicKey={publicKey}
              onConnectWallet={onConnectWallet}
              crossroadsSection={crossroadsSection}
              resetToLanding={resetCrossroadsToLanding}
              animationPhase={loginAnimationPhase}
            />
          </div>
          <div
            className={`${styles.tabWrapper} ${mainHudActiveTab === 'canvas' ? '' : styles.hidden}`}
          >
            <div className={styles.canvasView}>
              <div className={styles.canvasBackground} />
              <div className={styles.canvasEmpty}>
                <p className={styles.canvasMessage}>Empty Canvas</p>
                <p className={styles.canvasHint}>Click the logo to return to Hecate</p>
              </div>
            </div>
          </div>
          {mainHudActiveTab !== 'crossroads' &&
            mainHudActiveTab !== 'canvas' &&
            mainHudActiveTab !== null && (
              <div className={styles.defaultTab}>
                <p>Connect your wallet to access full features</p>
              </div>
            )}
        </>
      );
    }

    return (
      <>
        <div
          className={`${styles.tabWrapper} ${mainHudActiveTab === 'canvas' ? '' : styles.hidden}`}
        >
          <div className={styles.canvasView}>
            <div className={styles.canvasBackground} />
            <div className={styles.canvasEmpty}>
              <p className={styles.canvasMessage}>Empty Canvas</p>
              <p className={styles.canvasHint}>Click the logo to return to Hecate</p>
            </div>
          </div>
        </div>
        <div
          className={`${styles.tabWrapper} ${mainHudActiveTab === 'crossroads' ? '' : styles.hidden}`}
        >
          <Crossroads
            publicKey={publicKey}
            onConnectWallet={onConnectWallet}
            crossroadsSection={crossroadsSection}
            resetToLanding={resetCrossroadsToLanding}
            animationPhase={loginAnimationPhase}
          />
        </div>
        <div
          className={`${styles.tabWrapper} ${mainHudActiveTab === 'memcache' ? '' : styles.hidden}`}
        >
          <MemCache
            publicKey={publicKey}
            activeSection={memcacheSection}
            taskManagement={taskManagementInterface}
            modelManagement={modelManagementInterface}
            availableModels={modelManagement.availableModels}
            activeAgent={chat.activeAgent}
            setActiveAgent={chat.setActiveAgent}
            hasApiKey={hasApiKeys === true}
          />
        </div>
        {mainHudActiveTab !== 'canvas' &&
          mainHudActiveTab !== 'crossroads' &&
          mainHudActiveTab !== 'memcache' &&
          !inVoidMode && (
            <div className={styles.defaultTab}>
              <p>Select a tab to view content</p>
            </div>
          )}
      </>
    );
  };

  const handleTabSelect = (tab: 'crossroads' | 'memcache') => {
    if (mainHudActiveTab === tab) {
      setMainHudActiveTab(null);
    } else {
      setMainHudActiveTab(tab);
    }
  };


  const handleResetToVoid = () => {
    setMainHudActiveTab(null);
    setResetCrossroadsToLanding(true);
    setCrossroadsSection('hype');
    setShowMobileMenu(false);
  };

  // Handle void overlay actions
  const handleOpenSynapse = () => {
    setShowSettingsPanel(true);
  };

  const handleDismissVoidWelcome = () => {
    setHasSeenVoidWelcome(true);

    if (typeof window !== 'undefined') {
      localStorage.setItem('nullblock_void_welcome_seen', 'true');
    }
  };

  const renderMainContent = () => (
    <div className={styles.mainContent}>
      {renderTabContent()}
      <SettingsPanel
        isOpen={showSettingsPanel}
        onClose={() => setShowSettingsPanel(false)}
        userId={userProfile?.id || null}
        publicKey={publicKey}
        isLoadingUser={isLoadingUser}
        userProfile={userProfile}
        onDisconnect={onDisconnect}
      />
    </div>
  );

  return (
    <div
      className={`${styles.echoContainer} ${publicKey ? styles[theme] : styles.loggedOut} ${inVoidMode ? styles.voidMode : ''}`}
    >
      {/* VoidOverlay navbar - visible during navbar phase and after */}
      {(loginAnimationPhase === 'navbar' || loginAnimationPhase === 'complete') && (
        <VoidOverlay
          onOpenSynapse={handleOpenSynapse}
          onTabSelect={handleTabSelect}
          onDisconnect={onDisconnect}
          onConnectWallet={() => onConnectWallet()}
          onResetToVoid={handleResetToVoid}
          showWelcome={inVoidMode && !hasSeenVoidWelcome}
          onDismissWelcome={handleDismissVoidWelcome}
          publicKey={publicKey}
          activeTab={
            mainHudActiveTab === 'crossroads' || mainHudActiveTab === 'memcache'
              ? mainHudActiveTab
              : null
          }
          memcacheSection={memcacheSection}
          onMemcacheSectionChange={setMemcacheSection}
          crossroadsSection={crossroadsSection}
          onCrossroadsSectionChange={setCrossroadsSection}
          onEnterCrossroads={onEnterCrossroads}
          pendingCrossroadsTransition={pendingCrossroadsTransition}
        />
      )}
      {renderMainContent()}
    </div>
  );
};

export default HUD;
