import type { WalletAdapter, WalletInfo, ConnectionResult, SignatureResult } from './types';
import { ChainType } from './types';

export abstract class BaseWalletAdapter implements WalletAdapter {
  abstract readonly id: string;
  abstract readonly info: WalletInfo;

  abstract isInstalled(): boolean;
  abstract connect(chain?: ChainType): Promise<ConnectionResult>;
  abstract disconnect(): Promise<void>;
  abstract signMessage(message: string): Promise<SignatureResult>;
  abstract getProvider(chain: ChainType): unknown;

  async isConnected(): Promise<boolean> {
    try {
      const evmProvider = this.getProvider(ChainType.EVM);
      const solanaProvider = this.getProvider(ChainType.SOLANA);

      if (evmProvider && typeof evmProvider === 'object' && 'selectedAddress' in evmProvider) {
        return Boolean((evmProvider as { selectedAddress?: string }).selectedAddress);
      }

      if (solanaProvider && typeof solanaProvider === 'object' && 'isConnected' in solanaProvider) {
        return Boolean((solanaProvider as { isConnected?: boolean }).isConnected);
      }

      return false;
    } catch {
      return false;
    }
  }

  detectChain(address: string): ChainType | null {
    if (this.isEvmAddress(address)) {
      return ChainType.EVM;
    }

    if (this.isSolanaAddress(address)) {
      return ChainType.SOLANA;
    }

    return null;
  }

  protected isEvmAddress(address: string): boolean {
    return /^0x[a-fA-F0-9]{40}$/.test(address);
  }

  protected isSolanaAddress(address: string): boolean {
    // Solana addresses are Base58 encoded, 32-44 characters
    // Valid Base58 alphabet (no 0, O, I, l)
    const base58Regex = /^[1-9A-HJ-NP-Za-km-z]{32,44}$/;

    return base58Regex.test(address);
  }

  protected encodeMessage(message: string): Uint8Array {
    return new TextEncoder().encode(message);
  }

  protected bytesToString(bytes: Uint8Array): string {
    return Array.from(bytes).toString();
  }
}
