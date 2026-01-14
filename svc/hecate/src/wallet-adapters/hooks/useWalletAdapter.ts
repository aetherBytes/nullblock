import { useState, useCallback, useEffect } from 'react';
import { createWalletChallenge, verifyWalletSignature } from '../../common/services/erebus-api';
import { walletRegistry } from '../registry';
import type { WalletAdapter, ConnectionResult, WalletInfo } from '../types';
import { ChainType } from '../types';

interface UseWalletAdapterReturn {
  // State
  isConnecting: boolean;
  error: string | null;
  connectedWallet: WalletAdapter | null;
  connectedAddress: string | null;
  connectedChain: ChainType | null;
  sessionToken: string | null;

  // Actions
  connect: (walletId: string, chain?: ChainType) => Promise<ConnectionResult>;
  disconnect: () => Promise<void>;
  clearError: () => void;

  // Wallet info
  getInstalledWallets: () => WalletAdapter[];
  getAllWallets: () => WalletAdapter[];
  getAllWalletsInfo: () => WalletInfo[];
  getWalletsForChain: (chain: ChainType) => WalletAdapter[];
}

const STORAGE_KEYS = {
  WALLET_PUBLIC_KEY: 'walletPublickey',
  WALLET_TYPE: 'walletType',
  WALLET_CHAIN: 'walletChain',
  SESSION_TOKEN: 'sessionToken',
  LAST_AUTH_TIME: 'lastAuthTime',
  HAS_SEEN_HUD: 'hasSeenHUD',
};

const SESSION_TIMEOUT_MS = 30 * 60 * 1000; // 30 minutes

