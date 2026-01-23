# Raydium Integration Reference

> Reference documentation for Raydium APIs. See [Trading Strategies](./strategies.md) for how these integrate with ArbFarm.

---

## Trade API

**Base URL:** `https://transaction-v1.raydium.io`

### Swap Quote Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/compute/swap-base-in` | GET | Quote with fixed input amount |
| `/compute/swap-base-out` | GET | Quote with fixed output amount |

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `inputMint` | string | Yes | Source token address |
| `outputMint` | string | Yes | Destination token address |
| `amount` | string | Yes | Amount in base units (lamports) |
| `slippageBps` | number | Yes | Slippage tolerance (50 = 0.5%) |
| `txVersion` | string | Yes | `V0` or `LEGACY` |

### Transaction Serialization

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/transaction/swap-base-in` | POST | Build swap transaction |

**Request Body:**

```json
{
  "swapResponse": { /* quote response */ },
  "wallet": "user_pubkey",
  "txVersion": "V0",
  "wrapSol": true,
  "unwrapSol": true,
  "computeUnitPriceMicroLamports": 100000
}
```

### Priority Fees

| Endpoint | Method | Description |
|----------|--------|-------------|
| `https://api-v3.raydium.io/main/auto-fee` | GET | Network fee recommendations |

---

## Program Addresses

### Mainnet

| Program | Address |
|---------|---------|
| **CPMM** (Constant Product) | `CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C` |
| **Legacy AMM v4** | `675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8` |
| **CLMM** (Concentrated Liquidity) | `CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK` |
| **Stable Swap AMM** | `5quBtoiQqxF9Jv6KYKctB59NT3gtJD2Y65kdnB1Uev3h` |
| **AMM Routing** | `routeUGWgWzqBWFcrCfv8tritsqukccJPu3q5GPP3xS` |
| **LaunchLab** | `LanMV9sAd7wArD4vJFi2qDdfnVhFxYSUg6eADduJ3uj` |
| **Burn & Earn** (LP Locker) | `LockrWmn6K5twhz3y9w1dQERbmgSaRkfnTeTKbpofwE` |

### Devnet

| Program | Address |
|---------|---------|
| **CPMM** | `DRaycpLY18LhpbydsBWbVJtxpNv9oXPgjRSfpF2bWpYb` |
| **Legacy AMM v4** | `DRaya7Kj3aMWQSy19kSjvmuwq9docCHofyP9kanQGaav` |
| **CLMM** | `DRayAUgENGQBKVaX8owNhgzkEDyoHTGVEGHVJT1E9pfH` |
| **LaunchLab** | `DRay6fNdQ5J82H7xV6uq2aV3mNrUZ1J4PgSKsWgptcm6` |

---

## Token-2022 Support

### Supported Programs
- CLMM (Concentrated Liquidity)
- CPMM (Constant Product)

### Limitations
- Manual mint address entry required (no search by name/ticker)
- Risk acknowledgment required before interaction
- Token logos/names may not display
- Users must confirm before routing through Token-2022 pools

### Transfer Fee Extension
- Applies to swaps, liquidity ops, and all transfers
- **Controlled by token creator, not Raydium**
- Fees can change at any time without warning

### Unsupported Extensions
These cannot create permissionless CLMM pools:

| Extension | Reason |
|-----------|--------|
| Permanent delegate | Grants unlimited token authority |
| Non-transferable | Prevents trading (soulbound) |
| Default account state | Freezes new accounts |
| Confidential transfers | ZK incompatible with AMM |
| Transfer hook | Custom logic interferes with pools |

---

## SDK

**Package:** `@raydium-io/raydium-sdk-v2`

```bash
yarn add @raydium-io/raydium-sdk-v2
```

**Features:**
- Swaps
- Pool creation
- Liquidity management
- LaunchLab integration

**Resources:**
- [raydium-sdk-V2](https://github.com/raydium-io/raydium-sdk-V2) - Main SDK
- [raydium-sdk-V2-demo](https://github.com/raydium-io/raydium-sdk-V2-demo) - Examples
- [raydium-ui-v3-public](https://github.com/raydium-io/raydium-ui-v3-public) - UI reference

---

## LaunchLab (Future Integration)

> **Status:** Reference only - not yet integrated

### Overview
Raydium's community-driven token launch platform. Similar to pump.fun but with Raydium-native liquidity.

### Launch Mechanism
- **JustSendit**: Preset configuration, instant launch
- **LaunchLab**: Customizable parameters
- Bonding curve model with auto-migration to Raydium AMM
- Migration trigger: **85 SOL** threshold (JustSendit default)
- LP tokens burned or locked post-migration

### Revenue Model
- 50% of trading fees to community participants
- Creators receive trading fee share post-AMM migration

### Integration Opportunity
- Monitor LaunchLab graduations (similar to pump.fun)
- Entry on bonding curve, exit on AMM
- Direct pool access without Jupiter routing

### Program Address
- **Mainnet:** `LanMV9sAd7wArD4vJFi2qDdfnVhFxYSUg6eADduJ3uj`
- **Devnet:** `DRay6fNdQ5J82H7xV6uq2aV3mNrUZ1J4PgSKsWgptcm6`

---

## Current ArbFarm Integration

### What's Implemented
- ✅ Pool discovery via `/pools/info/list`
- ✅ Pool info extraction
- ✅ Swap computation (quote only)
- ✅ Health checks
- ✅ **Trade API integration for post-graduation sells**
  - Direct swap via `/compute/swap-base-in` + `/transaction/swap-base-in`
  - ~200ms latency vs ~1000ms Jupiter
  - Falls back to Jupiter if Raydium fails

### What's Still Missing
- DEX arbitrage detection
- LaunchLab integration
- Raydium pool scanning for entry signals

### Implementation Details

**File:** `src/execution/curve_builder.rs`

```rust
pub async fn build_raydium_sell(&self, params: &CurveSellParams)
    -> AppResult<PostGraduationSellResult>
```

**Routing (graduation_sniper.rs):**
```
Token graduated?
  └─► build_raydium_sell()
        ├─► Success → Return tx
        └─► Fail → build_post_graduation_sell() (Jupiter fallback)
```
