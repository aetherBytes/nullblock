import React, { useState, useEffect } from 'react';
import { ApiKeyProvider, ApiKeyResponse } from '../../types/user';
import { apiKeysService } from '../../common/services/api-keys-service';
import styles from './ApiKeyManagement.module.scss';

interface ApiKeyManagementProps {
  userId: string;
}

const PROVIDERS: { value: ApiKeyProvider; label: string }[] = [
  { value: 'openai', label: 'OpenAI' },
  { value: 'anthropic', label: 'Anthropic' },
  { value: 'groq', label: 'Groq' },
  { value: 'openrouter', label: 'OpenRouter' },
  { value: 'huggingface', label: 'HuggingFace' },
  { value: 'ollama', label: 'Ollama (Local)' },
];

const ApiKeyManagement: React.FC<ApiKeyManagementProps> = ({ userId }) => {
  const [keys, setKeys] = useState<ApiKeyResponse[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showAddForm, setShowAddForm] = useState(false);
  const [selectedProvider, setSelectedProvider] = useState<ApiKeyProvider>('openrouter');
  const [apiKeyInput, setApiKeyInput] = useState('');
  const [keyNameInput, setKeyNameInput] = useState('');
  const [showKeyInput, setShowKeyInput] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [deleteConfirm, setDeleteConfirm] = useState<string | null>(null);

  const loadKeys = async () => {
    setIsLoading(true);
    setError(null);
    try {
      const response = await apiKeysService.listKeys(userId);
      if (response.success && response.data) {
        setKeys(response.data);
      } else {
        setError(response.error || 'Failed to load API keys');
      }
    } catch (err) {
      setError('Network error loading API keys');
      console.error('Load keys error:', err);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    loadKeys();
  }, [userId]);

  const handleAddKey = async () => {
    if (!apiKeyInput.trim()) {
      setError('API key cannot be empty');
      return;
    }

    setIsSubmitting(true);
    setError(null);

    try {
      const response = await apiKeysService.createKey(userId, {
        provider: selectedProvider,
        api_key: apiKeyInput,
        key_name: keyNameInput.trim() || undefined,
      });

      if (response.success) {
        setApiKeyInput('');
        setKeyNameInput('');
        setShowAddForm(false);
        setShowKeyInput(false);
        await loadKeys();
      } else {
        setError(response.error || 'Failed to add API key');
      }
    } catch (err) {
      setError('Network error adding API key');
      console.error('Add key error:', err);
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleDeleteKey = async (keyId: string) => {
    setIsSubmitting(true);
    setError(null);

    try {
      const response = await apiKeysService.deleteKey(userId, keyId);

      if (response.success) {
        setDeleteConfirm(null);
        await loadKeys();
      } else {
        setError(response.error || 'Failed to delete API key');
      }
    } catch (err) {
      setError('Network error deleting API key');
      console.error('Delete key error:', err);
    } finally {
      setIsSubmitting(false);
    }
  };

  const formatLastUsed = (lastUsed?: string): string => {
    if (!lastUsed) return 'Never used';

    try {
      const date = new Date(lastUsed);
      const now = new Date();
      const diffMs = now.getTime() - date.getTime();
      const diffMins = Math.floor(diffMs / 60000);
      const diffHours = Math.floor(diffMs / 3600000);
      const diffDays = Math.floor(diffMs / 86400000);

      if (diffMins < 1) return 'Just now';
      if (diffMins < 60) return `${diffMins} min${diffMins > 1 ? 's' : ''} ago`;
      if (diffHours < 24) return `${diffHours} hour${diffHours > 1 ? 's' : ''} ago`;
      if (diffDays < 7) return `${diffDays} day${diffDays > 1 ? 's' : ''} ago`;

      return date.toLocaleDateString();
    } catch (err) {
      return 'Unknown';
    }
  };

  const getProviderIcon = (provider: string): string => {
    switch (provider.toLowerCase()) {
      case 'openai':
        return '🤖';
      case 'anthropic':
        return '🧠';
      case 'groq':
        return '⚡';
      case 'openrouter':
        return '🌐';
      case 'huggingface':
        return '🤗';
      case 'ollama':
        return '🦙';
      default:
        return '🔑';
    }
  };

  return (
    <div className={styles.apiKeyManagement}>
      {error && (
        <div className={styles.errorBanner}>
          <span>❌</span>
          <p>{error}</p>
          <button onClick={() => setError(null)}>Dismiss</button>
        </div>
      )}

      {!showAddForm && (
        <button className={styles.addButton} onClick={() => setShowAddForm(true)}>
          <span>➕</span>
          <span>Add New API Key</span>
        </button>
      )}

      {showAddForm && (
        <div className={styles.addForm}>
          <h4>Add New API Key</h4>

          <div className={styles.formGroup}>
            <label>Provider</label>
            <select
              value={selectedProvider}
              onChange={(e) => setSelectedProvider(e.target.value as ApiKeyProvider)}
              disabled={isSubmitting}
            >
              {PROVIDERS.map((provider) => (
                <option key={provider.value} value={provider.value}>
                  {provider.label}
                </option>
              ))}
            </select>
          </div>

          <div className={styles.formGroup}>
            <label>API Key</label>
            <div className={styles.passwordInput}>
              <input
                type={showKeyInput ? 'text' : 'password'}
                value={apiKeyInput}
                onChange={(e) => setApiKeyInput(e.target.value)}
                placeholder="Enter your API key..."
                disabled={isSubmitting}
              />
              <button
                type="button"
                className={styles.toggleVisibility}
                onClick={() => setShowKeyInput(!showKeyInput)}
              >
                {showKeyInput ? '👁️' : '🔒'}
              </button>
            </div>
          </div>

          <div className={styles.formGroup}>
            <label>Key Name (Optional)</label>
            <input
              type="text"
              value={keyNameInput}
              onChange={(e) => setKeyNameInput(e.target.value)}
              placeholder="e.g., Production Key"
              disabled={isSubmitting}
            />
          </div>

          <div className={styles.formActions}>
            <button
              className={styles.saveButton}
              onClick={handleAddKey}
              disabled={isSubmitting || !apiKeyInput.trim()}
            >
              {isSubmitting ? 'Saving...' : 'Save Key'}
            </button>
            <button
              className={styles.cancelButton}
              onClick={() => {
                setShowAddForm(false);
                setApiKeyInput('');
                setKeyNameInput('');
                setShowKeyInput(false);
              }}
              disabled={isSubmitting}
            >
              Cancel
            </button>
          </div>
        </div>
      )}

      {isLoading && (
        <div className={styles.loadingState}>
          <div className={styles.spinner} />
          <p>Loading API keys...</p>
        </div>
      )}

      {!isLoading && keys.length === 0 && !showAddForm && (
        <div className={styles.emptyState}>
          <span className={styles.emptyIcon}>🔑</span>
          <p>No API keys configured</p>
          <p className={styles.emptyHint}>Add your first API key to get started</p>
        </div>
      )}

      {!isLoading && keys.length > 0 && (
        <div className={styles.keysList}>
          {keys.map((key) => (
            <div key={key.id} className={styles.keyCard}>
              <div className={styles.keyHeader}>
                <div className={styles.keyTitle}>
                  <span className={styles.providerIcon}>{getProviderIcon(key.provider)}</span>
                  <h5>{key.key_name || key.provider.charAt(0).toUpperCase() + key.provider.slice(1)}</h5>
                </div>
                <span className={`${styles.statusBadge} ${key.is_active ? styles.active : styles.inactive}`}>
                  {key.is_active ? 'Active' : 'Inactive'}
                </span>
              </div>

              <div className={styles.keyDetails}>
                <div className={styles.keyValue}>
                  <span className={styles.keyPrefix}>{key.key_prefix || 'sk'}-...</span>
                  <span className={styles.keySuffix}>{key.key_suffix || '****'}</span>
                </div>

                <div className={styles.keyMeta}>
                  <div className={styles.metaItem}>
                    <span className={styles.metaLabel}>Last used:</span>
                    <span className={styles.metaValue}>{formatLastUsed(key.last_used_at)}</span>
                  </div>
                  <div className={styles.metaItem}>
                    <span className={styles.metaLabel}>Usage:</span>
                    <span className={styles.metaValue}>{key.usage_count} calls</span>
                  </div>
                </div>
              </div>

              <div className={styles.keyActions}>
                {deleteConfirm === key.id ? (
                  <div className={styles.confirmDelete}>
                    <p>Delete this key?</p>
                    <button
                      className={styles.confirmButton}
                      onClick={() => handleDeleteKey(key.id)}
                      disabled={isSubmitting}
                    >
                      {isSubmitting ? 'Deleting...' : 'Confirm'}
                    </button>
                    <button
                      className={styles.cancelButton}
                      onClick={() => setDeleteConfirm(null)}
                      disabled={isSubmitting}
                    >
                      Cancel
                    </button>
                  </div>
                ) : (
                  <button
                    className={styles.deleteButton}
                    onClick={() => setDeleteConfirm(key.id)}
                    disabled={isSubmitting}
                  >
                    🗑️ Delete
                  </button>
                )}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

export default ApiKeyManagement;
