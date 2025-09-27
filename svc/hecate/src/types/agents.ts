export interface Agent {
  name: string;
  type: string;
  status: string;
  endpoint: string;
  capabilities: string[];
  description: string;
  metrics?: {
    tasks_processed?: number;
    content_themes?: number;
    twitter_integration?: string;
    llm_factory?: string;
    last_activity?: string;
    success_rate?: number;
    uptime?: string;
    orchestration_enabled?: boolean;
    campaigns_active?: number;
  };
  hecate_status?: any;
  marketing_status?: any;
  note?: string;
}

export interface AgentDiscoveryResponse {
  agents: Agent[];
  total_count: number;
  discovery_time_ms: number;
  message: string;
}

export interface AgentServiceResponse<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  timestamp: Date;
}

export type AgentType = 'conversational' | 'specialized' | 'system';
export type AgentStatus = 'healthy' | 'unhealthy' | 'unknown';

export interface AgentMetrics {
  tasks_processed?: number;
  content_themes?: number;
  twitter_integration?: string;
  llm_factory?: string;
  last_activity?: string;
  success_rate?: number;
  uptime?: string;
  orchestration_enabled?: boolean;
  campaigns_active?: number;
}