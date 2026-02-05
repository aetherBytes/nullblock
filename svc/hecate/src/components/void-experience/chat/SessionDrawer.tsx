import React, { useState, useEffect } from 'react';
import { agentService } from '../../../common/services/agent-service';
import type { SessionSummary } from '../../../types/sessions';
import styles from './sessionDrawer.module.scss';

interface SessionDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  walletAddress: string | null;
  currentSessionId: string | null;
  onNewSession: () => void;
  onResumeSession: (sessionId: string) => void;
  onDeleteSession: (sessionId: string) => void;
}

const SessionDrawer: React.FC<SessionDrawerProps> = ({
  isOpen,
  onClose,
  walletAddress,
  currentSessionId,
  onNewSession,
  onResumeSession,
  onDeleteSession,
}) => {
  const [sessions, setSessions] = useState<SessionSummary[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (isOpen && walletAddress) {
      fetchSessions();
    }
  }, [isOpen, walletAddress]);

  const fetchSessions = async () => {
    if (!walletAddress) return;

    setIsLoading(true);
    setError(null);

    try {
      const response = await agentService.listSessions(walletAddress, 50);
      if (response.success && response.data) {
        setSessions(response.data.data || []);
      } else {
        setError(response.error || 'Failed to fetch sessions');
      }
    } catch (err) {
      setError('Failed to connect to service');
    } finally {
      setIsLoading(false);
    }
  };

  const handleDelete = async (e: React.MouseEvent, sessionId: string) => {
    e.stopPropagation();
    if (!walletAddress) return;

    try {
      const response = await agentService.deleteSession(walletAddress, sessionId);
      if (response.success) {
        setSessions(prev => prev.filter(s => s.session_id !== sessionId));
        onDeleteSession(sessionId);
      }
    } catch (err) {
      console.error('Failed to delete session:', err);
    }
  };

  const formatDate = (dateStr: string) => {
    const date = new Date(dateStr);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const days = Math.floor(diff / (1000 * 60 * 60 * 24));

    if (days === 0) {
      return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    } else if (days === 1) {
      return 'Yesterday';
    } else if (days < 7) {
      return `${days}d ago`;
    } else {
      return date.toLocaleDateString([], { month: 'short', day: 'numeric' });
    }
  };

  if (!isOpen) return null;

  return (
    <div className={styles.drawerOverlay} onClick={onClose}>
      <div className={styles.drawer} onClick={e => e.stopPropagation()}>
        <div className={styles.header}>
          <h3>Chat Sessions</h3>
          <button className={styles.closeButton} onClick={onClose}>
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M18 6L6 18M6 6l12 12" />
            </svg>
          </button>
        </div>

        <button className={styles.newSessionButton} onClick={onNewSession}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M12 5v14M5 12h14" />
          </svg>
          New Session
        </button>

        <div className={styles.sessionList}>
          {isLoading ? (
            <div className={styles.loading}>Loading sessions...</div>
          ) : error ? (
            <div className={styles.error}>{error}</div>
          ) : sessions.length === 0 ? (
            <div className={styles.empty}>No sessions yet. Start a new conversation.</div>
          ) : (
            sessions.map(session => (
              <div
                key={session.session_id}
                className={`${styles.sessionItem} ${session.session_id === currentSessionId ? styles.active : ''}`}
                onClick={() => onResumeSession(session.session_id)}
              >
                <div className={styles.sessionInfo}>
                  <div className={styles.sessionTitle}>
                    {session.is_pinned && <span className={styles.pinnedIcon}>*</span>}
                    {session.title || 'Untitled'}
                  </div>
                  <div className={styles.sessionMeta}>
                    <span>{session.message_count} messages</span>
                    <span>{formatDate(session.updated_at)}</span>
                  </div>
                </div>
                {!session.is_pinned && (
                  <button
                    className={styles.deleteButton}
                    onClick={(e) => handleDelete(e, session.session_id)}
                    title="Delete session"
                  >
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                      <polyline points="3 6 5 6 21 6" />
                      <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
                    </svg>
                  </button>
                )}
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
};

export default SessionDrawer;
