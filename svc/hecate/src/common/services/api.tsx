import axios from 'axios';

interface WalletData {
  balance: number;
  address: string;
  transactionCount: number;
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

export { fetchWalletData };
