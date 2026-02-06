import React, { useState, useEffect } from 'react';
import styles from './ContentView.module.scss';

interface ContentItem {
  id: string;
  theme: string;
  text: string;
  tags: string[];
  image_prompt: string | null;
  image_path: string | null;
  status: 'pending' | 'approved' | 'posted' | 'failed' | 'deleted';
  scheduled_at: string | null;
  posted_at: string | null;
  tweet_url: string | null;
  metadata: any;
  created_at: string;
  updated_at: string;
}

interface QueueResponse {
  items: ContentItem[];
  total: number;
}

const THEMES = [
  { value: 'morning_insight', label: 'Morning Insight', emoji: '🌅' },
  { value: 'progress_update', label: 'Progress Update', emoji: '🚀' },
  { value: 'educational', label: 'Educational', emoji: '📚' },
  { value: 'eerie_fun', label: 'Eerie Fun', emoji: '🌑' },
  { value: 'community', label: 'Community', emoji: '💬' },
];

const STATUS_COLORS: Record<string, string> = {
  pending: '#fbbf24',
  approved: '#34d399',
  posted: '#60a5fa',
  failed: '#f87171',
  deleted: '#9ca3af',
};

export const ContentView: React.FC = () => {
  const [content, setContent] = useState<ContentItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [generationEnabled, setGenerationEnabled] = useState(false);
  const [selectedTheme, setSelectedTheme] = useState<string>('morning_insight');
  const [includeImage, setIncludeImage] = useState(false);
  const [filterStatus, setFilterStatus] = useState<string>('all');
  const [refreshInterval, setRefreshInterval] = useState<number>(30000);

  useEffect(() => {
    fetchQueue();
    const interval = setInterval(fetchQueue, refreshInterval);
    return () => clearInterval(interval);
  }, [filterStatus, refreshInterval]);

  const fetchQueue = async () => {
    try {
      const statusParam = filterStatus !== 'all' ? `?status=${filterStatus}` : '';
      const response = await fetch(`http://localhost:3000/api/content/queue${statusParam}`);
      if (!response.ok) throw new Error(`HTTP ${response.status}`);
      const data: QueueResponse = await response.json();
      setContent(data.items);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch queue');
    }
  };

  const generateContent = async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await fetch('http://localhost:3000/api/content/generate', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          theme: selectedTheme,
          include_image: includeImage,
        }),
      });
      if (!response.ok) throw new Error(`HTTP ${response.status}`);
      await response.json();
      await fetchQueue();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to generate content');
    } finally {
      setLoading(false);
    }
  };

  const updateStatus = async (id: string, status: string) => {
    try {
      const response = await fetch(`http://localhost:3000/api/content/queue/${id}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ status }),
      });
      if (!response.ok) throw new Error(`HTTP ${response.status}`);
      await fetchQueue();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update status');
    }
  };

  const deleteContent = async (id: string) => {
    if (!confirm('Delete this content?')) return;
    try {
      const response = await fetch(`http://localhost:3000/api/content/queue/${id}`, {
        method: 'DELETE',
      });
      if (!response.ok) throw new Error(`HTTP ${response.status}`);
      await fetchQueue();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete content');
    }
  };

  const getThemeEmoji = (theme: string) => {
    return THEMES.find(t => t.value === theme)?.emoji || '📝';
  };

  return (
    <div className={styles.contentView}>
      <div className={styles.header}>
        <h2>📝 Content Service</h2>
        <div className={styles.headerActions}>
          <button
            className={`${styles.toggleButton} ${generationEnabled ? styles.enabled : styles.disabled}`}
            onClick={() => setGenerationEnabled(!generationEnabled)}
          >
            {generationEnabled ? '🟢 Generation ON' : '🔴 Generation OFF'}
          </button>
          <button className={styles.refreshButton} onClick={fetchQueue} disabled={loading}>
            🔄 Refresh
          </button>
        </div>
      </div>

      {error && (
        <div className={styles.error}>
          ⚠️ {error}
        </div>
      )}

      <div className={styles.controls}>
        <div className={styles.controlGroup}>
          <label>Theme:</label>
          <select
            value={selectedTheme}
            onChange={(e) => setSelectedTheme(e.target.value)}
            className={styles.select}
          >
            {THEMES.map(theme => (
              <option key={theme.value} value={theme.value}>
                {theme.emoji} {theme.label}
              </option>
            ))}
          </select>
        </div>

        <div className={styles.controlGroup}>
          <label>
            <input
              type="checkbox"
              checked={includeImage}
              onChange={(e) => setIncludeImage(e.target.checked)}
            />
            Include Image
          </label>
        </div>

        <button
          className={styles.generateButton}
          onClick={generateContent}
          disabled={loading || !generationEnabled}
        >
          {loading ? '⏳ Generating...' : '✨ Generate Content'}
        </button>

        <div className={styles.controlGroup}>
          <label>Filter:</label>
          <select
            value={filterStatus}
            onChange={(e) => setFilterStatus(e.target.value)}
            className={styles.select}
          >
            <option value="all">All</option>
            <option value="pending">Pending</option>
            <option value="approved">Approved</option>
            <option value="posted">Posted</option>
            <option value="failed">Failed</option>
          </select>
        </div>
      </div>

      <div className={styles.stats}>
        <div className={styles.stat}>
          <span className={styles.statLabel}>Total:</span>
          <span className={styles.statValue}>{content.length}</span>
        </div>
        <div className={styles.stat}>
          <span className={styles.statLabel}>Pending:</span>
          <span className={styles.statValue}>
            {content.filter(c => c.status === 'pending').length}
          </span>
        </div>
        <div className={styles.stat}>
          <span className={styles.statLabel}>Approved:</span>
          <span className={styles.statValue}>
            {content.filter(c => c.status === 'approved').length}
          </span>
        </div>
      </div>

      <div className={styles.queue}>
        {content.length === 0 ? (
          <div className={styles.empty}>
            No content in queue. Generate some content to get started!
          </div>
        ) : (
          content.map(item => (
            <div key={item.id} className={styles.contentItem}>
              <div className={styles.itemHeader}>
                <span className={styles.theme}>
                  {getThemeEmoji(item.theme)} {item.theme}
                </span>
                <span
                  className={styles.status}
                  style={{ backgroundColor: STATUS_COLORS[item.status] }}
                >
                  {item.status}
                </span>
                <span className={styles.date}>
                  {new Date(item.created_at).toLocaleString()}
                </span>
              </div>

              <div className={styles.itemText}>{item.text}</div>

              {item.tags.length > 0 && (
                <div className={styles.tags}>
                  {item.tags.map((tag, i) => (
                    <span key={i} className={styles.tag}>
                      #{tag}
                    </span>
                  ))}
                </div>
              )}

              {item.image_prompt && (
                <div className={styles.imagePrompt}>
                  🎨 Image: {item.image_prompt}
                </div>
              )}

              <div className={styles.actions}>
                {item.status === 'pending' && (
                  <button
                    className={styles.actionButton}
                    onClick={() => updateStatus(item.id, 'approved')}
                  >
                    ✅ Approve
                  </button>
                )}
                {item.status === 'approved' && (
                  <button
                    className={styles.actionButton}
                    onClick={() => updateStatus(item.id, 'pending')}
                  >
                    ⏪ Unapprove
                  </button>
                )}
                <button
                  className={`${styles.actionButton} ${styles.danger}`}
                  onClick={() => deleteContent(item.id)}
                >
                  🗑️ Delete
                </button>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
};
