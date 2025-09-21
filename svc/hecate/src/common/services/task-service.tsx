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
  MotivationState
} from '../../types/tasks';

export interface TaskServiceResponse<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  timestamp: Date;
}

class TaskService {
  private erebusUrl: string;
  private isConnected: boolean = false;

  constructor(erebusUrl: string = import.meta.env.VITE_EREBUS_API_URL || 'http://localhost:3000') {
    this.erebusUrl = erebusUrl;
  }

  async connect(): Promise<boolean> {
    try {
      const response = await fetch(`${this.erebusUrl}/health`);
      this.isConnected = response.ok;
      return this.isConnected;
    } catch (error) {
      console.error('Failed to connect to Erebus:', error);
      this.isConnected = false;
      return false;
    }
  }

  private async makeRequest<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<TaskServiceResponse<T>> {
    try {
      if (!this.isConnected) {
        await this.connect();
      }

      const response = await fetch(`${this.erebusUrl}/api/agents/tasks${endpoint}`, {
        headers: {
          'Content-Type': 'application/json',
          ...options.headers,
        },
        ...options,
      });

      const data = await response.json();

      return {
        success: response.ok,
        data: response.ok ? data : undefined,
        error: response.ok ? undefined : data.message || 'Request failed',
        timestamp: new Date(),
      };
    } catch (error) {
      console.error('Task service request failed:', error);
      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date(),
      };
    }
  }

  // Task CRUD Operations
  async createTask(request: TaskCreationRequest): Promise<TaskServiceResponse<Task>> {
    // Transform frontend request to match backend expectations
    const backendRequest = {
      name: request.name,
      description: request.description,
      task_type: request.type,  // Frontend uses 'type', backend expects 'task_type'
      category: request.category,
      priority: request.priority,
      parameters: request.parameters,
      dependencies: request.dependencies,
      auto_start: request.autoStart,  // Frontend uses camelCase, backend expects snake_case
      user_approval_required: request.userApprovalRequired
    };

    return this.makeRequest<Task>('', {
      method: 'POST',
      body: JSON.stringify(backendRequest),
    });
  }

  async getTask(id: string): Promise<TaskServiceResponse<Task>> {
    return this.makeRequest<Task>(`/${id}`);
  }

  async updateTask(request: TaskUpdateRequest): Promise<TaskServiceResponse<Task>> {
    return this.makeRequest<Task>(`/${request.id}`, {
      method: 'PUT',
      body: JSON.stringify(request),
    });
  }

  async deleteTask(id: string): Promise<TaskServiceResponse<void>> {
    return this.makeRequest<void>(`/${id}`, {
      method: 'DELETE',
    });
  }

  async getTasks(filter?: TaskFilter): Promise<TaskServiceResponse<Task[]>> {
    const queryParams = filter ? `?${new URLSearchParams(this.filterToParams(filter))}` : '';
    return this.makeRequest<Task[]>(`${queryParams}`);
  }

  // Task Lifecycle Operations
  async startTask(id: string): Promise<TaskServiceResponse<Task>> {
    return this.makeRequest<Task>(`/${id}/start`, {
      method: 'POST',
    });
  }

  async pauseTask(id: string): Promise<TaskServiceResponse<Task>> {
    return this.makeRequest<Task>(`/${id}/pause`, {
      method: 'POST',
    });
  }

  async resumeTask(id: string): Promise<TaskServiceResponse<Task>> {
    return this.makeRequest<Task>(`/${id}/resume`, {
      method: 'POST',
    });
  }

  async cancelTask(id: string): Promise<TaskServiceResponse<Task>> {
    return this.makeRequest<Task>(`/${id}/cancel`, {
      method: 'POST',
    });
  }

  async retryTask(id: string): Promise<TaskServiceResponse<Task>> {
    return this.makeRequest<Task>(`/${id}/retry`, {
      method: 'POST',
    });
  }

  // Task Queue Operations
  async getQueues(): Promise<TaskServiceResponse<TaskQueue[]>> {
    return this.makeRequest<TaskQueue[]>('/queues');
  }

  async getQueueTasks(queueId: string): Promise<TaskServiceResponse<Task[]>> {
    return this.makeRequest<Task[]>(`/queues/${queueId}/tasks`);
  }

  async moveTaskToQueue(taskId: string, queueId: string): Promise<TaskServiceResponse<Task>> {
    return this.makeRequest<Task>(`/${taskId}/move`, {
      method: 'POST',
      body: JSON.stringify({ queueId }),
    });
  }

  // Task Templates
  async getTemplates(): Promise<TaskServiceResponse<TaskTemplate[]>> {
    return this.makeRequest<TaskTemplate[]>('/templates');
  }

  async createFromTemplate(
    templateId: string,
    parameters: Record<string, any>
  ): Promise<TaskServiceResponse<Task>> {
    return this.makeRequest<Task>('/from-template', {
      method: 'POST',
      body: JSON.stringify({ templateId, parameters }),
    });
  }

  // Analytics & Stats
  async getStats(filter?: TaskFilter): Promise<TaskServiceResponse<TaskStats>> {
    const queryParams = filter ? `?${new URLSearchParams(this.filterToParams(filter))}` : '';
    return this.makeRequest<TaskStats>(`/stats${queryParams}`);
  }

  // Notifications
  async getNotifications(): Promise<TaskServiceResponse<TaskNotification[]>> {
    return this.makeRequest<TaskNotification[]>('/notifications');
  }

  async markNotificationRead(id: string): Promise<TaskServiceResponse<void>> {
    return this.makeRequest<void>(`/notifications/${id}/read`, {
      method: 'POST',
    });
  }

  async handleNotificationAction(
    id: string,
    action: string
  ): Promise<TaskServiceResponse<void>> {
    return this.makeRequest<void>(`/notifications/${id}/action`, {
      method: 'POST',
      body: JSON.stringify({ action }),
    });
  }

  // Events & Automation
  async getEvents(taskId?: string): Promise<TaskServiceResponse<TaskEvent[]>> {
    const endpoint = taskId ? `/events?taskId=${taskId}` : '/events';
    return this.makeRequest<TaskEvent[]>(endpoint);
  }

  async publishEvent(event: Omit<TaskEvent, 'id'>): Promise<TaskServiceResponse<TaskEvent>> {
    return this.makeRequest<TaskEvent>('/events', {
      method: 'POST',
      body: JSON.stringify(event),
    });
  }

  // Hecate Motivation System
  async getMotivationState(): Promise<TaskServiceResponse<MotivationState>> {
    return this.makeRequest<MotivationState>('/motivation');
  }

  async updateMotivationState(
    updates: Partial<MotivationState>
  ): Promise<TaskServiceResponse<MotivationState>> {
    return this.makeRequest<MotivationState>('/motivation', {
      method: 'PUT',
      body: JSON.stringify(updates),
    });
  }

  async suggestTasks(context: Record<string, any>): Promise<TaskServiceResponse<TaskCreationRequest[]>> {
    return this.makeRequest<TaskCreationRequest[]>('/suggestions', {
      method: 'POST',
      body: JSON.stringify({ context }),
    });
  }

  async learnFromOutcome(taskId: string, feedback: Record<string, any>): Promise<TaskServiceResponse<void>> {
    return this.makeRequest<void>(`/${taskId}/learn`, {
      method: 'POST',
      body: JSON.stringify(feedback),
    });
  }

  // Real-time Updates
  async subscribeToUpdates(
    callback: (task: Task) => void,
    filter?: TaskFilter
  ): Promise<() => void> {
    const eventSource = new EventSource(
      `${this.erebusUrl}/api/agents/tasks/stream${filter ? `?${new URLSearchParams(this.filterToParams(filter))}` : ''}`
    );

    eventSource.onmessage = (event) => {
      try {
        const task: Task = JSON.parse(event.data);
        callback(task);
      } catch (error) {
        console.error('Failed to parse task update:', error);
      }
    };

    eventSource.onerror = (error) => {
      console.error('Task stream error:', error);
    };

    return () => {
      eventSource.close();
    };
  }

  // Utility methods
  private filterToParams(filter: TaskFilter): Record<string, string> {
    const params: Record<string, string> = {};

    if (filter.status) {
      params.status = filter.status.join(',');
    }
    if (filter.type) {
      params.type = filter.type.join(',');
    }
    if (filter.category) {
      params.category = filter.category.join(',');
    }
    if (filter.priority) {
      params.priority = filter.priority.join(',');
    }
    if (filter.assignedAgent) {
      params.assignedAgent = filter.assignedAgent;
    }
    if (filter.searchTerm) {
      params.search = filter.searchTerm;
    }
    if (filter.dateRange) {
      params.startDate = filter.dateRange.start.toISOString();
      params.endDate = filter.dateRange.end.toISOString();
    }

    return params;
  }

  isTaskStale(task: Task): boolean {
    const now = new Date();
    const lastUpdate = new Date(task.updatedAt);
    const staleThreshold = 5 * 60 * 1000; // 5 minutes
    return now.getTime() - lastUpdate.getTime() > staleThreshold;
  }

  getTaskPriorityScore(task: Task): number {
    const priorityScores = {
      low: 1,
      medium: 2,
      high: 3,
      urgent: 4,
      critical: 5,
    };
    return priorityScores[task.priority];
  }

  estimateTaskComplexity(task: Task): 'simple' | 'moderate' | 'complex' {
    const factors = [
      task.dependencies.length,
      task.subTasks.length,
      Object.keys(task.parameters).length,
      task.requiredCapabilities.length,
    ];

    const totalComplexity = factors.reduce((sum, factor) => sum + factor, 0);

    if (totalComplexity <= 3) return 'simple';
    if (totalComplexity <= 8) return 'moderate';
    return 'complex';
  }
}

export const taskService = new TaskService();
export default TaskService;