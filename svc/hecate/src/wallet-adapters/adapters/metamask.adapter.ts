import { BaseWalletAdapter } from '../base-adapter';
import type { WalletInfo, ConnectionResult, SignatureResult, EthereumProvider } from '../types';
import { ChainType } from '../types';

export class MetaMaskAdapter extends BaseWalletAdapter {
  readonly id = 'metamask';

  readonly info: WalletInfo = {
    id: 'metamask',
    name: 'MetaMask',
    description: 'Ethereum Wallet - Industry standard for Ethereum dApps',
    icon: 'https://raw.githubusercontent.com/MetaMask/brand-resources/master/SVG/SVG_MetaMask_Icon_Color.svg',
    supportedChains: [ChainType.EVM],
    installUrl: 'https://metamask.io/',
  };

  isInstalled(): boolean {
    if (typeof window === 'undefined') {
      return false;
    }

    return this.getMetaMaskProvider() !== null;
  }

  getProvider(chain: ChainType): EthereumProvider | null {
    if (chain !== ChainType.EVM) {
      return null;
    }

    return this.getMetaMaskProvider();
  }

  private getMetaMaskProvider(): EthereumProvider | null {
    if (typeof window === 'undefined' || !window.ethereum) {
      return null;
    }

    // Check for multiple providers (e.g., when Coinbase and MetaMask are both installed)
    if (window.ethereum.providers?.length) {
      const metaMaskProvider = window.ethereum.providers.find((p) => p.isMetaMask);

      if (metaMaskProvider) {
        return metaMaskProvider;
      }
    }

    // Single provider
    if (window.ethereum.isMetaMask) {
      return window.ethereum;
    }

    return null;
  }

  async connect(chain: ChainType = ChainType.EVM): Promise<ConnectionResult> {
    if (chain !== ChainType.EVM) {
      return {
        success: false,
        chain,
        error: 'MetaMask only supports EVM chains',
      };
    }

    const provider = this.getMetaMaskProvider();

    if (!provider) {
      return {
        success: false,
        chain,
        error: 'MetaMask not installed',
      };
    }

    try {
      // First check for existing accounts
      let accounts = (await provider.request({ method: 'eth_accounts' })) as string[];

      // If no accounts, request connection
      if (!accounts.length) {
        accounts = (await provider.request({
          method: 'eth_requestAccounts',
        })) as string[];
      }

      if (!accounts.length) {
        return {
          success: false,
          chain,
          error: 'No accounts available',
        };
      }

      return {
        success: true,
        address: accounts[0],
        chain,
      };
    } catch (error: unknown) {
      const err = error as { code?: number; message?: string };

      if (err.code === -32002) {
        return {
          success: false,
          chain,
          error: 'Connection request pending. Please check MetaMask.',
        };
      }

      if (err.code === 4001) {
        return {
          success: false,
          chain,
          error: 'Connection rejected by user',
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
    // MetaMask doesn't have a disconnect method
    // The connection persists until the user disconnects from MetaMask itself
  }

  async signMessage(message: string): Promise<SignatureResult> {
    const provider = this.getMetaMaskProvider();

    if (!provider) {
      return { success: false, error: 'MetaMask not installed' };
    }

    if (!provider.selectedAddress) {
      return { success: false, error: 'Not connected to MetaMask' };
    }

    try {
      const signature = (await provider.request({
        method: 'personal_sign',
        params: [message, provider.selectedAddress],
      })) as string;

      return { success: true, signature };
    } catch (error: unknown) {
      const err = error as { code?: number; message?: string };

      if (err.code === 4001) {
        return { success: false, error: 'Signature rejected by user' };
      }

      return { success: false, error: err.message || 'Signing failed' };
    }
  }
}
