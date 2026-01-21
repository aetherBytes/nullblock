import React, { useEffect, useState, useCallback } from 'react';
import styles from '../arbfarm.module.scss';
import { arbFarmService } from '../../../../common/services/arbfarm-service';
import type {
  Recommendation,
  RecommendationStatus,
  RecommendationCategory,
  LearningSummary,
} from '../../../../types/consensus';

const formatValue = (value: unknown): string => {
  if (value === null || value === undefined) return 'null';
  if (typeof value === 'object') {
    return JSON.stringify(value, null, 2);
  }
  return String(value);
};

const STATUS_LABELS: Record<RecommendationStatus, string> = {
  pending: 'Pending',
  acknowledged: 'Acknowledged',
  applied: 'Applied',
  rejected: 'Rejected',
};

const STATUS_COLORS: Record<RecommendationStatus, string> = {
  pending: '#F59E0B',
  acknowledged: '#3B82F6',
  applied: '#22C55E',
  rejected: '#EF4444',
};

const CATEGORY_LABELS: Record<RecommendationCategory, string> = {
  strategy: 'Strategy',
  risk: 'Risk',
  timing: 'Timing',
  venue: 'Venue',
  position: 'Position',
};

const CATEGORY_ICONS: Record<RecommendationCategory, string> = {
  strategy: '🎯',
  risk: '⚠️',
  timing: '⏱️',
  venue: '🏛️',
  position: '📊',
};

