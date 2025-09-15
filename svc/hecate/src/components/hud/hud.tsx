import React, { useState, useEffect, useRef } from 'react';
import { useModelManagement } from '../../hooks/useModelManagement';
import { useChat } from '../../hooks/useChat';
import { useAuthentication } from '../../hooks/useAuthentication';
import Crossroads from './Crossroads';
import HecateChat from './HecateChat';
import Scopes from './Scopes';
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

interface HUDProps {
  publicKey: string | null;
  onDisconnect: () => void;
  onConnectWallet: (walletType?: 'phantom' | 'metamask') => void;
  theme?: Theme;
  onClose: () => void;
  onThemeChange: (theme: 'null' | 'cyber' | 'light' | 'dark') => void;
  systemStatus: SystemStatus;
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
    'crossroads' | 'tasks' | 'agents' | 'logs' | 'hecate'
  >(publicKey ? 'hecate' : 'crossroads');

  // Tab functionality state
  const [tasks, setTasks] = useState<any[]>([]);
  const [logs, setLogs] = useState<any[]>([]);
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
  const modelDropdownRef = useRef<HTMLDivElement>(null);
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
  const modelManagement = useModelManagement(publicKey);
  const chat = useChat(publicKey);

  // MCP initialization is now handled by useAuthentication hook
  useEffect(() => {
    const loadWalletData = async () => {
      if (publicKey) {
        try {
          console.log('Wallet connected:', publicKey);
          const savedName = localStorage.getItem(`walletName_${publicKey}`);
          if (savedName) {
            setWalletName(savedName);
          }
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

    setTimeout(() => {
      if (chat.chatEndRef.current) {
        chat.chatEndRef.current.scrollIntoView({ behavior: 'smooth' });
      }
    }, 100);

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

  // Debounced search effect
  useEffect(() => {
    const timeoutId = setTimeout(() => {
      if (modelSearchQuery.trim()) {
        searchModels(modelSearchQuery);
      } else {
        setSearchResults([]);
      }
    }, 300);

    return () => clearTimeout(timeoutId);
  }, [modelSearchQuery]);

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

  // Initialize mock data and live updates for tabs
  useEffect(() => {
    if (!publicKey) return;

    const mockTasks = [
      {
        id: '1',
        name: 'Arbitrage Opportunity Scan',
        status: 'running',
        type: 'trading',
        description: 'Scanning DEXes for arbitrage opportunities across Uniswap V3, SushiSwap, and PancakeSwap',
        startTime: new Date(Date.now() - 300000),
        progress: 65,
        logs: [],
      },
      {
        id: '2',
        name: 'Social Sentiment Analysis',
        status: 'completed',
        type: 'agent',
        description: 'Analyzing social media sentiment for trading signals from Twitter, Reddit, and Discord',
        startTime: new Date(Date.now() - 600000),
        endTime: new Date(Date.now() - 300000),
        progress: 100,
        logs: [],
      },
      {
        id: '3',
        name: 'Portfolio Rebalancing',
        status: 'pending',
        type: 'system',
        description: 'Automated portfolio rebalancing based on market conditions and risk parameters',
        startTime: new Date(Date.now() - 100000),
        logs: [],
      },
      {
        id: '4',
        name: 'Flashbots Bundle Construction',
        status: 'running',
        type: 'mcp',
        description: 'Building MEV-protected transaction bundle for optimal execution',
        startTime: new Date(Date.now() - 150000),
        progress: 45,
        logs: [],
      },
    ];

    const mockMcpOperations = [
      {
        id: '1',
        name: 'Flashbots Bundle',
        status: 'active',
        endpoint: '/flashbots/bundle',
        lastActivity: new Date(),
        responseTime: 150,
      },
      {
        id: '2',
        name: 'MEV Protection',
        status: 'active',
        endpoint: '/mev/protect',
        lastActivity: new Date(Date.now() - 5000),
        responseTime: 89,
      },
      {
        id: '3',
        name: 'Social Trading Signals',
        status: 'idle',
        endpoint: '/social/signals',
        lastActivity: new Date(Date.now() - 30000),
      },
      {
        id: '4',
        name: 'Portfolio Analytics',
        status: 'active',
        endpoint: '/portfolio/analytics',
        lastActivity: new Date(Date.now() - 2000),
        responseTime: 234,
      },
      {
        id: '5',
        name: 'Risk Assessment',
        status: 'active',
        endpoint: '/risk/assessment',
        lastActivity: new Date(Date.now() - 1000),
        responseTime: 67,
      },
      {
        id: '6',
        name: 'Market Data Feed',
        status: 'active',
        endpoint: '/market/feed',
        lastActivity: new Date(),
        responseTime: 12,
      },
    ];

    const mockLogs = [
      {
        id: '1',
        timestamp: new Date(),
        level: 'info',
        source: 'main.js:124',
        message: 'NullView interface initialized',
        data: { component: 'HUD', loadTime: '45ms', memory: '12.4MB' }
      },
      {
        id: '2',
        timestamp: new Date(Date.now() - 1000),
        level: 'info',
        source: 'mcp-client.ts:87',
        message: 'WebSocket connection established to localhost:8001',
        data: { protocol: 'ws', latency: '23ms', status: 'connected' }
      },
      {
        id: '3',
        timestamp: new Date(Date.now() - 2000),
        level: 'success',
        source: 'arbitrage.service.ts:203',
        message: 'DEX opportunity found: ETH/USDC spread 0.23%',
        data: { pair: 'ETH/USDC', spread: '0.23%', volume: '$15,234', dex: ['Uniswap', 'SushiSwap'] }
      },
      {
        id: '4',
        timestamp: new Date(Date.now() - 3000),
        level: 'warning',
        source: 'portfolio.controller.ts:156',
        message: 'Portfolio variance exceeded threshold (5.2% > 5.0%)',
        data: { currentVariance: '5.2%', threshold: '5.0%', recommendation: 'rebalance' }
      },
      {
        id: '5',
        timestamp: new Date(Date.now() - 4000),
        level: 'error',
        source: 'social.api.ts:45',
        message: 'Failed to fetch Twitter API data',
        data: { status: 429, error: 'Rate limit exceeded', retryAfter: '15min' }
      },
    ];

    setTasks(mockTasks);
    setLogs(mockLogs);

    const interval = setInterval(() => {
      const logMessages = [
        { level: 'info', source: 'market.feed.ts:89', message: 'Price update received', data: { symbol: 'ETH/USD', price: '$2,847.32', change: '+1.2%' }},
        { level: 'success', source: 'flashbots.client.ts:203', message: 'MEV bundle included in block', data: { blockNumber: 18945672, profit: '$23.45' }},
        { level: 'warning', source: 'gas.monitor.ts:67', message: 'Gas price spike detected', data: { currentGas: '95 gwei', increase: '111%' }},
      ];

      const randomLog = logMessages[Math.floor(Math.random() * logMessages.length)];
      const newLog = {
        id: Date.now().toString(),
        timestamp: new Date(),
        level: randomLog.level,
        source: randomLog.source,
        message: randomLog.message,
        data: randomLog.data,
      };

      setLogs((prev) => [...prev, newLog]);
    }, 4000);

    const progressInterval = setInterval(() => {
      setTasks((prev) =>
        prev.map((task) => {
          if (task.status === 'running' && task.progress !== undefined && task.progress < 100) {
            return {
              ...task,
              progress: Math.min(100, task.progress + Math.random() * 5),
            };
          }
          return task;
        }),
      );
    }, 5000);

    return () => {
      clearInterval(interval);
      clearInterval(progressInterval);
    };
  }, [publicKey]);

  // Auto-scroll effect for logs
  useEffect(() => {
    if (autoScroll && logsEndRef.current) {
      logsEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [logs, autoScroll]);


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

      const queryLower = query.toLowerCase();
      const results = searchableModels.filter((model: any) => {
        const name = (model.name || '').toLowerCase();
        const displayName = (model.display_name || '').toLowerCase();
        const description = (model.description || '').toLowerCase();

        return name.includes(queryLower) ||
               displayName.includes(queryLower) ||
               description.includes(queryLower);
      }).slice(0, 20);

      setSearchResults(results);
      console.log(`Found ${results.length} models matching "${query}" in cached data`);
    } catch (error) {
      console.error('Error searching cached models:', error);
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

  const getPremiumModels = (models: any[], limit: number = 10) => {
    return models
      .filter(model => model.available && model.tier === 'premium')
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

  const renderTabContent = () => {
    if (!publicKey) {
      switch (mainHudActiveTab) {
        case 'crossroads':
          return <Crossroads publicKey={publicKey} onConnectWallet={onConnectWallet} />;
        default:
          return (
            <div className={styles.defaultTab}>
              <p>Connect your wallet to access full features</p>
            </div>
          );
      }
    } else {
      switch (mainHudActiveTab) {
        case 'crossroads':
          return <Crossroads publicKey={publicKey} onConnectWallet={onConnectWallet} />;
        case 'hecate':
          return (
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
                        isChatExpanded={isChatExpanded}
                        setIsChatExpanded={setIsChatExpanded}
                        isScopesExpanded={isScopesExpanded}
                        setIsScopesExpanded={setIsScopesExpanded}
                        activeScope={activeScope}
                        setActiveLens={setActiveLens}
                        onChatSubmit={(e) => chat.handleChatSubmit(e, modelManagement.isModelChanging, nullviewState, modelManagement.defaultModelReady, modelManagement.currentSelectedModel, (state: string) => setNulleyeState(state as any))}
                        onChatInputChange={chat.handleChatInputChange}
                        onChatScroll={chat.handleChatScroll}
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
                          tasks={tasks}
                          logs={logs}
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
                          activeQuickAction={activeQuickAction}
                          categoryModels={categoryModels}
                          isLoadingCategory={isLoadingCategory}
                          setCategoryModels={setCategoryModels}
                          loadCategoryModels={loadCategoryModels}
                          handleModelSelection={modelManagement.handleModelSelection}
                          getFreeModels={getFreeModels}
                          getFastModels={getFastModels}
                          getPremiumModels={getPremiumModels}
                          getThinkerModels={getThinkerModels}
                          getInstructModels={getInstructModels}
                          getLatestModels={getLatestModels}
                        />
                    </>
                  </div>
                </div>
              </div>
            </div>
          );
        default:
          return (
            <div className={styles.defaultTab}>
              <p>Select a tab to view content</p>
            </div>
          );
      }
    }
  };

  const renderHomeScreen = () => (
    <div className={styles.hudScreen}>
      <div className={styles.innerHudMenuBar}>
        <button
          className={`${styles.menuButton} ${mainHudActiveTab === 'crossroads' ? styles.active : ''}`}
          onClick={() => setMainHudActiveTab('crossroads')}
        >
          Crossroads
        </button>

        {publicKey && (
          <>
            <button
              className={`${styles.menuButton} ${styles.fadeIn} ${mainHudActiveTab === 'hecate' ? styles.active : ''}`}
              onClick={() => setMainHudActiveTab('hecate')}
            >
              Hecate
            </button>
          </>
        )}
      </div>
      <div className={styles.homeContent}>
        <div className={styles.landingContent}>
          <div className={styles.mainHudContent}>
            {renderTabContent()}
          </div>
        </div>
      </div>
    </div>
  );

  const renderControlScreen = () => (
    <nav className={styles.verticalNavbar}>
      <div className={styles.nullblockTitle}>
        NULLBL<span className={styles.irisO}>O</span>CK
        <div
          className={`${styles.nullview} ${styles[nullviewState]}`}
          onClick={() => {
            if (!publicKey) {
              setNulleyeState('error');
              setTimeout(() => setNulleyeState('base'), 1500);
              alert(
                '🔒 SECURE ACCESS REQUIRED\n\nConnect your Web3 wallet to unlock the NullView interface and access advanced features.',
              );
              return;
            }

            setMainHudActiveTab('hecate');
            setNulleyeState('thinking');
          }}
          title={!publicKey ? '🔒 Connect wallet to unlock NullView' : '🔓 Access NullView Interface'}
        >
          <div className={styles.pulseRing}></div>
          <div className={styles.dataStream}>
            <div className={styles.streamLine}></div>
            <div className={styles.streamLine}></div>
            <div className={styles.streamLine}></div>
          </div>
          <div className={styles.lightningContainer}>
            <div className={styles.lightningArc}></div>
            <div className={styles.lightningArc}></div>
            <div className={styles.lightningArc}></div>
            <div className={styles.lightningArc}></div>
            <div className={styles.lightningArc}></div>
            <div className={styles.lightningArc}></div>
            <div className={styles.lightningArc}></div>
            <div className={styles.lightningArc}></div>
          </div>
          <div className={styles.staticField}></div>
          <div className={styles.coreNode}></div>
        </div>
      </div>

      <div className={styles.navbarButtons}>
        <button
          className={`${styles.walletMenuButton} ${publicKey ? styles.connected : ''}`}
          onClick={publicKey ? onDisconnect : () => onConnectWallet()}
          title={publicKey ? 'Disconnect Wallet' : 'Connect Wallet'}
        >
          <span className={styles.walletMenuText}>{publicKey ? 'Disconnect' : 'Connect'}</span>
        </button>
        <button
          className={styles.docsMenuButton}
          onClick={() => window.open('https://aetherbytes.github.io/nullblock-sdk/', '_blank')}
          title="Documentation & Developer Resources"
        >
          <span className={styles.docsMenuText}>Docs</span>
        </button>
      </div>
    </nav>
  );

  return (
    <div className={`${styles.echoContainer} ${styles[theme]}`}>
      {renderControlScreen()}
      {renderHomeScreen()}
    </div>
  );
};

export default HUD;