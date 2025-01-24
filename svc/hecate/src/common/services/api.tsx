import axios from 'axios';

interface WalletData {
  balance: number;
  address: string;
  transactionCount: number;
  // Add other properties as needed
}

/**
 * Fetches wallet data from a backend FastAPI service for a given public key.
 * @param publicKey - The wallet's public key as a string.
 * @returns A promise that resolves to wallet data.
 * @throws If there's an error fetching the data from the server.
 */
const fetchWalletData = async (publicKey: string): Promise<WalletData> => {
  try {
    // Assuming your FastAPI backend is running on localhost:8000, change this URL as needed
    const response = await axios.get<WalletData>(`${process.env.REACT_APP_FAST_API_BACKEND_URL}/api/wallet/${publicKey}`);

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
    throw error; // Re-throw the error for the caller to handle
  }
};

export { fetchWalletData };