const RecommendationsTab: React.FC = () => {
  const [recommendations, setRecommendations] = useState<Recommendation[]>([]);
  const [summary, setSummary] = useState<LearningSummary | null>(null);
  const [loading, setLoading] = useState(true);
  const [statusFilter, setStatusFilter] = useState<string>('');
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [updating, setUpdating] = useState<string | null>(null);

  const fetchData = useCallback(async () => {
    try {
      const [recRes, summaryRes] = await Promise.all([
        arbFarmService.listRecommendations(statusFilter || undefined, 50),
        arbFarmService.getLearningSummary(),
      ]);

      if (recRes.success && recRes.data) {
        setRecommendations(recRes.data.recommendations);
      }
      if (summaryRes.success && summaryRes.data) {
        setSummary(summaryRes.data);
      }
    } catch (error) {
      console.error('Failed to fetch recommendations:', error);
    } finally {
      setLoading(false);
    }
  }, [statusFilter]);

  useEffect(() => {
    setLoading(true);
    fetchData();
  }, [fetchData]);

  const handleUpdateStatus = async (id: string, newStatus: RecommendationStatus) => {
    setUpdating(id);
    try {
      const res = await arbFarmService.updateRecommendationStatus(id, newStatus);
      if (res.success) {
        fetchData();
      }
    } catch (error) {
      console.error('Failed to update recommendation status:', error);
    } finally {
      setUpdating(null);
    }
  };

  const formatTimestamp = (ts: string) => {
    const date = new Date(ts);
    return date.toLocaleString();
  };

  const getConfidenceLevel = (confidence: number): 'high' | 'medium' | 'low' => {
    if (confidence >= 0.8) return 'high';
    if (confidence >= 0.6) return 'medium';
    return 'low';
  };

  if (loading) {
    return (
      <div className={styles.loadingContainer}>
        <div className={styles.spinner} />
        <span>Loading recommendations...</span>
      </div>
    );
  }

  return (
    <div className={styles.recommendationsTabV2}>
      {/* Summary Section */}
      <div className={styles.recSummarySection}>
        <div className={styles.recSummaryHeader}>
          <h3>Learning Recommendations</h3>
          <span className={styles.recSubtitle}>
            AI-powered suggestions from consensus analysis
          </span>
        </div>

        {summary && (
          <div className={styles.recStatsGrid}>
            <div className={styles.recStatCard}>
              <div className={styles.recStatIcon}>📋</div>
              <div className={styles.recStatContent}>
                <span className={styles.recStatValue}>{summary.total_recommendations}</span>
                <span className={styles.recStatLabel}>Total</span>
              </div>
            </div>
            <div className={`${styles.recStatCard} ${styles.pending}`}>
              <div className={styles.recStatIcon}>⏳</div>
              <div className={styles.recStatContent}>
                <span className={styles.recStatValue}>{summary.pending_recommendations}</span>
                <span className={styles.recStatLabel}>Pending Review</span>
              </div>
            </div>
            <div className={`${styles.recStatCard} ${styles.applied}`}>
              <div className={styles.recStatIcon}>✅</div>
              <div className={styles.recStatContent}>
                <span className={styles.recStatValue}>{summary.applied_recommendations}</span>
                <span className={styles.recStatLabel}>Applied</span>
              </div>
            </div>
            <div className={styles.recStatCard}>
              <div className={styles.recStatIcon}>💬</div>
              <div className={styles.recStatContent}>
                <span className={styles.recStatValue}>{summary.total_conversations}</span>
                <span className={styles.recStatLabel}>Conversations</span>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Filter Section */}
      <div className={styles.recFilterSection}>
        <div className={styles.recFilterLabel}>Filter by Status</div>
        <div className={styles.recFilterChips}>
          <button
            className={`${styles.recFilterChip} ${statusFilter === '' ? styles.active : ''}`}
            onClick={() => setStatusFilter('')}
          >
            All
          </button>
          <button
            className={`${styles.recFilterChip} ${statusFilter === 'pending' ? styles.active : ''} ${styles.pending}`}
            onClick={() => setStatusFilter('pending')}
          >
            Pending
          </button>
          <button
            className={`${styles.recFilterChip} ${statusFilter === 'acknowledged' ? styles.active : ''} ${styles.acknowledged}`}
            onClick={() => setStatusFilter('acknowledged')}
          >
            Acknowledged
          </button>
          <button
            className={`${styles.recFilterChip} ${statusFilter === 'applied' ? styles.active : ''} ${styles.applied}`}
            onClick={() => setStatusFilter('applied')}
          >
            Applied
          </button>
          <button
            className={`${styles.recFilterChip} ${statusFilter === 'rejected' ? styles.active : ''} ${styles.rejected}`}
            onClick={() => setStatusFilter('rejected')}
          >
            Rejected
          </button>
        </div>
      </div>

      {/* Recommendations List */}
      {recommendations.length === 0 ? (
        <div className={styles.emptyState}>
          <p>No recommendations found</p>
          <span>The LLM consensus engine will generate recommendations based on trade analysis and pattern discovery.</span>
        </div>
      ) : (
        <div className={styles.recCardsList}>
          {recommendations.map(rec => {
            const confidenceLevel = getConfidenceLevel(rec.confidence);
            const isExpanded = expandedId === rec.recommendation_id;

            return (
              <div
                key={rec.recommendation_id}
                className={`${styles.recCard} ${styles[rec.status]}`}
              >
                <div
                  className={styles.recCardHeader}
                  onClick={() => setExpandedId(isExpanded ? null : rec.recommendation_id)}
                >
                  <div className={styles.recCardLeft}>
                    <span className={styles.recCategoryIcon}>
                      {CATEGORY_ICONS[rec.category]}
                    </span>
                    <div className={styles.recCardMeta}>
                      <div className={styles.recCardTitle}>{rec.title}</div>
                      <div className={styles.recCardTags}>
                        <span className={styles.recCategoryTag}>
                          {CATEGORY_LABELS[rec.category]}
                        </span>
                        <span className={`${styles.recConfidenceTag} ${styles[confidenceLevel]}`}>
                          {(rec.confidence * 100).toFixed(0)}% confidence
                        </span>
                      </div>
                    </div>
                  </div>
                  <div className={styles.recCardRight}>
                    <span
                      className={styles.recStatusBadge}
                      style={{ backgroundColor: STATUS_COLORS[rec.status] }}
                    >
                      {STATUS_LABELS[rec.status]}
                    </span>
                    <span className={styles.recExpandIcon}>
                      {isExpanded ? '▼' : '▶'}
                    </span>
                  </div>
                </div>

                {isExpanded && (
                  <div className={styles.recCardBody}>
                    <p className={styles.recDescription}>{rec.description}</p>

                    <div className={styles.recActionBox}>
                      <div className={styles.recActionHeader}>
                        <span className={styles.recActionTitle}>Suggested Action</span>
                        <span className={styles.recActionType}>
                          {rec.suggested_action.action_type.replace(/_/g, ' ')}
                        </span>
                      </div>

                      <div className={styles.recActionGrid}>
                        <div className={styles.recActionItem}>
                          <span className={styles.recActionLabel}>Target</span>
                          <code className={styles.recActionCode}>{rec.suggested_action.target}</code>
                        </div>

                        {rec.suggested_action.current_value !== undefined && (
                          <div className={styles.recActionItem}>
                            <span className={styles.recActionLabel}>Current Value</span>
                            <pre className={styles.recValuePre}>
                              {formatValue(rec.suggested_action.current_value)}
                            </pre>
                          </div>
                        )}

                        <div className={`${styles.recActionItem} ${styles.suggested}`}>
                          <span className={styles.recActionLabel}>Suggested Value</span>
                          <pre className={styles.recValuePre}>
                            {formatValue(rec.suggested_action.suggested_value)}
                          </pre>
                        </div>
                      </div>

                      <div className={styles.recReasoning}>
                        <span className={styles.recReasoningLabel}>Reasoning</span>
                        <p>{rec.suggested_action.reasoning}</p>
                      </div>
                    </div>

                    <div className={styles.recSupportingData}>
                      <span className={styles.recDataItem}>
                        📊 {rec.supporting_data.trades_analyzed} trades analyzed
                      </span>
                      <span className={styles.recDataItem}>
                        📅 {rec.supporting_data.time_period}
                      </span>
                      <span className={styles.recDataItem}>
                        🔗 Source: {rec.source.replace(/_/g, ' ')}
                      </span>
                    </div>

                    <div className={styles.recTimestamps}>
                      <span>Created: {formatTimestamp(rec.created_at)}</span>
                      {rec.applied_at && (
                        <span>Applied: {formatTimestamp(rec.applied_at)}</span>
                      )}
                    </div>

                    {(rec.status === 'pending' || rec.status === 'acknowledged') && (
                      <div className={styles.recActions}>
                        {rec.status === 'pending' && (
                          <button
                            className={styles.recAcknowledgeBtn}
                            onClick={() => handleUpdateStatus(rec.recommendation_id, 'acknowledged')}
                            disabled={updating === rec.recommendation_id}
                          >
                            Acknowledge
                          </button>
                        )}
                        <button
                          className={styles.recApplyBtn}
                          onClick={() => handleUpdateStatus(rec.recommendation_id, 'applied')}
                          disabled={updating === rec.recommendation_id}
                        >
                          {rec.status === 'pending' ? 'Apply' : 'Apply Now'}
                        </button>
                        <button
                          className={styles.recRejectBtn}
                          onClick={() => handleUpdateStatus(rec.recommendation_id, 'rejected')}
                          disabled={updating === rec.recommendation_id}
                        >
                          Reject
                        </button>
                      </div>
                    )}
                  </div>
                )}
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
};

export default RecommendationsTab;
