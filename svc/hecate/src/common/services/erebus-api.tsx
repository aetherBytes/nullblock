import axios from 'axios';

// Erebus API base URL
const EREBUS_API_BASE_URL = 'http://localhost:3000';

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
  walletType: string
): Promise<WalletChallengeResponse> => {
  try {
    const request: WalletChallengeRequest = {
      wallet_address: walletAddress,
      wallet_type: walletType,
    };

    console.log('Creating wallet challenge:', request);
    const response = await axios.post<WalletChallengeResponse>(
      `${EREBUS_API_BASE_URL}/api/wallets/challenge`,
      request
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
  walletAddress: string
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
      request
    );

    console.log('Wallet verification response:', response.data);
    return response.data;
  } catch (error) {
    console.error('Failed to verify wallet signature:', error);
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