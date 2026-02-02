import { useState, useEffect, useCallback, useRef } from 'react';
import type { LogEntry, LogLevel, LogCategory, LogsQuery } from '../types/logs';

const EREBUS_API_URL = import.meta.env.VITE_EREBUS_API_URL || 'http://localhost:3000';

interface UseLogsOptions {
  autoConnect?: boolean;
  maxLogs?: number;
  filters?: LogsQuery;
}

export const useLogs = (options: UseLogsOptions = {}) => {
  const { autoConnect = true, maxLogs = 500, filters } = options;

  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [isConnected, setIsConnected] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const eventSourceRef = useRef<EventSource | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const reconnectAttemptsRef = useRef(0);
  const maxReconnectAttempts = 5;

  const fetchRecentLogs = useCallback(async () => {
    setIsLoading(true);
    setError(null);

    try {
      const params = new URLSearchParams();

      if (filters?.limit) {
        params.append('limit', filters.limit.toString());
      }

      if (filters?.category) {
        params.append('category', filters.category);
      }

      if (filters?.level) {
        params.append('level', filters.level);
      }

      const url = `${EREBUS_API_URL}/api/logs/recent?${params.toString()}`;
      const response = await fetch(url);

      if (!response.ok) {
        throw new Error(`Failed to fetch logs: ${response.statusText}`);
      }

      const data = await response.json();

      if (data.success && Array.isArray(data.data)) {
        setLogs(data.data);
      } else {
        throw new Error('Invalid response format');
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to fetch logs';

      setError(errorMessage);
      console.error('❌ Error fetching logs:', err);
    } finally {
      setIsLoading(false);
    }
  }, [filters]);

  const connectToStream = useCallback(() => {
    if (eventSourceRef.current) {
      console.warn('⚠️ Already connected to log stream');

      return;
    }

    console.log('📡 Connecting to log stream...');

    const url = `${EREBUS_API_URL}/api/logs/stream`;
    const eventSource = new EventSource(url);

    eventSource.onopen = () => {
      console.log('✅ Connected to log stream');
      setIsConnected(true);
      setError(null);
      reconnectAttemptsRef.current = 0;
    };

    eventSource.onmessage = (event) => {
      if (event.data === 'keepalive') {
        return;
      }

      try {
        const logEntry: LogEntry = JSON.parse(event.data);

        setLogs((prevLogs) => {
          const newLogs = [...prevLogs, logEntry];

          if (newLogs.length > maxLogs) {
            return newLogs.slice(newLogs.length - maxLogs);
          }

          return newLogs;
        });
      } catch (err) {
        console.error('❌ Failed to parse log entry:', err);
      }
    };

    eventSource.onerror = () => {
      setIsConnected(false);

      if (eventSource.readyState === EventSource.CLOSED) {
        if (reconnectAttemptsRef.current < maxReconnectAttempts) {
          const delay = Math.min(1000 * Math.pow(2, reconnectAttemptsRef.current), 30000);

          reconnectAttemptsRef.current += 1;

          reconnectTimeoutRef.current = setTimeout(() => {
            if (eventSourceRef.current) {
              eventSourceRef.current.close();
              eventSourceRef.current = null;
            }

            connectToStream();
          }, delay);
        } else {
          setError('Log stream unavailable');
          eventSource.close();
          eventSourceRef.current = null;
        }
      }
    };

    eventSourceRef.current = eventSource;
  }, [maxLogs]);

  const disconnectFromStream = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }

    if (eventSourceRef.current) {
      console.log('📡 Disconnecting from log stream...');
      eventSourceRef.current.close();
      eventSourceRef.current = null;
      setIsConnected(false);
    }
  }, []);

  const clearLogs = useCallback(() => {
    setLogs([]);
  }, []);

  const filterLogs = useCallback(
    (predicate: (log: LogEntry) => boolean): LogEntry[] => logs.filter(predicate),
    [logs],
  );

  const getLogsByLevel = useCallback(
    (level: LogLevel): LogEntry[] => filterLogs((log) => log.level === level),
    [filterLogs],
  );

  const getLogsByCategory = useCallback(
    (category: LogCategory): LogEntry[] => filterLogs((log) => log.category === category),
    [filterLogs],
  );

  const searchLogs = useCallback(
    (searchTerm: string): LogEntry[] => {
      const term = searchTerm.toLowerCase();

      return filterLogs(
        (log) =>
          log.message.toLowerCase().includes(term) ||
          log.source.toLowerCase().includes(term) ||
          log.category.toLowerCase().includes(term),
      );
    },
    [filterLogs],
  );

  useEffect(() => {
    fetchRecentLogs();

    if (autoConnect) {
      connectToStream();
    }

    return () => {
      disconnectFromStream();
    };
  }, []);

  return {
    logs,
    isConnected,
    isLoading,
    error,
    fetchRecentLogs,
    connectToStream,
    disconnectFromStream,
    clearLogs,
    filterLogs,
    getLogsByLevel,
    getLogsByCategory,
    searchLogs,
  };
};
