import React, { useState, useEffect } from 'react';
import styles from './defi-dashboard.module.scss';

interface YieldFarmingStrategy {
  id: string;
  protocol: 'aave' | 'compound' | 'uniswap' | 'sushiswap' | 'curve' | 'yearn';
  type: 'lending' | 'liquidity_pool' | 'staking' | 'yield_farming';
  asset: string;
  pair?: string;
  currentApy: number;
  tvl: number;
  risk: 'LOW' | 'MEDIUM' | 'HIGH' | 'EXTREME';
  autoCompound: boolean;
  minDeposit: number;
  active: boolean;
  allocatedAmount: number;
  earnings: number;
  lastUpdate: Date;
}

interface AutomatedStrategy {
  id: string;
  name: string;
  description: string;
  triggers: string[];
  actions: string[];
  status: 'active' | 'paused' | 'stopped';
  lastExecution: Date | null;
  profitLoss: number;
  executionCount: number;
}

interface LiquidityPosition {
  id: string;
  protocol: string;
  pair: string;
  amount0: number;
  amount1: number;
  totalValue: number;
  fees24h: number;
  apr: number;
  impermanentLoss: number;
  range: {
    min: number;
    max: number;
    current: number;
    inRange: boolean;
  };
}

interface DeFiMetrics {
  totalDeposited: number;
  totalEarnings: number;
  averageApy: number;
  activePositions: number;
  impermanentLoss: number;
  gasSpent: number;
}

interface DeFiDashboardProps {
  onClose: () => void;
}

