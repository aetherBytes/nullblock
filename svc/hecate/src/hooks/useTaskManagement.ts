import { useState, useEffect, useCallback, useRef } from 'react';
import {
  Task,
  TaskCreationRequest,
  TaskUpdateRequest,
  TaskFilter,
  TaskStats,
  TaskQueue,
  TaskTemplate,
  TaskNotification,
  TaskEvent,
  MotivationState,
  TaskStatus,
  TaskPriority
} from '../types/tasks';
import { taskService } from '../common/services/task-service';

interface UseTaskManagementReturn {
  // State
  tasks: Task[];
  filteredTasks: Task[];
  activeTask: Task | null;
  queues: TaskQueue[];
  templates: TaskTemplate[];
  notifications: TaskNotification[];
  stats: TaskStats | null;
  motivationState: MotivationState | null;
  isLoading: boolean;
  error: string | null;

  // Filters & Search
  filter: TaskFilter;
  setFilter: (filter: TaskFilter) => void;
  searchTerm: string;
  setSearchTerm: (term: string) => void;

  // Task Operations
  createTask: (request: TaskCreationRequest) => Promise<boolean>;
  updateTask: (request: TaskUpdateRequest) => Promise<boolean>;
  deleteTask: (id: string) => Promise<boolean>;
  startTask: (id: string) => Promise<boolean>;
  pauseTask: (id: string) => Promise<boolean>;
  resumeTask: (id: string) => Promise<boolean>;
  cancelTask: (id: string) => Promise<boolean>;
  retryTask: (id: string) => Promise<boolean>;

  // Task Selection
  selectTask: (id: string | null) => void;
  getTask: (id: string) => Task | undefined;

  // Batch Operations
  startMultipleTasks: (ids: string[]) => Promise<boolean>;
  deleteMultipleTasks: (ids: string[]) => Promise<boolean>;
  updateTaskPriority: (id: string, priority: TaskPriority) => Promise<boolean>;

  // Templates
  createFromTemplate: (templateId: string, parameters: Record<string, any>) => Promise<boolean>;

  // Notifications
  markNotificationRead: (id: string) => Promise<boolean>;
  handleNotificationAction: (id: string, action: string) => Promise<boolean>;
  unreadNotificationCount: number;

  // Analytics
  refreshStats: () => Promise<void>;
  getTasksByStatus: (status: TaskStatus) => Task[];
  getTasksByPriority: (priority: TaskPriority) => Task[];

  // Motivation System
  suggestTasks: (context?: Record<string, any>) => Promise<TaskCreationRequest[]>;
  updateMotivation: (updates: Partial<MotivationState>) => Promise<boolean>;
  learnFromTask: (taskId: string, feedback: Record<string, any>) => Promise<boolean>;

  // Real-time
  subscribeToUpdates: boolean;
  setSubscribeToUpdates: (subscribe: boolean) => void;

  // Utility
  refresh: () => Promise<void>;
  clearError: () => void;
}

