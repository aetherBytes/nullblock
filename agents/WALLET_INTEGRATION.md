# Wallet Integration System

This document explains NullBlock's wallet authentication system, including how wallet types are registered (compile-time) and how users are registered (runtime).

## Table of Contents

- [Architecture Overview](#architecture-overview)
- [Two Types of Registration](#two-types-of-registration)
- [Wallet Type Registration (Compile-Time)](#wallet-type-registration-compile-time)
- [User Registration Flow (Runtime)](#user-registration-flow-runtime)
- [Supported Wallets](#supported-wallets)
- [Adding a New Wallet](#adding-a-new-wallet)
- [Chain Support](#chain-support)
- [API Reference](#api-reference)

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           WALLET AUTHENTICATION                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌──────────────┐     ┌──────────────┐     ┌──────────────┐                │
│  │   MetaMask   │     │   Phantom    │     │   Bitget     │                │
│  │   Adapter    │     │   Adapter    │     │   Adapter    │                │
│  └──────┬───────┘     └──────┬───────┘     └──────┬───────┘                │
│         │                    │                    │                         │
│         └────────────────────┼────────────────────┘                         │
│                              │                                              │
│                    ┌─────────▼─────────┐                                   │
│                    │  WalletRegistry   │  ← Compile-time registration      │
│                    │  (Static)         │                                   │
│                    └─────────┬─────────┘                                   │
│                              │                                              │
│              ┌───────────────┼───────────────┐                             │
│              │               │               │                              │
│    ┌─────────▼─────┐  ┌──────▼──────┐  ┌────▼────────┐                    │
│    │ EVM Verifier  │  │ Solana      │  │ Future      │                    │
│    │ (ECDSA)       │  │ Verifier    │  │ Chains...   │                    │
│    └───────────────┘  │ (Ed25519)   │  └─────────────┘                    │
│                       └─────────────┘                                      │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                           USER REGISTRATION                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  On successful wallet authentication:                                        │
│                                                                              │
│    POST /api/users/register                                                 │
│    {                                                                         │
│      source_type: { type: "Web3Wallet", provider: "bitget", network: "evm" }│
│      source_identifier: "0x1234..."  // wallet address                      │
│    }                                                                         │
│                                                                              │
│    → Creates user_reference in Erebus DB                                    │
│    → Replicates to Agents DB via PostgreSQL logical replication             │
│    → Returns user_id UUID                                                   │
│                                                                              │
│  1 Wallet Address = 1 User (idempotent - same wallet always returns same ID)│
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Two Types of Registration

### 1. Wallet Type Registration (Compile-Time)

**What**: Adding support for a new wallet provider (e.g., MetaMask, Phantom, Bitget)

**When**: Done by developers when adding new wallet support to NullBlock

**Where**: Code changes in backend (`WalletRegistry`) and frontend (`WalletAdapterRegistry`)

**Frequency**: Occasional (when new wallet types are added)

### 2. User Registration (Runtime)

**What**: Creating a user account when someone connects their wallet for the first time

**When**: Automatically during wallet authentication flow

**Where**: Backend `/api/users/register` endpoint

**Frequency**: Every time a new wallet address connects

---

## Wallet Type Registration (Compile-Time)

Wallet types are registered in the codebase at compile time. This provides type safety and ensures all wallet adapters are properly configured.

### Backend Registration (`svc/erebus/`)

```rust
// svc/erebus/src/resources/wallets/registry.rs

pub static WALLET_REGISTRY: LazyLock<WalletRegistry> = LazyLock::new(|| {
    let mut registry = WalletRegistry::new();

    // Register wallet types (compile-time)
    registry.register(Box::new(MetaMaskAdapter::new()));
    registry.register(Box::new(PhantomAdapter::new()));
    registry.register(Box::new(BitgetAdapter::new()));
    // Add new wallets here...

    registry
});
```

### Frontend Registration (`svc/hecate/`)

```typescript
// svc/hecate/src/wallet-adapters/registry.ts

class WalletAdapterRegistry {
  constructor() {
    // Register wallet types (compile-time)
    this.register(new MetaMaskAdapter());
    this.register(new PhantomAdapter());
    this.register(new BitgetAdapter());
    // Add new wallets here...
  }
}

export const walletRegistry = new WalletAdapterRegistry();
```

---

## User Registration Flow (Runtime)

When a user connects their wallet for the first time, they are automatically registered as a NullBlock user. This happens during the authentication flow.

### Complete Authentication + Registration Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ STEP 1: User Initiates Connection                                           │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  User clicks "Connect Wallet" → Selects wallet (e.g., Bitget)               │
│                                                                              │
│  Frontend:                                                                   │
│    const adapter = walletRegistry.get('bitget');                            │
│    const result = await adapter.connect(ChainType.EVM);                     │
│    // result.address = "0x1234..."                                          │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ STEP 2: Create Authentication Challenge                                      │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Frontend → Backend:                                                         │
│    POST /api/wallets/challenge                                              │
│    {                                                                         │
│      wallet_address: "0x1234...",                                           │
│      wallet_type: "bitget",                                                 │
│      chain: "evm"                                                           │
│    }                                                                         │
│                                                                              │
│  Backend:                                                                    │
│    1. Lookup adapter: WALLET_REGISTRY.get("bitget")                         │
│    2. Validate address format for chain                                     │
│    3. Generate challenge_id (UUID)                                          │
│    4. Create challenge message via adapter                                  │
│    5. Store challenge in memory (5-minute TTL)                              │
│                                                                              │
│  Response:                                                                   │
│    {                                                                         │
│      challenge_id: "abc-123-def",                                           │
│      message: "Welcome to Nullblock!\n\nSign this message...",              │
│      wallet_address: "0x1234..."                                            │
│    }                                                                         │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ STEP 3: User Signs Challenge                                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Frontend:                                                                   │
│    const signResult = await adapter.signMessage(challenge.message);         │
│    // signResult.signature = "0xabc123..."                                  │
│                                                                              │
│  Wallet popup appears → User clicks "Sign"                                  │
│                                                                              │
│  Chain-specific signing:                                                     │
│                                                                              │
│    EVM (MetaMask, Bitget EVM):                                              │
│      provider.request({ method: 'personal_sign', params: [msg, addr] })    │
│                                                                              │
│    Solana (Phantom, Bitget Solana):                                         │
│      provider.signMessage(new TextEncoder().encode(msg))                    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ STEP 4: Verify Signature + Register User                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Frontend → Backend:                                                         │
│    POST /api/wallets/verify                                                 │
│    {                                                                         │
│      challenge_id: "abc-123-def",                                           │
│      signature: "0xabc123...",                                              │
│      wallet_address: "0x1234..."                                            │
│    }                                                                         │
│                                                                              │
│  Backend verification:                                                       │
│    1. Retrieve challenge from storage                                       │
│    2. Verify address matches                                                │
│    3. Lookup adapter: WALLET_REGISTRY.get(challenge.wallet_type)            │
│    4. Verify signature via chain verifier                                   │
│                                                                              │
│  ═══════════════════════════════════════════════════════════════════════    │
│  ║ ON SUCCESS: AUTOMATIC USER REGISTRATION                              ║    │
│  ═══════════════════════════════════════════════════════════════════════    │
│                                                                              │
│    Internal call to register_user_in_database():                            │
│                                                                              │
│      POST /api/users/register                                               │
│      {                                                                       │
│        source_type: {                                                       │
│          type: "Web3Wallet",                                                │
│          provider: "bitget",       // wallet type                           │
│          network: "ethereum"       // chain network                         │
│        },                                                                    │
│        source_identifier: "0x1234..."  // wallet address (unique ID)        │
│      }                                                                       │
│                                                                              │
│    This is IDEMPOTENT:                                                       │
│      - First connection: Creates new user_reference, returns new user_id    │
│      - Subsequent connections: Finds existing user, returns same user_id    │
│                                                                              │
│  Response:                                                                   │
│    {                                                                         │
│      success: true,                                                         │
│      session_token: "session-xyz-789",                                      │
│      user_id: "uuid-user-123",                                              │
│      network: "ethereum"                                                    │
│    }                                                                         │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ STEP 5: Session Established                                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Frontend stores session:                                                    │
│    localStorage.setItem('walletPublickey', address);                        │
│    localStorage.setItem('walletType', 'bitget');                            │
│    localStorage.setItem('walletChain', 'evm');                              │
│    localStorage.setItem('sessionToken', token);                             │
│    localStorage.setItem('lastAuthTime', Date.now());                        │
│                                                                              │
│  User is now authenticated and can:                                          │
│    - Access protected features                                              │
│    - Create and manage tasks                                                │
│    - Interact with agents                                                   │
│    - Use the Crossroads marketplace                                         │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### User Identity Model

```
1 Wallet Address = 1 User

source_identifier (wallet address) is the unique key:
  - "0x1234abcd..." → User A (EVM)
  - "5FHn3x9K..." → User B (Solana)
  - Same address connecting again → Same user returned

Users can have multiple wallets by:
  - Connecting additional wallets to their account (future feature)
  - Each wallet address is a separate user_reference
```

---

## Supported Wallets

| Wallet | ID | Chains | Provider Detection |
|--------|-----|--------|-------------------|
| MetaMask | `metamask` | EVM | `window.ethereum?.isMetaMask` |
| Phantom | `phantom` | Solana | `window.phantom?.solana` |
| Bitget | `bitget` | EVM, Solana | `window.bitkeep` |

### Provider Detection Patterns

```typescript
// MetaMask (EVM only)
const isMetaMaskInstalled = () => {
  if (typeof window.ethereum === 'undefined') return false;
  if (window.ethereum.providers?.length) {
    return window.ethereum.providers.some(p => p.isMetaMask);
  }
  return window.ethereum.isMetaMask;
};

// Phantom (Solana only)
const isPhantomInstalled = () => {
  return typeof window.phantom?.solana !== 'undefined';
};

// Bitget (EVM + Solana)
const isBitgetInstalled = () => {
  return typeof window.bitkeep !== 'undefined';
};
const getBitgetEvmProvider = () => window.bitkeep?.ethereum;
const getBitgetSolanaProvider = () => window.bitkeep?.solana;
```

---

## Adding a New Wallet

To add support for a new wallet (e.g., "CoolWallet"), you need to:

### Backend (2 files)

**1. Create adapter: `svc/erebus/src/resources/wallets/adapters/coolwallet.rs`**

```rust
use super::super::traits::{WalletAdapter, WalletInfo, ChallengeContext, ChainType, WalletError};
use super::super::chains::{EvmSignatureVerifier, ChainSignatureVerifier};

#[derive(Debug)]
pub struct CoolWalletAdapter {
    evm_verifier: EvmSignatureVerifier,
}

impl CoolWalletAdapter {
    pub fn new() -> Self {
        Self { evm_verifier: EvmSignatureVerifier }
    }
}

impl WalletAdapter for CoolWalletAdapter {
    fn id(&self) -> &'static str { "coolwallet" }

    fn info(&self) -> WalletInfo {
        WalletInfo {
            id: "coolwallet".to_string(),
            name: "CoolWallet".to_string(),
            description: "The coolest wallet around".to_string(),
            icon: "https://coolwallet.io/icon.svg".to_string(),
            supported_chains: vec![ChainType::Evm],
            install_url: "https://coolwallet.io/".to_string(),
        }
    }

    fn supported_chains(&self) -> &[ChainType] { &[ChainType::Evm] }

    fn validate_address(&self, address: &str, chain: &ChainType) -> bool {
        match chain {
            ChainType::Evm => self.evm_verifier.validate_address(address),
            _ => false,
        }
    }

    fn create_challenge_message(&self, context: &ChallengeContext) -> String {
        format!(
            "Welcome to Nullblock!\n\nSign to authenticate your CoolWallet.\n\n\
             Address: {}\nChallenge: {}\nTimestamp: {}",
            context.wallet_address, context.challenge_id, context.timestamp
        )
    }

    fn verify_signature(&self, message: &str, signature: &str,
                        wallet_address: &str, chain: &ChainType) -> Result<bool, WalletError> {
        match chain {
            ChainType::Evm => self.evm_verifier.verify_signature(message, signature, wallet_address),
            _ => Err(WalletError::UnsupportedChain(chain.clone())),
        }
    }

    fn detect_chain_from_address(&self, address: &str) -> Option<ChainType> {
        if self.evm_verifier.validate_address(address) { Some(ChainType::Evm) } else { None }
    }

    fn get_network_for_chain(&self, chain: &ChainType) -> &'static str {
        match chain { ChainType::Evm => "ethereum", _ => "unknown" }
    }
}
```

**2. Register in: `svc/erebus/src/resources/wallets/registry.rs`**

```rust
// Add to imports
use super::adapters::coolwallet::CoolWalletAdapter;

// Add to registry initialization
registry.register(Box::new(CoolWalletAdapter::new()));
```

### Frontend (2 files)

**1. Create adapter: `svc/hecate/src/wallet-adapters/adapters/coolwallet.adapter.ts`**

```typescript
import { BaseWalletAdapter } from '../base-adapter';
import { WalletInfo, ConnectionResult, SignatureResult, ChainType } from '../types';

declare global {
  interface Window {
    coolwallet?: { ethereum?: any };
  }
}

export class CoolWalletAdapter extends BaseWalletAdapter {
  readonly id = 'coolwallet';

  readonly info: WalletInfo = {
    id: 'coolwallet',
    name: 'CoolWallet',
    description: 'The coolest wallet around',
    icon: 'https://coolwallet.io/icon.svg',
    supportedChains: [ChainType.EVM],
    installUrl: 'https://coolwallet.io/',
  };

  isInstalled(): boolean {
    return typeof window.coolwallet?.ethereum !== 'undefined';
  }

  getProvider(chain: ChainType): any {
    if (chain === ChainType.EVM) return window.coolwallet?.ethereum;
    return null;
  }

  async connect(chain: ChainType = ChainType.EVM): Promise<ConnectionResult> {
    const provider = this.getProvider(chain);
    if (!provider) {
      return { success: false, chain, error: 'CoolWallet not installed' };
    }

    try {
      const accounts = await provider.request({ method: 'eth_requestAccounts' });
      return { success: true, address: accounts[0], chain };
    } catch (error: any) {
      return { success: false, chain, error: error.message };
    }
  }

  async disconnect(): Promise<void> { /* CoolWallet cleanup */ }

  async signMessage(message: string): Promise<SignatureResult> {
    const provider = this.getProvider(ChainType.EVM);
    if (!provider?.selectedAddress) {
      return { success: false, error: 'Not connected' };
    }

    try {
      const signature = await provider.request({
        method: 'personal_sign',
        params: [message, provider.selectedAddress],
      });
      return { success: true, signature };
    } catch (error: any) {
      return { success: false, error: error.message };
    }
  }
}
```

**2. Register in: `svc/hecate/src/wallet-adapters/registry.ts`**

```typescript
import { CoolWalletAdapter } from './adapters/coolwallet.adapter';

// In constructor:
this.register(new CoolWalletAdapter());
```

### That's it!

No changes needed to:
- Routes
- WalletManager
- WalletService
- useWalletAdapter hook
- Wallet modal UI (auto-populates from registry)

---

## Chain Support

### EVM Chains (Ethereum-compatible)

- Ethereum Mainnet
- Polygon
- Optimism
- Arbitrum
- Base
- **Monad** (primary focus)

All EVM wallets share the same signature verification (ECDSA/secp256k1).

### Solana

- Mainnet Beta
- Devnet
- Testnet

Solana wallets use Ed25519 signature verification.

### Address Formats

```
EVM:     0x[40 hex chars]     e.g., 0x742d35Cc6634C0532925a3b844Bc9e7595f1
Solana:  [32-44 base58 chars] e.g., 5FHn3x9KBCzGCFPYhGBbDvPXVJu5HkKfUY6
```

---

## API Reference

### Wallet Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/api/wallets` | Get supported wallets list |
| `POST` | `/api/wallets/detect` | Detect installed wallets |
| `POST` | `/api/wallets/challenge` | Create auth challenge |
| `POST` | `/api/wallets/verify` | Verify signature (+ register user) |
| `GET` | `/api/wallets/status` | Get current session status |
| `POST` | `/api/wallets/sessions/validate` | Validate session token |

### Request/Response Examples

**Create Challenge**
```json
// POST /api/wallets/challenge
{
  "wallet_address": "0x742d35Cc6634C0532925a3b844Bc9e7595f1",
  "wallet_type": "bitget",
  "chain": "evm"
}

// Response
{
  "challenge_id": "550e8400-e29b-41d4-a716-446655440000",
  "message": "Welcome to Nullblock!\n\nSign this message...",
  "wallet_address": "0x742d35Cc6634C0532925a3b844Bc9e7595f1"
}
```

**Verify Signature**
```json
// POST /api/wallets/verify
{
  "challenge_id": "550e8400-e29b-41d4-a716-446655440000",
  "signature": "0x1234abcd...",
  "wallet_address": "0x742d35Cc6634C0532925a3b844Bc9e7595f1"
}

// Response (success - includes user registration)
{
  "success": true,
  "session_token": "session-token-uuid",
  "user_id": "user-uuid-123",
  "message": "Wallet authenticated successfully",
  "network": "ethereum"
}
```

---

## Troubleshooting

### Common Issues

**"Wallet not installed"**
- User needs to install the browser extension
- Check provider detection is correct for the wallet

**"Invalid signature format"**
- EVM signatures must start with `0x` and be 132+ chars
- Solana signatures are byte arrays converted to string

**"Challenge expired"**
- Challenges expire after 5 minutes
- User needs to restart the connection flow

**"User registration failed"**
- Check Erebus database connection
- Verify user_references table exists
- Check source_type format matches expected schema

### Debug Logging

Enable verbose logging in development:

```bash
# Backend
RUST_LOG=debug cargo run

# Frontend
localStorage.setItem('DEBUG_WALLET', 'true');
```
