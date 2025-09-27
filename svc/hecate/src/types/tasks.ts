export type TaskStatus =
  | 'created'
  | 'running'
  | 'paused'
  | 'completed'
  | 'failed'
  | 'cancelled';

export type TaskPriority = 'low' | 'medium' | 'high' | 'urgent' | 'critical';

export type TaskCategory =
  | 'autonomous'
  | 'user_assigned'
  | 'system_generated'
  | 'event_triggered';

export type TaskType =
  | 'arbitrage'
  | 'social'
  | 'portfolio'
  | 'mcp'
  | 'system'
  | 'user_assigned';


export interface TaskOutcome {
  success: boolean;
  result?: any;
  error?: string;
  metrics?: Record<string, any>;
}

export interface Task {
  id: string;
  name: string;
  description: string;
  task_type: TaskType;
  category: TaskCategory;
  status: TaskStatus;
  priority: TaskPriority;

  // Lifecycle
  created_at: Date;
  updated_at: Date;
  started_at?: Date;
  completed_at?: Date;

  // Execution
  progress: number;
  estimated_duration?: number;
  actual_duration?: number;

  // Relationships
  sub_tasks: string[];
  dependencies: string[];

  // Context
  context: Record<string, any>;
  parameters: Record<string, any>;

  // Results
  outcome?: TaskOutcome;
  logs: string[];

  // Automation
  triggers: string[];
  auto_retry: boolean;
  max_retries: number;
  current_retries: number;

  // Agent assignment
  assigned_agent?: string;
  required_capabilities: string[];

  // User interaction
  user_approval_required: boolean;
  user_notifications: boolean;

  // Action tracking fields
  actioned_at?: Date;
  action_result?: string;
  action_metadata: Record<string, any>;
  action_duration?: number;

  // Source tracking fields
  source_identifier?: string;
  source_metadata: Record<string, any>;
}

export interface TaskFilter {
  status?: TaskStatus[];
  type?: TaskType[];
  category?: TaskCategory[];
  priority?: TaskPriority[];
  assigned_agent?: string;
  date_range?: {
    start: Date;
    end: Date;
  };
  search_term?: string;
}

export interface TaskCreationRequest {
  name: string;
  description: string;
  task_type: TaskType;
  category?: TaskCategory;
  priority?: TaskPriority;
  parameters?: Record<string, any>;
  dependencies?: string[];
  user_approval_required?: boolean;
  auto_start?: boolean;
}

export interface TaskUpdateRequest {
  id: string;
  status?: TaskStatus;
  progress?: number;
  parameters?: Record<string, any>;
  priority?: TaskPriority;
  outcome?: TaskOutcome;
}