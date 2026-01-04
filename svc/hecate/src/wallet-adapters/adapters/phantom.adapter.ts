import { BaseWalletAdapter } from '../base-adapter';
import {
  WalletInfo,
  ConnectionResult,
  SignatureResult,
  ChainType,
  SolanaProvider,
} from '../types';

export class PhantomAdapter extends BaseWalletAdapter {
  readonly id = 'phantom';

  readonly info: WalletInfo = {
    id: 'phantom',
    name: 'Phantom',
    description: 'Solana Wallet - The #1 wallet for Solana DeFi and NFTs',
    icon: 'https://phantom.app/img/phantom-icon-purple.svg',
    supportedChains: [ChainType.SOLANA],
    installUrl: 'https://phantom.app/',
  };

  isInstalled(): boolean {
    if (typeof window === 'undefined') return false;
    return !!window.phantom?.solana;
  }

  getProvider(chain: ChainType): SolanaProvider | null {
    if (chain !== ChainType.SOLANA) return null;
    return window.phantom?.solana || null;
  }

  async connect(chain: ChainType = ChainType.SOLANA): Promise<ConnectionResult> {
    if (chain !== ChainType.SOLANA) {
      return {
        success: false,
        chain,
        error: 'Phantom only supports Solana',
      };
    }

    const provider = this.getProvider(ChainType.SOLANA);
    if (!provider) {
      return {
        success: false,
        chain,
        error: 'Phantom not installed',
      };
    }

    try {
      const response = await provider.connect();
      const address = response.publicKey.toString();

      return {
        success: true,
        address,
        publicKey: address,
        chain,
      };
    } catch (error: unknown) {
      const err = error as { message?: string; name?: string };

      if (err.message?.includes('User rejected')) {
        return {
          success: false,
          chain,
          error: 'Connection rejected by user',
        };
      }

      if (err.message?.includes('Unexpected error') || err.message?.includes('-32603')) {
        return {
          success: false,
          chain,
          error:
            'Phantom wallet is not responding. Try restarting your browser or re-enabling the extension.',
        };
      }

      if (err.name === 'WalletNotReadyError') {
        return {
          success: false,
          chain,
          error: 'Phantom wallet not ready. Please install or unlock it.',
        };
      }

      return {
        success: false,
        chain,
        error: err.message || 'Connection failed',
      };
    }
  }

  async disconnect(): Promise<void> {
    const provider = this.getProvider(ChainType.SOLANA);
    if (provider?.disconnect) {
      try {
        await provider.disconnect();
      } catch {
        // Ignore disconnect errors
      }
    }
  }

  async signMessage(message: string): Promise<SignatureResult> {
    const provider = this.getProvider(ChainType.SOLANA);
    if (!provider) {
      return { success: false, error: 'Phantom not installed' };
    }

    if (!provider.isConnected) {
      return { success: false, error: 'Not connected to Phantom' };
    }

    try {
      const encodedMessage = this.encodeMessage(message);
      const { signature } = await provider.signMessage(encodedMessage, 'utf8');

      // Convert signature bytes to string format expected by backend
      const signatureStr = this.bytesToString(signature);

      return { success: true, signature: signatureStr };
    } catch (error: unknown) {
      const err = error as { message?: string };

      if (err.message?.includes('User rejected')) {
        return { success: false, error: 'Signature rejected by user' };
      }

      return { success: false, error: err.message || 'Signing failed' };
    }
  }
}
