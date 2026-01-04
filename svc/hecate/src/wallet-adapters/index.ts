// Types
export { ChainType } from './types';
export type {
  WalletAdapter,
  WalletInfo,
  ConnectionResult,
  SignatureResult,
  WalletAdapterEvents,
  EthereumProvider,
  SolanaProvider,
} from './types';

// Base adapter
export { BaseWalletAdapter } from './base-adapter';

// Adapters
export { MetaMaskAdapter, PhantomAdapter, BitgetAdapter } from './adapters';

// Registry
export { walletRegistry } from './registry';

// Hooks
export { useWalletAdapter } from './hooks/useWalletAdapter';
