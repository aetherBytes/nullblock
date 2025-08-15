import React, { useState, useEffect, useCallback } from 'react';
import styles from './social-trading-dashboard.module.scss';

interface SocialSignal {
  id: string;
  source: 'twitter' | 'gmgn' | 'dextools';
  tokenSymbol: string;
  tokenAddress?: string;
  sentimentScore: number;
  engagementScore: number;
  content: string;
  author: string;
  timestamp: Date;
  url?: string;
}

interface TradingSignal {
  tokenSymbol: string;
  signalType: 'BUY' | 'SELL' | 'HOLD';
  strength: number;
  confidence: number;
  sentimentScore: number;
  priceTarget?: number;
  stopLoss?: number;
  reasoning: string[];
}

interface PositionSizing {
  tokenSymbol: string;
  recommendedSizeUsd: number;
  finalSizeUsd: number;
  positionPercentage: number;
  riskLevel: 'LOW' | 'MEDIUM' | 'HIGH' | 'EXTREME';
}

interface SocialTradingDashboardProps {
  onClose: () => void;
}

const SocialTradingDashboard: React.FC<SocialTradingDashboardProps> = ({ onClose }) => {
  const [activeView, setActiveView] = useState<'signals' | 'sentiment' | 'trades' | 'portfolio'>(
    'signals',
  );
  const [isConnected, setIsConnected] = useState(false);
  const [socialSignals, setSocialSignals] = useState<SocialSignal[]>([]);
  const [tradingSignals, setTradingSignals] = useState<TradingSignal[]>([]);
  const [positions, setPositions] = useState<PositionSizing[]>([]);
  const [selectedToken, setSelectedToken] = useState<string>('');
  const [lastUpdate, setLastUpdate] = useState<Date>(new Date());
  const [isLoading, setIsLoading] = useState(false);

  // Mock data - in production this would come from the social trading agents
  const mockSocialSignals: SocialSignal[] = [
    {
      id: '1',
      source: 'twitter',
      tokenSymbol: 'BONK',
      tokenAddress: 'DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263',
      sentimentScore: 0.8,
      engagementScore: 0.7,
      content: 'BONK is going to the moon! 🚀 This meme coin season is just getting started!',
      author: '@crypto_degen_420',
      timestamp: new Date(Date.now() - 300000), // 5 minutes ago
      url: 'https://twitter.com/crypto_degen_420/status/123',
    },
    {
      id: '2',
      source: 'gmgn',
      tokenSymbol: 'WIF',
      tokenAddress: 'EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm',
      sentimentScore: 0.6,
      engagementScore: 0.9,
      content: 'WIF trending +25% in last hour. Strong momentum detected.',
      author: 'GMGN.ai',
      timestamp: new Date(Date.now() - 600000), // 10 minutes ago
    },
    {
      id: '3',
      source: 'dextools',
      tokenSymbol: 'POPCAT',
      tokenAddress: '7GCihgDB8fe6KNjn2MYtkzZcRjQy3t9GHdC8uHYmW2hr',
      sentimentScore: -0.3,
      engagementScore: 0.4,
      content: 'POPCAT showing bearish divergence on 15min chart. High volatility alert.',
      author: 'DEXTools',
      timestamp: new Date(Date.now() - 900000), // 15 minutes ago
    },
    {
      id: '4',
      source: 'twitter',
      tokenSymbol: 'MYRO',
      tokenAddress: 'HhJpBhRRn4g56VsyLuT8DL5Bv31HkXqsrahTTUCZeZg4',
      sentimentScore: 0.4,
      engagementScore: 0.6,
      content: 'MYRO holders are strong. This dip is just a shakeout before the next leg up.',
      author: '@solana_alpha',
      timestamp: new Date(Date.now() - 1800000), // 30 minutes ago
    },
  ];

  const mockTradingSignals: TradingSignal[] = [
    {
      tokenSymbol: 'BONK',
      signalType: 'BUY',
      strength: 0.8,
      confidence: 0.75,
      sentimentScore: 0.8,
      priceTarget: 0.000028,
      stopLoss: 0.000021,
      reasoning: ['Strong bullish sentiment', 'High social engagement', 'Momentum building'],
    },
    {
      tokenSymbol: 'WIF',
      signalType: 'BUY',
      strength: 0.6,
      confidence: 0.8,
      sentimentScore: 0.6,
      priceTarget: 4.2,
      stopLoss: 2.95,
      reasoning: ['GMGN trending signal', 'Good technical setup', 'Strong volume'],
    },
    {
      tokenSymbol: 'POPCAT',
      signalType: 'HOLD',
      strength: 0.2,
      confidence: 0.4,
      sentimentScore: -0.3,
      reasoning: ['Mixed signals', 'High volatility warning', 'Wait for clearer direction'],
    },
  ];

  const mockPositions: PositionSizing[] = [
    {
      tokenSymbol: 'BONK',
      recommendedSizeUsd: 500,
      finalSizeUsd: 450,
      positionPercentage: 4.5,
      riskLevel: 'MEDIUM',
    },
    {
      tokenSymbol: 'WIF',
      recommendedSizeUsd: 300,
      finalSizeUsd: 300,
      positionPercentage: 3.0,
      riskLevel: 'MEDIUM',
    },
  ];

  useEffect(() => {
    // Initialize with mock data
    setSocialSignals(mockSocialSignals);
    setTradingSignals(mockTradingSignals);
    setPositions(mockPositions);
    setIsConnected(true);

    // Simulate real-time updates
    const interval = setInterval(() => {
      setLastUpdate(new Date());
      // In production, this would fetch new data from the social trading agents
    }, 30000); // Update every 30 seconds

    return () => clearInterval(interval);
  }, []);

  const connectToSocialTrading = useCallback(async () => {
    setIsLoading(true);
    try {
      // In production, this would connect to the social trading MCP server
      await new Promise((resolve) => setTimeout(resolve, 2000)); // Simulate connection
      setIsConnected(true);
    } catch (error) {
      console.error('Failed to connect to social trading:', error);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const getSentimentColor = (score: number): string => {
    if (score > 0.2) {
      return styles.bullish;
    }

    if (score < -0.2) {
      return styles.bearish;
    }

    return styles.neutral;
  };

  const getSignalTypeColor = (type: string): string => {
    switch (type) {
      case 'BUY':
        return styles.buySignal;
      case 'SELL':
        return styles.sellSignal;
      default:
        return styles.holdSignal;
    }
  };

  const getRiskLevelColor = (level: string): string => {
    switch (level) {
      case 'LOW':
        return styles.lowRisk;
      case 'MEDIUM':
        return styles.mediumRisk;
      case 'HIGH':
        return styles.highRisk;
      case 'EXTREME':
        return styles.extremeRisk;
      default:
        return styles.neutral;
    }
  };

  const formatTimeAgo = (timestamp: Date): string => {
    const diff = Date.now() - timestamp.getTime();
    const minutes = Math.floor(diff / 60000);
    const hours = Math.floor(minutes / 60);

    if (hours > 0) {
      return `${hours}h ago`;
    }

    return `${minutes}m ago`;
  };

  const getSourceIcon = (source: string): string => {
    switch (source) {
      case 'twitter':
        return '🐦';
      case 'gmgn':
        return '📊';
      case 'dextools':
        return '🔧';
      default:
        return '📡';
    }
  };

  const renderConnectionStatus = () => (
    <div className={styles.connectionStatus}>
      <div className={styles.statusIndicator}>
        <span
          className={`${styles.statusDot} ${isConnected ? styles.connected : styles.disconnected}`}
        >
        </span>
        <span className={styles.statusText}>
          Social Trading Agent: {isConnected ? 'CONNECTED' : 'DISCONNECTED'}
        </span>
      </div>
      <div className={styles.lastUpdate}>Last Update: {lastUpdate.toLocaleTimeString()}</div>
      {!isConnected && (
        <button
          className={styles.connectButton}
          onClick={connectToSocialTrading}
          disabled={isLoading}
        >
          {isLoading ? 'CONNECTING...' : 'CONNECT TO AGENTS'}
        </button>
      )}
    </div>
  );

  const renderSocialSignals = () => (
    <div className={styles.signalsView}>
      <div className={styles.signalsHeader}>
        <h3>SOCIAL SIGNALS</h3>
        <div className={styles.signalsStats}>
          <span>Total: {socialSignals.length}</span>
          <span>Bullish: {socialSignals.filter((s) => s.sentimentScore > 0.2).length}</span>
          <span>Bearish: {socialSignals.filter((s) => s.sentimentScore < -0.2).length}</span>
        </div>
      </div>

      <div className={styles.signalsList}>
        {socialSignals.map((signal) => (
          <div key={signal.id} className={styles.signalCard}>
            <div className={styles.signalHeader}>
              <div className={styles.signalSource}>
                <span className={styles.sourceIcon}>{getSourceIcon(signal.source)}</span>
                <span className={styles.sourceName}>{signal.source.toUpperCase()}</span>
              </div>
              <div className={styles.signalMeta}>
                <span className={styles.token}>${signal.tokenSymbol}</span>
                <span className={styles.timestamp}>{formatTimeAgo(signal.timestamp)}</span>
              </div>
            </div>

            <div className={styles.signalContent}>
              <p className={styles.signalText}>{signal.content}</p>
              <div className={styles.signalAuthor}>— {signal.author}</div>
            </div>

            <div className={styles.signalMetrics}>
              <div className={styles.metric}>
                <span className={styles.metricLabel}>Sentiment:</span>
                <span
                  className={`${styles.metricValue} ${getSentimentColor(signal.sentimentScore)}`}
                >
                  {signal.sentimentScore > 0 ? '+' : ''}
                  {(signal.sentimentScore * 100).toFixed(1)}%
                </span>
              </div>
              <div className={styles.metric}>
                <span className={styles.metricLabel}>Engagement:</span>
                <span className={styles.metricValue}>
                  {(signal.engagementScore * 100).toFixed(0)}%
                </span>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );

  const renderSentimentAnalysis = () => (
    <div className={styles.sentimentView}>
      <div className={styles.sentimentHeader}>
        <h3>SENTIMENT ANALYSIS</h3>
        <div className={styles.fearGreedIndex}>
          <div className={styles.indexLabel}>Fear & Greed Index</div>
          <div className={styles.indexValue}>72</div>
          <div className={styles.indexLabel}>GREED</div>
        </div>
      </div>

      <div className={styles.sentimentGrid}>
        <div className={styles.sentimentCard}>
          <h4>Token Sentiment</h4>
          <div className={styles.tokenSentiments}>
            {['BONK', 'WIF', 'POPCAT', 'MYRO'].map((token) => {
              const signals = socialSignals.filter((s) => s.tokenSymbol === token);
              const avgSentiment =
                signals.length > 0
                  ? signals.reduce((acc, s) => acc + s.sentimentScore, 0) / signals.length
                  : 0;

              return (
                <div key={token} className={styles.tokenSentiment}>
                  <span className={styles.tokenName}>${token}</span>
                  <div className={styles.sentimentBar}>
                    <div
                      className={`${styles.sentimentFill} ${getSentimentColor(avgSentiment)}`}
                      style={{ width: `${Math.abs(avgSentiment) * 100}%` }}
                    ></div>
                  </div>
                  <span className={`${styles.sentimentScore} ${getSentimentColor(avgSentiment)}`}>
                    {avgSentiment > 0 ? '+' : ''}
                    {(avgSentiment * 100).toFixed(1)}%
                  </span>
                </div>
              );
            })}
          </div>
        </div>

        <div className={styles.sentimentCard}>
          <h4>Market Mood</h4>
          <div className={styles.moodIndicators}>
            <div className={styles.moodItem}>
              <span className={styles.emoji}>🚀</span>
              <span className={styles.moodLabel}>Euphoria</span>
              <span className={styles.moodValue}>25%</span>
            </div>
            <div className={styles.moodItem}>
              <span className={styles.emoji}>📈</span>
              <span className={styles.moodLabel}>Optimism</span>
              <span className={styles.moodValue}>45%</span>
            </div>
            <div className={styles.moodItem}>
              <span className={styles.emoji}>😐</span>
              <span className={styles.moodLabel}>Neutral</span>
              <span className={styles.moodValue}>20%</span>
            </div>
            <div className={styles.moodItem}>
              <span className={styles.emoji}>📉</span>
              <span className={styles.moodLabel}>Fear</span>
              <span className={styles.moodValue}>10%</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );

  const renderTradingSignals = () => (
    <div className={styles.tradesView}>
      <div className={styles.tradesHeader}>
        <h3>TRADING SIGNALS</h3>
        <div className={styles.signalsSummary}>
          <span className={styles.buyCount}>
            BUY: {tradingSignals.filter((s) => s.signalType === 'BUY').length}
          </span>
          <span className={styles.sellCount}>
            SELL: {tradingSignals.filter((s) => s.signalType === 'SELL').length}
          </span>
          <span className={styles.holdCount}>
            HOLD: {tradingSignals.filter((s) => s.signalType === 'HOLD').length}
          </span>
        </div>
      </div>

      <div className={styles.signalsList}>
        {tradingSignals.map((signal) => (
          <div key={signal.tokenSymbol} className={styles.tradeSignalCard}>
            <div className={styles.signalHeader}>
              <div className={styles.tokenInfo}>
                <span className={styles.tokenSymbol}>${signal.tokenSymbol}</span>
                <span className={`${styles.signalType} ${getSignalTypeColor(signal.signalType)}`}>
                  {signal.signalType}
                </span>
              </div>
              <div className={styles.signalStrength}>
                <div className={styles.strengthBar}>
                  <div
                    className={styles.strengthFill}
                    style={{ width: `${signal.strength * 100}%` }}
                  ></div>
                </div>
                <span className={styles.strengthValue}>{(signal.strength * 100).toFixed(0)}%</span>
              </div>
            </div>

            <div className={styles.signalDetails}>
              <div className={styles.priceTargets}>
                {signal.priceTarget && (
                  <div className={styles.target}>
                    <span className={styles.targetLabel}>Target:</span>
                    <span className={styles.targetValue}>${signal.priceTarget.toFixed(6)}</span>
                  </div>
                )}
                {signal.stopLoss && (
                  <div className={styles.target}>
                    <span className={styles.targetLabel}>Stop Loss:</span>
                    <span className={styles.targetValue}>${signal.stopLoss.toFixed(6)}</span>
                  </div>
                )}
              </div>

              <div className={styles.reasoning}>
                <span className={styles.reasoningLabel}>Reasoning:</span>
                <ul className={styles.reasoningList}>
                  {signal.reasoning.slice(0, 3).map((reason, index) => (
                    <li key={index}>{reason}</li>
                  ))}
                </ul>
              </div>
            </div>

            <div className={styles.signalFooter}>
              <div className={styles.confidence}>
                Confidence:{' '}
                <span className={styles.confidenceValue}>
                  {(signal.confidence * 100).toFixed(0)}%
                </span>
              </div>
              <button
                className={styles.viewDetailButton}
                onClick={() => setSelectedToken(signal.tokenSymbol)}
              >
                View Details
              </button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );

  const renderPortfolio = () => (
    <div className={styles.portfolioView}>
      <div className={styles.portfolioHeader}>
        <h3>POSITION SIZING</h3>
        <div className={styles.portfolioStats}>
          <span>Total Positions: {positions.length}</span>
          <span>
            Total Allocation:{' '}
            {positions.reduce((acc, p) => acc + p.positionPercentage, 0).toFixed(1)}%
          </span>
        </div>
      </div>

      <div className={styles.positionsList}>
        {positions.map((position) => (
          <div key={position.tokenSymbol} className={styles.positionCard}>
            <div className={styles.positionHeader}>
              <span className={styles.tokenSymbol}>${position.tokenSymbol}</span>
              <span className={`${styles.riskLevel} ${getRiskLevelColor(position.riskLevel)}`}>
                {position.riskLevel} RISK
              </span>
            </div>

            <div className={styles.positionDetails}>
              <div className={styles.positionSize}>
                <span className={styles.sizeLabel}>Position Size:</span>
                <span className={styles.sizeValue}>${position.finalSizeUsd}</span>
                <span className={styles.sizePercentage}>({position.positionPercentage}%)</span>
              </div>

              <div className={styles.allocationBar}>
                <div
                  className={styles.allocationFill}
                  style={{ width: `${position.positionPercentage * 4}%` }} // Scale for visibility
                ></div>
              </div>
            </div>

            <div className={styles.positionActions}>
              <button className={styles.executeButton} disabled={position.riskLevel === 'EXTREME'}>
                {position.riskLevel === 'EXTREME' ? 'TOO RISKY' : 'EXECUTE TRADE'}
              </button>
              <button className={styles.adjustButton}>Adjust Size</button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );

  return (
    <div className={styles.socialTradingDashboard}>
      <div className={styles.dashboardHeader}>
        <div className={styles.titleSection}>
          <h2>SOCIAL TRADING COMMAND CENTER</h2>
          <button className={styles.closeButton} onClick={onClose}>
            ×
          </button>
        </div>
        {renderConnectionStatus()}
      </div>

      <div className={styles.navigationTabs}>
        <button
          className={`${styles.navTab} ${activeView === 'signals' ? styles.active : ''}`}
          onClick={() => setActiveView('signals')}
        >
          📡 SIGNALS
        </button>
        <button
          className={`${styles.navTab} ${activeView === 'sentiment' ? styles.active : ''}`}
          onClick={() => setActiveView('sentiment')}
        >
          🧠 SENTIMENT
        </button>
        <button
          className={`${styles.navTab} ${activeView === 'trades' ? styles.active : ''}`}
          onClick={() => setActiveView('trades')}
        >
          ⚡ TRADES
        </button>
        <button
          className={`${styles.navTab} ${activeView === 'portfolio' ? styles.active : ''}`}
          onClick={() => setActiveView('portfolio')}
        >
          💰 PORTFOLIO
        </button>
      </div>

      <div className={styles.dashboardContent}>
        {activeView === 'signals' && renderSocialSignals()}
        {activeView === 'sentiment' && renderSentimentAnalysis()}
        {activeView === 'trades' && renderTradingSignals()}
        {activeView === 'portfolio' && renderPortfolio()}
      </div>
    </div>
  );
};

export default SocialTradingDashboard;
