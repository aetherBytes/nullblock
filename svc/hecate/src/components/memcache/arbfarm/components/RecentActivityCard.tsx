import React, { useState, useEffect, useRef } from 'react';
import styles from '../arbfarm.module.scss';

export interface ActivityEvent {
  id?: string;
  event_type: string;
  timestamp: string;
  payload?: Record<string, unknown>;
  source?: {
    type: string;
    id: string;
  };
}

interface RecentActivityCardProps {
  events: ActivityEvent[];
  onEventClick?: (event: ActivityEvent) => void;
  maxEvents?: number;
}

const EVENT_TYPE_CONFIG: Record<string, { icon: string; label: string; color: string }> = {
  'edge_detected': { icon: '🔍', label: 'Edge Detected', color: '#3b82f6' },
  'edge_executed': { icon: '⚡', label: 'Edge Executed', color: '#22c55e' },
  'edge_failed': { icon: '❌', label: 'Execution Failed', color: '#ef4444' },
  'position_opened': { icon: '📈', label: 'Position Opened', color: '#22c55e' },
  'position_closed': { icon: '📉', label: 'Position Closed', color: '#f59e0b' },
  'stop_loss_triggered': { icon: '🛑', label: 'Stop Loss', color: '#ef4444' },
  'take_profit_triggered': { icon: '🎯', label: 'Take Profit', color: '#22c55e' },
  'trailing_stop_triggered': { icon: '📍', label: 'Trailing Stop', color: '#f59e0b' },
  'curve_tracked': { icon: '👁️', label: 'Curve Tracked', color: '#8b5cf6' },
  'curve_graduated': { icon: '🎓', label: 'Graduation', color: '#22c55e' },
  'auto_execution': { icon: '🤖', label: 'Auto Execution', color: '#3b82f6' },
  'approval_created': { icon: '📝', label: 'Approval Created', color: '#f59e0b' },
  'approval_approved': { icon: '✅', label: 'Approved', color: '#22c55e' },
  'approval_rejected': { icon: '🚫', label: 'Rejected', color: '#ef4444' },
  'wallet_sync': { icon: '🔄', label: 'Wallet Sync', color: '#6366f1' },
  'monitor_started': { icon: '▶️', label: 'Monitor Started', color: '#22c55e' },
  'monitor_stopped': { icon: '⏹️', label: 'Monitor Stopped', color: '#6b7280' },
  'price_update': { icon: '💹', label: 'Price Update', color: '#3b82f6' },
  'error': { icon: '⚠️', label: 'Error', color: '#ef4444' },
};

const RecentActivityCard: React.FC<RecentActivityCardProps> = ({
  events,
  onEventClick,
  maxEvents = 8,
}) => {
  const [newEventIds, setNewEventIds] = useState<Set<string>>(new Set());
  const prevEventsRef = useRef<string[]>([]);

  useEffect(() => {
    const currentIds = events.map((e, i) => e.id || `event-${i}-${e.timestamp}`);
    const newIds = currentIds.filter(id => !prevEventsRef.current.includes(id));

    if (newIds.length > 0 && prevEventsRef.current.length > 0) {
      setNewEventIds(prev => new Set([...prev, ...newIds]));
      setTimeout(() => {
        setNewEventIds(prev => {
          const next = new Set(prev);
          newIds.forEach(id => next.delete(id));
          return next;
        });
      }, 2000);
    }

    prevEventsRef.current = currentIds;
  }, [events]);

  const formatTime = (timestamp: string): string => {
    const date = new Date(timestamp);
    const now = new Date();
    const diff = now.getTime() - date.getTime();

    if (diff < 60000) return 'Just now';
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
    return date.toLocaleDateString();
  };

  const getEventConfig = (eventType: string) => {
    const normalizedType = eventType.toLowerCase().replace(/[^a-z_]/g, '_');

    for (const [key, config] of Object.entries(EVENT_TYPE_CONFIG)) {
      if (normalizedType.includes(key) || key.includes(normalizedType)) {
        return config;
      }
    }

    return { icon: '📋', label: eventType.replace(/_/g, ' '), color: '#6b7280' };
  };

  const getEventSummary = (event: ActivityEvent): string => {
    const payload = event.payload || {};

    if (payload.token_symbol) return String(payload.token_symbol);
    if (payload.symbol) return String(payload.symbol);
    if (payload.mint) return String(payload.mint).slice(0, 8) + '...';
    if (payload.token_mint) return String(payload.token_mint).slice(0, 8) + '...';
    if (payload.edge_id) return `Edge: ${String(payload.edge_id).slice(0, 6)}`;
    if (payload.position_id) return `Pos: ${String(payload.position_id).slice(0, 6)}`;
    if (payload.message) return String(payload.message).slice(0, 30);

    return '';
  };

  const displayEvents = events.slice(0, maxEvents);

  return (
    <div className={styles.activityCard}>
      <div className={styles.activityCardHeader}>
        <h3 className={styles.activityCardTitle}>
          {events.length > 0 && newEventIds.size > 0 && (
            <span className={styles.liveIndicator} />
          )}
          Recent Activity
        </h3>
        {events.length > 0 && (
          <span className={styles.activityBadge}>{events.length} events</span>
        )}
      </div>

      <div className={styles.activityCardContent}>
        {events.length === 0 ? (
          <div className={styles.activityEmptyState}>
            <span className={styles.activityEmptyIcon}>📡</span>
            <span>No recent activity</span>
            <span className={styles.activityEmptyHint}>Events will stream here in real-time</span>
          </div>
        ) : (
          <div className={styles.activityStream}>
            {displayEvents.map((event, idx) => {
              const eventId = event.id || `event-${idx}-${event.timestamp}`;
              const config = getEventConfig(event.event_type);
              const summary = getEventSummary(event);
              const isNew = newEventIds.has(eventId);

              return (
                <div
                  key={eventId}
                  className={`${styles.activityStreamItem} ${isNew ? styles.newItem : ''}`}
                  onClick={() => onEventClick?.(event)}
                  style={{ '--event-color': config.color } as React.CSSProperties}
                >
                  <div className={styles.streamItemIcon}>
                    <span>{config.icon}</span>
                  </div>
                  <div className={styles.streamItemContent}>
                    <div className={styles.streamItemHeader}>
                      <span className={styles.streamItemLabel}>{config.label}</span>
                      <span className={styles.streamItemTime}>{formatTime(event.timestamp)}</span>
                    </div>
                    {summary && (
                      <span className={styles.streamItemSummary}>{summary}</span>
                    )}
                    {event.source && (
                      <span className={styles.streamItemSource}>
                        {event.source.type}: {event.source.id.slice(0, 8)}
                      </span>
                    )}
                  </div>
                  <div className={styles.streamItemArrow}>→</div>
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
};

export default RecentActivityCard;
