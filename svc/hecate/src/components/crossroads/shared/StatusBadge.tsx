import React from 'react';
import styles from '../crossroads.module.scss';

export type ServiceStatus = 'healthy' | 'degraded' | 'unhealthy' | 'inactive';

interface StatusBadgeProps {
  status: ServiceStatus;
}

const statusLabels: Record<ServiceStatus, string> = {
  healthy: 'Active',
  degraded: 'Degraded',
  unhealthy: 'Offline',
  inactive: 'Inactive',
};

const StatusBadge: React.FC<StatusBadgeProps> = ({ status }) => {
  return (
    <span className={`${styles.statusBadge} ${styles[status]}`}>
      <span className={styles.statusDot} />
      <span>{statusLabels[status]}</span>
    </span>
  );
};

export default StatusBadge;

