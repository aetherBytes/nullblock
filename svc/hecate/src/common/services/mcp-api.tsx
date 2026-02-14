import type { AxiosResponse } from 'axios';
import axios from 'axios';

// MCP Server API base URL - now points to the new MCP server
const MCP_API_BASE_URL = import.meta.env.VITE_EREBUS_API_URL || 'http://localhost:3000';

// MCP API Interfaces
interface MCPAuthChallenge {
  message: string;
  nonce: string;
  expires_at: string;
}

interface MCPAuthResponse {
  success: boolean;
  session_id?: string;
  message: string;
}

interface MCPHealthResponse {
  status: string;
  timestamp: string;
  services: {
    web3: boolean;
    mev_protection: boolean;
    context_storage: boolean;
    security: boolean;
  };
}

interface MCPUserContext {
  wallet_address: string;
  preferences: Record<string, any>;
  agent_settings: Record<string, any>;
  trading_profile: Record<string, any>;
  risk_profile: Record<string, any>;
  created_at: string;
  updated_at: string;
  version: number;
}

interface MCPWalletBalance {
  address: string;
  balance: number;
}

interface MCPTradingCommandResponse {
  success: boolean;
  result: any;
  message: string;
  protected: boolean;
}

// Session management
let currentSession: string | null = null;

const getAuthHeaders = () => {
  if (!currentSession) {
    throw new Error('No active session. Please authenticate first.');
  }

  return {
    Authorization: `Bearer ${currentSession}`,
    'Content-Type': 'application/json',
  };
};

// Authentication functions
export const createAuthChallenge = async (walletAddress: string): Promise<MCPAuthChallenge> => {
  try {
    const response: AxiosResponse<MCPAuthChallenge> = await axios.post(
      `${MCP_API_BASE_URL}/auth/challenge`,
      {
        wallet_address: walletAddress,
      },
    );

    return response.data;
  } catch (error) {
    console.error('Failed to create auth challenge:', error);
    throw error;
  }
};

export const verifyAuthChallenge = async (
  walletAddress: string,
  signature: string,
  provider: string = 'phantom',
): Promise<MCPAuthResponse> => {
  try {
    const response: AxiosResponse<MCPAuthResponse> = await axios.post(
      `${MCP_API_BASE_URL}/auth/verify`,
      {
        wallet_address: walletAddress,
        signature,
        provider,
      },
    );

    if (response.data.success && response.data.session_id) {
      currentSession = response.data.session_id;
      // Store session in localStorage for persistence
      localStorage.setItem('mcp_session_id', response.data.session_id);
    }

    return response.data;
  } catch (error) {
    console.error('Failed to verify auth challenge:', error);
    throw error;
  }
};

export const restoreSession = (): boolean => {
  const storedSession = localStorage.getItem('mcp_session_id');

  if (storedSession) {
    currentSession = storedSession;

    return true;
  }

  return false;
};

export const clearSession = (): void => {
  currentSession = null;
  localStorage.removeItem('mcp_session_id');
};

// MCP API functions
export const checkMCPHealth = async (): Promise<MCPHealthResponse> => {
  try {
    const response: AxiosResponse<MCPHealthResponse> = await axios.get(
      `${MCP_API_BASE_URL}/health`,
    );

    return response.data;
  } catch (error) {
    console.error('Failed to check MCP health:', error);
    throw error;
  }
};

export const getUserContext = async (): Promise<MCPUserContext> => {
  try {
    const response: AxiosResponse<MCPUserContext> = await axios.get(`${MCP_API_BASE_URL}/context`, {
      headers: getAuthHeaders(),
    });

    return response.data;
  } catch (error) {
    console.error('Failed to get user context:', error);
    throw error;
  }
};

export const updateUserContext = async (
  updates: Record<string, any>,
): Promise<{ success: boolean; message: string }> => {
  try {
    const response = await axios.post(
      `${MCP_API_BASE_URL}/context/update`,
      {
        updates,
      },
      {
        headers: getAuthHeaders(),
      },
    );

    return response.data;
  } catch (error) {
    console.error('Failed to update user context:', error);
    throw error;
  }
};

