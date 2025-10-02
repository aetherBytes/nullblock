export type TaskState =
  | 'submitted'
  | 'working'
  | 'input-required'
  | 'completed'
  | 'canceled'
  | 'failed'
  | 'rejected'
  | 'auth-required'
  | 'unknown';

export interface TaskStatus {
  state: TaskState;
  message?: string;
  timestamp?: string;
}

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

export interface MessagePart {
  type: 'text' | 'file' | 'data';
  text?: string;
  file?: {
    type: 'bytes' | 'uri';
    name: string;
    bytes?: string;
    uri?: string;
    mimeType: string;
  };
  data?: any;
  mimeType?: string;
}

export interface A2AMessage {
  messageId: string;
  role: 'user' | 'agent';
  parts: MessagePart[];
  timestamp?: string;
  metadata?: Record<string, any>;
  extensions?: string[];
  referenceTaskIds?: string[];
  taskId?: string;
  contextId?: string;
  kind: string;
}

export interface A2AArtifact {
  id: string;
  parts: MessagePart[];
  metadata?: Record<string, any>;
}

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

  // A2A Protocol required fields
  contextId: string;
  kind: string;
  status: TaskStatus;

  // A2A Protocol optional fields
  history?: A2AMessage[];
  artifacts?: A2AArtifact[];
  metadata?: Record<string, any>;

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
  assigned_agent_id?: string;
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
  status?: TaskState[];
  type?: TaskType[];
  category?: TaskCategory[];
  priority?: TaskPriority[];
  assigned_agent_id?: string;
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
  status?: TaskState;
  progress?: number;
  parameters?: Record<string, any>;
  priority?: TaskPriority;
  outcome?: TaskOutcome;
}