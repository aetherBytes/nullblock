import React from 'react';
import styles from '../crossroads.module.scss';
import type { DiscoveredProtocol } from '../../../types/crossroads';

interface ProtocolListProps {
  protocols: DiscoveredProtocol[];
  loading?: boolean;
}

const ProtocolList: React.FC<ProtocolListProps> = ({ protocols, loading }) => {
  const getProtocolIcon = (type: string): string => {
    switch (type.toLowerCase()) {
      case 'mcp': return '🔌';
      case 'a2a': return '🔗';
      default: return '📡';
    }
  };

  if (loading) {
    return (
      <div className={styles.protocolListLoading}>
        {[1, 2].map(i => (
          <div key={i} className={styles.protocolSkeleton}>
            <div className={styles.skeletonHeader} />
            <div className={styles.skeletonBody} />
          </div>
        ))}
      </div>
    );
  }

  if (protocols.length === 0) {
    return (
      <div className={styles.protocolListEmpty}>
        <span className={styles.emptyIcon}>📡</span>
        <p>No protocols discovered</p>
      </div>
    );
  }

  return (
    <div className={styles.protocolList}>
      {protocols.map((protocol) => (
        <div key={`${protocol.name}-${protocol.provider}`} className={styles.protocolCard}>
          <div className={styles.protocolCardHeader}>
            <span className={styles.protocolIcon}>
              {getProtocolIcon(protocol.protocolType)}
            </span>
            <h3 className={styles.protocolName}>{protocol.name}</h3>
            <span className={styles.protocolVersion}>v{protocol.version}</span>
          </div>

          {protocol.description && (
            <p className={styles.protocolDescription}>{protocol.description}</p>
          )}

          <div className={styles.protocolMeta}>
            <div className={styles.protocolMetaItem}>
              <span className={styles.metaLabel}>Type</span>
              <span className={styles.metaValueBadge}>{protocol.protocolType.toUpperCase()}</span>
            </div>
            <div className={styles.protocolMetaItem}>
              <span className={styles.metaLabel}>Provider</span>
              <span className={styles.metaValue}>{protocol.provider}</span>
            </div>
          </div>

          <div className={styles.protocolEndpoint}>
            <span className={styles.endpointLabel}>Endpoint:</span>
            <code className={styles.endpointValue}>{protocol.endpoint}</code>
          </div>
        </div>
      ))}
    </div>
  );
};

export default ProtocolList;
