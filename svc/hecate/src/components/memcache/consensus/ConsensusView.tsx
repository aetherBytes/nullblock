import React, { useState, useEffect, useCallback } from 'react';
import { arbFarmService } from '../../../common/services/arbfarm-service';
import type {
  ConsensusConfig,
  ConsensusModelConfig,
  ConsensusStatsResponse,
  ConsensusHistoryEntry,
  ConversationLog,
  Recommendation,
  DiscoveredModel,
} from '../../../types/consensus';
import styles from './consensus.module.scss';

type ConsensusTab = 'dashboard' | 'models' | 'history' | 'recommendations' | 'conversations' | 'config' | 'api';

interface ConsensusViewProps {
  embedded?: boolean;
}

const NAV_ITEMS: { id: ConsensusTab; icon: string; label: string; section: string }[] = [
  { id: 'dashboard', icon: '📊', label: 'Dashboard', section: 'Overview' },
  { id: 'models', icon: '🤖', label: 'Models', section: 'Overview' },
  { id: 'history', icon: '📜', label: 'History', section: 'Data' },
  { id: 'recommendations', icon: '💡', label: 'Recommendations', section: 'Data' },
  { id: 'conversations', icon: '💬', label: 'Conversations', section: 'Data' },
  { id: 'config', icon: '⚙️', label: 'Config', section: 'System' },
  { id: 'api', icon: '🔌', label: 'API', section: 'System' },
];

