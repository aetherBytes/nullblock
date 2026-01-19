import React from 'react';
import styles from '../crossroads.module.scss';
import type { DiscoveredAgent } from '../../../types/crossroads';

interface AgentListProps {
  agents: DiscoveredAgent[];
  loading?: boolean;
}

const AgentList: React.FC<AgentListProps> = ({ agents, loading }) => {
  const getStatusIcon = (status: string): string => {
    switch (status) {
      case 'healthy': return '🟢';
      case 'degraded': return '🟡';
      case 'unhealthy': return '🔴';
      default: return '⚪';
    }
  };

  const getAgentTypeIcon = (type: string): string => {
    switch (type) {
      case 'conversational': return '💬';
      case 'specialized': return '🎯';
      default: return '🤖';
    }
  };

  if (loading) {
    return (
      <div className={styles.agentListLoading}>
        {[1, 2, 3].map(i => (
          <div key={i} className={styles.agentSkeleton}>
            <div className={styles.skeletonHeader} />
            <div className={styles.skeletonBody} />
          </div>
        ))}
      </div>
    );
  }

  if (agents.length === 0) {
    return (
      <div className={styles.agentListEmpty}>
        <span className={styles.emptyIcon}>🤖</span>
        <p>No agents discovered</p>
      </div>
    );
  }

  return (
    <div className={styles.agentList}>
      {agents.map((agent) => (
        <div key={agent.name} className={styles.agentCard}>
          <div className={styles.agentCardHeader}>
            <span className={styles.agentTypeIcon}>
              {getAgentTypeIcon(agent.agentType)}
            </span>
            <h3 className={styles.agentName}>{agent.name}</h3>
            <span className={styles.agentStatus}>
              {getStatusIcon(agent.status)} {agent.status}
            </span>
          </div>

          {agent.description && (
            <p className={styles.agentDescription}>{agent.description}</p>
          )}

          <div className={styles.agentMeta}>
            <div className={styles.agentMetaItem}>
              <span className={styles.metaLabel}>Type</span>
              <span className={styles.metaValue}>{agent.agentType}</span>
            </div>
            <div className={styles.agentMetaItem}>
              <span className={styles.metaLabel}>Provider</span>
              <span className={styles.metaValue}>{agent.provider}</span>
            </div>
            {agent.model && (
              <div className={styles.agentMetaItem}>
                <span className={styles.metaLabel}>Model</span>
                <span className={styles.metaValueCode}>{agent.model}</span>
              </div>
            )}
          </div>

          {agent.capabilities.length > 0 && (
            <div className={styles.agentCapabilities}>
              <span className={styles.capabilitiesLabel}>Capabilities:</span>
              <div className={styles.capabilitiesList}>
                {agent.capabilities.map((cap) => (
                  <span key={cap} className={styles.capabilityBadge}>
                    {cap}
                  </span>
                ))}
              </div>
            </div>
          )}

          <div className={styles.agentEndpoint}>
            <span className={styles.endpointLabel}>Endpoint:</span>
            <code className={styles.endpointValue}>{agent.endpoint}</code>
          </div>
        </div>
      ))}
    </div>
  );
};

export default AgentList;
