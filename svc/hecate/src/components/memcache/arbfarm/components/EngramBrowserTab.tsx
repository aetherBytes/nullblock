import React, { useEffect, useState, useCallback } from 'react';
import styles from '../arbfarm.module.scss';
import { arbFarmService } from '../../../../common/services/arbfarm-service';
import type { Engram, EngramType, EngramBrowserFilter } from '../../../../types/engram';
import { A2A_TAG_LEARNING } from '../../../../types/engram';
import CodeBlock, { formatJson } from '../../../common/CodeBlock';

const ENGRAM_TYPE_LABELS: Record<EngramType, string> = {
  persona: 'Persona',
  preference: 'Preference',
  strategy: 'Strategy',
  knowledge: 'Knowledge',
  compliance: 'Compliance',
};

const ENGRAM_TYPE_COLORS: Record<EngramType, string> = {
  persona: '#9C27B0',
  preference: '#2196F3',
  strategy: '#4CAF50',
  knowledge: '#FF9800',
  compliance: '#F44336',
};

const EngramBrowserTab: React.FC = () => {
  const [engrams, setEngrams] = useState<Engram[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedEngram, setSelectedEngram] = useState<Engram | null>(null);
  const [filter, setFilter] = useState<EngramBrowserFilter>({
    limit: 50,
  });
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedType, setSelectedType] = useState<EngramType | ''>('');
  const [learningOnly, setLearningOnly] = useState(false);

  const fetchEngrams = useCallback(async () => {
    try {
      const filterToApply: EngramBrowserFilter = {
        ...filter,
        query: searchQuery || undefined,
        engram_type: selectedType || undefined,
        tags: learningOnly ? [A2A_TAG_LEARNING] : undefined,
      };

      const res = await arbFarmService.listEngrams(filterToApply);
      if (res.success && res.data) {
        setEngrams(res.data.engrams);
      }
    } catch (error) {
      console.error('Failed to fetch engrams:', error);
    } finally {
      setLoading(false);
    }
  }, [filter, searchQuery, selectedType, learningOnly]);

  useEffect(() => {
    setLoading(true);
    fetchEngrams();
  }, [fetchEngrams]);

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    fetchEngrams();
  };

  const formatTimestamp = (ts: string) => {
    const date = new Date(ts);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);

    if (diffMins < 1) return 'just now';
    if (diffMins < 60) return `${diffMins} min ago`;
    const diffHours = Math.floor(diffMins / 60);
    if (diffHours < 24) return `${diffHours}h ago`;
    const diffDays = Math.floor(diffHours / 24);
    if (diffDays < 7) return `${diffDays}d ago`;
    return date.toLocaleDateString();
  };

  const parseContent = (content: string) => {
    try {
      return JSON.parse(content);
    } catch {
      return content;
    }
  };

  const getTypeColor = (type: EngramType) => {
    return ENGRAM_TYPE_COLORS[type] || '#666';
  };

  const getTypeLabel = (type: EngramType) => {
    return ENGRAM_TYPE_LABELS[type] || type;
  };

  const truncateKey = (key: string, maxLen: number = 40) => {
    if (key.length <= maxLen) return key;
    return key.substring(0, maxLen) + '...';
  };

  if (loading && engrams.length === 0) {
    return (
      <div className={styles.loadingContainer}>
        <div className={styles.spinner} />
        <span>Loading engrams...</span>
      </div>
    );
  }

  return (
    <div className={styles.engramBrowserTab}>
      <div className={styles.sectionHeader}>
        <h3>Engram Browser</h3>
        <span className={styles.count}>{engrams.length} engrams</span>
      </div>

      <form className={styles.filterSection} onSubmit={handleSearch}>
        <div className={styles.searchRow}>
          <input
            type="text"
            placeholder="Search by key pattern (e.g., arb.learning.*)"
            value={searchQuery}
            onChange={e => setSearchQuery(e.target.value)}
            className={styles.searchInput}
          />
          <select
            value={selectedType}
            onChange={e => setSelectedType(e.target.value as EngramType | '')}
            className={styles.typeSelect}
          >
            <option value="">All Types</option>
            <option value="persona">Persona</option>
            <option value="preference">Preference</option>
            <option value="strategy">Strategy</option>
            <option value="knowledge">Knowledge</option>
            <option value="compliance">Compliance</option>
          </select>
          <label className={styles.checkboxLabel}>
            <input
              type="checkbox"
              checked={learningOnly}
              onChange={e => setLearningOnly(e.target.checked)}
            />
            Learning Only
          </label>
          <button type="submit" className={styles.searchButton}>
            Search
          </button>
        </div>
      </form>

      <div className={styles.browserLayout}>
        <div className={styles.engramsList}>
          {engrams.length === 0 ? (
            <div className={styles.emptyState}>
              <p>No engrams found</p>
              <span>Try adjusting your filters or search query.</span>
            </div>
          ) : (
            <table className={styles.engramsTable}>
              <thead>
                <tr>
                  <th>Key</th>
                  <th>Type</th>
                  <th>Updated</th>
                </tr>
              </thead>
              <tbody>
                {engrams.map(engram => (
                  <tr
                    key={engram.id}
                    className={selectedEngram?.id === engram.id ? styles.selected : ''}
                    onClick={() => setSelectedEngram(engram)}
                  >
                    <td className={styles.keyCell}>
                      <span title={engram.key}>{truncateKey(engram.key)}</span>
                    </td>
                    <td>
                      <span
                        className={styles.typeBadge}
                        style={{ backgroundColor: getTypeColor(engram.engram_type as EngramType) }}
                      >
                        {getTypeLabel(engram.engram_type as EngramType)}
                      </span>
                    </td>
                    <td className={styles.timestampCell}>
                      {formatTimestamp(engram.updated_at)}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>

        <div className={styles.engramDetail}>
          {selectedEngram ? (
            <>
              <div className={styles.detailHeader}>
                <h4>Engram Details</h4>
                <span
                  className={styles.typeBadge}
                  style={{ backgroundColor: getTypeColor(selectedEngram.engram_type as EngramType) }}
                >
                  {getTypeLabel(selectedEngram.engram_type as EngramType)}
                </span>
              </div>

              <div className={styles.detailMeta}>
                <div className={styles.metaItem}>
                  <label>Key</label>
                  <code>{selectedEngram.key}</code>
                </div>
                <div className={styles.metaItem}>
                  <label>Wallet</label>
                  <code>{selectedEngram.wallet_address}</code>
                </div>
                <div className={styles.metaItem}>
                  <label>Created</label>
                  <span>{new Date(selectedEngram.created_at).toLocaleString()}</span>
                </div>
                <div className={styles.metaItem}>
                  <label>Updated</label>
                  <span>{new Date(selectedEngram.updated_at).toLocaleString()}</span>
                </div>
                {selectedEngram.version && (
                  <div className={styles.metaItem}>
                    <label>Version</label>
                    <span>{selectedEngram.version}</span>
                  </div>
                )}
              </div>

              {selectedEngram.tags.length > 0 && (
                <div className={styles.tagsList}>
                  <label>Tags</label>
                  <div className={styles.tags}>
                    {selectedEngram.tags.map((tag, idx) => (
                      <span
                        key={idx}
                        className={`${styles.tag} ${tag === A2A_TAG_LEARNING ? styles.learningTag : ''}`}
                      >
                        {tag}
                      </span>
                    ))}
                  </div>
                </div>
              )}

              <div className={styles.contentSection}>
                <label>Content</label>
                <CodeBlock
                  code={formatJson(parseContent(selectedEngram.content))}
                  language="json"
                  maxHeight="300px"
                />
              </div>

              {selectedEngram.metadata && (
                <div className={styles.metadataSection}>
                  <label>Metadata</label>
                  <CodeBlock
                    code={formatJson(selectedEngram.metadata)}
                    language="json"
                    collapsible
                    defaultCollapsed
                    title="Metadata"
                  />
                </div>
              )}

              <div className={styles.detailActions}>
                {selectedEngram.parent_id && (
                  <span className={styles.lineageInfo}>
                    Forked from: {selectedEngram.parent_id.substring(0, 8)}...
                  </span>
                )}
              </div>
            </>
          ) : (
            <div className={styles.noSelection}>
              <p>Select an engram to view details</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default EngramBrowserTab;
