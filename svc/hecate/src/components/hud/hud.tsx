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
    'status' | 'crossroads' | 'tasks' | 'agents' | 'logs' | 'hecate'
  >(publicKey ? 'status' : 'status');
  
  // Additional state needed for tab functionality
  const [tasks, setTasks] = useState<any[]>([]);
  const [mcpOperations, setMcpOperations] = useState<any[]>([]);
  const [logs, setLogs] = useState<any[]>([]);
  const [agents, setAgents] = useState<any[]>([]);
  const [autoScroll, setAutoScroll] = useState(true);
  const [searchTerm, setSearchTerm] = useState('');
  const [logFilter, setLogFilter] = useState<'all' | 'info' | 'warning' | 'error' | 'success' | 'debug'>('all');
  const logsEndRef = useRef<HTMLDivElement>(null);
  
  // Hecate specific state
  const [chatMessages, setChatMessages] = useState<any[]>([]);
  const [chatInput, setChatInput] = useState('');
  const [showSuggestions, setShowSuggestions] = useState(false);
  const [activeScope, setActiveLens] = useState<string | null>(null);
  const chatEndRef = useRef<HTMLDivElement>(null);
  const chatMessagesRef = useRef<HTMLDivElement>(null);
  const [chatAutoScroll, setChatAutoScroll] = useState(true);
  const [isUserScrolling, setIsUserScrolling] = useState(false);
  const userScrollTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const [availableModels, setAvailableModels] = useState<any[]>([]);
  const [showModelDropdown, setShowModelDropdown] = useState(false);
  const [currentSelectedModel, setCurrentSelectedModel] = useState<string | null>(null);
  const [isModelChanging, setIsModelChanging] = useState(false);
  const modelDropdownRef = useRef<HTMLDivElement>(null);

  // Sandbox state
  const [sandboxCode, setSandboxCode] = useState('# Welcome to NullBlock Sandbox\n# Select a language and start coding!\n\nprint("Hello NullBlock!")');
  const [selectedLanguage, setSelectedLanguage] = useState('python');
  const [sandboxOutput, setSandboxOutput] = useState('');
  const [isRunning, setIsRunning] = useState(false);
  const [savedScripts, setSavedScripts] = useState<{[key: string]: string}>({});

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
    if (!publicKey) return;
    
    // Initialize with empty chat
    setChatMessages([]);
    
    // Load available models
    loadAvailableModels();
    
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
  }, [publicKey]);

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

  // Helper functions for Hecate functionality
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
    { id: 'sandbox', icon: '⚡', title: 'Sandbox', description: 'Code execution', color: '#ffa502' },
    { id: 'templates', icon: '📋', title: 'Templates', description: 'Task templates', color: '#00d4ff' },
    { id: 'settings', icon: '⚙️', title: 'Settings', description: 'Theme & social links', color: '#747d8c' },
  ];

  const loadAvailableModels = async () => {
    try {
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
      setCurrentSelectedModel(modelsData.current_model);
      
      console.log('Available models loaded:', modelsData.models?.length || 0);
      
    } catch (error) {
      console.error('Error loading available models:', error);
    }
  };

  const handleModelSelection = async (modelName: string) => {
    if (isModelChanging) return;

    try {
      setIsModelChanging(true);
      setShowModelDropdown(false);
      
      setNulleyeState('thinking');
      
      // Import the hecate agent service
      const { hecateAgent } = await import('../../common/services/hecate-agent');
      
      // Ensure connection to Hecate agent
      const connected = await hecateAgent.connect();
      if (!connected) {
        throw new Error('Failed to connect to Hecate agent');
      }

      // First, eject the current model if one is loaded
      if (currentSelectedModel && currentSelectedModel !== modelName) {
        console.log(`Ejecting current model: ${currentSelectedModel}`);
        try {
          await hecateAgent.unloadModel(currentSelectedModel);
          console.log(`Successfully ejected model: ${currentSelectedModel}`);
          
          // Add ejection message
          const ejectionMessage = {
            id: Date.now().toString(),
            timestamp: new Date(),
            sender: 'hecate',
            message: `🔄 Ejected ${currentSelectedModel}`,
            type: 'update'
          };
          setChatMessages(prev => [...prev, ejectionMessage]);
          
          // Brief pause to allow ejection to complete
          await new Promise(resolve => setTimeout(resolve, 1000));
        } catch (ejectionError) {
          console.warn('Failed to eject current model:', ejectionError);
          // Continue with model loading even if ejection fails
        }
      }

      // Set the new model using the Hecate agent service
      console.log(`Loading new model: ${modelName}`);
      const success = await hecateAgent.setModel(modelName);
      
      if (!success) {
        throw new Error(`Failed to switch to model: ${modelName}`);
      }
      
      console.log(`Successfully switched to model: ${modelName}`);
      setCurrentSelectedModel(modelName);
      
      const systemMessage = {
        id: (Date.now() + 1).toString(),
        timestamp: new Date(),
        sender: 'hecate',
        message: `✅ Loaded ${modelName}`,
        type: 'update'
      };
      setChatMessages(prev => [...prev, systemMessage]);
      
      setNulleyeState('success');
      setTimeout(() => setNulleyeState('base'), 2000);
      
    } catch (error) {
      console.error('Error setting model:', error);
      setNulleyeState('error');
      setTimeout(() => setNulleyeState('base'), 2000);
    } finally {
      setIsModelChanging(false);
    }
  };

  const handleChatSubmit = (e: React.FormEvent) => {
    e.preventDefault();

    // Block submission if model is changing or Hecate is thinking
    if (isModelChanging || nullviewState === 'thinking') {
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
      setShowSuggestions(false);

      setNulleyeState('thinking');

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

      // Send message and get response
      const response = await hecateAgent.sendMessage(message, {
        wallet_address: publicKey || undefined,
        wallet_type: localStorage.getItem('walletType') || undefined,
        session_time: new Date().toISOString()
      });

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

      // Set NullView state based on response confidence/quality
      if (response.confidence_score > 0.8) {
        setNulleyeState('success');
      } else if (response.metadata?.finish_reason === 'question') {
        setNulleyeState('question');
      } else {
        setNulleyeState('response');
      }

      // Return to base state after a delay
      setTimeout(() => {
        setNulleyeState('base');
      }, 3000);

    } catch (error) {
      console.error('Failed to get Hecate response:', error);
      
      // Create error message
      const errorMessage = {
        id: (Date.now() + 1).toString(),
        timestamp: new Date(),
        sender: 'hecate',
        message: `I encountered an error processing your message: ${error instanceof Error ? error.message : 'Unknown error'}. Please try again.`,
        type: 'error',
      };

      setChatMessages((prev) => [...prev, errorMessage]);
      setNulleyeState('error');
      setTimeout(() => setNulleyeState('base'), 3000);
    }
  };

  const handleChatInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { value } = e.target;
    setChatInput(value);
    setShowSuggestions(value.length > 2);
  };

  const handleSuggestionClick = (suggestion: string) => {
    setChatInput(suggestion);
    setShowSuggestions(false);
  };

  const handleScopesClick = (scopeId: string) => {
    setActiveLens(activeScope === scopeId ? null : scopeId);
  };

  const handleChatScroll = (e: React.UIEvent<HTMLDivElement>) => {
    const container = e.currentTarget;
    const scrollTop = container.scrollTop;
    const scrollHeight = container.scrollHeight;
    const clientHeight = container.clientHeight;
    
    const isAwayFromBottom = scrollHeight - scrollTop - clientHeight > 100;
    setChatAutoScroll(!isAwayFromBottom);
  };

  // Sandbox functionality
  const languageOptions = [
    { id: 'python', name: 'Python', extension: '.py' },
    { id: 'javascript', name: 'JavaScript', extension: '.js' },
    { id: 'typescript', name: 'TypeScript', extension: '.ts' },
    { id: 'rust', name: 'Rust', extension: '.rs' },
    { id: 'go', name: 'Go', extension: '.go' },
  ];

  const handleLanguageChange = (language: string) => {
    setSelectedLanguage(language);
    const examples = {
      python: '# Welcome to NullBlock Sandbox\n# Python execution environment\n\nprint("Hello NullBlock!")',
      javascript: '// Welcome to NullBlock Sandbox\n// JavaScript execution environment\n\nconsole.log("Hello NullBlock!");',
      typescript: '// Welcome to NullBlock Sandbox\n// TypeScript execution environment\n\nconst message: string = "Hello NullBlock!";\nconsole.log(message);',
      rust: '// Welcome to NullBlock Sandbox\n// Rust execution environment\n\nfn main() {\n    println!("Hello NullBlock!");\n}',
      go: '// Welcome to NullBlock Sandbox\n// Go execution environment\n\npackage main\n\nimport "fmt"\n\nfunc main() {\n    fmt.Println("Hello NullBlock!")\n}',
    };
    setSandboxCode(examples[language as keyof typeof examples] || examples.python);
  };

  const handleRunCode = async () => {
    setIsRunning(true);
    setSandboxOutput('Running code...');
    
    try {
      // Simulate code execution - in real implementation, this would call NullBlock SDK
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      if (selectedLanguage === 'javascript') {
        // Simple JavaScript evaluation for demo
        try {
          // Create a safe execution environment for JavaScript
          const jsCode = sandboxCode;
          // Capture console.log output by overriding it temporarily
          const originalLog = console.log;
          let capturedOutput = '';
          console.log = (...args) => {
            capturedOutput += args.join(' ') + '\n';
          };
          
          // Execute the code
          eval(jsCode);
          
          // Restore original console.log
          console.log = originalLog;
          
          setSandboxOutput(`Output: ${capturedOutput.trim() || 'Code executed successfully'}`);
        } catch (error) {
          setSandboxOutput(`Error: ${error}`);
        }
      } else {
        setSandboxOutput(`✓ ${selectedLanguage} code would be executed via NullBlock SDK\nOutput: Hello NullBlock!`);
      }
    } catch (error) {
      setSandboxOutput(`Error: ${error}`);
    } finally {
      setIsRunning(false);
    }
  };

  const handleSaveScript = () => {
    const scriptName = prompt('Enter script name:');
    if (scriptName && scriptName.trim()) {
      setSavedScripts(prev => ({
        ...prev,
        [scriptName.trim()]: sandboxCode
      }));
      setSandboxOutput(`Script "${scriptName}" saved successfully!`);
    }
  };

  const handleLoadScript = (scriptName: string) => {
    if (savedScripts[scriptName]) {
      setSandboxCode(savedScripts[scriptName]);
      setSandboxOutput(`Script "${scriptName}" loaded!`);
    }
  };

  const renderTabContent = () => {
    if (!publicKey) {
      // Non-authenticated user sees only Status and Crossroads tabs
      switch (mainHudActiveTab) {
        case 'status':
          return (
            <div className={styles.statusTab}>
              <h3>System Status</h3>
              <div className={styles.statusGrid}>
                <div className={styles.statusItem}>
                  <span className={styles.statusLabel}>HUD:</span>
                  <span className={`${styles.statusValue} ${systemStatus.hud ? styles.online : styles.offline}`}>
                    {systemStatus.hud ? 'Online' : 'Offline'}
                  </span>
                </div>
                <div className={styles.statusItem}>
                  <span className={styles.statusLabel}>MCP:</span>
                  <span className={`${styles.statusValue} ${systemStatus.mcp ? styles.online : styles.offline}`}>
                    {systemStatus.mcp ? 'Online' : 'Offline'}
                  </span>
                </div>
                <div className={styles.statusItem}>
                  <span className={styles.statusLabel}>Orchestration:</span>
                  <span className={`${styles.statusValue} ${systemStatus.orchestration ? styles.online : styles.offline}`}>
                    {systemStatus.orchestration ? 'Online' : 'Offline'}
                  </span>
                </div>
                <div className={styles.statusItem}>
                  <span className={styles.statusLabel}>Agents:</span>
                  <span className={`${styles.statusValue} ${systemStatus.agents ? styles.online : styles.offline}`}>
                    {systemStatus.agents ? 'Online' : 'Offline'}
                  </span>
                </div>
                <div className={styles.statusItem}>
                  <span className={styles.statusLabel}>Portfolio:</span>
                  <span className={`${styles.statusValue} ${systemStatus.portfolio ? styles.online : styles.offline}`}>
                    {systemStatus.portfolio ? 'Online' : 'Offline'}
                  </span>
                </div>
                <div className={styles.statusItem}>
                  <span className={styles.statusLabel}>DeFi:</span>
                  <span className={`${styles.statusValue} ${systemStatus.defi ? styles.online : styles.offline}`}>
                    {systemStatus.defi ? 'Online' : 'Offline'}
                  </span>
                </div>
                <div className={styles.statusItem}>
                  <span className={styles.statusLabel}>Social:</span>
                  <span className={`${styles.statusValue} ${systemStatus.social ? styles.online : styles.offline}`}>
                    {systemStatus.social ? 'Online' : 'Offline'}
                  </span>
                </div>
                <div className={styles.statusItem}>
                  <span className={styles.statusLabel}>Arbitrage:</span>
                  <span className={`${styles.statusValue} ${systemStatus.arbitrage ? styles.online : styles.offline}`}>
                    {systemStatus.arbitrage ? 'Online' : 'Offline'}
                  </span>
                </div>
                <div className={styles.statusItem}>
                  <span className={styles.statusLabel}>Hecate:</span>
                  <span className={`${styles.statusValue} ${systemStatus.hecate ? styles.online : styles.offline}`}>
                    {systemStatus.hecate ? 'Online' : 'Offline'}
                  </span>
                </div>
                <div className={styles.statusItem}>
                  <span className={styles.statusLabel}>Erebus:</span>
                  <span className={`${styles.statusValue} ${systemStatus.erebus ? styles.online : styles.offline}`}>
                    {systemStatus.erebus ? 'Online' : 'Offline'}
                  </span>
                </div>
              </div>
              <div className={styles.connectionPrompt}>
                <div className={styles.walletRequired}>
                  <h4>🔒 Wallet Connection Required</h4>
                  <p>Connect your Web3 wallet to access the full NullBlock interface and agent features.</p>
                  <button 
                    className={styles.connectPromptButton}
                    onClick={() => onConnectWallet()}
                  >
                    Connect Wallet
                  </button>
                </div>
              </div>
            </div>
          );
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
        case 'status':
          return (
            <div className={styles.statusTab}>
              <h3>System Status</h3>
              <div className={styles.statusGrid}>
                <div className={styles.statusItem}>
                  <span className={styles.statusLabel}>HUD:</span>
                  <span className={`${styles.statusValue} ${systemStatus.hud ? styles.online : styles.offline}`}>
                    {systemStatus.hud ? 'Online' : 'Offline'}
                  </span>
                </div>
                <div className={styles.statusItem}>
                  <span className={styles.statusLabel}>MCP:</span>
                  <span className={`${styles.statusValue} ${systemStatus.mcp ? styles.online : styles.offline}`}>
                    {systemStatus.mcp ? 'Online' : 'Offline'}
                  </span>
                </div>
                <div className={styles.statusItem}>
                  <span className={styles.statusLabel}>Orchestration:</span>
                  <span className={`${styles.statusValue} ${systemStatus.orchestration ? styles.online : styles.offline}`}>
                    {systemStatus.orchestration ? 'Online' : 'Offline'}
                  </span>
                </div>
                <div className={styles.statusItem}>
                  <span className={styles.statusLabel}>Agents:</span>
                  <span className={`${styles.statusValue} ${systemStatus.agents ? styles.online : styles.offline}`}>
                    {systemStatus.agents ? 'Online' : 'Offline'}
                  </span>
                </div>
                <div className={styles.statusItem}>
                  <span className={styles.statusLabel}>Portfolio:</span>
                  <span className={`${styles.statusValue} ${systemStatus.portfolio ? styles.online : styles.offline}`}>
                    {systemStatus.portfolio ? 'Online' : 'Offline'}
                  </span>
                </div>
                <div className={styles.statusItem}>
                  <span className={styles.statusLabel}>DeFi:</span>
                  <span className={`${styles.statusValue} ${systemStatus.defi ? styles.online : styles.offline}`}>
                    {systemStatus.defi ? 'Online' : 'Offline'}
                  </span>
                </div>
                <div className={styles.statusItem}>
                  <span className={styles.statusLabel}>Social:</span>
                  <span className={`${styles.statusValue} ${systemStatus.social ? styles.online : styles.offline}`}>
                    {systemStatus.social ? 'Online' : 'Offline'}
                  </span>
                </div>
                <div className={styles.statusItem}>
                  <span className={styles.statusLabel}>Arbitrage:</span>
                  <span className={`${styles.statusValue} ${systemStatus.arbitrage ? styles.online : styles.offline}`}>
                    {systemStatus.arbitrage ? 'Online' : 'Offline'}
                  </span>
                </div>
                <div className={styles.statusItem}>
                  <span className={styles.statusLabel}>Hecate:</span>
                  <span className={`${styles.statusValue} ${systemStatus.hecate ? styles.online : styles.offline}`}>
                    {systemStatus.hecate ? 'Online' : 'Offline'}
                  </span>
                </div>
                <div className={styles.statusItem}>
                  <span className={styles.statusLabel}>Erebus:</span>
                  <span className={`${styles.statusValue} ${systemStatus.erebus ? styles.online : styles.offline}`}>
                    {systemStatus.erebus ? 'Online' : 'Offline'}
                  </span>
                </div>
              </div>
              <div className={styles.userInfo}>
                <h4>User Profile</h4>
                <div className={styles.profileInfo}>
                  <div className={styles.profileItem}>
                    <span className={styles.profileLabel}>ID:</span>
                    <span className={styles.profileValue}>{userProfile.id}</span>
                  </div>
                  <div className={styles.profileItem}>
                    <span className={styles.profileLabel}>Ascent Level:</span>
                    <span className={styles.profileValue}>{userProfile.ascent}</span>
                  </div>
                  <div className={styles.profileItem}>
                    <span className={styles.profileLabel}>Cache Value:</span>
                    <span className={styles.profileValue}>{userProfile.cacheValue}</span>
                  </div>
                  <div className={styles.profileItem}>
                    <span className={styles.profileLabel}>Memories:</span>
                    <span className={styles.profileValue}>{userProfile.memories}</span>
                  </div>
                </div>
              </div>
              <div className={styles.sessionInfo}>
                <h4>Session Details</h4>
                <div className={styles.profileInfo}>
                  <div className={styles.profileItem}>
                    <span className={styles.profileLabel}>Wallet Identity:</span>
                    <span className={styles.profileValue}>
                      {getUserStats().walletName || getUserStats().walletAddress}
                    </span>
                  </div>
                  <div className={styles.profileItem}>
                    <span className={styles.profileLabel}>Wallet Type:</span>
                    <span className={styles.profileValue}>{getUserStats().walletType}</span>
                  </div>
                  <div className={styles.profileItem}>
                    <span className={styles.profileLabel}>Session Time:</span>
                    <span className={styles.profileValue}>{getUserStats().sessionDuration}</span>
                  </div>
                  <div className={styles.profileItem}>
                    <span className={styles.profileLabel}>Connection Status:</span>
                    <span className={styles.profileValue}>{getUserStats().connectionStatus}</span>
                  </div>
                </div>
              </div>
            </div>
          );
        case 'crossroads':
          return (
            <div className={styles.crossroadsTab}>
              <h3>Crossroads</h3>
              <div className={styles.crossroadsContent}>
                <div className={styles.crossroadsWelcome}>
                  <h4>Welcome back to NullBlock</h4>
                  <p>Your agents are active and systems are operational. Explore your autonomous workflows.</p>
                </div>
                
                <div className={styles.crossroadsFeatures}>
                  <div className={styles.featureCard}>
                    <div className={styles.featureIcon}>📊</div>
                    <h5>Live Tasks</h5>
                    <p>{tasks.filter(t => t.status === 'running').length} active tasks, {tasks.filter(t => t.status === 'completed').length} completed</p>
                  </div>
                  
                  <div className={styles.featureCard}>
                    <div className={styles.featureIcon}>🔗</div>
                    <h5>MCP Operations</h5>
                    <p>{mcpOperations.filter(op => op.status === 'active').length} active operations, avg response: {Math.round(mcpOperations.reduce((acc, op) => acc + (op.responseTime || 0), 0) / mcpOperations.length)}ms</p>
                  </div>
                  
                  <div className={styles.featureCard}>
                    <div className={styles.featureIcon}>📝</div>
                    <h5>System Logs</h5>
                    <p>{logs.length} entries, {logs.filter(l => l.level === 'error').length} errors, {logs.filter(l => l.level === 'warning').length} warnings</p>
                  </div>
                  
                  <div className={styles.featureCard}>
                    <div className={styles.featureIcon}>🤖</div>
                    <h5>Agent Status</h5>
                    <p>All systems operational. Hecate interface ready for commands.</p>
                  </div>
                </div>
                
                <div className={styles.crossroadsActions}>
                  <h4>Quick Actions</h4>
                  <div className={styles.actionButtons}>
                    <button 
                      className={styles.actionButton}
                      onClick={() => setMainHudActiveTab('tasks')}
                    >
                      📋 View Tasks
                    </button>
                    <button 
                      className={styles.actionButton}
                      onClick={() => setMainHudActiveTab('agents')}
                    >
                      🤖 Agent Status
                    </button>
                    <button 
                      className={styles.actionButton}
                      onClick={() => setMainHudActiveTab('logs')}
                    >
                      📝 System Logs
                    </button>
                    <button 
                      className={styles.primaryActionButton}
                      onClick={() => setMainHudActiveTab('hecate')}
                    >
                      🚀 Launch Hecate
                    </button>
                  </div>
                </div>
              </div>
            </div>
          );
        case 'tasks':
          return (
            <div className={styles.tasksContainer}>
              <div className={styles.tasksHeader}>
                <h3>Active Tasks</h3>
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
                      <div className={styles.progressBar}>
                        <div className={styles.progressFill} style={{ width: `${task.progress}%` }}></div>
                        <span className={styles.progressText}>{Math.round(task.progress)}%</span>
                      </div>
                    )}
                    <div className={styles.taskTiming}>
                      <span>Started: {task.startTime.toLocaleTimeString()}</span>
                      {task.endTime && <span>Ended: {task.endTime.toLocaleTimeString()}</span>}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          );
        case 'agents':
          return (
            <div className={styles.agentsContainer}>
              <div className={styles.agentsHeader}>
                <h3>Active Agents</h3>
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
                    <span className={styles.agentStatus}>Idle</span>
                  </div>
                  <div className={styles.agentMetrics}>
                    <span>Rebalancing Events: 3</span>
                    <span>Risk Score: 0.23</span>
                    <span>Last Action: 15m ago</span>
                  </div>
                </div>
                <div className={styles.agentItem}>
                  <div className={styles.agentInfo}>
                    <span className={styles.agentName}>MCP Operations</span>
                    <span className={styles.agentStatus}>Active</span>
                  </div>
                  <div className={styles.agentMetrics}>
                    <span>Active Operations: {mcpOperations.filter(op => op.status === 'active').length}</span>
                    <span>Avg Response: {Math.round(mcpOperations.reduce((acc, op) => acc + (op.responseTime || 0), 0) / mcpOperations.length)}ms</span>
                    <span>Uptime: 99.8%</span>
                  </div>
                </div>
              </div>
            </div>
          );
        case 'logs':
          return (
            <div className={styles.logsContainer}>
              <div className={styles.logsHeader}>
                <h3>System Logs</h3>
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
              <div className={styles.logsContent}>
                {filteredLogs.map((log) => (
                  <div key={log.id} className={`${styles.logEntry} ${getLogLevelColor(log.level)}`}>
                    <div className={styles.logTimestamp}>{log.timestamp.toLocaleTimeString()}</div>
                    <div className={styles.logLevel}>{log.level.toUpperCase()}</div>
                    <div className={styles.logSource}>{log.source}</div>
                    <div className={styles.logMessage}>
                      {log.message}
                      {log.data && (
                        <details className={styles.logDetails}>
                          <summary className={styles.logSummary}>📊 Data</summary>
                          <pre className={styles.logData}>
                            {JSON.stringify(log.data, null, 2)}
                          </pre>
                        </details>
                      )}
                    </div>
                  </div>
                ))}
                <div ref={logsEndRef} />
              </div>
            </div>
          );
        case 'hecate':
          return (
            <div className={styles.hecateContainer}>
              <div className={styles.hecateContent}>
                <div className={styles.hecateMain}>
                  <div className={styles.hecateInterface}>
                  <div className={styles.chatSection}>
                    <div className={styles.hecateChat}>
                      <div className={styles.chatHeader}>
                        <div className={styles.chatTitle}>
                          {currentSelectedModel ? (
                            <div className={styles.modelSelector} ref={modelDropdownRef}>
                              <button 
                                className={`${styles.modelDropdownBtn} ${isModelChanging ? styles.modelChanging : ''}`}
                                onClick={() => !isModelChanging && setShowModelDropdown(!showModelDropdown)}
                                disabled={isModelChanging}
                                title={isModelChanging ? "Switching models..." : "Select model"}
                              >
                                {isModelChanging ? (
                                  <>⚡ Switching... <span className={styles.loadingSpinner}>⟳</span></>
                                ) : (
                                  <>{currentSelectedModel} <span className={styles.dropdownArrow}>▼</span></>
                                )}
                              </button>
                              {showModelDropdown && !isModelChanging && (
                                <div className={styles.modelDropdown}>
                                  <div className={styles.dropdownHeader}>Select Model</div>
                                  {availableModels.filter(model => model.available).map((model) => (
                                    <button
                                      key={model.name}
                                      className={`${styles.modelOption} ${model.name === currentSelectedModel ? styles.selected : ''}`}
                                      onClick={() => handleModelSelection(model.name)}
                                      title={`${model.display_name} (${model.provider})`}
                                    >
                                      <span className={styles.modelName}>{model.display_name}</span>
                                      <span className={styles.modelProvider}>{model.provider}</span>
                                      {model.name === currentSelectedModel && <span className={styles.selected}>✓</span>}
                                    </button>
                                  ))}
                                </div>
                              )}
                            </div>
                          ) : (
                            <h4>Chat with Hecate</h4>
                          )}
                        </div>
                        <div className={styles.chatHeaderControls}>
                          {!chatAutoScroll && (
                            <button
                              className={styles.scrollToBottomBtn}
                              onClick={() => {
                                setChatAutoScroll(true);
                                setIsUserScrolling(false);
                                
                                if (chatMessagesRef.current) {
                                  chatMessagesRef.current.scrollTop = chatMessagesRef.current.scrollHeight;
                                }
                                
                                if (chatEndRef.current) {
                                  chatEndRef.current.scrollIntoView({ behavior: 'smooth' });
                                }
                              }}
                              title="Scroll to bottom and resume auto-scroll"
                            >
                              ↓ Jump to latest
                            </button>
                          )}
                          <span className={styles.chatStatus}>Live</span>
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
                                    <div className={`${styles.nullviewChat} ${styles[`chat-${message.type || 'base'}`]} ${styles.clickableNulleyeChat}`}>
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
                            <div className={styles.messageContent}>{message.message}</div>
                          </div>
                        ))}
                        <div ref={chatEndRef} />
                      </div>

                      <form className={styles.chatInput} onSubmit={handleChatSubmit}>
                        <input
                          type="text"
                          value={chatInput}
                          onChange={handleChatInputChange}
                          placeholder={
                            isModelChanging 
                              ? "Switching models..." 
                              : nullviewState === 'thinking' 
                                ? "Hecate is thinking..." 
                                : "Ask Hecate anything..."
                          }
                          className={styles.chatInputField}
                          disabled={isModelChanging || nullviewState === 'thinking'}
                        />
                        <button 
                          type="submit" 
                          className={styles.chatSendButton}
                          disabled={isModelChanging || nullviewState === 'thinking'}
                        >
                          <span>➤</span>
                        </button>
                      </form>

                      {showSuggestions && !isModelChanging && nullviewState !== 'thinking' && (
                        <div className={styles.chatSuggestions}>
                          <div className={styles.suggestionsHeader}>
                            <span>💡 Quick Actions</span>
                          </div>
                          <div className={styles.suggestionsList}>
                            <button
                              className={styles.suggestionButton}
                              onClick={() => handleSuggestionClick('Show me available templates')}
                            >
                              📋 Browse Templates
                            </button>
                            <button
                              className={styles.suggestionButton}
                              onClick={() => handleSuggestionClick('Create a new workflow')}
                            >
                              🔗 New Workflow
                            </button>
                            <button
                              className={styles.suggestionButton}
                              onClick={() => handleSuggestionClick('Analyze market data')}
                            >
                              📊 Market Analysis
                            </button>
                            <button
                              className={styles.suggestionButton}
                              onClick={() => handleSuggestionClick('Generate code for trading bot')}
                            >
                              ⚡ Code Generator
                            </button>
                          </div>
                        </div>
                      )}
                    </div>
                  </div>

                  <div className={styles.scopesSection}>
                    {activeScope ? (
                      <div className={styles.scopesExpanded}>
                        <div className={styles.scopesContent}>
                          <div className={styles.scopesHeader}>
                            <h5>{scopesOptions.find(s => s.id === activeScope)?.icon} {activeScope.charAt(0).toUpperCase() + activeScope.slice(1)}</h5>
                            <button className={styles.closeScopes} onClick={() => setActiveLens(null)}>
                              ×
                            </button>
                          </div>
                          <div className={styles.scopesContent}>
                            {activeScope === 'sandbox' && (
                              <div className={styles.sandboxScope}>
                                <div className={styles.sandboxControls}>
                                  <select 
                                    value={selectedLanguage} 
                                    onChange={(e) => handleLanguageChange(e.target.value)}
                                    className={styles.languageSelect}
                                  >
                                    {languageOptions.map(lang => (
                                      <option key={lang.id} value={lang.id}>{lang.name}</option>
                                    ))}
                                  </select>
                                  <button 
                                    onClick={handleRunCode}
                                    disabled={isRunning}
                                    className={styles.runButton}
                                  >
                                    {isRunning ? '⏳ Running...' : '▶️ Run'}
                                  </button>
                                  <button 
                                    onClick={handleSaveScript}
                                    className={styles.saveButton}
                                  >
                                    💾 Save
                                  </button>
                                </div>
                                
                                <div className={styles.sandboxEditor}>
                                  <textarea
                                    value={sandboxCode}
                                    onChange={(e) => setSandboxCode(e.target.value)}
                                    className={styles.codeEditor}
                                    placeholder="Write your code here..."
                                    spellCheck={false}
                                  />
                                </div>
                                
                                <div className={styles.sandboxOutput}>
                                  <h6>Output:</h6>
                                  <pre className={styles.outputText}>{sandboxOutput || 'Ready to execute code...'}</pre>
                                </div>

                                {Object.keys(savedScripts).length > 0 && (
                                  <div className={styles.savedScripts}>
                                    <h6>Saved Scripts:</h6>
                                    <div className={styles.scriptsList}>
                                      {Object.keys(savedScripts).map(name => (
                                        <button 
                                          key={name}
                                          onClick={() => handleLoadScript(name)}
                                          className={styles.scriptButton}
                                        >
                                          📄 {name}
                                        </button>
                                      ))}
                                    </div>
                                  </div>
                                )}
                              </div>
                            )}

                            {activeScope === 'templates' && (
                              <div className={styles.templatesScope}>
                                <p>Task templates for common workflows</p>
                                <div className={styles.templatesList}>
                                  <div className={styles.templateCard}>
                                    <h6>📊 Data Analysis</h6>
                                    <p>Template for data processing workflows</p>
                                  </div>
                                  <div className={styles.templateCard}>
                                    <h6>🤖 Trading Bot</h6>
                                    <p>Template for automated trading strategies</p>
                                  </div>
                                  <div className={styles.templateCard}>
                                    <h6>📈 Portfolio Monitor</h6>
                                    <p>Template for portfolio tracking</p>
                                  </div>
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
                      <div className={styles.scopesScrollContainer}>
                        <div className={styles.scopesInfoPanel}>
                          <div className={styles.scopesInfoContent}>
                            <div className={styles.headerWithTooltip}>
                              <h3>🎯 Scopes</h3>
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
          className={`${styles.menuButton} ${mainHudActiveTab === 'status' ? styles.active : ''}`}
          onClick={() => setMainHudActiveTab('status')}
        >
          Status
        </button>
        <button 
          className={`${styles.menuButton} ${mainHudActiveTab === 'crossroads' ? styles.active : ''}`}
          onClick={() => setMainHudActiveTab('crossroads')}
        >
          Crossroads
        </button>
        
        {publicKey && (
          <>
            <button 
              className={`${styles.menuButton} ${styles.fadeIn} ${mainHudActiveTab === 'tasks' ? styles.active : ''}`}
              onClick={() => setMainHudActiveTab('tasks')}
            >
              Tasks
            </button>
            <button 
              className={`${styles.menuButton} ${styles.fadeIn} ${mainHudActiveTab === 'agents' ? styles.active : ''}`}
              onClick={() => setMainHudActiveTab('agents')}
            >
              Agents
            </button>
            <button 
              className={`${styles.menuButton} ${styles.fadeIn} ${mainHudActiveTab === 'logs' ? styles.active : ''}`}
              onClick={() => setMainHudActiveTab('logs')}
            >
              Logs
            </button>
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
        NULLBLOCK
      </div>

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
          setNulleyeState('processing');
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