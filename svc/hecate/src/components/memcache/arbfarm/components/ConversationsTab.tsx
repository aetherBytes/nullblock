import React, { useEffect, useState, useCallback } from 'react';
import styles from '../arbfarm.module.scss';
import { arbFarmService } from '../../../../common/services/arbfarm-service';
import type { ConversationLog, ConversationTopic } from '../../../../types/consensus';

const TOPIC_LABELS: Record<ConversationTopic, string> = {
  trade_analysis: 'Trade Analysis',
  risk_assessment: 'Risk Assessment',
  strategy_review: 'Strategy Review',
  pattern_discovery: 'Pattern Discovery',
  market_conditions: 'Market Conditions',
};

const TOPIC_COLORS: Record<ConversationTopic, string> = {
  trade_analysis: '#4CAF50',
  risk_assessment: '#FF9800',
  strategy_review: '#2196F3',
  pattern_discovery: '#9C27B0',
  market_conditions: '#00BCD4',
};

const ConversationsTab: React.FC = () => {
  const [conversations, setConversations] = useState<ConversationLog[]>([]);
  const [loading, setLoading] = useState(true);
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [topicFilter, setTopicFilter] = useState<string>('');

  const fetchConversations = useCallback(async () => {
    try {
      const res = await arbFarmService.listConversations(50, topicFilter || undefined);
      if (res.success && res.data) {
        setConversations(res.data.conversations);
      }
    } catch (error) {
      console.error('Failed to fetch conversations:', error);
    } finally {
      setLoading(false);
    }
  }, [topicFilter]);

  useEffect(() => {
    setLoading(true);
    fetchConversations();
  }, [fetchConversations]);

  const formatTimestamp = (ts: string) => {
    const date = new Date(ts);
    return date.toLocaleString();
  };

  const getTopicColor = (topic: ConversationTopic) => {
    return TOPIC_COLORS[topic] || '#666';
  };

  const getTopicLabel = (topic: ConversationTopic) => {
    return TOPIC_LABELS[topic] || topic;
  };

  if (loading) {
    return (
      <div className={styles.loadingContainer}>
        <div className={styles.spinner} />
        <span>Loading conversations...</span>
      </div>
    );
  }

  return (
    <div className={styles.conversationsTab}>
      <div className={styles.sectionHeader}>
        <h3>Consensus Conversations</h3>
        <span className={styles.count}>{conversations.length} conversations</span>
      </div>

      <div className={styles.filterRow}>
        <label>Filter by Topic:</label>
        <select
          value={topicFilter}
          onChange={e => setTopicFilter(e.target.value)}
        >
          <option value="">All Topics</option>
          <option value="trade_analysis">Trade Analysis</option>
          <option value="risk_assessment">Risk Assessment</option>
          <option value="strategy_review">Strategy Review</option>
          <option value="pattern_discovery">Pattern Discovery</option>
          <option value="market_conditions">Market Conditions</option>
        </select>
      </div>

      {conversations.length === 0 ? (
        <div className={styles.emptyState}>
          <p>No conversations found</p>
          <span>Consensus conversations will appear here when the LLM engines analyze trades and patterns.</span>
        </div>
      ) : (
        <div className={styles.conversationsList}>
          {conversations.map(conv => (
            <div key={conv.session_id} className={styles.conversationItem}>
              <div
                className={styles.conversationHeader}
                onClick={() => setExpandedId(expandedId === conv.session_id ? null : conv.session_id)}
              >
                <div className={styles.headerLeft}>
                  <span
                    className={styles.topicBadge}
                    style={{ backgroundColor: getTopicColor(conv.topic) }}
                  >
                    {getTopicLabel(conv.topic)}
                  </span>
                  <span className={styles.timestamp}>
                    {formatTimestamp(conv.created_at)}
                  </span>
                </div>
                <div className={styles.headerRight}>
                  <span className={styles.participants}>
                    {conv.participants.length} participants
                  </span>
                  <span className={styles.messageCount}>
                    {conv.messages.length} messages
                  </span>
                  <span className={styles.expandIcon}>
                    {expandedId === conv.session_id ? '▼' : '▶'}
                  </span>
                </div>
              </div>

              {expandedId === conv.session_id && (
                <div className={styles.conversationContent}>
                  <div className={styles.contextSection}>
                    <h5>Context</h5>
                    <div className={styles.contextDetails}>
                      <span>Trigger: {conv.context.trigger.replace('_', ' ')}</span>
                      {conv.context.trades_in_scope && (
                        <span>Trades: {conv.context.trades_in_scope}</span>
                      )}
                      {conv.context.time_period && (
                        <span>Period: {conv.context.time_period}</span>
                      )}
                    </div>
                  </div>

                  <div className={styles.messagesSection}>
                    <h5>Messages</h5>
                    <div className={styles.messagesList}>
                      {conv.messages.map((msg, idx) => (
                        <div key={idx} className={styles.message}>
                          <div className={styles.messageHeader}>
                            <span className={styles.role}>{msg.role}</span>
                            <span className={styles.msgTimestamp}>
                              {formatTimestamp(msg.timestamp)}
                            </span>
                            {msg.latency_ms && (
                              <span className={styles.latency}>
                                {msg.latency_ms}ms
                              </span>
                            )}
                          </div>
                          <div className={styles.messageContent}>
                            {msg.content}
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>

                  <div className={styles.outcomeSection}>
                    <h5>Outcome</h5>
                    <div className={styles.outcomeDetails}>
                      <span className={conv.outcome.consensus_reached ? styles.success : styles.warning}>
                        {conv.outcome.consensus_reached ? 'Consensus Reached' : 'No Consensus'}
                      </span>
                      {conv.outcome.recommendations_generated > 0 && (
                        <span className={styles.recommendationCount}>
                          {conv.outcome.recommendations_generated} recommendations generated
                        </span>
                      )}
                      {conv.outcome.summary && (
                        <p className={styles.summary}>{conv.outcome.summary}</p>
                      )}
                    </div>
                  </div>

                  {conv.outcome.engram_refs.length > 0 && (
                    <div className={styles.refsSection}>
                      <h5>Related Engrams</h5>
                      <div className={styles.refsList}>
                        {conv.outcome.engram_refs.map((ref, idx) => (
                          <span key={idx} className={styles.refBadge}>
                            {ref}
                          </span>
                        ))}
                      </div>
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

export default ConversationsTab;
