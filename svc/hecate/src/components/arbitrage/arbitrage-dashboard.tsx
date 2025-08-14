import React, { useState, useEffect } from 'react';
import styles from './arbitrage-dashboard.module.scss';
import {
  findArbitrageOpportunities,
  executeArbitrageTrade,
  getArbitrageHistory,
  getArbitrageMetrics,
  updateTradingSettings,
  isAuthenticated,
  handleMCPError
} from '../../common/services/mcp-api';

interface ArbitrageOpportunity {
  id: string;
  token_pair: string;
  buy_dex: string;
  sell_dex: string;
  buy_price: number;
  sell_price: number;
  profit_percentage: number;
  profit_amount: number;
  trade_amount: number;
  gas_cost: number;
  net_profit: number;
  confidence: number;
  timestamp: string;
}

interface ArbitrageMetrics {
  total_trades: number;
  successful_trades: number;
  success_rate: number;
  total_profit: number;
  total_gas_spent: number;
  net_profit: number;
  average_profit_per_trade: number;
  largest_profit: number;
  largest_loss: number;
}

interface ArbitrageDashboardProps {
  onClose: () => void;
}

const ArbitrageDashboard: React.FC<ArbitrageDashboardProps> = ({ onClose }) => {
  const [opportunities, setOpportunities] = useState<ArbitrageOpportunity[]>([]);
  const [metrics, setMetrics] = useState<ArbitrageMetrics | null>(null);
  const [history, setHistory] = useState<any[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<'opportunities' | 'history' | 'metrics' | 'settings'>('opportunities');
  
  // Settings state
  const [settings, setSettings] = useState({
    min_profit_threshold: 0.5,
    max_trade_amount: 1000,
    risk_tolerance: 'medium',
    preferred_dexes: ['uniswap', 'sushiswap'],
    enable_mev_protection: true
  });

  useEffect(() => {
    if (!isAuthenticated()) {
      setError('Please authenticate with MCP first');
      return;
    }
    
    loadOpportunities();
    loadMetrics();
    loadHistory();
  }, []);

  const loadOpportunities = async () => {
    try {
      setLoading(true);
      setError(null);
      
      const response = await findArbitrageOpportunities({
        min_profit_percentage: settings.min_profit_threshold,
        max_trade_amount: settings.max_trade_amount
      });
      
      if (response.success && response.result?.opportunities) {
        setOpportunities(response.result.opportunities);
      } else {
        setError(response.message || 'Failed to load opportunities');
      }
    } catch (err) {
      setError(handleMCPError(err));
    } finally {
      setLoading(false);
    }
  };

  const loadMetrics = async () => {
    try {
      const response = await getArbitrageMetrics();
      if (response.success && response.result) {
        setMetrics(response.result);
      }
    } catch (err) {
      console.error('Failed to load metrics:', err);
    }
  };

  const loadHistory = async () => {
    try {
      const response = await getArbitrageHistory();
      if (response.success && response.result?.history) {
        setHistory(response.result.history);
      }
    } catch (err) {
      console.error('Failed to load history:', err);
    }
  };

  const handleExecuteTrade = async (opportunity: ArbitrageOpportunity) => {
    try {
      setLoading(true);
      setError(null);
      
      const response = await executeArbitrageTrade({
        opportunity_id: opportunity.id,
        trade_amount: opportunity.trade_amount,
        max_slippage: 0.5
      });
      
      if (response.success) {
        // Refresh data after successful trade
        await loadOpportunities();
        await loadMetrics();
        await loadHistory();
        
        // Show success message
        alert(`Trade executed successfully! ${response.protected ? 'MEV protection was used.' : ''}`);
      } else {
        setError(response.message || 'Trade execution failed');
      }
    } catch (err) {
      setError(handleMCPError(err));
    } finally {
      setLoading(false);
    }
  };

  const handleUpdateSettings = async () => {
    try {
      setLoading(true);
      setError(null);
      
      const response = await updateTradingSettings(settings);
      
      if (response.success) {
        // Refresh opportunities with new settings
        await loadOpportunities();
        alert('Settings updated successfully!');
      } else {
        setError(response.message || 'Failed to update settings');
      }
    } catch (err) {
      setError(handleMCPError(err));
    } finally {
      setLoading(false);
    }
  };

  const renderOpportunities = () => (
    <div className={styles.opportunitiesTab}>
      <div className={styles.tabHeader}>
        <h3>Arbitrage Opportunities</h3>
        <button 
          className={styles.refreshButton} 
          onClick={loadOpportunities}
          disabled={loading}
        >
          {loading ? 'Loading...' : 'Refresh'}
        </button>
      </div>
      
      {error && <div className={styles.error}>{error}</div>}
      
      <div className={styles.opportunitiesList}>
        {opportunities.length === 0 ? (
          <div className={styles.emptyState}>
            <p>No arbitrage opportunities found</p>
            <p>Adjust your settings to find more opportunities</p>
          </div>
        ) : (
          opportunities.map((opp) => (
            <div key={opp.id} className={styles.opportunityCard}>
              <div className={styles.opportunityHeader}>
                <h4>{opp.token_pair}</h4>
                <div className={`${styles.confidence} ${opp.confidence > 0.7 ? styles.high : opp.confidence > 0.5 ? styles.medium : styles.low}`}>
                  {(opp.confidence * 100).toFixed(0)}% confidence
                </div>
              </div>
              
              <div className={styles.opportunityDetails}>
                <div className={styles.route}>
                  <span className={styles.buy}>Buy: {opp.buy_dex}</span>
                  <span className={styles.arrow}>→</span>
                  <span className={styles.sell}>Sell: {opp.sell_dex}</span>
                </div>
                
                <div className={styles.metrics}>
                  <div className={styles.metric}>
                    <span className={styles.label}>Profit:</span>
                    <span className={styles.value}>{opp.profit_percentage.toFixed(2)}%</span>
                  </div>
                  <div className={styles.metric}>
                    <span className={styles.label}>Amount:</span>
                    <span className={styles.value}>${opp.profit_amount.toFixed(2)}</span>
                  </div>
                  <div className={styles.metric}>
                    <span className={styles.label}>Gas:</span>
                    <span className={styles.value}>${opp.gas_cost.toFixed(2)}</span>
                  </div>
                  <div className={styles.metric}>
                    <span className={styles.label}>Net:</span>
                    <span className={`${styles.value} ${opp.net_profit > 0 ? styles.profit : styles.loss}`}>
                      ${opp.net_profit.toFixed(2)}
                    </span>
                  </div>
                </div>
                
                <div className={styles.prices}>
                  <span>Buy: ${opp.buy_price.toFixed(4)}</span>
                  <span>Sell: ${opp.sell_price.toFixed(4)}</span>
                </div>
              </div>
              
              <div className={styles.opportunityActions}>
                <button 
                  className={styles.executeButton}
                  onClick={() => handleExecuteTrade(opp)}
                  disabled={loading || opp.net_profit <= 0}
                >
                  Execute Trade
                </button>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );

  const renderMetrics = () => (
    <div className={styles.metricsTab}>
      <h3>Performance Metrics</h3>
      
      {metrics ? (
        <div className={styles.metricsGrid}>
          <div className={styles.metricCard}>
            <h4>Total Trades</h4>
            <div className={styles.metricValue}>{metrics.total_trades}</div>
          </div>
          
          <div className={styles.metricCard}>
            <h4>Success Rate</h4>
            <div className={styles.metricValue}>{(metrics.success_rate * 100).toFixed(1)}%</div>
          </div>
          
          <div className={styles.metricCard}>
            <h4>Net Profit</h4>
            <div className={`${styles.metricValue} ${metrics.net_profit >= 0 ? styles.profit : styles.loss}`}>
              ${metrics.net_profit.toFixed(2)}
            </div>
          </div>
          
          <div className={styles.metricCard}>
            <h4>Avg Profit/Trade</h4>
            <div className={styles.metricValue}>${metrics.average_profit_per_trade.toFixed(2)}</div>
          </div>
          
          <div className={styles.metricCard}>
            <h4>Largest Profit</h4>
            <div className={`${styles.metricValue} ${styles.profit}`}>
              ${metrics.largest_profit.toFixed(2)}
            </div>
          </div>
          
          <div className={styles.metricCard}>
            <h4>Total Gas Spent</h4>
            <div className={styles.metricValue}>${metrics.total_gas_spent.toFixed(2)}</div>
          </div>
        </div>
      ) : (
        <div className={styles.emptyState}>
          <p>No metrics available</p>
          <p>Execute some trades to see performance data</p>
        </div>
      )}
    </div>
  );

  const renderHistory = () => (
    <div className={styles.historyTab}>
      <h3>Trade History</h3>
      
      {history.length === 0 ? (
        <div className={styles.emptyState}>
          <p>No trade history</p>
          <p>Your executed trades will appear here</p>
        </div>
      ) : (
        <div className={styles.historyList}>
          {history.map((trade, index) => (
            <div key={index} className={styles.historyItem}>
              <div className={styles.tradeHeader}>
                <span className={styles.pair}>{trade.token_pair}</span>
                <span className={`${styles.status} ${styles[trade.status]}`}>
                  {trade.status}
                </span>
              </div>
              
              <div className={styles.tradeDetails}>
                <span>Profit: ${trade.actual_profit?.toFixed(2) || 'N/A'}</span>
                <span>Gas: ${trade.gas_used?.toFixed(2) || 'N/A'}</span>
                <span>Time: {new Date(trade.executed_at).toLocaleString()}</span>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );

  const renderSettings = () => (
    <div className={styles.settingsTab}>
      <h3>Trading Settings</h3>
      
      <div className={styles.settingsForm}>
        <div className={styles.settingGroup}>
          <label>Minimum Profit Threshold (%)</label>
          <input
            type="number"
            step="0.1"
            value={settings.min_profit_threshold}
            onChange={(e) => setSettings(prev => ({
              ...prev,
              min_profit_threshold: parseFloat(e.target.value)
            }))}
          />
        </div>
        
        <div className={styles.settingGroup}>
          <label>Maximum Trade Amount ($)</label>
          <input
            type="number"
            value={settings.max_trade_amount}
            onChange={(e) => setSettings(prev => ({
              ...prev,
              max_trade_amount: parseFloat(e.target.value)
            }))}
          />
        </div>
        
        <div className={styles.settingGroup}>
          <label>Risk Tolerance</label>
          <select
            value={settings.risk_tolerance}
            onChange={(e) => setSettings(prev => ({
              ...prev,
              risk_tolerance: e.target.value
            }))}
          >
            <option value="low">Low</option>
            <option value="medium">Medium</option>
            <option value="high">High</option>
          </select>
        </div>
        
        <div className={styles.settingGroup}>
          <label>Preferred DEXes</label>
          <div className={styles.checkboxGroup}>
            {['uniswap', 'sushiswap', 'balancer', 'curve'].map(dex => (
              <label key={dex} className={styles.checkbox}>
                <input
                  type="checkbox"
                  checked={settings.preferred_dexes.includes(dex)}
                  onChange={(e) => {
                    const newDexes = e.target.checked 
                      ? [...settings.preferred_dexes, dex]
                      : settings.preferred_dexes.filter(d => d !== dex);
                    setSettings(prev => ({
                      ...prev,
                      preferred_dexes: newDexes
                    }));
                  }}
                />
                {dex.charAt(0).toUpperCase() + dex.slice(1)}
              </label>
            ))}
          </div>
        </div>
        
        <div className={styles.settingGroup}>
          <label className={styles.checkbox}>
            <input
              type="checkbox"
              checked={settings.enable_mev_protection}
              onChange={(e) => setSettings(prev => ({
                ...prev,
                enable_mev_protection: e.target.checked
              }))}
            />
            Enable MEV Protection (Flashbots)
          </label>
        </div>
        
        <button 
          className={styles.saveButton}
          onClick={handleUpdateSettings}
          disabled={loading}
        >
          {loading ? 'Saving...' : 'Save Settings'}
        </button>
      </div>
    </div>
  );

  return (
    <div className={styles.arbitrageDashboard}>
      <div className={styles.header}>
        <h2>Arbitrage Trading Dashboard</h2>
        <button className={styles.closeButton} onClick={onClose}>×</button>
      </div>
      
      <div className={styles.tabs}>
        <button 
          className={`${styles.tab} ${activeTab === 'opportunities' ? styles.active : ''}`}
          onClick={() => setActiveTab('opportunities')}
        >
          Opportunities
        </button>
        <button 
          className={`${styles.tab} ${activeTab === 'metrics' ? styles.active : ''}`}
          onClick={() => setActiveTab('metrics')}
        >
          Metrics
        </button>
        <button 
          className={`${styles.tab} ${activeTab === 'history' ? styles.active : ''}`}
          onClick={() => setActiveTab('history')}
        >
          History
        </button>
        <button 
          className={`${styles.tab} ${activeTab === 'settings' ? styles.active : ''}`}
          onClick={() => setActiveTab('settings')}
        >
          Settings
        </button>
      </div>
      
      <div className={styles.tabContent}>
        {activeTab === 'opportunities' && renderOpportunities()}
        {activeTab === 'metrics' && renderMetrics()}
        {activeTab === 'history' && renderHistory()}
        {activeTab === 'settings' && renderSettings()}
      </div>
    </div>
  );
};

export default ArbitrageDashboard;