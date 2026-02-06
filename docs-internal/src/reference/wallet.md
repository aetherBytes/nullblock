# Wallet Integration

Web3 wallet authentication and management.

## Supported Sources

| Source Type | Provider | Description |
|-------------|----------|-------------|
| `web3_wallet` | MetaMask, WalletConnect | Ethereum-compatible wallets |
| `api_key` | NullBlock | API key authentication |
| `email` | Various | Email/password (future) |
| `oauth` | Google, GitHub | OAuth providers (future) |
| `system` | Internal | System agents |

## Authentication Flow

```
1. User clicks "Connect Wallet"
2. Frontend requests challenge:
   POST /api/wallets/challenge
   {"wallet_address": "0x...", "wallet_chain": "ethereum"}

3. User signs challenge message in wallet

4. Frontend verifies signature:
   POST /api/wallets/verify
   {"wallet_address": "0x...", "signature": "0x..."}

5. If new user, auto-register:
   POST /api/users/register (internal)

6. Return session token
```

## OnchainKit Integration

Hecate frontend uses Coinbase OnchainKit for:
- Wallet connection
- ENS/Basename resolution
- Identity display

### Components

```tsx
import { ConnectWallet } from '@coinbase/onchainkit/wallet';
import { Identity } from '@coinbase/onchainkit/identity';

<ConnectWallet />
<Identity address={address} />
```

## Session Management

### Session Token

JWT token returned on successful verification:

```json
{
  "user_id": "uuid",
  "wallet_address": "0x...",
  "wallet_chain": "ethereum",
  "exp": 1234567890
}
```

### Storage

- Stored in React state (ephemeral)
- Cleared on refresh/logout
- Re-authenticate after expiration

## Database Schema

### user_references

```sql
CREATE TABLE user_references (
    id UUID PRIMARY KEY,
    source_identifier VARCHAR NOT NULL,  -- wallet address
    source_type JSONB NOT NULL,          -- {type, provider, network}
    display_name VARCHAR,
    avatar_url VARCHAR,
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ
);
```

### wallets

```sql
CREATE TABLE wallets (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES user_references(id),
    address VARCHAR NOT NULL,
    chain VARCHAR NOT NULL,
    is_primary BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ
);
```

## Logout Flow

```
1. Clear frontend state (messages, session)
2. POST /api/agents/clear-conversation (optional)
3. Disconnect wallet via OnchainKit
4. Redirect to pre-login view
```

## Multi-Chain Support

| Chain | Status |
|-------|--------|
| Ethereum | ✅ Supported |
| Polygon | 🔄 Planned |
| Base | 🔄 Planned |

## Security Notes

- Never store private keys
- Signatures verified server-side
- Session tokens have expiration
- Challenge messages include nonce

## Related

- [API Endpoints](./api.md)
- [Erebus Router](../services/erebus.md)
