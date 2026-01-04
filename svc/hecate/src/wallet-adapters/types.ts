export enum ChainType {
  EVM = 'evm',
  SOLANA = 'solana',
}

export interface WalletInfo {
  id: string;
  name: string;
  description: string;
  icon: string;
  supportedChains: ChainType[];
  installUrl: string;
}

export interface ConnectionResult {
  success: boolean;
  address?: string;
  publicKey?: string;
  chain: ChainType;
  error?: string;
}

export interface SignatureResult {
  success: boolean;
  signature?: string;
  error?: string;
}

export interface WalletAdapter {
  readonly id: string;
  readonly info: WalletInfo;

  isInstalled(): boolean;
  isConnected(): Promise<boolean>;

  connect(chain?: ChainType): Promise<ConnectionResult>;
  disconnect(): Promise<void>;

  signMessage(message: string): Promise<SignatureResult>;

  getProvider(chain: ChainType): unknown;
  detectChain(address: string): ChainType | null;
}

export interface WalletAdapterEvents {
  onConnect?: (result: ConnectionResult) => void;
  onDisconnect?: () => void;
  onAccountChange?: (address: string) => void;
  onChainChange?: (chainId: string) => void;
  onError?: (error: Error) => void;
}

// Ethereum provider types
export interface EthereumProvider {
  isMetaMask?: boolean;
  isBitKeep?: boolean;
  selectedAddress?: string;
  providers?: EthereumProvider[];
  request: (args: { method: string; params?: unknown[] }) => Promise<unknown>;
  on?: (event: string, handler: (...args: unknown[]) => void) => void;
  removeListener?: (event: string, handler: (...args: unknown[]) => void) => void;
}

// Solana provider types
export interface SolanaProvider {
  isPhantom?: boolean;
  isBitKeep?: boolean;
  isConnected?: boolean;
  publicKey?: { toString: () => string; toBase58?: () => string };
  connect: (options?: { onlyIfTrusted?: boolean }) => Promise<{ publicKey: { toString: () => string } }>;
  disconnect: () => Promise<void>;
  signMessage: (message: Uint8Array, encoding?: string) => Promise<{ signature: Uint8Array }>;
  on?: (event: string, handler: (...args: unknown[]) => void) => void;
  off?: (event: string, handler: (...args: unknown[]) => void) => void;
}

// Window type extensions
declare global {
  interface Window {
    ethereum?: EthereumProvider;
    phantom?: { solana?: SolanaProvider };
    bitkeep?: {
      ethereum?: EthereumProvider;
      solana?: SolanaProvider;
    };
  }
}
