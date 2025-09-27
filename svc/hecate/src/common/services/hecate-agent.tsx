/**
 * Hecate Agent Service
 * 
 * Frontend service for communicating with the Hecate agent backend.
 * Handles chat messages, model information, and agent status.
 */

interface ChatMessage {
  id: string;
  timestamp: Date;
  sender: 'user' | 'hecate';
  message: string;
  type: 'text' | 'system' | 'error';
  model_used?: string;
  metadata?: any;
}

interface HecateResponse {
  content: string;
  model_used: string;
  latency_ms: number;
  confidence_score: number;
  metadata: {
    personality: string;
    cost_estimate: number;
    token_usage: any;
    finish_reason: string;
    conversation_length: number;
  };
}

interface ModelStatus {
  status: string;
  current_model: string | null;
  health: any;
  stats: any;
  conversation_length: number;
}

class HecateAgentService {
  private erebusUrl: string;
  private isConnected: boolean = false;

  constructor(erebusUrl: string = import.meta.env.VITE_EREBUS_API_URL || 'http://localhost:3000') {
    this.erebusUrl = erebusUrl;
  }

  /**
   * Initialize connection to Hecate agent via Erebus
   */
  async connect(): Promise<boolean> {
    try {
      const response = await fetch(`${this.erebusUrl}/api/agents/health`);
      this.isConnected = response.ok;
      return this.isConnected;
    } catch (error) {
      console.error('Failed to connect to Hecate agent via Erebus:', error);
      this.isConnected = false;
      return false;
    }
  }

  /**
   * Send a chat message to Hecate agent
   */
  async sendMessage(
    message: string, 
    userContext?: { 
      wallet_address?: string; 
      wallet_type?: string; 
      session_time?: string; 
    }
  ): Promise<HecateResponse> {
    if (!this.isConnected) {
      throw new Error('Not connected to Hecate agent. Call connect() first.');
    }

    try {
      // Create AbortController for timeout
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), 300000); // 5 minutes timeout for thinking models

