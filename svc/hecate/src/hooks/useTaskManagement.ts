import { useState, useEffect, useCallback, useRef } from 'react';
import {
  Task,
  TaskCreationRequest,
  TaskUpdateRequest,
  TaskFilter,
  TaskStatus,
  TaskPriority,
  TaskLifecycleEvent
} from '../types/tasks';
import { taskService } from '../common/services/task-service';
import { useSSE } from './useSSE';

interface UseTaskManagementReturn {
  // State
  tasks: Task[];
  filteredTasks: Task[];
  activeTask: Task | null;
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
  processTask: (id: string, isAutoProcessing?: boolean) => Promise<boolean>;

  // Task Selection
  selectTask: (id: string | null) => void;
  getTask: (id: string) => Task | undefined;

  // Analytics
  getTasksByStatus: (status: TaskStatus) => Task[];
  getTasksByPriority: (priority: TaskPriority) => Task[];

  // Utility
  refresh: () => Promise<void>;
  clearError: () => void;
}

export const useTaskManagement = (
  walletPublicKey?: string | null,
  initialFilter: TaskFilter = {},
  autoSubscribe: boolean = true,
  addChatNotification?: (taskId: string, taskName: string, message: string, processingTime?: number) => void
): UseTaskManagementReturn => {
  // Helper function to ensure we always have a valid array
  const ensureArray = (value: any): Task[] => {
    if (Array.isArray(value)) return value;
    return [];
  };

  // Core state
  const [tasks, setTasks] = useState<Task[]>([]);
  const [activeTask, setActiveTask] = useState<Task | null>(null);

  // UI state
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [filter, setFilter] = useState<TaskFilter>(initialFilter);
  const [searchTerm, setSearchTerm] = useState('');

  // Refs for cleanup
  const isConnectedRef = useRef(false);
  const activeTaskIdsRef = useRef<Set<string>>(new Set());

  // Computed state
  const filteredTasks = Array.isArray(tasks) ? tasks.filter(task => {
    if (filter.status && !filter.status.includes(task.status.state)) return false;
    if (filter.type && !filter.type.includes(task.task_type)) return false;
    if (filter.category && !filter.category.includes(task.category)) return false;
    if (filter.priority && !filter.priority.includes(task.priority)) return false;
    if (filter.assigned_agent && task.assigned_agent !== filter.assigned_agent) return false;
    if (filter.date_range) {
      const taskDate = new Date(task.created_at);
      if (taskDate < filter.date_range.start || taskDate > filter.date_range.end) return false;
    }
    if (searchTerm) {
      const searchLower = searchTerm.toLowerCase();
      if (
        !task.name.toLowerCase().includes(searchLower) &&
        !task.description.toLowerCase().includes(searchLower) &&
        !task.task_type.toLowerCase().includes(searchLower)
      ) {
        return false;
      }
    }
    return true;
  }) : [];

  // Data loading functions - defined early to avoid hoisting issues
  const loadTasks = useCallback(async () => {
    const response = await taskService.getTasks(filter);
    if (response.success && response.data) {
      setTasks(response.data);
    } else {
      setError(response.error || 'Failed to load tasks');
    }
  }, [filter]);

  // SSE integration for real-time task updates
  const handleTaskUpdate = useCallback((event: TaskLifecycleEvent) => {
    setTasks(prevTasks => {
      const taskIndex = prevTasks.findIndex(t => t.id === event.task_id);
      if (taskIndex >= 0) {
        const updatedTasks = [...prevTasks];
        updatedTasks[taskIndex] = {
          ...updatedTasks[taskIndex],
          status: {
            state: event.state,
            message: event.message,
            timestamp: event.timestamp
          },
          updated_at: new Date(event.timestamp)
        };
        return updatedTasks;
      }
      return prevTasks;
    });

    if (activeTask?.id === event.task_id) {
      setActiveTask(prev => prev ? {
        ...prev,
        status: {
          state: event.state,
          message: event.message,
          timestamp: event.timestamp
        },
        updated_at: new Date(event.timestamp)
      } : null);
    }

    if (event.state === 'completed' || event.state === 'failed' || event.state === 'canceled') {
      if (addChatNotification) {
        const task = tasks.find(t => t.id === event.task_id);
        const taskName = task?.name || 'Task';
        const message = event.state === 'completed'
          ? `Task "${taskName}" completed successfully!`
          : `Task "${taskName}" ${event.state}: ${event.message || 'No details'}`;
        addChatNotification(event.task_id, taskName, message);
      }
    }
  }, [activeTask, tasks, addChatNotification]);

  const sseHook = useSSE({
    onTaskUpdate: handleTaskUpdate,
    onError: (error) => {
      setError(error.message);
    },
    autoReconnect: true,
    reconnectInterval: 5000,
  });

  // Fallback polling for active tasks (until SSE is fully working)
  useEffect(() => {
    // Skip if not connected
    if (!isConnectedRef.current) return;

    const activeTasks = Array.isArray(tasks) ? tasks.filter(task =>
      task.status.state === 'submitted' ||
      task.status.state === 'working' ||
      task.status.state === 'input-required'
    ) : [];

    // Only poll when there are active tasks
    if (activeTasks.length > 0) {
      const pollInterval = setInterval(async () => {
        try {
          await loadTasks();
        } catch (e) {
          // Polling failure - will retry on next interval
        }
      }, 2000); // Poll every 2 seconds for smooth updates

      return () => {
        clearInterval(pollInterval);
      };
    }
  }, [tasks, loadTasks]);

  // Initialize connection
  useEffect(() => {
    const initializeConnection = async () => {
      setIsLoading(true);
      try {
        // Set wallet context for task service
        const walletType = localStorage.getItem('walletType');
        const chain = walletType === 'phantom' ? 'solana' : walletType === 'metamask' ? 'ethereum' : 'solana';
        taskService.setWalletContext(walletPublicKey, chain);

        const connected = await taskService.connect();
        isConnectedRef.current = connected;

        if (connected) {
          await loadTasks();
        } else {
          setError('Task service is unavailable. Please check your connection.');
        }
      } catch (err) {
        setError((err as Error).message);
      } finally {
        setIsLoading(false);
      }
    };

    if (walletPublicKey) {
      initializeConnection();
    } else {
      // Clear wallet context when no wallet connected
      taskService.setWalletContext(null);
      setTasks([]);
      setActiveTask(null);
      setError(null);
      setIsLoading(false);
      isConnectedRef.current = false;
    }
  }, [walletPublicKey]);

  // Task operations
  const createTask = useCallback(async (request: TaskCreationRequest): Promise<boolean> => {
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

        // Refresh tasks immediately to show the new task
        setTimeout(async () => {
          try {
            await loadTasks();
          } catch (e) {
            // Failed to refresh - will sync on next poll
          }
        }, 500); // Quick refresh to show task in UI

        return true;
      } else {
        setError(response.error || 'Failed to create task');
        return false;
      }
    } catch (error) {
      setError((error as Error).message);
      return false;
    } finally {
      setIsLoading(false);
    }
  }, []);

  const updateTask = useCallback(async (request: TaskUpdateRequest): Promise<boolean> => {
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
        return true;
      } else {
        setError(response.error || 'Failed to update task');
        return false;
      }
    } catch (error) {
      setError((error as Error).message);
      return false;
    }
  }, [activeTask?.id]);

  const deleteTask = useCallback(async (id: string): Promise<boolean> => {
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
        return true;
      } else {
        setError(response.error || 'Failed to delete task');
        return false;
      }
    } catch (error) {
      setError((error as Error).message);
      return false;
    }
  }, [activeTask?.id]);

  const startTask = useCallback(async (id: string): Promise<boolean> => {
    if (!isConnectedRef.current) {
      setError('Task service is not available');
      return false;
    }

    try {
      const response = await taskService.startTask(id);
      if (response.success && response.data) {
        setTasks(prev => ensureArray(prev).map(t => t.id === id ? response.data! : t));
        if (activeTask?.id === id) {
          setActiveTask(response.data);
        }
        return true;
      } else {
        setError(response.error || 'Failed to start task');
        return false;
      }
    } catch (error) {
      setError((error as Error).message);
      return false;
    }
  }, [activeTask?.id]);

  const pauseTask = useCallback(async (id: string): Promise<boolean> => {
    try {
      const response = await taskService.pauseTask(id);
      if (response.success && response.data) {
        setTasks(prev => ensureArray(prev).map(t => t.id === id ? response.data! : t));
        if (activeTask?.id === id) {
          setActiveTask(response.data);
        }
        return true;
      } else {
        setError(response.error || 'Failed to pause task');
        return false;
      }
    } catch (error) {
      setError((error as Error).message);
      return false;
    }
  }, [activeTask?.id]);

  const resumeTask = useCallback(async (id: string): Promise<boolean> => {
    try {
      const response = await taskService.resumeTask(id);
      if (response.success && response.data) {
        setTasks(prev => ensureArray(prev).map(t => t.id === id ? response.data! : t));
        if (activeTask?.id === id) {
          setActiveTask(response.data);
        }
        return true;
      } else {
        setError(response.error || 'Failed to resume task');
        return false;
      }
    } catch (error) {
      setError((error as Error).message);
      return false;
    }
  }, [activeTask?.id]);

  const cancelTask = useCallback(async (id: string): Promise<boolean> => {
    try {
      const response = await taskService.cancelTask(id);
      if (response.success && response.data) {
        setTasks(prev => ensureArray(prev).map(t => t.id === id ? response.data! : t));
        if (activeTask?.id === id) {
          setActiveTask(response.data);
        }
        return true;
      } else {
        setError(response.error || 'Failed to cancel task');
        return false;
      }
    } catch (error) {
      setError((error as Error).message);
      return false;
    }
  }, [activeTask?.id]);

  const retryTask = useCallback(async (id: string): Promise<boolean> => {
    try {
      const response = await taskService.retryTask(id);
      if (response.success && response.data) {
        setTasks(prev => ensureArray(prev).map(t => t.id === id ? response.data! : t));
        if (activeTask?.id === id) {
          setActiveTask(response.data);
        }
        return true;
      } else {
        setError(response.error || 'Failed to retry task');
        return false;
      }
    } catch (error) {
      setError((error as Error).message);
      return false;
    }
  }, [activeTask?.id]);

  const processTask = useCallback(async (id: string, isAutoProcessing: boolean = false): Promise<boolean> => {
    // First try to get task from current tasks array
    let task = ensureArray(tasks).find(t => t.id === id);
    let taskName = task?.name;

    // If task not found in current array, try to fetch it first
    if (!task) {
      try {
        const taskResponse = await taskService.getTask(id);
        if (taskResponse.success && taskResponse.data) {
          task = taskResponse.data;
          taskName = task.name;
        }
      } catch (error) {
        // Failed to fetch task details - continue with processing
      }
    }

    try {
      const response = await taskService.processTask(id);
      if (response.success && response.data) {
        // Use the task name from the response data if available, otherwise fall back to what we found
        const finalTaskName = response.data.name || taskName || 'Unknown Task';

        setTasks(prev => ensureArray(prev).map(t => t.id === id ? response.data! : t));
        if (activeTask?.id === id) {
          setActiveTask(response.data);
        }

        // Add chat notification for task completion
        if (addChatNotification) {
          const processingTime = response.data.action_duration || undefined;
          let notificationMessage = `Task "${finalTaskName}" has been completed successfully!`;

          if (response.data.action_result) {
            // Clean up the result message - remove any extra whitespace/newlines
            const cleanResult = response.data.action_result.trim();
            notificationMessage += `\n\n**Result:**\n${cleanResult}`;
          }

          if (processingTime) {
            notificationMessage += `\n\n*Processing time: ${(processingTime / 1000).toFixed(2)}s*`;
          }

          addChatNotification(id, finalTaskName, notificationMessage, processingTime);
        }

        // Refresh tasks to get updated results
        setTimeout(() => loadTasks(), 1000);
        return true;
      } else {
        const finalTaskName = taskName || 'Unknown Task';

        // Only set global error state if this is not auto-processing
        if (!isAutoProcessing) {
          setError(response.error || 'Failed to process task');
        }

        // Add chat notification for task failure
        if (addChatNotification) {
          addChatNotification(id, finalTaskName, `Task "${finalTaskName}" failed to process: ${response.error || 'Unknown error'}`);
        }

        return false;
      }
    } catch (error) {
      const finalTaskName = taskName || 'Unknown Task';

      // Only set global error state if this is not auto-processing
      if (!isAutoProcessing) {
        setError((error as Error).message);
      }

      // Add chat notification for task error
      if (addChatNotification) {
        addChatNotification(id, finalTaskName, `Task "${finalTaskName}" encountered an error: ${(error as Error).message}`);
      }

      return false;
    }
  }, [activeTask?.id, loadTasks, tasks, addChatNotification]);


  // Analytics & queries
  const getTasksByStatus = useCallback((status: TaskStatus): Task[] => {
    return Array.isArray(tasks) ? tasks.filter(task => task.status.state === status.state) : [];
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


  // Utility functions
  const refresh = useCallback(async () => {
    setIsLoading(true);
    try {
      await loadTasks();
    } catch (err) {
      setError((err as Error).message);
    } finally {
      setIsLoading(false);
    }
  }, [loadTasks]);

  const clearError = useCallback(() => {
    setError(null);
  }, []);

  return {
    // State
    tasks,
    filteredTasks,
    activeTask,
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
    processTask,

    // Task Selection
    selectTask,
    getTask,

    // Analytics
    getTasksByStatus,
    getTasksByPriority,

    // Utility
    refresh,
    clearError,
  };
};