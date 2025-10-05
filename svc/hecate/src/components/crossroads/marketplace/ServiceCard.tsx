import React from 'react';
import { Identity, Avatar, Name, Badge } from '@coinbase/onchainkit/identity';
import { base } from 'viem/chains';
import styles from '../crossroads.module.scss';
import CategoryBadge from '../shared/CategoryBadge';
import StatusBadge from '../shared/StatusBadge';
import type { ServiceListing } from '../types';

interface ServiceCardProps {
  service: ServiceListing;
  onClick?: () => void;
}

const ServiceCard: React.FC<ServiceCardProps> = ({ service, onClick }) => {
  const formatNumber = (num: number): string => {
    if (num >= 1000) {
      return `${(num / 1000).toFixed(1)}k`;
    }
    return num.toString();
  };

  const handleClick = () => {
    if (!service.is_coming_soon) {
      onClick?.();
    }
  };

  return (
    <div 
      className={`${styles.serviceCard} ${service.is_coming_soon ? styles.comingSoon : ''}`} 
      onClick={handleClick}
      style={{ cursor: service.is_coming_soon ? 'not-allowed' : 'pointer' }}
    >
      <div className={styles.cardHeader}>
        <div className={styles.headerLeft}>
          <div className={styles.serviceIcon}>
            {service.icon_url ? (
              <img src={service.icon_url} alt={service.title} style={{ width: '100%', height: '100%', objectFit: 'cover' }} />
            ) : (
              <span style={{ fontSize: '1.5rem' }}>
                {service.listing_type === 'Agent' && '🤖'}
                {service.listing_type === 'Workflow' && '🔄'}
                {service.listing_type === 'Tool' && '🔧'}
                {service.listing_type === 'McpServer' && '🌐'}
                {service.listing_type === 'Dataset' && '📊'}
                {service.listing_type === 'Model' && '🧠'}
              </span>
            )}
          </div>
          <div className={styles.serviceInfo}>
            <CategoryBadge category={service.listing_type} />
          </div>
        </div>
        {service.is_featured && (
          <div className={styles.featuredBadge}>
            ⭐ Featured
          </div>
        )}
      </div>

      {service.is_coming_soon && (
        <div className={styles.comingSoonOverlay}>
          <span className={styles.comingSoonBadge}>Coming Soon</span>
        </div>
      )}
      
      <div className={styles.cardBody}>
        <h3 className={styles.title}>{service.title}</h3>
        <p className={styles.description}>{service.short_description}</p>

        <div className={styles.ownerInfo}>
          <Identity
            address={service.owner_address as `0x${string}`}
            chain={base}
            className={styles.ownerIdentity}
          >
            <Avatar className={styles.ownerAvatar} />
            <Name className={styles.ownerName} />
            <Badge />
          </Identity>
        </div>

        <div className={styles.metrics}>
          {service.rating_average && (
            <span className={styles.rating}>
              ⭐ {service.rating_average.toFixed(1)} ({service.rating_count})
            </span>
          )}
          {service.deployment_count > 0 && (
            <span className={styles.deployments}>
              🚀 {formatNumber(service.deployment_count)}
            </span>
          )}
          <span className={styles.status}>
            <StatusBadge status={service.health_status} />
          </span>
        </div>

        <div className={`${styles.priceTag} ${service.is_free ? styles.free : ''}`}>
          {service.is_free ? (
            'Free'
          ) : (
            <>
              {service.price_usd ? `$${service.price_usd}` : `${service.price_eth} ETH`}
              {service.pricing_model === 'Subscription' && '/mo'}
            </>
          )}
        </div>
      </div>

      <div className={styles.cardActions}>
        <button className={styles.primary} onClick={(e) => { e.stopPropagation(); onClick?.(); }}>
          View Details
        </button>
        {service.is_free && (
          <button className={styles.secondary} onClick={(e) => { e.stopPropagation(); }}>
            Quick Deploy
          </button>
        )}
      </div>
    </div>
  );
};

export default ServiceCard;

