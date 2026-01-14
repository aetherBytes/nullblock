import type {
  ApiKeyListResponse,
  ApiKeySingleResponse,
  CreateApiKeyRequest,
} from '../../types/user';

const EREBUS_BASE_URL = import.meta.env.VITE_EREBUS_API_URL || 'http://localhost:3000';

export const apiKeysService = {
  async listKeys(userId: string): Promise<ApiKeyListResponse> {
    try {
      const response = await fetch(`${EREBUS_BASE_URL}/api/users/${userId}/api-keys`, {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));

        return {
          success: false,
          error: errorData.error || `HTTP ${response.status}: ${response.statusText}`,
        };
      }

      return await response.json();
    } catch (error) {
      console.error('List API keys error:', error);

      return {
        success: false,
        error: error instanceof Error ? error.message : 'Network error fetching API keys',
      };
    }
  },

  async createKey(userId: string, request: CreateApiKeyRequest): Promise<ApiKeySingleResponse> {
    try {
      const response = await fetch(`${EREBUS_BASE_URL}/api/users/${userId}/api-keys`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(request),
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));

        return {
          success: false,
          error: errorData.error || `HTTP ${response.status}: ${response.statusText}`,
        };
      }

      return await response.json();
    } catch (error) {
      console.error('Create API key error:', error);

      return {
        success: false,
        error: error instanceof Error ? error.message : 'Network error creating API key',
      };
    }
  },

  async deleteKey(userId: string, keyId: string): Promise<ApiKeySingleResponse> {
    try {
      const response = await fetch(`${EREBUS_BASE_URL}/api/users/${userId}/api-keys/${keyId}`, {
        method: 'DELETE',
        headers: {
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));

        return {
          success: false,
          error: errorData.error || `HTTP ${response.status}: ${response.statusText}`,
        };
      }

      return await response.json();
    } catch (error) {
      console.error('Delete API key error:', error);

      return {
        success: false,
        error: error instanceof Error ? error.message : 'Network error deleting API key',
      };
    }
  },
};