export const getWalletBalance = async (): Promise<MCPWalletBalance> => {
  try {
    const response: AxiosResponse<MCPWalletBalance> = await axios.get(
      `${MCP_API_BASE_URL}/wallet/balance`,
      {
        headers: getAuthHeaders(),
      },
    );

    return response.data;
  } catch (error) {
    console.error('Failed to get wallet balance:', error);
    throw error;
  }
};

export const executeTradingCommand = async (
  command: string,
  parameters: Record<string, any> = {},
): Promise<MCPTradingCommandResponse> => {
  try {
    const response: AxiosResponse<MCPTradingCommandResponse> = await axios.post(
      `${MCP_API_BASE_URL}/trading/command`,
      {
        command,
        parameters,
      },
      {
        headers: getAuthHeaders(),
      },
    );

    return response.data;
  } catch (error) {
    console.error('Failed to execute trading command:', error);
    throw error;
  }
};

// Arbitrage-specific functions
export const findArbitrageOpportunities = async (
  params: {
    min_profit_percentage?: number;
    max_trade_amount?: number;
  } = {},
): Promise<MCPTradingCommandResponse> =>
  executeTradingCommand('arbitrage', {
    action: 'find_opportunities',
    ...params,
  });

export const executeArbitrageTrade = async (params: {
  opportunity_id: string;
  trade_amount: number;
  max_slippage?: number;
}): Promise<MCPTradingCommandResponse> =>
  executeTradingCommand('arbitrage', {
    action: 'execute',
    ...params,
  });

export const getArbitrageHistory = async (): Promise<MCPTradingCommandResponse> =>
  executeTradingCommand('arbitrage', {
    action: 'get_history',
  });

export const getArbitrageMetrics = async (): Promise<MCPTradingCommandResponse> =>
  executeTradingCommand('arbitrage', {
    action: 'get_metrics',
  });

// Portfolio management
export const rebalancePortfolio = async (params: {
  strategy?: string;
  risk_tolerance?: string;
}): Promise<MCPTradingCommandResponse> => executeTradingCommand('rebalance', params);

export const updateTradingSettings = async (settings: {
  min_profit_threshold?: number;
  max_trade_amount?: number;
  risk_tolerance?: string;
  preferred_dexes?: string[];
  enable_mev_protection?: boolean;
}): Promise<MCPTradingCommandResponse> => executeTradingCommand('set', settings);

// Utility functions for UI integration
export const isAuthenticated = (): boolean => currentSession !== null;

export const getSessionId = (): string | null => currentSession;

// Error handling utility
export const handleMCPError = (error: any): string => {
  if (error.response?.status === 401) {
    clearSession();

    return 'Authentication expired. Please reconnect your wallet.';
  }

  if (error.response?.status === 403) {
    return 'Access denied. Input may have been blocked for security reasons.';
  }

  if (error.response?.data?.message) {
    return error.response.data.message;
  }

  return 'An unexpected error occurred. Please try again.';
};

// Migration helper - maps old API calls to new MCP calls
export const migrateToMCP = {
  // Map old wallet data fetch to new MCP calls
  fetchWalletData: async (_publicKey: string) => {
    if (!isAuthenticated()) {
      throw new Error('MCP authentication required');
    }

    const balance = await getWalletBalance();

    return {
      balance: balance.balance,
      address: balance.address,
      transactionCount: 0, // MCP doesn't track this currently
    };
  },

  // Map old user profile to MCP context
  fetchUserProfile: async (_publicKey: string) => {
    if (!isAuthenticated()) {
      throw new Error('MCP authentication required');
    }

    const context = await getUserContext();

    return {
      balance: 0, // Will be fetched separately
      address: context.wallet_address,
      transaction_count: 0,
      risk_score: context.risk_profile.overall_score || 0.5,
      last_activity: context.updated_at,
      active_tokens: context.trading_profile.preferred_tokens || [],
    };
  },
};

export default {
  createAuthChallenge,
  verifyAuthChallenge,
  restoreSession,
  clearSession,
  checkMCPHealth,
  getUserContext,
  updateUserContext,
  getWalletBalance,
  executeTradingCommand,
  findArbitrageOpportunities,
  executeArbitrageTrade,
  getArbitrageHistory,
  getArbitrageMetrics,
  rebalancePortfolio,
  updateTradingSettings,
  isAuthenticated,
  getSessionId,
  handleMCPError,
  migrateToMCP,
};
