import React, { useState, useEffect, useCallback } from 'react';
import styles from './memcache.module.scss';
import EngramsShelf from './EngramsShelf';
import { Engram, EngramType } from '../../types/engrams';

interface MemCacheProps {
  publicKey: string | null;
}

const EREBUS_BASE_URL = import.meta.env.VITE_EREBUS_URL || 'http://localhost:3000';

const MemCache: React.FC<MemCacheProps> = ({ publicKey }) => {
  const [engrams, setEngrams] = useState<Engram[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedType, setSelectedType] = useState<EngramType | 'all'>('all');
  const [showCreateModal, setShowCreateModal] = useState(false);

  const fetchEngrams = useCallback(async () => {
    if (!publicKey) return;

    setIsLoading(true);
    setError(null);

    try {
      const url = `${EREBUS_BASE_URL}/api/engrams/wallet/${publicKey}`;
      const response = await fetch(url);

      if (!response.ok) {
        throw new Error(`Failed to fetch engrams: ${response.status}`);
      }

      const data = await response.json();
      setEngrams(data.data || data || []);
    } catch (err) {
      console.error('Error fetching engrams:', err);
      setError(err instanceof Error ? err.message : 'Failed to load engrams');
    } finally {
      setIsLoading(false);
    }
  }, [publicKey]);

  useEffect(() => {
    fetchEngrams();
  }, [fetchEngrams]);

  const handleDeleteEngram = async (engramId: string) => {
    try {
      const response = await fetch(`${EREBUS_BASE_URL}/api/engrams/${engramId}`, {
        method: 'DELETE',
      });

      if (!response.ok) {
        throw new Error('Failed to delete engram');
      }

      setEngrams(prev => prev.filter(e => e.id !== engramId));
    } catch (err) {
      console.error('Error deleting engram:', err);
      setError(err instanceof Error ? err.message : 'Failed to delete engram');
    }
  };

  const handleCreateEngram = async (newEngram: {
    engram_type: EngramType;
    key: string;
    content: string;
    metadata?: Record<string, unknown>;
  }) => {
    if (!publicKey) return;

    try {
      const response = await fetch(`${EREBUS_BASE_URL}/api/engrams`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          wallet_address: publicKey,
          ...newEngram,
        }),
      });

      if (!response.ok) {
        throw new Error('Failed to create engram');
      }

      const data = await response.json();
      setEngrams(prev => [data.data || data, ...prev]);
      setShowCreateModal(false);
    } catch (err) {
      console.error('Error creating engram:', err);
      setError(err instanceof Error ? err.message : 'Failed to create engram');
    }
  };

  const filteredEngrams = selectedType === 'all'
    ? engrams
    : engrams.filter(e => e.engram_type === selectedType);

  if (!publicKey) {
    return (
      <div className={styles.memcacheContainer}>
        <div className={styles.disconnectedState}>
          <div className={styles.disconnectedIcon}>🧠</div>
          <h2>The Mem Cache</h2>
          <p className={styles.tagline}>Your memories persist. The void remembers.</p>
          <p className={styles.connectPrompt}>Connect your wallet to access your engrams</p>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.memcacheContainer}>
      <div className={styles.memcacheHeader}>
        <div className={styles.headerLeft}>
          <h1 className={styles.title}>The Mem Cache</h1>
          <p className={styles.tagline}>Your memories persist. The void remembers.</p>
        </div>
        <div className={styles.headerRight}>
          <button
            className={styles.createButton}
            onClick={() => setShowCreateModal(true)}
          >
            + New Engram
          </button>
        </div>
      </div>

      <div className={styles.hecateOpener}>
        <span className={styles.hecateIcon}>◐</span>
        <div className={styles.hecateMessage}>
          <p>"Welcome to The Mem Cache, Neuron.</p>
          <p>Your engrams are safe here.</p>
          <p>What shall we retrieve?"</p>
        </div>
      </div>

      <div className={styles.filterBar}>
        <button
          className={`${styles.filterChip} ${selectedType === 'all' ? styles.active : ''}`}
          onClick={() => setSelectedType('all')}
        >
          All ({engrams.length})
        </button>
        {(['persona', 'preference', 'strategy', 'knowledge', 'compliance'] as EngramType[]).map(type => {
          const count = engrams.filter(e => e.engram_type === type).length;
          return (
            <button
              key={type}
              className={`${styles.filterChip} ${styles[type]} ${selectedType === type ? styles.active : ''}`}
              onClick={() => setSelectedType(type)}
            >
              {type.charAt(0).toUpperCase() + type.slice(1)} ({count})
            </button>
          );
        })}
      </div>

      {error && (
        <div className={styles.errorBanner}>
          <span>{error}</span>
          <button onClick={() => setError(null)}>Dismiss</button>
        </div>
      )}

      <EngramsShelf
        engrams={filteredEngrams}
        isLoading={isLoading}
        onDelete={handleDeleteEngram}
        onRefresh={fetchEngrams}
      />

      {showCreateModal && (
        <CreateEngramModal
          onClose={() => setShowCreateModal(false)}
          onCreate={handleCreateEngram}
        />
      )}
    </div>
  );
};

interface CreateEngramModalProps {
  onClose: () => void;
  onCreate: (engram: {
    engram_type: EngramType;
    key: string;
    content: string;
    metadata?: Record<string, unknown>;
  }) => void;
}

const CreateEngramModal: React.FC<CreateEngramModalProps> = ({ onClose, onCreate }) => {
  const [engramType, setEngramType] = useState<EngramType>('knowledge');
  const [key, setKey] = useState('');
  const [content, setContent] = useState('');

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!key.trim() || !content.trim()) return;
    onCreate({ engram_type: engramType, key: key.trim(), content: content.trim() });
  };

  return (
    <div className={styles.modalOverlay} onClick={onClose}>
      <div className={styles.modal} onClick={e => e.stopPropagation()}>
        <div className={styles.modalHeader}>
          <h2>Create New Engram</h2>
          <button className={styles.closeButton} onClick={onClose}>×</button>
        </div>
        <form onSubmit={handleSubmit} className={styles.modalForm}>
          <div className={styles.formGroup}>
            <label>Type</label>
            <select
              value={engramType}
              onChange={e => setEngramType(e.target.value as EngramType)}
            >
              <option value="persona">Persona</option>
              <option value="preference">Preference</option>
              <option value="strategy">Strategy</option>
              <option value="knowledge">Knowledge</option>
              <option value="compliance">Compliance</option>
            </select>
          </div>
          <div className={styles.formGroup}>
            <label>Key</label>
            <input
              type="text"
              value={key}
              onChange={e => setKey(e.target.value)}
              placeholder="e.g., trading_style, risk_tolerance"
            />
          </div>
          <div className={styles.formGroup}>
            <label>Content</label>
            <textarea
              value={content}
              onChange={e => setContent(e.target.value)}
              placeholder="Enter the engram content..."
              rows={5}
            />
          </div>
          <div className={styles.modalActions}>
            <button type="button" className={styles.cancelButton} onClick={onClose}>
              Cancel
            </button>
            <button type="submit" className={styles.submitButton} disabled={!key.trim() || !content.trim()}>
              Create Engram
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};

export default MemCache;
