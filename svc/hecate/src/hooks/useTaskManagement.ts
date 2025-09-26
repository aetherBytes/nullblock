import { useState, useEffect, useCallback, useRef } from 'react';
import {
  Task,
  TaskCreationRequest,
  TaskUpdateRequest,
  TaskFilter,
  TaskStatus,
  TaskPriority
} from '../types/tasks';
import { taskService } from '../common/services/task-service';

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

  // Computed state
  const filteredTasks = Array.isArray(tasks) ? tasks.filter(task => {
    if (filter.status && !filter.status.includes(task.status)) return false;
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

  // Auto-refresh for running tasks
  useEffect(() => {
    const runningTasks = Array.isArray(tasks) ? tasks.filter(task => task.status === 'running') : [];

    if (runningTasks.length > 0 && isConnectedRef.current) {
      console.log(`🔄 Setting up polling for ${runningTasks.length} running tasks`);

      const pollInterval = setInterval(async () => {
        try {
          console.log('🔄 Polling for task updates...');
          await loadTasks();
        } catch (e) {
          console.warn('⚠️ Failed to poll for task updates:', e);
        }
      }, 5000); // Poll every 5 seconds when there are running tasks

      return () => {
        console.log('⏹️ Stopping task polling');
        clearInterval(pollInterval);
      };
    }
  }, [tasks, loadTasks]);

  // Initialize connection
  useEffect(() => {
    const initializeConnection = async () => {
      setIsLoading(true);
      try {
        console.log('🔗 Attempting to connect to task service...');

        // Set wallet context for task service
        const walletType = localStorage.getItem('walletType');
        const chain = walletType === 'phantom' ? 'solana' : walletType === 'metamask' ? 'ethereum' : 'solana';
        taskService.setWalletContext(walletPublicKey, chain);

        const connected = await taskService.connect();
        isConnectedRef.current = connected;
        console.log('🔗 Task service connection:', connected ? 'SUCCESS' : 'FAILED');

        if (connected) {
          console.log('📋 Loading task data for wallet:', walletPublicKey, 'on chain:', chain);
          await loadTasks();
        } else {
          console.log('⚠️ Task service unavailable - no tasks will be loaded');
          setError('Task service is unavailable. Please check your connection.');
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
    } else {
      // Clear wallet context when no wallet connected
      taskService.setWalletContext(null);
      setTasks([]);
      setActiveTask(null);
      setError(null);
      isConnectedRef.current = false;
    }
  }, [walletPublicKey]);

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

        // If auto_start is true, the backend handles processing automatically
        if (request.auto_start) {
          console.log('🔄 Auto-start task created, backend will handle processing automatically');
          // Backend already processes auto_start tasks, no need for frontend processing
          // Set up polling to monitor completion
          setTimeout(async () => {
            try {
              console.log('🔄 Refreshing tasks to check auto-processing completion');
              await loadTasks();
            } catch (e) {
              console.warn('⚠️ Failed to refresh tasks after auto-start:', e);
            }
          }, 3000); // 3 second delay to allow backend processing
        }

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
      console.error('❌ Task start error:', error);
      setError((error as Error).message);
      return false;
    }
  }, [activeTask?.id]);

  const pauseTask = useCallback(async (id: string): Promise<boolean> => {
    console.log('⏸️ Pausing task:', id);
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
      console.error('❌ Task pause error:', error);
      setError((error as Error).message);
      return false;
    }
  }, [activeTask?.id]);

  const resumeTask = useCallback(async (id: string): Promise<boolean> => {
    console.log('▶️ Resuming task:', id);
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
      console.error('❌ Task resume error:', error);
      setError((error as Error).message);
      return false;
    }
  }, [activeTask?.id]);

  const cancelTask = useCallback(async (id: string): Promise<boolean> => {
    console.log('🚫 Cancelling task:', id);
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
      console.error('❌ Task cancel error:', error);
      setError((error as Error).message);
      return false;
    }
  }, [activeTask?.id]);

  const retryTask = useCallback(async (id: string): Promise<boolean> => {
    console.log('🔄 Retrying task:', id);
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
      console.error('❌ Task retry error:', error);
      setError((error as Error).message);
      return false;
    }
  }, [activeTask?.id]);

  const processTask = useCallback(async (id: string, isAutoProcessing: boolean = false): Promise<boolean> => {
    console.log('⚡ Processing task:', id);

    // First try to get task from current tasks array
    let task = ensureArray(tasks).find(t => t.id === id);
    let taskName = task?.name;

    // If task not found in current array, try to fetch it first
    if (!task) {
      console.log('🔍 Task not found in current array, fetching task details...');
      try {
        const taskResponse = await taskService.getTask(id);
        if (taskResponse.success && taskResponse.data) {
          task = taskResponse.data;
          taskName = task.name;
        }
      } catch (error) {
        console.warn('⚠️ Failed to fetch task details:', error);
      }
    }

    try {
      console.log(`🔧 Making processTask API call for task: ${id}`);
      const response = await taskService.processTask(id);
      console.log(`📤 ProcessTask API response:`, response);
      if (response.success && response.data) {
        // Use the task name from the response data if available, otherwise fall back to what we found
        const finalTaskName = response.data.name || taskName || 'Unknown Task';

        console.log('✅ Task processed successfully:', finalTaskName);

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

        console.log(`❌ ProcessTask failed for task ${id}:`, response);
        // Only set global error state if this is not auto-processing
        if (!isAutoProcessing) {
          setError(response.error || 'Failed to process task');
        } else {
          console.warn('⚠️ Auto-processing failed:', response.error || 'Failed to process task');
        }

        // Add chat notification for task failure
        if (addChatNotification) {
          addChatNotification(id, finalTaskName, `Task "${finalTaskName}" failed to process: ${response.error || 'Unknown error'}`);
        }

        return false;
      }
    } catch (error) {
      console.error('❌ Task processing error:', error);
      const finalTaskName = taskName || 'Unknown Task';

      // Only set global error state if this is not auto-processing
      if (!isAutoProcessing) {
        setError((error as Error).message);
      } else {
        console.warn('⚠️ Auto-processing error:', (error as Error).message);
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