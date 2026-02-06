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
  onCleanup?: () => void;
}

const SessionDrawer: React.FC<SessionDrawerProps> = ({
  isOpen,
  onClose,
  walletAddress,
  currentSessionId,
  onNewSession,
  onResumeSession,
  onDeleteSession,
  onCleanup,
}) => {
  const [sessions, setSessions] = useState<SessionSummary[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showCleanupConfirm, setShowCleanupConfirm] = useState(false);
  const [isCleaningUp, setIsCleaningUp] = useState(false);

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

  const handlePin = async (e: React.MouseEvent, sessionId: string, isPinned: boolean) => {
    e.stopPropagation();
    if (!walletAddress) return;

    try {
      const response = isPinned
        ? await agentService.unpinSession(walletAddress, sessionId)
        : await agentService.pinSession(walletAddress, sessionId);

      if (response.success) {
        setSessions(prev => prev.map(s =>
          s.session_id === sessionId
            ? { ...s, is_pinned: !isPinned }
            : s
        ));
      }
    } catch (err) {
      console.error('Failed to toggle pin:', err);
    }
  };

  const handleCleanup = async () => {
    if (!walletAddress) return;

    setIsCleaningUp(true);
    try {
      const response = await agentService.cleanupSessions(walletAddress);
      if (response.success) {
        await fetchSessions();
        setShowCleanupConfirm(false);
        onCleanup?.();
      }
    } catch (err) {
      console.error('Failed to cleanup sessions:', err);
    } finally {
      setIsCleaningUp(false);
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

  const truncateSummary = (summary: string | undefined, maxLen: number = 40) => {
    if (!summary) return null;
    return summary.length > maxLen ? `${summary.substring(0, maxLen)}...` : summary;
  };

  if (!isOpen) return null;

  return (
    <>
      <div className={styles.drawerOverlay} onClick={onClose}>
        <div className={styles.drawer} onClick={e => e.stopPropagation()}>
          <div className={styles.header}>
            <h3>Chat Sessions</h3>
            <div className={styles.headerActions}>
              <button
                className={styles.cleanupAllButton}
                onClick={() => setShowCleanupConfirm(true)}
                title="Cleanup old sessions"
                disabled={sessions.length === 0}
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6" />
                  <path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
                </svg>
              </button>
              <button className={styles.closeButton} onClick={onClose}>
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M18 6L6 18M6 6l12 12" />
                </svg>
              </button>
            </div>
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
                      {session.is_pinned && <span className={styles.pinnedIcon}>📌</span>}
                      {session.title || 'Untitled'}
                    </div>
                    <div className={styles.sessionMeta}>
                      <span>{session.message_count} messages</span>
                      <span>{formatDate(session.updated_at)}</span>
                    </div>
                    {session.summary && (
                      <div className={styles.sessionSummary}>
                        {truncateSummary(session.summary)}
                      </div>
                    )}
                  </div>
                  <div className={styles.sessionActions}>
                    <button
                      className={`${styles.pinButton} ${session.is_pinned ? styles.isPinned : ''}`}
                      onClick={(e) => handlePin(e, session.session_id, session.is_pinned)}
                      title={session.is_pinned ? 'Unpin session' : 'Pin session'}
                    >
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                        <path d="M12 2L12 12" />
                        <circle cx="12" cy="16" r="4" />
                      </svg>
                    </button>
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
                </div>
              ))
            )}
          </div>
        </div>
      </div>

      {showCleanupConfirm && (
        <div className={styles.confirmDialog} onClick={() => setShowCleanupConfirm(false)}>
          <div className={styles.confirmContent} onClick={e => e.stopPropagation()}>
            <h4>Cleanup Sessions?</h4>
            <p>
              This will delete old sessions, keeping only the 5 most recent and any pinned sessions.
              This action cannot be undone.
            </p>
            <div className={styles.confirmActions}>
              <button
                className={styles.confirmCancel}
                onClick={() => setShowCleanupConfirm(false)}
              >
                Cancel
              </button>
              <button
                className={styles.confirmDelete}
                onClick={handleCleanup}
                disabled={isCleaningUp}
              >
                {isCleaningUp ? 'Cleaning up...' : 'Cleanup'}
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
};

export default SessionDrawer;