export const useTaskManagement = (
  walletPublicKey?: string | null,
  initialFilter: TaskFilter = {},
  autoSubscribe: boolean = true
): UseTaskManagementReturn => {
  // Helper function to ensure we always have a valid array
  const ensureArray = (value: any): Task[] => {
    if (Array.isArray(value)) return value;
    return [];
  };

  const ensureNotificationArray = (value: any): TaskNotification[] => {
    if (Array.isArray(value)) return value;
    return [];
  };

  // Core state
  const [tasks, setTasks] = useState<Task[]>([]);
  const [activeTask, setActiveTask] = useState<Task | null>(null);
  const [queues, setQueues] = useState<TaskQueue[]>([]);
  const [templates, setTemplates] = useState<TaskTemplate[]>([]);
  const [notifications, setNotifications] = useState<TaskNotification[]>([]);
  const [stats, setStats] = useState<TaskStats | null>(null);
  const [motivationState, setMotivationState] = useState<MotivationState | null>(null);

  // UI state
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [filter, setFilter] = useState<TaskFilter>(initialFilter);
  const [searchTerm, setSearchTerm] = useState('');
  const [subscribeToUpdates, setSubscribeToUpdates] = useState(autoSubscribe);

  // Refs for cleanup
  const unsubscribeRef = useRef<(() => void) | null>(null);
  const isConnectedRef = useRef(false);

  // Computed state
  const filteredTasks = Array.isArray(tasks) ? tasks.filter(task => {
    if (filter.status && !filter.status.includes(task.status)) return false;
    if (filter.type && !filter.type.includes(task.type)) return false;
    if (filter.category && !filter.category.includes(task.category)) return false;
    if (filter.priority && !filter.priority.includes(task.priority)) return false;
    if (filter.assignedAgent && task.assignedAgent !== filter.assignedAgent) return false;
    if (filter.dateRange) {
      const taskDate = new Date(task.createdAt);
      if (taskDate < filter.dateRange.start || taskDate > filter.dateRange.end) return false;
    }
    if (searchTerm) {
      const searchLower = searchTerm.toLowerCase();
      if (
        !task.name.toLowerCase().includes(searchLower) &&
        !task.description.toLowerCase().includes(searchLower) &&
        !task.type.toLowerCase().includes(searchLower)
      ) {
        return false;
      }
    }
    return true;
  }) : [];

  const unreadNotificationCount = Array.isArray(notifications) ? notifications.filter(n => !n.read).length : 0;

  // Initialize connection
  useEffect(() => {
    const initializeConnection = async () => {
      setIsLoading(true);
      try {
        console.log('🔗 Attempting to connect to task service...');
        const connected = await taskService.connect();
        isConnectedRef.current = connected;
        console.log('🔗 Task service connection:', connected ? 'SUCCESS' : 'FAILED');

        if (connected) {
          console.log('📋 Loading task data...');
          await Promise.all([
            loadTasks(),
            loadQueues(),
            loadTemplates(),
            loadNotifications(),
            loadStats(),
            loadMotivationState()
          ]);
        } else {
          console.log('⚠️ Task service unavailable - no tasks will be loaded');
        }
      } catch (err) {
        console.error('❌ Task management initialization error:', err);
        setError((err as Error).message);
      } finally {
        setIsLoading(false);
      }
    };

    if (walletPublicKey) {
      initializeConnection();
    }

    return () => {
      if (unsubscribeRef.current) {
        unsubscribeRef.current();
      }
    };
  }, [walletPublicKey]);


  // Real-time updates subscription
  useEffect(() => {
    if (!walletPublicKey || !subscribeToUpdates || !isConnectedRef.current) {
      return;
    }

    const setupSubscription = async () => {
      try {
        const unsubscribe = await taskService.subscribeToUpdates(
          (updatedTask: Task) => {
            setTasks(prev => {
              const currentTasks = ensureArray(prev);
              const index = currentTasks.findIndex(t => t.id === updatedTask.id);
              if (index >= 0) {
                const newTasks = [...currentTasks];
                newTasks[index] = updatedTask;
                return newTasks;
              } else {
                return [updatedTask, ...currentTasks];
              }
            });

            if (activeTask?.id === updatedTask.id) {
              setActiveTask(updatedTask);
            }
          },
          filter
        );

        unsubscribeRef.current = unsubscribe;
      } catch (err) {
        console.error('Failed to setup task updates subscription:', err);
      }
    };

    setupSubscription();

    return () => {
      if (unsubscribeRef.current) {
        unsubscribeRef.current();
        unsubscribeRef.current = null;
      }
    };
  }, [walletPublicKey, subscribeToUpdates, filter, activeTask?.id]);

  // Data loading functions
  const loadTasks = useCallback(async () => {
    const response = await taskService.getTasks(filter);
    if (response.success && response.data) {
      setTasks(response.data);
    } else {
      setError(response.error || 'Failed to load tasks');
    }
  }, [filter]);

  const loadQueues = useCallback(async () => {
    const response = await taskService.getQueues();
    if (response.success && response.data) {
      setQueues(response.data);
    }
  }, []);

  const loadTemplates = useCallback(async () => {
    const response = await taskService.getTemplates();
    if (response.success && response.data) {
      setTemplates(response.data);
    }
  }, []);

  const loadNotifications = useCallback(async () => {
    const response = await taskService.getNotifications();
    if (response.success && response.data) {
      setNotifications(response.data);
    }
  }, []);

  const loadStats = useCallback(async () => {
    const response = await taskService.getStats(filter);
    if (response.success && response.data) {
      setStats(response.data);
    }
  }, [filter]);

  const loadMotivationState = useCallback(async () => {
    const response = await taskService.getMotivationState();
    if (response.success && response.data) {
      setMotivationState(response.data);
    }
  }, []);

  // Task operations
  const createTask = useCallback(async (request: TaskCreationRequest): Promise<boolean> => {
    console.log('📋 Creating task:', request);
    setError(null);

    if (!isConnectedRef.current) {
      setError('Task service is not available');
      return false;
    }

    try {
      setIsLoading(true);
      const response = await taskService.createTask(request);
      if (response.success && response.data) {
        setTasks(prev => [response.data!, ...ensureArray(prev)]);
        console.log('✅ Task created via backend:', response.data);
        return true;
      } else {
        setError(response.error || 'Failed to create task');
        return false;
      }
    } catch (error) {
      console.error('❌ Task creation error:', error);
      setError((error as Error).message);
      return false;
    } finally {
      setIsLoading(false);
    }
  }, []);

  const updateTask = useCallback(async (request: TaskUpdateRequest): Promise<boolean> => {
    console.log('📝 Updating task:', request);

    if (!isConnectedRef.current) {
      setError('Task service is not available');
      return false;
    }

    try {
      const response = await taskService.updateTask(request);
      if (response.success && response.data) {
        setTasks(prev => ensureArray(prev).map(t => t.id === request.id ? response.data! : t));
        if (activeTask?.id === request.id) {
          setActiveTask(response.data);
        }
        console.log('✅ Task updated via backend');
        return true;
      } else {
        setError(response.error || 'Failed to update task');
        return false;
      }
    } catch (error) {
      console.error('❌ Task update error:', error);
      setError((error as Error).message);
      return false;
    }
  }, [activeTask?.id]);

  const deleteTask = useCallback(async (id: string): Promise<boolean> => {
    console.log('🗑️ Deleting task:', id);

    if (!isConnectedRef.current) {
      setError('Task service is not available');
      return false;
    }

    try {
      const response = await taskService.deleteTask(id);
      if (response.success) {
        setTasks(prev => ensureArray(prev).filter(t => t.id !== id));
        if (activeTask?.id === id) {
          setActiveTask(null);
        }
        console.log('✅ Task deleted via backend');
        return true;
      } else {
        setError(response.error || 'Failed to delete task');
        return false;
      }
    } catch (error) {
      console.error('❌ Task deletion error:', error);
      setError((error as Error).message);
      return false;
    }
  }, [activeTask?.id]);

  const startTask = useCallback(async (id: string): Promise<boolean> => {
    console.log('▶️ Starting task:', id);
    return updateTask({
      id,
      status: 'running',
      startedAt: new Date(),
      progress: 0
    });
  }, [updateTask]);

  const pauseTask = useCallback(async (id: string): Promise<boolean> => {
    console.log('⏸️ Pausing task:', id);
    return updateTask({
      id,
      status: 'paused'
    });
  }, [updateTask]);

  const resumeTask = useCallback(async (id: string): Promise<boolean> => {
    console.log('▶️ Resuming task:', id);
    return updateTask({
      id,
      status: 'running'
    });
  }, [updateTask]);

  const cancelTask = useCallback(async (id: string): Promise<boolean> => {
    console.log('🚫 Cancelling task:', id);
    return updateTask({
      id,
      status: 'cancelled'
    });
  }, [updateTask]);

  const retryTask = useCallback(async (id: string): Promise<boolean> => {
    console.log('🔄 Retrying task:', id);
    return updateTask({
      id,
      status: 'running',
      currentRetries: 0,
      progress: 0,
      startedAt: new Date()
    });
  }, [updateTask]);

  // Batch operations
  const startMultipleTasks = useCallback(async (ids: string[]): Promise<boolean> => {
    const results = await Promise.all(ids.map(id => startTask(id)));
    return results.every(success => success);
  }, [startTask]);

  const deleteMultipleTasks = useCallback(async (ids: string[]): Promise<boolean> => {
    const results = await Promise.all(ids.map(id => deleteTask(id)));
    return results.every(success => success);
  }, [deleteTask]);

  const updateTaskPriority = useCallback(async (id: string, priority: TaskPriority): Promise<boolean> => {
    return updateTask({ id, priority });
  }, [updateTask]);

  // Template operations
  const createFromTemplate = useCallback(async (
    templateId: string,
    parameters: Record<string, any>
  ): Promise<boolean> => {
    setIsLoading(true);
    try {
      const response = await taskService.createFromTemplate(templateId, parameters);
      if (response.success && response.data) {
        setTasks(prev => [response.data!, ...ensureArray(prev)]);
        return true;
      } else {
        setError(response.error || 'Failed to create task from template');
        return false;
      }
    } finally {
      setIsLoading(false);
    }
  }, []);

  // Notification operations
  const markNotificationRead = useCallback(async (id: string): Promise<boolean> => {
    const response = await taskService.markNotificationRead(id);
    if (response.success) {
      setNotifications(prev => ensureNotificationArray(prev).map(n => n.id === id ? { ...n, read: true } : n));
      return true;
    }
    return false;
  }, []);

  const handleNotificationAction = useCallback(async (id: string, action: string): Promise<boolean> => {
    const response = await taskService.handleNotificationAction(id, action);
    if (response.success) {
      await loadNotifications();
      return true;
    }
    return false;
  }, [loadNotifications]);

  // Analytics & queries
  const getTasksByStatus = useCallback((status: TaskStatus): Task[] => {
    return Array.isArray(tasks) ? tasks.filter(task => task.status === status) : [];
  }, [tasks]);

  const getTasksByPriority = useCallback((priority: TaskPriority): Task[] => {
    return Array.isArray(tasks) ? tasks.filter(task => task.priority === priority) : [];
  }, [tasks]);

  const getTask = useCallback((id: string): Task | undefined => {
    return Array.isArray(tasks) ? tasks.find(task => task.id === id) : undefined;
  }, [tasks]);

  const selectTask = useCallback((id: string | null) => {
    if (id) {
      const task = getTask(id);
      setActiveTask(task || null);
    } else {
      setActiveTask(null);
    }
  }, [getTask]);

  // Motivation system
  const suggestTasks = useCallback(async (context?: Record<string, any>): Promise<TaskCreationRequest[]> => {
    const response = await taskService.suggestTasks(context || {});
    return response.success && response.data ? response.data : [];
  }, []);

  const updateMotivation = useCallback(async (updates: Partial<MotivationState>): Promise<boolean> => {
    const response = await taskService.updateMotivationState(updates);
    if (response.success && response.data) {
      setMotivationState(response.data);
      return true;
    }
    return false;
  }, []);

  const learnFromTask = useCallback(async (taskId: string, feedback: Record<string, any>): Promise<boolean> => {
    const response = await taskService.learnFromOutcome(taskId, feedback);
    return response.success;
  }, []);

  // Utility functions
  const refresh = useCallback(async () => {
    setIsLoading(true);
    try {
      await Promise.all([
        loadTasks(),
        loadQueues(),
        loadTemplates(),
        loadNotifications(),
        loadStats(),
        loadMotivationState()
      ]);
    } catch (err) {
      setError((err as Error).message);
    } finally {
      setIsLoading(false);
    }
  }, [loadTasks, loadQueues, loadTemplates, loadNotifications, loadStats, loadMotivationState]);

  const refreshStats = useCallback(async () => {
    await loadStats();
  }, [loadStats]);

  const clearError = useCallback(() => {
    setError(null);
  }, []);

  return {
    // State
    tasks,
    filteredTasks,
    activeTask,
    queues,
    templates,
    notifications,
    stats,
    motivationState,
    isLoading,
    error,

    // Filters & Search
    filter,
    setFilter,
    searchTerm,
    setSearchTerm,

    // Task Operations
    createTask,
    updateTask,
    deleteTask,
    startTask,
    pauseTask,
    resumeTask,
    cancelTask,
    retryTask,

    // Task Selection
    selectTask,
    getTask,

    // Batch Operations
    startMultipleTasks,
    deleteMultipleTasks,
    updateTaskPriority,

    // Templates
    createFromTemplate,

    // Notifications
    markNotificationRead,
    handleNotificationAction,
    unreadNotificationCount,

    // Analytics
    refreshStats,
    getTasksByStatus,
    getTasksByPriority,

    // Motivation System
    suggestTasks,
    updateMotivation,
    learnFromTask,

    // Real-time
    subscribeToUpdates,
    setSubscribeToUpdates,

    // Utility
    refresh,
    clearError,
  };
};