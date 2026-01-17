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

### Required Safeguards (TODO)
- [ ] Position tracking: Don't buy if already holding token
- [ ] Cooldown per mint: Minimum time between buys on same token
- [ ] Max position size: Cap total exposure per token
- [ ] Scanner deduplication: Track recently acted-on signals

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
