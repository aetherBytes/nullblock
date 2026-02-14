import { useState, useEffect, useCallback } from 'react';
import type { EventSubscription, EventRule } from '../common/services/event-system';
import { eventSystem } from '../common/services/event-system';
import type { TaskEvent, EventType, TaskCreationRequest } from '../types/tasks';

interface UseEventSystemReturn {
  // Event publishing
  publishEvent: (event: Omit<TaskEvent, 'id'>) => Promise<void>;
  publishPriceChange: (symbol: string, price: string, change: string) => void;
  publishMarketOpportunity: (type: string, data: any) => void;
  publishUserInteraction: (action: string, context: any) => void;
  publishAgentCompletion: (agentName: string, taskId: string, result: string, data?: any) => void;
  publishSystemAlert: (level: string, message: string, component: string, data?: any) => void;
  publishThresholdBreach: (metric: string, value: number, threshold: number, data?: any) => void;

  // Event subscription
  subscribe: (
    eventType: EventType,
    callback: (event: TaskEvent) => void,
    filter?: (event: TaskEvent) => boolean,
  ) => string;
  unsubscribe: (subscriptionId: string) => boolean;

  // Event data
  recentEvents: TaskEvent[];
  rules: EventRule[];
  subscriptions: EventSubscription[];

  // Automation rules
  addRule: (rule: EventRule) => void;
  removeRule: (ruleId: string) => boolean;

  // Performance controls
  setPerformanceMode: (enabled: boolean) => void;
  isPerformanceMode: boolean;

  // Integration with task management
  onTaskCreated?: (task: TaskCreationRequest) => void;
}

export const useEventSystem = (
  enableAutoRefresh: boolean = true,
  refreshInterval: number = 5000,
): UseEventSystemReturn => {
  const [recentEvents, setRecentEvents] = useState<TaskEvent[]>([]);
  const [rules, setRules] = useState<EventRule[]>([]);
  const [subscriptions, setSubscriptions] = useState<EventSubscription[]>([]);
  const [isPerformanceMode, setIsPerformanceMode] = useState(false);

  // Refresh data from event system
  const refreshData = useCallback(() => {
    setRecentEvents(eventSystem.getRecentEvents(20));
    setRules(eventSystem.getRules());
    setSubscriptions(eventSystem.getSubscriptions());
  }, []);

  const currentInterval = isPerformanceMode ? 10000 : refreshInterval;

  // Auto-refresh effect
  useEffect(() => {
    refreshData();

    if (enableAutoRefresh) {
      const interval = setInterval(refreshData, currentInterval);

      return () => clearInterval(interval);
    }
  }, [enableAutoRefresh, currentInterval, refreshData]);

  // Event publishing functions
  const publishEvent = useCallback(
    async (event: Omit<TaskEvent, 'id'>) => {
      await eventSystem.publishEvent(event);
      refreshData();
    },
    [refreshData],
  );

  const publishPriceChange = useCallback(
    (symbol: string, price: string, change: string) => {
      eventSystem.publishPriceChange(symbol, price, change);
      refreshData();
    },
    [refreshData],
  );

  const publishMarketOpportunity = useCallback(
    (type: string, data: any) => {
      eventSystem.publishMarketOpportunity(type, data);
      refreshData();
    },
    [refreshData],
  );

  const publishUserInteraction = useCallback(
    (action: string, context: any) => {
      eventSystem.publishUserInteraction(action, context);
      refreshData();
    },
    [refreshData],
  );

  const publishAgentCompletion = useCallback(
    (agentName: string, taskId: string, result: string, data?: any) => {
      eventSystem.publishAgentCompletion(agentName, taskId, result, data);
      refreshData();
    },
    [refreshData],
  );

  const publishSystemAlert = useCallback(
    (level: string, message: string, component: string, data?: any) => {
      eventSystem.publishSystemAlert(level, message, component, data);
      refreshData();
    },
    [refreshData],
  );

  const publishThresholdBreach = useCallback(
    (metric: string, value: number, threshold: number, data?: any) => {
      eventSystem.publishThresholdBreach(metric, value, threshold, data);
      refreshData();
    },
    [refreshData],
  );

  // Event subscription functions
  const subscribe = useCallback(
    (
      eventType: EventType,
      callback: (event: TaskEvent) => void,
      filter?: (event: TaskEvent) => boolean,
    ): string => {
      const subscriptionId = eventSystem.subscribe({
        eventType,
        callback,
        filter,
      });

      refreshData();

      return subscriptionId;
    },
    [refreshData],
  );

  const unsubscribe = useCallback(
    (subscriptionId: string): boolean => {
      const result = eventSystem.unsubscribe(subscriptionId);

      refreshData();

      return result;
    },
    [refreshData],
  );

  // Automation rule functions
  const addRule = useCallback(
    (rule: EventRule) => {
      eventSystem.addRule(rule);
      refreshData();
    },
    [refreshData],
  );

  const removeRule = useCallback(
    (ruleId: string): boolean => {
      const result = eventSystem.removeRule(ruleId);

      refreshData();

      return result;
    },
    [refreshData],
  );

  const setPerformanceMode = useCallback((enabled: boolean) => {
    setIsPerformanceMode(enabled);
  }, []);

  return {
    // Event publishing
    publishEvent,
    publishPriceChange,
    publishMarketOpportunity,
    publishUserInteraction,
    publishAgentCompletion,
    publishSystemAlert,
    publishThresholdBreach,

    // Event subscription
    subscribe,
    unsubscribe,

    // Event data
    recentEvents,
    rules,
    subscriptions,

    // Automation rules
    addRule,
    removeRule,

    // Performance controls
    setPerformanceMode,
    isPerformanceMode,
  };
};
