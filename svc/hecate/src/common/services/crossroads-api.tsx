import axios from 'axios';
import type {
  ToolsResponse,
  AgentsResponse,
  ProtocolsResponse,
  HotItemsResponse,
  HealthResponse,
  DiscoveryResponse,
  MCPTool,
  DiscoveredAgent,
} from '../../types/crossroads';

const EREBUS_API_BASE_URL = import.meta.env.VITE_EREBUS_API_URL || 'http://localhost:3000';

const snakeToCamel = (str: string): string =>
  str.replace(/_([a-z])/g, (_, letter) => letter.toUpperCase());

const transformKeys = <T,>(obj: unknown): T => {
  if (Array.isArray(obj)) {
    return obj.map((item) => transformKeys(item)) as T;
  }
  if (obj !== null && typeof obj === 'object') {
    const transformed: Record<string, unknown> = {};
    for (const [key, value] of Object.entries(obj)) {
      transformed[snakeToCamel(key)] = transformKeys(value);
    }
    return transformed as T;
  }
  return obj as T;
};

export const discoverAllMcpTools = async (): Promise<ToolsResponse> => {
  try {
    const response = await axios.get(`${EREBUS_API_BASE_URL}/api/discovery/tools`);
    const data = transformKeys<ToolsResponse>(response.data);
    console.log('Discovered MCP tools:', data.totalCount);
    return data;
  } catch (error) {
    console.error('Failed to discover MCP tools:', error);
    throw error;
  }
};

export const discoverAgents = async (): Promise<AgentsResponse> => {
  try {
    const response = await axios.get(`${EREBUS_API_BASE_URL}/api/discovery/agents`);
    const data = transformKeys<AgentsResponse>(response.data);
    console.log('Discovered agents:', data.totalCount);
    return data;
  } catch (error) {
    console.error('Failed to discover agents:', error);
    throw error;
  }
};

export const discoverProtocols = async (): Promise<ProtocolsResponse> => {
  try {
    const response = await axios.get(`${EREBUS_API_BASE_URL}/api/discovery/protocols`);
    const data = transformKeys<ProtocolsResponse>(response.data);
    console.log('Discovered protocols:', data.totalCount);
    return data;
  } catch (error) {
    console.error('Failed to discover protocols:', error);
    throw error;
  }
};

export const discoverHotItems = async (): Promise<HotItemsResponse> => {
  try {
    const response = await axios.get(`${EREBUS_API_BASE_URL}/api/discovery/hot`);
    const data = transformKeys<HotItemsResponse>(response.data);
    console.log('Discovered hot items:', data.totalCount);
    return data;
  } catch (error) {
    console.error('Failed to discover hot items:', error);
    throw error;
  }
};

export const discoverAll = async (): Promise<DiscoveryResponse> => {
  try {
    const response = await axios.get(`${EREBUS_API_BASE_URL}/api/discovery/all`);
    const data = transformKeys<DiscoveryResponse>(response.data);
    console.log('Discovery all response:', {
      tools: data.tools.length,
      agents: data.agents.length,
      protocols: data.protocols.length,
      hot: data.hot.length,
    });
    return data;
  } catch (error) {
    console.error('Failed to fetch all discovery data:', error);
    throw error;
  }
};

export const checkDiscoveryHealth = async (): Promise<HealthResponse> => {
  try {
    const response = await axios.get(`${EREBUS_API_BASE_URL}/api/discovery/health`);
    const data = transformKeys<HealthResponse>(response.data);
    console.log('Discovery health:', data.overallStatus);
    return data;
  } catch (error) {
    console.error('Failed to check discovery health:', error);
    throw error;
  }
};

export const filterToolsByCategory = (
  tools: MCPTool[],
  category: string | null
): MCPTool[] => {
  if (!category || category === 'All') {
    return tools;
  }
  return tools.filter((tool) => tool.category === category.toLowerCase());
};

export const searchTools = (tools: MCPTool[], query: string): MCPTool[] => {
  if (!query.trim()) {
    return tools;
  }
  const lowerQuery = query.toLowerCase();
  return tools.filter(
    (tool) =>
      tool.name.toLowerCase().includes(lowerQuery) ||
      tool.description.toLowerCase().includes(lowerQuery) ||
      tool.provider.toLowerCase().includes(lowerQuery)
  );
};

export const groupToolsByCategory = (
  tools: MCPTool[]
): Record<string, MCPTool[]> => {
  const grouped: Record<string, MCPTool[]> = {};
  for (const tool of tools) {
    const category = tool.category;
    if (!grouped[category]) {
      grouped[category] = [];
    }
    grouped[category].push(tool);
  }
  return grouped;
};

export const getHotTools = (tools: MCPTool[]): MCPTool[] => {
  return tools.filter((tool) => tool.isHot);
};

export const getHealthyAgents = (agents: DiscoveredAgent[]): DiscoveredAgent[] => {
  return agents.filter((agent) => agent.status === 'healthy');
};
