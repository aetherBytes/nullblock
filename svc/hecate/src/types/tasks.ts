export type TaskStatus =
  | 'created'
  | 'assigned'
  | 'running'
  | 'completed'
  | 'failed'
  | 'paused'
  | 'cancelled';

export type TaskPriority = 'low' | 'medium' | 'high' | 'urgent' | 'critical';

export type TaskCategory =
  | 'autonomous'
  | 'user-assigned'
  | 'system-generated'
  | 'event-triggered';

export type TaskType =
  | 'trading'
  | 'arbitrage'
  | 'portfolio'
  | 'social'
  | 'research'
  | 'monitoring'
  | 'analysis'
  | 'automation'
  | 'communication'
  | 'mcp'
  | 'agent'
  | 'system';

export type EventType =
  | 'price_change'
  | 'market_opportunity'
  | 'user_interaction'
  | 'agent_completion'
  | 'system_alert'
  | 'time_trigger'
  | 'threshold_breach'
  | 'new_data';

export interface TaskEvent {
  id: string;
  type: EventType;
  timestamp: Date;
  data: any;
  source: string;
  processed: boolean;
}

export interface TaskDependency {
  taskId: string;
  type: 'blocks' | 'triggers' | 'notifies';
  condition?: string;
}

export interface TaskOutcome {
  success: boolean;
  result?: any;
  error?: string;
  metrics?: Record<string, number>;
  nextActions?: string[];
}

export interface TaskMetrics {
  executionTime?: number;
  resourceUsage?: number;
  successRate?: number;
  userSatisfaction?: number;
  profitability?: number;
  efficiency?: number;
}

export interface Task {
  id: string;
  name: string;
  description: string;
  type: TaskType;
  category: TaskCategory;
  status: TaskStatus;
  priority: TaskPriority;

  // Lifecycle
  createdAt: Date;
  assignedAt?: Date;
  startedAt?: Date;
  completedAt?: Date;
  updatedAt: Date;

  // Execution
  progress: number;
  estimatedDuration?: number;
  actualDuration?: number;

  // Relationships
  parentTaskId?: string;
  subTasks: string[];
  dependencies: TaskDependency[];

  // Context
  context: Record<string, any>;
  parameters: Record<string, any>;
  constraints?: Record<string, any>;

  // Results
  outcome?: TaskOutcome;
  metrics?: TaskMetrics;
  logs: TaskLog[];

  // Automation
  triggers: TaskEvent[];
  autoRetry: boolean;
  maxRetries: number;
  currentRetries: number;

  // Agent assignment
  assignedAgent?: string;
  requiredCapabilities: string[];

  // User interaction
  userApprovalRequired: boolean;
  userNotifications: boolean;

  // Learning
  feedbackScore?: number;
  improvements?: string[];
}

export interface TaskLog {
  id: string;
  timestamp: Date;
  level: 'debug' | 'info' | 'warning' | 'error' | 'success';
  message: string;
  data?: any;
  source: string;
}

export interface TaskQueue {
  id: string;
  name: string;
  priority: TaskPriority;
  tasks: Task[];
  maxConcurrency: number;
  currentlyRunning: number;
  paused: boolean;
}

export interface TaskTemplate {
  id: string;
  name: string;
  description: string;
  type: TaskType;
  category: TaskCategory;
  defaultPriority: TaskPriority;
  requiredParameters: string[];
  optionalParameters: string[];
  defaultContext: Record<string, any>;
  estimatedDuration: number;
  requiredCapabilities: string[];
  successCriteria: string[];
}

export interface TaskFilter {
  status?: TaskStatus[];
  type?: TaskType[];
  category?: TaskCategory[];
  priority?: TaskPriority[];
  assignedAgent?: string;
  dateRange?: {
    start: Date;
    end: Date;
  };
  searchTerm?: string;
}

export interface TaskStats {
  total: number;
  byStatus: Record<TaskStatus, number>;
  byType: Record<TaskType, number>;
  byPriority: Record<TaskPriority, number>;
  avgExecutionTime: number;
  successRate: number;
  totalProfit?: number;
}

export interface TaskNotification {
  id: string;
  taskId: string;
  type: 'created' | 'completed' | 'failed' | 'requires_approval' | 'progress_update';
  title: string;
  message: string;
  timestamp: Date;
  read: boolean;
  actionRequired: boolean;
  actions?: {
    label: string;
    action: string;
    variant: 'primary' | 'secondary' | 'danger';
  }[];
}

export interface MotivationState {
  currentGoals: Task[];
  priorities: TaskPriority[];
  focus: TaskType[];
  autonomyLevel: number;
  learningMode: boolean;
  conversationContext: Record<string, any>;
  userPreferences: Record<string, any>;
  marketConditions: Record<string, any>;
}

export interface TaskCreationRequest {
  name: string;
  description: string;
  type: TaskType;
  category?: TaskCategory;
  priority?: TaskPriority;
  parameters?: Record<string, any>;
  dependencies?: TaskDependency[];
  userApprovalRequired?: boolean;
  autoStart?: boolean;
}

export interface TaskUpdateRequest {
  id: string;
  status?: TaskStatus;
  progress?: number;
  parameters?: Record<string, any>;
  priority?: TaskPriority;
  outcome?: TaskOutcome;
  addLogs?: TaskLog[];
}