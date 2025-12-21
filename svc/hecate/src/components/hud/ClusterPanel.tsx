import React, { useState, useEffect } from 'react';
import styles from './ClusterPanel.module.scss';

interface ClusterData {
  id: string;
  name: string;
  type: 'agent' | 'protocol' | 'service' | 'tool';
  status: 'healthy' | 'unhealthy' | 'unknown';
  description?: string;
  color: string;
  metrics?: {
    tasksProcessed?: number;
    uptime?: string;
    lastActive?: string;
  };
}

interface LogEntry {
  id: string;
  timestamp: string;
  message: string;
  level: 'info' | 'warning' | 'error' | 'success';
}

interface ClusterPanelProps {
  cluster: ClusterData;
  onClose: () => void;
  onDiveToCrossroads: (cluster: ClusterData) => void;
}

const ClusterPanel: React.FC<ClusterPanelProps> = ({
  cluster,
  onClose,
  onDiveToCrossroads,
}) => {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  // Real log fetching would connect to an API here
  useEffect(() => {
    // No mock data - show empty state until real logging is implemented
    setIsLoading(false);
    setLogs([]);
  }, [cluster.id]);

  const formatTimestamp = (timestamp: string) => {
    const date = new Date(timestamp);
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'healthy': return '#00ff9d';
      case 'unhealthy': return '#ff3333';
      default: return '#e8e8e8';
    }
  };

  const getLogLevelColor = (level: string) => {
    switch (level) {
      case 'success': return '#00ff9d';
      case 'warning': return '#e6c200';
      case 'error': return '#ff3333';
      default: return '#00d4ff';
    }
  };

  return (
    <div className={styles.panelOverlay} onClick={onClose}>
      <div
        className={styles.panel}
        onClick={(e) => e.stopPropagation()}
        style={{ '--cluster-color': cluster.color } as React.CSSProperties}
      >
        {/* Header */}
        <div className={styles.panelHeader}>
          <div className={styles.headerContent}>
            <div className={styles.clusterIcon} style={{ backgroundColor: cluster.color + '20', borderColor: cluster.color }}>
              <span style={{ color: cluster.color }}>
                {cluster.type === 'agent' ? '🤖' :
                 cluster.type === 'protocol' ? '🔗' :
                 cluster.type === 'tool' ? '🔧' : '⚙️'}
              </span>
            </div>
            <div className={styles.headerText}>
              <h3 className={styles.clusterName}>{cluster.name}</h3>
              <span className={styles.clusterType}>{cluster.type}</span>
            </div>
          </div>
          <button className={styles.closeButton} onClick={onClose} aria-label="Close panel">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M18 6L6 18M6 6l12 12" />
            </svg>
          </button>
        </div>

        {/* Status */}
        <div className={styles.statusSection}>
          <span
            className={styles.statusBadge}
            style={{ backgroundColor: getStatusColor(cluster.status) + '20', color: getStatusColor(cluster.status) }}
          >
            <span className={styles.statusDot} style={{ backgroundColor: getStatusColor(cluster.status) }} />
            {cluster.status}
          </span>
          {cluster.metrics?.uptime && (
            <span className={styles.uptime}>Uptime: {cluster.metrics.uptime}</span>
          )}
        </div>

        {/* Description */}
        {cluster.description && (
          <p className={styles.description}>{cluster.description}</p>
        )}

        {/* Metrics */}
        {cluster.metrics && (
          <div className={styles.metricsGrid}>
            {cluster.metrics.tasksProcessed !== undefined && (
              <div className={styles.metricCard}>
                <span className={styles.metricValue}>{cluster.metrics.tasksProcessed}</span>
                <span className={styles.metricLabel}>Tasks Processed</span>
              </div>
            )}
            {cluster.metrics.lastActive && (
              <div className={styles.metricCard}>
                <span className={styles.metricValue}>{cluster.metrics.lastActive}</span>
                <span className={styles.metricLabel}>Last Active</span>
              </div>
            )}
          </div>
        )}

        {/* Logs */}
        <div className={styles.logsSection}>
          <h4 className={styles.logsTitle}>Recent Activity</h4>
          <div className={styles.logsList}>
            {isLoading ? (
              <div className={styles.logsLoading}>Loading logs...</div>
            ) : logs.length === 0 ? (
              <div className={styles.noLogs}>No recent activity</div>
            ) : (
              logs.map((log) => (
                <div key={log.id} className={styles.logEntry}>
                  <span className={styles.logTime}>{formatTimestamp(log.timestamp)}</span>
                  <span
                    className={styles.logLevel}
                    style={{ color: getLogLevelColor(log.level) }}
                  >
                    [{log.level}]
                  </span>
                  <span className={styles.logMessage}>{log.message}</span>
                </div>
              ))
            )}
          </div>
        </div>

        {/* Footer */}
        <div className={styles.panelFooter}>
          <button
            className={styles.diveButton}
            onClick={() => onDiveToCrossroads(cluster)}
          >
            <span>Dive into Crossroads</span>
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M5 12h14M12 5l7 7-7 7" />
            </svg>
          </button>
        </div>
      </div>
    </div>
  );
};

export default ClusterPanel;
