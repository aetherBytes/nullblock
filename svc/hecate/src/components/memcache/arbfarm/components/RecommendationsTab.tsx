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
  pending: '#FF9800',
  acknowledged: '#2196F3',
  applied: '#4CAF50',
  rejected: '#F44336',
};

const CATEGORY_LABELS: Record<RecommendationCategory, string> = {
  strategy: 'Strategy',
  risk: 'Risk',
  timing: 'Timing',
  venue: 'Venue',
  position: 'Position',
};

const CATEGORY_COLORS: Record<RecommendationCategory, string> = {
  strategy: '#4CAF50',
  risk: '#FF9800',
  timing: '#2196F3',
  venue: '#9C27B0',
  position: '#00BCD4',
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

  const getStatusColor = (status: RecommendationStatus) => {
    return STATUS_COLORS[status] || '#666';
  };

  const getStatusLabel = (status: RecommendationStatus) => {
    return STATUS_LABELS[status] || status;
  };

  const getCategoryColor = (category: RecommendationCategory) => {
    return CATEGORY_COLORS[category] || '#666';
  };

  const getCategoryLabel = (category: RecommendationCategory) => {
    return CATEGORY_LABELS[category] || category;
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
    <div className={styles.recommendationsTab}>
      <div className={styles.sectionHeader}>
        <h3>Learning Recommendations</h3>
      </div>

      {summary && (
        <div className={styles.summaryStats}>
          <div className={styles.statCard}>
            <span className={styles.statValue}>{summary.total_recommendations}</span>
            <span className={styles.statLabel}>Total</span>
          </div>
          <div className={`${styles.statCard} ${styles.pending}`}>
            <span className={styles.statValue}>{summary.pending_recommendations}</span>
            <span className={styles.statLabel}>Pending</span>
          </div>
          <div className={`${styles.statCard} ${styles.applied}`}>
            <span className={styles.statValue}>{summary.applied_recommendations}</span>
            <span className={styles.statLabel}>Applied</span>
          </div>
          <div className={styles.statCard}>
            <span className={styles.statValue}>{summary.total_conversations}</span>
            <span className={styles.statLabel}>Conversations</span>
          </div>
        </div>
      )}

      <div className={styles.filterRow}>
        <label>Filter by Status:</label>
        <select
          value={statusFilter}
          onChange={e => setStatusFilter(e.target.value)}
        >
          <option value="">All</option>
          <option value="pending">Pending</option>
          <option value="acknowledged">Acknowledged</option>
          <option value="applied">Applied</option>
          <option value="rejected">Rejected</option>
        </select>
      </div>

      {recommendations.length === 0 ? (
        <div className={styles.emptyState}>
          <p>No recommendations found</p>
          <span>The LLM consensus engine will generate recommendations based on trade analysis and pattern discovery.</span>
        </div>
      ) : (
        <div className={styles.recommendationsList}>
          {recommendations.map(rec => (
            <div key={rec.recommendation_id} className={styles.recommendationItem}>
              <div
                className={styles.recommendationHeader}
                onClick={() => setExpandedId(expandedId === rec.recommendation_id ? null : rec.recommendation_id)}
              >
                <div className={styles.headerLeft}>
                  <span
                    className={styles.categoryBadge}
                    style={{ backgroundColor: getCategoryColor(rec.category) }}
                  >
                    {getCategoryLabel(rec.category)}
                  </span>
                  <span className={styles.title}>{rec.title}</span>
                </div>
                <div className={styles.headerRight}>
                  <span
                    className={styles.statusBadge}
                    style={{ backgroundColor: getStatusColor(rec.status) }}
                  >
                    {getStatusLabel(rec.status)}
                  </span>
                  <span className={styles.confidence}>
                    {(rec.confidence * 100).toFixed(0)}% confidence
                  </span>
                  <span className={styles.expandIcon}>
                    {expandedId === rec.recommendation_id ? '▼' : '▶'}
                  </span>
                </div>
              </div>

              {expandedId === rec.recommendation_id && (
                <div className={styles.recommendationContent}>
                  <p className={styles.description}>{rec.description}</p>

                  <div className={styles.suggestedAction}>
                    <h5>Suggested Action</h5>
                    <div className={styles.actionDetails}>
                      <div className={styles.actionRow}>
                        <span className={styles.actionLabel}>Type:</span>
                        <span className={styles.actionValue}>
                          {rec.suggested_action.action_type.replace('_', ' ')}
                        </span>
                      </div>
                      <div className={styles.actionRow}>
                        <span className={styles.actionLabel}>Target:</span>
                        <code>{rec.suggested_action.target}</code>
                      </div>
                      {rec.suggested_action.current_value !== undefined && (
                        <div className={styles.actionRow}>
                          <span className={styles.actionLabel}>Current:</span>
                          <pre className={styles.valueCode}>
                            {formatValue(rec.suggested_action.current_value)}
                          </pre>
                        </div>
                      )}
                      <div className={styles.actionRow}>
                        <span className={styles.actionLabel}>Suggested:</span>
                        <pre className={`${styles.valueCode} ${styles.suggested}`}>
                          {formatValue(rec.suggested_action.suggested_value)}
                        </pre>
                      </div>
                      <div className={styles.reasoningRow}>
                        <span className={styles.actionLabel}>Reasoning:</span>
                        <p>{rec.suggested_action.reasoning}</p>
                      </div>
                    </div>
                  </div>

                  <div className={styles.supportingData}>
                    <h5>Supporting Data</h5>
                    <div className={styles.dataDetails}>
                      <span>Trades Analyzed: {rec.supporting_data.trades_analyzed}</span>
                      <span>Time Period: {rec.supporting_data.time_period}</span>
                    </div>
                  </div>

                  <div className={styles.metaInfo}>
                    <span>Source: {rec.source.replace('_', ' ')}</span>
                    <span>Created: {formatTimestamp(rec.created_at)}</span>
                    {rec.applied_at && (
                      <span>Applied: {formatTimestamp(rec.applied_at)}</span>
                    )}
                  </div>

                  {rec.status === 'pending' && (
                    <div className={styles.actionButtons}>
                      <button
                        className={styles.acknowledgeButton}
                        onClick={() => handleUpdateStatus(rec.recommendation_id, 'acknowledged')}
                        disabled={updating === rec.recommendation_id}
                      >
                        Acknowledge
                      </button>
                      <button
                        className={styles.applyButton}
                        onClick={() => handleUpdateStatus(rec.recommendation_id, 'applied')}
                        disabled={updating === rec.recommendation_id}
                      >
                        Apply
                      </button>
                      <button
                        className={styles.rejectButton}
                        onClick={() => handleUpdateStatus(rec.recommendation_id, 'rejected')}
                        disabled={updating === rec.recommendation_id}
                      >
                        Reject
                      </button>
                    </div>
                  )}

                  {rec.status === 'acknowledged' && (
                    <div className={styles.actionButtons}>
                      <button
                        className={styles.applyButton}
                        onClick={() => handleUpdateStatus(rec.recommendation_id, 'applied')}
                        disabled={updating === rec.recommendation_id}
                      >
                        Apply Now
                      </button>
                      <button
                        className={styles.rejectButton}
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
          ))}
        </div>
      )}
    </div>
  );
};

export default RecommendationsTab;
