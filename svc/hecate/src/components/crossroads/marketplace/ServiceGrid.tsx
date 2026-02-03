import React from 'react';
import styles from '../crossroads.module.scss';
import type { ServiceListing } from '../types';
import ServiceCard from './ServiceCard';

interface ServiceGridProps {
  services: ServiceListing[];
  loading?: boolean;
  viewMode?: 'grid' | 'list';
  onServiceClick?: (service: ServiceListing) => void;
}

const SkeletonCard: React.FC = () => (
  <div className={styles.skeletonCard}>
    <div className={styles.skeletonHeader}>
      <div className={styles.skeletonIcon} />
      <div className={styles.skeletonText}>
        <div className={styles.skeletonLine} />
        <div className={`${styles.skeletonLine} ${styles.short}`} />
      </div>
    </div>
    <div className={styles.skeletonBody}>
      <div className={styles.skeletonLine} />
      <div className={styles.skeletonLine} />
      <div className={`${styles.skeletonLine} ${styles.short}`} />
    </div>
  </div>
);

const ServiceGrid: React.FC<ServiceGridProps> = ({
  services,
  loading,
  viewMode: _viewMode = 'grid',
  onServiceClick,
}) => {
  if (loading) {
    return (
      <div className={styles.loadingGrid}>
        {[...Array(6)].map((_, i) => (
          <SkeletonCard key={i} />
        ))}
      </div>
    );
  }

  if (services.length === 0) {
    return (
      <div className={styles.emptyState}>
        <div className={styles.emptyIcon}>🔍</div>
        <h3>No services found</h3>
        <p>Try adjusting your filters or search terms</p>
        <button onClick={() => window.location.reload()}>Clear Filters</button>
      </div>
    );
  }

  return (
    <div className={styles.serviceGrid}>
      {services.map((service) => (
        <ServiceCard key={service.id} service={service} onClick={() => onServiceClick?.(service)} />
      ))}
    </div>
  );
};

export default ServiceGrid;
