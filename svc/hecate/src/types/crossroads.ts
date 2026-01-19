export type ToolCategory =
  | 'scanner'
  | 'edge'
  | 'strategy'
  | 'curve'
  | 'research'
  | 'kol'
  | 'threat'
  | 'event'
  | 'engram'
  | 'learning'
  | 'consensus'
  | 'swarm'
  | 'approval'
  | 'position'
  | 'utility'
  | 'integration'
  | 'analysis'
  | 'unknown';

export type HealthStatus = 'healthy' | 'degraded' | 'unhealthy' | 'unknown';

export interface MCPTool {
  name: string;
  description: string;
  inputSchema: Record<string, unknown>;
  category: ToolCategory;
  isHot: boolean;
  provider: string;
  relatedCow?: string;
  endpoint: string;
}

export interface DiscoveredAgent {
  name: string;
  agentType: string;
  status: HealthStatus;
  capabilities: string[];
  endpoint: string;
  provider: string;
  description?: string;
  model?: string;
}

export interface DiscoveredProtocol {
  name: string;
  protocolType: string;
  version: string;
  endpoint: string;
  provider: string;
  description?: string;
}

export interface ProviderHealth {
  provider: string;
  status: HealthStatus;
  latencyMs?: number;
  lastChecked: string;
  error?: string;
}

export interface CategorySummary {
  category: ToolCategory;
  count: number;
  icon: string;
}

export interface ToolsResponse {
  tools: MCPTool[];
  totalCount: number;
  hotCount: number;
  categories: CategorySummary[];
  discoveryTimeMs: number;
}

export interface AgentsResponse {
  agents: DiscoveredAgent[];
  totalCount: number;
  healthyCount: number;
  discoveryTimeMs: number;
}

export interface ProtocolsResponse {
  protocols: DiscoveredProtocol[];
  totalCount: number;
  discoveryTimeMs: number;
}

export interface HotItemsResponse {
  tools: MCPTool[];
  totalCount: number;
  discoveryTimeMs: number;
}

export interface HealthResponse {
  providers: ProviderHealth[];
  overallStatus: HealthStatus;
  checkedAt: string;
}

export interface DiscoveryResponse {
  tools: MCPTool[];
  agents: DiscoveredAgent[];
  protocols: DiscoveredProtocol[];
  hot: MCPTool[];
  providerHealth: ProviderHealth[];
  discoveryTimeMs: number;
}

export const TOOL_CATEGORY_ICONS: Record<ToolCategory, string> = {
  scanner: '🔍',
  edge: '📊',
  strategy: '♟️',
  curve: '📈',
  research: '🔬',
  kol: '👥',
  threat: '🛡️',
  event: '📡',
  engram: '🧠',
  learning: '📚',
  consensus: '🔥',
  swarm: '🐝',
  approval: '✅',
  position: '💰',
  utility: '🔧',
  integration: '🔌',
  analysis: '📋',
  unknown: '❓',
};

export const TOOL_CATEGORY_LABELS: Record<ToolCategory, string> = {
  scanner: 'Scanner',
  edge: 'Edge Detection',
  strategy: 'Strategy',
  curve: 'Bonding Curve',
  research: 'Research',
  kol: 'KOL Tracking',
  threat: 'Threat Detection',
  event: 'Events',
  engram: 'Engrams',
  learning: 'Learning',
  consensus: 'LLM Consensus',
  swarm: 'Swarm',
  approval: 'Approvals',
  position: 'Positions',
  utility: 'Utility',
  integration: 'Integration',
  analysis: 'Analysis',
  unknown: 'Other',
};
