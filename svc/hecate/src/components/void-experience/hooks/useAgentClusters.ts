import { useState, useEffect, useCallback } from 'react';
import { agentService } from '../../../common/services/agent-service';
import type { Agent } from '../../../types/agents';
import type { ClusterData } from '../VoidExperience';

// Color mapping for different agent types
const TYPE_COLORS: Record<string, string> = {
  conversational: '#e6c200', // Gold for HECATE vessel AI
  specialized: '#b967ff',    // Purple for specialized agents
  system: '#00d4ff',         // Cyan for system services
  protocol: '#00ff9d',       // Green for protocols
  tool: '#ff6b6b',           // Red for tools
  default: '#e8e8e8',        // Silver default
};

// Map agent status strings to our status types
const mapStatus = (status: string): 'healthy' | 'unhealthy' | 'unknown' => {
  const lowerStatus = status.toLowerCase();
  if (lowerStatus.includes('healthy') || lowerStatus.includes('running') || lowerStatus.includes('active')) {
    return 'healthy';
  }
  if (lowerStatus.includes('unhealthy') || lowerStatus.includes('error') || lowerStatus.includes('failed')) {
    return 'unhealthy';
  }
  return 'unknown';
};

// Map agent type to cluster type
const mapType = (type: string): ClusterData['type'] => {
  const lowerType = type.toLowerCase();
  if (lowerType.includes('agent') || lowerType.includes('conversational') || lowerType.includes('specialized')) {
    return 'agent';
  }
  if (lowerType.includes('protocol') || lowerType.includes('a2a') || lowerType.includes('mcp')) {
    return 'protocol';
  }
  if (lowerType.includes('tool')) {
    return 'tool';
  }
  return 'service';
};

// Transform Agent to ClusterData
const agentToCluster = (agent: Agent, index: number): ClusterData => {
  const type = mapType(agent.type);
  const status = mapStatus(agent.status);

  // Determine color based on agent name or type
  let color = TYPE_COLORS[agent.type] || TYPE_COLORS.default;
  if (agent.name.toLowerCase().includes('hecate')) {
    color = '#e6c200'; // Gold for Hecate
  } else if (agent.name.toLowerCase().includes('siren')) {
    color = '#b967ff'; // Purple for Siren
  } else if (agent.name.toLowerCase().includes('erebus')) {
    color = '#00d4ff'; // Cyan for Erebus
  } else if (agent.name.toLowerCase().includes('protocol')) {
    color = '#00ff9d'; // Green for protocols
  }

  return {
    id: `agent-${agent.name}-${index}`,
    name: agent.name,
    type,
    status,
    description: agent.description,
    color,
    metrics: {
      tasksProcessed: agent.metrics?.tasks_processed,
      uptime: agent.metrics?.uptime,
      lastActive: agent.metrics?.last_activity,
    },
  };
};

// Fallback clusters when no agents are discovered
const FALLBACK_CLUSTERS: ClusterData[] = [
  {
    id: 'hecate-fallback',
    name: 'HECATE',
    type: 'agent',
    status: 'unknown',
    description: 'MK1 Vessel AI',
    color: '#e6c200',
  },
  {
    id: 'siren-fallback',
    name: 'Siren',
    type: 'agent',
    status: 'unknown',
    description: 'Marketing Agent',
    color: '#b967ff',
  },
  {
    id: 'erebus-fallback',
    name: 'Erebus',
    type: 'service',
    status: 'unknown',
    description: 'Router Service',
    color: '#00d4ff',
  },
  {
    id: 'protocols-fallback',
    name: 'Protocols',
    type: 'protocol',
    status: 'unknown',
    description: 'A2A/MCP Protocols',
    color: '#00ff9d',
  },
];

interface UseAgentClustersResult {
  clusters: ClusterData[];
  isLoading: boolean;
  error: string | null;
  refetch: () => Promise<void>;
}

export const useAgentClusters = (): UseAgentClustersResult => {
  const [clusters, setClusters] = useState<ClusterData[]>(FALLBACK_CLUSTERS);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchClusters = useCallback(async () => {
    setIsLoading(true);
    setError(null);

    try {
      const response = await agentService.getAgents();

      if (response.success && response.data?.agents) {
        const agentClusters = response.data.agents.map(agentToCluster);

        // If we got agents, use them; otherwise keep fallback
        if (agentClusters.length > 0) {
          setClusters(agentClusters);
        } else {
          setClusters(FALLBACK_CLUSTERS);
        }
      } else {
        setError(response.error || 'Failed to fetch agents');
        setClusters(FALLBACK_CLUSTERS);
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error';
      console.error('Failed to fetch agent clusters:', errorMessage);
      setError(errorMessage);
      setClusters(FALLBACK_CLUSTERS);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchClusters();

    // Refetch every 30 seconds to keep status updated
    const interval = setInterval(fetchClusters, 30000);
    return () => clearInterval(interval);
  }, [fetchClusters]);

  return {
    clusters,
    isLoading,
    error,
    refetch: fetchClusters,
  };
};

export default useAgentClusters;
