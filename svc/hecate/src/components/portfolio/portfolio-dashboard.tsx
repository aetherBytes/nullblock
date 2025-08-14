import React, { useState, useEffect } from 'react';
import styles from './portfolio-dashboard.module.scss';

interface Asset {
  symbol: string;
  name: string;
  address: string;
  balance: number;
  usdValue: number;
  price: number;
  change24h: number;
  allocation: number;
  chain: 'ethereum' | 'solana' | 'polygon';
}

interface PortfolioStats {
  totalValue: number;
  change24h: number;
  change24hPercent: number;
  totalAssets: number;
  diversificationScore: number;
  riskScore: number;
}

interface RebalanceRecommendation {
  asset: string;
  currentAllocation: number;
  targetAllocation: number;
  action: 'BUY' | 'SELL' | 'HOLD';
  amount: number;
  reasoning: string;
}

interface PortfolioDashboardProps {
  onClose: () => void;
}

const PortfolioDashboard: React.FC<PortfolioDashboardProps> = ({ onClose }) => {
  const [activeView, setActiveView] = useState<'overview' | 'assets' | 'rebalance' | 'analytics'>('overview');
  const [assets, setAssets] = useState<Asset[]>([]);
  const [portfolioStats, setPortfolioStats] = useState<PortfolioStats | null>(null);
  const [rebalanceRecommendations, setRebalanceRecommendations] = useState<RebalanceRecommendation[]>([]);
  const [selectedChain, setSelectedChain] = useState<'all' | 'ethereum' | 'solana' | 'polygon'>('all');
  // const [isLoading, setIsLoading] = useState(false);
  const [lastUpdate, setLastUpdate] = useState<Date>(new Date());

  // Mock portfolio data
  const mockAssets: Asset[] = [
    {
      symbol: 'SOL',
      name: 'Solana',
      address: 'So11111111111111111111111111111111111111112',
      balance: 15.4,
      usdValue: 3234.56,
      price: 210.03,
      change24h: 5.23,
      allocation: 32.1,
      chain: 'solana'
    },
    {
      symbol: 'BONK',
      name: 'Bonk',
      address: 'DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263',
      balance: 2500000,
      usdValue: 1456.78,
      price: 0.0000583,
      change24h: -2.15,
      allocation: 14.5,
      chain: 'solana'
    },
    {
      symbol: 'ETH',
      name: 'Ethereum',
      address: '0x0000000000000000000000000000000000000000',
      balance: 2.1,
      usdValue: 5432.10,
      price: 2587.67,
      change24h: 3.45,
      allocation: 53.9,
      chain: 'ethereum'
    },
    {
      symbol: 'USDC',
      name: 'USD Coin',
      address: 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v',
      balance: 500.0,
      usdValue: 500.0,
      price: 1.0,
      change24h: 0.01,
      allocation: 5.0,
      chain: 'solana'
    }
  ];

  const mockStats: PortfolioStats = {
    totalValue: 10623.44,
    change24h: 287.35,
    change24hPercent: 2.78,
    totalAssets: 4,
    diversificationScore: 7.2,
    riskScore: 6.8
  };

  const mockRebalanceRecommendations: RebalanceRecommendation[] = [
    {
      asset: 'SOL',
      currentAllocation: 32.1,
      targetAllocation: 25.0,
      action: 'SELL',
      amount: 2.3,
      reasoning: 'Reduce concentration risk by trimming overweight position'
    },
    {
      asset: 'ETH',
      currentAllocation: 53.9,
      targetAllocation: 50.0,
      action: 'SELL',
      amount: 0.15,
      reasoning: 'Slight rebalancing to maintain strategic allocation'
    },
    {
      asset: 'USDC',
      currentAllocation: 5.0,
      targetAllocation: 10.0,
      action: 'BUY',
      amount: 531.17,
      reasoning: 'Increase stablecoin allocation for better risk management'
    }
  ];

  useEffect(() => {
    // Initialize with mock data
    setAssets(mockAssets);
    setPortfolioStats(mockStats);
    setRebalanceRecommendations(mockRebalanceRecommendations);

    // Simulate real-time price updates
    const interval = setInterval(() => {
      setLastUpdate(new Date());
      // In production, this would fetch real price data
      setAssets(prevAssets => 
        prevAssets.map(asset => ({
          ...asset,
          price: asset.price * (0.99 + Math.random() * 0.02), // ±1% random fluctuation
          change24h: -5 + Math.random() * 10 // Random 24h change between -5% and +5%
        }))
      );
    }, 30000); // Update every 30 seconds

    return () => clearInterval(interval);
  }, []);

  const filteredAssets = selectedChain === 'all' 
    ? assets 
    : assets.filter(asset => asset.chain === selectedChain);

  const getChangeColor = (change: number): string => {
    if (change > 0) return styles.positive;
    if (change < 0) return styles.negative;
    return styles.neutral;
  };

  const getChainIcon = (chain: string): string => {
    switch (chain) {
      case 'ethereum': return '⟠';
      case 'solana': return '◎';
      case 'polygon': return '⬟';
      default: return '🔗';
    }
  };

  const getActionColor = (action: string): string => {
    switch (action) {
      case 'BUY': return styles.buyAction;
      case 'SELL': return styles.sellAction;
      default: return styles.holdAction;
    }
  };

  const getRiskScoreColor = (score: number): string => {
    if (score >= 8) return styles.highRisk;
    if (score >= 6) return styles.mediumRisk;
    return styles.lowRisk;
  };

  const renderOverview = () => (
    <div className={styles.overviewView}>
      <div className={styles.statsGrid}>
        <div className={styles.statCard}>
          <h4>Total Portfolio Value</h4>
          <div className={styles.statValue}>${portfolioStats?.totalValue.toLocaleString()}</div>
          <div className={`${styles.statChange} ${getChangeColor(portfolioStats?.change24hPercent || 0)}`}>
            {portfolioStats?.change24hPercent && portfolioStats.change24hPercent > 0 ? '+' : ''}
            {portfolioStats?.change24hPercent.toFixed(2)}% 
            (${portfolioStats?.change24h.toFixed(2)})
          </div>
        </div>
        
        <div className={styles.statCard}>
          <h4>Total Assets</h4>
          <div className={styles.statValue}>{portfolioStats?.totalAssets}</div>
          <div className={styles.statLabel}>Across {new Set(assets.map(a => a.chain)).size} chains</div>
        </div>
        
        <div className={styles.statCard}>
          <h4>Diversification Score</h4>
          <div className={styles.statValue}>{portfolioStats?.diversificationScore.toFixed(1)}/10</div>
          <div className={styles.statLabel}>
            {(portfolioStats?.diversificationScore || 0) >= 7 ? 'Well Diversified' : 'Needs Improvement'}
          </div>
        </div>
        
        <div className={styles.statCard}>
          <h4>Risk Score</h4>
          <div className={`${styles.statValue} ${getRiskScoreColor(portfolioStats?.riskScore || 0)}`}>
            {portfolioStats?.riskScore.toFixed(1)}/10
          </div>
          <div className={styles.statLabel}>
            {(portfolioStats?.riskScore || 0) >= 8 ? 'High Risk' : 
             (portfolioStats?.riskScore || 0) >= 6 ? 'Medium Risk' : 'Low Risk'}
          </div>
        </div>
      </div>
      
      <div className={styles.allocationChart}>
        <h4>Asset Allocation</h4>
        <div className={styles.pieChart}>
          {/* Simple pie chart representation */}
          <div className={styles.allocationList}>
            {assets.map((asset, index) => (
              <div key={asset.symbol} className={styles.allocationItem}>
                <div className={styles.allocationColor} style={{ backgroundColor: `hsl(${index * 60}, 70%, 50%)` }}></div>
                <span className={styles.allocationSymbol}>{asset.symbol}</span>
                <span className={styles.allocationPercent}>{asset.allocation.toFixed(1)}%</span>
                <span className={styles.allocationValue}>${asset.usdValue.toFixed(2)}</span>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );

  const renderAssets = () => (
    <div className={styles.assetsView}>
      <div className={styles.assetsHeader}>
        <h3>Portfolio Assets</h3>
        <div className={styles.chainFilters}>
          <button 
            className={`${styles.chainFilter} ${selectedChain === 'all' ? styles.active : ''}`}
            onClick={() => setSelectedChain('all')}
          >
            All Chains
          </button>
          <button 
            className={`${styles.chainFilter} ${selectedChain === 'ethereum' ? styles.active : ''}`}
            onClick={() => setSelectedChain('ethereum')}
          >
            ⟠ Ethereum
          </button>
          <button 
            className={`${styles.chainFilter} ${selectedChain === 'solana' ? styles.active : ''}`}
            onClick={() => setSelectedChain('solana')}
          >
            ◎ Solana
          </button>
        </div>
      </div>
      
      <div className={styles.assetsList}>
        {filteredAssets.map(asset => (
          <div key={asset.symbol} className={styles.assetCard}>
            <div className={styles.assetHeader}>
              <div className={styles.assetInfo}>
                <span className={styles.chainIcon}>{getChainIcon(asset.chain)}</span>
                <div className={styles.assetDetails}>
                  <span className={styles.assetSymbol}>{asset.symbol}</span>
                  <span className={styles.assetName}>{asset.name}</span>
                </div>
              </div>
              <div className={styles.assetAllocation}>
                {asset.allocation.toFixed(1)}%
              </div>
            </div>
            
            <div className={styles.assetMetrics}>
              <div className={styles.metricRow}>
                <span className={styles.metricLabel}>Balance:</span>
                <span className={styles.metricValue}>{asset.balance.toLocaleString()}</span>
              </div>
              <div className={styles.metricRow}>
                <span className={styles.metricLabel}>Price:</span>
                <span className={styles.metricValue}>
                  ${asset.price < 0.01 ? asset.price.toExponential(3) : asset.price.toFixed(4)}
                </span>
              </div>
              <div className={styles.metricRow}>
                <span className={styles.metricLabel}>24h Change:</span>
                <span className={`${styles.metricValue} ${getChangeColor(asset.change24h)}`}>
                  {asset.change24h > 0 ? '+' : ''}{asset.change24h.toFixed(2)}%
                </span>
              </div>
              <div className={styles.metricRow}>
                <span className={styles.metricLabel}>USD Value:</span>
                <span className={styles.metricValue}>${asset.usdValue.toFixed(2)}</span>
              </div>
            </div>
            
            <div className={styles.assetActions}>
              <button className={styles.tradeButton}>Trade</button>
              <button className={styles.detailsButton}>Details</button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );

  const renderRebalance = () => (
    <div className={styles.rebalanceView}>
      <div className={styles.rebalanceHeader}>
        <h3>Portfolio Rebalancing</h3>
        <div className={styles.rebalanceStats}>
          <span>Recommendations: {rebalanceRecommendations.length}</span>
          <span>Estimated Gas: $12.50</span>
        </div>
      </div>
      
      <div className={styles.recommendationsList}>
        {rebalanceRecommendations.map(rec => (
          <div key={rec.asset} className={styles.recommendationCard}>
            <div className={styles.recHeader}>
              <span className={styles.recAsset}>{rec.asset}</span>
              <span className={`${styles.recAction} ${getActionColor(rec.action)}`}>
                {rec.action}
              </span>
            </div>
            
            <div className={styles.recDetails}>
              <div className={styles.allocationChange}>
                <span className={styles.currentAllocation}>
                  Current: {rec.currentAllocation.toFixed(1)}%
                </span>
                <span className={styles.arrow}>→</span>
                <span className={styles.targetAllocation}>
                  Target: {rec.targetAllocation.toFixed(1)}%
                </span>
              </div>
              
              <div className={styles.recAmount}>
                Amount: {rec.amount.toLocaleString()} {rec.asset !== 'USDC' ? rec.asset : ''}
              </div>
              
              <div className={styles.recReasoning}>
                {rec.reasoning}
              </div>
            </div>
            
            <div className={styles.recActions}>
              <button className={styles.executeButton}>
                Execute {rec.action}
              </button>
              <button className={styles.skipButton}>Skip</button>
            </div>
          </div>
        ))}
      </div>
      
      <div className={styles.rebalanceActions}>
        <button className={styles.executeAllButton}>
          Execute All Recommendations
        </button>
        <button className={styles.customRebalanceButton}>
          Custom Rebalance
        </button>
      </div>
    </div>
  );

  const renderAnalytics = () => (
    <div className={styles.analyticsView}>
      <div className={styles.analyticsHeader}>
        <h3>Portfolio Analytics</h3>
        <div className={styles.analyticsFilters}>
          <button className={styles.timeFilter}>1D</button>
          <button className={styles.timeFilter}>7D</button>
          <button className={styles.timeFilter}>30D</button>
          <button className={`${styles.timeFilter} ${styles.active}`}>90D</button>
        </div>
      </div>
      
      <div className={styles.analyticsGrid}>
        <div className={styles.analyticsCard}>
          <h4>Performance vs Benchmarks</h4>
          <div className={styles.benchmarkComparison}>
            <div className={styles.benchmarkItem}>
              <span className={styles.benchmarkName}>Your Portfolio</span>
              <span className={`${styles.benchmarkValue} ${styles.positive}`}>+12.4%</span>
            </div>
            <div className={styles.benchmarkItem}>
              <span className={styles.benchmarkName}>SOL</span>
              <span className={`${styles.benchmarkValue} ${styles.positive}`}>+8.2%</span>
            </div>
            <div className={styles.benchmarkItem}>
              <span className={styles.benchmarkName}>ETH</span>
              <span className={`${styles.benchmarkValue} ${styles.positive}`}>+15.1%</span>
            </div>
            <div className={styles.benchmarkItem}>
              <span className={styles.benchmarkName}>DeFi Index</span>
              <span className={`${styles.benchmarkValue} ${styles.positive}`}>+9.8%</span>
            </div>
          </div>
        </div>
        
        <div className={styles.analyticsCard}>
          <h4>Risk Metrics</h4>
          <div className={styles.riskMetrics}>
            <div className={styles.riskMetric}>
              <span className={styles.riskLabel}>Volatility (30d):</span>
              <span className={styles.riskValue}>18.4%</span>
            </div>
            <div className={styles.riskMetric}>
              <span className={styles.riskLabel}>Beta vs SOL:</span>
              <span className={styles.riskValue}>1.12</span>
            </div>
            <div className={styles.riskMetric}>
              <span className={styles.riskLabel}>Max Drawdown:</span>
              <span className={`${styles.riskValue} ${styles.negative}`}>-15.2%</span>
            </div>
            <div className={styles.riskMetric}>
              <span className={styles.riskLabel}>Sharpe Ratio:</span>
              <span className={styles.riskValue}>1.34</span>
            </div>
          </div>
        </div>
        
        <div className={styles.analyticsCard}>
          <h4>Chain Distribution</h4>
          <div className={styles.chainDistribution}>
            <div className={styles.chainItem}>
              <span className={styles.chainIcon}>◎</span>
              <span className={styles.chainName}>Solana</span>
              <span className={styles.chainValue}>$5,191.34</span>
              <span className={styles.chainPercent}>48.9%</span>
            </div>
            <div className={styles.chainItem}>
              <span className={styles.chainIcon}>⟠</span>
              <span className={styles.chainName}>Ethereum</span>
              <span className={styles.chainValue}>$5,432.10</span>
              <span className={styles.chainPercent}>51.1%</span>
            </div>
          </div>
        </div>
        
        <div className={styles.analyticsCard}>
          <h4>Top Performers</h4>
          <div className={styles.performers}>
            <div className={styles.performerItem}>
              <span className={styles.performerRank}>#1</span>
              <span className={styles.performerSymbol}>SOL</span>
              <span className={`${styles.performerReturn} ${styles.positive}`}>+5.23%</span>
            </div>
            <div className={styles.performerItem}>
              <span className={styles.performerRank}>#2</span>
              <span className={styles.performerSymbol}>ETH</span>
              <span className={`${styles.performerReturn} ${styles.positive}`}>+3.45%</span>
            </div>
            <div className={styles.performerItem}>
              <span className={styles.performerRank}>#3</span>
              <span className={styles.performerSymbol}>USDC</span>
              <span className={`${styles.performerReturn} ${styles.neutral}`}>+0.01%</span>
            </div>
            <div className={styles.performerItem}>
              <span className={styles.performerRank}>#4</span>
              <span className={styles.performerSymbol}>BONK</span>
              <span className={`${styles.performerReturn} ${styles.negative}`}>-2.15%</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );

  return (
    <div className={styles.portfolioDashboard}>
      <div className={styles.dashboardHeader}>
        <div className={styles.titleSection}>
          <h2>PORTFOLIO COMMAND CENTER</h2>
          <button className={styles.closeButton} onClick={onClose}>×</button>
        </div>
        <div className={styles.connectionStatus}>
          <div className={styles.statusIndicator}>
            <span className={`${styles.statusDot} ${styles.connected}`}></span>
            <span className={styles.statusText}>Portfolio Agent: CONNECTED</span>
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
          className={`${styles.navTab} ${activeView === 'assets' ? styles.active : ''}`}
          onClick={() => setActiveView('assets')}
        >
          💰 ASSETS
        </button>
        <button 
          className={`${styles.navTab} ${activeView === 'rebalance' ? styles.active : ''}`}
          onClick={() => setActiveView('rebalance')}
        >
          ⚖️ REBALANCE
        </button>
        <button 
          className={`${styles.navTab} ${activeView === 'analytics' ? styles.active : ''}`}
          onClick={() => setActiveView('analytics')}
        >
          📈 ANALYTICS
        </button>
      </div>
      
      <div className={styles.dashboardContent}>
        {activeView === 'overview' && renderOverview()}
        {activeView === 'assets' && renderAssets()}
        {activeView === 'rebalance' && renderRebalance()}
        {activeView === 'analytics' && renderAnalytics()}
      </div>
    </div>
  );
};

export default PortfolioDashboard;