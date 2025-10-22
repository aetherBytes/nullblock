import React, { useState, useEffect, useRef } from 'react';
import { useModelManagement } from '../../hooks/useModelManagement';
import { useChat } from '../../hooks/useChat';
import { useAuthentication } from '../../hooks/useAuthentication';
import { useTaskManagement } from '../../hooks/useTaskManagement';
import { useEventSystem } from '../../hooks/useEventSystem';
import { useLogs } from '../../hooks/useLogs';
import Crossroads from '../crossroads/Crossroads';
import HecateChat from './HecateChat';
import Scopes from './Scopes';
import HecateWelcome from '../crossroads/landing/HecateWelcome';
import NullblockLogo from './NullblockLogo';
import styles from './hud.module.scss';
import { Task, TaskCreationRequest } from '../../types/tasks';

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

interface HUDProps {
  publicKey: string | null;
  onDisconnect: () => void;
  onConnectWallet: (walletType?: 'phantom' | 'metamask') => void;
  theme?: Theme;
  onClose: () => void;
  onThemeChange: (theme: 'null' | 'light' | 'dark') => void;
  systemStatus: SystemStatus;
  initialTab?: 'crossroads' | 'tasks' | 'agents' | 'logs' | 'hecate' | 'canvas' | null;
}

interface AscentLevel {
  level: number;
  title: string;
  description: string;
  progress: number;
  nextLevel: number;
  nextTitle: string;
  nextDescription: string;
  accolades: string[];
}

