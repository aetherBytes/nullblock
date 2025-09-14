import React, { useState, useEffect, useRef } from 'react';
import xLogo from '../../assets/images/X_logo_black.png';
import nullviewLogo from '../../assets/images/nullview_logo.png';
import type { MissionData } from '../../common/services/api';
import {
  fetchWalletData,
  fetchUserProfile,
  fetchAscentLevel,
  fetchActiveMission,
} from '../../common/services/api';
import {
  isAuthenticated,
  restoreSession,
  createAuthChallenge,
  verifyAuthChallenge,
  checkMCPHealth,
} from '../../common/services/mcp-api';
import { hecateAgent, type ChatMessage as HecateChatMessage, type HecateResponse } from '../../common/services/hecate-agent';
import MarkdownRenderer from '../common/MarkdownRenderer';
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

interface UserProfile {
  id: string;
  ascent: number;
  nether: number | null;
  cacheValue: number;
  memories: number;
  matrix: {
    level: string;
    rarity: string;
    status: string;
  };
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
  onClose,
  onThemeChange,
  systemStatus,
}) => {
  const [walletData, setWalletData] = useState<any>(null);
  const [userProfile, setUserProfile] = useState<UserProfile>({
    id: publicKey ? `${publicKey.slice(0, 4)}...${publicKey.slice(-4)}.sol` : '',
    ascent: 1,
    nether: null,
    cacheValue: 0,
    memories: 0,
    matrix: {
      level: 'NONE',
      rarity: 'NONE',
      status: 'N/A',
    },
  });
  const [ascentLevel, setAscentLevel] = useState<AscentLevel | null>(null);
  const [walletName, setWalletName] = useState<string | null>(null);
  const [activeMission, setActiveMission] = useState<MissionData | null>(null);
  const [mcpAuthenticated, setMcpAuthenticated] = useState<boolean>(false);
  const [mcpHealthStatus, setMcpHealthStatus] = useState<any>(null);
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
  
  // Additional state needed for tab functionality
  const [tasks, setTasks] = useState<any[]>([]);
  const [mcpOperations, setMcpOperations] = useState<any[]>([]);
  const [logs, setLogs] = useState<any[]>([]);
  const [agents, setAgents] = useState<any[]>([]);
  const [autoScroll, setAutoScroll] = useState(true);
  const [searchTerm, setSearchTerm] = useState('');
  const [logFilter, setLogFilter] = useState<'all' | 'info' | 'warning' | 'error' | 'success' | 'debug'>('all');
  const logsEndRef = useRef<HTMLDivElement>(null);
  
  // Model category state for cached data
  const [categoryModels, setCategoryModels] = useState<any[]>([]);
  const [isLoadingCategory, setIsLoadingCategory] = useState(false);
  
  // Hecate specific state
  const [chatMessages, setChatMessages] = useState<any[]>([]);
  const [chatInput, setChatInput] = useState('');
  const [activeScope, setActiveLens] = useState<string | null>(null);
  const [isProcessingChat, setIsProcessingChat] = useState(false);
  const chatEndRef = useRef<HTMLDivElement>(null);
  const chatMessagesRef = useRef<HTMLDivElement>(null);
  const chatInputRef = useRef<HTMLInputElement>(null);
  const [chatAutoScroll, setChatAutoScroll] = useState(true);
  const [isUserScrolling, setIsUserScrolling] = useState(false);
  const userScrollTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const [availableModels, setAvailableModels] = useState<any[]>([]);
  const [showModelDropdown, setShowModelDropdown] = useState(false);
  const [currentSelectedModel, setCurrentSelectedModel] = useState<string | null>(null);
  const [isModelChanging, setIsModelChanging] = useState(false);
  const [modelSearchQuery, setModelSearchQuery] = useState('');
  const [isSearchingModels, setIsSearchingModels] = useState(false);
  const [searchResults, setSearchResults] = useState<any[]>([]);
  const [defaultModelLoaded, setDefaultModelLoaded] = useState(false);
  const [isLoadingModels, setIsLoadingModels] = useState(false);
  const [lastStatusMessageModel, setLastStatusMessageModel] = useState<string | null>(null);
  const modelDropdownRef = useRef<HTMLDivElement>(null);
  const isLoadingModelsRef = useRef(false);
  const defaultModelLoadingRef = useRef(false);

  // Model info state
  const [modelInfo, setModelInfo] = useState<any>(null);
  const [isLoadingModelInfo, setIsLoadingModelInfo] = useState(false);
  const [showFullDescription, setShowFullDescription] = useState(false);
  const [showModelSelection, setShowModelSelection] = useState(false);

  // Quick actions state
  const [activeQuickAction, setActiveQuickAction] = useState<string | null>(null);

  // Session-based models caching state (only refresh on page reload)
  const [modelsCached, setModelsCached] = useState(false);
  const [sessionStartTime] = useState<Date>(new Date());
  const [defaultModelReady, setDefaultModelReady] = useState(false);

  // Expand states for containers
  const [isChatExpanded, setIsChatExpanded] = useState(false);
  const [isScopesExpanded, setIsScopesExpanded] = useState(false);



  // MCP initialization effect
  useEffect(() => {
    const initializeMCP = async () => {
      try {
        // Try to restore existing session
        const hasSession = restoreSession();

        setMcpAuthenticated(hasSession && isAuthenticated());

        // Check MCP health
        const health = await checkMCPHealth();

        setMcpHealthStatus(health);
      } catch (error) {
        console.error('Failed to initialize MCP:', error);
        setMcpHealthStatus(null);
      }
    };

    initializeMCP();
  }, []);

  useEffect(() => {
    const loadWalletData = async () => {
      if (publicKey) {
        try {
          // Skip old backend wallet data fetch for now - using Erebus for wallet ops
          console.log('Wallet connected:', publicKey);
          
          // Check for saved wallet name
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

  // Initialize mock data and live updates for tabs
  useEffect(() => {
    if (!publicKey) return; // Only initialize when logged in
    
    // Mock data from HecateHud
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
    setMcpOperations(mockMcpOperations);
    setLogs(mockLogs);

    // Simulate real-time log updates
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

    // Simulate task progress updates
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

  // Initialize Hecate specific functionality
  useEffect(() => {
    if (!publicKey) {
      // Reset state when wallet disconnected
      setChatMessages([]);
      setAvailableModels([]);
      setCurrentSelectedModel(null);
      setDefaultModelLoaded(false);
      setDefaultModelReady(false);
      setIsLoadingModels(false);
      setLastStatusMessageModel(null);
      isLoadingModelsRef.current = false;
      setIsChatExpanded(false);
      setIsScopesExpanded(false);
      setModelsCached(false);
      setNulleyeState('base'); // Reset NullEye to base state
      return;
    }
    
    // Initialize with empty chat
    setChatMessages([]);
    
    // Set initial loading state when wallet connects ONLY if no model is ready
    if (!defaultModelReady && !currentSelectedModel) {
      setNulleyeState('thinking');
    } else {
      // If model is already ready, go straight to base state
      setNulleyeState('base');
    }
    
    // Load default model immediately for instant chat availability
    loadDefaultModel();
    
    // Then load full model catalog in background (only if not already cached)
    if (!modelsCached) {
      console.log('Session started - loading full model catalog in background');
      // Start loading catalog in background without blocking default model
      setTimeout(() => {
        loadAvailableModels();
      }, 500); // Reduced delay - start loading sooner but still after default
    }
    
    // Ensure we start scrolled to bottom
    setTimeout(() => {
      if (chatEndRef.current) {
        chatEndRef.current.scrollIntoView({ behavior: 'smooth' });
      }
    }, 100);

    return () => {
      if (userScrollTimeoutRef.current) {
        clearTimeout(userScrollTimeoutRef.current);
      }
    };
  }, [publicKey, modelsCached]);

  // Reset expanded states when switching away from Hecate tab
  useEffect(() => {
    if (mainHudActiveTab !== 'hecate') {
      setIsChatExpanded(false);
      setIsScopesExpanded(false);
    }
  }, [mainHudActiveTab]);

  // Load models when Hecate tab becomes active (use cached data)
  useEffect(() => {
    if (mainHudActiveTab === 'hecate' && publicKey) {
      // If model is already ready when switching to Hecate tab, ensure base state
      if (defaultModelReady && currentSelectedModel) {
        setNulleyeState('base');
      } else if (!defaultModelReady && !currentSelectedModel) {
        // Load default model immediately when switching to Hecate tab
        console.log('Tab switch triggered default model loading');
        loadDefaultModel();
      }
    }
    
    if (mainHudActiveTab === 'hecate' && publicKey && availableModels.length === 0 && !isLoadingModels && !modelsCached) {
      // Load full catalog in background
      console.log('Tab switch triggered background model catalog loading');
      setTimeout(() => {
        loadAvailableModels();
      }, 500);
    }
  }, [mainHudActiveTab, publicKey, modelsCached, defaultModelReady, currentSelectedModel]);

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
    console.log('Model info effect triggered:', { currentSelectedModel, activeScope });
    if (currentSelectedModel && activeScope === 'modelinfo') {
      console.log('Loading model info automatically for:', currentSelectedModel);
      loadModelInfo(currentSelectedModel);
    }
  }, [currentSelectedModel, activeScope]);

  // Safety effect to ensure NullEye returns to base state when ready
  useEffect(() => {
    if (defaultModelReady && currentSelectedModel && publicKey && mainHudActiveTab === 'hecate') {
      // If everything is ready but we're still in thinking state, force return to base
      // BUT NOT if we're actively processing a chat message
      if (nullviewState === 'thinking' && !isModelChanging && !isLoadingModels && !isProcessingChat) {
        console.log('🔧 Forcing NullEye to base state - model ready but stuck in thinking');
        setNulleyeState('base');
      }
    }
  }, [defaultModelReady, currentSelectedModel, publicKey, mainHudActiveTab, nullviewState, isModelChanging, isLoadingModels, isProcessingChat]);

  // Additional safety check - if model is ready and nothing is loading, ensure base state
  useEffect(() => {
    if (
      publicKey && 
      mainHudActiveTab === 'hecate' && 
      defaultModelReady && 
      currentSelectedModel && 
      !isModelChanging && 
      !isLoadingModels && 
      !defaultModelLoadingRef.current &&
      !isProcessingChat &&
      nullviewState === 'thinking'
    ) {
      console.log('🚨 Emergency NullEye state reset - everything ready but stuck in thinking');
      const timer = setTimeout(() => {
        setNulleyeState('base');
      }, 500); // Slightly longer delay for emergency reset
      
      return () => clearTimeout(timer);
    }
  }, [
    publicKey, 
    mainHudActiveTab, 
    defaultModelReady, 
    currentSelectedModel, 
    isModelChanging, 
    isLoadingModels, 
    isProcessingChat,
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
    if (mainHudActiveTab === 'hecate' && publicKey && defaultModelReady && currentSelectedModel && !isProcessingChat) {
      // Small delay to ensure the input is rendered and enabled
      const timer = setTimeout(() => {
        if (chatInputRef.current) {
          chatInputRef.current.focus();
        }
      }, 100);
      
      return () => clearTimeout(timer);
    }
  }, [mainHudActiveTab, publicKey, defaultModelReady, currentSelectedModel, isProcessingChat]);

  // Auto-focus input when chat is expanded
  useEffect(() => {
    if (isChatExpanded && !isProcessingChat) {
      const timer = setTimeout(() => {
        if (chatInputRef.current) {
          chatInputRef.current.focus();
        }
      }, 100);
      
      return () => clearTimeout(timer);
    }
  }, [isChatExpanded, isProcessingChat]);


  const handleMCPAuthentication = async () => {
    if (!publicKey) {
      alert('Please connect your wallet first');

      return;
    }

    try {
      // Create auth challenge
      const challenge = await createAuthChallenge(publicKey);

      // For Phantom wallet, sign the challenge
      if ('phantom' in window) {
        const provider = (window as any).phantom?.solana;

        if (provider) {
          const message = new TextEncoder().encode(challenge.message);
          const signedMessage = await provider.signMessage(message, 'utf8');
          const signature = Array.from(signedMessage.signature);

          // Verify the signature with MCP
          const authResponse = await verifyAuthChallenge(
            publicKey,
            signature.toString(),
            'phantom',
          );

          if (authResponse.success) {
            setMcpAuthenticated(true);
            alert('Successfully authenticated with MCP!');
          } else {
            alert(`Authentication failed: ${authResponse.message}`);
          }
        }
      }
    } catch (error) {
      console.error('MCP authentication failed:', error);
      alert('Authentication failed. Please try again.');
    }
  };

  // Helper functions for tab rendering
  const getStatusColor = (status: string) => {
    switch (status) {
      case 'running':
      case 'active':
        return styles.statusRunning;
      case 'completed':
      case 'success':
        return styles.statusCompleted;
      case 'failed':
      case 'error':
        return styles.statusFailed;
      case 'pending':
      case 'idle':
        return styles.statusPending;
      default:
        return styles.statusPending;
    }
  };

  const getLogLevelColor = (level: string) => {
    switch (level) {
      case 'error':
        return styles.logError;
      case 'warning':
        return styles.logWarning;
      case 'success':
        return styles.logSuccess;
      case 'debug':
        return styles.logDebug;
      default:
        return styles.logInfo;
    }
  };

  const filteredLogs = logs.filter((log) => {
    const matchesSearch =
      log.message.toLowerCase().includes(searchTerm.toLowerCase()) ||
      log.source.toLowerCase().includes(searchTerm.toLowerCase());
    const matchesFilter = logFilter === 'all' || log.level === logFilter;
    return matchesSearch && matchesFilter;
  });

  // Auto-scroll effect for logs
  useEffect(() => {
    if (autoScroll && logsEndRef.current) {
      logsEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [logs, autoScroll]);

  // Track previous message count for auto-scroll
  const prevMessageCountRef = useRef(0);

  // Auto-scroll effect for chat messages - only when new messages are added
  useEffect(() => {
    const currentMessageCount = chatMessages.length;
    const prevMessageCount = prevMessageCountRef.current;
    
    // Only auto-scroll if new messages were added and auto-scroll is enabled
    if (chatAutoScroll && currentMessageCount > prevMessageCount && chatEndRef.current) {
      chatEndRef.current.scrollIntoView({ behavior: 'auto' });
    }
    
    // Update the previous message count
    prevMessageCountRef.current = currentMessageCount;
  }, [chatMessages, chatAutoScroll]);

  // Helper functions for Hecate functionality
  // Load default model immediately for instant chat availability
  const loadDefaultModel = async () => {
    if (defaultModelReady || !publicKey || defaultModelLoadingRef.current) {
      // If model is already ready, ensure we're in base state
      if (defaultModelReady && currentSelectedModel) {
        setNulleyeState('base');
      }
      return;
    }
    
    defaultModelLoadingRef.current = true;

    try {
      console.log('🚀 Loading default model immediately...');
      
      const { hecateAgent } = await import('../../common/services/hecate-agent');
      
      // Ensure connection to Hecate agent
      const connected = await hecateAgent.connect();
      if (!connected) {
        console.warn('Failed to connect to Hecate agent for default model');
        setNulleyeState('error');
        setTimeout(() => setNulleyeState('base'), 3000);
        return;
      }

      // Check if a model is already set on the backend
      const status = await hecateAgent.getModelStatus();
      if (status.current_model) {
        console.log('✅ Model already loaded on backend:', status.current_model);
        setCurrentSelectedModel(status.current_model);
        setDefaultModelReady(true);
        
        // Only show message if we haven't shown it yet
        if (lastStatusMessageModel !== status.current_model) {
          const readyMessage = {
            id: Date.now().toString(),
            timestamp: new Date(),
            sender: 'hecate',
            message: `🤖 ${status.current_model} ready`,
            type: 'update'
          };
          setChatMessages(prev => [...prev, readyMessage]);
          setLastStatusMessageModel(status.current_model);
        }
        
        // Go directly to base state since model is already ready
        setNulleyeState('base');
        return;
      }

      // Only set thinking state if we actually need to load a new model
      setNulleyeState('thinking');

      // Load DeepSeek Chat as default (fastest free model)
      const defaultModelName = 'deepseek/deepseek-chat-v3.1:free';
      console.log('Loading default model:', defaultModelName);
      
      const loadingMessage = {
        id: Date.now().toString(),
        timestamp: new Date(),
        sender: 'hecate',
        message: `🚀 Loading DeepSeek Chat (fast & free)...`,
        type: 'update'
      };
      setChatMessages(prev => [...prev, loadingMessage]);
      
      const success = await hecateAgent.setModel(defaultModelName);
      if (success) {
        setCurrentSelectedModel(defaultModelName);
        setDefaultModelReady(true);
        setNulleyeState('success'); // Model loaded successfully
        
        const successMessage = {
          id: (Date.now() + 1).toString(),
          timestamp: new Date(),
          sender: 'hecate',
          message: `✅ DeepSeek Chat ready - ask me anything!`,
          type: 'update'
        };
        setChatMessages(prev => [...prev, successMessage]);
        setLastStatusMessageModel(defaultModelName);
        
        console.log('✅ Default model loaded successfully');
        
        // Return to base state after showing success
        setTimeout(() => setNulleyeState('base'), 2000);
      } else {
        console.warn('Failed to load default model');
        setNulleyeState('error'); // Show error state
        setTimeout(() => setNulleyeState('base'), 3000);
      }
    } catch (error) {
      console.error('Error loading default model:', error);
      setNulleyeState('error'); // Show error state for any failures
      setTimeout(() => setNulleyeState('base'), 3000);
    } finally {
      defaultModelLoadingRef.current = false;
    }
  };

  const getUserStats = () => {
    const sessionStart = localStorage.getItem('lastAuthTime');
    const sessionTime = sessionStart ? Date.now() - parseInt(sessionStart) : 0;
    const sessionMinutes = Math.floor(sessionTime / (1000 * 60));
    
    return {
      walletAddress: publicKey ? `${publicKey.slice(0, 6)}...${publicKey.slice(-4)}` : 'Not Connected',
      walletName: walletName || null,
      walletType: 'phantom',
      sessionDuration: sessionMinutes > 0 ? `${sessionMinutes}m` : 'Just started',
      connectionStatus: publicKey ? 'Connected' : 'Disconnected',
    };
  };

  const scopesOptions = [
    { id: 'modelinfo', icon: '🤖', title: 'Model Info', description: 'Current model details', color: '#ff6b6b' },
    { id: 'tasks', icon: '📋', title: 'Tasks', description: 'Active agent tasks', color: '#4ecdc4' },
    { id: 'agents', icon: '🤖', title: 'Agents', description: 'Agent monitoring', color: '#45b7d1' },
    { id: 'logs', icon: '📄', title: 'Logs', description: 'System logs', color: '#96ceb4' },
    { id: 'settings', icon: '⚙️', title: 'Settings', description: 'Theme & social links', color: '#747d8c' },
  ];

  const loadModelInfo = async (modelName?: string) => {
    if (isLoadingModelInfo) {
      return;
    }

    try {
      setIsLoadingModelInfo(true);

      const { hecateAgent } = await import('../../common/services/hecate-agent');
      
      // Ensure connection to Hecate agent
      const connected = await hecateAgent.connect();
      if (!connected) {
        console.warn('Failed to connect to Hecate agent for model info');
        return;
      }

      // Use cached available models instead of fetching again
      const currentModelName = modelName || currentSelectedModel;
      
      if (!currentModelName) {
        console.warn('No model currently selected for model info');
        console.log('currentSelectedModel:', currentSelectedModel);
        console.log('modelName param:', modelName);
        setModelInfo({ error: 'No model currently selected' });
        return;
      }

      // Find the current model in cached available models
      let currentModelInfo = availableModels?.find((model: any) => model.name === currentModelName);
      
      // If not found in cache and cache is empty, load models first
      if (!currentModelInfo && availableModels.length === 0) {
        console.log('Cache is empty, loading models for info');
        await loadAvailableModels();
        currentModelInfo = availableModels?.find((model: any) => model.name === currentModelName);
      }
      
      if (!currentModelInfo) {
        setModelInfo({ error: `Model ${currentModelName} not found in available models (${availableModels.length} cached)` });
        return;
      }

      // Use the rich model info from available models
      console.log('Model info loaded:', currentModelInfo);
      
      // Debug pricing information structure
      console.log('=== FULL MODEL DATA STRUCTURE ===');
      console.log('Full model info:', JSON.stringify(currentModelInfo, null, 2));
      console.log('=== PRICING ANALYSIS ===');
      console.log('Pricing object exists:', !!currentModelInfo.pricing);
      console.log('Pricing object:', currentModelInfo.pricing);
      console.log('cost_per_1k_tokens:', currentModelInfo.cost_per_1k_tokens);
      console.log('tier:', currentModelInfo.tier);
      console.log('=== END DEBUG ===');
      
      // Add debugging for cost information
      if (currentModelInfo.cost_per_1k_tokens === 0 && currentModelInfo.tier !== 'economical') {
        console.warn(`⚠️ Model ${currentModelName} shows $0 cost but tier is ${currentModelInfo.tier} - pricing may be outdated`);
      }
      
      if (!currentModelInfo.pricing && currentModelInfo.cost_per_1k_tokens === 0) {
        console.warn(`⚠️ Model ${currentModelName} missing pricing object and shows $0 cost`);
      }
      
      // Add is_current field based on current selection
      const enrichedModelInfo = {
        ...currentModelInfo,
        is_current: currentModelName === currentSelectedModel,
        // Add timestamp for when this info was loaded
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

  const loadAvailableModels = async () => {
    // Prevent concurrent executions using synchronous ref
    if (isLoadingModelsRef.current) {
      console.log('Model loading already in progress (ref guard), skipping duplicate call');
      return;
    }

    // Skip if models are already cached for this session
    if (modelsCached && availableModels.length > 0) {
      console.log('Models already cached for this session, skipping API call');
      return;
    }

    try {
      console.log('=== LOADING MODELS START ===');
      console.log('isLoadingModels:', isLoadingModels);
      console.log('isLoadingModelsRef:', isLoadingModelsRef.current);
      console.log('defaultModelLoaded:', defaultModelLoaded);
      console.log('availableModels.length:', availableModels.length);
      console.log('currentSelectedModel:', currentSelectedModel);

      // Set both the ref (synchronous) and state (for UI)
      isLoadingModelsRef.current = true;
      setIsLoadingModels(true);
      
      // Only show loading state if default model isn't ready yet
      if (!defaultModelReady && !currentSelectedModel) {
        setNulleyeState('thinking');
      }

      // Import the hecate agent service
      const { hecateAgent } = await import('../../common/services/hecate-agent');
      
      // Ensure connection to Hecate agent
      const connected = await hecateAgent.connect();
      if (!connected) {
        console.warn('Failed to connect to Hecate agent for model loading');
        return;
      }

      // Get available models
      const modelsData = await hecateAgent.getAvailableModels();
      setAvailableModels(modelsData.models || []);
      
      console.log('Available models loaded:', modelsData.models?.length || 0);
      console.log('Current model from backend:', modelsData.current_model);
      
      // Debug: Check for models with created timestamps for latest filtering
      const modelsWithTimestamps = (modelsData.models || []).filter(m => m.created || m.created_at);
      console.log(`🔍 Models with timestamps: ${modelsWithTimestamps.length} out of ${modelsData.models?.length || 0}`);
      
      // Check for specific newer models the user mentioned
      const newerModelKeywords = ['sonoma', 'qwen', 'kimi', 'dusk', 'sky'];
      const foundNewerModels = (modelsData.models || []).filter((model: any) => {
        const name = (model.display_name || model.name || '').toLowerCase();
        const id = (model.id || '').toLowerCase();
        return newerModelKeywords.some(keyword => name.includes(keyword) || id.includes(keyword));
      });
      
      console.log('🔍 Frontend check - Models matching newer keywords (sonoma, qwen, kimi, dusk, sky):', foundNewerModels.length);
      if (foundNewerModels.length > 0) {
        console.log('📋 Found newer models in frontend:');
        foundNewerModels.slice(0, 5).forEach((model: any, i) => {
          console.log(`  ${i + 1}. ${model.display_name || model.name} (${model.id})`);
          console.log(`     - created: ${model.created} (${typeof model.created})`);
          console.log(`     - created_at: ${model.created_at}`);
        });
      } else {
        console.log('⚠️ No newer models (Sonoma, Qwen, Kimi, Dusk, Sky) found in frontend response');
        console.log('📋 Sample of what we got instead (first 5):');
        (modelsData.models || []).slice(0, 5).forEach((model: any, i) => {
          console.log(`  ${i + 1}. ${model.display_name || model.name} (${model.id})`);
        });
      }
      
      if (modelsWithTimestamps.length > 0) {
        console.log('📋 Sample models with timestamps:');
        modelsWithTimestamps.slice(0, 3).forEach((model, i) => {
          console.log(`  ${i + 1}. ${model.display_name || model.name}:`);
          console.log(`     - created: ${model.created} (${typeof model.created})`);
          console.log(`     - created_at: ${model.created_at} (${typeof model.created_at})`);
          if (model.created) {
            const date = new Date(typeof model.created === 'number' ? model.created * 1000 : model.created);
            console.log(`     - date: ${date.toLocaleDateString()}`);
          }
        });
      }
      
      // If a model is already set, just update the UI state (default model may already be loaded)
      if (modelsData.current_model) {
        // Only update the selected model if it's not already set (prevents overriding default model)
        if (!currentSelectedModel) {
          setCurrentSelectedModel(modelsData.current_model);
        }
        setDefaultModelLoaded(true);
        
        console.log('=== LOADING MODELS END (model already set) ===');
        return;
      }
      
      // Models loaded but no current model - this is handled by default model loading

      console.log('=== LOADING MODELS END ===');
      
      // Mark models as cached for this session
      setModelsCached(true);
      console.log('Models successfully cached for session started at:', sessionStartTime.toISOString());
      
      // Ensure NullEye returns to proper state after models are loaded
      if (defaultModelReady && currentSelectedModel) {
        // Model is ready, go directly to base state
        setNulleyeState('base');
      } else if (!defaultModelReady && !currentSelectedModel) {
        // No model ready yet, keep current state (likely thinking from default model loading)
        console.log('Models catalog loaded but no default model ready yet');
      }
      
    } catch (error) {
      console.error('Error loading available models:', error);
      setDefaultModelLoaded(false); // Reset on error
      
      // Show error state only if default model isn't ready
      if (!defaultModelReady) {
        setNulleyeState('error');
        setTimeout(() => setNulleyeState('base'), 3000);
      }
    } finally {
      // Reset both the ref and state
      isLoadingModelsRef.current = false;
      setIsLoadingModels(false);
    }
  };

  const searchModels = async (query: string) => {
    if (!query.trim()) {
      setSearchResults([]);
      return;
    }

    try {
      setIsSearchingModels(true);
      
      // Use cached models for search instead of API call
      let searchableModels = availableModels;
      
      // If no cached models, load them first
      if (searchableModels.length === 0) {
        await loadAvailableModels();
        searchableModels = availableModels;
      }
      
      // Perform client-side search on cached models
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
      
      // Use cached models instead of making a fresh API call
      let allModels = availableModels;
      
      // If no cached models available, load them first
      if (allModels.length === 0) {
        console.log('No cached models available, loading first...');
        await loadAvailableModels();
        allModels = availableModels;
      }
      
      console.log(`Using ${allModels.length} cached models for ${category} filtering`);
      
      // Filter models based on category
      let filteredModels: any[] = [];
      
      switch (category) {
        case 'latest':
          filteredModels = allModels
            .filter(model => {
              if (!model || !model.available) return false;
              // Check for both created_at and created fields
              const hasCreatedAt = model.created_at !== undefined && model.created_at !== null;
              const hasCreated = model.created !== undefined && model.created !== null && model.created !== 0;
              return hasCreatedAt || hasCreated;
            })
            .sort((a, b) => {
              // Get the created timestamp, prioritizing created_at over created
              let aCreated = a.created_at || a.created;
              let bCreated = b.created_at || b.created;
              
              // Convert ISO date strings to timestamps for comparison
              if (typeof aCreated === 'string') {
                aCreated = new Date(aCreated).getTime();
              }
              if (typeof bCreated === 'string') {
                bCreated = new Date(bCreated).getTime();
              }
              
              // Ensure we have valid numbers
              if (isNaN(aCreated) || isNaN(bCreated)) {
                return 0; // Keep original order if timestamps are invalid
              }
              
              return bCreated - aCreated; // Newest first (higher timestamp first)
            })
            .slice(0, 15);
          
          console.log(`🔍 Latest models filtering result:`);
          console.log(`  - Total models: ${allModels.length}`);
          console.log(`  - Models with timestamps: ${allModels.filter(m => m && (m.created_at || m.created)).length}`);
          console.log(`  - Final filtered models: ${filteredModels.length}`);
          if (filteredModels.length > 0) {
            console.log('  - Top 3 results:');
            filteredModels.slice(0, 3).forEach((model, i) => {
              const timestamp = model.created_at || model.created;
              const date = typeof timestamp === 'string' ? new Date(timestamp) : new Date(timestamp);
              console.log(`    ${i + 1}. ${model.display_name}: ${timestamp} (${date.toLocaleDateString()})`);
            });
          }
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
  const getLastUpdatedModels = (models: any[], limit: number = 10) => {
    return models
      .filter(model => model.available && model.updated_at)
      .sort((a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime())
      .slice(0, limit);
  };

  const getLatestModels = (models: any[], limit: number = 10) => {
    const filtered = models.filter(model => {
      if (!model || typeof model !== 'object') return false;
      // Check for both created_at and created fields
      const hasCreatedAt = model.created_at !== undefined && model.created_at !== null;
      const hasCreated = model.created !== undefined && model.created !== null && model.created !== 0;
      const isAvailable = model.available !== false;
      return (hasCreatedAt || hasCreated) && isAvailable;
    });
    
    if (filtered.length === 0) {
      return models.filter(model => model && model.available !== false).slice(0, limit);
    }
    
    const sorted = filtered.sort((a, b) => {
      // Get the created timestamp, prioritizing created_at over created
      let aCreated = a.created_at || a.created;
      let bCreated = b.created_at || b.created;
      
      // Convert ISO date strings to timestamps for comparison
      if (typeof aCreated === 'string') {
        aCreated = new Date(aCreated).getTime();
      }
      if (typeof bCreated === 'string') {
        bCreated = new Date(bCreated).getTime();
      }
      
      // Ensure we have valid numbers
      if (isNaN(aCreated) || isNaN(bCreated)) {
        return 0;
      }
      
      return bCreated - aCreated; // Newest first
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

  const handleModelSelection = async (modelName: string) => {
    if (isModelChanging) return;
    
    // Don't switch if already using this model
    if (currentSelectedModel === modelName) {
      console.log(`Already using model: ${modelName}`);
      setShowModelDropdown(false);
      setShowModelSelection(false);
      setActiveQuickAction(null); // Reset quick action when closing
      return;
    }

    try {
      setIsModelChanging(true);
      setShowModelDropdown(false);
      setShowModelSelection(false);
      setActiveQuickAction(null);
      
      console.log(`=== MODEL SWITCH START: ${currentSelectedModel} -> ${modelName} ===`);
      setNulleyeState('thinking');
      
      // Import the hecate agent service
      const { hecateAgent } = await import('../../common/services/hecate-agent');
      
      // Ensure connection to Hecate agent
      const connected = await hecateAgent.connect();
      if (!connected) {
        throw new Error('Failed to connect to Hecate agent');
      }

      // Load the new model directly (no ejection needed for OpenRouter)
      console.log(`Loading new model: ${modelName}`);
      
      const loadingMessage = {
        id: Date.now().toString(),
        timestamp: new Date(),
        sender: 'hecate',
        message: `⚡ Switching to ${modelName}...`,
        type: 'update'
      };
      setChatMessages(prev => [...prev, loadingMessage]);
      
      const success = await hecateAgent.setModel(modelName);
      
      if (!success) {
        throw new Error(`Failed to switch to model: ${modelName}`);
      }
      
      console.log(`Successfully switched to model: ${modelName}`);
      console.log(`=== MODEL SWITCH COMPLETE ===`);
      
      // Update state with new model
      setCurrentSelectedModel(modelName);
      
      const systemMessage = {
        id: (Date.now() + 1).toString(),
        timestamp: new Date(),
        sender: 'hecate',
        message: `✅ ${modelName} ready`,
        type: 'update'
      };
      setChatMessages(prev => [...prev, systemMessage]);
      
      // Automatically reload model info if the modelinfo scope is active
      if (activeScope === 'modelinfo') {
        loadModelInfo(modelName);
      }
      
      setNulleyeState('success');
      setTimeout(() => setNulleyeState('base'), 2000);
      
    } catch (error) {
      console.error('Error setting model:', error);
      
      const errorMessage = {
        id: (Date.now() + 1).toString(),
        timestamp: new Date(),
        sender: 'hecate',
        message: `❌ Failed to switch to ${modelName}: ${(error as Error).message}`,
        type: 'error'
      };
      setChatMessages(prev => [...prev, errorMessage]);
      
      setNulleyeState('error');
      setTimeout(() => setNulleyeState('base'), 3000);
    } finally {
      setIsModelChanging(false);
    }
  };

  const handleChatSubmit = (e: React.FormEvent) => {
    e.preventDefault();

    // Block submission if model is changing, Hecate is thinking, or no model is ready
    if (isModelChanging || isProcessingChat || nullviewState === 'thinking' || (!defaultModelReady && !currentSelectedModel)) {
      console.log('🚫 Chat submission blocked:', {
        isModelChanging,
        isProcessingChat,
        nullviewState,
        defaultModelReady,
        currentSelectedModel,
        blockReason: isModelChanging ? 'Model changing' : 
                    isProcessingChat ? 'Chat processing' :
                    nullviewState === 'thinking' ? 'NullEye thinking' : 
                    'No model ready'
      });
      return;
    }

    if (chatInput.trim()) {
      const userMessage = {
        id: Date.now().toString(),
        timestamp: new Date(),
        sender: 'user',
        message: chatInput.trim(),
        type: 'text',
      };

      setChatMessages((prev) => [...prev, userMessage]);
      setChatInput('');

      setNulleyeState('thinking');
      setIsProcessingChat(true);
      console.log('🧠 Thinking state set, starting async response...');

      // Restore focus to input after clearing it
      setTimeout(() => {
        if (chatInputRef.current) {
          chatInputRef.current.focus();
        }
      }, 0);

      // Send real message to Hecate agent
      handleRealChatResponse(userMessage.message);
    }
  };

  const handleRealChatResponse = async (message: string) => {
    try {
      // Import the hecate agent service
      const { hecateAgent } = await import('../../common/services/hecate-agent');
      
      // Ensure connection to Hecate agent
      const connected = await hecateAgent.connect();
      if (!connected) {
        throw new Error('Failed to connect to Hecate agent');
      }

      // Send message and get response - thinking state remains active during this entire operation
      console.log('🔄 Sending message to Hecate agent, thinking state should be active...');
      const response = await hecateAgent.sendMessage(message, {
        wallet_address: publicKey || undefined,
        wallet_type: localStorage.getItem('walletType') || undefined,
        session_time: new Date().toISOString()
      });
      console.log('✅ Received response from Hecate, changing from thinking state...');

      // Create Hecate response message
      const hecateMessage = {
        id: (Date.now() + 1).toString(),
        timestamp: new Date(),
        sender: 'hecate',
        message: response.content,
        type: 'text',
        model_used: response.model_used,
        metadata: response.metadata
      };

      setChatMessages((prev) => [...prev, hecateMessage]);

      // Clear processing flag and change from thinking to appropriate response state
      setIsProcessingChat(false);
      if (response.confidence_score > 0.8) {
        setNulleyeState('success');
      } else if (response.metadata?.finish_reason === 'question') {
        setNulleyeState('question');
      } else {
        setNulleyeState('response');
      }

      // Restore focus to input when response is received
      setTimeout(() => {
        if (chatInputRef.current) {
          chatInputRef.current.focus();
        }
      }, 100);

      // Return to base state after a delay
      setTimeout(() => {
        setNulleyeState('base');
      }, 3000);

    } catch (error) {
      console.error('Failed to get Hecate response:', error);
      
      // Create user-friendly error message
      let userFriendlyMessage = "I'm having trouble processing your request right now. Please try again in a moment.";
      
      // Check for specific error types and provide better messages
      if (error instanceof Error) {
        const errorMsg = error.message.toLowerCase();
        
        if (errorMsg.includes('lm studio') || errorMsg.includes('localhost:1234')) {
          userFriendlyMessage = "I'm currently offline. Please check that the local AI service is running and try again.";
        } else if (errorMsg.includes('connection') || errorMsg.includes('network')) {
          userFriendlyMessage = "I'm having connection issues. Please check your network and try again.";
        } else if (errorMsg.includes('timeout')) {
          userFriendlyMessage = "That request took too long to process. Please try a shorter message or try again later.";
        } else if (errorMsg.includes('model') || errorMsg.includes('load')) {
          userFriendlyMessage = "I'm having trouble with my current AI model. Please try switching models or try again later.";
        } else if (errorMsg.includes('auth') || errorMsg.includes('permission')) {
          userFriendlyMessage = "I don't have permission to complete that request. Please check your authentication.";
        } else if (errorMsg.includes('rate limit')) {
          userFriendlyMessage = "I'm receiving too many requests right now. Please wait a moment and try again.";
        }
      }
      
      const errorMessage = {
        id: (Date.now() + 1).toString(),
        timestamp: new Date(),
        sender: 'hecate',
        message: userFriendlyMessage,
        type: 'error',
      };

      setChatMessages((prev) => [...prev, errorMessage]);
      setIsProcessingChat(false);
      setNulleyeState('error');
      
      // Restore focus to input when error occurs
      setTimeout(() => {
        if (chatInputRef.current) {
          chatInputRef.current.focus();
        }
      }, 100);
      
      setTimeout(() => setNulleyeState('base'), 3000);
    }
  };

  const handleChatInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { value } = e.target;
    setChatInput(value);
  };


  const handleScopesClick = (scopeId: string) => {
    const newScope = activeScope === scopeId ? null : scopeId;
    setActiveLens(newScope);
    
    // Load data when specific scopes are opened
    if (newScope === 'modelinfo') {
      console.log('Loading model info for scope click, currentSelectedModel:', currentSelectedModel);
      if (!currentSelectedModel) {
        console.warn('No current model selected, checking cache first');
        // Load models if not cached
        if (!modelsCached) {
          loadAvailableModels().then(() => {
            if (currentSelectedModel) {
              loadModelInfo(currentSelectedModel);
            }
          });
        }
      } else {
        loadModelInfo(currentSelectedModel);
      }
    }
  };

  const handleChatScroll = (e: React.UIEvent<HTMLDivElement>) => {
    const container = e.currentTarget;
    const scrollTop = container.scrollTop;
    const scrollHeight = container.scrollHeight;
    const clientHeight = container.clientHeight;
    
    // Consider user to be at bottom if within 50px of bottom
    const isNearBottom = scrollHeight - scrollTop - clientHeight <= 50;
    
    // Only disable auto-scroll if user has manually scrolled up significantly
    if (!isNearBottom && !isUserScrolling) {
      setIsUserScrolling(true);
      setChatAutoScroll(false);
      
      // Clear any existing timeout
      if (userScrollTimeoutRef.current) {
        clearTimeout(userScrollTimeoutRef.current);
      }
      
      // Re-enable auto-scroll after 3 seconds of no scrolling
      userScrollTimeoutRef.current = setTimeout(() => {
        setIsUserScrolling(false);
        setChatAutoScroll(true);
      }, 3000);
    } else if (isNearBottom && isUserScrolling) {
      // User scrolled back to bottom, re-enable auto-scroll immediately
      setIsUserScrolling(false);
      setChatAutoScroll(true);
      
      if (userScrollTimeoutRef.current) {
        clearTimeout(userScrollTimeoutRef.current);
      }
    }
  };


  const renderTabContent = () => {
    if (!publicKey) {
      // Non-authenticated user sees only Crossroads tab
      switch (mainHudActiveTab) {
        case 'crossroads':
          return (
            <div className={styles.crossroadsTab}>
              <h3>Crossroads</h3>
              <div className={styles.crossroadsContent}>
                <div className={styles.crossroadsWelcome}>
                  <h4>Welcome to NullBlock</h4>
                  <p>Your gateway to autonomous agent workflows and Web3 automation.</p>
                </div>
                
                <div className={styles.crossroadsFeatures}>
                  <div className={styles.featureCard}>
                    <div className={styles.featureIcon}>🤖</div>
                    <h5>Agent Orchestration</h5>
                    <p>Deploy autonomous agents for trading, social monitoring, and DeFi operations.</p>
                  </div>
                  
                  <div className={styles.featureCard}>
                    <div className={styles.featureIcon}>🔗</div>
                    <h5>MCP Integration</h5>
                    <p>Model Context Protocol for seamless AI agent communication and coordination.</p>
                  </div>
                  
                  <div className={styles.featureCard}>
                    <div className={styles.featureIcon}>⚡</div>
                    <h5>MEV Protection</h5>
                    <p>Advanced MEV protection through Flashbots integration and privacy pools.</p>
                  </div>
                  
                  <div className={styles.featureCard}>
                    <div className={styles.featureIcon}>📊</div>
                    <h5>Analytics & Insights</h5>
                    <p>Real-time portfolio analytics, risk assessment, and performance monitoring.</p>
                  </div>
                </div>
                
                <div className={styles.crossroadsActions}>
                  <h4>Get Started</h4>
                  <div className={styles.actionButtons}>
                    <button 
                      className={styles.actionButton}
                      onClick={() => window.open('https://aetherbytes.github.io/nullblock-sdk/', '_blank')}
                    >
                      📚 Documentation
                    </button>
                    <button 
                      className={styles.actionButton}
                      onClick={() => window.open('https://x.com/Nullblock_io', '_blank')}
                    >
                      𝕏 Follow Updates
                    </button>
                    <button 
                      className={styles.actionButton}
                      onClick={() => window.open('https://discord.gg/nullblock', '_blank')}
                    >
                      💬 Join Discord
                    </button>
                    <button 
                      className={styles.primaryActionButton}
                      onClick={() => onConnectWallet()}
                    >
                      🚀 Connect & Launch
                    </button>
                  </div>
                </div>
              </div>
            </div>
          );
        default:
          return (
            <div className={styles.defaultTab}>
              <p>Connect your wallet to access full features</p>
            </div>
          );
      }
    } else {
      // Authenticated user gets individual tabs
      switch (mainHudActiveTab) {
        case 'crossroads':
          return (
            <div className={styles.crossroadsTab}>
              <h3>Crossroads Marketplace</h3>
              <div className={styles.crossroadsContent}>
                <div className={styles.crossroadsWelcome}>
                  <h4>Marketplace & Discovery Service</h4>
                  <p>Test the new Crossroads marketplace for agents, workflows, tools, and MCP servers.</p>
                </div>
                
                <div className={styles.marketplaceTests}>
                  <h4>API Testing</h4>
                  <div className={styles.testGrid}>
                    <div className={styles.testCard}>
                      <h5>🏥 Service Health</h5>
                      <button 
                        className={styles.testButton}
                        onClick={async () => {
                          try {
                            const response = await fetch('/api/crossroads/health');
                            const data = await response.json();
                            alert(`Status: ${data.status}\nService: ${data.service}\nMessage: ${data.message}`);
                          } catch (error) {
                            alert(`Error: ${error}`);
                          }
                        }}
                      >
                        Test Health
                      </button>
                    </div>
                    
                    <div className={styles.testCard}>
                      <h5>📋 Get Listings</h5>
                      <button 
                        className={styles.testButton}
                        onClick={async () => {
                          try {
                            const response = await fetch('/api/marketplace/listings');
                            const data = await response.json();
                            alert(`Found ${data.listings?.length || 0} listings\n\nFirst listing: ${JSON.stringify(data.listings?.[0] || 'None', null, 2)}`);
                          } catch (error) {
                            alert(`Error: ${error}`);
                          }
                        }}
                      >
                        Get Listings
                      </button>
                    </div>
                    
                    <div className={styles.testCard}>
                      <h5>🤖 Discover Agents</h5>
                      <button 
                        className={styles.testButton}
                        onClick={async () => {
                          try {
                            const response = await fetch('/api/discovery/agents');
                            const data = await response.json();
                            alert(`Found ${data.agents?.length || 0} agents\n\nAgents: ${JSON.stringify(data.agents || [], null, 2)}`);
                          } catch (error) {
                            alert(`Error: ${error}`);
                          }
                        }}
                      >
                        Discover Agents
                      </button>
                    </div>
                    
                    <div className={styles.testCard}>
                      <h5>📊 System Stats</h5>
                      <button 
                        className={styles.testButton}
                        onClick={async () => {
                          try {
                            const response = await fetch('/api/admin/system/stats');
                            const data = await response.json();
                            alert(`System Stats:\nTotal Listings: ${data.total_listings || 0}\nActive Agents: ${data.active_agents || 0}\nUptime: ${data.uptime || 'N/A'}`);
                          } catch (error) {
                            alert(`Error: ${error}`);
                          }
                        }}
                      >
                        Get Stats
                      </button>
                    </div>
                  </div>
                </div>
                
                <div className={styles.createListing}>
                  <h4>Create Test Listing</h4>
                  <div className={styles.listingForm}>
                    <select 
                      className={styles.formSelect}
                      defaultValue="Agent"
                      id="listingType"
                    >
                      <option value="Agent">Agent</option>
                      <option value="Workflow">Workflow</option>
                      <option value="Tool">Tool</option>
                      <option value="McpServer">MCP Server</option>
                      <option value="Dataset">Dataset</option>
                      <option value="Model">Model</option>
                    </select>
                    <input 
                      type="text"
                      className={styles.formInput}
                      placeholder="Listing title..."
                      id="listingTitle"
                    />
                    <textarea 
                      className={styles.formTextarea}
                      placeholder="Description..."
                      rows={3}
                      id="listingDescription"
                    />
                    <button 
                      className={styles.createButton}
                      onClick={async () => {
                        const typeSelect = document.getElementById('listingType') as HTMLSelectElement;
                        const titleInput = document.getElementById('listingTitle') as HTMLInputElement;
                        const descriptionInput = document.getElementById('listingDescription') as HTMLTextAreaElement;
                        
                        if (!titleInput.value.trim()) {
                          alert('Please enter a title');
                          return;
                        }
                        
                        try {
                          const response = await fetch('/api/marketplace/listings', {
                            method: 'POST',
                            headers: {
                              'Content-Type': 'application/json',
                            },
                            body: JSON.stringify({
                              title: titleInput.value,
                              description: descriptionInput.value || 'Test listing',
                              listing_type: typeSelect?.value || 'Agent',
                              price: 0.0,
                              tags: ['test', 'demo']
                            })
                          });
                          const data = await response.json();
                          alert(`Listing created!\nID: ${data.listing?.id || 'N/A'}\nTitle: ${data.listing?.title || 'N/A'}`);
                          
                          titleInput.value = '';
                          descriptionInput.value = '';
                        } catch (error) {
                          alert(`Error: ${error}`);
                        }
                      }}
                    >
                      🚀 Create Listing
                    </button>
                  </div>
                </div>
              </div>
            </div>
          );
        case 'hecate':
          return (
            <>
              {/* Full-screen overlay for expanded chat */}
              {isChatExpanded && (
                <div className={styles.fullscreenOverlay}>
                  <div className={`${styles.chatSection} ${styles.expanded}`}>
                    <div className={styles.hecateChat}>
                      <div className={styles.chatHeader}>
                        <div className={styles.chatTitle}>
                          <h4>💬 Hecate Chat</h4>
                          <span className={styles.modelStatus}>
                            {defaultModelReady || currentSelectedModel ? 'Ready' : 'Loading...'}
                          </span>
                        </div>
                        <div className={styles.chatHeaderControls}>
                          <button 
                            className={styles.expandButton}
                            onClick={() => {
                              setIsChatExpanded(false);
                              if (activeScope) setActiveLens(null);
                            }}
                            title="Exit full screen"
                          >
                            ⊟
                          </button>
                        </div>
                      </div>
                      {/* Chat content will be rendered here */}
                    </div>
                  </div>
                </div>
              )}

              {/* Full-screen overlay for expanded scopes */}
              {isScopesExpanded && (
                <div className={styles.fullscreenOverlay}>
                  <div className={`${styles.scopesSection} ${styles.expanded}`}>
                    {activeScope ? (
                      <div className={styles.scopesExpanded}>
                        <div className={styles.scopesContent}>
                          <div className={styles.scopesContentHeader}>
                            <h5>
                              {activeScope === 'modelinfo' ? 'Model Information' : activeScope.charAt(0).toUpperCase() + activeScope.slice(1)}
                            </h5>
                            <div className={styles.scopesHeaderControls}>
                              <button 
                                className={styles.expandButton}
                                onClick={() => {
                                  setIsScopesExpanded(false);
                                }}
                                title="Exit full screen"
                              >
                                ⊟
                              </button>
                              <button className={styles.closeScopes} onClick={() => setActiveLens(null)}>
                                ×
                              </button>
                            </div>
                          </div>
                          {/* Scopes content will be rendered here */}
                        </div>
                      </div>
                    ) : (
                      <div className={styles.scopesScrollContainer}>
                        <div className={styles.chatHeader}>
                          <div className={styles.chatTitle}>
                            <h4>🎯 Scopes</h4>
                          </div>
                          <div className={styles.chatHeaderControls}>
                            <button 
                              className={styles.expandButton}
                              onClick={() => {
                                setIsScopesExpanded(false);
                              }}
                              title="Exit full screen"
                            >
                              ⊟
                            </button>
                          </div>
                        </div>
                        {/* Scopes content will be rendered here */}
                      </div>
                    )}
                  </div>
                </div>
              )}

              {/* Regular HUD content */}
              <div className={`${styles.hecateContainer} ${isChatExpanded ? styles.chatExpanded : ''} ${isScopesExpanded ? styles.scopesExpanded : ''}`}>
                <div className={styles.hecateContent}>
                  <div className={styles.hecateMain}>
                    <div className={styles.hecateInterface}>
                    <div className={`${styles.chatSection} ${isChatExpanded ? styles.hidden : ''} ${isScopesExpanded ? styles.hidden : ''}`}>
                    <div className={styles.hecateChat}>
                      <div className={styles.chatHeader}>
                        <div className={styles.chatTitle}>
                          <h4>{currentSelectedModel ? `HECATE:${currentSelectedModel.split('/').pop()?.split(':')[0]?.toUpperCase() || 'MODEL'}` : 'HECATE:LOADING'}</h4>
                          <span className={styles.chatStatus}>
                            {defaultModelReady || currentSelectedModel ? 'Ready' : 'Loading...'}
                          </span>
                        </div>
                        <div className={styles.chatHeaderControls}>
                          <button 
                            className={styles.expandButton}
                            onClick={() => {
                              const newChatExpanded = !isChatExpanded;
                              setIsChatExpanded(newChatExpanded);
                              if (isScopesExpanded) setIsScopesExpanded(false); // Close scopes if open
                              if (newChatExpanded && activeScope) setActiveLens(null); // Close active scope when expanding chat
                            }}
                            title={isChatExpanded ? "Exit full screen" : "Expand chat full screen"}
                          >
                            {isChatExpanded ? '⊟' : '⊞'}
                          </button>
                        </div>
                      </div>

                      <div className={styles.chatMessages} ref={chatMessagesRef} onScroll={handleChatScroll}>
                        {chatMessages.map((message) => (
                          <div
                            key={message.id}
                            className={`${styles.chatMessage} ${styles[`message-${message.sender}`]} ${message.type ? styles[`type-${message.type}`] : ''}`}
                          >
                            <div className={styles.messageHeader}>
                              <span className={styles.messageSender}>
                                {message.sender === 'hecate' ? (
                                  <span className={styles.hecateMessageSender}>
                                    <div className={`${styles.nullviewChat} ${styles[`chat-${nullviewState === 'thinking' ? 'thinking' : (message.type || 'base')}`]} ${styles.clickableNulleyeChat}`}>
                                      <div className={styles.staticFieldChat}></div>
                                      <div className={styles.coreNodeChat}></div>
                                      <div className={styles.streamLineChat}></div>
                                      <div className={styles.lightningSparkChat}></div>
                                    </div>
                                    Hecate
                                  </span>
                                ) : (
                                  '👤 You'
                                )}
                              </span>
                              <span className={styles.messageTime}>
                                {message.timestamp.toLocaleTimeString()}
                              </span>
                            </div>
                            <div className={styles.messageContent}>
                              <MarkdownRenderer content={message.message} />
                            </div>
                          </div>
                        ))}
                        
                        {/* Show thinking indicator when Hecate is processing */}
                        {nullviewState === 'thinking' && (
                          <div className={`${styles.chatMessage} ${styles['message-hecate']} ${styles['type-thinking']}`}>
                            <div className={styles.messageHeader}>
                              <span className={styles.messageSender}>
                                <span className={styles.hecateMessageSender}>
                                  <div className={`${styles.nullviewChat} ${styles['chat-thinking']} ${styles.clickableNulleyeChat}`}>
                                    <div className={styles.staticFieldChat}></div>
                                    <div className={styles.coreNodeChat}></div>
                                    <div className={styles.streamLineChat}></div>
                                    <div className={styles.lightningSparkChat}></div>
                                  </div>
                                  Hecate
                                </span>
                              </span>
                              <span className={styles.messageTime}>
                                {new Date().toLocaleTimeString()}
                              </span>
                            </div>
                            <div className={styles.messageContent}>
                              <div className={styles.thinkingIndicator}>
                                <span className={styles.thinkingDots}>●</span>
                                <span className={styles.thinkingDots}>●</span>
                                <span className={styles.thinkingDots}>●</span>
                                <span className={styles.thinkingText}>Thinking...</span>
                              </div>
                            </div>
                          </div>
                        )}
                        
                        <div ref={chatEndRef} />
                      </div>

                      <form className={styles.chatInput} onSubmit={handleChatSubmit}>
                        <input
                          ref={chatInputRef}
                          type="text"
                          value={chatInput}
                          onChange={handleChatInputChange}
                          placeholder={
                            isModelChanging 
                              ? "Switching models..." 
                              : isProcessingChat || nullviewState === 'thinking'
                                ? "Hecate is thinking..." 
                                : "Ask Hecate anything..."
                          }
                          className={styles.chatInputField}
                          disabled={isModelChanging || isProcessingChat || nullviewState === 'thinking' || (!defaultModelReady && !currentSelectedModel)}
                        />
                        <button 
                          type="submit" 
                          className={styles.chatSendButton}
                          disabled={isModelChanging || isProcessingChat || nullviewState === 'thinking' || (!defaultModelReady && !currentSelectedModel)}
                        >
                          <span>➤</span>
                        </button>
                      </form>

                    </div>
                  </div>

                  <div className={`${styles.scopesSection} ${isScopesExpanded ? styles.expanded : ''} ${isChatExpanded ? styles.hidden : ''}`}>
                    {activeScope ? (
                      <div className={styles.scopesExpanded}>
                        <div className={styles.scopesContent}>
                          <div className={styles.scopesHeader}>
                            <h5>
                              {activeScope === 'modelinfo' ? 'Model Information' : activeScope.charAt(0).toUpperCase() + activeScope.slice(1)}
                            </h5>
                            <div className={styles.scopesHeaderControls}>
                              <button 
                                className={styles.expandButton}
                                onClick={() => {
                                  const newScopesExpanded = !isScopesExpanded;
                                  setIsScopesExpanded(newScopesExpanded);
                                  if (isChatExpanded) setIsChatExpanded(false); // Close chat if open
                                  // Don't close active scope when expanding scopes - we want to keep it visible
                                }}
                                title={isScopesExpanded ? "Exit full screen" : "Expand scopes full screen"}
                              >
                                {isScopesExpanded ? '⊟' : '⊞'}
                              </button>
                              <button className={styles.closeScopes} onClick={() => setActiveLens(null)}>
                                ×
                              </button>
                            </div>
                          </div>
                          <div className={styles.scopesContent}>

                            {activeScope === 'modelinfo' && (
                              <div className={styles.modelInfoScope}>
                                {isLoadingModelInfo ? (
                                  <div className={styles.modelInfoLoading}>
                                    <p>🔄 Loading model information...</p>
                                  </div>
                                ) : modelInfo?.error ? (
                                  <div className={styles.modelInfoError}>
                                    <h6>❌ Error Loading Model Info</h6>
                                    <p>{modelInfo.error}</p>
                                    <div style={{marginTop: '10px', fontSize: '12px', color: '#666'}}>
                                      <p>Debug info:</p>
                                      <p>• Current selected model: {currentSelectedModel || 'None'}</p>
                                      <p>• Available models: {availableModels.length}</p>
                                      <p>• Default loaded: {defaultModelLoaded ? 'Yes' : 'No'}</p>
                                    </div>
                                    <button 
                                      onClick={() => {
                                        console.log('Manual reload triggered from error - clearing cache');
                                        setModelsCached(false);
                                        loadAvailableModels();
                                      }}
                                      style={{marginTop: '10px', padding: '5px 10px', border: '1px solid #ccc', borderRadius: '4px'}}
                                    >
                                      🔄 Reload Models
                                    </button>
                                  </div>
                                ) : showModelSelection ? (
                                  <div className={styles.modelSelectionContent}>
                                    <div className={styles.modelSelectionHeader}>
                                      <input
                                        type="text"
                                        placeholder="Search LLM database..."
                                        value={modelSearchQuery}
                                        onChange={(e) => setModelSearchQuery(e.target.value)}
                                        className={styles.modelSearchInput}
                                      />
                                      {isSearchingModels && (
                                        <div className={styles.searchingIndicator}>⟳ Searching...</div>
                                      )}
                                      <button 
                                        className={styles.backButton}
                                        onClick={() => {
                                          setShowModelSelection(false);
                                          setActiveQuickAction(null);
                                        }}
                                        title="Back to model info"
                                      >
                                        ← Back
                                      </button>
                                    </div>
                                    
                                    {/* Quick Actions Menu */}
                                    <div style={{overflowX: 'auto', overflowY: 'hidden', paddingBottom: '8px'}}>
                                      <div className={styles.quickActionsMenu} style={{display: 'flex', flexWrap: 'nowrap', gap: '8px', minWidth: 'fit-content'}}>
                                      <button 
                                        onClick={() => {
                                          setModelSearchQuery('');
                                          setActiveQuickAction('clear');
                                          setTimeout(() => setActiveQuickAction(null), 500);
                                        }}
                                        className={`${styles.quickActionTab} ${activeQuickAction === 'clear' ? styles.active : ''}`}
                                      >
                                        Clear Search
                                      </button>
                                      <button 
                                        onClick={() => {
                                          const newAction = activeQuickAction === 'latest' ? null : 'latest';
                                          setActiveQuickAction(newAction);
                                          setModelSearchQuery('');
                                          if (newAction === 'latest') {
                                            setCategoryModels([]); // Clear previous data
                                            loadCategoryModels('latest');
                                          }
                                        }}
                                        className={`${styles.quickActionTab} ${activeQuickAction === 'latest' ? styles.active : ''}`}
                                      >
                                        Latest
                                      </button>
                                      <button 
                                        onClick={() => {
                                          const newAction = activeQuickAction === 'free' ? null : 'free';
                                          setActiveQuickAction(newAction);
                                          setModelSearchQuery('');
                                          if (newAction === 'free') {
                                            setCategoryModels([]); // Clear previous data
                                            loadCategoryModels('free');
                                          }
                                        }}
                                        className={`${styles.quickActionTab} ${activeQuickAction === 'free' ? styles.active : ''}`}
                                      >
                                        Free
                                      </button>
                                      <button 
                                        onClick={() => {
                                          const newAction = activeQuickAction === 'fast' ? null : 'fast';
                                          setActiveQuickAction(newAction);
                                          setModelSearchQuery('');
                                          if (newAction === 'fast') {
                                            setCategoryModels([]); // Clear previous data
                                            loadCategoryModels('fast');
                                          }
                                        }}
                                        className={`${styles.quickActionTab} ${activeQuickAction === 'fast' ? styles.active : ''}`}
                                      >
                                        Fast
                                      </button>
                                      <button 
                                        onClick={() => {
                                          const newAction = activeQuickAction === 'premium' ? null : 'premium';
                                          setActiveQuickAction(newAction);
                                          setModelSearchQuery('');
                                          if (newAction === 'premium') {
                                            setCategoryModels([]); // Clear previous data
                                            loadCategoryModels('premium');
                                          }
                                        }}
                                        className={`${styles.quickActionTab} ${activeQuickAction === 'premium' ? styles.active : ''}`}
                                      >
                                        Premium
                                      </button>
                                      <button 
                                        onClick={() => {
                                          const newAction = activeQuickAction === 'thinkers' ? null : 'thinkers';
                                          setActiveQuickAction(newAction);
                                          setModelSearchQuery('');
                                          if (newAction === 'thinkers') {
                                            setCategoryModels([]); // Clear previous data
                                            loadCategoryModels('thinkers');
                                          }
                                        }}
                                        className={`${styles.quickActionTab} ${activeQuickAction === 'thinkers' ? styles.active : ''}`}
                                      >
                                        Thinkers
                                      </button>
                                      <button 
                                        onClick={() => {
                                          const newAction = activeQuickAction === 'instruct' ? null : 'instruct';
                                          setActiveQuickAction(newAction);
                                          setModelSearchQuery('');
                                          if (newAction === 'instruct') {
                                            setCategoryModels([]); // Clear previous data
                                            loadCategoryModels('instruct');
                                          }
                                        }}
                                        className={`${styles.quickActionTab} ${activeQuickAction === 'instruct' ? styles.active : ''}`}
                                      >
                                        Instruct
                                      </button>
                                      </div>
                                    </div>
                                    
                                    {/* Latest Models */}
                                    {activeQuickAction === 'latest' && (
                                      <div className={styles.modelSection}>
                                        <h6>
                                          Latest Models ({isLoadingCategory ? '...' : categoryModels.length})
                                        </h6>
                                        {isLoadingCategory ? (
                                          <div style={{textAlign: 'center', padding: '20px', opacity: 0.7}}>
                                            🔄 Loading latest models from OpenRouter...
                                          </div>
                                        ) : (
                                        <div className={styles.modelsList}>
                                          {categoryModels.map((model, index) => (
                                            <button
                                              key={`category-${model.name}-${index}`}
                                              onClick={() => handleModelSelection(model.name)}
                                              className={`${styles.modelSelectButton} ${model.name === currentSelectedModel ? styles.currentModel : ''}`}
                                            >
                                              <div className={styles.modelSelectInfo}>
                                                <span className={styles.modelSelectIcon}>{model.icon || '🤖'}</span>
                                                <div>
                                                  <div className={styles.modelSelectName}>{model.display_name}</div>
                                                  <div className={styles.modelSelectProvider}>{model.provider}</div>
                                                </div>
                                              </div>
                                              <div className={styles.modelSelectMeta}>
                                                <span className={styles.modelSelectTier}>
                                                  {model.tier === 'economical' ? '🆓' : 
                                                   model.tier === 'fast' ? '⚡' : 
                                                   model.tier === 'standard' ? '⭐' : 
                                                   model.tier === 'premium' ? '💎' : '🤖'}
                                                </span>
                                                {model.name === currentSelectedModel && <span className={styles.currentBadge}>✓</span>}
                                              </div>
                                            </button>
                                          ))}
                                        </div>
                                        )}
                                      </div>
                                    )}

                                    {/* Free Models */}
                                    {activeQuickAction === 'free' && (
                                      <div className={styles.modelSection}>
                                        <h6>Free Models ({getFreeModels(availableModels).length})</h6>
                                        <div className={styles.modelsList}>
                                          {getFreeModels(availableModels).map((model, index) => (
                                            <button
                                              key={`free-${model.name}-${index}`}
                                              onClick={() => handleModelSelection(model.name)}
                                              className={`${styles.modelSelectButton} ${model.name === currentSelectedModel ? styles.currentModel : ''}`}
                                            >
                                              <div className={styles.modelSelectInfo}>
                                                <span className={styles.modelSelectIcon}>{model.icon || '🆓'}</span>
                                                <div>
                                                  <div className={styles.modelSelectName}>{model.display_name}</div>
                                                  <div className={styles.modelSelectProvider}>{model.provider}</div>
                                                </div>
                                              </div>
                                              <div className={styles.modelSelectMeta}>
                                                <span className={styles.modelSelectTier}>🆓</span>
                                                {model.name === currentSelectedModel && <span className={styles.currentBadge}>✓</span>}
                                              </div>
                                            </button>
                                          ))}
                                        </div>
                                      </div>
                                    )}

                                    {/* Fast Models */}
                                    {activeQuickAction === 'fast' && (
                                      <div className={styles.modelSection}>
                                        <h6>Fast Models ({getFastModels(availableModels).length})</h6>
                                        <div className={styles.modelsList}>
                                          {getFastModels(availableModels).map((model, index) => (
                                            <button
                                              key={`fast-${model.name}-${index}`}
                                              onClick={() => handleModelSelection(model.name)}
                                              className={`${styles.modelSelectButton} ${model.name === currentSelectedModel ? styles.currentModel : ''}`}
                                            >
                                              <div className={styles.modelSelectInfo}>
                                                <span className={styles.modelSelectIcon}>{model.icon || '⚡'}</span>
                                                <div>
                                                  <div className={styles.modelSelectName}>{model.display_name}</div>
                                                  <div className={styles.modelSelectProvider}>{model.provider}</div>
                                                </div>
                                              </div>
                                              <div className={styles.modelSelectMeta}>
                                                <span className={styles.modelSelectTier}>⚡</span>
                                                {model.name === currentSelectedModel && <span className={styles.currentBadge}>✓</span>}
                                              </div>
                                            </button>
                                          ))}
                                        </div>
                                      </div>
                                    )}

                                    {/* Premium Models */}
                                    {activeQuickAction === 'premium' && (
                                      <div className={styles.modelSection}>
                                        <h6>Premium Models ({getPremiumModels(availableModels).length})</h6>
                                        <div className={styles.modelsList}>
                                          {getPremiumModels(availableModels).map((model, index) => (
                                            <button
                                              key={`premium-${model.name}-${index}`}
                                              onClick={() => handleModelSelection(model.name)}
                                              className={`${styles.modelSelectButton} ${model.name === currentSelectedModel ? styles.currentModel : ''}`}
                                            >
                                              <div className={styles.modelSelectInfo}>
                                                <span className={styles.modelSelectIcon}>{model.icon || '💎'}</span>
                                                <div>
                                                  <div className={styles.modelSelectName}>{model.display_name}</div>
                                                  <div className={styles.modelSelectProvider}>{model.provider}</div>
                                                </div>
                                              </div>
                                              <div className={styles.modelSelectMeta}>
                                                <span className={styles.modelSelectTier}>💎</span>
                                                {model.name === currentSelectedModel && <span className={styles.currentBadge}>✓</span>}
                                              </div>
                                            </button>
                                          ))}
                                        </div>
                                      </div>
                                    )}

                                    {/* Thinking Models */}
                                    {activeQuickAction === 'thinkers' && (
                                      <div className={styles.modelSection}>
                                        <h6>Thinking Models ({getThinkerModels(availableModels).length})</h6>
                                        <div className={styles.modelsList}>
                                          {getThinkerModels(availableModels).map((model, index) => (
                                            <button
                                              key={`thinker-${model.name}-${index}`}
                                              onClick={() => handleModelSelection(model.name)}
                                              className={`${styles.modelSelectButton} ${model.name === currentSelectedModel ? styles.currentModel : ''}`}
                                            >
                                              <div className={styles.modelSelectInfo}>
                                                <span className={styles.modelSelectIcon}>{model.icon || '🧠'}</span>
                                                <div>
                                                  <div className={styles.modelSelectName}>{model.display_name}</div>
                                                  <div className={styles.modelSelectProvider}>{model.provider}</div>
                                                </div>
                                              </div>
                                              <div className={styles.modelSelectMeta}>
                                                <span className={styles.modelSelectTier}>
                                                  {model.tier === 'economical' ? '🆓' : 
                                                   model.tier === 'fast' ? '⚡' : 
                                                   model.tier === 'standard' ? '⭐' : 
                                                   model.tier === 'premium' ? '💎' : '🧠'}
                                                </span>
                                                {model.name === currentSelectedModel && <span className={styles.currentBadge}>✓</span>}
                                              </div>
                                            </button>
                                          ))}
                                        </div>
                                      </div>
                                    )}

                                    {/* Instruct Models */}
                                    {activeQuickAction === 'instruct' && (
                                      <div className={styles.modelSection}>
                                        <h6>Instruct Models ({getInstructModels(availableModels).length})</h6>
                                        <div className={styles.modelsList}>
                                          {getInstructModels(availableModels).map((model, index) => (
                                            <button
                                              key={`instruct-${model.name}-${index}`}
                                              onClick={() => handleModelSelection(model.name)}
                                              className={`${styles.modelSelectButton} ${model.name === currentSelectedModel ? styles.currentModel : ''}`}
                                            >
                                              <div className={styles.modelSelectInfo}>
                                                <span className={styles.modelSelectIcon}>{model.icon || '💬'}</span>
                                                <div>
                                                  <div className={styles.modelSelectName}>{model.display_name}</div>
                                                  <div className={styles.modelSelectProvider}>{model.provider}</div>
                                                </div>
                                              </div>
                                              <div className={styles.modelSelectMeta}>
                                                <span className={styles.modelSelectTier}>
                                                  {model.tier === 'economical' ? '🆓' : 
                                                   model.tier === 'fast' ? '⚡' : 
                                                   model.tier === 'standard' ? '⭐' : 
                                                   model.tier === 'premium' ? '💎' : '💬'}
                                                </span>
                                                {model.name === currentSelectedModel && <span className={styles.currentBadge}>✓</span>}
                                              </div>
                                            </button>
                                          ))}
                                        </div>
                                      </div>
                                    )}

                                    {/* Search Results */}
                                    {searchResults.length > 0 && !activeQuickAction && (
                                      <div className={styles.modelSection}>
                                        <h6>Search Results ({searchResults.length})</h6>
                                        <div className={styles.modelsList}>
                                          {searchResults.slice(0, 5).map((model, index) => (
                                            <button
                                              key={`search-${model.name}-${index}`}
                                              onClick={() => handleModelSelection(model.name)}
                                              className={`${styles.modelSelectButton} ${model.name === currentSelectedModel ? styles.currentModel : ''}`}
                                            >
                                              <div className={styles.modelSelectInfo}>
                                                <span className={styles.modelSelectIcon}>{model.icon || '🤖'}</span>
                                                <div>
                                                  <div className={styles.modelSelectName}>{model.display_name}</div>
                                                  <div className={styles.modelSelectProvider}>{model.provider}</div>
                                                </div>
                                              </div>
                                              <div className={styles.modelSelectMeta}>
                                                <span className={styles.modelSelectTier}>
                                                  {model.tier === 'economical' ? '🆓' : 
                                                   model.tier === 'fast' ? '⚡' : 
                                                   model.tier === 'standard' ? '⭐' : 
                                                   model.tier === 'premium' ? '💎' : '🤖'}
                                                </span>
                                                {model.name === currentSelectedModel && <span className={styles.currentBadge}>✓</span>}
                                              </div>
                                            </button>
                                          ))}
                                        </div>
                                      </div>
                                    )}
                                    
                                    {/* Model Overview */}
                                    {!activeQuickAction && searchResults.length === 0 && (
                                      <div className={styles.modelSection}>
                                        <h6>Model Categories</h6>
                                        <div className={styles.categoryOverview}>
                                          <button 
                                            className={styles.categoryButton}
                                            onClick={() => setActiveQuickAction('latest')}
                                          >
                                            <span className={styles.categoryIcon}>🆕</span>
                                            <div className={styles.categoryInfo}>
                                              <div className={styles.categoryName}>Latest</div>
                                              <div className={styles.categoryCount}>{getLatestModels(availableModels).length} models</div>
                                            </div>
                                          </button>
                                          <button 
                                            className={styles.categoryButton}
                                            onClick={() => setActiveQuickAction('free')}
                                          >
                                            <span className={styles.categoryIcon}>🆓</span>
                                            <div className={styles.categoryInfo}>
                                              <div className={styles.categoryName}>Free</div>
                                              <div className={styles.categoryCount}>{getFreeModels(availableModels).length} models</div>
                                            </div>
                                          </button>
                                          <button 
                                            className={styles.categoryButton}
                                            onClick={() => setActiveQuickAction('fast')}
                                          >
                                            <span className={styles.categoryIcon}>⚡</span>
                                            <div className={styles.categoryInfo}>
                                              <div className={styles.categoryName}>Fast</div>
                                              <div className={styles.categoryCount}>{getFastModels(availableModels).length} models</div>
                                            </div>
                                          </button>
                                          <button 
                                            className={styles.categoryButton}
                                            onClick={() => setActiveQuickAction('premium')}
                                          >
                                            <span className={styles.categoryIcon}>💎</span>
                                            <div className={styles.categoryInfo}>
                                              <div className={styles.categoryName}>Premium</div>
                                              <div className={styles.categoryCount}>{getPremiumModels(availableModels).length} models</div>
                                            </div>
                                          </button>
                                          <button 
                                            className={styles.categoryButton}
                                            onClick={() => setActiveQuickAction('thinkers')}
                                          >
                                            <span className={styles.categoryIcon}>🧠</span>
                                            <div className={styles.categoryInfo}>
                                              <div className={styles.categoryName}>Thinkers</div>
                                              <div className={styles.categoryCount}>{getThinkerModels(availableModels).length} models</div>
                                            </div>
                                          </button>
                                          <button 
                                            className={styles.categoryButton}
                                            onClick={() => setActiveQuickAction('instruct')}
                                          >
                                            <span className={styles.categoryIcon}>💬</span>
                                            <div className={styles.categoryInfo}>
                                              <div className={styles.categoryName}>Instruct</div>
                                              <div className={styles.categoryCount}>{getInstructModels(availableModels).length} models</div>
                                            </div>
                                          </button>
                                        </div>
                                      </div>
                                    )}
                                    
                                    {/* Database Statistics */}
                                    {!activeQuickAction && searchResults.length === 0 && (
                                      <div className={styles.modelSection}>
                                        <h6>Database Statistics</h6>
                                        <div className={styles.modelCounts}>
                                          <p>📊 Total Available: {availableModels.filter(m => m.available).length}</p>
                                          <p>🆓 Free Models: {getFreeModels(availableModels, 999).length}</p>
                                          <p>⚡ Fast Models: {getFastModels(availableModels, 999).length}</p>
                                          <p>💎 Premium Models: {getPremiumModels(availableModels, 999).length}</p>
                                          <p>🧠 Thinking Models: {getThinkerModels(availableModels, 999).length}</p>
                                          <p>💬 Instruct Models: {getInstructModels(availableModels, 999).length}</p>
                                          <p>🆕 Latest Added: {getLatestModels(availableModels, 999).length}</p>
                                        </div>
                                      </div>
                                    )}
                                  </div>
                                ) : modelInfo ? (
                                  <div className={styles.modelInfoContent}>
                                    <div className={styles.modelInfoHeader}>
                                      <div className={styles.modelInfoTitle}>
                                        <span className={styles.modelIcon}>{modelInfo.icon || '🤖'}</span>
                                        <div>
                                          <h6>{modelInfo.display_name || modelInfo.name}</h6>
                                          <span className={styles.modelProvider}>{modelInfo.provider}</span>
                                        </div>
                                      </div>
                                      <div className={styles.modelStatus}>
                                        <div style={{display: 'flex', alignItems: 'center', gap: '8px'}}>
                                          <button 
                                            onClick={() => {
                                              console.log('Force reloading models - clearing cache');
                                              setModelsCached(false);
                                              loadAvailableModels();
                                            }}
                                            title="Reload models (forces fresh API call)"
                                            style={{
                                              background: 'none',
                                              border: 'none',
                                              padding: '4px',
                                              cursor: 'pointer',
                                              fontSize: '16px',
                                              lineHeight: '1',
                                              opacity: 0.7,
                                              transition: 'opacity 0.2s'
                                            }}
                                            onMouseEnter={(e) => (e.target as HTMLElement).style.opacity = '1'}
                                            onMouseLeave={(e) => (e.target as HTMLElement).style.opacity = '0.7'}
                                          >
                                            🔄
                                          </button>
                                          <button 
                                            className={styles.switchModelButton}
                                            onClick={() => {
                                              setShowModelSelection(true);
                                              setActiveQuickAction('latest');
                                              setCategoryModels([]); // Clear any existing data
                                              loadCategoryModels('latest');
                                            }}
                                            title="Switch to a different model"
                                          >
                                            Switch Model
                                          </button>
                                        </div>
                                      </div>
                                    </div>

                                    {modelInfo.description && (
                                      <div className={styles.modelInfoSection}>
                                        <h6>📝 Description</h6>
                                        <p>
                                          {modelInfo.description.length > 300 && !showFullDescription 
                                            ? `${modelInfo.description.substring(0, 300)}...`
                                            : modelInfo.description
                                          }
                                        </p>
                                        {modelInfo.description.length > 300 && (
                                          <button 
                                            onClick={() => setShowFullDescription(!showFullDescription)}
                                            className={styles.showMoreButton}
                                          >
                                            {showFullDescription ? 'Show Less' : 'Show More'}
                                          </button>
                                        )}
                                        {/* Add reasoning capability note */}
                                        {(modelInfo.supports_reasoning || (modelInfo.capabilities && modelInfo.capabilities.includes('reasoning'))) && 
                                         !(modelInfo.capabilities && modelInfo.capabilities.includes('reasoning_tokens')) && (
                                          <div className={styles.reasoningNote}>
                                            <p>
                                              <strong>💡 Note:</strong> This model supports general reasoning but not step-by-step reasoning tokens. 
                                              For complex reasoning tasks, consider using a model with reasoning tokens like DeepSeek-R1.
                                            </p>
                                          </div>
                                        )}
                                      </div>
                                    )}

                                    <div className={styles.modelInfoSection}>
                                      <h6>⚙️ Technical Specifications</h6>
                                      <div className={styles.modelSpecs}>
                                        <div className={styles.specItem}>
                                          <span className={styles.specLabel}>Context Length:</span>
                                          <span className={styles.specValue}>{modelInfo.context_length?.toLocaleString() || 'N/A'} tokens</span>
                                        </div>
                                        {modelInfo.top_provider?.max_completion_tokens && (
                                          <div className={styles.specItem}>
                                            <span className={styles.specLabel}>Max Output:</span>
                                            <span className={styles.specValue}>{modelInfo.top_provider.max_completion_tokens.toLocaleString()} tokens</span>
                                          </div>
                                        )}
                                        {modelInfo.architecture?.tokenizer && (
                                          <div className={styles.specItem}>
                                            <span className={styles.specLabel}>Tokenizer:</span>
                                            <span className={styles.specValue}>{modelInfo.architecture.tokenizer}</span>
                                          </div>
                                        )}
                                        {modelInfo.architecture?.instruct_type && (
                                          <div className={styles.specItem}>
                                            <span className={styles.specLabel}>Instruct Type:</span>
                                            <span className={styles.specValue}>{modelInfo.architecture.instruct_type}</span>
                                          </div>
                                        )}
                                        <div className={styles.specItem}>
                                          <span className={styles.specLabel}>Input Modalities:</span>
                                          <span className={styles.specValue}>
                                            {modelInfo.architecture?.input_modalities ? 
                                              modelInfo.architecture.input_modalities.map((m: string) => m === 'text' ? '📝' : m === 'image' ? '🖼️' : m).join(' ') : 
                                              '📝 Text'}
                                          </span>
                                        </div>
                                        <div className={styles.specItem}>
                                          <span className={styles.specLabel}>Output Modalities:</span>
                                          <span className={styles.specValue}>
                                            {modelInfo.architecture?.output_modalities ? 
                                              modelInfo.architecture.output_modalities.map((m: string) => m === 'text' ? '📝' : m === 'image' ? '🖼️' : m).join(' ') : 
                                              '📝 Text'}
                                          </span>
                                        </div>
                                        <div className={styles.specItem}>
                                          <span className={styles.specLabel}>Reasoning:</span>
                                          <span className={styles.specValue}>
                                            {modelInfo.supports_reasoning || (modelInfo.capabilities && modelInfo.capabilities.includes('reasoning')) 
                                              ? '✅ Yes (General reasoning)' 
                                              : '❌ No'}
                                          </span>
                                        </div>
                                        <div className={styles.specItem}>
                                          <span className={styles.specLabel}>Reasoning Tokens:</span>
                                          <span className={styles.specValue}>
                                            {modelInfo.supports_reasoning || (modelInfo.capabilities && modelInfo.capabilities.includes('reasoning_tokens'))
                                              ? '✅ Yes (Step-by-step thinking)' 
                                              : '❌ No (General reasoning only)'}
                                          </span>
                                        </div>
                                        <div className={styles.specItem}>
                                          <span className={styles.specLabel}>Vision:</span>
                                          <span className={styles.specValue}>
                                            {modelInfo.architecture?.input_modalities?.includes('image') ? '✅ Yes' : '❌ No'}
                                          </span>
                                        </div>
                                        <div className={styles.specItem}>
                                          <span className={styles.specLabel}>Function Calls:</span>
                                          <span className={styles.specValue}>{modelInfo.supports_function_calling ? '✅ Yes' : '❌ No'}</span>
                                        </div>
                                        {modelInfo.top_provider?.is_moderated !== undefined && (
                                          <div className={styles.specItem}>
                                            <span className={styles.specLabel}>Moderation:</span>
                                            <span className={styles.specValue}>
                                              {modelInfo.top_provider.is_moderated ? '🛡️ Moderated' : '🔓 Unmoderated'}
                                            </span>
                                          </div>
                                        )}
                                        {modelInfo.created && (
                                          <div className={styles.specItem}>
                                            <span className={styles.specLabel}>Created:</span>
                                            <span className={styles.specValue}>
                                              {new Date(modelInfo.created * 1000).toLocaleDateString()}
                                            </span>
                                          </div>
                                        )}
                                        {modelInfo.hugging_face_id && (
                                          <div className={styles.specItem}>
                                            <span className={styles.specLabel}>HuggingFace ID:</span>
                                            <span className={styles.specValue}>{modelInfo.hugging_face_id}</span>
                                          </div>
                                        )}
                                      </div>
                                    </div>

                                    <div className={styles.modelInfoSection}>
                                      <h6>💰 Cost Information</h6>
                                      <div className={styles.costInfo}>
                                        <div className={styles.specItem}>
                                          <span className={styles.specLabel}>Tier:</span>
                                          <span className={styles.specValue}>
                                            {modelInfo.tier === 'economical' ? '🆓 Free' : 
                                             modelInfo.tier === 'fast' ? '⚡ Fast' : 
                                             modelInfo.tier === 'standard' ? '⭐ Standard' : 
                                             modelInfo.tier === 'premium' ? '💎 Premium' : 
                                             modelInfo.tier || 'Unknown'}
                                          </span>
                                        </div>
                                        {modelInfo.pricing ? (
                                          <>
                                            <div className={styles.specItem}>
                                              <span className={styles.specLabel}>Input tokens (1M):</span>
                                              <span className={styles.specValue}>
                                                {!modelInfo.pricing.prompt || modelInfo.pricing.prompt === "0" || modelInfo.pricing.prompt === 0 
                                                  ? '🆓 Free' 
                                                  : `$${(parseFloat(modelInfo.pricing.prompt) * 1000000).toFixed(3)}`}
                                              </span>
                                            </div>
                                            <div className={styles.specItem}>
                                              <span className={styles.specLabel}>Output tokens (1M):</span>
                                              <span className={styles.specValue}>
                                                {!modelInfo.pricing.completion || modelInfo.pricing.completion === "0" || modelInfo.pricing.completion === 0
                                                  ? '🆓 Free'
                                                  : `$${(parseFloat(modelInfo.pricing.completion) * 1000000).toFixed(3)}`}
                                              </span>
                                            </div>
                                            {modelInfo.pricing.image && modelInfo.pricing.image !== "0" && (
                                              <div className={styles.specItem}>
                                                <span className={styles.specLabel}>Image processing:</span>
                                                <span className={styles.specValue}>${parseFloat(modelInfo.pricing.image).toFixed(6)} per image</span>
                                              </div>
                                            )}
                                            {modelInfo.pricing.internal_reasoning && modelInfo.pricing.internal_reasoning !== "0" && (
                                              <div className={styles.specItem}>
                                                <span className={styles.specLabel}>Internal reasoning:</span>
                                                <span className={styles.specValue}>${parseFloat(modelInfo.pricing.internal_reasoning).toFixed(6)} per token</span>
                                              </div>
                                            )}
                                          </>
                                        ) : modelInfo.cost_per_1k_tokens !== undefined ? (
                                          <>
                                            <div className={styles.specItem}>
                                              <span className={styles.specLabel}>Combined cost (1K tokens):</span>
                                              <span className={styles.specValue}>
                                                {modelInfo.cost_per_1k_tokens === 0 || modelInfo.cost_per_1k_tokens === null || modelInfo.cost_per_1k_tokens === undefined
                                                  ? (modelInfo.tier === 'economical' ? '🆓 Free' : '❓ Variable pricing') 
                                                  : `$${Number(modelInfo.cost_per_1k_tokens).toFixed(8)}`}
                                              </span>
                                            </div>
                                            <div className={styles.specItem}>
                                              <span className={styles.specLabel}>Combined cost (1M tokens):</span>
                                              <span className={styles.specValue}>
                                                {modelInfo.cost_per_1k_tokens === 0 || modelInfo.cost_per_1k_tokens === null || modelInfo.cost_per_1k_tokens === undefined
                                                  ? (modelInfo.tier === 'economical' ? '🆓 Free' : '❓ Variable pricing') 
                                                  : `$${(Number(modelInfo.cost_per_1k_tokens) * 1000).toFixed(5)}`}
                                              </span>
                                            </div>
                                          </>
                                        ) : (
                                          <div className={styles.specItem}>
                                            <span className={styles.specLabel}>⚠️ Pricing data:</span>
                                            <span className={styles.specValue}>
                                              <span style={{color: 'red'}}>No pricing information available</span>
                                            </span>
                                          </div>
                                        )}
                                        {modelInfo.estimated_session_cost !== undefined && (
                                          <div className={styles.specItem}>
                                            <span className={styles.specLabel}>Estimated session cost:</span>
                                            <span className={styles.specValue}>${modelInfo.estimated_session_cost.toFixed(6)}</span>
                                          </div>
                                        )}
                                        <div className={styles.costWarning}>
                                          <p>
                                            <strong>ℹ️ Pricing Info:</strong> Data sourced from OpenRouter API. 
                                            Rates are updated regularly but may vary by provider and usage patterns. 
                                            Free tier usage may have daily limits.
                                          </p>
                                          {modelInfo.info_loaded_at && (
                                            <p><small>Last updated: {new Date(modelInfo.info_loaded_at).toLocaleString()}</small></p>
                                          )}
                                        </div>
                                      </div>
                                    </div>

                                    {modelInfo.capabilities && modelInfo.capabilities.length > 0 && (
                                      <div className={styles.modelInfoSection}>
                                        <h6>🎯 Capabilities</h6>
                                        <div className={styles.capabilitiesList}>
                                          {modelInfo.capabilities.map((capability: string) => (
                                            <span 
                                              key={capability} 
                                              className={styles.capabilityTag}
                                              title={capability.replace('_', ' ')}
                                            >
                                              {capability.replace('_', ' ')}
                                            </span>
                                          ))}
                                        </div>
                                      </div>
                                    )}

                                    {modelInfo.supported_parameters && modelInfo.supported_parameters.length > 0 && (
                                      <div className={styles.modelInfoSection}>
                                        <h6>⚙️ Supported Parameters</h6>
                                        <div className={styles.parametersList}>
                                          {modelInfo.supported_parameters.map((param: string) => (
                                            <span 
                                              key={param} 
                                              className={styles.parameterTag}
                                              title={`Supports ${param} parameter`}
                                            >
                                              {param}
                                            </span>
                                          ))}
                                        </div>
                                      </div>
                                    )}

                                    {modelInfo.is_current && modelInfo.conversation_length > 0 && (
                                      <div className={styles.modelInfoSection}>
                                        <h6>📊 Session Statistics</h6>
                                        <div className={styles.sessionStats}>
                                          <div className={styles.specItem}>
                                            <span className={styles.specLabel}>Messages in conversation:</span>
                                            <span className={styles.specValue}>{modelInfo.conversation_length}</span>
                                          </div>
                                        </div>
                                      </div>
                                    )}

                                  </div>
                                ) : (
                                  <div className={styles.modelInfoEmpty}>
                                    <p>No model information available</p>
                                  </div>
                                )}

                          </div>
                        )}

                            {activeScope === 'tasks' && (
                              <div className={styles.tasksScope}>
                                <div className={styles.tasksHeader}>
                                  <h6>📋 Active Tasks</h6>
                                  <div className={styles.taskStats}>
                                    <span className={styles.stat}>
                                      Running: {tasks.filter((t) => t.status === 'running').length}
                                    </span>
                                    <span className={styles.stat}>
                                      Completed: {tasks.filter((t) => t.status === 'completed').length}
                                    </span>
                                    <span className={styles.stat}>
                                      Failed: {tasks.filter((t) => t.status === 'failed').length}
                                    </span>
                                  </div>
                                </div>
                                <div className={styles.tasksList}>
                                  {tasks.map((task) => (
                                    <div key={task.id} className={`${styles.taskItem} ${getStatusColor(task.status)}`}>
                                      <div className={styles.taskHeader}>
                                        <div className={styles.taskInfo}>
                                          <span className={styles.taskName}>{task.name}</span>
                                          <span className={styles.taskType}>{task.type}</span>
                                        </div>
                                        <div className={styles.taskStatus}>
                                          <span className={styles.statusDot}></span>
                                          {task.status}
                                        </div>
                                      </div>
                                      <div className={styles.taskDescription}>{task.description}</div>
                                      {task.progress !== undefined && (
                                        <div className={styles.taskProgress}>
                                          <div className={styles.progressBar}>
                                            <div 
                                              className={styles.progressFill} 
                                              style={{ width: `${task.progress}%` }}
                                            ></div>
                                          </div>
                                          <span className={styles.progressText}>{Math.round(task.progress)}%</span>
                                        </div>
                                      )}
                                      <div className={styles.taskMetadata}>
                                        <span className={styles.taskTime}>
                                          Started: {task.startTime.toLocaleTimeString()}
                                        </span>
                                        {task.endTime && (
                                          <span className={styles.taskTime}>
                                            Ended: {task.endTime.toLocaleTimeString()}
                                          </span>
                                        )}
                                      </div>
                                    </div>
                                  ))}
                                </div>
                              </div>
                            )}

                            {activeScope === 'agents' && (
                              <div className={styles.agentsScope}>
                                <div className={styles.agentsHeader}>
                                  <h6>🤖 Active Agents</h6>
                                </div>
                                <div className={styles.agentsList}>
                                  <div className={styles.agentItem}>
                                    <div className={styles.agentInfo}>
                                      <span className={styles.agentName}>Arbitrage Agent</span>
                                      <span className={styles.agentStatus}>Active</span>
                                    </div>
                                    <div className={styles.agentMetrics}>
                                      <span>Opportunities Found: 12</span>
                                      <span>Executed Trades: 8</span>
                                      <span>Success Rate: 92%</span>
                                    </div>
                                  </div>
                                  <div className={styles.agentItem}>
                                    <div className={styles.agentInfo}>
                                      <span className={styles.agentName}>Social Trading Agent</span>
                                      <span className={styles.agentStatus}>Active</span>
                                    </div>
                                    <div className={styles.agentMetrics}>
                                      <span>Signals Generated: 45</span>
                                      <span>Accuracy: 78%</span>
                                      <span>Last Update: 2m ago</span>
                                    </div>
                                  </div>
                                  <div className={styles.agentItem}>
                                    <div className={styles.agentInfo}>
                                      <span className={styles.agentName}>Portfolio Manager</span>
                                      <span className={styles.agentStatus}>Monitoring</span>
                                    </div>
                                    <div className={styles.agentMetrics}>
                                      <span>Assets Under Management: $12,450</span>
                                      <span>24h Performance: +2.3%</span>
                                      <span>Risk Level: Medium</span>
                                    </div>
                                  </div>
                                </div>
                              </div>
                            )}

                            {activeScope === 'logs' && (
                              <div className={styles.logsScope}>
                                <div className={styles.logsHeader}>
                                  <h6>📄 System Logs</h6>
                                  <div className={styles.logsControls}>
                                    <input
                                      type="text"
                                      placeholder="Search logs..."
                                      value={searchTerm}
                                      onChange={(e) => setSearchTerm(e.target.value)}
                                      className={styles.searchInput}
                                    />
                                    <select
                                      value={logFilter}
                                      onChange={(e) => setLogFilter(e.target.value as any)}
                                      className={styles.filterSelect}
                                    >
                                      <option value="all">All Levels</option>
                                      <option value="info">Info</option>
                                      <option value="warning">Warning</option>
                                      <option value="error">Error</option>
                                      <option value="success">Success</option>
                                      <option value="debug">Debug</option>
                                    </select>
                                    <label className={styles.autoScrollLabel}>
                                      <input
                                        type="checkbox"
                                        checked={autoScroll}
                                        onChange={(e) => setAutoScroll(e.target.checked)}
                                      />
                                      Auto-scroll
                                    </label>
                                  </div>
                                </div>
                                <div className={styles.logsContainer}>
                                  {filteredLogs.map((log) => (
                                    <div key={log.id} className={`${styles.logEntry} ${getLogLevelColor(log.level)}`}>
                                      <div className={styles.logHeader}>
                                        <span className={styles.logTimestamp}>
                                          {log.timestamp.toLocaleTimeString()}
                                        </span>
                                        <span className={styles.logLevel}>[{log.level.toUpperCase()}]</span>
                                        <span className={styles.logSource}>{log.source}</span>
                                      </div>
                                      <div className={styles.logMessage}>{log.message}</div>
                                      {log.data && (
                                        <div className={styles.logData}>
                                          <pre>{JSON.stringify(log.data, null, 2)}</pre>
                                        </div>
                                      )}
                                    </div>
                                  ))}
                                  <div ref={logsEndRef} />
                                </div>
                              </div>
                            )}

                            {activeScope === 'settings' && (
                              <div className={styles.settingsScope}>
                                <div className={styles.settingsSection}>
                                  <h6>🎨 Theme</h6>
                                  <div className={styles.themeSelector}>
                                    <button 
                                      className={`${styles.themeButton} ${theme === 'dark' ? styles.active : ''}`}
                                      onClick={() => onThemeChange('dark')}
                                    >
                                      🌙 Dark
                                    </button>
                                    <button 
                                      className={`${styles.themeButton} ${theme === 'light' ? styles.active : ''}`}
                                      onClick={() => onThemeChange('light')}
                                    >
                                      ☀️ Light
                                    </button>
                                    <button 
                                      className={`${styles.themeButton} ${theme === 'null' ? styles.active : ''}`}
                                      onClick={() => onThemeChange('null')}
                                    >
                                      ⚡ Cyber
                                    </button>
                                  </div>
                                </div>

                                <div className={styles.settingsSection}>
                                  <h6>ℹ️ Version Info</h6>
                                  <div className={styles.versionInfo}>
                                    <p><strong>NullBlock Platform:</strong> v1.0.0-beta</p>
                                    <p><strong>Hecate Agent:</strong> v0.8.2</p>
                                    <p><strong>MCP Protocol:</strong> v0.1.0</p>
                                    <p><strong>Build:</strong> {new Date().toLocaleDateString()}</p>
                                  </div>
                                </div>

                                <div className={styles.settingsSection}>
                                  <h6>🔗 Social Links</h6>
                                  <div className={styles.socialLinks}>
                                    <button 
                                      onClick={() => window.open('https://x.com/Nullblock_io', '_blank')}
                                      className={styles.socialButton}
                                    >
                                      🐦 𝕏
                                    </button>
                                    <button 
                                      onClick={() => window.open('https://discord.gg/nullblock', '_blank')}
                                      className={styles.socialButton}
                                    >
                                      💬 Discord
                                    </button>
                                    <button 
                                      onClick={() => window.open('https://aetherbytes.github.io/nullblock-sdk/', '_blank')}
                                      className={styles.socialButton}
                                    >
                                      📚 Docs
                                    </button>
                                  </div>
                                </div>
                              </div>
                            )}
                          </div>
                        </div>
                      </div>
                    ) : (
                      <div className={`${styles.scopesScrollContainer} ${isChatExpanded ? styles.hidden : ''}`}>
                        <div className={styles.chatHeader}>
                          <div className={styles.chatTitle}>
                            <h4>🎯 Scopes</h4>
                            <div className={styles.tooltipContainer}>
                              <div className={styles.helpIcon}>?</div>
                              <div className={styles.tooltip}>
                                <div className={styles.tooltipContent}>
                                  <h4>Scopes</h4>
                                  <p>
                                    Scopes are focused work environments, each tailored for specific tasks
                                    like code generation, data analysis, automation, and more. Select a
                                    scope to access its specialized toolset.
                                  </p>
                                </div>
                              </div>
                            </div>
                          </div>
                          <div className={styles.chatHeaderControls}>
                            <button 
                              className={styles.expandButton}
                              onClick={() => {
                                const newScopesExpanded = !isScopesExpanded;
                                setIsScopesExpanded(newScopesExpanded);
                                if (isChatExpanded) setIsChatExpanded(false); // Close chat if open
                              }}
                              title={isScopesExpanded ? "Exit full screen" : "Expand scopes full screen"}
                            >
                              {isScopesExpanded ? '⊟' : '⊞'}
                            </button>
                          </div>
                        </div>
                        <div className={styles.scopesInfoPanel}>
                          <div className={styles.scopesInfoContent}>

                            <div className={styles.scopesAppsSection}>
                              <div className={styles.scopesAppsGrid}>
                                {scopesOptions.map((scope) => (
                                  <button
                                    key={scope.id}
                                    className={styles.scopesAppButton}
                                    onClick={() => handleScopesClick(scope.id)}
                                    style={{ '--scopes-color': scope.color } as React.CSSProperties}
                                  >
                                    <span className={styles.scopesAppIcon}>{scope.icon}</span>
                                    <span className={styles.scopesAppTitle}>{scope.title}</span>
                                  </button>
                                ))}
                              </div>
                            </div>

                          </div>
                        </div>
                      </div>
                    )}
                    
                    {/* Hecate Avatar static at bottom of scopes container */}
                    {!activeScope && (
                      <div className={styles.scopesAvatar}>
                        <div className={styles.avatarCircle}>
                          <div className={`${styles.nullviewAvatar} ${styles[nullviewState]} ${styles.clickableNulleye}`}>
                            <div className={styles.pulseRingAvatar}></div>
                            <div className={styles.dataStreamAvatar}>
                              <div className={styles.streamLineAvatar}></div>
                              <div className={styles.streamLineAvatar}></div>
                              <div className={styles.streamLineAvatar}></div>
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
                            <div className={styles.coreNodeAvatar}></div>
                          </div>
                        </div>
                        <div className={styles.avatarInfo}>
                          <h4>Hecate</h4>
                          <p>Primary Interface Agent</p>
                        </div>
                      </div>
                    )}
                  </div>
                  </div>
                </div>
              </div>
            </div>
            </>
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

            // For connected users, navigate to hecate tab
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