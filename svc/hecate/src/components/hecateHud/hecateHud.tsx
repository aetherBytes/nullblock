import React, { useState, useEffect, useRef } from 'react';
import styles from './hecateHud.module.scss';

interface HecateHudProps {
  onClose: () => void;
  theme?: 'null' | 'light' | 'dark';
  initialActiveTab?: 'tasks' | 'mcp' | 'logs' | 'agents' | 'hecate';
  onTabChange?: (tab: 'tasks' | 'mcp' | 'logs' | 'agents' | 'hecate') => void;
  publicKey?: string | null;
  walletName?: string | null;
  walletType?: string;
  onThemeChange?: (theme: 'null' | 'cyber' | 'light' | 'dark') => void;
}

interface ChatMessage {
  id: string;
  timestamp: Date;
  sender: 'user' | 'hecate';
  message: string;
  type?: 'text' | 'update' | 'question' | 'suggestion';
}

interface LensOption {
  id: string;
  icon: string;
  title: string;
  description: string;
  color: string;
  expanded?: boolean;
}

interface Task {
  id: string;
  name: string;
  status: 'running' | 'completed' | 'failed' | 'pending';
  type: 'mcp' | 'agent' | 'system' | 'trading';
  description: string;
  startTime: Date;
  endTime?: Date;
  progress?: number;
  logs: LogEntry[];
}

interface LogEntry {
  id: string;
  timestamp: Date;
  level: 'info' | 'warning' | 'error' | 'success' | 'debug';
  source: string;
  message: string;
  data?: any;
}

interface MCPOperation {
  id: string;
  name: string;
  status: 'active' | 'idle' | 'error';
  endpoint: string;
  lastActivity: Date;
  responseTime?: number;
}

