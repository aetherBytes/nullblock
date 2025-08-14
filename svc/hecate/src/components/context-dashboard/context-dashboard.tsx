import React, { useState, useEffect, useRef } from 'react';
import styles from './context-dashboard.module.scss';

interface ContextDashboardProps {
  onClose: () => void;
  theme?: 'null' | 'light';
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

const ContextDashboard: React.FC<ContextDashboardProps> = ({ onClose, theme = 'light' }) => {
  const [activeTab, setActiveTab] = useState<'tasks' | 'mcp' | 'logs' | 'agents'>('tasks');
  const [tasks, setTasks] = useState<Task[]>([]);
  const [mcpOperations, setMcpOperations] = useState<MCPOperation[]>([]);
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [agents, setAgents] = useState<any[]>([]);
  const [autoScroll, setAutoScroll] = useState(true);
  const logsEndRef = useRef<HTMLDivElement>(null);
  const [searchTerm, setSearchTerm] = useState('');
  const [logFilter, setLogFilter] = useState<'all' | 'info' | 'warning' | 'error' | 'success' | 'debug'>('all');

  // Mock data for demonstration
  useEffect(() => {
    // Initialize with mock data
    const mockTasks: Task[] = [
      {
        id: '1',
        name: 'Arbitrage Opportunity Scan',
        status: 'running',
        type: 'trading',
        description: 'Scanning DEXes for arbitrage opportunities across Uniswap V3, SushiSwap, and PancakeSwap',
        startTime: new Date(Date.now() - 300000),
        progress: 65,
        logs: [
          { id: '1', timestamp: new Date(), level: 'info', source: 'arbitrage-agent', message: 'Scanning Uniswap V3 pools' },
          { id: '2', timestamp: new Date(), level: 'info', source: 'arbitrage-agent', message: 'Found 3 potential opportunities' },
          { id: '3', timestamp: new Date(Date.now() - 1000), level: 'success', source: 'arbitrage-agent', message: 'Analyzing MEV protection requirements' }
        ]
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
        logs: [
          { id: '4', timestamp: new Date(Date.now() - 350000), level: 'success', source: 'sentiment-agent', message: 'Analysis completed successfully' },
          { id: '5', timestamp: new Date(Date.now() - 400000), level: 'info', source: 'sentiment-agent', message: 'Processed 1,247 social media posts' }
        ]
      },
      {
        id: '3',
        name: 'Portfolio Rebalancing',
        status: 'pending',
        type: 'system',
        description: 'Automated portfolio rebalancing based on market conditions and risk parameters',
        startTime: new Date(Date.now() - 100000),
        logs: []
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
          { id: '6', timestamp: new Date(), level: 'info', source: 'flashbots-agent', message: 'Calculating optimal gas prices' },
          { id: '7', timestamp: new Date(Date.now() - 2000), level: 'warning', source: 'flashbots-agent', message: 'High network congestion detected' }
        ]
      }
    ];

    const mockMcpOperations: MCPOperation[] = [
      {
        id: '1',
        name: 'Flashbots Bundle',
        status: 'active',
        endpoint: '/flashbots/bundle',
        lastActivity: new Date(),
        responseTime: 150
      },
      {
        id: '2',
        name: 'MEV Protection',
        status: 'active',
        endpoint: '/mev/protect',
        lastActivity: new Date(Date.now() - 5000),
        responseTime: 89
      },
      {
        id: '3',
        name: 'Social Trading Signals',
        status: 'idle',
        endpoint: '/social/signals',
        lastActivity: new Date(Date.now() - 30000)
      },
      {
        id: '4',
        name: 'Portfolio Analytics',
        status: 'active',
        endpoint: '/portfolio/analytics',
        lastActivity: new Date(Date.now() - 2000),
        responseTime: 234
      },
      {
        id: '5',
        name: 'Risk Assessment',
        status: 'active',
        endpoint: '/risk/assessment',
        lastActivity: new Date(Date.now() - 1000),
        responseTime: 67
      },
      {
        id: '6',
        name: 'Market Data Feed',
        status: 'active',
        endpoint: '/market/feed',
        lastActivity: new Date(),
        responseTime: 12
      }
    ];

    const mockLogs: LogEntry[] = [
      { id: '1', timestamp: new Date(), level: 'info', source: 'system', message: 'Context dashboard initialized' },
      { id: '2', timestamp: new Date(Date.now() - 1000), level: 'info', source: 'mcp-server', message: 'MCP connection established' },
      { id: '3', timestamp: new Date(Date.now() - 2000), level: 'success', source: 'arbitrage-agent', message: 'Arbitrage opportunity detected' },
      { id: '4', timestamp: new Date(Date.now() - 3000), level: 'warning', source: 'portfolio-agent', message: 'Portfolio variance threshold exceeded' },
      { id: '5', timestamp: new Date(Date.now() - 4000), level: 'error', source: 'social-agent', message: 'Failed to fetch Twitter sentiment data' },
      { id: '6', timestamp: new Date(Date.now() - 5000), level: 'debug', source: 'system', message: 'Memory usage: 45%' },
      { id: '7', timestamp: new Date(Date.now() - 6000), level: 'info', source: 'flashbots-agent', message: 'Bundle submitted to Flashbots relay' },
      { id: '8', timestamp: new Date(Date.now() - 7000), level: 'success', source: 'risk-manager', message: 'Risk assessment completed - portfolio within limits' },
      { id: '9', timestamp: new Date(Date.now() - 8000), level: 'info', source: 'market-data', message: 'Updated price feeds for 1,247 tokens' },
      { id: '10', timestamp: new Date(Date.now() - 9000), level: 'warning', source: 'network-monitor', message: 'High gas prices detected on Ethereum mainnet' },
      { id: '11', timestamp: new Date(Date.now() - 10000), level: 'debug', source: 'system', message: 'CPU usage: 23% | Network latency: 45ms' }
    ];

    setTasks(mockTasks);
    setMcpOperations(mockMcpOperations);
    setLogs(mockLogs);

    // Simulate real-time log updates
    const interval = setInterval(() => {
      const logMessages = [
        { level: 'info', source: 'market-data', message: 'Price update for ETH/USD: $2,847.32' },
        { level: 'info', source: 'arbitrage-agent', message: 'Scanning for new opportunities...' },
        { level: 'success', source: 'flashbots-agent', message: 'Bundle accepted by relay' },
        { level: 'warning', source: 'network-monitor', message: 'Gas price fluctuation detected' },
        { level: 'debug', source: 'system', message: 'Memory usage: 47% | Active connections: 12' },
        { level: 'info', source: 'portfolio-agent', message: 'Portfolio value: $12,847.32 (+2.3%)' },
        { level: 'success', source: 'risk-manager', message: 'Risk assessment passed' },
        { level: 'info', source: 'social-agent', message: 'Processing 23 new social signals' }
      ];
      
      const randomLog = logMessages[Math.floor(Math.random() * logMessages.length)];
      const newLog: LogEntry = {
        id: Date.now().toString(),
        timestamp: new Date(),
        level: randomLog.level as any,
        source: randomLog.source,
        message: randomLog.message
      };
      setLogs(prev => [...prev, newLog]);
    }, 4000);

    // Simulate task progress updates
    const progressInterval = setInterval(() => {
      setTasks(prev => prev.map(task => {
        if (task.status === 'running' && task.progress !== undefined && task.progress < 100) {
          return {
            ...task,
            progress: Math.min(100, task.progress + Math.random() * 5)
          };
        }
        return task;
      }));
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

  const filteredLogs = logs.filter(log => {
    const matchesSearch = log.message.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         log.source.toLowerCase().includes(searchTerm.toLowerCase());
    const matchesFilter = logFilter === 'all' || log.level === logFilter;
    return matchesSearch && matchesFilter;
  });

  const renderTasks = () => (
    <div className={styles.tasksContainer}>
      <div className={styles.tasksHeader}>
        <h3>Active Tasks</h3>
        <div className={styles.taskStats}>
          <span className={styles.stat}>
            Running: {tasks.filter(t => t.status === 'running').length}
          </span>
          <span className={styles.stat}>
            Completed: {tasks.filter(t => t.status === 'completed').length}
          </span>
          <span className={styles.stat}>
            Failed: {tasks.filter(t => t.status === 'failed').length}
          </span>
        </div>
      </div>
      <div className={styles.tasksList}>
        {tasks.map(task => (
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
                <div 
                  className={styles.progressFill} 
                  style={{ width: `${task.progress}%` }}
                ></div>
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
            Active: {mcpOperations.filter(op => op.status === 'active').length}
          </span>
          <span className={styles.stat}>
            Idle: {mcpOperations.filter(op => op.status === 'idle').length}
          </span>
        </div>
      </div>
      <div className={styles.mcpList}>
        {mcpOperations.map(operation => (
          <div key={operation.id} className={`${styles.mcpItem} ${getStatusColor(operation.status)}`}>
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
                <span className={styles.mcpResponseTime}>
                  Response: {operation.responseTime}ms
                </span>
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
        {filteredLogs.map(log => (
          <div key={log.id} className={`${styles.logEntry} ${getLogLevelColor(log.level)}`}>
            <div className={styles.logTimestamp}>
              {log.timestamp.toLocaleTimeString()}
            </div>
            <div className={styles.logLevel}>
              {log.level.toUpperCase()}
            </div>
            <div className={styles.logSource}>
              {log.source}
            </div>
            <div className={styles.logMessage}>
              {log.message}
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

  return (
    <div className={`${styles.contextDashboard} ${styles[`theme-${theme}`]}`}>
      <div className={styles.dashboardHeader}>
        <div className={styles.headerLeft}>
          <h2>Context Dashboard</h2>
          <span className={styles.subtitle}>System Operations & Agent Communications</span>
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
      </div>

      <div className={styles.dashboardContent}>
        {activeTab === 'tasks' && renderTasks()}
        {activeTab === 'mcp' && renderMCPOperations()}
        {activeTab === 'logs' && renderLogs()}
        {activeTab === 'agents' && renderAgents()}
      </div>
    </div>
  );
};

export default ContextDashboard;
