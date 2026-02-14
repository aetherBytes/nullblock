import type { Task, TaskCreationRequest, TaskUpdateRequest, TaskFilter } from '../../types/tasks';

export interface TaskServiceResponse<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  timestamp: Date;
}

class TaskService {
  private erebusUrl: string;
  private isConnected: boolean = false;
  private walletAddress: string | null = null;
  private walletChain: string | null = null;

  constructor(erebusUrl: string = import.meta.env.VITE_EREBUS_API_URL || 'http://localhost:3000') {
    this.erebusUrl = erebusUrl;
  }

  setWalletContext(walletAddress: string | null, chain: string = 'solana') {
    this.walletAddress = walletAddress;
    this.walletChain = chain;
  }

  async connect(): Promise<boolean> {
    try {
      const response = await fetch(`${this.erebusUrl}/health`);

      this.isConnected = response.ok;

      return this.isConnected;
    } catch (error) {
      console.debug('Task service unavailable (Erebus not running)');
      this.isConnected = false;

      return false;
    }
  }

  private async makeRequest<T>(
    endpoint: string,
    options: RequestInit = {},
  ): Promise<TaskServiceResponse<T>> {
    try {
      if (!this.isConnected) {
        await this.connect();
      }

      const headers: Record<string, string> = {
        'Content-Type': 'application/json',
        ...(options.headers as Record<string, string>),
      };

      if (this.walletAddress) {
        headers['x-wallet-address'] = this.walletAddress;
      }

      if (this.walletChain) {
        headers['x-wallet-chain'] = this.walletChain;
      }

      const url = `${this.erebusUrl}/api/agents/tasks${endpoint}`;

      const response = await fetch(url, {
        ...options,
        headers,
      });

      const responseJson = await response.json();

      // Handle backend API response format which wraps data in { data: [...], success: true }
      const actualData =
        response.ok && responseJson.data !== undefined ? responseJson.data : responseJson;

      const result = {
        success: response.ok,
        data: response.ok ? actualData : undefined,
        error: response.ok
          ? undefined
          : responseJson.message || responseJson.error || 'Request failed',
        timestamp: new Date(),
      };

      return result;
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
    return this.makeRequest<Task>('', {
      method: 'POST',
      body: JSON.stringify(request),
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

  async processTask(id: string): Promise<TaskServiceResponse<Task>> {
    return this.makeRequest<Task>(`/${id}/process`, {
      method: 'POST',
    });
  }

  // User management
  async registerUser(
    walletAddress: string,
    chain: string = 'solana',
  ): Promise<TaskServiceResponse<any>> {
    try {
      // Determine provider based on chain
      const provider =
        chain === 'solana' ? 'phantom' : chain === 'ethereum' ? 'metamask' : 'unknown';

      const requestBody = {
        source_identifier: walletAddress,
        network: chain, // Primary field (required)
        chain, // Legacy field for backward compatibility
        source_type: {
          type: 'web3_wallet',
          provider,
          network: chain,
          metadata: {},
        },
        wallet_type: null,
      };

      // Use direct Erebus endpoint for user registration (not through tasks)
      const url = `${this.erebusUrl}/api/agents/users/register`;

      const response = await fetch(url, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'x-wallet-address': walletAddress,
          'x-wallet-chain': chain,
        },
        body: JSON.stringify(requestBody),
      });

      const data = await response.json();

      if (response.ok) {
        // Update wallet context after successful registration
        this.setWalletContext(walletAddress, chain);
      }

      return {
        success: response.ok,
        data: response.ok ? data : undefined,
        error: response.ok ? undefined : data.message || 'Request failed',
        timestamp: new Date(),
      };
    } catch (error) {
      console.error('❌ User registration error:', error);

      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date(),
      };
    }
  }

  // Utility methods
  private filterToParams(filter: TaskFilter): Record<string, string> {
    const params: Record<string, string> = {};

    if (filter.status) {
      params.status = filter.status.join(',');
    }

    if (filter.type) {
      params.task_type = filter.type.join(',');
    }

    if (filter.category) {
      params.category = filter.category.join(',');
    }

    if (filter.priority) {
      params.priority = filter.priority.join(',');
    }

    if (filter.assigned_agent_id) {
      params.assigned_agent_id = filter.assigned_agent_id;
    }

    if (filter.search_term) {
      params.search = filter.search_term;
    }

    if (filter.date_range) {
      params.start_date = filter.date_range.start.toISOString();
      params.end_date = filter.date_range.end.toISOString();
    }

    return params;
  }

  isTaskStale(task: Task): boolean {
    const now = new Date();
    const lastUpdate = new Date(task.updated_at);
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
      task.sub_tasks.length,
      Object.keys(task.parameters).length,
      task.required_capabilities.length,
    ];

    const totalComplexity = factors.reduce((sum, factor) => sum + factor, 0);

    if (totalComplexity <= 3) {
      return 'simple';
    }

    if (totalComplexity <= 8) {
      return 'moderate';
    }

    return 'complex';
  }
}

export const taskService = new TaskService();

export default TaskService;
