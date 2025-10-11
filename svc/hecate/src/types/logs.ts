export type LogLevel = 'info' | 'warning' | 'error' | 'success' | 'debug';

export type LogCategory =
  | 'AgentConversation'
  | 'LlmApiCall'
  | 'ErebusRouting'
  | 'TaskLifecycle'
  | 'HealthCheck'
  | 'SystemError'
  | 'DatabaseQuery'
  | 'Unknown';

export interface LogEntry {
  timestamp: string;
  level: LogLevel;
  source: string;
  message: string;
  category: LogCategory;
  metadata: Record<string, any>;
  sanitized: boolean;
}

export interface LogsResponse {
  success: boolean;
  data: LogEntry[];
  total: number;
  timestamp: string;
}

export interface LogsQuery {
  limit?: number;
  category?: LogCategory;
  level?: LogLevel;
}

export interface LogMetadata {
  request_id?: string;
  duration_ms?: number;
  status_code?: number;
  model_name?: string;
  tokens_used?: number;
  cost?: number;
  user_id?: string;
  task_id?: string;
  agent_name?: string;
  error_type?: string;
  [key: string]: any;
}

export interface LogStats {
  total_logs: number;
  by_level: Record<LogLevel, number>;
  by_category: Record<LogCategory, number>;
  by_source: Record<string, number>;
}

export const LOG_LEVEL_COLORS: Record<LogLevel, string> = {
  info: '#4a90e2',
  warning: '#ff7f00',
  error: '#ff3333',
  success: '#00ff9d',
  debug: '#808080',
};

export const LOG_CATEGORY_ICONS: Record<LogCategory, string> = {
  AgentConversation: '🤖',
  LlmApiCall: '🌐',
  ErebusRouting: '🛣️',
  TaskLifecycle: '📋',
  HealthCheck: '🏥',
  SystemError: '⚠️',
  DatabaseQuery: '🗄️',
  Unknown: '❓',
};

export const LOG_CATEGORY_COLORS: Record<LogCategory, string> = {
  AgentConversation: '#e6c200',
  LlmApiCall: '#b967ff',
  ErebusRouting: '#00a8ff',
  TaskLifecycle: '#4ecdc4',
  HealthCheck: '#00ff9d',
  SystemError: '#ff3333',
  DatabaseQuery: '#96ceb4',
  Unknown: '#808080',
};
