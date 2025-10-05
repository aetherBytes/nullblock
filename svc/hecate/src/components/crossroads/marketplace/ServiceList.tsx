import React, { useState } from 'react';
import styles from '../crossroads.module.scss';
import CategoryBadge from '../shared/CategoryBadge';
import StatusBadge from '../shared/StatusBadge';
import type { ServiceListing } from '../types';

interface ServiceListProps {
  services: ServiceListing[];
  loading?: boolean;
  onServiceClick?: (service: ServiceListing) => void;
}

const ServiceList: React.FC<ServiceListProps> = ({ services, loading, onServiceClick }) => {
  const [expandedId, setExpandedId] = useState<string | null>(null);

  const handleServiceClick = (service: ServiceListing) => {
    if (service.is_coming_soon) {
      return; // Don't allow expanding coming soon services
    }
    
    if (expandedId === service.id) {
      setExpandedId(null);
    } else {
      setExpandedId(service.id);
      onServiceClick?.(service);
    }
  };

  if (loading) {
    return (
      <div className={styles.serviceList}>
        {[...Array(6)].map((_, i) => (
          <div key={i} className={styles.listItemSkeleton}>
            <div className={styles.skeletonLine} />
            <div className={`${styles.skeletonLine} ${styles.short}`} />
          </div>
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
    <div className={styles.serviceList}>
      {services.map((service) => (
        <div
          key={service.id}
          className={`${styles.listItem} ${expandedId === service.id ? styles.expanded : ''} ${service.is_coming_soon ? styles.comingSoon : ''}`}
        >
          {service.is_coming_soon && (
            <div className={styles.comingSoonOverlay}>
              <span className={styles.comingSoonBadge}>Coming Soon</span>
            </div>
          )}
          
          {/* Main service row */}
          <div
            className={styles.listItemMain}
            onClick={() => handleServiceClick(service)}
            style={{ cursor: service.is_coming_soon ? 'not-allowed' : 'pointer' }}
          >
            <div className={styles.listItemLeft}>
              <div className={styles.listItemIcon}>
                {service.listing_type === 'Agent' && '🤖'}
                {service.listing_type === 'McpServer' && '🌐'}
                {service.listing_type === 'Workflow' && '🔄'}
                {service.listing_type === 'Tool' && '🔧'}
                {service.listing_type === 'Dataset' && '📊'}
                {service.listing_type === 'Model' && '🧠'}
              </div>
              <div className={styles.listItemInfo}>
                <h3 className={styles.listItemTitle}>{service.title}</h3>
                <p className={styles.listItemDescription}>{service.short_description}</p>
              </div>
            </div>

            <div className={styles.listItemRight}>
              <CategoryBadge category={service.listing_type} />
              <StatusBadge status={service.health_status} />
              <div className={styles.listItemMeta}>
                <span className={styles.metaRating}>
                  ⭐ {service.rating_average?.toFixed(1) || 'N/A'}
                </span>
                <span className={styles.metaPrice}>
                  {service.is_free ? 'Free' : service.price_usd ? `$${service.price_usd}` : 'Paid'}
                </span>
              </div>
              <div className={styles.expandIcon}>
                {expandedId === service.id ? '▼' : '▶'}
              </div>
            </div>
          </div>

          {/* Expanded details panel */}
          {expandedId === service.id && (
            <div className={styles.listItemDetails}>
              <div className={styles.detailsGrid}>
                {/* Capabilities */}
                <div className={styles.detailSection}>
                  <h4 className={styles.detailTitle}>Capabilities</h4>
                  <div className={styles.capabilityList}>
                    {service.capabilities.map((cap, idx) => (
                      <div key={idx} className={styles.capabilityItem}>
                        ✓ {cap}
                      </div>
                    ))}
                  </div>
                </div>

                {/* Tags */}
                <div className={styles.detailSection}>
                  <h4 className={styles.detailTitle}>Tags</h4>
                  <div className={styles.tagList}>
                    {service.category_tags.map((tag, idx) => (
                      <span key={idx} className={styles.tag}>
                        {tag}
                      </span>
                    ))}
                  </div>
                </div>

                {/* Stats */}
                <div className={styles.detailSection}>
                  <h4 className={styles.detailTitle}>Statistics</h4>
                  <div className={styles.statsList}>
                    <div className={styles.statItem}>
                      <span className={styles.statLabel}>Deployments:</span>
                      <span className={styles.statValue}>{service.deployment_count?.toLocaleString() || 0}</span>
                    </div>
                    <div className={styles.statItem}>
                      <span className={styles.statLabel}>Rating:</span>
                      <span className={styles.statValue}>
                        ⭐ {service.rating_average?.toFixed(1) || 'N/A'} ({service.rating_count || 0} reviews)
                      </span>
                    </div>
                    <div className={styles.statItem}>
                      <span className={styles.statLabel}>Updated:</span>
                      <span className={styles.statValue}>
                        {new Date(service.updated_at).toLocaleDateString()}
                      </span>
                    </div>
                  </div>
                </div>

                {/* Actions */}
                <div className={styles.detailSection}>
                  <h4 className={styles.detailTitle}>Actions</h4>
                  <div className={styles.actionButtons}>
                    <button className={styles.primaryButton}>
                      View Details
                    </button>
                    <button className={styles.secondaryButton}>
                      Deploy
                    </button>
                    <button className={styles.secondaryButton}>
                      Save
                    </button>
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>
      ))}
    </div>
  );
};

export default ServiceList;

