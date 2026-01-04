import {
  Agent,
  AgentServiceResponse,
  AgentDiscoveryResponse,
  AgentType,
  AgentStatus
} from '../../types/agents';

class AgentService {
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
  ): Promise<AgentServiceResponse<T>> {
    try {
      if (!this.isConnected) {
        await this.connect();
      }

      const headers: Record<string, string> = {
        'Content-Type': 'application/json',
        ...options.headers,
      };

      const url = `${this.erebusUrl}${endpoint}`;

      const response = await fetch(url, {
        headers,
        ...options,
      });

      const responseJson = await response.json();

      return {
        success: response.ok,
        data: response.ok ? responseJson : undefined,
        error: response.ok ? undefined : responseJson.message || responseJson.error || 'Request failed',
        timestamp: new Date(),
      };
    } catch (error) {
      console.error('Agent service request failed:', error);
      return {
        success: false,
        error: (error as Error).message,
        timestamp: new Date(),
      };
    }
  }

  // Agent Discovery Operations
  async getAgents(): Promise<AgentServiceResponse<AgentDiscoveryResponse>> {
    return this.makeRequest<AgentDiscoveryResponse>('/api/discovery/agents');
  }

  async getAgentHealth(agentName: string): Promise<AgentServiceResponse<any>> {
    return this.makeRequest<any>(`/api/agents/${agentName}/status`);
  }

  async getAgentCapabilities(agentName: string): Promise<AgentServiceResponse<any>> {

    // For specialized endpoints, route to specific agent capabilities
    if (agentName === 'siren') {
      return this.makeRequest<any>(`/api/agents/siren/themes`);
    } else if (agentName === 'hecate') {
      return this.makeRequest<any>(`/api/agents/hecate/model-info`);
    }

    // Fallback to general status
    return this.getAgentHealth(agentName);
  }

  // Agent Interaction Operations
  async chatWithAgent(agentName: string, message: string): Promise<AgentServiceResponse<any>> {
    return this.makeRequest<any>(`/api/agents/${agentName}/chat`, {
      method: 'POST',
      body: JSON.stringify({ message }),
    });
  }

  async setAgentModel(agentName: string, modelName: string): Promise<AgentServiceResponse<any>> {
    return this.makeRequest<any>(`/api/agents/${agentName}/set-model`, {
      method: 'POST',
      body: JSON.stringify({ model_name: modelName }),
    });
  }

  async assignTaskToAgent(agentName: string, taskId: string): Promise<AgentServiceResponse<any>> {
    return this.makeRequest<any>(`/api/agents/tasks/${taskId}`, {
      method: 'PUT',
      body: JSON.stringify({ assigned_agent: agentName }),
    });
  }

  async clearConversation(agentName: string): Promise<AgentServiceResponse<boolean>> {
    return this.makeRequest<boolean>(`/api/agents/${agentName}/clear`, {
      method: 'POST',
    });
  }

  // Utility methods
  getAgentStatusColor(status: string): string {
    switch (status) {
      case 'healthy':
        return '#4ecdc4'; // Green
      case 'unhealthy':
        return '#ff6b6b'; // Red
      case 'unknown':
      default:
        return '#feca57'; // Yellow
    }
  }

  getAgentTypeIcon(type: string): string {
    switch (type) {
      case 'conversational':
        return '💬';
      case 'specialized':
        return '🎯';
      case 'system':
        return '⚙️';
      default:
        return '🤖';
    }
  }

  getCapabilityIcon(capability: string): string {
    switch (capability) {
      case 'chat':
        return '💬';
      case 'reasoning':
        return '🧠';
      case 'model_switching':
        return '🔄';
      case 'task_execution':
        return '⚡';
      case 'content_generation':
        return '📝';
      case 'social_media_management':
        return '📱';
      case 'siren_automation':
        return '🎯';
      case 'community_engagement':
        return '🤝';
      case 'brand_management':
        return '🏷️';
      default:
        return '🔧';
    }
  }

  isAgentOnline(agent: Agent): boolean {
    return agent.status === 'healthy';
  }

  getAgentDescription(agent: Agent): string {
    return agent.description || `${agent.type} agent for ${agent.name}`;
  }

  getAgentMetrics(agent: Agent): string[] {
    const metrics: string[] = [];

    if (agent.metrics) {
      // Common base stats for all agents
      if (agent.metrics.tasks_processed !== undefined) {
        metrics.push(`Tasks: ${agent.metrics.tasks_processed}`);
      }
      if (agent.metrics.last_activity) {
        metrics.push(`Last: ${agent.metrics.last_activity}`);
      }

      // Custom stats based on agent type/name
      if (agent.name === 'hecate') {
        // Hecate-specific stats
        if (agent.metrics.llm_factory) {
          metrics.push(`LLM: ${agent.metrics.llm_factory}`);
        }
        if (agent.metrics.orchestration_enabled) {
          metrics.push(`Orchestration: Active`);
        }
      } else if (agent.name === 'siren') {
        // Siren-specific marketing stats
        if (agent.metrics.content_themes !== undefined) {
          metrics.push(`Themes: ${agent.metrics.content_themes}`);
        }
        if (agent.metrics.twitter_integration) {
          metrics.push(`Twitter: ${agent.metrics.twitter_integration}`);
        }
        if (agent.metrics.campaigns_active !== undefined) {
          metrics.push(`Campaigns: ${agent.metrics.campaigns_active}`);
        }
      } else {
        // Generic stats for other agents
        if (agent.metrics.success_rate !== undefined) {
          metrics.push(`Success: ${agent.metrics.success_rate}%`);
        }
        if (agent.metrics.uptime) {
          metrics.push(`Uptime: ${agent.metrics.uptime}`);
        }
      }
    }

    return metrics;
  }
}

export const agentService = new AgentService();
export default AgentService;