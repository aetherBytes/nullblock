import { BaseWalletAdapter } from '../base-adapter';
import {
  WalletInfo,
  ConnectionResult,
  SignatureResult,
  ChainType,
  EthereumProvider,
  SolanaProvider,
} from '../types';

export class BitgetAdapter extends BaseWalletAdapter {
  readonly id = 'bitget';

  readonly info: WalletInfo = {
    id: 'bitget',
    name: 'Bitget Wallet',
    description: 'Multi-chain wallet supporting EVM and Solana networks',
    icon: 'https://web3.bitget.com/favicon.ico',
    supportedChains: [ChainType.EVM, ChainType.SOLANA],
    installUrl: 'https://web3.bitget.com/',
  };

  // Track which chain we're connected to
  private connectedChain: ChainType | null = null;

  isInstalled(): boolean {
    if (typeof window === 'undefined') return false;
    return !!window.bitkeep;
  }

  getProvider(chain: ChainType): EthereumProvider | SolanaProvider | null {
    if (!window.bitkeep) return null;

    switch (chain) {
      case ChainType.EVM:
        return window.bitkeep.ethereum || null;
      case ChainType.SOLANA:
        return window.bitkeep.solana || null;
      default:
        return null;
    }
  }

  async connect(chain: ChainType = ChainType.EVM): Promise<ConnectionResult> {
    if (!this.isInstalled()) {
      return {
        success: false,
        chain,
        error: 'Bitget Wallet not installed',
      };
    }

    try {
      if (chain === ChainType.EVM) {
        return await this.connectEvm();
      } else if (chain === ChainType.SOLANA) {
        return await this.connectSolana();
      }

      return {
        success: false,
        chain,
        error: `Unsupported chain: ${chain}`,
      };
    } catch (error: unknown) {
      const err = error as { message?: string };
      return {
        success: false,
        chain,
        error: err.message || 'Connection failed',
      };
    }
  }

  private async connectEvm(): Promise<ConnectionResult> {
    const provider = this.getProvider(ChainType.EVM) as EthereumProvider | null;
    if (!provider) {
      return { success: false, chain: ChainType.EVM, error: 'EVM provider not available' };
    }

    try {
      const accounts = (await provider.request({
        method: 'eth_requestAccounts',
      })) as string[];

      if (!accounts.length) {
        return { success: false, chain: ChainType.EVM, error: 'No accounts available' };
      }

      this.connectedChain = ChainType.EVM;

      return {
        success: true,
        address: accounts[0],
        chain: ChainType.EVM,
      };
    } catch (error: unknown) {
      const err = error as { code?: number; message?: string };

      if (err.code === 4001) {
        return { success: false, chain: ChainType.EVM, error: 'Connection rejected by user' };
      }

      return { success: false, chain: ChainType.EVM, error: err.message || 'Connection failed' };
    }
  }

  private async connectSolana(): Promise<ConnectionResult> {
    const provider = this.getProvider(ChainType.SOLANA) as SolanaProvider | null;
    if (!provider) {
      return { success: false, chain: ChainType.SOLANA, error: 'Solana provider not available' };
    }

    try {
      const response = await provider.connect();
      const address = response.publicKey.toString();

      this.connectedChain = ChainType.SOLANA;

      return {
        success: true,
        address,
        publicKey: address,
        chain: ChainType.SOLANA,
      };
    } catch (error: unknown) {
      const err = error as { message?: string };

      if (err.message?.includes('User rejected')) {
        return { success: false, chain: ChainType.SOLANA, error: 'Connection rejected by user' };
      }

      return {
        success: false,
        chain: ChainType.SOLANA,
        error: err.message || 'Connection failed',
      };
    }
  }

  async disconnect(): Promise<void> {
    try {
      // Disconnect from Solana if connected
      const solanaProvider = this.getProvider(ChainType.SOLANA) as SolanaProvider | null;
      if (solanaProvider?.disconnect) {
        await solanaProvider.disconnect();
      }
    } catch {
      // Ignore disconnect errors
    }

    this.connectedChain = null;
  }

  async signMessage(message: string): Promise<SignatureResult> {
    // Determine which provider to use based on connection or detect from context
    const evmProvider = this.getProvider(ChainType.EVM) as EthereumProvider | null;
    const solanaProvider = this.getProvider(ChainType.SOLANA) as SolanaProvider | null;

    try {
      // Try EVM first if we have an active address
      if (evmProvider?.selectedAddress) {
        const signature = (await evmProvider.request({
          method: 'personal_sign',
          params: [message, evmProvider.selectedAddress],
        })) as string;

        return { success: true, signature };
      }

      // Try Solana if connected
      if (solanaProvider?.isConnected) {
        const encodedMessage = this.encodeMessage(message);
        const { signature } = await solanaProvider.signMessage(encodedMessage);
        const signatureStr = this.bytesToString(signature);

        return { success: true, signature: signatureStr };
      }

      return { success: false, error: 'No active wallet connection' };
    } catch (error: unknown) {
      const err = error as { code?: number; message?: string };

      if (err.code === 4001 || err.message?.includes('User rejected')) {
        return { success: false, error: 'Signature rejected by user' };
      }

      return { success: false, error: err.message || 'Signing failed' };
    }
  }

  // Override to check both providers
  async isConnected(): Promise<boolean> {
    const evmProvider = this.getProvider(ChainType.EVM) as EthereumProvider | null;
    const solanaProvider = this.getProvider(ChainType.SOLANA) as SolanaProvider | null;

    return !!evmProvider?.selectedAddress || !!solanaProvider?.isConnected;
  }

  getConnectedChain(): ChainType | null {
    return this.connectedChain;
  }
}