export function useWalletAdapter(): UseWalletAdapterReturn {
  const [isConnecting, setIsConnecting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [connectedWallet, setConnectedWallet] = useState<WalletAdapter | null>(null);
  const [connectedAddress, setConnectedAddress] = useState<string | null>(null);
  const [connectedChain, setConnectedChain] = useState<ChainType | null>(null);
  const [sessionToken, setSessionToken] = useState<string | null>(null);

  // Restore session on mount
  useEffect(() => {
    const restoreSession = () => {
      try {
        const storedAddress = localStorage.getItem(STORAGE_KEYS.WALLET_PUBLIC_KEY);
        const storedWalletType = localStorage.getItem(STORAGE_KEYS.WALLET_TYPE);
        const storedChain = localStorage.getItem(STORAGE_KEYS.WALLET_CHAIN) as ChainType | null;
        const storedToken = localStorage.getItem(STORAGE_KEYS.SESSION_TOKEN);
        const lastAuthTime = localStorage.getItem(STORAGE_KEYS.LAST_AUTH_TIME);

        if (!storedAddress || !storedWalletType || !storedToken) {
          return;
        }

        // Check session expiry
        if (lastAuthTime) {
          const elapsed = Date.now() - Number.parseInt(lastAuthTime, 10);

          if (elapsed > SESSION_TIMEOUT_MS) {
            console.log('Session expired, clearing...');
            clearSession();

            return;
          }
        }

        // Restore adapter
        const adapter = walletRegistry.get(storedWalletType);

        if (!adapter) {
          console.warn(`Unknown wallet type: ${storedWalletType}`);

          return;
        }

        setConnectedWallet(adapter);
        setConnectedAddress(storedAddress);
        setConnectedChain(storedChain || adapter.detectChain(storedAddress));
        setSessionToken(storedToken);

        console.log(`Session restored for ${storedWalletType}: ${storedAddress}`);
      } catch (err) {
        console.error('Failed to restore session:', err);
      }
    };

    restoreSession();
  }, []);

  const clearSession = useCallback(() => {
    localStorage.removeItem(STORAGE_KEYS.WALLET_PUBLIC_KEY);
    localStorage.removeItem(STORAGE_KEYS.WALLET_TYPE);
    localStorage.removeItem(STORAGE_KEYS.WALLET_CHAIN);
    localStorage.removeItem(STORAGE_KEYS.SESSION_TOKEN);
    localStorage.removeItem(STORAGE_KEYS.LAST_AUTH_TIME);

    setConnectedWallet(null);
    setConnectedAddress(null);
    setConnectedChain(null);
    setSessionToken(null);
  }, []);

  const saveSession = useCallback(
    (address: string, walletId: string, chain: ChainType, token: string) => {
      localStorage.setItem(STORAGE_KEYS.WALLET_PUBLIC_KEY, address);
      localStorage.setItem(STORAGE_KEYS.WALLET_TYPE, walletId);
      localStorage.setItem(STORAGE_KEYS.WALLET_CHAIN, chain);
      localStorage.setItem(STORAGE_KEYS.SESSION_TOKEN, token);
      localStorage.setItem(STORAGE_KEYS.LAST_AUTH_TIME, Date.now().toString());
      localStorage.setItem(STORAGE_KEYS.HAS_SEEN_HUD, 'true');
    },
    [],
  );

  const connect = useCallback(
    async (walletId: string, chain?: ChainType): Promise<ConnectionResult> => {
      setIsConnecting(true);
      setError(null);

      try {
        // Get adapter
        const adapter = walletRegistry.get(walletId);

        if (!adapter) {
          throw new Error(`Unknown wallet: ${walletId}`);
        }

        if (!adapter.isInstalled()) {
          throw new Error(
            `${adapter.info.name} is not installed. Please install it and try again.`,
          );
        }

        // Connect to wallet
        const connectionResult = await adapter.connect(chain);

        if (!connectionResult.success) {
          throw new Error(connectionResult.error || 'Connection failed');
        }

        const address = connectionResult.address!;
        const connectedToChain = connectionResult.chain;

        console.log(`Connected to ${walletId} (${connectedToChain}): ${address}`);

        // Create challenge via backend
        const challengeResponse = await createWalletChallenge(address, walletId);

        console.log('Challenge created:', challengeResponse.challenge_id);

        // Sign challenge
        const signResult = await adapter.signMessage(challengeResponse.message);

        if (!signResult.success) {
          throw new Error(signResult.error || 'Signing failed');
        }

        console.log('Challenge signed, verifying...');

        // Verify signature via backend
        const verifyResponse = await verifyWalletSignature(
          challengeResponse.challenge_id,
          signResult.signature!,
          address,
        );

        if (!verifyResponse.success) {
          throw new Error(verifyResponse.message || 'Verification failed');
        }

        console.log('Signature verified, session created');

        // Update state
        setConnectedWallet(adapter);
        setConnectedAddress(address);
        setConnectedChain(connectedToChain);
        setSessionToken(verifyResponse.session_token || null);

        // Save session
        if (verifyResponse.session_token) {
          saveSession(address, walletId, connectedToChain, verifyResponse.session_token);
        }

        return connectionResult;
      } catch (err: unknown) {
        const errorMessage = err instanceof Error ? err.message : 'Unknown error';

        console.error('Wallet connection error:', errorMessage);
        setError(errorMessage);

        return {
          success: false,
          chain: chain || ChainType.EVM,
          error: errorMessage,
        };
      } finally {
        setIsConnecting(false);
      }
    },
    [saveSession],
  );

  const disconnect = useCallback(async () => {
    try {
      if (connectedWallet) {
        await connectedWallet.disconnect();
      }
    } catch (err) {
      console.warn('Disconnect error:', err);
    }

    clearSession();
    console.log('Disconnected');
  }, [connectedWallet, clearSession]);

  const clearError = useCallback(() => {
    setError(null);
  }, []);

  return {
    // State
    isConnecting,
    error,
    connectedWallet,
    connectedAddress,
    connectedChain,
    sessionToken,

    // Actions
    connect,
    disconnect,
    clearError,

    // Wallet info
    getInstalledWallets: () => walletRegistry.getInstalled(),
    getAllWallets: () => walletRegistry.getAll(),
    getAllWalletsInfo: () => walletRegistry.getAllInfo(),
    getWalletsForChain: (chain: ChainType) => walletRegistry.getForChain(chain),
  };
}
