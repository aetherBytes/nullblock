import axios from 'axios';

// API base URL
const API_BASE_URL = import.meta.env.VITE_FAST_API_BACKEND_URL || 'http://localhost:8000';

interface WalletData {
  balance: number;
  address: string;
  transactionCount: number;
  username?: string; // Optional username field
}

interface UserProfileData {
  balance: number;
  address: string;
  transaction_count: number;
  risk_score: number;
  last_activity: string;
  active_tokens: string[];
  username?: string; // Add username field to match API response
}

interface AscentLevelData {
  level: number;
  name: string;
  description: string;
  progress: number;
  accolades: string[];
}

export interface MissionData {
  id: string;
  title: string;
  description: string;
  status: string;
  reward: string;
  x_url: string;
}

const fetchWalletData = async (publicKey: string): Promise<WalletData> => {
  try {
    // Use Vite's environment variable with a fallback
    const baseUrl = import.meta.env.VITE_FAST_API_BACKEND_URL || 'http://localhost:8000'; // Default if not set
    const url = `${baseUrl}/api/wallet/${publicKey}`;

    const response = await axios.get<WalletData>(url);

    if (response.status !== 200) {
      throw new Error(`Unexpected response status: ${response.status}`);
    }

    return response.data;
  } catch (error) {
    if (axios.isAxiosError(error)) {
      console.error('Failed to fetch wallet data from backend:', error.message);
    } else {
      console.error('Unexpected error:', error);
    }

    throw error;
  }
};

// Function to get user profile including username if available
const fetchUserProfile = async (publicKey: string): Promise<UserProfileData> => {
  try {
    // Use Vite's environment variable with a fallback
    const baseUrl = import.meta.env.VITE_FAST_API_BACKEND_URL || 'http://localhost:8000'; // Default if not set
    const url = `${baseUrl}/api/wallet/health/${publicKey}`;

    const response = await axios.get<UserProfileData>(url);

    if (response.status !== 200) {
      throw new Error(`Unexpected response status: ${response.status}`);
    }

    // Log the response to see what fields are available
    console.log('User profile data:', response.data);

    return response.data;
  } catch (error) {
    if (axios.isAxiosError(error)) {
      console.error('Failed to fetch user profile from backend:', error.message);
    } else {
      console.error('Unexpected error:', error);
    }

    throw error;
  }
};

// Function to get user's ascent level data
const fetchAscentLevel = async (publicKey: string): Promise<AscentLevelData> => {
  try {
    // For now, return mock data since the backend endpoint doesn't exist yet
    // In a real implementation, this would call the backend API
    return {
      level: 1,
      name: 'Net Dweller',
      description: 'A newcomer to the digital realm, exploring the boundaries of the network.',
      progress: 0.35,
      accolades: [
        'First Connection',
        'Wallet Initiated',
        'Basic Navigation',
        'Token Discovery',
        'Transaction Initiate',
        'Network Explorer',
        'Data Collector',
        'Interface Familiar',
      ],
    };
  } catch (error) {
    console.error('Failed to fetch ascent level:', error);
    throw error;
  }
};

export const fetchActiveMission = async (publicKey: string): Promise<MissionData> => {
  try {
    const response = await fetch(`${API_BASE_URL}/api/missions/${publicKey}`);

    if (!response.ok) {
      throw new Error('Failed to fetch active mission');
    }

    return await response.json();
  } catch (error) {
    console.error('Error fetching active mission:', error);

    // Return a default mission if the API call fails
    return {
      id: '1',
      title: 'Share on X',
      description: 'Share your Base Camp on X to earn GLIMMER',
      status: 'active',
      reward: 'TBD GLIMMER AIRDROP',
      x_url:
        'https://twitter.com/intent/tweet?text=Check%20out%20my%20Base%20Camp%20in%20the%20Nullblock%20universe!%20%40Nullblock_io%20%24GLIMMER',
    };
  }
};

export { fetchWalletData, fetchUserProfile, fetchAscentLevel };