      const response = await fetch(`${this.erebusUrl}/api/agents/hecate/chat`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          message,
          user_context: userContext
        }),
        signal: controller.signal
      });

      clearTimeout(timeoutId);

      if (!response.ok) {
        // Try to get the actual error message from the response body
        let errorMessage = `HTTP error! status: ${response.status}`;
        try {
          const errorData = await response.json();
          if (errorData.message) {
            errorMessage = errorData.message;
          } else if (errorData.error) {
            errorMessage = errorData.error;
          } else if (errorData.details) {
            errorMessage = errorData.details;
          }
        } catch {
          // If we can't parse the error response, use the default message
        }
        throw new Error(errorMessage);
      }

      const data = await response.json();
      return data;
    } catch (error) {
      console.error('Failed to send message to Hecate agent:', error);
      if (error instanceof DOMException && error.name === 'AbortError') {
        throw new Error('Request timed out - the model may be thinking too long or server overloaded');
      }
      throw error;
    }
  }

  /**
   * Get current model status and information
   */
  async getModelStatus(): Promise<ModelStatus> {
    if (!this.isConnected) {
      throw new Error('Not connected to Hecate agent. Call connect() first.');
    }

    try {
      const response = await fetch(`${this.erebusUrl}/api/agents/hecate/status`);
      
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const data = await response.json();
      return data;
    } catch (error) {
      console.error('Failed to get model status:', error);
      throw error;
    }
  }

  /**
   * Get available models
   */
  async getAvailableModels(): Promise<{ models: any[], current_model: string | null }> {
    if (!this.isConnected) {
      throw new Error('Not connected to Hecate agent. Call connect() first.');
    }

    try {
      console.log('🔄 Requesting available models from backend via Erebus:', `${this.erebusUrl}/api/agents/hecate/available-models`);
      const response = await fetch(`${this.erebusUrl}/api/agents/hecate/available-models`);
      
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const data = await response.json();
      console.log('✅ Received models from backend:', {
        totalModels: data.models?.length || 0,
        currentModel: data.current_model,
        sampleModels: data.models?.slice(0, 3).map((m: any) => ({
          id: m.id,
          name: m.display_name || m.name,
          created: m.created,
          created_at: m.created_at
        }))
      });
      return data;
    } catch (error) {
      console.error('Failed to get available models:', error);
      throw error;
    }
  }

  /**
   * Set preferred model
   */
  async setModel(modelName: string): Promise<boolean> {
    if (!this.isConnected) {
      throw new Error('Not connected to Hecate agent. Call connect() first.');
    }

    try {
      const response = await fetch(`${this.erebusUrl}/api/agents/hecate/set-model`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ model_name: modelName }),
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const data = await response.json();
      return data.success || false;
    } catch (error) {
      console.error('Failed to set model:', error);
      throw error;
    }
  }



  /**
   * Set agent personality
   */
  async setPersonality(personality: 'helpful_cyberpunk' | 'technical_expert' | 'concise_assistant'): Promise<boolean> {
    if (!this.isConnected) {
      throw new Error('Not connected to Hecate agent. Call connect() first.');
    }

    try {
      const response = await fetch(`${this.erebusUrl}/api/agents/hecate/personality`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ personality }),
      });

      return response.ok;
    } catch (error) {
      console.error('Failed to set personality:', error);
      return false;
    }
  }

  /**
   * Clear conversation history
   */
  async clearConversation(): Promise<boolean> {
    if (!this.isConnected) {
      throw new Error('Not connected to Hecate agent. Call connect() first.');
    }

    try {
      const response = await fetch(`${this.erebusUrl}/api/agents/hecate/clear`, {
        method: 'POST',
      });

      return response.ok;
    } catch (error) {
      console.error('Failed to clear conversation:', error);
      return false;
    }
  }

  /**
   * Get conversation history
   */
  async getConversationHistory(): Promise<ChatMessage[]> {
    if (!this.isConnected) {
      throw new Error('Not connected to Hecate agent. Call connect() first.');
    }

    try {
      const response = await fetch(`${this.erebusUrl}/api/agents/hecate/history`);
      
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const data = await response.json();
      
      // Convert backend format to frontend ChatMessage format
      return data.map((msg: any) => ({
        id: `${msg.timestamp}-${msg.role}`,
        timestamp: new Date(msg.timestamp),
        sender: msg.role === 'user' ? 'user' : 'hecate',
        message: msg.content,
        type: msg.role === 'system' ? 'system' : 'text',
        model_used: msg.model_used,
        metadata: msg.metadata
      }));
    } catch (error) {
      console.error('Failed to get conversation history:', error);
      throw error;
    }
  }

  /**
   * Get detailed model information
   */
  async getModelInfo(modelName?: string): Promise<any> {
    if (!this.isConnected) {
      throw new Error('Not connected to Hecate agent. Call connect() first.');
    }

    try {
      const params = modelName ? `?model_name=${encodeURIComponent(modelName)}` : '';
      const response = await fetch(`${this.erebusUrl}/api/agents/hecate/model-info${params}`);
      
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const data = await response.json();
      return data;
    } catch (error) {
      console.error('Failed to get model info:', error);
      throw error;
    }
  }

  /**
   * Search models
   */
  async searchModels(query: string, limit: number = 20): Promise<{ results: any[] }> {
    if (!this.isConnected) {
      throw new Error('Not connected to Hecate agent. Call connect() first.');
    }

    try {
      const response = await fetch(`${this.erebusUrl}/api/agents/hecate/search-models?q=${encodeURIComponent(query)}&limit=${limit}`);
      
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const data = await response.json();
      return data;
    } catch (error) {
      console.error('Failed to search models:', error);
      throw error;
    }
  }

  /**
   * Check if service is connected
   */
  isAgentConnected(): boolean {
    return this.isConnected;
  }

  /**
   * Get connection status
   */
  getConnectionStatus(): { connected: boolean; url: string } {
    return {
      connected: this.isConnected,
      url: this.erebusUrl
    };
  }
}

// Create singleton instance
export const hecateAgent = new HecateAgentService();

export default HecateAgentService;
export type { ChatMessage, HecateResponse, ModelStatus };