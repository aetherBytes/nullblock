import { BitgetAdapter } from './adapters/bitget.adapter';
import { MetaMaskAdapter } from './adapters/metamask.adapter';
import { PhantomAdapter } from './adapters/phantom.adapter';
import type { WalletAdapter, WalletInfo, ChainType } from './types';

class WalletAdapterRegistry {
  private adapters: Map<string, WalletAdapter> = new Map();

  constructor() {
    // Register all adapters at initialization
    this.register(new MetaMaskAdapter());
    this.register(new PhantomAdapter());
    this.register(new BitgetAdapter());
  }

  private register(adapter: WalletAdapter): void {
    this.adapters.set(adapter.id, adapter);
  }

  get(id: string): WalletAdapter | undefined {
    return this.adapters.get(id);
  }

  getAll(): WalletAdapter[] {
    return Array.from(this.adapters.values());
  }

  getInstalled(): WalletAdapter[] {
    return this.getAll().filter((adapter) => adapter.isInstalled());
  }

  getForChain(chain: ChainType): WalletAdapter[] {
    return this.getAll().filter((adapter) => adapter.info.supportedChains.includes(chain));
  }

  getAllInfo(): WalletInfo[] {
    return this.getAll().map((adapter) => adapter.info);
  }

  getInstalledInfo(): WalletInfo[] {
    return this.getInstalled().map((adapter) => adapter.info);
  }

  detectFromAddress(address: string): WalletAdapter | null {
    const adapters = this.getAll();
    for (const adapter of adapters) {
      if (adapter.detectChain(address)) {
        return adapter;
      }
    }

    return null;
  }

  detectChainFromAddress(address: string): ChainType | null {
    const adapters = this.getAll();
    for (const adapter of adapters) {
      const chain = adapter.detectChain(address);

      if (chain) {
        return chain;
      }
    }

    return null;
  }
}

// Singleton instance
export const walletRegistry = new WalletAdapterRegistry();
