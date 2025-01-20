// src/services/api.tsx

import { Connection, PublicKey } from '@solana/web3.js';

interface WalletData {
  balance: number;
  address: string;
  transactionCount: number;
  // Add other properties as needed
}

/**
 * Fetches wallet data from Solana network for a given public key.
 * @param publicKey - The wallet's public key as a string.
 * @returns A promise that resolves to wallet data.
 * @throws If there's an error fetching the data.
 */
const fetchWalletData = async (publicKey: string): Promise<WalletData> => {
  try {
    // Use environment variable for network endpoint
    const connection = new Connection(process.env.REACT_APP_SOLANA_RPC_URL || 'https://api.mainnet-beta.solana.com', 'confirmed');

    // Fetch balance in lamports and convert to SOL
    const solBalance = await connection.getBalance(new PublicKey(publicKey));
    const balance = solBalance / 1e9; // Convert lamports to SOL

    // Fetch transaction count
    const transactionCount = await connection.getTransactionCount(new PublicKey(publicKey));

    return {
      balance: balance,
      address: publicKey,
      transactionCount: transactionCount,
    };
  } catch (error) {
    console.error('Failed to fetch wallet data:', error);
    throw error; // Re-throw the error for the caller to handle
  }
};

export { fetchWalletData };
