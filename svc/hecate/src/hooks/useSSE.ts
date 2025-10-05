import { useEffect, useRef, useCallback } from 'react';
import { TaskLifecycleEvent } from '../types/tasks';

interface UseSSEOptions {
  onTaskUpdate?: (event: TaskLifecycleEvent) => void;
  onMessageUpdate?: (event: any) => void;
  onError?: (error: Error) => void;
  onConnect?: () => void;
  onDisconnect?: () => void;
  autoReconnect?: boolean;
  reconnectInterval?: number;
}

interface UseSSEReturn {
  subscribeToTask: (taskId: string) => void;
  unsubscribeFromTask: (taskId: string) => void;
  subscribeToMessages: () => void;
  unsubscribeFromMessages: () => void;
  isConnected: boolean;
}

export const useSSE = (options: UseSSEOptions = {}): UseSSEReturn => {
  const {
    onTaskUpdate,
    onMessageUpdate,
    onError,
    onConnect,
    onDisconnect,
    autoReconnect = true,
    reconnectInterval = 5000,
  } = options;

  const taskSubscriptions = useRef<Map<string, EventSource>>(new Map());
  const messageSubscription = useRef<EventSource | null>(null);
  const isConnectedRef = useRef(false);
  const reconnectTimeouts = useRef<Map<string, NodeJS.Timeout>>(new Map());

  const erebusUrl = import.meta.env.VITE_EREBUS_API_URL || 'http://localhost:3000';

  const createTaskSubscription = useCallback((taskId: string) => {
    if (taskSubscriptions.current.has(taskId)) {
      console.log(`🔌 Already subscribed to task ${taskId}`);
      return;
    }

    const url = `${erebusUrl}/a2a/tasks/${taskId}/sse`;
    console.log(`🔌 Subscribing to task SSE: ${url}`);

    try {
      const eventSource = new EventSource(url);

      eventSource.onopen = () => {
        console.log(`✅ SSE connection opened for task ${taskId}`);
        isConnectedRef.current = true;
        if (onConnect) {
          onConnect();
        }
      };

      eventSource.onmessage = (event) => {
        try {
          if (event.data && event.data !== 'keep-alive') {
            const parsedEvent = JSON.parse(event.data) as TaskLifecycleEvent;
            console.log(`📨 Task SSE event for ${taskId}:`, parsedEvent);

            if (onTaskUpdate) {
              onTaskUpdate(parsedEvent);
            }
          }
        } catch (error) {
          console.error(`❌ Failed to parse SSE event for task ${taskId}:`, error);
          if (onError) {
            onError(error as Error);
          }
        }
      };

      eventSource.onerror = (error) => {
        console.error(`❌ SSE error for task ${taskId}:`, error);
        eventSource.close();
        taskSubscriptions.current.delete(taskId);
        isConnectedRef.current = taskSubscriptions.current.size > 0 || messageSubscription.current !== null;

        if (onDisconnect) {
          onDisconnect();
        }

        if (onError) {
          onError(new Error(`SSE connection error for task ${taskId}`));
        }

        if (autoReconnect) {
          const timeout = setTimeout(() => {
            console.log(`🔄 Reconnecting to task ${taskId} SSE...`);
            createTaskSubscription(taskId);
          }, reconnectInterval);
          reconnectTimeouts.current.set(taskId, timeout);
        }
      };

      taskSubscriptions.current.set(taskId, eventSource);
    } catch (error) {
      console.error(`❌ Failed to create SSE connection for task ${taskId}:`, error);
      if (onError) {
        onError(error as Error);
      }
    }
  }, [erebusUrl, onTaskUpdate, onError, onConnect, onDisconnect, autoReconnect, reconnectInterval]);

  const destroyTaskSubscription = useCallback((taskId: string) => {
    const eventSource = taskSubscriptions.current.get(taskId);
    if (eventSource) {
      console.log(`🔌 Unsubscribing from task ${taskId}`);
      eventSource.close();
      taskSubscriptions.current.delete(taskId);

      const timeout = reconnectTimeouts.current.get(taskId);
      if (timeout) {
        clearTimeout(timeout);
        reconnectTimeouts.current.delete(taskId);
      }

      isConnectedRef.current = taskSubscriptions.current.size > 0 || messageSubscription.current !== null;

      if (!isConnectedRef.current && onDisconnect) {
        onDisconnect();
      }
    }
  }, [onDisconnect]);

  const createMessageSubscription = useCallback(() => {
    if (messageSubscription.current) {
      console.log('🔌 Already subscribed to messages');
      return;
    }

    const url = `${erebusUrl}/a2a/messages/sse`;
    console.log(`🔌 Subscribing to message SSE: ${url}`);

    try {
      const eventSource = new EventSource(url);

      eventSource.onopen = () => {
        console.log('✅ Message SSE connection opened');
        isConnectedRef.current = true;
        if (onConnect) {
          onConnect();
        }
      };

      eventSource.onmessage = (event) => {
        try {
          if (event.data && event.data !== 'keep-alive') {
            const parsedEvent = JSON.parse(event.data);
            console.log('📨 Message SSE event:', parsedEvent);

            if (onMessageUpdate) {
              onMessageUpdate(parsedEvent);
            }
          }
        } catch (error) {
          console.error('❌ Failed to parse message SSE event:', error);
          if (onError) {
            onError(error as Error);
          }
        }
      };

      eventSource.onerror = (error) => {
        console.error('❌ Message SSE error:', error);
        eventSource.close();
        messageSubscription.current = null;
        isConnectedRef.current = taskSubscriptions.current.size > 0;

        if (onDisconnect) {
          onDisconnect();
        }

        if (onError) {
          onError(new Error('Message SSE connection error'));
        }

        if (autoReconnect) {
          setTimeout(() => {
            console.log('🔄 Reconnecting to message SSE...');
            createMessageSubscription();
          }, reconnectInterval);
        }
      };

      messageSubscription.current = eventSource;
    } catch (error) {
      console.error('❌ Failed to create message SSE connection:', error);
      if (onError) {
        onError(error as Error);
      }
    }
  }, [erebusUrl, onMessageUpdate, onError, onConnect, onDisconnect, autoReconnect, reconnectInterval]);

  const destroyMessageSubscription = useCallback(() => {
    if (messageSubscription.current) {
      console.log('🔌 Unsubscribing from messages');
      messageSubscription.current.close();
      messageSubscription.current = null;
      isConnectedRef.current = taskSubscriptions.current.size > 0;

      if (!isConnectedRef.current && onDisconnect) {
        onDisconnect();
      }
    }
  }, [onDisconnect]);

  useEffect(() => {
    return () => {
      console.log('🧹 Cleaning up all SSE connections');

      taskSubscriptions.current.forEach((eventSource, taskId) => {
        eventSource.close();
        console.log(`🔌 Closed task subscription: ${taskId}`);
      });
      taskSubscriptions.current.clear();

      reconnectTimeouts.current.forEach((timeout) => {
        clearTimeout(timeout);
      });
      reconnectTimeouts.current.clear();

      if (messageSubscription.current) {
        messageSubscription.current.close();
        console.log('🔌 Closed message subscription');
      }
    };
  }, []);

  return {
    subscribeToTask: createTaskSubscription,
    unsubscribeFromTask: destroyTaskSubscription,
    subscribeToMessages: createMessageSubscription,
    unsubscribeFromMessages: destroyMessageSubscription,
    isConnected: isConnectedRef.current,
  };
};