const DeFiDashboard: React.FC<DeFiDashboardProps> = ({ onClose }) => {
  const [activeView, setActiveView] = useState<'overview' | 'yield' | 'liquidity' | 'automation'>('overview');
  const [yieldStrategies, setYieldStrategies] = useState<YieldFarmingStrategy[]>([]);
  const [automatedStrategies, setAutomatedStrategies] = useState<AutomatedStrategy[]>([]);
  const [liquidityPositions, setLiquidityPositions] = useState<LiquidityPosition[]>([]);
  const [defiMetrics, setDeFiMetrics] = useState<DeFiMetrics | null>(null);
  const [selectedProtocol, setSelectedProtocol] = useState<'all' | 'aave' | 'uniswap' | 'compound'>('all');
  // const [isLoading, setIsLoading] = useState(false);
  const [lastUpdate, setLastUpdate] = useState<Date>(new Date());

  // Mock yield farming strategies
  const mockYieldStrategies: YieldFarmingStrategy[] = [
    {
      id: '1',
      protocol: 'aave',
      type: 'lending',
      asset: 'USDC',
      currentApy: 4.2,
      tvl: 1200000000,
      risk: 'LOW',
      autoCompound: true,
      minDeposit: 100,
      active: true,
      allocatedAmount: 5000,
      earnings: 125.50,
      lastUpdate: new Date(Date.now() - 3600000)
    },
    {
      id: '2',
      protocol: 'uniswap',
      type: 'liquidity_pool',
      asset: 'ETH',
      pair: 'ETH/USDC',
      currentApy: 12.8,
      tvl: 450000000,
      risk: 'MEDIUM',
      autoCompound: true,
      minDeposit: 1000,
      active: true,
      allocatedAmount: 3000,
      earnings: 89.30,
      lastUpdate: new Date(Date.now() - 7200000)
    },
    {
      id: '3',
      protocol: 'compound',
      type: 'lending',
      asset: 'DAI',
      currentApy: 5.1,
      tvl: 800000000,
      risk: 'LOW',
      autoCompound: false,
      minDeposit: 50,
      active: false,
      allocatedAmount: 0,
      earnings: 0,
      lastUpdate: new Date(Date.now() - 86400000)
    },
    {
      id: '4',
      protocol: 'curve',
      type: 'yield_farming',
      asset: 'USDT',
      pair: '3Pool',
      currentApy: 8.7,
      tvl: 350000000,
      risk: 'MEDIUM',
      autoCompound: true,
      minDeposit: 200,
      active: true,
      allocatedAmount: 2500,
      earnings: 67.25,
      lastUpdate: new Date(Date.now() - 1800000)
    }
  ];

  const mockAutomatedStrategies: AutomatedStrategy[] = [
    {
      id: '1',
      name: 'APY Hunter',
      description: 'Automatically moves funds to the highest yielding stable strategies',
      triggers: ['APY > 5%', 'Risk = LOW', 'TVL > $100M'],
      actions: ['Withdraw from current position', 'Deposit to new strategy', 'Update portfolio allocation'],
      status: 'active',
      lastExecution: new Date(Date.now() - 14400000),
      profitLoss: 234.50,
      executionCount: 7
    },
    {
      id: '2',
      name: 'IL Protection',
      description: 'Exits liquidity positions when impermanent loss exceeds 2%',
      triggers: ['IL > 2%', 'Price divergence > 10%'],
      actions: ['Exit liquidity position', 'Convert to stable assets', 'Alert user'],
      status: 'active',
      lastExecution: null,
      profitLoss: -45.20,
      executionCount: 0
    },
    {
      id: '3',
      name: 'Gas Optimizer',
      description: 'Compounds rewards when gas fees are below optimal threshold',
      triggers: ['Gas < 30 gwei', 'Pending rewards > $10'],
      actions: ['Compound all rewards', 'Rebalance if needed'],
      status: 'paused',
      lastExecution: new Date(Date.now() - 43200000),
      profitLoss: 89.75,
      executionCount: 12
    }
  ];

  const mockLiquidityPositions: LiquidityPosition[] = [
    {
      id: '1',
      protocol: 'Uniswap V3',
      pair: 'ETH/USDC',
      amount0: 1.25,
      amount1: 3234.50,
      totalValue: 6468.75,
      fees24h: 12.50,
      apr: 15.2,
      impermanentLoss: -0.8,
      range: {
        min: 2500,
        max: 2800,
        current: 2587,
        inRange: true
      }
    },
    {
      id: '2',
      protocol: 'SushiSwap',
      pair: 'WBTC/ETH',
      amount0: 0.05,
      amount1: 0.95,
      totalValue: 4521.30,
      fees24h: 8.75,
      apr: 11.8,
      impermanentLoss: 1.2,
      range: {
        min: 15.5,
        max: 17.2,
        current: 16.8,
        inRange: true
      }
    }
  ];

  const mockMetrics: DeFiMetrics = {
    totalDeposited: 10500,
    totalEarnings: 376.30,
    averageApy: 7.8,
    activePositions: 4,
    impermanentLoss: -15.60,
    gasSpent: 145.80
  };

  useEffect(() => {
    // Initialize with mock data
    setYieldStrategies(mockYieldStrategies);
    setAutomatedStrategies(mockAutomatedStrategies);
    setLiquidityPositions(mockLiquidityPositions);
    setDeFiMetrics(mockMetrics);

    // Simulate real-time updates
    const interval = setInterval(() => {
      setLastUpdate(new Date());
      // Update APY values with small fluctuations
      setYieldStrategies(prevStrategies =>
        prevStrategies.map(strategy => ({
          ...strategy,
          currentApy: strategy.currentApy * (0.98 + Math.random() * 0.04) // ±2% fluctuation
        }))
      );
    }, 30000); // Update every 30 seconds

    return () => clearInterval(interval);
  }, []);

  const getProtocolIcon = (protocol: string): string => {
    switch (protocol.toLowerCase()) {
      case 'aave': return '👻';
      case 'compound': return '🏛️';
      case 'uniswap': return '🦄';
      case 'sushiswap': return '🍣';
      case 'curve': return '🌀';
      case 'yearn': return '💙';
      default: return '🔗';
    }
  };

  const getRiskColor = (risk: string): string => {
    switch (risk) {
      case 'LOW': return styles.lowRisk;
      case 'MEDIUM': return styles.mediumRisk;
      case 'HIGH': return styles.highRisk;
      case 'EXTREME': return styles.extremeRisk;
      default: return styles.neutral;
    }
  };

  const getStatusColor = (status: string): string => {
    switch (status) {
      case 'active': return styles.active;
      case 'paused': return styles.paused;
      case 'stopped': return styles.stopped;
      default: return styles.neutral;
    }
  };

  const formatTimeAgo = (date: Date): string => {
    const diff = Date.now() - date.getTime();
    const hours = Math.floor(diff / 3600000);
    const days = Math.floor(hours / 24);
    
    if (days > 0) return `${days}d ago`;
    if (hours > 0) return `${hours}h ago`;
    return 'Just now';
  };

  const renderOverview = () => (
    <div className={styles.overviewView}>
      <div className={styles.metricsGrid}>
        <div className={styles.metricCard}>
          <h4>Total Deposited</h4>
          <div className={styles.metricValue}>${defiMetrics?.totalDeposited.toLocaleString()}</div>
          <div className={styles.metricLabel}>Across {defiMetrics?.activePositions} positions</div>
        </div>
        
        <div className={styles.metricCard}>
          <h4>Total Earnings</h4>
          <div className={`${styles.metricValue} ${styles.positive}`}>
            ${defiMetrics?.totalEarnings.toFixed(2)}
          </div>
          <div className={styles.metricLabel}>
            {((defiMetrics?.totalEarnings || 0) / (defiMetrics?.totalDeposited || 1) * 100).toFixed(2)}% return
          </div>
        </div>
        
        <div className={styles.metricCard}>
          <h4>Average APY</h4>
          <div className={styles.metricValue}>{defiMetrics?.averageApy.toFixed(1)}%</div>
          <div className={styles.metricLabel}>Weighted by allocation</div>
        </div>
        
        <div className={styles.metricCard}>
          <h4>Net P&L</h4>
          <div className={`${styles.metricValue} ${(defiMetrics?.totalEarnings || 0) + (defiMetrics?.impermanentLoss || 0) - (defiMetrics?.gasSpent || 0) > 0 ? styles.positive : styles.negative}`}>
            ${((defiMetrics?.totalEarnings || 0) + (defiMetrics?.impermanentLoss || 0) - (defiMetrics?.gasSpent || 0)).toFixed(2)}
          </div>
          <div className={styles.metricLabel}>After IL and gas fees</div>
        </div>
      </div>
      
      <div className={styles.positionsOverview}>
        <h4>Active Positions</h4>
        <div className={styles.positionsList}>
          {yieldStrategies.filter(s => s.active).map(strategy => (
            <div key={strategy.id} className={styles.positionCard}>
              <div className={styles.positionHeader}>
                <span className={styles.protocolIcon}>{getProtocolIcon(strategy.protocol)}</span>
                <div className={styles.positionInfo}>
                  <span className={styles.positionAsset}>{strategy.pair || strategy.asset}</span>
                  <span className={styles.positionProtocol}>{strategy.protocol.toUpperCase()}</span>
                </div>
                <div className={styles.positionMetrics}>
                  <span className={styles.apy}>{strategy.currentApy.toFixed(1)}% APY</span>
                  <span className={styles.allocation}>${strategy.allocatedAmount.toLocaleString()}</span>
                </div>
              </div>
              
              <div className={styles.positionProgress}>
                <div className={styles.progressBar}>
                  <div 
                    className={styles.progressFill}
                    style={{ width: `${Math.min(strategy.earnings / strategy.allocatedAmount * 100 * 10, 100)}%` }}
                  ></div>
                </div>
                <span className={styles.earnings}>+${strategy.earnings.toFixed(2)} earned</span>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );

  const renderYieldFarming = () => (
    <div className={styles.yieldView}>
      <div className={styles.yieldHeader}>
        <h3>Yield Farming Strategies</h3>
        <div className={styles.protocolFilters}>
          <button 
            className={`${styles.protocolFilter} ${selectedProtocol === 'all' ? styles.active : ''}`}
            onClick={() => setSelectedProtocol('all')}
          >
            All Protocols
          </button>
          <button 
            className={`${styles.protocolFilter} ${selectedProtocol === 'aave' ? styles.active : ''}`}
            onClick={() => setSelectedProtocol('aave')}
          >
            👻 Aave
          </button>
          <button 
            className={`${styles.protocolFilter} ${selectedProtocol === 'uniswap' ? styles.active : ''}`}
            onClick={() => setSelectedProtocol('uniswap')}
          >
            🦄 Uniswap
          </button>
          <button 
            className={`${styles.protocolFilter} ${selectedProtocol === 'compound' ? styles.active : ''}`}
            onClick={() => setSelectedProtocol('compound')}
          >
            🏛️ Compound
          </button>
        </div>
      </div>
      
      <div className={styles.strategiesList}>
        {yieldStrategies
          .filter(strategy => selectedProtocol === 'all' || strategy.protocol === selectedProtocol)
          .map(strategy => (
          <div key={strategy.id} className={styles.strategyCard}>
            <div className={styles.strategyHeader}>
              <div className={styles.strategyInfo}>
                <span className={styles.protocolIcon}>{getProtocolIcon(strategy.protocol)}</span>
                <div className={styles.strategyDetails}>
                  <h4>{strategy.pair || strategy.asset}</h4>
                  <span className={styles.strategyType}>{strategy.type.replace('_', ' ')}</span>
                  <span className={styles.strategyProtocol}>{strategy.protocol.toUpperCase()}</span>
                </div>
              </div>
              
              <div className={styles.strategyMetrics}>
                <div className={styles.apy}>
                  <span className={styles.apyValue}>{strategy.currentApy.toFixed(1)}%</span>
                  <span className={styles.apyLabel}>APY</span>
                </div>
                <span className={`${styles.risk} ${getRiskColor(strategy.risk)}`}>
                  {strategy.risk} RISK
                </span>
              </div>
            </div>
            
            <div className={styles.strategyStats}>
              <div className={styles.stat}>
                <span className={styles.statLabel}>TVL:</span>
                <span className={styles.statValue}>${(strategy.tvl / 1000000).toFixed(0)}M</span>
              </div>
              <div className={styles.stat}>
                <span className={styles.statLabel}>Min Deposit:</span>
                <span className={styles.statValue}>${strategy.minDeposit}</span>
              </div>
              <div className={styles.stat}>
                <span className={styles.statLabel}>Auto Compound:</span>
                <span className={`${styles.statValue} ${strategy.autoCompound ? styles.positive : styles.negative}`}>
                  {strategy.autoCompound ? 'Yes' : 'No'}
                </span>
              </div>
            </div>
            
            {strategy.active && (
              <div className={styles.activePosition}>
                <div className={styles.allocation}>
                  <span className={styles.allocationLabel}>Allocated:</span>
                  <span className={styles.allocationValue}>${strategy.allocatedAmount.toLocaleString()}</span>
                </div>
                <div className={styles.earnings}>
                  <span className={styles.earningsLabel}>Earnings:</span>
                  <span className={`${styles.earningsValue} ${styles.positive}`}>
                    +${strategy.earnings.toFixed(2)}
                  </span>
                </div>
                <div className={styles.lastUpdate}>
                  Last updated: {formatTimeAgo(strategy.lastUpdate)}
                </div>
              </div>
            )}
            
            <div className={styles.strategyActions}>
              {strategy.active ? (
                <>
                  <button className={styles.withdrawButton}>Withdraw</button>
                  <button className={styles.addButton}>Add Funds</button>
                  <button className={styles.compoundButton}>Compound</button>
                </>
              ) : (
                <>
                  <button className={styles.depositButton}>Deposit</button>
                  <button className={styles.simulateButton}>Simulate</button>
                </>
              )}
            </div>
          </div>
        ))}
      </div>
    </div>
  );

  const renderLiquidity = () => (
    <div className={styles.liquidityView}>
      <div className={styles.liquidityHeader}>
        <h3>Liquidity Positions</h3>
        <div className={styles.liquidityStats}>
          <span>Total Value: ${liquidityPositions.reduce((acc, pos) => acc + pos.totalValue, 0).toLocaleString()}</span>
          <span>24h Fees: +${liquidityPositions.reduce((acc, pos) => acc + pos.fees24h, 0).toFixed(2)}</span>
        </div>
      </div>
      
      <div className={styles.positionsList}>
        {liquidityPositions.map(position => (
          <div key={position.id} className={styles.liquidityCard}>
            <div className={styles.liquidityHeader}>
              <div className={styles.liquidityInfo}>
                <h4>{position.pair}</h4>
                <span className={styles.protocol}>{position.protocol}</span>
              </div>
              
              <div className={styles.rangeStatus}>
                <span className={`${styles.rangeIndicator} ${position.range.inRange ? styles.inRange : styles.outOfRange}`}>
                  {position.range.inRange ? 'IN RANGE' : 'OUT OF RANGE'}
                </span>
              </div>
            </div>
            
            <div className={styles.liquidityDetails}>
              <div className={styles.amounts}>
                <div className={styles.amount}>
                  <span className={styles.amountValue}>{position.amount0.toFixed(4)}</span>
                  <span className={styles.amountToken}>{position.pair.split('/')[0]}</span>
                </div>
                <div className={styles.amount}>
                  <span className={styles.amountValue}>{position.amount1.toFixed(2)}</span>
                  <span className={styles.amountToken}>{position.pair.split('/')[1]}</span>
                </div>
              </div>
              
              <div className={styles.priceRange}>
                <div className={styles.rangeBar}>
                  <div className={styles.rangeIndicators}>
                    <span className={styles.rangeMin}>${position.range.min}</span>
                    <div className={styles.currentPrice}>
                      <span className={styles.currentPriceValue}>${position.range.current}</span>
                      <div className={styles.priceMarker}></div>
                    </div>
                    <span className={styles.rangeMax}>${position.range.max}</span>
                  </div>
                  <div className={styles.rangeTrack}>
                    <div 
                      className={styles.rangeFill}
                      style={{ 
                        left: '0%', 
                        width: '100%',
                        backgroundColor: position.range.inRange ? '#00ff88' : '#ff4444'
                      }}
                    ></div>
                  </div>
                </div>
              </div>
            </div>
            
            <div className={styles.liquidityMetrics}>
              <div className={styles.metric}>
                <span className={styles.metricLabel}>Total Value:</span>
                <span className={styles.metricValue}>${position.totalValue.toLocaleString()}</span>
              </div>
              <div className={styles.metric}>
                <span className={styles.metricLabel}>24h Fees:</span>
                <span className={`${styles.metricValue} ${styles.positive}`}>
                  +${position.fees24h.toFixed(2)}
                </span>
              </div>
              <div className={styles.metric}>
                <span className={styles.metricLabel}>APR:</span>
                <span className={styles.metricValue}>{position.apr.toFixed(1)}%</span>
              </div>
              <div className={styles.metric}>
                <span className={styles.metricLabel}>IL:</span>
                <span className={`${styles.metricValue} ${position.impermanentLoss >= 0 ? styles.positive : styles.negative}`}>
                  {position.impermanentLoss > 0 ? '+' : ''}{position.impermanentLoss.toFixed(2)}%
                </span>
              </div>
            </div>
            
            <div className={styles.liquidityActions}>
              <button className={styles.adjustRangeButton}>Adjust Range</button>
              <button className={styles.collectFeesButton}>Collect Fees</button>
              <button className={styles.removeLiquidityButton}>Remove</button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );

  const renderAutomation = () => (
    <div className={styles.automationView}>
      <div className={styles.automationHeader}>
        <h3>Automated Strategies</h3>
        <button className={styles.createStrategyButton}>Create New Strategy</button>
      </div>
      
      <div className={styles.strategiesList}>
        {automatedStrategies.map(strategy => (
          <div key={strategy.id} className={styles.automationCard}>
            <div className={styles.automationHeader}>
              <div className={styles.automationInfo}>
                <h4>{strategy.name}</h4>
                <p className={styles.automationDescription}>{strategy.description}</p>
              </div>
              
              <div className={styles.automationStatus}>
                <span className={`${styles.statusBadge} ${getStatusColor(strategy.status)}`}>
                  {strategy.status.toUpperCase()}
                </span>
              </div>
            </div>
            
            <div className={styles.automationDetails}>
              <div className={styles.triggers}>
                <h5>Triggers:</h5>
                <ul>
                  {strategy.triggers.map((trigger, index) => (
                    <li key={index}>{trigger}</li>
                  ))}
                </ul>
              </div>
              
              <div className={styles.actions}>
                <h5>Actions:</h5>
                <ul>
                  {strategy.actions.map((action, index) => (
                    <li key={index}>{action}</li>
                  ))}
                </ul>
              </div>
            </div>
            
            <div className={styles.automationMetrics}>
              <div className={styles.metric}>
                <span className={styles.metricLabel}>Executions:</span>
                <span className={styles.metricValue}>{strategy.executionCount}</span>
              </div>
              <div className={styles.metric}>
                <span className={styles.metricLabel}>P&L:</span>
                <span className={`${styles.metricValue} ${strategy.profitLoss >= 0 ? styles.positive : styles.negative}`}>
                  {strategy.profitLoss > 0 ? '+' : ''}${strategy.profitLoss.toFixed(2)}
                </span>
              </div>
              <div className={styles.metric}>
                <span className={styles.metricLabel}>Last Run:</span>
                <span className={styles.metricValue}>
                  {strategy.lastExecution ? formatTimeAgo(strategy.lastExecution) : 'Never'}
                </span>
              </div>
            </div>
            
            <div className={styles.automationActions}>
              {strategy.status === 'active' ? (
                <button className={styles.pauseButton}>Pause</button>
              ) : strategy.status === 'paused' ? (
                <>
                  <button className={styles.resumeButton}>Resume</button>
                  <button className={styles.stopButton}>Stop</button>
                </>
              ) : (
                <button className={styles.startButton}>Start</button>
              )}
              <button className={styles.editButton}>Edit</button>
              <button className={styles.deleteButton}>Delete</button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );

  return (
    <div className={styles.defiDashboard}>
      <div className={styles.dashboardHeader}>
        <div className={styles.titleSection}>
          <h2>DEFI AUTOMATION CENTER</h2>
          <button className={styles.closeButton} onClick={onClose}>×</button>
        </div>
        <div className={styles.connectionStatus}>
          <div className={styles.statusIndicator}>
            <span className={`${styles.statusDot} ${styles.connected}`}></span>
            <span className={styles.statusText}>DeFi Agent: CONNECTED</span>
          </div>
          <div className={styles.lastUpdate}>
            Last Update: {lastUpdate.toLocaleTimeString()}
          </div>
        </div>
      </div>
      
      <div className={styles.navigationTabs}>
        <button 
          className={`${styles.navTab} ${activeView === 'overview' ? styles.active : ''}`}
          onClick={() => setActiveView('overview')}
        >
          📊 OVERVIEW
        </button>
        <button 
          className={`${styles.navTab} ${activeView === 'yield' ? styles.active : ''}`}
          onClick={() => setActiveView('yield')}
        >
          🌾 YIELD
        </button>
        <button 
          className={`${styles.navTab} ${activeView === 'liquidity' ? styles.active : ''}`}
          onClick={() => setActiveView('liquidity')}
        >
          💧 LIQUIDITY
        </button>
        <button 
          className={`${styles.navTab} ${activeView === 'automation' ? styles.active : ''}`}
          onClick={() => setActiveView('automation')}
        >
          🤖 AUTOMATION
        </button>
      </div>
      
      <div className={styles.dashboardContent}>
        {activeView === 'overview' && renderOverview()}
        {activeView === 'yield' && renderYieldFarming()}
        {activeView === 'liquidity' && renderLiquidity()}
        {activeView === 'automation' && renderAutomation()}
      </div>
    </div>
  );
};

export default DeFiDashboard;