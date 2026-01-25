# ArbFarm Trading Strategies

> Last Updated: 2026-01-25 (LLM consensus optimizations: emergency exit circuit breaker, SL 30%→40%, scanner time 7→3 min, scanner TP 15%→25%)

This document describes how ArbFarm's trading strategies work - from token discovery through position management and exit execution.

---

## Table of Contents

1. [Overview](#overview)
2. [Strategy 1: Bonding Curve Scanner](#strategy-1-bonding-curve-scanner)
3. [Strategy 2: Graduation Sniper](#strategy-2-graduation-sniper)
4. [Position Management](#position-management)
5. [Exit Strategies](#exit-strategies)
6. [Risk Management](#risk-management)
7. [Wallet Reconciliation](#wallet-reconciliation)
8. [Configuration Reference](#configuration-reference)

---

## Overview

ArbFarm operates as an autonomous multi-agent system with the following components:

```
┌─────────────────────────────────────────────────────────────────┐
│                         DISCOVERY LAYER                          │
├─────────────────────────────────────────────────────────────────┤
│  Scanner Agent          Graduation Sniper        KOL Discovery  │
│  (Curve monitoring)     (Graduation events)      (Copy trading) │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                         EXECUTION LAYER                          │
├─────────────────────────────────────────────────────────────────┤
│  Autonomous Executor    Position Manager        Capital Manager  │
│  (Buy execution)        (Position tracking)     (SOL allocation) │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                         MONITORING LAYER                         │
├─────────────────────────────────────────────────────────────────┤
│  Position Monitor       Graduation Tracker      Risk Monitor     │
│  (Exit conditions)      (Progress watching)     (Daily limits)   │
└─────────────────────────────────────────────────────────────────┘
```

---

## Strategy Isolation

Scanner and Sniper strategies operate **independently** - they can hold positions in the same token without blocking each other.

### Per-Strategy Budget Allocation

At startup, wallet balance is divided between active strategies:

```
Available SOL ÷ Active Strategy Count = Per-Strategy Budget
```

Example with 2 SOL available and 2 active strategies:
- Scanner: 1.0 SOL max position
- Sniper: 1.0 SOL max position

### Independent Position Tracking

Position checks are **per-strategy**, not global:

| Scenario | Before | After |
|----------|--------|-------|
| Scanner buys at 40% | ✅ Works | ✅ Works |
| Sniper buys at 85% (scanner has position) | ❌ Blocked | ✅ Works |
| Same mint, different strategies | Blocked | Independent |

### Strategy-Specific Exit Configs

| Strategy | Take Profit | Stop Loss | Time Limit | Use Case |
|----------|-------------|-----------|------------|----------|
| Scanner (curve_arb) | 25% | 20% | 3 min | Quick flips at 30-70% progress |
| Sniper (graduation_snipe) | 100% | 40% | 15 min | Let winners run post-graduation |

> **Note:** Stop loss increased from 30% to 40% (LLM consensus - reduces premature exits on recoverable dips). Scanner time reduced from 7 to 3 min (winning trades avg 2.1 min). Scanner TP increased from 15% to 25% (improves TP/SL ratio from 0.75 to 1.25). Scanner now uses its own adaptive TP targets (10%/25%) instead of sniper defaults (100%/150%).

### Momentum Toggle API

```bash
# Enable momentum-adaptive exits for scanner
curl -X POST localhost:9007/strategies/{id}/momentum \
  -H "Content-Type: application/json" \
  -d '{"enabled": true}'

# Disable momentum for faster exits
curl -X POST localhost:9007/strategies/{id}/momentum \
  -H "Content-Type: application/json" \
  -d '{"enabled": false}'
```

---

## Strategy 1: Bonding Curve Scanner

**File:** `src/agents/scanner.rs`

### How It Works

The Scanner Agent continuously monitors pump.fun bonding curves for entry opportunities.

### Discovery Flow

```
1. Scan pump.fun for active curves (every 2-6 seconds)
2. Filter by metrics:
   - Progress: 70-95% toward graduation
   - Volume: Recent buy activity
   - Holders: Growing holder count
   - Velocity: Positive progress momentum
3. Generate CurveGraduation signal
4. Pass to Strategy Engine for validation
5. If approved → Autonomous Executor places buy
```

### Buy Signal Criteria

| Metric | Threshold | Weight |
|--------|-----------|--------|
| Progress | 70-95% | Required |
| Progress Velocity | > 0%/min | High |
| Volume (24h) | > 5 SOL | Medium |
| Holder Count | > 50 | Medium |
| Curve Age | < 24h | Low |

### Signal Confidence Scoring

```
98%+ progress + positive velocity → 0.95 confidence
95%+ progress + positive velocity → 0.85 confidence
90%+ progress + positive velocity → 0.75 confidence
Below 90%                         → 0.60 confidence
```

### Metrics Collection

**Files:** `src/agents/curve_metrics.rs`, `src/handlers/curves.rs`

The metrics collector populates detailed token metrics from venue APIs (DexScreener):

```
┌─────────────────────────────────────────────────────────────────┐
│                         DATA SOURCES                              │
├─────────────────────────────────────────────────────────────────┤
│  DexScreener API        pump.fun API         On-Chain RPC       │
│  (Volume, Price)        (Holder stats)       (Curve state)      │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                   CurveMetricsCollector                           │
├─────────────────────────────────────────────────────────────────┤
│  populate_from_pump_fun()  │  Merges venue data with on-chain   │
│  populate_from_moonshot()  │  Caches for scorer consumption     │
└─────────────────────────────────────────────────────────────────┘
```

**Populated Metrics:**

| Metric | Source | Fallback |
|--------|--------|----------|
| `volume_24h` | DexScreener | 0.0 |
| `volume_1h` | Estimated (24h/24) | 0.0 |
| `holder_count` | pump.fun API | Helius largest accounts count |
| `top_10_concentration` | pump.fun API | Helius calculation |
| `price_momentum_24h` | DexScreener | 0.0 |
| `graduation_progress` | On-chain RPC | Estimated from market cap |
| `market_cap_sol` | On-chain RPC | DexScreener estimate |
| `liquidity_depth_sol` | On-chain RPC | 0.0 |

### Opportunity Scoring

**File:** `src/agents/curve_scorer.rs`

The scorer combines metrics into an actionable recommendation:

| Score Range | Recommendation | Action |
|-------------|----------------|--------|
| 80-100 | StrongBuy | Auto-execute |
| 60-79 | Buy | Auto-execute |
| 40-59 | Hold | Skip |
| 0-39 | Avoid | Skip |

**Scoring Weights (Default):**

| Factor | Weight | Description |
|--------|--------|-------------|
| Graduation | 30% | Progress toward graduation |
| Volume | 20% | Trading activity |
| Holders | 20% | Distribution quality |
| Momentum | 15% | Price/velocity trends |
| Risk | 15% | Penalty for red flags |

### Buy Execution

1. Build pump.fun buy transaction via `CurveTransactionBuilder`
2. Sign with Turnkey DevWallet
3. Send via Jito bundle (priority) or Helius fallback
4. On success → Register position with `PositionManager`
5. Apply `ExitConfig::for_curve_bonding()` (tiered exit strategy)

### Entry Parameters (from Global Risk Config)

| Parameter | Default | Description |
|-----------|---------|-------------|
| `max_position_sol` | 0.30 SOL | Maximum buy amount |
| `slippage_bps` | 1500 (15%) | Bonding curve slippage |

---

## Strategy 2: Graduation Sniper

**File:** `src/agents/graduation_sniper.rs`

The Graduation Sniper operates in two phases:

### Phase 1: Pre-Graduation Entry

**Trigger:** Token approaching graduation (90%+ progress)

```
Scanner detects high-progress curve
  → GraduationTracker monitors progress
  → At 95%+ with positive velocity
  → Autonomous Executor buys on bonding curve
  → Position waits for graduation event
```

**Exit on Graduation:**
When the token graduates to Raydium:
1. Receive `arb.curve.graduated` event
2. Calculate adaptive slippage (15% base for post-grad liquidity)
3. Execute sell via **Raydium Trade API** (direct, ~200ms latency)
   - If Raydium fails → Fallback to Jupiter (~1000ms)
4. Record P&L to engrams

**Raydium vs Jupiter Performance:**
| Metric | Raydium (Direct) | Jupiter (Aggregator) |
|--------|------------------|---------------------|
| Latency | ~200ms | ~1000ms |
| Routing overhead | None (direct) | 50-200 bps |
| Best for | Post-graduation (single pool) | Multi-hop routes |

### Phase 2: Post-Graduation Quick-Flip

**Trigger:** Graduation event for token we DON'T hold

```
Token graduates to Raydium
  → No existing position?
  → Execute quick-flip buy via Jupiter
  → Apply tight exit strategy (8% TP, 5% SL)
  → Monitor for fast exit
```

**Post-Graduation Entry Config:**

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| Entry Amount | `risk_config.max_position_sol` | Uses global risk config |
| Take Profit | 8% | Quick scalp target |
| Stop Loss | 5% | Tight risk control |
| Time Limit | 5 minutes | Don't hold post-grad long |
| Max Entry Delay | 200ms | Beat competition |

### Adaptive Slippage (Graduation Sells)

```rust
const MIN_SLIPPAGE_BPS: u32 = 500;   // 5% floor
const MAX_SLIPPAGE_BPS: u32 = 2000;  // 20% cap
const POST_GRAD_BASE: u32 = 1500;    // 15% for thin liquidity
const PROFIT_SACRIFICE: f64 = 0.25;  // 25% of profits

// For profitable positions:
slippage = max(profit_percent * 0.25 * 100, 500)
slippage = min(slippage, 2000)
```

### Sniper Controls

```bash
just arb-sniper-start    # Start sniper
just arb-sniper-stop     # Stop sniper
just arb-sniper-status   # Check stats
just dev-mac no-snipe    # Start without sniper
```

---

## Strategy 3: KOL Copy Trading

**Files:**
- `src/agents/kol_discovery.rs` - KOL wallet discovery and trust scoring
- `src/execution/copy_executor.rs` - Copy trade execution
- `src/handlers/webhooks.rs` - Helius webhook handler for trade detection

> **⚠️ Requires Public URL:** KOL copy trading depends on Helius webhooks pushing wallet activity to ArbFarm. This only works when deployed with a publicly accessible URL (not localhost). See CLAUDE.md "AWS Deployment TODO" section for setup instructions.

### Signal Flow

```
Helius Webhook (swap detected)
    │
    ▼
webhooks.rs: Look up KOL entity by wallet
    │
    ├── Not tracked → Skip
    │
    ▼
Record KOL trade in DB (arb_kol_trades)
    │
    ▼
Emit event: kol_topics::TRADE_DETECTED
    │
    ▼
autonomous_executor.rs: handle_kol_trade()
    │
    ▼
copy_executor.rs: execute_copy()
    │
    ├── Buy → Build pump.fun tx, sign, submit
    │
    └── Sell → Queue priority exit via PositionManager
```

### KOL Discovery

KOL Discovery Agent scans pump.fun and DexScreener every 60 seconds for profitable traders:

**Discovery Criteria:**
- Win rate ≥ 65%
- Profitability ≥ 20% APE
- Minimum 3 trades tracked

**Trust Score (0-100):**
- Based on win rate, profitability, trade count
- Consecutive wins bonus
- Minimum 60 trust score required for copy trading

### Copy Trade Execution

**Buy Execution:**
1. Fetch curve state, check if token graduated
2. Build pump.fun buy transaction with 500 bps slippage
3. Sign with dev wallet
4. Submit via Helius sender

**Sell Execution:**
1. Get open position from position manager
2. Queue priority exit
3. Poll for 30 seconds waiting for position monitor to close
4. **Emergency fallback:** If not closed after 30s, force direct market sell with 25% slippage (EMERGENCY_SLIPPAGE_BPS = 2500)
5. Try Raydium if curve sell fails (graduated tokens)

### Security

**Webhook Authentication (REQUIRED for production):**

Set `HELIUS_WEBHOOK_AUTH_TOKEN` environment variable to secure the webhook endpoint:

```bash
export HELIUS_WEBHOOK_AUTH_TOKEN="your-secret-token-here"
```

When registering webhooks with Helius, set the same token as the `auth_header`:
```json
{
  "webhookURL": "https://your-domain/api/arb/webhooks/helius",
  "authHeader": "your-secret-token-here"
}
```

Without this, anyone can POST fake trade signals and drain your wallet.

### Configuration

**CopyExecutorConfig defaults** (global executor settings):
- `enabled`: true
- `default_copy_percentage`: 50% (copy 50% of KOL trade amount)
- `max_position_sol`: 0.5 SOL cap per trade
- `min_trust_score`: 60.0 threshold
- `copy_delay_ms`: 500ms delay before execution

**CopyTradeConfig** (per-KOL settings stored in DB):
- `copy_percentage`: 50% (matches executor default)
- `max_position_sol`: 0.5 SOL
- `token_whitelist`: Optional list of allowed tokens
- `token_blacklist`: Optional list of blocked tokens

### API Endpoints

```bash
# Add KOL for tracking (registers Helius webhook)
POST /api/arb/kols

# Enable copy trading for a KOL
PUT /api/arb/kols/{id}/copy-trading/enable

# Get copy trade history
GET /api/arb/kols/{id}/copy-history

# Start KOL discovery scan
POST /api/arb/discovery/start
```

---

## Position Management

**File:** `src/execution/position_manager.rs`

### Position Lifecycle

```
┌──────────┐     ┌──────────────────┐     ┌─────────────┐
│   Open   │────▶│ PartiallyExited  │────▶│   Closed    │
└──────────┘     └──────────────────┘     └─────────────┘
     │                                          ▲
     │           ┌──────────────┐               │
     └──────────▶│ PendingExit  │───────────────┘
                 └──────────────┘
                        │
                        ▼
                 ┌──────────────┐
                 │   Orphaned   │  (Dead token, no recovery)
                 └──────────────┘
```

### Position States

| Status | Description |
|--------|-------------|
| `Open` | Active position being monitored |
| `PartiallyExited` | Some tokens sold (tiered exit) |
| `PendingExit` | Exit in progress or queued for retry |
| `Closed` | Successfully exited |
| `Orphaned` | Dead token, no liquidity, gave up |

### Position Data Tracked

```rust
struct OpenPosition {
    id: Uuid,
    token_mint: String,
    token_symbol: Option<String>,

    // Entry data
    entry_amount_base: f64,      // SOL spent
    entry_token_amount: f64,     // Tokens received
    entry_price: f64,            // Price per token
    entry_time: DateTime<Utc>,
    entry_tx_signature: Option<String>,

    // Current state
    current_price: f64,
    current_value_base: f64,
    remaining_amount_base: f64,  // After partial exits
    high_water_mark: f64,        // Peak price (for trailing stop)

    // P&L
    unrealized_pnl: f64,
    unrealized_pnl_percent: f64,
    realized_pnl: f64,

    // Momentum tracking
    momentum: MomentumData,

    // Exit configuration
    exit_config: ExitConfig,
    status: PositionStatus,
}
```

---

## Exit Strategies

**File:** `src/execution/position_monitor.rs`

### Exit Config: Scanner (for_scanner)

Used for scanner-discovered positions at 30-70% progress:

```
Phase 1: At 10% gain
  → Sell 35% to recover partial capital

Phase 2: At 25% gain
  → Sell remaining 65%
  → Quick exit, no moon bag
```

**Configuration:**

| Parameter | Value | Notes |
|-----------|-------|-------|
| Stop Loss | 20% | |
| Take Profit (Phase 1) | 10% | Increased from 8% |
| Take Profit (Phase 2) | 25% | Increased from 15% (better TP/SL ratio) |
| Trailing Stop | 12% | |
| Time Limit | 3 minutes | Reduced from 7 min (winning trades avg 2.1 min) |

### Exit Config: Sniper (for_curve_bonding)

Used for graduation snipe positions at 85%+ progress:

```
Phase 1: At 2x (100% gain)
  → Sell 50% to recover capital
  → Capital is now "free"

Phase 2: At 2.5x (150% gain from remaining)
  → Sell 25% to lock profits
  → Still holding 25%

Phase 3: Moon bag with trailing stop
  → Remaining 25% rides with 20% trailing stop
  → Let winners run
```

**Configuration:**

| Parameter | Value | Notes |
|-----------|-------|-------|
| Stop Loss | 40% | Increased from 30% (LLM consensus - reduces premature exits) |
| Take Profit (Phase 1) | 100% | |
| Take Profit (Phase 2) | 150% | |
| Trailing Stop | 20% | |
| Time Limit | 15 minutes | |

### Exit Triggers

The Position Monitor checks these conditions every 2 seconds:

```rust
enum ExitReason {
    Emergency,          // CIRCUIT BREAKER: -50% catastrophic loss protection
    StopLoss,           // Price dropped below configured SL (40% for sniper, 20% for scanner)
    TakeProfit,         // Hit take profit target
    TrailingStop,       // Dropped 20% from peak
    TimeLimit,          // Held > time limit (3 min scanner, 15 min sniper)
    PartialTakeProfit,  // Tiered exit phase
    MomentumDecay,      // Velocity declining sustained
    MomentumReversal,   // Strong reversal detected
    Salvage,            // Dead token recovery
    Manual,             // User-triggered exit
}
```

### Emergency Exit Circuit Breaker

**Added 2026-01-25** - Prevents catastrophic losses from rug pulls and crashes.

```
If unrealized_pnl_percent <= -50%:
  → IMMEDIATELY exit at Critical urgency
  → Bypasses all other exit logic
  → Would have prevented -84% and -91% losses seen in trade history
```

This circuit breaker triggers regardless of the configured stop loss, providing a hard floor to limit maximum single-position loss.

### Momentum-Based Exits

The system tracks price velocity and acceleration:

```rust
struct MomentumData {
    velocity: f64,          // % change per minute
    momentum_score: f64,    // -100 to +100 (acceleration)
    momentum_decay_count: u32,
}
```

**Momentum Exit Rules:**

| Hold Time | Velocity Threshold | Momentum Threshold | Decay Count Min |
|-----------|-------------------|-------------------|-----------------|
| < 5 min | < -5.0%/min | < -60 | 11 |
| 5-10 min | < -3.0%/min | < -40 | 8 |
| > 10 min | < -2.5%/min | < -35 | 7 |

**Profitable Position Reversal:** Requires `decay_count >= 4` with `velocity < -0.5` and `momentum_score < -10`.

**Momentum Slowing Exit:** Requires `decay_count >= 3` (or velocity stalled or 2+ consecutive negatives).

**Peak Drop Protection:** If position is profitable and has dropped 6%+ from peak P&L, exit to protect gains. This is independent of trailing stop and triggers earlier to prevent giving back profits.

*Note: Decay thresholds were increased ~30% on 2026-01-24 to reduce premature exits. Peak drop was increased from 3% to 6% to allow more volatility.*

### Slippage Calculation

```rust
fn calculate_profit_aware_slippage(position, signal) -> u16 {
    // Dead token? Maximum slippage to salvage anything
    if is_dead_token {
        return 9000; // 90%
    }

    // Calculate P&L-aware slippage
    let pnl_pct = (current - entry) / entry * 100;
    let base = if pnl_pct > 0 {
        (pnl_pct * 0.25 * 100) as u16  // 25% of profits
    } else {
        500  // 5% minimum
    };

    // Apply urgency multiplier
    let multiplier = match urgency {
        Critical => 1.5,
        High => 1.25,
        Normal => 1.0,
    };

    min(base * multiplier, 2000)  // Cap at 20%
}
```

### Transaction Execution Path

```
1. Build sell transaction:
   - Pre-graduation: pump.fun bonding curve
   - Post-graduation: Raydium (direct) → Jupiter (fallback)
2. Sign with DevWallet
3. Try Jito bundle (priority fee for speed)
4. If Jito fails/times out → Helius fallback
5. On timeout → Check wallet balance
   - If tokens gone → Treat as success
   - If tokens remain → Retry
6. Update position status
7. Record to engrams
```

**Post-Graduation Sell Routing:**
```
Token graduated?
  └─► Try Raydium Trade API
        ├─► Success → Execute via Jito
        └─► Fail → Try Jupiter Aggregator
              ├─► Success → Execute via Jito
              └─► Fail → Retry with backoff
```

---

## Risk Management

**File:** `src/execution/risk.rs`

### Global Risk Config

```rust
struct RiskConfig {
    max_position_sol: f64,           // 0.30 SOL default
    daily_loss_limit_sol: f64,       // 1.0 SOL default
    max_drawdown_percent: f64,       // 40% default (increased from 30% per LLM consensus)
    max_concurrent_positions: u32,   // 10 default
    cooldown_after_loss_ms: u64,     // 5000ms default
    auto_pause_on_drawdown: bool,    // true default
}
```

> **Note:** `max_drawdown_percent` increased from 30% to 40% based on LLM consensus analysis showing tokens often recover from 30%+ dips. Emergency circuit breaker at -50% provides catastrophic loss protection.

### Risk Checks (Before Every Buy)

1. **Daily Loss Limit** - Stop trading if daily losses exceed limit
2. **Position Size** - Cap individual position size
3. **Concurrent Positions** - Limit active positions
4. **Loss Cooldown** - Pause 5s after each loss
5. **Risk Score** - Higher risk = smaller position
6. **Signal Deduplication** - Prevents duplicate buy attempts for same signal
7. **Mint Cooldown** - 5 min cooldown per token (only applied AFTER successful buy)

### Signal Deduplication

**File:** `src/agents/strategy_engine.rs`

The StrategyEngine tracks processed signal IDs to prevent duplicate buy attempts:

```rust
// In process_signals(), before matching:
if processed_signals.contains(&signal.id) {
    continue;  // Skip already-processed signal
}

// After creating edge:
processed_signals.insert(signal.id);

// Auto-cleanup when cache exceeds 10,000 entries
if processed_signals.len() > 10_000 {
    processed_signals.clear();
}
```

This prevents the same signal from triggering multiple buy attempts if it matches multiple strategies or is processed multiple times.

### Mint Cooldown Fix

**File:** `src/agents/autonomous_executor.rs`

The mint cooldown is only applied AFTER a successful transaction:

```
BEFORE (Bug): Cooldown inserted before buy → failed TXs block retries for 5 min
AFTER (Fixed): Cooldown inserted after success → failed TXs can be retried immediately
```

This allows immediate retry of failed transactions while still preventing rapid-fire buys of successful tokens.

### Volatility-Adjusted Sizing

```rust
risk_factor = 1.0 - (risk_score / 200.0)  // 0.5 to 1.0
adjusted = base_size * risk_factor.max(0.25)
final = min(adjusted, max_position_sol)
```

---

## Wallet Reconciliation

**File:** `src/execution/position_manager.rs`

### Periodic Reconciliation (Every 10 seconds)

```
1. Fetch all token balances from Helius DAS
2. Compare against tracked positions
3. Handle discrepancies:
   - Token sold externally → Mark position closed
   - Token received externally → Create discovered position
   - Balance mismatch → Update remaining_amount
4. Skip dust tokens (< 0.0001 balance)
```

### Dead Token Detection

Tokens are considered "dead" if:
- Zero bonding curve reserves
- No Jupiter liquidity
- Zero recent volume

**Handling:**
1. Attempt salvage sell with 90% slippage
2. If sell fails → Mark as orphaned
3. Log for analysis

---

## Configuration Reference

### Just Commands

```bash
# Scanner control
just arb-scanner-start
just arb-scanner-stop
just arb-scanner-status

# Sniper control
just arb-sniper-start
just arb-sniper-stop
just arb-sniper-status

# Position management
just arb-positions          # List open positions
just arb-pnl               # P&L summary

# Dev environment
just dev-mac               # Start all services
just dev-mac no-scan       # Without scanner
just dev-mac no-snipe      # Without sniper
just dev-mac "no-scan no-snipe"  # Without both
```

### API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/positions` | GET | List open positions |
| `/positions/{id}` | GET | Get position details |
| `/positions/{id}/exit` | POST | Manual exit |
| `/pnl/summary` | GET | P&L summary |
| `/scanner/start` | POST | Start scanner |
| `/scanner/stop` | POST | Stop scanner |
| `/sniper/start` | POST | Start sniper |
| `/sniper/stop` | POST | Stop sniper |
| `/config/risk` | GET/POST | Risk configuration |
| `/strategies` | GET | List all strategies |
| `/strategies/{id}/momentum` | POST | Toggle momentum-adaptive exits |

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `ARB_SCANNER_AUTO_START` | true | Auto-start scanner |
| `/tmp/arb-no-scan` | - | Sentinel to disable scanner |
| `/tmp/arb-no-snipe` | - | Sentinel to disable sniper |

---

## Implementation Status

### Fully Implemented

- Bonding curve scanner
- Graduation sniper (pre/post-grad)
- Raydium Trade API integration (post-graduation sells)
- Position manager with P&L tracking
- Position monitor with adaptive exits
- Strategy isolation (per-strategy budgets + position tracking)
- Strategy-specific exit configs (scanner vs sniper)
- Tiered exit strategy
- Momentum-based exits (with API toggle)
- Risk management
- Engrams integration
- Dead token salvage

### Stubbed (Warnings Expected)

- DEX arbitrage (mev_hunter.rs)
- Raydium venue scanner (pool discovery)
- Direct Anthropic/OpenAI providers

### Not Implemented

- Threat detection (rug pulls)
- Research/DD agent
