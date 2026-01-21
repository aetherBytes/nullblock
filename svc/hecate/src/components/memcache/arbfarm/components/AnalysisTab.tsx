import React, { useEffect, useState, useCallback } from 'react';
import styles from '../arbfarm.module.scss';
import { arbFarmService } from '../../../../common/services/arbfarm-service';
import type {
  ConsensusConfig,
  ConsensusModelConfig,
  TradeAnalysis,
  PatternSummary,
  ConversationLog,
} from '../../../../types/consensus';

const EXIT_REASON_COLORS: Record<string, string> = {
  StopLoss: '#EF4444',
  TakeProfit: '#22C55E',
  MaxHoldTime: '#F59E0B',
  Manual: '#6B7280',
  Unknown: '#9CA3AF',
};

const AnalysisTab: React.FC = () => {
  const [config, setConfig] = useState<ConsensusConfig | null>(null);
  const [availableModels, setAvailableModels] = useState<ConsensusModelConfig[]>([]);
  const [isDevWallet, setIsDevWallet] = useState(false);
  const [tradeAnalyses, setTradeAnalyses] = useState<TradeAnalysis[]>([]);
  const [patternSummary, setPatternSummary] = useState<PatternSummary | null>(null);
  const [conversations, setConversations] = useState<ConversationLog[]>([]);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [expandedTradeId, setExpandedTradeId] = useState<string | null>(null);
  const [showConfigSection, setShowConfigSection] = useState(false);
  const [showConversations, setShowConversations] = useState(false);
  const [showAddModel, setShowAddModel] = useState(false);
  const [selectedModelToAdd, setSelectedModelToAdd] = useState<string>('');
  const [newModelWeight, setNewModelWeight] = useState<number>(1.0);

  const fetchData = useCallback(async () => {
    try {
      const [configRes, analysesRes, patternRes, conversationsRes] = await Promise.all([
        arbFarmService.getConsensusConfig(),
        arbFarmService.getTradeAnalyses(50),
        arbFarmService.getPatternSummary(),
        arbFarmService.listConversations(10),
      ]);

      if (configRes.success && configRes.data) {
        setConfig(configRes.data.config);
        setAvailableModels(configRes.data.available_models);
        setIsDevWallet(configRes.data.is_dev_wallet);
      }

      if (analysesRes.success && analysesRes.data) {
        setTradeAnalyses(analysesRes.data.analyses);
      }

      if (patternRes.success && patternRes.data && patternRes.data.summary) {
        setPatternSummary(patternRes.data.summary);
      }

      if (conversationsRes.success && conversationsRes.data) {
        setConversations(conversationsRes.data.conversations);
      }
    } catch (error) {
      console.error('Failed to fetch analysis data:', error);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  const handleToggleEnabled = async () => {
    if (!config) return;
    setSaving(true);
    try {
      const res = await arbFarmService.updateConsensusConfig({
        enabled: !config.enabled,
      });
      if (res.success && res.data) {
        setConfig(res.data.config);
      }
    } catch (error) {
      console.error('Failed to toggle consensus:', error);
    } finally {
      setSaving(false);
    }
  };

  const handleToggleModel = async (modelId: string, enabled: boolean) => {
    if (!config) return;
    setSaving(true);
    try {
      const updatedModels = config.models.map(m =>
        m.model_id === modelId ? { ...m, enabled } : m
      );
      const res = await arbFarmService.updateConsensusConfig({
        models: updatedModels,
      });
      if (res.success && res.data) {
        setConfig(res.data.config);
      }
    } catch (error) {
      console.error('Failed to toggle model:', error);
    } finally {
      setSaving(false);
    }
  };

  const handleUpdateWeight = async (modelId: string, weight: number) => {
    if (!config) return;
    setSaving(true);
    try {
      const updatedModels = config.models.map(m =>
        m.model_id === modelId ? { ...m, weight } : m
      );
      const res = await arbFarmService.updateConsensusConfig({
        models: updatedModels,
      });
      if (res.success && res.data) {
        setConfig(res.data.config);
      }
    } catch (error) {
      console.error('Failed to update weight:', error);
    } finally {
      setSaving(false);
    }
  };

  const handleRemoveModel = async (modelId: string) => {
    if (!config) return;
    setSaving(true);
    try {
      const updatedModels = config.models.filter(m => m.model_id !== modelId);
      const res = await arbFarmService.updateConsensusConfig({
        models: updatedModels,
      });
      if (res.success && res.data) {
        setConfig(res.data.config);
      }
    } catch (error) {
      console.error('Failed to remove model:', error);
    } finally {
      setSaving(false);
    }
  };

  const handleAddModel = async () => {
    if (!config || !selectedModelToAdd) return;
    const modelToAdd = availableModels.find(m => m.model_id === selectedModelToAdd);
    if (!modelToAdd) return;

    setSaving(true);
    try {
      const updatedModels = [
        ...config.models,
        { ...modelToAdd, weight: newModelWeight, enabled: true },
      ];
      const res = await arbFarmService.updateConsensusConfig({
        models: updatedModels,
      });
      if (res.success && res.data) {
        setConfig(res.data.config);
        setShowAddModel(false);
        setSelectedModelToAdd('');
        setNewModelWeight(1.0);
      }
    } catch (error) {
      console.error('Failed to add model:', error);
    } finally {
      setSaving(false);
    }
  };

  const handleUpdateReviewInterval = async (hours: number) => {
    if (!config) return;
    setSaving(true);
    try {
      const res = await arbFarmService.updateConsensusConfig({
        review_interval_hours: hours,
      });
      if (res.success && res.data) {
        setConfig(res.data.config);
      }
    } catch (error) {
      console.error('Failed to update review interval:', error);
    } finally {
      setSaving(false);
    }
  };

  const formatTimestamp = (ts: string) => {
    const date = new Date(ts);
    return date.toLocaleString();
  };

  const getExitReasonColor = (reason: string) => {
    return EXIT_REASON_COLORS[reason] || EXIT_REASON_COLORS.Unknown;
  };

  const availableModelsToAdd = availableModels.filter(
    am => !config?.models.some(m => m.model_id === am.model_id)
  );

  if (loading) {
    return (
      <div className={styles.loadingContainer}>
        <div className={styles.spinner} />
        <span>Loading analysis data...</span>
      </div>
    );
  }

  return (
    <div className={styles.analysisTab}>
      {/* Config Section (Collapsible) */}
      <div className={styles.collapsibleSection}>
        <div
          className={styles.collapsibleHeader}
          onClick={() => setShowConfigSection(!showConfigSection)}
        >
          <div className={styles.headerLeft}>
            <span className={styles.expandIcon}>{showConfigSection ? '▼' : '▶'}</span>
            <h3>Analysis Configuration</h3>
            {isDevWallet && <span className={styles.devBadge}>Dev Wallet</span>}
          </div>
          <div className={styles.headerRight}>
            <span className={config?.enabled ? styles.statusEnabled : styles.statusDisabled}>
              {config?.enabled ? 'Enabled' : 'Disabled'}
            </span>
            <span className={styles.modelCount}>
              {config?.models.filter(m => m.enabled).length || 0} models active
            </span>
          </div>
        </div>

        {showConfigSection && (
          <div className={styles.collapsibleContent}>
            <div className={styles.configSection}>
              <div className={styles.configRow}>
                <label className={styles.checkboxLabel}>
                  <input
                    type="checkbox"
                    checked={config?.enabled ?? false}
                    onChange={handleToggleEnabled}
                    disabled={saving}
                  />
                  Enable Consensus Engine
                </label>

                <div className={styles.configField}>
                  <label>Review Interval</label>
                  <select
                    value={config?.review_interval_hours ?? 1}
                    onChange={e => handleUpdateReviewInterval(parseInt(e.target.value))}
                    disabled={saving}
                  >
                    <option value={1}>1 hour</option>
                    <option value={2}>2 hours</option>
                    <option value={4}>4 hours</option>
                    <option value={6}>6 hours</option>
                    <option value={12}>12 hours</option>
                    <option value={24}>24 hours</option>
                  </select>
                </div>
              </div>

              <div className={styles.modelsSection}>
                <div className={styles.sectionHeader}>
                  <h4>Active Models</h4>
                  <button
                    className={styles.addButton}
                    onClick={() => setShowAddModel(true)}
                    disabled={availableModelsToAdd.length === 0}
                  >
                    + Add Model
                  </button>
                </div>

                <div className={styles.modelsList}>
                  {config?.models.map(model => (
                    <div key={model.model_id} className={styles.modelItem}>
                      <div className={styles.modelInfo}>
                        <span className={styles.modelName}>{model.display_name}</span>
                        <span className={styles.modelProvider}>{model.provider}</span>
                      </div>
                      <div className={styles.modelControls}>
                        <label className={styles.weightLabel}>
                          Weight:
                          <input
                            type="number"
                            min={0.1}
                            max={3}
                            step={0.1}
                            value={model.weight}
                            onChange={e => handleUpdateWeight(model.model_id, parseFloat(e.target.value))}
                            disabled={saving}
                            className={styles.weightInput}
                          />
                        </label>
                        <label className={styles.checkboxLabel}>
                          <input
                            type="checkbox"
                            checked={model.enabled}
                            onChange={e => handleToggleModel(model.model_id, e.target.checked)}
                            disabled={saving}
                          />
                          Enabled
                        </label>
                        <button
                          className={styles.removeButton}
                          onClick={() => handleRemoveModel(model.model_id)}
                          disabled={saving || config.models.length <= 1}
                          title="Remove model"
                        >
                          &times;
                        </button>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Trade Analysis Table (Main Content) */}
      <div className={styles.tradeAnalysisSection}>
        <div className={styles.sectionHeader}>
          <h3>Trade Analysis</h3>
          <span className={styles.count}>{tradeAnalyses.length} analyzed trades</span>
        </div>

        {tradeAnalyses.length === 0 ? (
          <div className={styles.emptyState}>
            <p>No trade analyses yet</p>
            <span>Trade analyses will appear here after the consensus LLM reviews your trades.</span>
          </div>
        ) : (
          <div className={styles.tradeAnalysisTable}>
            <table>
              <thead>
                <tr>
                  <th>Token</th>
                  <th>Venue</th>
                  <th>PnL</th>
                  <th>Exit Reason</th>
                  <th>Root Cause</th>
                  <th>Suggested Fix</th>
                </tr>
              </thead>
              <tbody>
                {tradeAnalyses.map(analysis => (
                  <React.Fragment key={analysis.analysis_id}>
                    <tr
                      className={`${styles.tradeRow} ${analysis.pnl_sol < 0 ? styles.losingTrade : styles.winningTrade}`}
                      onClick={() => setExpandedTradeId(expandedTradeId === analysis.analysis_id ? null : analysis.analysis_id)}
                    >
                      <td className={styles.tokenCell}>
                        <span className={styles.tokenSymbol}>{analysis.token_symbol}</span>
                      </td>
                      <td>
                        <span className={styles.venueBadge}>{analysis.venue}</span>
                      </td>
                      <td className={analysis.pnl_sol < 0 ? styles.negative : styles.positive}>
                        {analysis.pnl_sol >= 0 ? '+' : ''}{analysis.pnl_sol.toFixed(4)} SOL
                      </td>
                      <td>
                        <span
                          className={styles.exitReasonBadge}
                          style={{ backgroundColor: getExitReasonColor(analysis.exit_reason) }}
                        >
                          {analysis.exit_reason}
                        </span>
                      </td>
                      <td className={styles.rootCauseCell}>
                        <span className={styles.truncatedText}>{analysis.root_cause}</span>
                      </td>
                      <td className={styles.suggestedFixCell}>
                        <span className={styles.truncatedText}>
                          {analysis.suggested_fix || '-'}
                        </span>
                      </td>
                    </tr>
                    {expandedTradeId === analysis.analysis_id && (
                      <tr className={styles.expandedRow}>
                        <td colSpan={6}>
                          <div className={styles.expandedContent}>
                            <div className={styles.expandedField}>
                              <strong>Root Cause:</strong>
                              <p>{analysis.root_cause}</p>
                            </div>
                            {analysis.config_issue && (
                              <div className={styles.expandedField}>
                                <strong>Config Issue:</strong>
                                <p>{analysis.config_issue}</p>
                              </div>
                            )}
                            {analysis.pattern && (
                              <div className={styles.expandedField}>
                                <strong>Pattern:</strong>
                                <p>{analysis.pattern}</p>
                              </div>
                            )}
                            {analysis.suggested_fix && (
                              <div className={styles.expandedField}>
                                <strong>Suggested Fix:</strong>
                                <p>{analysis.suggested_fix}</p>
                              </div>
                            )}
                            <div className={styles.expandedMeta}>
                              <span>Confidence: {(analysis.confidence * 100).toFixed(0)}%</span>
                              <span>Analyzed: {formatTimestamp(analysis.created_at)}</span>
                            </div>
                          </div>
                        </td>
                      </tr>
                    )}
                  </React.Fragment>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {/* Pattern Summary */}
      {patternSummary && (
        <div className={styles.patternSummarySection}>
          <div className={styles.sectionHeader}>
            <h3>Pattern Summary</h3>
            <span className={styles.meta}>
              {patternSummary.trades_analyzed} trades analyzed | {patternSummary.time_period}
            </span>
          </div>

          <div className={styles.patternGrid}>
            <div className={styles.patternColumn}>
              <h4 className={styles.losingHeader}>Losing Patterns</h4>
              {patternSummary.losing_patterns.length === 0 ? (
                <p className={styles.noData}>No patterns identified</p>
              ) : (
                <ul className={styles.patternList}>
                  {patternSummary.losing_patterns.map((pattern, idx) => (
                    <li key={idx} className={styles.losingPattern}>{pattern}</li>
                  ))}
                </ul>
              )}
            </div>

            <div className={styles.patternColumn}>
              <h4 className={styles.winningHeader}>Winning Patterns</h4>
              {patternSummary.winning_patterns.length === 0 ? (
                <p className={styles.noData}>No patterns identified</p>
              ) : (
                <ul className={styles.patternList}>
                  {patternSummary.winning_patterns.map((pattern, idx) => (
                    <li key={idx} className={styles.winningPattern}>{pattern}</li>
                  ))}
                </ul>
              )}
            </div>

            <div className={styles.patternColumn}>
              <h4 className={styles.recommendationHeader}>Config Recommendations</h4>
              {patternSummary.config_recommendations.length === 0 ? (
                <p className={styles.noData}>No recommendations</p>
              ) : (
                <ul className={styles.patternList}>
                  {patternSummary.config_recommendations.map((rec, idx) => (
                    <li key={idx} className={styles.recommendationItem}>{rec}</li>
                  ))}
                </ul>
              )}
            </div>
          </div>
        </div>
      )}

      {/* Conversation History (Collapsible) */}
      <div className={styles.collapsibleSection}>
        <div
          className={styles.collapsibleHeader}
          onClick={() => setShowConversations(!showConversations)}
        >
          <div className={styles.headerLeft}>
            <span className={styles.expandIcon}>{showConversations ? '▼' : '▶'}</span>
            <h3>Conversation History</h3>
          </div>
          <div className={styles.headerRight}>
            <span className={styles.count}>{conversations.length} conversations</span>
          </div>
        </div>

        {showConversations && (
          <div className={styles.collapsibleContent}>
            {conversations.length === 0 ? (
              <div className={styles.emptyState}>
                <p>No conversations yet</p>
              </div>
            ) : (
              <div className={styles.conversationsList}>
                {conversations.map(conv => (
                  <div key={conv.session_id} className={styles.conversationItem}>
                    <div className={styles.conversationHeader}>
                      <span className={styles.topicBadge}>
                        {conv.topic.replace('_', ' ')}
                      </span>
                      <span className={styles.timestamp}>
                        {formatTimestamp(conv.created_at)}
                      </span>
                      <span className={styles.participants}>
                        {conv.participants.length} participants
                      </span>
                      {conv.outcome.consensus_reached && (
                        <span className={styles.consensusBadge}>Consensus</span>
                      )}
                    </div>
                    {conv.outcome.summary && (
                      <p className={styles.conversationSummary}>{conv.outcome.summary}</p>
                    )}
                  </div>
                ))}
              </div>
            )}
          </div>
        )}
      </div>

      {/* Add Model Modal */}
      {showAddModel && (
        <div className={styles.addModelModal}>
          <div className={styles.modalContent}>
            <h4>Add Model</h4>
            <div className={styles.modalField}>
              <label>Select Model</label>
              <select
                value={selectedModelToAdd}
                onChange={e => setSelectedModelToAdd(e.target.value)}
              >
                <option value="">-- Select --</option>
                {availableModelsToAdd.map(m => (
                  <option key={m.model_id} value={m.model_id}>
                    {m.display_name} ({m.provider})
                  </option>
                ))}
              </select>
            </div>
            <div className={styles.modalField}>
              <label>Weight</label>
              <input
                type="number"
                min={0.1}
                max={3}
                step={0.1}
                value={newModelWeight}
                onChange={e => setNewModelWeight(parseFloat(e.target.value))}
              />
            </div>
            <div className={styles.modalActions}>
              <button onClick={() => setShowAddModel(false)}>Cancel</button>
              <button
                onClick={handleAddModel}
                disabled={!selectedModelToAdd || saving}
                className={styles.primaryButton}
              >
                Add
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default AnalysisTab;