const HecateHud: React.FC<HecateHudProps> = ({
  onClose,
  theme = 'light',
  initialActiveTab = 'tasks',
  onTabChange,
  publicKey,
  walletName,
  walletType,
  onThemeChange,
}) => {
  const [activeTab, setActiveTab] = useState<'tasks' | 'mcp' | 'logs' | 'agents' | 'hecate'>(
    initialActiveTab,
  );
  const [tasks, setTasks] = useState<Task[]>([]);
  const [mcpOperations, setMcpOperations] = useState<MCPOperation[]>([]);
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [agents, setAgents] = useState<any[]>([]);
  const [autoScroll, setAutoScroll] = useState(true);
  const logsEndRef = useRef<HTMLDivElement>(null);
  const [searchTerm, setSearchTerm] = useState('');
  const [logFilter, setLogFilter] = useState<
    'all' | 'info' | 'warning' | 'error' | 'success' | 'debug'
  >('all');
  const [chatMessages, setChatMessages] = useState<ChatMessage[]>([]);
  const [chatInput, setChatInput] = useState('');
  const [showSuggestions, setShowSuggestions] = useState(false);
  const [activeLens, setActiveLens] = useState<string | null>(null);
  const chatEndRef = useRef<HTMLDivElement>(null);
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
  >('base');

  // Helper functions for user-specific stats
  const getUserStats = () => {
    const sessionStart = localStorage.getItem('lastAuthTime');
    const sessionTime = sessionStart ? Date.now() - parseInt(sessionStart) : 0;
    const sessionMinutes = Math.floor(sessionTime / (1000 * 60));
    
    return {
      walletAddress: publicKey ? `${publicKey.slice(0, 6)}...${publicKey.slice(-4)}` : 'Not Connected',
      walletName: walletName || null,
      walletType: walletType || 'Unknown',
      sessionDuration: sessionMinutes > 0 ? `${sessionMinutes}m` : 'Just started',
      connectionStatus: publicKey ? 'Connected' : 'Disconnected',
    };
  };

  // Define lens options outside useEffect
  const lensOptions: LensOption[] = [
    {
      id: 'templates',
      icon: '📋',
      title: 'Templates',
      description: 'Task templates',
      color: '#00d4ff',
    },
    {
      id: 'workflow',
      icon: '🔗',
      title: 'Workflow',
      description: 'Build workflows',
      color: '#a55eea',
    },
    {
      id: 'suggestions',
      icon: '💡',
      title: 'Suggestions',
      description: 'AI suggestions',
      color: '#00ff88',
    },
    {
      id: 'visualizer',
      icon: '📊',
      title: 'Visualizer',
      description: 'Data visualization',
      color: '#ff6b35',
    },
    {
      id: 'sandbox',
      icon: '⚡',
      title: 'Sandbox',
      description: 'Code execution',
      color: '#ffa502',
    },
    {
      id: 'voice',
      icon: '🎤',
      title: 'Voice',
      description: 'Voice controls',
      color: '#ff4757',
    },
    {
      id: 'collaboration',
      icon: '👥',
      title: 'Collaborate',
      description: 'Team sharing',
      color: '#2ed573',
    },
    {
      id: 'feedback',
      icon: '🔄',
      title: 'Feedback',
      description: 'Refine outputs',
      color: '#5352ed',
    },
    {
      id: 'analytics',
      icon: '📈',
      title: 'Analytics',
      description: 'Data insights',
      color: '#ff9ff3',
    },
    {
      id: 'automation',
      icon: '🤖',
      title: 'Automation',
      description: 'Auto workflows',
      color: '#54a0ff',
    },
    {
      id: 'security',
      icon: '🔒',
      title: 'Security',
      description: 'Access control & settings',
      color: '#ff6348',
    },
    {
      id: 'integration',
      icon: '🔌',
      title: 'Integration',
      description: 'API connections',
      color: '#5f27cd',
    },
    {
      id: 'settings',
      icon: '⚙️',
      title: 'Settings',
      description: 'Theme & social links',
      color: '#747d8c',
    },
  ];

  // Mock data for demonstration
  useEffect(() => {
    // Initialize with mock data
    const mockTasks: Task[] = [
      {
        id: '1',
        name: 'Arbitrage Opportunity Scan',
        status: 'running',
        type: 'trading',
        description:
          'Scanning DEXes for arbitrage opportunities across Uniswap V3, SushiSwap, and PancakeSwap',
        startTime: new Date(Date.now() - 300000),
        progress: 65,
        logs: [
          {
            id: '1',
            timestamp: new Date(),
            level: 'info',
            source: 'arbitrage-agent',
            message: 'Scanning Uniswap V3 pools',
          },
          {
            id: '2',
            timestamp: new Date(),
            level: 'info',
            source: 'arbitrage-agent',
            message: 'Found 3 potential opportunities',
          },
          {
            id: '3',
            timestamp: new Date(Date.now() - 1000),
            level: 'success',
            source: 'arbitrage-agent',
            message: 'Analyzing MEV protection requirements',
          },
        ],
      },
      {
        id: '2',
        name: 'Social Sentiment Analysis',
        status: 'completed',
        type: 'agent',
        description:
          'Analyzing social media sentiment for trading signals from Twitter, Reddit, and Discord',
        startTime: new Date(Date.now() - 600000),
        endTime: new Date(Date.now() - 300000),
        progress: 100,
        logs: [
          {
            id: '4',
            timestamp: new Date(Date.now() - 350000),
            level: 'success',
            source: 'sentiment-agent',
            message: 'Analysis completed successfully',
          },
          {
            id: '5',
            timestamp: new Date(Date.now() - 400000),
            level: 'info',
            source: 'sentiment-agent',
            message: 'Processed 1,247 social media posts',
          },
        ],
      },
      {
        id: '3',
        name: 'Portfolio Rebalancing',
        status: 'pending',
        type: 'system',
        description:
          'Automated portfolio rebalancing based on market conditions and risk parameters',
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
        logs: [
          {
            id: '6',
            timestamp: new Date(),
            level: 'info',
            source: 'flashbots-agent',
            message: 'Calculating optimal gas prices',
          },
          {
            id: '7',
            timestamp: new Date(Date.now() - 2000),
            level: 'warning',
            source: 'flashbots-agent',
            message: 'High network congestion detected',
          },
        ],
      },
    ];

    const mockMcpOperations: MCPOperation[] = [
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

    const mockLogs: LogEntry[] = [
      {
        id: '1',
        timestamp: new Date(),
        level: 'info',
        source: 'main.js:124',
        message: 'NullView interface initialized',
        data: { component: 'HecateHud', loadTime: '45ms', memory: '12.4MB' }
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
      {
        id: '6',
        timestamp: new Date(Date.now() - 5000),
        level: 'debug',
        source: 'system.monitor.ts:78',
        message: 'Performance metrics',
        data: { memory: '45%', cpu: '23%', heap: '89.2MB', uptime: '2h 34m' }
      },
      {
        id: '7',
        timestamp: new Date(Date.now() - 6000),
        level: 'info',
        source: 'flashbots.service.ts:134',
        message: 'Bundle submitted to MEV relay',
        data: { bundleHash: '0x1a2b3c...', gasPrice: '45 gwei', estimatedProfit: '$12.45' }
      },
      {
        id: '8',
        timestamp: new Date(Date.now() - 7000),
        level: 'success',
        source: 'risk.manager.ts:92',
        message: 'Risk assessment completed - all checks passed',
        data: { riskScore: 0.23, maxDrawdown: '2.1%', sharpeRatio: 1.87 }
      },
      {
        id: '9',
        timestamp: new Date(Date.now() - 8000),
        level: 'info',
        source: 'market.feed.ts:67',
        message: 'Price feed update complete',
        data: { tokens: 1247, sources: ['CoinGecko', 'Binance', 'Uniswap'], latency: '156ms' }
      },
      {
        id: '10',
        timestamp: new Date(Date.now() - 9000),
        level: 'warning',
        source: 'network.monitor.ts:89',
        message: 'Ethereum gas prices elevated',
        data: { currentGas: '85 gwei', average: '45 gwei', recommendation: 'delay_non_urgent' }
      },
      {
        id: '11',
        timestamp: new Date(Date.now() - 10000),
        level: 'debug',
        source: 'wallet.service.ts:123',
        message: 'Wallet balance check',
        data: { address: '0x742d...35', balance: '12.5 ETH', usdValue: '$29,847' }
      },
    ];

    setTasks(mockTasks);
    setMcpOperations(mockMcpOperations);
    setLogs(mockLogs);

    // Initialize chat with welcome message
    const initialChatMessages: ChatMessage[] = [
      {
        id: '1',
        timestamp: new Date(Date.now() - 5000),
        sender: 'hecate',
        message:
          "Hecate interface active. Autonomous agent protocols initialized. How may I assist with your workflows?",
        type: 'text',
      },
      {
        id: '2',
        timestamp: new Date(Date.now() - 3000),
        sender: 'hecate',
        message:
          "I've detected 3 new arbitrage opportunities across Uniswap V3 and SushiSwap. Would you like me to analyze them?",
        type: 'suggestion',
      },
      {
        id: '3',
        timestamp: new Date(Date.now() - 1000),
        sender: 'hecate',
        message:
          'Your portfolio has increased by 2.3% in the last hour. All systems are running optimally.',
        type: 'update',
      },
    ];

    setChatMessages(initialChatMessages);

    // Simulate real-time log updates
    const interval = setInterval(() => {
      const logMessages = [
        { 
          level: 'info', 
          source: 'market.feed.ts:89', 
          message: 'Price update received',
          data: { symbol: 'ETH/USD', price: '$2,847.32', change: '+1.2%', volume24h: '$1.2B' }
        },
        { 
          level: 'info', 
          source: 'arbitrage.scanner.ts:145', 
          message: 'DEX scan initiated',
          data: { dexes: ['Uniswap', 'SushiSwap', 'Curve'], pairs: 247, scanTime: '1.2s' }
        },
        { 
          level: 'success', 
          source: 'flashbots.client.ts:203', 
          message: 'MEV bundle included in block',
          data: { blockNumber: 18945672, bundleHash: '0x4f5e6d...', profit: '$23.45' }
        },
        { 
          level: 'warning', 
          source: 'gas.monitor.ts:67', 
          message: 'Gas price spike detected',
          data: { currentGas: '95 gwei', previousGas: '45 gwei', increase: '111%' }
        },
        { 
          level: 'debug', 
          source: 'performance.monitor.ts:34', 
          message: 'System metrics update',
          data: { memory: '47%', cpu: '23%', heap: '102.4MB', connections: 12, uptime: '3h 12m' }
        },
        {
          level: 'info',
          source: 'portfolio.service.ts:178',
          message: 'Portfolio valuation updated',
          data: { totalValue: '$12,847.32', change24h: '+2.3%', assets: 8, lastUpdate: new Date().toISOString() }
        },
        { 
          level: 'success', 
          source: 'risk.engine.ts:124', 
          message: 'Risk assessment completed',
          data: { riskScore: 0.18, confidence: '94%', checksPassed: 15, checksTotal: 15 }
        },
        { 
          level: 'info', 
          source: 'sentiment.analyzer.ts:89', 
          message: 'Social sentiment batch processed',
          data: { sources: ['Twitter', 'Reddit', 'Discord'], posts: 23, bullish: 18, bearish: 5 }
        },
      ];

      const randomLog = logMessages[Math.floor(Math.random() * logMessages.length)];
      const newLog: LogEntry = {
        id: Date.now().toString(),
        timestamp: new Date(),
        level: randomLog.level as any,
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
  }, []);

  useEffect(() => {
    if (autoScroll && logsEndRef.current) {
      logsEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [logs, autoScroll]);

  useEffect(() => {
    if (chatEndRef.current) {
      chatEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [chatMessages]);

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

  const handleChatSubmit = (e: React.FormEvent) => {
    e.preventDefault();

    if (chatInput.trim()) {
      const userMessage: ChatMessage = {
        id: Date.now().toString(),
        timestamp: new Date(),
        sender: 'user',
        message: chatInput.trim(),
        type: 'text',
      };

      setChatMessages((prev) => [...prev, userMessage]);
      setChatInput('');
      setShowSuggestions(false);

      // Set NullView to thinking state
      setNulleyeState('thinking');

      // Simulate Hecate response
      setTimeout(
        () => {
          const responses = [
            {
              message: "I'm analyzing your request. Let me check the current system status...",
              type: 'update',
            },
            {
              message:
                'Based on the current market conditions, I recommend monitoring the arbitrage opportunities.',
              type: 'suggestion',
            },
            {
              message:
                "I've updated your portfolio settings. The changes will take effect immediately.",
              type: 'update',
            },
            {
              message:
                'The social sentiment analysis shows positive signals for your current positions.',
              type: 'text',
            },
            {
              message:
                "I've detected a potential MEV opportunity. Would you like me to prepare a bundle?",
              type: 'question',
            },
            {
              message:
                'Your risk assessment has been completed. All parameters are within acceptable limits.',
              type: 'update',
            },
            {
              message: "I'm processing your request. This may take a few moments...",
              type: 'text',
            },
            {
              message: 'The market data indicates favorable conditions for your trading strategy.',
              type: 'suggestion',
            },
            {
              message:
                'I can help you with that! Would you like me to use a template or create a custom workflow?',
              type: 'question',
            },
            {
              message: 'Let me suggest some relevant templates for your request...',
              type: 'suggestion',
            },
            {
              message:
                "I've prepared a workflow template that matches your needs. Would you like to customize it?",
              type: 'question',
            },
          ];

          const randomResponse = responses[Math.floor(Math.random() * responses.length)];
          const hecateMessage: ChatMessage = {
            id: (Date.now() + 1).toString(),
            timestamp: new Date(),
            sender: 'hecate',
            message: randomResponse.message,
            type: randomResponse.type as 'text' | 'update' | 'question' | 'suggestion',
          };

          setChatMessages((prev) => [...prev, hecateMessage]);

          // Set NullView state based on response type
          switch (randomResponse.type) {
            case 'update':
              setNulleyeState('response');
              break;
            case 'question':
              setNulleyeState('question');
              break;
            case 'suggestion':
              setNulleyeState('success');
              break;
            default:
              setNulleyeState('base');
          }

          // Return to base state after a delay
          setTimeout(() => {
            setNulleyeState('base');
          }, 3000);
        },
        1000 + Math.random() * 2000,
      );
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

  const handleLensClick = (lensId: string) => {
    setActiveLens(activeLens === lensId ? null : lensId);
  };

  const handleNullViewClick = () => {
    setActiveTab('hecate');

    if (onTabChange) {
      onTabChange('hecate');
    }
  };

  const renderLensContent = (lensId: string) => {
    switch (lensId) {
      case 'templates':
        return (
          <div className={styles.lensContent}>
            <div className={styles.lensHeader}>
              <h5>📋 Task Templates</h5>
              <button className={styles.closeLens} onClick={() => setActiveLens(null)}>
                ×
              </button>
            </div>
            <div className={styles.templateGrid}>
              <div className={styles.templateCard}>
                <div className={styles.templateIcon}>🐍</div>
                <h6>Python Script</h6>
                <p>Generate Python code for automation</p>
                <button className={styles.useTemplate}>Use Template</button>
              </div>
              <div className={styles.templateCard}>
                <div className={styles.templateIcon}>📊</div>
                <h6>Data Analysis</h6>
                <p>Analyze CSV data with visualizations</p>
                <button className={styles.useTemplate}>Use Template</button>
              </div>
              <div className={styles.templateCard}>
                <div className={styles.templateIcon}>📝</div>
                <h6>Content Writer</h6>
                <p>Create blog posts and articles</p>
                <button className={styles.useTemplate}>Use Template</button>
              </div>
              <div className={styles.templateCard}>
                <div className={styles.templateIcon}>🤖</div>
                <h6>Bot Builder</h6>
                <p>Create automated workflows</p>
                <button className={styles.useTemplate}>Use Template</button>
              </div>
            </div>
          </div>
        );

      case 'workflow':
        return (
          <div className={styles.lensContent}>
            <div className={styles.lensHeader}>
              <h5>🔗 Workflow Builder</h5>
              <button className={styles.closeLens} onClick={() => setActiveLens(null)}>
                ×
              </button>
            </div>
            <div className={styles.workflowCanvas}>
              <div className={styles.workflowNode}>
                <div className={styles.nodeIcon}>📥</div>
                <span>Input Data</span>
              </div>
              <div className={styles.workflowArrow}>→</div>
              <div className={styles.workflowNode}>
                <div className={styles.nodeIcon}>⚙️</div>
                <span>Process</span>
              </div>
              <div className={styles.workflowArrow}>→</div>
              <div className={styles.workflowNode}>
                <div className={styles.nodeIcon}>📤</div>
                <span>Output</span>
              </div>
            </div>
            <div className={styles.workflowControls}>
              <button className={styles.workflowBtn}>Add Node</button>
              <button className={styles.workflowBtn}>Save Workflow</button>
              <button className={styles.workflowBtn}>Run</button>
            </div>
          </div>
        );

      case 'visualizer':
        return (
          <div className={styles.lensContent}>
            <div className={styles.lensHeader}>
              <h5>📊 Data Visualizer</h5>
              <button className={styles.closeLens} onClick={() => setActiveLens(null)}>
                ×
              </button>
            </div>
            <div className={styles.visualizerCanvas}>
              <div className={styles.chartPlaceholder}>
                <div className={styles.chartBar} style={{ height: '60%' }}></div>
                <div className={styles.chartBar} style={{ height: '80%' }}></div>
                <div className={styles.chartBar} style={{ height: '40%' }}></div>
                <div className={styles.chartBar} style={{ height: '90%' }}></div>
                <div className={styles.chartBar} style={{ height: '70%' }}></div>
              </div>
            </div>
            <div className={styles.visualizerControls}>
              <button className={styles.vizBtn}>Bar Chart</button>
              <button className={styles.vizBtn}>Line Chart</button>
              <button className={styles.vizBtn}>Pie Chart</button>
              <button className={styles.vizBtn}>Export</button>
            </div>
          </div>
        );

      case 'sandbox':
        return (
          <div className={styles.lensContent}>
            <div className={styles.lensHeader}>
              <h5>⚡ Code Sandbox</h5>
              <button className={styles.closeLens} onClick={() => setActiveLens(null)}>
                ×
              </button>
            </div>
            <div className={styles.codeEditor}>
              <div className={styles.editorHeader}>
                <span>main.py</span>
                <div className={styles.editorControls}>
                  <button className={styles.editorBtn}>Run</button>
                  <button className={styles.editorBtn}>Save</button>
                </div>
              </div>
              <textarea
                className={styles.codeTextarea}
                placeholder="print('Hello, World!')"
                defaultValue="import requests&#10;&#10;response = requests.get('https://api.example.com/data')&#10;print(response.json())"
              />
            </div>
            <div className={styles.outputPanel}>
              <h6>Output:</h6>
              <div className={styles.outputContent}>
                <span className={styles.outputLine}>Running code...</span>
                <span className={styles.outputLine}>Data fetched successfully</span>
              </div>
            </div>
          </div>
        );

      case 'analytics':
        return (
          <div className={styles.lensContent}>
            <div className={styles.lensHeader}>
              <h5>📈 Analytics Dashboard</h5>
              <button className={styles.closeLens} onClick={() => setActiveLens(null)}>
                ×
              </button>
            </div>
            <div className={styles.analyticsGrid}>
              <div className={styles.analyticsCard}>
                <div className={styles.analyticsIcon}>📊</div>
                <h6>Performance</h6>
                <div className={styles.analyticsValue}>98.5%</div>
              </div>
              <div className={styles.analyticsCard}>
                <div className={styles.analyticsIcon}>⚡</div>
                <h6>Speed</h6>
                <div className={styles.analyticsValue}>2.3s</div>
              </div>
              <div className={styles.analyticsCard}>
                <div className={styles.analyticsIcon}>👥</div>
                <h6>Users</h6>
                <div className={styles.analyticsValue}>1,247</div>
              </div>
              <div className={styles.analyticsCard}>
                <div className={styles.analyticsIcon}>🔄</div>
                <h6>Uptime</h6>
                <div className={styles.analyticsValue}>99.9%</div>
              </div>
            </div>
          </div>
        );

      case 'automation':
        return (
          <div className={styles.lensContent}>
            <div className={styles.lensHeader}>
              <h5>🤖 Automation Hub</h5>
              <button className={styles.closeLens} onClick={() => setActiveLens(null)}>
                ×
              </button>
            </div>
            <div className={styles.automationList}>
              <div className={styles.automationItem}>
                <div className={styles.automationIcon}>📧</div>
                <div className={styles.automationInfo}>
                  <h6>Email Automation</h6>
                  <p>Auto-respond to customer inquiries</p>
                </div>
                <div className={styles.automationStatus}>Active</div>
              </div>
              <div className={styles.automationItem}>
                <div className={styles.automationIcon}>📊</div>
                <div className={styles.automationInfo}>
                  <h6>Data Sync</h6>
                  <p>Sync data across platforms</p>
                </div>
                <div className={styles.automationStatus}>Active</div>
              </div>
              <div className={styles.automationItem}>
                <div className={styles.automationIcon}>🔔</div>
                <div className={styles.automationInfo}>
                  <h6>Notifications</h6>
                  <p>Smart alert system</p>
                </div>
                <div className={styles.automationStatus}>Paused</div>
              </div>
            </div>
          </div>
        );

      case 'security':
        return (
          <div className={styles.lensContent}>
            <div className={styles.lensHeader}>
              <h5>🔒 Security Center</h5>
              <button className={styles.closeLens} onClick={() => setActiveLens(null)}>
                ×
              </button>
            </div>
            <div className={styles.securityGrid}>
              <div className={styles.securityCard}>
                <div className={styles.securityIcon}>🛡️</div>
                <h6>Access Control</h6>
                <p>Manage user permissions</p>
                <button className={styles.securityBtn}>Configure</button>
              </div>
              <div className={styles.securityCard}>
                <div className={styles.securityIcon}>🔐</div>
                <h6>Encryption</h6>
                <p>Data protection settings</p>
                <button className={styles.securityBtn}>Settings</button>
              </div>
              <div className={styles.securityCard}>
                <div className={styles.securityIcon}>📋</div>
                <h6>Audit Log</h6>
                <p>Activity monitoring</p>
                <button className={styles.securityBtn}>View Log</button>
              </div>
              <div className={styles.securityCard}>
                <div className={styles.securityIcon}>🚨</div>
                <h6>Alerts</h6>
                <p>Security notifications</p>
                <button className={styles.securityBtn}>Configure</button>
              </div>
            </div>
          </div>
        );

      case 'integration':
        return (
          <div className={styles.lensContent}>
            <div className={styles.lensHeader}>
              <h5>🔌 Integration Hub</h5>
              <button className={styles.closeLens} onClick={() => setActiveLens(null)}>
                ×
              </button>
            </div>
            <div className={styles.integrationList}>
              <div className={styles.integrationItem}>
                <div className={styles.integrationIcon}>📊</div>
                <div className={styles.integrationInfo}>
                  <h6>Google Analytics</h6>
                  <p>Connected • Last sync: 2 min ago</p>
                </div>
                <div className={styles.integrationStatus}>Active</div>
              </div>
              <div className={styles.integrationItem}>
                <div className={styles.integrationIcon}>💳</div>
                <div className={styles.integrationInfo}>
                  <h6>Stripe</h6>
                  <p>Connected • Last sync: 5 min ago</p>
                </div>
                <div className={styles.integrationStatus}>Active</div>
              </div>
              <div className={styles.integrationItem}>
                <div className={styles.integrationIcon}>📧</div>
                <div className={styles.integrationInfo}>
                  <h6>Mailchimp</h6>
                  <p>Disconnected</p>
                </div>
                <div className={styles.integrationStatus}>Inactive</div>
              </div>
              <div className={styles.integrationItem}>
                <div className={styles.integrationIcon}>☁️</div>
                <div className={styles.integrationInfo}>
                  <h6>AWS S3</h6>
                  <p>Connected • Last sync: 1 min ago</p>
                </div>
                <div className={styles.integrationStatus}>Active</div>
              </div>
            </div>
          </div>
        );

      case 'settings':
        return (
          <div className={styles.lensContent}>
            <div className={styles.lensHeader}>
              <h5>⚙️ Settings</h5>
              <button className={styles.closeLens} onClick={() => setActiveLens(null)}>
                ×
              </button>
            </div>
            <div className={styles.settingsGrid}>
              {/* Theme Settings */}
              <div className={styles.settingsCard}>
                <div className={styles.settingsIcon}>🎨</div>
                <h6>Theme Settings</h6>
                <p>Customize interface appearance</p>
                <div className={styles.themeControls}>
                  <button
                    className={`${styles.themeControlBtn} ${theme === 'light' ? styles.active : ''}`}
                    onClick={() => onThemeChange && onThemeChange('light')}
                  >
                    ☀️ Light
                  </button>
                  <button
                    className={`${styles.themeControlBtn} ${theme === 'dark' ? styles.active : ''}`}
                    onClick={() => onThemeChange && onThemeChange('dark')}
                  >
                    🌙 Dark
                  </button>
                  <button
                    className={`${styles.themeControlBtn} ${theme === 'null' ? styles.active : ''}`}
                    onClick={() => onThemeChange && onThemeChange('cyber')}
                  >
                    ⚡ Cyber
                  </button>
                </div>
              </div>
              
              {/* Social Links */}
              <div className={styles.settingsCard}>
                <div className={styles.settingsIcon}>🌍</div>
                <h6>Social Links</h6>
                <p>Connect with the community</p>
                <div className={styles.socialLinks}>
                  <a
                    href="https://x.com/Nullblock_io"
                    target="_blank"
                    rel="noopener noreferrer"
                    className={styles.socialLinkBtn}
                  >
                    𝕏 Follow on X
                  </a>
                  <a
                    href="https://discord.gg/nullblock"
                    target="_blank"
                    rel="noopener noreferrer"
                    className={styles.socialLinkBtn}
                  >
                    🎮 Discord
                  </a>
                  <a
                    href="https://github.com/nullblock-io"
                    target="_blank"
                    rel="noopener noreferrer"
                    className={styles.socialLinkBtn}
                  >
                    💻 GitHub
                  </a>
                </div>
              </div>
              
              {/* Documentation */}
              <div className={styles.settingsCard}>
                <div className={styles.settingsIcon}>📚</div>
                <h6>Documentation</h6>
                <p>Learn about Nullblock features</p>
                <a
                  href="https://aetherbytes.github.io/nullblock-sdk/"
                  target="_blank"
                  rel="noopener noreferrer"
                  className={styles.docsBtn}
                >
                  📚 View Docs
                </a>
              </div>
              
              {/* Version Info */}
              <div className={styles.settingsCard}>
                <div className={styles.settingsIcon}>📝</div>
                <h6>Version</h6>
                <p>Current build information</p>
                <div className={styles.versionInfo}>
                  <span>Nullblock v0.8.17</span>
                  <span>Build: Alpha</span>
                </div>
              </div>
            </div>
          </div>
        );

      default:
        return (
          <div className={styles.lensContent}>
            <div className={styles.lensHeader}>
              <h5>💡 {lensId.charAt(0).toUpperCase() + lensId.slice(1)}</h5>
              <button className={styles.closeLens} onClick={() => setActiveLens(null)}>
                ×
              </button>
            </div>
            <div className={styles.lensPlaceholder}>
              <p>This lens feature is coming soon...</p>
              <button className={styles.comingSoonBtn}>Notify When Ready</button>
            </div>
          </div>
        );
    }
  };

  const renderTasks = () => (
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
                <span className={styles.progressText}>{task.progress}%</span>
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

  const renderMCPOperations = () => (
    <div className={styles.mcpContainer}>
      <div className={styles.mcpHeader}>
        <h3>MCP Operations</h3>
        <div className={styles.mcpStats}>
          <span className={styles.stat}>
            Active: {mcpOperations.filter((op) => op.status === 'active').length}
          </span>
          <span className={styles.stat}>
            Idle: {mcpOperations.filter((op) => op.status === 'idle').length}
          </span>
        </div>
      </div>
      <div className={styles.mcpList}>
        {mcpOperations.map((operation) => (
          <div
            key={operation.id}
            className={`${styles.mcpItem} ${getStatusColor(operation.status)}`}
          >
            <div className={styles.mcpHeader}>
              <span className={styles.mcpName}>{operation.name}</span>
              <span className={styles.mcpStatus}>{operation.status}</span>
            </div>
            <div className={styles.mcpDetails}>
              <span className={styles.mcpEndpoint}>{operation.endpoint}</span>
              <span className={styles.mcpLastActivity}>
                Last: {operation.lastActivity.toLocaleTimeString()}
              </span>
              {operation.responseTime && (
                <span className={styles.mcpResponseTime}>Response: {operation.responseTime}ms</span>
              )}
            </div>
          </div>
        ))}
      </div>
    </div>
  );

  const renderLogs = () => (
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

  const renderAgents = () => (
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
      </div>
    </div>
  );

  const renderHecate = () => (
    <div className={styles.hecateContainer}>
      <div className={styles.hecateContent}>
        <div className={styles.hecateMain}>
          <div className={styles.hecateAvatar}>
            <div className={styles.avatarCircle}>
              <div
                className={`${styles.nullviewAvatar} ${styles[nullviewState]} ${styles.clickableNulleye}`}
                onClick={handleNullViewClick}
              >
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

          <div className={styles.hecateStats}>
            <div className={styles.statCard}>
              <div className={styles.statValue}>
                {getUserStats().walletName || getUserStats().walletAddress}
              </div>
              <div className={styles.statLabel}>Wallet Identity</div>
            </div>
            <div className={styles.statCard}>
              <div className={styles.statValue}>{getUserStats().walletType}</div>
              <div className={styles.statLabel}>Wallet Type</div>
            </div>
            <div className={styles.statCard}>
              <div className={styles.statValue}>{getUserStats().sessionDuration}</div>
              <div className={styles.statLabel}>Session Time</div>
            </div>
            <div className={styles.statCard}>
              <div className={styles.statValue}>{getUserStats().connectionStatus}</div>
              <div className={styles.statLabel}>Connection Status</div>
            </div>
          </div>
        </div>

        <div className={styles.hecateInterface}>
          <div className={styles.chatSection}>
            <div className={styles.hecateChat}>
              <div className={styles.chatHeader}>
                <h4>Chat with Hecate</h4>
                <span className={styles.chatStatus}>Live</span>
              </div>

              <div className={styles.chatMessages}>
                {chatMessages.map((message) => (
                  <div
                    key={message.id}
                    className={`${styles.chatMessage} ${styles[`message-${message.sender}`]} ${message.type ? styles[`type-${message.type}`] : ''}`}
                  >
                    <div className={styles.messageHeader}>
                      <span className={styles.messageSender}>
                        {message.sender === 'hecate' ? (
                          <span className={styles.hecateMessageSender}>
                            <div
                              className={`${styles.nullviewChat} ${styles[`chat-${message.type || 'base'}`]} ${styles.clickableNulleyeChat}`}
                              onClick={handleNullViewClick}
                            >
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
                  placeholder="Ask Hecate anything..."
                  className={styles.chatInputField}
                />
                <button type="submit" className={styles.chatSendButton}>
                  <span>➤</span>
                </button>
              </form>

              {showSuggestions && (
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

          <div className={styles.lensSection}>
            {activeLens ? (
              <div className={styles.lensExpanded}>{renderLensContent(activeLens)}</div>
            ) : (
              <div className={styles.lensScrollContainer}>
                <div className={styles.lensInfoPanel}>
                  <div className={styles.lensInfoContent}>
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

                    <div className={styles.lensAppsSection}>
                      <div className={styles.lensAppsGrid}>
                        {lensOptions.map((lens) => (
                          <button
                            key={lens.id}
                            className={styles.lensAppButton}
                            onClick={() => handleLensClick(lens.id)}
                            style={{ '--lens-color': lens.color } as React.CSSProperties}
                          >
                            <span className={styles.lensAppIcon}>{lens.icon}</span>
                            <span className={styles.lensAppTitle}>{lens.title}</span>
                          </button>
                        ))}
                      </div>
                    </div>
                    
                    {/* Branding */}
                    <div className={styles.scopesBranding}>
                      <span className={styles.brandingText}>NULLBLOCK</span>
                    </div>
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );

  return (
    <div
      className={`${styles.contextDashboard} ${theme === 'dark' ? styles.dark : theme === 'light' ? styles.light : ''} ${styles[`theme-${theme}`] || ''}`}
    >
      <div className={styles.dashboardHeader}>
        <div className={styles.headerLeft}>
          <h2>NullView</h2>
          <span className={styles.subtitle}>System Overview & Agentic Interface</span>
        </div>
        <button className={styles.closeButton} onClick={onClose}>
          ×
        </button>
      </div>

      <div className={styles.dashboardTabs}>
        <button
          className={`${styles.tab} ${activeTab === 'tasks' ? styles.active : ''}`}
          onClick={() => setActiveTab('tasks')}
        >
          Tasks ({tasks.length})
        </button>
        <button
          className={`${styles.tab} ${activeTab === 'mcp' ? styles.active : ''}`}
          onClick={() => setActiveTab('mcp')}
        >
          MCP ({mcpOperations.length})
        </button>
        <button
          className={`${styles.tab} ${activeTab === 'logs' ? styles.active : ''}`}
          onClick={() => setActiveTab('logs')}
        >
          Logs ({logs.length})
        </button>
        <button
          className={`${styles.tab} ${activeTab === 'agents' ? styles.active : ''}`}
          onClick={() => setActiveTab('agents')}
        >
          Agents
        </button>
        <button
          className={`${styles.tab} ${activeTab === 'hecate' ? styles.active : ''}`}
          onClick={() => setActiveTab('hecate')}
        >
          Hecate
        </button>
      </div>

      <div className={styles.dashboardContent}>
        {activeTab === 'tasks' && renderTasks()}
        {activeTab === 'mcp' && renderMCPOperations()}
        {activeTab === 'logs' && renderLogs()}
        {activeTab === 'agents' && renderAgents()}
        {activeTab === 'hecate' && renderHecate()}
      </div>
    </div>
  );
};

export default HecateHud;
