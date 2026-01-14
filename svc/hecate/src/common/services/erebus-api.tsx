import axios from 'axios';

// Erebus API base URL
const EREBUS_API_BASE_URL = import.meta.env.VITE_EREBUS_API_URL || 'http://localhost:3000';

// New interfaces for backend-driven wallet interaction
interface WalletDetectionRequest {
  user_agent?: string;
  available_wallets: string[];
}

interface DetectedWallet {
  id: string;
  name: string;
  description: string;
  icon: string;
  is_available: boolean;
  install_url?: string;
}

interface InstallPrompt {
  wallet_id: string;
  wallet_name: string;
  install_url: string;
  description: string;
}

interface WalletDetectionResponse {
  available_wallets: DetectedWallet[];
  recommended_wallet?: string;
  install_prompts: InstallPrompt[];
}

interface WalletConnectionRequest {
  wallet_type: string;
  wallet_address: string;
  public_key?: string;
}

interface WalletConnectionResponse {
  success: boolean;
  session_token?: string;
  wallet_info?: WalletInfo;
  message: string;
  next_step?: string;
}

interface WalletStatusResponse {
  connected: boolean;
  wallet_type?: string;
  wallet_address?: string;
  session_valid: boolean;
  session_expires_at?: string;
}

interface WalletInfo {
  id: string;
  name: string;
  description: string;
  icon: string;
}

interface WalletListResponse {
  supported_wallets: WalletInfo[];
}

interface WalletChallengeRequest {
  wallet_address: string;
  wallet_type: string;
}

interface WalletChallengeResponse {
  challenge_id: string;
  message: string;
  wallet_address: string;
}

interface WalletVerifyRequest {
  challenge_id: string;
  signature: string;
  wallet_address: string;
}

interface WalletVerifyResponse {
  success: boolean;
  session_token?: string;
  message: string;
}

// Get supported wallets from Erebus
export const getSupportedWallets = async (): Promise<WalletListResponse> => {
  try {
    const response = await axios.get<WalletListResponse>(`${EREBUS_API_BASE_URL}/api/wallets`);

    console.log('Fetched supported wallets:', response.data);

    return response.data;
  } catch (error) {
    console.error('Failed to fetch supported wallets:', error);
    throw error;
  }
};

// Create wallet authentication challenge
export const createWalletChallenge = async (
  walletAddress: string,
  walletType: string,
): Promise<WalletChallengeResponse> => {
  try {
    const request: WalletChallengeRequest = {
      wallet_address: walletAddress,
      wallet_type: walletType,
    };

    console.log('Creating wallet challenge:', request);
    const response = await axios.post<WalletChallengeResponse>(
      `${EREBUS_API_BASE_URL}/api/wallets/challenge`,
      request,
    );

    console.log('Wallet challenge created:', response.data);

    return response.data;
  } catch (error) {
    console.error('Failed to create wallet challenge:', error);
    throw error;
  }
};

// Verify wallet signature
export const verifyWalletSignature = async (
  challengeId: string,
  signature: string,
  walletAddress: string,
): Promise<WalletVerifyResponse> => {
  try {
    const request: WalletVerifyRequest = {
      challenge_id: challengeId,
      signature,
      wallet_address: walletAddress,
    };

    console.log('Verifying wallet signature:', request);
    const response = await axios.post<WalletVerifyResponse>(
      `${EREBUS_API_BASE_URL}/api/wallets/verify`,
      request,
    );

    console.log('Wallet verification response:', response.data);

    return response.data;
  } catch (error) {
    console.error('Failed to verify wallet signature:', error);
    throw error;
  }
};

// Register wallet user with Erebus
interface UserRegistrationRequest {
  source_identifier: string;
  source_type: {
    type: string;
    network: string;
  };
  session_token?: string;
}

interface UserRegistrationResponse {
  success: boolean;
  user?: {
    user_id: string;
    source_identifier: string;
    created_at: string;
  };
  message?: string;
}

export const registerWalletUser = async (
  walletAddress: string,
  sessionToken: string,
  network: string = 'solana',
): Promise<UserRegistrationResponse> => {
  try {
    const request: UserRegistrationRequest = {
      source_identifier: walletAddress,
      source_type: {
        type: 'web3_wallet',
        network,
      },
      session_token: sessionToken,
    };

    console.log('Registering wallet user:', request);
    const response = await axios.post<UserRegistrationResponse>(
      `${EREBUS_API_BASE_URL}/api/users/register`,
      request,
    );

    console.log('User registration response:', response.data);

    return response.data;
  } catch (error: any) {
    console.error('Failed to register wallet user:', error);
    console.error('Registration error details:', error.response?.data || error.message);
    throw error;
  }
};

// Check Erebus health
export const checkErebusHealth = async (): Promise<any> => {
  try {
    const response = await axios.get(`${EREBUS_API_BASE_URL}/health`);

    console.log('Erebus health check:', response.data);

    return response.data;
  } catch (error) {
    console.error('Erebus health check failed:', error);
    throw error;
  }
};

// Backend-driven wallet detection
export const detectWallets = async (
  availableWallets: string[],
): Promise<WalletDetectionResponse> => {
  try {
    const request: WalletDetectionRequest = {
      user_agent: navigator.userAgent,
      available_wallets: availableWallets,
    };

    console.log('Detecting wallets:', request);
    const response = await axios.post<WalletDetectionResponse>(
      `${EREBUS_API_BASE_URL}/api/wallets/detect`,
      request,
    );

    console.log('Wallet detection response:', response.data);

    return response.data;
  } catch (error) {
    console.error('Failed to detect wallets:', error);
    throw error;
  }
};

// Backend-driven wallet connection initiation
export const initiateWalletConnection = async (
  walletType: string,
  walletAddress: string,
  publicKey?: string,
): Promise<WalletConnectionResponse> => {
  try {
    const request: WalletConnectionRequest = {
      wallet_type: walletType,
      wallet_address: walletAddress,
      public_key: publicKey,
    };

    console.log('Initiating wallet connection:', request);
    const response = await axios.post<WalletConnectionResponse>(
      `${EREBUS_API_BASE_URL}/api/wallets/connect`,
      request,
    );

    console.log('Wallet connection response:', response.data);

    return response.data;
  } catch (error) {
    console.error('Failed to initiate wallet connection:', error);
    throw error;
  }
};

// Get wallet status from backend
export const getWalletStatus = async (): Promise<WalletStatusResponse> => {
  try {
    const response = await axios.get<WalletStatusResponse>(
      `${EREBUS_API_BASE_URL}/api/wallets/status`,
    );

    console.log('Wallet status:', response.data);

    return response.data;
  } catch (error) {
    console.error('Failed to get wallet status:', error);
    throw error;
  }
};