const ConsensusView: React.FC<ConsensusViewProps> = ({ embedded = false }) => {
  const [activeTab, setActiveTab] = useState<ConsensusTab>('dashboard');
  const [config, setConfig] = useState<ConsensusConfig | null>(null);
  const [stats, setStats] = useState<ConsensusStatsResponse | null>(null);
  const [history, setHistory] = useState<ConsensusHistoryEntry[]>([]);
  const [recommendations, setRecommendations] = useState<Recommendation[]>([]);
  const [conversations, setConversations] = useState<ConversationLog[]>([]);
  const [discoveredModels, setDiscoveredModels] = useState<DiscoveredModel[]>([]);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [selectedHistoryItem, setSelectedHistoryItem] = useState<ConsensusHistoryEntry | null>(null);
  const [selectedConversation, setSelectedConversation] = useState<ConversationLog | null>(null);
  const [lastRefresh, setLastRefresh] = useState<Date>(new Date());

  const fetchData = useCallback(async () => {
    try {
      const [configRes, statsRes, historyRes] = await Promise.all([
        arbFarmService.getConsensusConfig(),
        arbFarmService.getConsensusStats(),
        arbFarmService.getConsensusHistory({ limit: 10 }),
      ]);

      if (configRes.success && configRes.data) {
        const configData = configRes.data as { config?: ConsensusConfig } & ConsensusConfig;
        setConfig(configData.config || configData);
      }
      if (statsRes.success && statsRes.data) {
        setStats(statsRes.data);
      }
      if (historyRes.success && historyRes.data) {
        setHistory(historyRes.data);
      }
      setLastRefresh(new Date());
    } catch (error) {
      console.error('Failed to fetch consensus data:', error);
    } finally {
      setLoading(false);
    }
  }, []);

  const fetchHistory = useCallback(async () => {
    try {
      const res = await arbFarmService.getConsensusHistory({ limit: 50 });
      if (res.success && res.data) {
        setHistory(res.data);
      }
    } catch (error) {
      console.error('Failed to fetch consensus history:', error);
    }
  }, []);

  const fetchRecommendations = useCallback(async () => {
    try {
      const res = await arbFarmService.getConsensusRecommendations();
      if (res.success && res.data) {
        setRecommendations(res.data);
      }
    } catch (error) {
      console.error('Failed to fetch recommendations:', error);
    }
  }, []);

  const fetchConversations = useCallback(async () => {
    try {
      const res = await arbFarmService.getConsensusConversations();
      if (res.success && res.data) {
        setConversations(res.data);
      }
    } catch (error) {
      console.error('Failed to fetch conversations:', error);
    }
  }, []);

  const fetchDiscoveredModels = useCallback(async () => {
    try {
      const res = await arbFarmService.getDiscoveredModels();
      if (res.success && res.data) {
        setDiscoveredModels(res.data);
      }
    } catch (error) {
      console.error('Failed to fetch discovered models:', error);
    }
  }, []);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  useEffect(() => {
    if (activeTab === 'history') fetchHistory();
    if (activeTab === 'recommendations') fetchRecommendations();
    if (activeTab === 'conversations') fetchConversations();
    if (activeTab === 'models') fetchDiscoveredModels();
  }, [activeTab, fetchHistory, fetchRecommendations, fetchConversations, fetchDiscoveredModels]);

  const extractConfig = (data: any): ConsensusConfig | null => {
    if (!data) return null;
    return data.config || data;
  };

  const handleRefreshAll = async () => {
    setRefreshing(true);
    await fetchData();
    setRefreshing(false);
  };

  const handleToggleEnabled = async () => {
    if (!config) return;
    try {
      const res = await arbFarmService.updateConsensusConfig({ enabled: !config.enabled });
      if (res.success && res.data) {
        setConfig(extractConfig(res.data));
      }
    } catch (error) {
      console.error('Failed to toggle consensus:', error);
    }
  };

  const handleUpdateThreshold = async (threshold: number) => {
    try {
      const res = await arbFarmService.updateConsensusConfig({ min_consensus_threshold: threshold });
      if (res.success && res.data) {
        setConfig(extractConfig(res.data));
      }
    } catch (error) {
      console.error('Failed to update threshold:', error);
    }
  };

  const handleToggleModel = async (modelId: string, enabled: boolean) => {
    if (!config) return;
    const updatedModels = config.models.map(m =>
      m.model_id === modelId ? { ...m, enabled } : m
    );
    try {
      const res = await arbFarmService.updateConsensusConfig({ models: updatedModels });
      if (res.success && res.data) {
        setConfig(extractConfig(res.data));
      }
    } catch (error) {
      console.error('Failed to toggle model:', error);
    }
  };

  const handleUpdateModelWeight = async (modelId: string, weight: number) => {
    if (!config) return;
    const updatedModels = config.models.map(m =>
      m.model_id === modelId ? { ...m, weight } : m
    );
    try {
      const res = await arbFarmService.updateConsensusConfig({ models: updatedModels });
      if (res.success && res.data) {
        setConfig(extractConfig(res.data));
      }
    } catch (error) {
      console.error('Failed to update model weight:', error);
    }
  };

  const handleAddModel = async (model: DiscoveredModel) => {
    if (!config) return;
    const newModel: ConsensusModelConfig = {
      model_id: model.id,
      display_name: model.name,
      provider: model.provider || 'openrouter',
      weight: 1.0,
      enabled: true,
      max_tokens: model.context_length || 4096,
    };
    const updatedModels = [...config.models, newModel];
    try {
      const res = await arbFarmService.updateConsensusConfig({ models: updatedModels });
      if (res.success && res.data) {
        setConfig(extractConfig(res.data));
      }
    } catch (error) {
      console.error('Failed to add model:', error);
    }
  };

  const handleRemoveModel = async (modelId: string) => {
    if (!config) return;
    const updatedModels = config.models.filter(m => m.model_id !== modelId);
    try {
      const res = await arbFarmService.updateConsensusConfig({ models: updatedModels });
      if (res.success && res.data) {
        setConfig(extractConfig(res.data));
      }
    } catch (error) {
      console.error('Failed to remove model:', error);
    }
  };

  const handleRefreshModels = async () => {
    setRefreshing(true);
    try {
      await arbFarmService.refreshDiscoveredModels();
      await fetchDiscoveredModels();
    } catch (error) {
      console.error('Failed to refresh models:', error);
    } finally {
      setRefreshing(false);
    }
  };

  const handleResetConfig = async () => {
    if (!confirm('Reset consensus configuration to defaults?')) return;
    try {
      const res = await arbFarmService.resetConsensusConfig();
      if (res.success && res.data) {
        setConfig(extractConfig(res.data));
      }
    } catch (error) {
      console.error('Failed to reset config:', error);
    }
  };

  const handleUpdateRecommendationStatus = async (id: string, status: 'acknowledged' | 'applied' | 'rejected') => {
    try {
      await arbFarmService.updateRecommendationStatus(id, status);
      await fetchRecommendations();
    } catch (error) {
      console.error('Failed to update recommendation status:', error);
    }
  };

  const formatTimestamp = (ts: string): string => {
    return new Date(ts).toLocaleString();
  };

  const formatTimeAgo = (ts: string): string => {
    const diff = Date.now() - new Date(ts).getTime();
    if (diff < 60000) return 'Just now';
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
    return `${Math.floor(diff / 86400000)}d ago`;
  };

  const renderSidebar = () => {
    const sections = NAV_ITEMS.reduce((acc, item) => {
      const section = item.section || 'Other';
      if (!acc[section]) acc[section] = [];
      acc[section].push(item);
      return acc;
    }, {} as Record<string, typeof NAV_ITEMS>);

    return (
      <aside className={styles.consensusSidebar}>
        {Object.entries(sections).map(([sectionName, items]) => (
          <div key={sectionName} className={styles.sidebarSection}>
            <div className={styles.sidebarTitle}>{sectionName}</div>
            {items.map((item) => (
              <button
                key={item.id}
                className={`${styles.navButton} ${activeTab === item.id ? styles.active : ''}`}
                onClick={() => setActiveTab(item.id)}
              >
                <span className={styles.navIcon}>{item.icon}</span>
                <span className={styles.navLabel}>{item.label}</span>
              </button>
            ))}
          </div>
        ))}

        <div className={styles.sidebarSection}>
          <div className={styles.sidebarTitle}>Status</div>
          <div className={styles.sidebarStatus}>
            <div className={styles.statusRow}>
              <span className={`${styles.statusDot} ${config?.enabled ? styles.active : styles.inactive}`} />
              <span>{config?.enabled ? 'Engine Active' : 'Engine Disabled'}</span>
            </div>
            <span className={styles.lastRefreshTime}>
              Last refresh: {lastRefresh.toLocaleTimeString()}
            </span>
            <button
              className={`${styles.refreshButton} ${refreshing ? styles.spinning : ''}`}
              onClick={handleRefreshAll}
              disabled={refreshing}
              title="Refresh all data"
            >
              ↻ Refresh
            </button>
          </div>
        </div>
      </aside>
    );
  };

  const renderMobileNav = () => (
    <div className={styles.mobileNav}>
      {NAV_ITEMS.map((item) => (
        <button
          key={item.id}
          className={`${styles.mobileButton} ${activeTab === item.id ? styles.active : ''}`}
          onClick={() => setActiveTab(item.id)}
        >
          {item.icon} {item.label}
        </button>
      ))}
    </div>
  );

  const renderDashboard = () => (
    <div className={styles.dashboardGrid}>
      <div className={styles.statsCard}>
        <h3 className={styles.cardTitle}>Consensus Engine</h3>
        <div className={styles.engineStatus}>
          <span className={`${styles.statusDot} ${config?.enabled ? styles.active : styles.inactive}`} />
          <span>{config?.enabled ? 'Active' : 'Disabled'}</span>
          <button
            className={`${styles.toggleButton} ${config?.enabled ? styles.enabled : ''}`}
            onClick={handleToggleEnabled}
          >
            {config?.enabled ? 'Disable' : 'Enable'}
          </button>
        </div>
        <div className={styles.statItems}>
          <div className={styles.statItem}>
            <span className={styles.statLabel}>Models Active</span>
            <span className={styles.statValue}>{config?.models.filter(m => m.enabled).length || 0}</span>
          </div>
          <div className={styles.statItem}>
            <span className={styles.statLabel}>Threshold</span>
            <span className={styles.statValue}>{((config?.min_consensus_threshold || 0.6) * 100).toFixed(0)}%</span>
          </div>
        </div>
      </div>

      <div className={styles.statsCard}>
        <h3 className={styles.cardTitle}>Decision Stats</h3>
        <div className={styles.statItems}>
          <div className={styles.statItem}>
            <span className={styles.statLabel}>Total Decisions</span>
            <span className={styles.statValue}>{stats?.total_decisions || 0}</span>
          </div>
          <div className={styles.statItem}>
            <span className={styles.statLabel}>Approved</span>
            <span className={`${styles.statValue} ${styles.success}`}>{stats?.approved_count || 0}</span>
          </div>
          <div className={styles.statItem}>
            <span className={styles.statLabel}>Rejected</span>
            <span className={`${styles.statValue} ${styles.danger}`}>{stats?.rejected_count || 0}</span>
          </div>
          <div className={styles.statItem}>
            <span className={styles.statLabel}>Last 24h</span>
            <span className={styles.statValue}>{stats?.decisions_last_24h || 0}</span>
          </div>
        </div>
      </div>

      <div className={styles.statsCard}>
        <h3 className={styles.cardTitle}>Performance</h3>
        <div className={styles.statItems}>
          <div className={styles.statItem}>
            <span className={styles.statLabel}>Avg Agreement</span>
            <span className={styles.statValue}>{((stats?.average_agreement || 0) * 100).toFixed(1)}%</span>
          </div>
          <div className={styles.statItem}>
            <span className={styles.statLabel}>Avg Confidence</span>
            <span className={styles.statValue}>{((stats?.average_confidence || 0) * 100).toFixed(1)}%</span>
          </div>
          <div className={styles.statItem}>
            <span className={styles.statLabel}>Avg Latency</span>
            <span className={styles.statValue}>{((stats?.average_latency_ms || 0) / 1000).toFixed(1)}s</span>
          </div>
        </div>
      </div>

      <div className={`${styles.statsCard} ${styles.wideCard}`}>
        <h3 className={styles.cardTitle}>Active Models</h3>
        <div className={styles.modelChips}>
          {config?.models.filter(m => m.enabled).map(model => (
            <div key={model.model_id} className={styles.modelChip}>
              <span className={styles.modelChipName}>{model.display_name}</span>
              <span className={styles.modelChipWeight}>w: {model.weight.toFixed(1)}</span>
            </div>
          ))}
          {(!config?.models || config.models.filter(m => m.enabled).length === 0) && (
            <span className={styles.noModels}>No models enabled</span>
          )}
        </div>
      </div>

      <div className={`${styles.statsCard} ${styles.wideCard}`}>
        <h3 className={styles.cardTitle}>Recent Decisions</h3>
        <div className={styles.recentList}>
          {history.slice(0, 5).map(entry => (
            <div
              key={entry.id}
              className={`${styles.recentItem} ${entry.result.approved ? styles.approved : styles.rejected}`}
              onClick={() => setSelectedHistoryItem(entry)}
            >
              <span className={styles.recentIcon}>{entry.result.approved ? '✓' : '✗'}</span>
              <span className={styles.recentEdge}>Edge {entry.edge_id.slice(0, 8)}</span>
              <span className={styles.recentAgreement}>{(entry.result.agreement_score * 100).toFixed(0)}%</span>
              <span className={styles.recentTime}>{formatTimeAgo(entry.created_at)}</span>
            </div>
          ))}
          {history.length === 0 && (
            <div className={styles.emptyState}>No decisions yet</div>
          )}
        </div>
      </div>
    </div>
  );

  const renderModels = () => (
    <div className={styles.modelsView}>
      <div className={styles.sectionHeader}>
        <h3>Configured Models</h3>
        <button className={styles.refreshButton} onClick={handleRefreshModels} disabled={refreshing}>
          {refreshing ? '⟳' : '↻'} Refresh
        </button>
      </div>

      <div className={styles.modelsList}>
        {config?.models.map(model => (
          <div key={model.model_id} className={`${styles.modelCard} ${model.enabled ? '' : styles.disabled}`}>
            <div className={styles.modelHeader}>
              <div className={styles.modelInfo}>
                <span className={styles.modelName}>{model.display_name}</span>
                <span className={styles.modelId}>{model.model_id}</span>
              </div>
              <div className={styles.modelControls}>
                <label className={styles.toggleSwitch}>
                  <input
                    type="checkbox"
                    checked={model.enabled}
                    onChange={(e) => handleToggleModel(model.model_id, e.target.checked)}
                  />
                  <span className={styles.slider} />
                </label>
              </div>
            </div>
            <div className={styles.modelDetails}>
              <div className={styles.modelDetail}>
                <span className={styles.detailLabel}>Provider</span>
                <span className={styles.detailValue}>{model.provider}</span>
              </div>
              <div className={styles.modelDetail}>
                <span className={styles.detailLabel}>Weight</span>
                <input
                  type="range"
                  min="0.1"
                  max="2.0"
                  step="0.1"
                  value={model.weight}
                  onChange={(e) => handleUpdateModelWeight(model.model_id, parseFloat(e.target.value))}
                  className={styles.weightSlider}
                />
                <span className={styles.weightValue}>{model.weight.toFixed(1)}</span>
              </div>
              <div className={styles.modelDetail}>
                <span className={styles.detailLabel}>Max Tokens</span>
                <span className={styles.detailValue}>{model.max_tokens.toLocaleString()}</span>
              </div>
            </div>
            <button
              className={styles.removeModelButton}
              onClick={() => handleRemoveModel(model.model_id)}
            >
              Remove
            </button>
          </div>
        ))}
      </div>

      <div className={styles.sectionHeader}>
        <h3>Available Models</h3>
        <span className={styles.modelCount}>{discoveredModels.length} discovered</span>
      </div>

      <div className={styles.availableModels}>
        {discoveredModels
          .filter(dm => !config?.models.some(m => m.model_id === dm.id))
          .slice(0, 12)
          .map(model => (
            <div key={model.id} className={styles.availableModelCard}>
              <div className={styles.availableModelInfo}>
                <span className={styles.availableModelName}>{model.name}</span>
                <span className={styles.availableModelProvider}>{model.provider}</span>
              </div>
              <button
                className={styles.addModelButton}
                onClick={() => handleAddModel(model)}
              >
                + Add
              </button>
            </div>
          ))}
      </div>
    </div>
  );

  const renderHistory = () => (
    <div className={styles.historyView}>
      {selectedHistoryItem ? (
        <div className={styles.historyDetail}>
          <button className={styles.backButton} onClick={() => setSelectedHistoryItem(null)}>
            ← Back to History
          </button>
          <div className={styles.detailCard}>
            <div className={styles.detailHeader}>
              <span className={`${styles.resultBadge} ${selectedHistoryItem.result.approved ? styles.approved : styles.rejected}`}>
                {selectedHistoryItem.result.approved ? 'APPROVED' : 'REJECTED'}
              </span>
              <span className={styles.detailTime}>{formatTimestamp(selectedHistoryItem.created_at)}</span>
            </div>
            <div className={styles.detailSection}>
              <h4>Edge Context</h4>
              <pre className={styles.contextPre}>{selectedHistoryItem.edge_context}</pre>
            </div>
            <div className={styles.detailSection}>
              <h4>Consensus Result</h4>
              <div className={styles.resultStats}>
                <div className={styles.resultStat}>
                  <span className={styles.resultLabel}>Agreement</span>
                  <span className={styles.resultValue}>{(selectedHistoryItem.result.agreement_score * 100).toFixed(1)}%</span>
                </div>
                <div className={styles.resultStat}>
                  <span className={styles.resultLabel}>Confidence</span>
                  <span className={styles.resultValue}>{(selectedHistoryItem.result.weighted_confidence * 100).toFixed(1)}%</span>
                </div>
                <div className={styles.resultStat}>
                  <span className={styles.resultLabel}>Latency</span>
                  <span className={styles.resultValue}>{(selectedHistoryItem.result.total_latency_ms / 1000).toFixed(2)}s</span>
                </div>
              </div>
            </div>
            <div className={styles.detailSection}>
              <h4>Model Votes</h4>
              <div className={styles.votesList}>
                {selectedHistoryItem.result.model_votes.map((vote, idx) => (
                  <div key={idx} className={`${styles.voteCard} ${vote.approved ? styles.voteApproved : styles.voteRejected}`}>
                    <div className={styles.voteHeader}>
                      <span className={styles.voteModel}>{vote.model}</span>
                      <span className={`${styles.voteBadge} ${vote.approved ? styles.approved : styles.rejected}`}>
                        {vote.approved ? '✓' : '✗'}
                      </span>
                    </div>
                    <div className={styles.voteStats}>
                      <span>Confidence: {(vote.confidence * 100).toFixed(0)}%</span>
                      <span>Latency: {(vote.latency_ms / 1000).toFixed(2)}s</span>
                    </div>
                    <p className={styles.voteReasoning}>{vote.reasoning}</p>
                  </div>
                ))}
              </div>
            </div>
            <div className={styles.detailSection}>
              <h4>Summary</h4>
              <p className={styles.reasoningSummary}>{selectedHistoryItem.result.reasoning_summary}</p>
            </div>
          </div>
        </div>
      ) : (
        <>
          <div className={styles.sectionHeader}>
            <h3>Consensus History</h3>
            <span className={styles.historyCount}>{history.length} decisions</span>
          </div>
          <div className={styles.historyList}>
            {history.map(entry => (
              <div
                key={entry.id}
                className={`${styles.historyItem} ${entry.result.approved ? styles.approved : styles.rejected}`}
                onClick={() => setSelectedHistoryItem(entry)}
              >
                <div className={styles.historyItemLeft}>
                  <span className={`${styles.historyIcon} ${entry.result.approved ? styles.iconApproved : styles.iconRejected}`}>
                    {entry.result.approved ? '✓' : '✗'}
                  </span>
                  <div className={styles.historyItemInfo}>
                    <span className={styles.historyEdgeId}>Edge: {entry.edge_id.slice(0, 12)}...</span>
                    <span className={styles.historyTime}>{formatTimestamp(entry.created_at)}</span>
                  </div>
                </div>
                <div className={styles.historyItemRight}>
                  <div className={styles.historyMetric}>
                    <span className={styles.metricLabel}>Agreement</span>
                    <span className={styles.metricValue}>{(entry.result.agreement_score * 100).toFixed(0)}%</span>
                  </div>
                  <div className={styles.historyMetric}>
                    <span className={styles.metricLabel}>Confidence</span>
                    <span className={styles.metricValue}>{(entry.result.weighted_confidence * 100).toFixed(0)}%</span>
                  </div>
                  <div className={styles.historyMetric}>
                    <span className={styles.metricLabel}>Models</span>
                    <span className={styles.metricValue}>{entry.result.model_votes.length}</span>
                  </div>
                </div>
                <span className={styles.historyArrow}>→</span>
              </div>
            ))}
            {history.length === 0 && (
              <div className={styles.emptyState}>
                <span className={styles.emptyIcon}>📊</span>
                <span>No consensus decisions yet</span>
                <span className={styles.emptyHint}>Decisions will appear here when trades are evaluated</span>
              </div>
            )}
          </div>
        </>
      )}
    </div>
  );

  const renderRecommendations = () => (
    <div className={styles.recommendationsView}>
      <div className={styles.sectionHeader}>
        <h3>AI Recommendations</h3>
        <div className={styles.filterTabs}>
          <button className={styles.filterTab}>All</button>
          <button className={styles.filterTab}>Pending</button>
          <button className={styles.filterTab}>Applied</button>
        </div>
      </div>

      <div className={styles.recommendationsList}>
        {recommendations.map(rec => (
          <div key={rec.recommendation_id} className={`${styles.recommendationCard} ${styles[rec.category]}`}>
            <div className={styles.recHeader}>
              <span className={styles.recCategory}>{rec.category}</span>
              <span className={`${styles.recStatus} ${styles[rec.status]}`}>{rec.status}</span>
            </div>
            <h4 className={styles.recTitle}>{rec.title}</h4>
            <p className={styles.recDescription}>{rec.description}</p>
            <div className={styles.recMeta}>
              <span className={styles.recConfidence}>
                Confidence: {(rec.confidence * 100).toFixed(0)}%
              </span>
              <span className={styles.recSource}>{rec.source.replace(/_/g, ' ')}</span>
              <span className={styles.recTime}>{formatTimeAgo(rec.created_at)}</span>
            </div>
            {rec.status === 'pending' && (
              <div className={styles.recActions}>
                <button
                  className={styles.applyButton}
                  onClick={() => handleUpdateRecommendationStatus(rec.recommendation_id, 'applied')}
                >
                  Apply
                </button>
                <button
                  className={styles.acknowledgeButton}
                  onClick={() => handleUpdateRecommendationStatus(rec.recommendation_id, 'acknowledged')}
                >
                  Acknowledge
                </button>
                <button
                  className={styles.rejectButton}
                  onClick={() => handleUpdateRecommendationStatus(rec.recommendation_id, 'rejected')}
                >
                  Reject
                </button>
              </div>
            )}
          </div>
        ))}
        {recommendations.length === 0 && (
          <div className={styles.emptyState}>
            <span className={styles.emptyIcon}>💡</span>
            <span>No recommendations yet</span>
            <span className={styles.emptyHint}>AI-generated insights will appear here</span>
          </div>
        )}
      </div>
    </div>
  );

  const renderConversations = () => (
    <div className={styles.conversationsView}>
      {selectedConversation ? (
        <div className={styles.conversationDetail}>
          <button className={styles.backButton} onClick={() => setSelectedConversation(null)}>
            ← Back to Conversations
          </button>
          <div className={styles.conversationCard}>
            <div className={styles.convHeader}>
              <span className={styles.convTopic}>{selectedConversation.topic.replace(/_/g, ' ')}</span>
              <span className={styles.convTime}>{formatTimestamp(selectedConversation.created_at)}</span>
            </div>
            <div className={styles.convParticipants}>
              <span className={styles.participantsLabel}>Participants:</span>
              {selectedConversation.participants.map((p, i) => (
                <span key={i} className={styles.participantChip}>{p}</span>
              ))}
            </div>
            <div className={styles.convMessages}>
              {selectedConversation.messages.map((msg, idx) => (
                <div key={idx} className={styles.messageItem}>
                  <div className={styles.messageHeader}>
                    <span className={styles.messageRole}>{msg.role}</span>
                    {msg.model && <span className={styles.messageModel}>{msg.model}</span>}
                  </div>
                  <p className={styles.messageContent}>{msg.content}</p>
                </div>
              ))}
            </div>
            {selectedConversation.outcome && (
              <div className={styles.convOutcome}>
                <h4>Outcome</h4>
                <p>{selectedConversation.outcome.summary}</p>
              </div>
            )}
          </div>
        </div>
      ) : (
        <>
          <div className={styles.sectionHeader}>
            <h3>LLM Conversations</h3>
            <span className={styles.convCount}>{conversations.length} conversations</span>
          </div>
          <div className={styles.conversationsList}>
            {conversations.map(conv => (
              <div
                key={conv.session_id}
                className={styles.convListItem}
                onClick={() => setSelectedConversation(conv)}
              >
                <div className={styles.convListLeft}>
                  <span className={styles.convListTopic}>{conv.topic.replace(/_/g, ' ')}</span>
                  <span className={styles.convListParticipants}>
                    {conv.participants.slice(0, 3).join(', ')}
                    {conv.participants.length > 3 && ` +${conv.participants.length - 3}`}
                  </span>
                </div>
                <div className={styles.convListRight}>
                  <span className={styles.convListMessages}>{conv.messages.length} messages</span>
                  <span className={styles.convListTime}>{formatTimeAgo(conv.created_at)}</span>
                </div>
                <span className={styles.convListArrow}>→</span>
              </div>
            ))}
            {conversations.length === 0 && (
              <div className={styles.emptyState}>
                <span className={styles.emptyIcon}>💬</span>
                <span>No conversations yet</span>
                <span className={styles.emptyHint}>Multi-model discussions will appear here</span>
              </div>
            )}
          </div>
        </>
      )}
    </div>
  );

  const renderConfig = () => (
    <div className={styles.configView}>
      <div className={styles.configSection}>
        <h3>Engine Settings</h3>
        <div className={styles.configCard}>
          <div className={styles.configItem}>
            <div className={styles.configItemHeader}>
              <span className={styles.configLabel}>Consensus Enabled</span>
              <label className={styles.toggleSwitch}>
                <input
                  type="checkbox"
                  checked={config?.enabled || false}
                  onChange={handleToggleEnabled}
                />
                <span className={styles.slider} />
              </label>
            </div>
            <span className={styles.configHint}>Enable or disable the multi-LLM consensus engine</span>
          </div>

          <div className={styles.configItem}>
            <div className={styles.configItemHeader}>
              <span className={styles.configLabel}>Minimum Threshold</span>
              <span className={styles.configValue}>{((config?.min_consensus_threshold || 0.6) * 100).toFixed(0)}%</span>
            </div>
            <input
              type="range"
              min="0.3"
              max="1.0"
              step="0.05"
              value={config?.min_consensus_threshold || 0.6}
              onChange={(e) => handleUpdateThreshold(parseFloat(e.target.value))}
              className={styles.configSlider}
            />
            <span className={styles.configHint}>Minimum agreement score required for approval</span>
          </div>

          <div className={styles.configItem}>
            <div className={styles.configItemHeader}>
              <span className={styles.configLabel}>Auto-Apply Recommendations</span>
              <label className={styles.toggleSwitch}>
                <input
                  type="checkbox"
                  checked={config?.auto_apply_recommendations || false}
                  onChange={async () => {
                    if (!config) return;
                    const res = await arbFarmService.updateConsensusConfig({
                      auto_apply_recommendations: !config.auto_apply_recommendations
                    });
                    if (res.success && res.data) setConfig(extractConfig(res.data));
                  }}
                />
                <span className={styles.slider} />
              </label>
            </div>
            <span className={styles.configHint}>Automatically apply high-confidence recommendations</span>
          </div>

          <div className={styles.configItem}>
            <div className={styles.configItemHeader}>
              <span className={styles.configLabel}>Timeout (ms)</span>
              <span className={styles.configValue}>{config?.timeout_ms || 30000}</span>
            </div>
            <span className={styles.configHint}>Maximum time to wait for model responses</span>
          </div>

          <div className={styles.configItem}>
            <div className={styles.configItemHeader}>
              <span className={styles.configLabel}>Max Tokens</span>
              <span className={styles.configValue}>{config?.max_tokens_per_request || 2048}</span>
            </div>
            <span className={styles.configHint}>Maximum tokens per consensus request</span>
          </div>
        </div>

        <button className={styles.resetButton} onClick={handleResetConfig}>
          Reset to Defaults
        </button>
      </div>
    </div>
  );

  const renderApi = () => (
    <div className={styles.apiView}>
      <div className={styles.apiSection}>
        <h3>REST API Endpoints</h3>
        <div className={styles.endpointList}>
          {[
            { method: 'POST', path: '/api/arb/consensus/request', desc: 'Request multi-LLM consensus' },
            { method: 'GET', path: '/api/arb/consensus/:id', desc: 'Get consensus decision details' },
            { method: 'GET', path: '/api/arb/consensus', desc: 'List consensus history' },
            { method: 'GET', path: '/api/arb/consensus/stats', desc: 'Get consensus statistics' },
            { method: 'GET', path: '/api/arb/consensus/config', desc: 'Get configuration' },
            { method: 'PUT', path: '/api/arb/consensus/config', desc: 'Update configuration' },
            { method: 'POST', path: '/api/arb/consensus/config/reset', desc: 'Reset to defaults' },
            { method: 'GET', path: '/api/arb/consensus/models', desc: 'List available models' },
            { method: 'GET', path: '/api/arb/consensus/recommendations', desc: 'List recommendations' },
            { method: 'GET', path: '/api/arb/consensus/conversations', desc: 'List conversations' },
          ].map((endpoint, idx) => (
            <div key={idx} className={styles.endpointItem}>
              <span className={`${styles.endpointMethod} ${styles[endpoint.method.toLowerCase()]}`}>
                {endpoint.method}
              </span>
              <code className={styles.endpointPath}>{endpoint.path}</code>
              <span className={styles.endpointDesc}>{endpoint.desc}</span>
            </div>
          ))}
        </div>
      </div>

      <div className={styles.apiSection}>
        <h3>MCP Tools</h3>
        <div className={styles.toolsList}>
          {[
            { name: 'consensus_request', type: 'Write', desc: 'Request multi-LLM consensus on a trade decision' },
            { name: 'consensus_result', type: 'Write', desc: 'Get result of a previous consensus request' },
            { name: 'consensus_history', type: 'Read', desc: 'Get history of consensus decisions' },
            { name: 'consensus_stats', type: 'Read', desc: 'Get consensus engine statistics' },
            { name: 'consensus_config_get', type: 'Read', desc: 'Get current configuration' },
            { name: 'consensus_config_update', type: 'Write', desc: 'Update configuration' },
            { name: 'consensus_models_list', type: 'Read', desc: 'List available LLM models' },
            { name: 'consensus_models_discovered', type: 'Write', desc: 'Discover models from OpenRouter' },
            { name: 'consensus_learning_summary', type: 'Read', desc: 'Get learning summary' },
          ].map((tool, idx) => (
            <div key={idx} className={styles.toolItem}>
              <code className={styles.toolName}>{tool.name}</code>
              <span className={`${styles.toolType} ${styles[tool.type.toLowerCase()]}`}>{tool.type}</span>
              <span className={styles.toolDesc}>{tool.desc}</span>
            </div>
          ))}
        </div>
      </div>

      <div className={styles.apiSection}>
        <h3>MCP Server</h3>
        <div className={styles.mcpInfo}>
          <p>Connect via JSON-RPC at:</p>
          <code className={styles.mcpEndpoint}>http://localhost:9007/mcp/jsonrpc</code>
          <p className={styles.mcpHint}>Protocol: MCP 2025-11-25</p>
        </div>
      </div>
    </div>
  );

  const renderContent = () => {
    switch (activeTab) {
      case 'dashboard':
        return renderDashboard();
      case 'models':
        return renderModels();
      case 'history':
        return renderHistory();
      case 'recommendations':
        return renderRecommendations();
      case 'conversations':
        return renderConversations();
      case 'config':
        return renderConfig();
      case 'api':
        return renderApi();
      default:
        return renderDashboard();
    }
  };

  if (loading) {
    return (
      <div className={styles.consensusLayout}>
        <div className={styles.loadingState}>
          <div className={styles.loadingSpinner} />
          <span>Loading consensus data...</span>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.consensusLayout}>
      {renderSidebar()}
      {renderMobileNav()}
      <div className={styles.consensusContent}>
        {renderContent()}
      </div>
    </div>
  );
};

export default ConsensusView;