const HUD: React.FC<HUDProps> = ({
  publicKey,
  onDisconnect,
  onConnectWallet,
  theme = 'light',
  onThemeChange,
  initialTab = null,
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
    'crossroads' | 'tasks' | 'agents' | 'logs' | 'hecate' | 'canvas' | null
  >(initialTab);
  const [showMobileMenu, setShowMobileMenu] = useState(false);
  const [showCrossroadsMarketplace, setShowCrossroadsMarketplace] = useState(false);
  const [resetCrossroadsToLanding, setResetCrossroadsToLanding] = useState(false);

  // Tab functionality state
  const [autoScroll, setAutoScroll] = useState(true);
  const [searchTerm, setSearchTerm] = useState('');
  const [logFilter, setLogFilter] = useState<'all' | 'info' | 'warning' | 'error' | 'success' | 'debug'>('all');
  const logsEndRef = useRef<HTMLDivElement>(null);

  // Model info and UI state
  const [categoryModels, setCategoryModels] = useState<any[]>([]);
  const [isLoadingCategory, setIsLoadingCategory] = useState(false);
  const [activeScope, setActiveLens] = useState<string | null>(null);
  const [showModelDropdown, setShowModelDropdown] = useState(false);
  const [modelSearchQuery, setModelSearchQuery] = useState('');
  const [isSearchingModels, setIsSearchingModels] = useState(false);
  const [searchResults, setSearchResults] = useState<any[]>([]);
  const [searchSubmitted, setSearchSubmitted] = useState(false);
  const [showSearchDropdown, setShowSearchDropdown] = useState(false);
  const modelDropdownRef = useRef<HTMLDivElement>(null);
  const searchDropdownRef = useRef<HTMLDivElement>(null);
  const [modelInfo, setModelInfo] = useState<any>(null);
  const [isLoadingModelInfo, setIsLoadingModelInfo] = useState(false);
  const [showFullDescription, setShowFullDescription] = useState(false);
  const [showModelSelection, setShowModelSelection] = useState(false);
  const [activeQuickAction, setActiveQuickAction] = useState<string | null>(null);
  const [isChatExpanded, setIsChatExpanded] = useState(false);
  const [isScopesExpanded, setIsScopesExpanded] = useState(false);
  const [showScopeDropdown, setShowScopeDropdown] = useState(false);
  const scopeDropdownRef = useRef<HTMLDivElement>(null);

  // Use custom hooks
  const chat = useChat(publicKey);
  const modelManagement = useModelManagement(publicKey, chat.activeAgent);
  const taskManagement = useTaskManagement(publicKey, {}, true, chat.addTaskNotification);
  const eventSystem = useEventSystem(true, 3000);
  const logsHook = useLogs({ autoConnect: !!publicKey, maxLogs: 500 });

  useEffect(() => {
    const hasImages = chat.chatMessages.some(msg => msg.content?.imageIds && msg.content.imageIds.length > 0);
    if (hasImages && !eventSystem.isPerformanceMode) {
      console.log('🖼️ Images detected in chat - enabling performance mode');
      eventSystem.setPerformanceMode(true);
    } else if (!hasImages && eventSystem.isPerformanceMode) {
      console.log('✅ No images in chat - disabling performance mode');
      eventSystem.setPerformanceMode(false);
    }
  }, [chat.chatMessages, eventSystem]);

  // Debug: Log task management state
  useEffect(() => {
    console.log('🔍 Task Management State Update:', {
      tasksCount: taskManagement.tasks.length,
      filteredTasksCount: taskManagement.filteredTasks.length,
      isLoading: taskManagement.isLoading,
      error: taskManagement.error,
      walletConnected: !!publicKey
    });
  }, [taskManagement.tasks, taskManagement.filteredTasks, taskManagement.isLoading, taskManagement.error, publicKey]);

  // Watch for initialTab prop changes
  useEffect(() => {
    if (initialTab !== undefined && initialTab !== mainHudActiveTab) {
      setMainHudActiveTab(initialTab);
    }
  }, [initialTab]);

  // Reset the showCrossroadsMarketplace flag after it's been used
  useEffect(() => {
    if (showCrossroadsMarketplace) {
      setShowCrossroadsMarketplace(false);
    }
  }, [showCrossroadsMarketplace]);

  // Reset the resetCrossroadsToLanding flag after it's been used
  useEffect(() => {
    if (resetCrossroadsToLanding) {
      setResetCrossroadsToLanding(false);
    }
  }, [resetCrossroadsToLanding]);

  // MCP initialization is now handled by useAuthentication hook
  useEffect(() => {
    const loadWalletData = async () => {
      if (publicKey) {
        try {
          console.log('Wallet connected:', publicKey);
        } catch (error) {
          console.error('Failed to fetch wallet data:', error);
        }
      }
    };

    loadWalletData();
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
      setIsChatExpanded(false);
      setIsScopesExpanded(false);
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
      console.log('Session started - loading full model catalog in background');
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

  // Reset expanded states when switching away from Hecate tab
  useEffect(() => {
    if (mainHudActiveTab !== 'hecate') {
      setIsChatExpanded(false);
      setIsScopesExpanded(false);
      setActiveLens(null);
    }
  }, [mainHudActiveTab]);

  // Load models when Hecate tab becomes active (use cached data)
  useEffect(() => {
    if (mainHudActiveTab === 'hecate' && publicKey) {
      if (!activeScope) {
        setActiveLens('tasks');
      }

      if (modelManagement.defaultModelReady && modelManagement.currentSelectedModel) {
        setNulleyeState('base');
      } else if (!modelManagement.defaultModelReady && !modelManagement.currentSelectedModel) {
        console.log('Tab switch triggered default model loading');
        modelManagement.loadDefaultModel();
      }
    }

    if (mainHudActiveTab === 'hecate' && publicKey && modelManagement.availableModels.length === 0 && !modelManagement.isLoadingModels && !modelManagement.modelsCached) {
      console.log('Tab switch triggered background model catalog loading');
      setTimeout(() => {
        modelManagement.loadAvailableModels();
      }, 500);
    }
  }, [mainHudActiveTab, publicKey, modelManagement.modelsCached, modelManagement.defaultModelReady, modelManagement.currentSelectedModel, activeScope]);

  // Set tasks as default scope when wallet is initially connected and Hecate tab is default
  useEffect(() => {
    if (publicKey && mainHudActiveTab === 'hecate' && !activeScope) {
      setActiveLens('tasks');
    }
  }, [publicKey, mainHudActiveTab, activeScope]);

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

  // Click outside handler for scope dropdown
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (scopeDropdownRef.current && !scopeDropdownRef.current.contains(event.target as Node)) {
        setShowScopeDropdown(false);
      }
    };

    if (showScopeDropdown) {
      document.addEventListener('mousedown', handleClickOutside);
    }

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [showScopeDropdown]);

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

  // Auto-load model info when current model changes
  useEffect(() => {
    console.log('Model info effect triggered:', { currentSelectedModel: modelManagement.currentSelectedModel, activeScope });
    if (modelManagement.currentSelectedModel && activeScope === 'modelinfo') {
      console.log('Loading model info automatically for:', modelManagement.currentSelectedModel);
      loadModelInfo(modelManagement.currentSelectedModel);
    }
  }, [modelManagement.currentSelectedModel, activeScope]);

  // Safety effect to ensure NullEye returns to base state when ready
  useEffect(() => {
    if (modelManagement.defaultModelReady && modelManagement.currentSelectedModel && publicKey && mainHudActiveTab === 'hecate') {
      if (nullviewState === 'thinking' && !modelManagement.isModelChanging && !modelManagement.isLoadingModels && !chat.isProcessingChat) {
        console.log('🔧 Forcing NullEye to base state - model ready but stuck in thinking');
        setNulleyeState('base');
      }
    }
  }, [modelManagement.defaultModelReady, modelManagement.currentSelectedModel, publicKey, mainHudActiveTab, nullviewState, modelManagement.isModelChanging, modelManagement.isLoadingModels, chat.isProcessingChat]);

  // Additional safety check - if model is ready and nothing is loading, ensure base state
  useEffect(() => {
    if (
      publicKey &&
      mainHudActiveTab === 'hecate' &&
      modelManagement.defaultModelReady &&
      modelManagement.currentSelectedModel &&
      !modelManagement.isModelChanging &&
      !modelManagement.isLoadingModels &&
      !modelManagement.defaultModelLoadingRef.current &&
      !chat.isProcessingChat &&
      nullviewState === 'thinking'
    ) {
      console.log('🚨 Emergency NullEye state reset - everything ready but stuck in thinking');
      const timer = setTimeout(() => {
        setNulleyeState('base');
      }, 500);

      return () => clearTimeout(timer);
    }
  }, [
    publicKey,
    mainHudActiveTab,
    modelManagement.defaultModelReady,
    modelManagement.currentSelectedModel,
    modelManagement.isModelChanging,
    modelManagement.isLoadingModels,
    chat.isProcessingChat,
    nullviewState
  ]);

  // Auto-load Latest models when model selection opens
  useEffect(() => {
    if (showModelSelection && activeQuickAction === 'latest' && categoryModels.length === 0 && !isLoadingCategory) {
      console.log('🔄 Auto-loading Latest models for model selection');
      loadCategoryModels('latest');
    }
  }, [showModelSelection, activeQuickAction, categoryModels.length, isLoadingCategory]);

  // Auto-focus input when Hecate tab becomes active and model is ready
  useEffect(() => {
    if (mainHudActiveTab === 'hecate' && publicKey && modelManagement.defaultModelReady && modelManagement.currentSelectedModel && !chat.isProcessingChat) {
      const timer = setTimeout(() => {
        if (chat.chatInputRef.current) {
          chat.chatInputRef.current.focus();
        }
      }, 100);

      return () => clearTimeout(timer);
    }
  }, [mainHudActiveTab, publicKey, modelManagement.defaultModelReady, modelManagement.currentSelectedModel, chat.isProcessingChat]);

  // Auto-focus input when chat is expanded
  useEffect(() => {
    if (isChatExpanded && !chat.isProcessingChat) {
      const timer = setTimeout(() => {
        if (chat.chatInputRef.current) {
          chat.chatInputRef.current.focus();
        }
      }, 100);

      return () => clearTimeout(timer);
    }
  }, [isChatExpanded, chat.isProcessingChat]);

  // Transform logs from useLogs hook to match existing log structure
  const transformedLogs = logsHook.logs.map((log) => ({
    id: `${log.timestamp}-${Math.random()}`,
    timestamp: new Date(log.timestamp),
    level: log.level,
    source: log.source,
    message: log.message,
    data: Object.keys(log.metadata).length > 0 ? log.metadata : undefined,
  }));

  // Event-driven motivation system demo
  useEffect(() => {
    if (!publicKey) return;

    // Simulate market events that drive Hecate's motivation
    const marketEventInterval = setInterval(() => {
      const events = [
        {
          type: 'price_change',
          symbol: 'ETH',
          price: `$${(2800 + Math.random() * 200).toFixed(2)}`,
          change: `${(Math.random() * 10 - 5).toFixed(2)}%`
        },
        {
          type: 'market_opportunity',
          opportunityType: 'arbitrage',
          symbol: 'BTC',
          profit: Math.random() * 2,
          exchanges: ['uniswap', 'sushiswap']
        },
        {
          type: 'threshold_breach',
          metric: 'portfolio_variance',
          value: 5.2,
          threshold: 5.0
        }
      ];

      const randomEvent = events[Math.floor(Math.random() * events.length)];

      switch (randomEvent.type) {
        case 'price_change':
          eventSystem.publishPriceChange(randomEvent.symbol, randomEvent.price, randomEvent.change);
          break;
        case 'market_opportunity':
          eventSystem.publishMarketOpportunity(randomEvent.opportunityType, {
            symbol: randomEvent.symbol,
            profit: randomEvent.profit,
            exchanges: randomEvent.exchanges,
            urgency: randomEvent.profit > 1 ? 'high' : 'medium'
          });
          break;
        case 'threshold_breach':
          eventSystem.publishThresholdBreach(randomEvent.metric, randomEvent.value, randomEvent.threshold);
          break;
      }
    }, 15000); // Every 15 seconds

    // Publish user interaction events when chat messages are sent
    const userInteractionSubscription = eventSystem.subscribe('user_interaction', (event) => {
      console.log('🧠 Hecate processing user interaction:', event.data);

      // This is where Hecate's motivation system would analyze user intent
      // and potentially create tasks based on conversation context
      if (event.data.action === 'chat_message') {
        const message = event.data.context?.message?.toLowerCase() || '';

        // Simple intent detection for demo
        if (message.includes('arbitrage') || message.includes('trading')) {
          console.log('💡 Hecate detected trading interest - suggesting arbitrage tasks');
        } else if (message.includes('portfolio') || message.includes('balance')) {
          console.log('💡 Hecate detected portfolio interest - suggesting rebalancing tasks');
        } else if (message.includes('social') || message.includes('sentiment')) {
          console.log('💡 Hecate detected social trading interest - suggesting sentiment analysis');
        }
      }
    });

    return () => {
      clearInterval(marketEventInterval);
      eventSystem.unsubscribe(userInteractionSubscription);
    };
  }, [publicKey, eventSystem.publishPriceChange, eventSystem.publishMarketOpportunity, eventSystem.publishThresholdBreach, eventSystem.subscribe, eventSystem.unsubscribe]);

  // Integrate chat system with event system
  useEffect(() => {
    // Publish user interaction events when chat messages are sent
    if (chat.chatMessages.length > 0) {
      const lastMessage = chat.chatMessages[chat.chatMessages.length - 1];
      if (lastMessage.sender === 'user') {
        eventSystem.publishUserInteraction('chat_message', {
          message: lastMessage.message,
          timestamp: lastMessage.timestamp,
          conversationLength: chat.chatMessages.length
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


  const loadModelInfo = async (modelName?: string) => {
    if (isLoadingModelInfo) {
      return;
    }

    try {
      setIsLoadingModelInfo(true);

      const { hecateAgent } = await import('../../common/services/hecate-agent');

      const connected = await hecateAgent.connect();
      if (!connected) {
        console.warn('Failed to connect to Hecate agent for model info');
        return;
      }

      const currentModelName = modelName || modelManagement.currentSelectedModel;

      if (!currentModelName) {
        console.warn('No model currently selected for model info');
        console.log('currentSelectedModel:', modelManagement.currentSelectedModel);
        console.log('modelName param:', modelName);
        setModelInfo({ error: 'No model currently selected' });
        return;
      }

      let currentModelInfo = modelManagement.availableModels?.find((model: any) => model.name === currentModelName);

      if (!currentModelInfo && modelManagement.availableModels.length === 0) {
        console.log('Cache is empty, loading models for info');
        await modelManagement.loadAvailableModels();
        currentModelInfo = modelManagement.availableModels?.find((model: any) => model.name === currentModelName);
      }

      if (!currentModelInfo) {
        setModelInfo({ error: `Model ${currentModelName} not found in available models (${modelManagement.availableModels.length} cached)` });
        return;
      }

      console.log('Model info loaded:', currentModelInfo);

      if (currentModelInfo.cost_per_1k_tokens === 0 && currentModelInfo.tier !== 'economical') {
        console.warn(`⚠️ Model ${currentModelName} shows $0 cost but tier is ${currentModelInfo.tier} - pricing may be outdated`);
      }

      if (!currentModelInfo.pricing && currentModelInfo.cost_per_1k_tokens === 0) {
        console.warn(`⚠️ Model ${currentModelName} missing pricing object and shows $0 cost`);
      }

      const enrichedModelInfo = {
        ...currentModelInfo,
        is_current: currentModelName === modelManagement.currentSelectedModel,
        info_loaded_at: new Date().toISOString()
      };

      setModelInfo(enrichedModelInfo);

    } catch (error) {
      console.error('Error loading model info:', error);
      setModelInfo({ error: (error as Error).message });
    } finally {
      setIsLoadingModelInfo(false);
    }
  };

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
    console.log('🔍 searchModels called with query:', query);

    if (!query.trim()) {
      console.log('🔍 Empty query, clearing results');
      setSearchResults([]);
      return;
    }

    try {
      setIsSearchingModels(true);

      let searchableModels = modelManagement.availableModels;
      console.log('🔍 Available models count:', searchableModels.length);

      if (searchableModels.length === 0) {
        console.log('🔍 No cached models, loading from API...');
        await modelManagement.loadAvailableModels();
        searchableModels = modelManagement.availableModels;
        console.log('🔍 Loaded models count:', searchableModels.length);
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
            providerMatch.score
          );

          const matches = nameMatch.matches || displayNameMatch.matches ||
                         descriptionMatch.matches || providerMatch.matches;

          return { model, score: maxScore, matches };
        })
        .filter((result: any) => result.matches)
        .sort((a: any, b: any) => b.score - a.score)
        .map((result: any) => result.model)
        .slice(0, 20);

      setSearchResults(results);
      console.log(`✅ Found ${results.length} models matching "${query}"`);
      if (results.length > 0) {
        console.log('🔍 Top 3 results:', results.slice(0, 3).map((m: any) => m.display_name || m.name));
      }
    } catch (error) {
      console.error('❌ Error searching models:', error);
      setSearchResults([]);
    } finally {
      setIsSearchingModels(false);
    }
  };

  const loadCategoryModels = async (category: string) => {
    if (isLoadingCategory) return;

    try {
      setIsLoadingCategory(true);
      console.log(`Filtering cached models for ${category} category`);

      let allModels = modelManagement.availableModels;

      if (allModels.length === 0) {
        console.log('No cached models available, loading first...');
        await modelManagement.loadAvailableModels();
        allModels = modelManagement.availableModels;
      }

      console.log(`Using ${allModels.length} cached models for ${category} filtering`);

      let filteredModels: any[] = [];

      switch (category) {
        case 'latest':
          filteredModels = allModels
            .filter(model => {
              if (!model || !model.available) return false;
              const hasCreatedAt = model.created_at !== undefined && model.created_at !== null;
              const hasCreated = model.created !== undefined && model.created !== null && model.created !== 0;
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

          console.log(`🔍 Latest models filtering result:`);
          console.log(`  - Total models: ${allModels.length}`);
          console.log(`  - Models with timestamps: ${allModels.filter(m => m && (m.created_at || m.created)).length}`);
          console.log(`  - Final filtered models: ${filteredModels.length}`);
          break;

        case 'free':
          filteredModels = allModels
            .filter(model => model && model.available && (model.tier === 'economical' || model.cost_per_1k_tokens === 0))
            .sort((a, b) => (a.display_name || a.name).localeCompare(b.display_name || b.name))
            .slice(0, 15);
          break;

        case 'premium':
          filteredModels = allModels
            .filter(model => model && model.available && model.tier === 'premium')
            .sort((a, b) => (a.display_name || a.name).localeCompare(b.display_name || b.name))
            .slice(0, 15);
          break;

        case 'fast':
          filteredModels = allModels
            .filter(model => model && model.available && model.tier === 'fast')
            .sort((a, b) => (a.display_name || a.name).localeCompare(b.display_name || b.name))
            .slice(0, 15);
          break;

        case 'thinkers':
          filteredModels = allModels
            .filter(model => {
              if (!model || !model.available) return false;
              const name = (model.display_name || model.name).toLowerCase();
              return (model.capabilities && (model.capabilities.includes('reasoning') || model.capabilities.includes('reasoning_tokens'))) ||
                     name.includes('reasoning') || name.includes('think') || name.includes('r1') || name.includes('o1');
            })
            .sort((a, b) => (a.display_name || a.name).localeCompare(b.display_name || b.name))
            .slice(0, 15);
          break;

        case 'instruct':
          filteredModels = allModels
            .filter(model => {
              if (!model || !model.available) return false;
              const name = (model.display_name || model.name).toLowerCase();
              return name.includes('instruct') || name.includes('it') || name.includes('chat');
            })
            .sort((a, b) => (a.display_name || a.name).localeCompare(b.display_name || b.name))
            .slice(0, 15);
          break;

        default:
          filteredModels = allModels.filter(model => model && model.available).slice(0, 15);
      }

      console.log(`Filtered to ${filteredModels.length} ${category} models`);
      setCategoryModels(filteredModels);

    } catch (error) {
      console.error(`Error filtering ${category} models:`, error);
      setCategoryModels([]);
    } finally {
      setIsLoadingCategory(false);
    }
  };

  // Legacy functions for stats display (using cached data)

  const getLatestModels = (models: any[], limit: number = 10) => {
    const filtered = models.filter(model => {
      if (!model || typeof model !== 'object') return false;
      const hasCreatedAt = model.created_at !== undefined && model.created_at !== null;
      const hasCreated = model.created !== undefined && model.created !== null && model.created !== 0;
      const isAvailable = model.available !== false;
      return (hasCreatedAt || hasCreated) && isAvailable;
    });

    if (filtered.length === 0) {
      return models.filter(model => model && model.available !== false).slice(0, limit);
    }

    const sorted = filtered.sort((a, b) => {
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
    });

    return sorted.slice(0, limit);
  };

  const getFreeModels = (models: any[], limit: number = 10) => {
    return models
      .filter(model => model.available && (model.tier === 'economical' || model.cost_per_1k_tokens === 0))
      .sort((a, b) => (a.display_name || a.name).localeCompare(b.display_name || b.name))
      .slice(0, limit);
  };

  const getFastModels = (models: any[], limit: number = 10) => {
    return models
      .filter(model => model.available && model.tier === 'fast')
      .sort((a, b) => (a.display_name || a.name).localeCompare(b.display_name || b.name))
      .slice(0, limit);
  };

  const getThinkerModels = (models: any[], limit: number = 10) => {
    return models
      .filter(model => {
        if (!model.available) return false;
        const name = (model.display_name || model.name).toLowerCase();
        return (model.capabilities && (model.capabilities.includes('reasoning') || model.capabilities.includes('reasoning_tokens'))) ||
               name.includes('reasoning') || name.includes('think') || name.includes('r1') || name.includes('o1');
      })
      .sort((a, b) => (a.display_name || a.name).localeCompare(b.display_name || b.name))
      .slice(0, limit);
  };

  const getInstructModels = (models: any[], limit: number = 10) => {
    return models
      .filter(model => {
        if (!model.available) return false;
        const name = (model.display_name || model.name).toLowerCase();
        return name.includes('instruct') || name.includes('it') || name.includes('chat');
      })
      .sort((a, b) => (a.display_name || a.name).localeCompare(b.display_name || b.name))
      .slice(0, limit);
  };

  const getImageModels = (models: any[], limit: number = 10) => {
    return models
      .filter(model => {
        if (!model.available) return false;
        return model.architecture?.output_modalities?.includes('image') ||
               (model.capabilities && model.capabilities.includes('image_generation'));
      })
      .sort((a, b) => (a.display_name || a.name).localeCompare(b.display_name || b.name))
      .slice(0, limit);
  };

  const renderTabContent = () => {
    if (!publicKey) {
      return (
        <>
          <div className={`${styles.tabWrapper} ${mainHudActiveTab === 'crossroads' || mainHudActiveTab === null ? '' : styles.hidden}`}>
            <Crossroads publicKey={publicKey} onConnectWallet={onConnectWallet} showMarketplace={showCrossroadsMarketplace} resetToLanding={resetCrossroadsToLanding} />
          </div>
          <div className={`${styles.tabWrapper} ${mainHudActiveTab === 'canvas' ? '' : styles.hidden}`}>
            <div className={styles.canvasView}>
              <div className={styles.canvasBackground} />
              <div className={styles.canvasEmpty}>
                <p className={styles.canvasMessage}>Empty Canvas</p>
                <p className={styles.canvasHint}>Click the logo to return to Hecate</p>
              </div>
            </div>
          </div>
          {mainHudActiveTab !== 'crossroads' && mainHudActiveTab !== 'canvas' && mainHudActiveTab !== null && (
            <div className={styles.defaultTab}>
              <p>Connect your wallet to access full features</p>
            </div>
          )}
        </>
      );
    } else {
      return (
        <>
          <div className={`${styles.tabWrapper} ${mainHudActiveTab === 'canvas' ? '' : styles.hidden}`}>
            <div className={styles.canvasView}>
              <div className={styles.canvasBackground} />
              <div className={styles.canvasEmpty}>
                <p className={styles.canvasMessage}>Empty Canvas</p>
                <p className={styles.canvasHint}>Click the logo to return to Hecate</p>
              </div>
            </div>
          </div>
          <div className={`${styles.tabWrapper} ${mainHudActiveTab === 'crossroads' ? '' : styles.hidden}`}>
            <Crossroads publicKey={publicKey} onConnectWallet={onConnectWallet} showMarketplace={showCrossroadsMarketplace} resetToLanding={resetCrossroadsToLanding} />
          </div>
          <div className={`${styles.tabWrapper} ${mainHudActiveTab === 'hecate' ? '' : styles.hidden}`}>
            <div className={`${styles.hecateContainer} ${isChatExpanded ? styles.chatExpanded : ''} ${isScopesExpanded ? styles.scopesExpanded : ''}`}>
              <div className={styles.hecateContent}>
                <div className={styles.hecateMain}>
                  <div className={styles.hecateInterface}>
                    <>
                      <HecateChat
                        chatMessages={chat.chatMessages}
                        chatInput={chat.chatInput}
                        setChatInput={chat.setChatInput}
                        chatInputRef={chat.chatInputRef}
                        chatMessagesRef={chat.chatMessagesRef}
                        chatEndRef={chat.chatEndRef}
                        nullviewState={nullviewState}
                        isModelChanging={modelManagement.isModelChanging}
                        isProcessingChat={chat.isProcessingChat}
                        defaultModelReady={modelManagement.defaultModelReady}
                        currentSelectedModel={modelManagement.currentSelectedModel}
                        agentHealthStatus={modelManagement.agentHealthStatus}
                        isChatExpanded={isChatExpanded}
                        setIsChatExpanded={setIsChatExpanded}
                        isScopesExpanded={isScopesExpanded}
                        setIsScopesExpanded={setIsScopesExpanded}
                        activeScope={activeScope}
                        setActiveLens={setActiveLens}
                        onChatSubmit={(e) => chat.handleChatSubmit(e, modelManagement.isModelChanging, nullviewState, modelManagement.defaultModelReady, modelManagement.currentSelectedModel, (state: string) => setNulleyeState(state as any))}
                        onChatInputChange={chat.handleChatInputChange}
                        onChatScroll={chat.handleChatScroll}
                        scrollToBottom={chat.scrollToBottom}
                        isUserScrolling={chat.isUserScrolling}
                        chatAutoScroll={chat.chatAutoScroll}
                        activeAgent={chat.activeAgent}
                        setActiveAgent={chat.setActiveAgent}
                        getImagesForMessage={chat.getImagesForMessage}
                      />

                      <Scopes
                          activeScope={activeScope}
                          setActiveLens={setActiveLens}
                          isScopesExpanded={isScopesExpanded}
                          setIsScopesExpanded={setIsScopesExpanded}
                          isChatExpanded={isChatExpanded}
                          setIsChatExpanded={setIsChatExpanded}
                          showScopeDropdown={showScopeDropdown}
                          setShowScopeDropdown={setShowScopeDropdown}
                          scopeDropdownRef={scopeDropdownRef}
                          nullviewState={nullviewState}
                          tasks={taskManagement.filteredTasks}
                          taskManagement={taskManagement}
                          logs={transformedLogs}
                          searchTerm={searchTerm}
                          setSearchTerm={setSearchTerm}
                          logFilter={logFilter}
                          setLogFilter={(filter: string) => setLogFilter(filter as any)}
                          autoScroll={autoScroll}
                          setAutoScroll={setAutoScroll}
                          logsEndRef={logsEndRef}
                          theme={theme}
                          onThemeChange={onThemeChange}
                          isLoadingModelInfo={isLoadingModelInfo}
                          modelInfo={modelInfo}
                          currentSelectedModel={modelManagement.currentSelectedModel}
                          availableModels={modelManagement.availableModels}
                          defaultModelLoaded={modelManagement.defaultModelLoaded}
                          showModelSelection={showModelSelection}
                          setShowModelSelection={setShowModelSelection}
                          setActiveQuickAction={setActiveQuickAction}
                          setModelsCached={modelManagement.setModelsCached}
                          loadAvailableModels={modelManagement.loadAvailableModels}
                          showFullDescription={showFullDescription}
                          setShowFullDescription={setShowFullDescription}
                          modelSearchQuery={modelSearchQuery}
                          setModelSearchQuery={setModelSearchQuery}
                          isSearchingModels={isSearchingModels}
                          searchResults={searchResults}
                          searchSubmitted={searchSubmitted}
                          setSearchSubmitted={setSearchSubmitted}
                          showSearchDropdown={showSearchDropdown}
                          setShowSearchDropdown={setShowSearchDropdown}
                          searchDropdownRef={searchDropdownRef}
                          activeQuickAction={activeQuickAction}
                          categoryModels={categoryModels}
                          isLoadingCategory={isLoadingCategory}
                          setCategoryModels={setCategoryModels}
                          loadCategoryModels={loadCategoryModels}
                          handleModelSelection={modelManagement.handleModelSelection}
                          getFreeModels={getFreeModels}
                          getFastModels={getFastModels}
                          getThinkerModels={getThinkerModels}
                          getInstructModels={getInstructModels}
                          getImageModels={getImageModels}
                          getLatestModels={getLatestModels}
                          activeAgent={chat.activeAgent}
                          setActiveAgent={chat.setActiveAgent}
                        />
                    </>
                  </div>
                </div>
              </div>
            </div>
          </div>
          {mainHudActiveTab !== 'canvas' && mainHudActiveTab !== 'crossroads' && mainHudActiveTab !== 'hecate' && (
            <div className={styles.defaultTab}>
              <p>Select a tab to view content</p>
            </div>
          )}
        </>
      );
    }
  };

  const renderUnifiedNavigation = () => (
    <div className={styles.unifiedNavbar}>
      {/* Left side - Logo, NULLBLOCK Text, and Welcome Quote */}
      <div className={styles.navbarLeft}>
        <NullblockLogo
          state={nullviewState}
          theme={theme}
          size="medium"
          onClick={() => {
            setMainHudActiveTab(null);
            setShowCrossroadsMarketplace(false);
            setResetCrossroadsToLanding(true);
          }}
          title="Return to Landing Page"
        />
        <div
          className={styles.nullblockTextLogo}
          onClick={() => {
            setMainHudActiveTab(null);
            setShowCrossroadsMarketplace(false);
            setResetCrossroadsToLanding(true);
          }}
          style={{ cursor: 'pointer' }}
          title="Return to Landing Page"
        >
          NULLBLOCK
        </div>
        <HecateWelcome compact={true} maxChars={80} />
      </div>

      {/* Center - Empty for spacer */}
      <div className={styles.navbarCenter}>
      </div>

      {/* Right side - All Buttons */}
      <div className={styles.navbarRight}>
        <button
          className={`${styles.menuButton} ${mainHudActiveTab === 'crossroads' ? styles.active : ''}`}
          onClick={() => {
            setMainHudActiveTab('crossroads');
            setShowCrossroadsMarketplace(true);
          }}
          title="Crossroads Marketplace"
        >
          <span>CROSSROADS</span>
        </button>

        {publicKey && (
          <button
            className={`${styles.menuButton} ${styles.fadeIn} ${mainHudActiveTab === 'hecate' ? styles.active : ''}`}
            onClick={() => setMainHudActiveTab('hecate')}
            title="Hecate Agent Interface"
          >
            <span>HECATE</span>
          </button>
        )}

        <button
          className={`${styles.walletMenuButton} ${publicKey ? styles.connected : ''}`}
          onClick={publicKey ? onDisconnect : () => onConnectWallet()}
          title={publicKey ? 'Disconnect Wallet' : 'Connect Wallet'}
        >
          <span className={styles.walletMenuText}>{publicKey ? 'Disconnect' : 'Connect'}</span>
        </button>
      </div>
    </div>
  );

  const renderMainContent = () => (
    <div className={styles.mainContent}>
      {renderTabContent()}
    </div>
  );

  return (
    <div className={`${styles.echoContainer} ${publicKey ? styles[theme] : styles.loggedOut}`}>
      {renderUnifiedNavigation()}
      {renderMainContent()}
    </div>
  );
};

export default HUD;