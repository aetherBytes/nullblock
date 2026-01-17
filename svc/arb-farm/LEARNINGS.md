# ArbFarm Learnings

## 2026-01-17: Pump.fun Execution Fix

### What Happened
- Fixed buy instruction (16 accounts with fee_config PDA)
- Fixed sell instruction (14 accounts - no volume accumulators)
- Lost ~0.27 SOL due to self-front-running

### Root Cause of Loss
1. Scanner auto-executed buys every 5 seconds on same token
2. No deduplication - kept buying same mint repeatedly
3. Each buy pushed price higher on bonding curve
4. Sold at loss after 6 consecutive buys

### Technical Fixes Made
- **Buy**: Added `fee_config` PDA with seeds `['fee_config', program_id]` derived from fee_program
- **Sell**: Removed `global_volume_accumulator` and `user_volume_accumulator` (only needed for buy)
- Buy uses 16 accounts, Sell uses 14 accounts

### Implemented Safeguards
- [x] Position tracking: Don't buy if already holding token (`has_open_position_for_mint`)
- [x] Position creation: Create `OpenPosition` after successful buy for tracking
- [x] Cooldown per mint: 5-minute cooldown between buys on same token
- [x] Max position size: Capped at 0.05 SOL for testing (strategy-level)
- [x] Liquidity contribution check: Skip if buy > 10% of pool liquidity
- [x] Minimum pool size: Skip if pool has < 5 SOL
- [x] Graduation check: Skip if token already graduated
- [x] Helius send endpoint: `/helius/sender/send` for programmatic selling

### Test Results (2026-01-17)

**Safeguard Verification**:
- Position deduplication: ✅ Working - "⏭️ Skipping: already have open position for this mint"
- Cooldown tracking: ✅ Working - "⏭️ Skipping: mint on cooldown (297s remaining)"
- Liquidity checks: ✅ Working - Buys logged ~6-7% contribution
- Position creation: ✅ Working - Positions appear in `/positions` API

**Round-trip Test**:
- 4 buys executed at 0.05 SOL each = 0.20 SOL deployed
- 4 sells recovered ~0.19 SOL
- Net cost: ~0.01 SOL (gas + slippage)

### Gas & Profit Calculations

**Transaction Costs (Solana mainnet)**:
- Base fee: ~5,000 lamports (0.000005 SOL)
- Priority fee: ~200,000 lamports (0.0002 SOL) for fast inclusion
- **Total per tx**: ~0.00025 SOL

**Round-trip cost (buy + sell)**:
- 2 transactions × 0.00025 = **0.0005 SOL minimum**

**Break-even for 0.01 SOL position**:
- Need 5% gain just to cover gas
- With 1% bonding curve fee (buy + sell = 2%): need 7% gain

**Profit Threshold Constants**:
```rust
const MIN_PROFIT_THRESHOLD_LAMPORTS: u64 = 500_000;  // 0.0005 SOL
const ESTIMATED_GAS_COST_LAMPORTS: u64 = 250_000;    // 0.00025 SOL per tx
```

### Liquidity Contribution Rules
- **Max contribution**: 10% of pool's real SOL reserves
- **Min pool size**: 5 SOL (ensures enough liquidity for exit)
- **Graduation check**: Skip if token already graduated

### Account Structures

**Buy (16 accounts)**:
1. global_state
2. fee_recipient
3. mint
4. bonding_curve
5. associated_bonding_curve
6. user_token_account
7. user (signer)
8. system_program
9. token_2022_program
10. creator_vault
11. event_authority
12. program_id
13. global_volume_accumulator
14. user_volume_accumulator
15. fee_config
16. fee_program

**Sell (14 accounts)**:
1. global_state
2. fee_recipient
3. mint
4. bonding_curve
5. associated_bonding_curve
6. user_token_account
7. user (signer)
8. system_program
9. creator_vault
10. token_2022_program
11. event_authority
12. program_id
13. fee_config
14. fee_program
