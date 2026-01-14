import React, { useState } from 'react';
import type { Engram } from '../../types/engrams';
import { ENGRAM_TYPE_ICONS, ENGRAM_TYPE_COLORS } from '../../types/engrams';
import styles from './memcache.module.scss';

interface EngramsShelfProps {
  engrams: Engram[];
  isLoading: boolean;
  onDelete: (id: string) => void;
  onRefresh: () => void;
}

const EngramsShelf: React.FC<EngramsShelfProps> = ({ engrams, isLoading, onDelete, onRefresh }) => {
  const [expandedId, setExpandedId] = useState<string | null>(null);

  if (isLoading) {
    return (
      <div className={styles.engramsShelf}>
        <div className={styles.loadingState}>
          <div className={styles.loadingSpinner} />
          <p>Loading engrams...</p>
        </div>
      </div>
    );
  }

  if (engrams.length === 0) {
    return (
      <div className={styles.engramsShelf}>
        <div className={styles.emptyState}>
          <div className={styles.emptyIcon}>💾</div>
          <p className={styles.emptyTitle}>No engrams etched yet.</p>
          <p className={styles.emptySubtitle}>Explore the Crossroads to create scars.</p>
          <button className={styles.refreshButton} onClick={onRefresh}>
            Refresh
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.engramsShelf}>
      <div className={styles.engramsGrid}>
        {engrams.map((engram) => (
          <EngramCard
            key={engram.id}
            engram={engram}
            isExpanded={expandedId === engram.id}
            onToggle={() => setExpandedId(expandedId === engram.id ? null : engram.id)}
            onDelete={() => onDelete(engram.id)}
          />
        ))}
      </div>
    </div>
  );
};

interface EngramCardProps {
  engram: Engram;
  isExpanded: boolean;
  onToggle: () => void;
  onDelete: () => void;
}

const EngramCard: React.FC<EngramCardProps> = ({ engram, isExpanded, onToggle, onDelete }) => {
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const typeColor = ENGRAM_TYPE_COLORS[engram.engram_type];
  const typeIcon = ENGRAM_TYPE_ICONS[engram.engram_type];

  const formatDate = (dateStr: string) => {
    const date = new Date(dateStr);

    return date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
    });
  };

  const truncateContent = (content: string, maxLength: number = 120) => {
    if (content.length <= maxLength) {
      return content;
    }

    return `${content.substring(0, maxLength)}...`;
  };

  const handleDelete = (e: React.MouseEvent) => {
    e.stopPropagation();

    if (showDeleteConfirm) {
      onDelete();
      setShowDeleteConfirm(false);
    } else {
      setShowDeleteConfirm(true);
      setTimeout(() => setShowDeleteConfirm(false), 3000);
    }
  };

  return (
    <div
      className={`${styles.engramCard} ${isExpanded ? styles.expanded : ''}`}
      onClick={onToggle}
      style={{ '--type-color': typeColor } as React.CSSProperties}
    >
      <div className={styles.cardHeader}>
        <div className={styles.typeIndicator}>
          <span className={styles.typeIcon}>{typeIcon}</span>
          <span className={styles.typeName}>{engram.engram_type}</span>
        </div>
        {engram.version > 1 && <span className={styles.versionBadge}>v{engram.version}</span>}
      </div>

      <h3 className={styles.cardTitle}>{engram.key}</h3>

      <div className={styles.cardContent}>
        {isExpanded ? engram.content : truncateContent(engram.content)}
      </div>

      {engram.metadata && Object.keys(engram.metadata).length > 0 && (
        <div className={styles.cardMetadata}>
          {Object.entries(engram.metadata)
            .slice(0, 3)
            .map(([key, value]) => (
              <span key={key} className={styles.metaTag}>
                {key}: {String(value)}
              </span>
            ))}
        </div>
      )}

      <div className={styles.cardFooter}>
        <span className={styles.cardDate}>
          {formatDate(engram.updated_at || engram.created_at)}
        </span>
        <div className={styles.cardActions}>
          {engram.is_public && (
            <span className={styles.publicBadge} title="Published to Crossroads">
              🌐
            </span>
          )}
          <button
            className={`${styles.actionButton} ${showDeleteConfirm ? styles.confirmDelete : ''}`}
            onClick={handleDelete}
            title={showDeleteConfirm ? 'Click again to confirm' : 'Delete'}
          >
            {showDeleteConfirm ? '✓' : '🗑️'}
          </button>
        </div>
      </div>

      {isExpanded && (
        <div className={styles.expandedActions}>
          <button className={styles.expandedButton} onClick={(e) => e.stopPropagation()}>
            Refine
          </button>
          <button className={styles.expandedButton} onClick={(e) => e.stopPropagation()}>
            Fork
          </button>
          <button className={styles.expandedButton} onClick={(e) => e.stopPropagation()}>
            Publish
          </button>
        </div>
      )}
    </div>
  );
};

export default EngramsShelf;
