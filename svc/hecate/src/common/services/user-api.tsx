import type { UserLookupRequest, UserLookupResponse } from '../../types/user';

const EREBUS_BASE_URL = import.meta.env.VITE_EREBUS_API_URL || 'http://localhost:3000';

export const userApi = {
  async lookupUser(sourceIdentifier: string, network: string): Promise<UserLookupResponse> {
    try {
      const response = await fetch(`${EREBUS_BASE_URL}/api/users/lookup`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          source_identifier: sourceIdentifier,
          network,
        } as UserLookupRequest),
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
      console.error('User lookup error:', error);

      return {
        success: false,
        error: error instanceof Error ? error.message : 'Network error during user lookup',
      };
    }
  },

  async getUserById(userId: string): Promise<UserLookupResponse> {
    try {
      const response = await fetch(`${EREBUS_BASE_URL}/api/users/${userId}`, {
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
      console.error('Get user by ID error:', error);

      return {
        success: false,
        error: error instanceof Error ? error.message : 'Network error fetching user',
      };
    }
  },
};
