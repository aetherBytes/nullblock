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
      console.log('🤖 Making agent request to:', url);
      console.log('📋 Headers:', headers);
      console.log('📋 Options:', options);

      const response = await fetch(url, {
        headers,
        ...options,
      });

      console.log('📤 Response status:', response.status);
      console.log('📤 Response headers:', Object.fromEntries(response.headers.entries()));

      const responseJson = await response.json();
      console.log('📤 Response data:', responseJson);

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
    console.log('🤖 Fetching available agents...');
    return this.makeRequest<AgentDiscoveryResponse>('/api/discovery/agents');
  }

  async getAgentHealth(agentName: string): Promise<AgentServiceResponse<any>> {
    console.log(`🏥 Checking health for agent: ${agentName}`);
    return this.makeRequest<any>(`/api/agents/${agentName}/status`);
  }

  async getAgentCapabilities(agentName: string): Promise<AgentServiceResponse<any>> {
    console.log(`⚙️ Fetching capabilities for agent: ${agentName}`);

    // For specialized endpoints, route to specific agent capabilities
    if (agentName === 'marketing') {
      return this.makeRequest<any>(`/api/agents/marketing/themes`);
    } else if (agentName === 'hecate') {
      return this.makeRequest<any>(`/api/agents/hecate/model-info`);
    }

    // Fallback to general status
    return this.getAgentHealth(agentName);
  }

  // Agent Interaction Operations
  async chatWithAgent(agentName: string, message: string): Promise<AgentServiceResponse<any>> {
    console.log(`💬 Sending message to ${agentName}:`, message);

    return this.makeRequest<any>(`/api/agents/${agentName}/chat`, {
      method: 'POST',
      body: JSON.stringify({ message }),
    });
  }

  async assignTaskToAgent(agentName: string, taskId: string): Promise<AgentServiceResponse<any>> {
    console.log(`📋 Assigning task ${taskId} to agent ${agentName}`);

    return this.makeRequest<any>(`/api/agents/tasks/${taskId}`, {
      method: 'PUT',
      body: JSON.stringify({ assigned_agent: agentName }),
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
      case 'marketing_automation':
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
      if (agent.metrics.tasks_processed !== undefined) {
        metrics.push(`Tasks: ${agent.metrics.tasks_processed}`);
      }
      if (agent.metrics.content_themes !== undefined) {
        metrics.push(`Themes: ${agent.metrics.content_themes}`);
      }
      if (agent.metrics.twitter_integration) {
        metrics.push(`Twitter: ${agent.metrics.twitter_integration}`);
      }
      if (agent.metrics.llm_factory) {
        metrics.push(`LLM: ${agent.metrics.llm_factory}`);
      }
      if (agent.metrics.last_activity) {
        metrics.push(`Last: ${agent.metrics.last_activity}`);
      }
    }

    return metrics;
  }
}

export const agentService = new AgentService();
export default AgentService;