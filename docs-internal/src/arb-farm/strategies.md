# ArbFarm Trading Strategies

> Last Updated: 2026-02-01 (Audit fixes: token age filter for stale tokens, webhook status indicators, strategy labels on trades/positions, SSE-driven dashboard, event broadcast logging.)

This document describes how ArbFarm's trading strategies work - from token discovery through position management and exit execution. It also covers the pluggable strategy architecture and how to register new strategies.

---

## Table of Contents

1. [Overview](#overview)
2. [Strategy Architecture](#strategy-architecture)
3. [Graduation Sniper](#graduation-sniper)
4. [KOL Copy Trading (WIP)](#kol-copy-trading-wip)
5. [Position Management](#position-management)
6. [Exit Strategies](#exit-strategies)
7. [Risk Management](#risk-management)
8. [Wallet Reconciliation](#wallet-reconciliation)
9. [Configuration Reference](#configuration-reference)

---

## Overview

ArbFarm operates as an autonomous multi-agent system with the following components:

```
┌─────────────────────────────────────────────────────────────────┐
│                         DISCOVERY LAYER                          │
├─────────────────────────────────────────────────────────────────┤
│  Scanner Agent                                                  │
│  (Polls venues, orchestrates strategies)                        │
│    ├─ GraduationSniperStrategy (active)                         │
│    │    (Filters 85%+ progress, generates signals)              │
│    └─ KolCopyStrategy (WIP - event-buffered)                    │
│         (Drains webhook buffer, generates KolTrade signals)     │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                         EXECUTION LAYER                          │
├─────────────────────────────────────────────────────────────────┤
│  Autonomous Executor    Position Manager        Capital Manager  │
│  (Buy execution)        (Position tracking)     (SOL allocation) │
│                                                                   │
│  Position Executor                                                │
│  (Centralized sell: builds/signs/submits exit TXs)               │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                         MONITORING LAYER                         │
├─────────────────────────────────────────────────────────────────┤
│  Position Monitor ──────► command channel ──► Position Executor  │
│  (Exit detection only)    (mpsc)               (Sell execution)  │
│  Copy Trade Executor ────►                                       │
│  (Emergency sells)                                               │
│  Manual API triggers ────►                                       │
│                                                                   │
│  Graduation Tracker      Risk Monitor                            │
│  (Progress watching)     (Daily limits)                          │
└─────────────────────────────────────────────────────────────────┘
```

**Active Strategies:**

| Strategy | Type | Status |
|----------|------|--------|
| Graduation Sniper | `graduation_snipe` | Active |
| Raydium Snipe | `raydium_snipe` | Standby (awaiting Helius webhooks — requires public URL) |
| KOL Copy Trading | `copy_trade` | Standby (webhook not configured — requires `HELIUS_WEBHOOK_AUTH_TOKEN`) |

---

## Strategy Isolation

Strategies operate **independently** - they can hold positions in the same token without blocking each other.

### Per-Strategy Budget Allocation

At startup, wallet balance is divided between active strategies:

```
Available SOL ÷ Active Strategy Count = Per-Strategy Budget
```

### Strategy-Specific Exit Configs

**DEFENSIVE MODE (Default):** All strategies use the defensive config for capital preservation.

| Mode | Take Profit | Stop Loss | Trailing | Time Limit | Momentum Behavior |
|------|-------------|-----------|----------|------------|-------------------|
| **Defensive (DEFAULT)** | 15% | 10% | 8% | 5 min | Strong: extends to 30%+, Weak: exit at 7.5% |

**Legacy Configs (Available via API):**

| Config | Take Profit | Stop Loss | Time Limit | Use Case |
|--------|-------------|-----------|------------|----------|
| `for_scanner()` | 25% | 20% | 3 min | Quick flips at 30-70% progress |
| `for_graduation_sniper()` | 50% | 40% | 15 min | Conservative baseline, momentum extends |
| `for_curve_bonding()` | 100% | 40% | 15 min | Let winners run to 2x |

### Momentum Toggle API

```bash
curl -X POST localhost:9007/strategies/{id}/momentum \
  -H "Content-Type: application/json" \
  -d '{"enabled": true}'
```

---

## Strategy Architecture

ArbFarm uses a **pluggable strategy factory pattern**. Strategies are split into two layers that work together:

1. **Behavioral Strategies** — Rust structs implementing the `BehavioralStrategy` trait. These are registered with the Scanner and generate `Signal`s from venue data.
2. **Database Strategies** — Persisted configuration records loaded into the `StrategyEngine`. These define risk parameters, execution mode, and venue filters. The engine matches incoming signals against these records to create `Edge`s for execution.

Both layers must be wired for a strategy to function end-to-end.

### Signal Flow (End-to-End)

```
Scanner polls venues (pump.fun API, RPC, DexScreener fallback)
        │
        ▼
Creates VenueSnapshot { tokens: [...] }
        │
        ▼
Calls BehavioralStrategy::scan(snapshot) for each active strategy
        │
        ├─→ GraduationSniperStrategy → Vec<Signal>
        ├─→ [Future] NewStrategy     → Vec<Signal>
        │
        ▼
Scanner emits "signal_detected" events
        │
        ▼
StrategyEngine::process_signals(signals)
        │
        ├── Check signal expiry (drop if stale)
        ├── Check dedup cache (drop if already processed)
        ├── Match signal to DB strategies:
        │     ├── Venue type match?
        │     ├── Strategy-specific filter? (e.g., progress >= 85%)
        │     └── Risk param validation? (profit > min_profit_bps?)
        │
        ▼
Create Edge from matched Signal + Strategy
        │
        ▼
AutonomousExecutor picks up Edge
        │
        ├── Capital allocation check
        ├── Risk limit check
        ├── Simulate transaction (if required)
        ├── Determine execution mode (autonomous vs agent_directed)
        │
        ▼
Execute: build TX → sign → submit via Jito → confirm
        │
        ▼
PositionManager registers open position
        │
        ▼
PositionMonitor detects exit conditions (every 2s)
        │
        ▼
PositionCommand sent via mpsc channel
        │
        ▼
PositionExecutor builds TX → signs → submits via Jito → confirms
```

### Key Abstractions

#### BehavioralStrategy Trait

**File:** `src/agents/strategies/mod.rs`

```rust
#[async_trait]
pub trait BehavioralStrategy: Send + Sync {
    fn strategy_type(&self) -> &str;
    fn name(&self) -> &str;
    fn supported_venues(&self) -> Vec<VenueType>;
    async fn scan(&self, snapshot: &VenueSnapshot) -> AppResult<Vec<Signal>>;
    fn create_edge(&self, _signal: &Signal, _risk: &RiskParams) -> Option<Edge> {
        None
    }
    fn is_active(&self) -> bool;
    async fn set_active(&self, active: bool);
}
```

| Method | Purpose |
|--------|---------|
| `strategy_type()` | Returns the string key (e.g. `"graduation_snipe"`) — must match the DB strategy's `strategy_type` |
| `name()` | Human-readable name for logs/UI |
| `supported_venues()` | Which venue types this strategy can scan (e.g. `[VenueType::BondingCurve]`) |
| `scan()` | Core logic: receives a `VenueSnapshot` of all tokens from a venue, returns `Vec<Signal>` of detected opportunities |
| `is_active()` / `set_active()` | Toggle via `AtomicBool` — inactive strategies are skipped during scan cycles |

#### StrategyRegistry

Manages all registered behavioral strategies. The Scanner holds a reference to this.

```rust
pub struct StrategyRegistry {
    strategies: Arc<RwLock<Vec<Arc<dyn BehavioralStrategy>>>>,
}

impl StrategyRegistry {
    pub async fn register(&self, strategy: Arc<dyn BehavioralStrategy>);
    pub async fn get_active(&self) -> Vec<Arc<dyn BehavioralStrategy>>;
    pub async fn get_by_venue(&self, venue_type: VenueType) -> Vec<Arc<dyn BehavioralStrategy>>;
    pub async fn toggle(&self, name: &str, active: bool) -> bool;
}
```

#### Database Strategy Model

**File:** `src/models/strategy.rs`

```rust
pub struct Strategy {
    pub id: Uuid,
    pub wallet_address: String,
    pub name: String,              // "Graduation Snipe"
    pub strategy_type: String,     // "graduation_snipe"
    pub venue_types: Vec<String>,  // ["bondingcurve", "BondingCurve"]
    pub execution_mode: String,    // "autonomous" or "agent_directed"
    pub risk_params: RiskParams,
    pub is_active: bool,
}
```

The `StrategyEngine` holds a map of these DB strategies. When a signal comes in, the engine iterates through them looking for a match on venue type, strategy-specific filters, and risk params.

#### StrategyEngine Signal Matching

**File:** `src/agents/strategy_engine.rs`

The engine's `signal_matches_strategy()` method controls which signals a DB strategy accepts:

```rust
fn signal_matches_strategy(&self, signal: &Signal, strategy: &Strategy) -> bool {
    // 1. Venue type match
    let venue_matches = strategy.venue_types.iter().any(|vt|
        vt.to_lowercase() == signal_venue_str
    );
    if !venue_matches { return false; }

    // 2. Strategy-specific filter
    match strategy.strategy_type.as_str() {
        "graduation_snipe" => {
            let progress = signal.metadata.get("progress_percent")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            progress >= 85.0
        },
        _ => true,
    }
}
```

Add a new match arm here if your strategy needs to filter signals beyond venue type.

#### Capital Manager

**File:** `src/execution/capital_manager.rs`

Each strategy gets an equal share of the wallet balance. The capital manager tracks per-strategy reservations and position counts.

```
Available SOL ÷ Active Strategy Count = Per-Strategy Budget
```

`rebalance_equal()` is called after behavioral strategies are registered to redistribute allocations.

### How to Register a New Strategy

Follow these steps to add a new strategy to the system. Use `GraduationSniperStrategy` as a reference implementation.

#### Step 1: Create the strategy struct

Create a new file `src/agents/strategies/my_strategy.rs`:

```rust
use async_trait::async_trait;
use std::sync::atomic::{AtomicBool, Ordering};
use crate::agents::strategies::{BehavioralStrategy, VenueSnapshot};
use crate::error::AppResult;
use crate::models::{Signal, SignalType, VenueType, Edge, RiskParams};

pub struct MyStrategy {
    name: String,
    is_active: AtomicBool,
}

impl MyStrategy {
    pub fn new() -> Self {
        Self {
            name: "My Strategy".to_string(),
            is_active: AtomicBool::new(true),
        }
    }
}

#[async_trait]
impl BehavioralStrategy for MyStrategy {
    fn strategy_type(&self) -> &str { "my_strategy" }
    fn name(&self) -> &str { &self.name }
    fn supported_venues(&self) -> Vec<VenueType> {
        vec![VenueType::BondingCurve]
    }

    async fn scan(&self, snapshot: &VenueSnapshot) -> AppResult<Vec<Signal>> {
        let mut signals = Vec::new();

        for token in &snapshot.tokens {
            // Your detection logic here
            // Filter tokens, calculate confidence, estimate profit

            let signal = Signal {
                id: uuid::Uuid::new_v4(),
                signal_type: SignalType::CurveGraduation,
                venue_id: snapshot.venue_id,
                venue_type: snapshot.venue_type.clone(),
                token_mint: Some(token.mint.clone()),
                pool_address: token.bonding_curve_address.clone(),
                estimated_profit_bps: 500,  // 5%
                confidence: 0.7,
                metadata: serde_json::json!({
                    "signal_source": "my_strategy",
                    "progress_percent": token.graduation_progress,
                }),
                detected_at: chrono::Utc::now(),
                expires_at: chrono::Utc::now() + chrono::Duration::minutes(5),
                ..Default::default()
            };
            signals.push(signal);
        }

        Ok(signals)
    }

    fn is_active(&self) -> bool {
        self.is_active.load(Ordering::SeqCst)
    }

    async fn set_active(&self, active: bool) {
        self.is_active.store(active, Ordering::SeqCst);
    }
}
```

#### Step 2: Export from `strategies/mod.rs`

```rust
pub mod my_strategy;
pub use my_strategy::MyStrategy;
```

#### Step 3: Register the behavioral strategy with Scanner

In `server.rs` (near the other behavioral strategy registrations):

```rust
use crate::agents::strategies::MyStrategy;

let my_strategy = Arc::new(MyStrategy::new());
scanner.register_behavioral_strategy(my_strategy).await;
```

#### Step 4: Create a DB strategy record

In `server.rs` (near the other `get_or_create_strategy` calls):

```rust
if let Some(my_db_strategy) = get_or_create_strategy(
    &strategy_repo,
    "My Strategy",             // Display name
    &default_wallet,
    "my_strategy",             // Must match strategy_type() return value
    vec!["bondingcurve".to_string(), "BondingCurve".to_string()],
    "autonomous",              // "autonomous" or "agent_directed"
    RiskParams {
        max_position_sol: 0.15,
        daily_loss_limit_sol: 1.0,
        min_profit_bps: 50,
        max_slippage_bps: 150,
        auto_execute_enabled: true,
        stop_loss_percent: Some(10.0),
        take_profit_percent: Some(15.0),
        trailing_stop_percent: Some(8.0),
        time_limit_minutes: Some(5),
        ..Default::default()
    },
).await {
    strategy_engine.add_strategy(my_db_strategy).await;
}
```

#### Step 5: Add signal matching (optional)

If your strategy needs to filter signals beyond venue type matching, add a match arm in `strategy_engine.rs`:

```rust
fn signal_matches_strategy(&self, signal: &Signal, strategy: &Strategy) -> bool {
    // ... venue match check ...

    match strategy.strategy_type.as_str() {
        "graduation_snipe" => { progress >= 85.0 },
        "my_strategy" => {
            // Custom filter logic
            true
        },
        _ => true,
    }
}
```

If your strategy accepts all signals matching its venue types, no changes are needed — the `_ => true` fallback handles it.

#### Note: Event-Driven Strategies (Buffer Pattern)

Not all strategies read from `VenueSnapshot.tokens`. Some strategies receive data asynchronously via external events (webhooks, WebSocket feeds, etc.) instead of polling a venue. These strategies use an internal buffer:

1. External source pushes events into the strategy via a method like `push_event()`
2. The strategy stores events in an `Arc<RwLock<Vec<Event>>>` buffer
3. When Scanner calls `scan()`, the strategy drains the buffer and converts events to `Signal`s
4. The `VenueSnapshot` parameter is ignored — signals come from the buffer, not the snapshot

This pattern keeps event-driven data sources in the same Scanner → StrategyEngine → Executor pipeline without requiring a new `VenueType`. See `KolCopyStrategy` (WIP) for a reference implementation of this pattern.

#### Step 6: Verify

1. `cargo check` — compiles clean
2. Restart arb-farm
3. `GET /scanner/strategies` — your strategy appears in the list
4. `GET /strategies` — DB strategy record exists with correct risk params
5. Watch logs for signals from your strategy being detected and matched

### What the Scanner Does

The Scanner (`src/agents/scanner.rs`) runs a polling loop on a configurable interval (2-6 seconds). Each cycle:

1. Queries all registered venues (pump.fun API primary, DexScreener fallback) for token data
2. Builds a `VenueSnapshot` containing all discovered tokens with metrics (progress, volume, holders, velocity)
3. Calls `scan()` on each active behavioral strategy, passing the snapshot
4. Collects returned signals and emits `signal_detected` events on the event bus
5. Caches signals for API reads, deduplicating by token mint + signal type + source
6. Passes signals to the `StrategyEngine` for matching against DB strategies

The Scanner itself doesn't make trading decisions — it orchestrates the behavioral strategies and feeds their output into the matching pipeline. Rate limiting (150ms min between venue requests) prevents API abuse.

**Signal Cache:** Signals are cached for API reads (`GET /scanner/signals`). Cache eviction uses `expires_at` (signal expiry) rather than detection time, so signals remain visible and re-insertable as long as they haven't expired. This prevents a blind window where expired signals block re-detection.

### What the StrategyEngine Does

The StrategyEngine (`src/agents/strategy_engine.rs`) is the bridge between signals and execution:

1. Receives `Vec<Signal>` from the Scanner
2. For each signal: checks expiry, checks dedup cache, attempts to match against all active DB strategies
3. On match: creates an `Edge` (a validated trading opportunity with risk parameters attached)
4. Emits `edge_detected` event
5. The `AutonomousExecutor` listens for edges and handles execution based on the strategy's `execution_mode`

### What the AutonomousExecutor Does

The AutonomousExecutor (`src/agents/autonomous_executor.rs`) handles the buy side:

1. Listens for `edge_detected` events
2. Checks capital allocation via `CapitalManager`
3. Validates against risk limits
4. Builds the buy transaction (pump.fun for curves, Jupiter for DEX)
5. Signs with the dev wallet, submits via Jito bundles
6. On success: registers position with `PositionManager`

For `autonomous` strategies, execution is immediate. For `agent_directed` strategies, the executor emits an approval request and waits for LLM consensus.

---

## Graduation Sniper

**Files:**
- `src/agents/graduation_sniper.rs` — Graduation event monitoring
- `src/agents/strategies/graduation_sniper_strategy.rs` — Behavioral strategy (signal generation)

The Graduation Sniper is the primary active strategy. It targets tokens approaching graduation (migration from pump.fun bonding curve to Raydium DEX).

### How It Works

The `GraduationSniperStrategy` behavioral strategy scans venue snapshots for tokens with high graduation progress:

```
VenueSnapshot arrives from Scanner
  → Filter: token age <= 48h if progress >= 85% (stale token filter in pump_fun.rs)
  → Filter: graduation_progress >= 85%
  → Calculate velocity (volume / market_cap ratio)
  → Filter: velocity >= 0.1 (or progress >= 95%)
  → Calculate confidence score
  → Emit Signal(CurveGraduation)
```

**Stale Token Filter:** Tokens older than 48 hours that are still at >=85% progress are skipped by the scanner. These are likely stuck tokens that will never graduate.

The `StrategyEngine` matches these signals to the `graduation_snipe` DB strategy (which only accepts signals with `progress_percent >= 85`), creates an Edge, and the AutonomousExecutor places the buy.

### Signal Confidence Scoring

**File:** `src/agents/strategies/graduation_sniper_strategy.rs`

Confidence is calculated from three weighted factors: graduation progress, price velocity, and holder count.

**When holder count is known (> 0):**

| Factor | Weight | Calculation |
|--------|--------|-------------|
| Progress | 60% | `((progress - 85) / 15).clamp(0, 1)` |
| Velocity | 25% | `(velocity * 5).min(1)` |
| Holders | 15% | `(holders / 50).min(1)` |

**When holder count is unknown (== 0):**

The 15% holder weight is redistributed to avoid penalizing tokens where bulk holder lookups aren't available:

| Factor | Weight | Calculation |
|--------|--------|-------------|
| Progress | 75% | `((progress - 85) / 15).clamp(0, 1)` |
| Velocity | 25% | `(velocity * 5).min(1)` |

This prevents a systematic 15% confidence penalty on tokens scanned via the pump.fun bulk API (which doesn't return holder counts).

### Phase 1: Pre-Graduation Entry

**Trigger:** Token at 90%+ graduation progress

```
Scanner detects high-progress curve
  → GraduationSniperStrategy generates signal
  → StrategyEngine matches to graduation_snipe
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

### Phase 2: Post-Graduation Quick-Flip (Raydium Snipe)

See [Raydium Snipe](#raydium-snipe) strategy below for the post-graduation buy implementation.

### Velocity Data

The scanner now fetches per-token trade data from the pump.fun API for tokens at 70%+ graduation progress. This provides non-zero velocity values for the graduation sniper's momentum scoring.

The per-token endpoint (`GET /coins/{mint}`) returns `volume_24h_usd` which is used to calculate velocity as `volume / market_cap`. Only high-progress tokens are queried to avoid rate limiting.

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

### Data Sources & Token Discovery

**File:** `src/venues/curves/pump_fun.rs`

Token discovery uses a two-URL architecture:

| URL | Purpose | Default |
|-----|---------|---------|
| `pump_api_url` | Bulk scanning (`/coins/currently-live`) | `https://frontend-api-v3.pump.fun` |
| `dexscreener_url` | Individual token info (`/tokens/{mint}`) | `https://api.dexscreener.com/latest/dex` |

**Why pump.fun API for scanning:** DexScreener's `/search?q=pumpfun` returns ~30-50 curated/trending pairs. Tokens in the 85-99% graduation window ($58.6k-$69k market cap) almost never appeared. The pump.fun `/coins/currently-live` endpoint returns actively-trading tokens including those near graduation.

**Fallback:** If the pump.fun API fails, scanning falls back to DexScreener search automatically.

**USD vs SOL market cap:** The pump.fun API provides both `usd_market_cap` (USD) and `market_cap` (SOL). The scanner passes `market_cap_sol` through to strategies so velocity calculations use correct SOL-denominated values. When SOL data is unavailable (DexScreener fallback), USD values are used as before.

### Metrics Collection

**Files:** `src/agents/curve_metrics.rs`, `src/handlers/curves.rs`

The metrics collector populates detailed token metrics from venue APIs:

```
┌─────────────────────────────────────────────────────────────────┐
│                         DATA SOURCES                              │
├─────────────────────────────────────────────────────────────────┤
│  pump.fun API           DexScreener API      On-Chain RPC       │
│  (Scanning, Holders)    (Token info)         (Curve state)      │
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
| `holder_count` | pump.fun API (per-token) | Helius largest accounts count |
| `top_10_concentration` | pump.fun API | Helius calculation |
| `price_momentum_24h` | DexScreener | 0.0 |
| `graduation_progress` | On-chain RPC | Estimated from market cap |
| `market_cap_sol` | pump.fun API / On-chain RPC | DexScreener estimate (USD) |
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
5. Apply `ExitConfig::for_defensive()` (15% TP, strong momentum extends)

### Entry Parameters (from Global Risk Config)

| Parameter | Default | Description |
|-----------|---------|-------------|
| `max_position_sol` | 0.08 SOL | Maximum buy amount (reduced from 0.30 per LLM consensus - 18% win rate = smaller bets) |
| `slippage_bps` | 1500 (15%) | Bonding curve slippage |

---

## KOL Copy Trading (WIP)

> **Status:** Infrastructure fully wired with a bypass implementation. Strategy stub (`kol_copy_strategy.rs`) created for future unification into the standard pipeline. The current bypass works but doesn't flow through Scanner → StrategyEngine like other strategies.

**Files:**
- `src/agents/kol_discovery.rs` — KOL wallet discovery and trust scoring
- `src/execution/copy_executor.rs` — Copy trade execution
- `src/handlers/webhooks.rs` — Helius webhook handler for trade detection
- `src/agents/strategies/kol_copy_strategy.rs` — WIP strategy stub (not registered yet)

> **Requires Public URL:** KOL copy trading depends on Helius webhooks pushing wallet activity to ArbFarm. This only works when deployed with a publicly accessible URL (not localhost). See CLAUDE.md "AWS Deployment TODO" section for setup instructions.

### Intended Architecture

KOL copy trading should eventually flow through the same pipeline as all other strategies:

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
KolCopyStrategy.push_trade(event)  ← buffer the event
    │
    ▼
Scanner calls strategy.scan() on next polling cycle
    │
    ▼
scan() drains buffer → Vec<Signal(KolTrade)>
    │
    ▼
StrategyEngine.process_signals()
    │
    ├── Check signal expiry (30s TTL for KOL trades)
    ├── Check dedup cache
    ├── Match to "copy_trade" DB strategy
    │     ├── Trust score >= min_trust_score?
    │     └── Token whitelist/blacklist check?
    │
    ▼
Create Edge → AutonomousExecutor.handle_edge_detected()
    │
    ▼
Standard buy/sell execution via CopyTradeExecutor
```

**Why KOL data is NOT a new VenueType:** Venues are poll-based market data sources (pump.fun API, DexScreener, on-chain RPC). KOL trades are push-based wallet activity from Helius webhooks. Instead of adding a `VenueType::Kol`, the strategy uses an internal buffer pattern: webhooks push events in, and `scan()` drains them. This keeps the Scanner → StrategyEngine pipeline uniform.

**What needs to change to wire this up:**
1. `webhooks.rs` — Call `kol_copy_strategy.push_trade(event)` instead of (or in addition to) emitting `kol_topics::TRADE_DETECTED`
2. `server.rs` — Register `KolCopyStrategy` with `StrategyRegistry` and create a `"copy_trade"` DB strategy record
3. `strategy_engine.rs` — Add `"copy_trade"` match arm in `signal_matches_strategy()` for trust score filtering
4. `autonomous_executor.rs` — Remove or gate the direct `handle_kol_trade()` bypass behind a feature flag

### Current Bypass Implementation

The current implementation bypasses the Scanner → StrategyEngine pipeline entirely:

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

This bypass works but means KOL trades don't benefit from signal deduplication, strategy-level risk params, or the StrategyEngine's matching/filtering pipeline.

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

## Raydium Snipe

**Files:**
- `src/agents/strategies/raydium_snipe_strategy.rs` — RaydiumSnipeStrategy (signal generation from push buffer)
- `src/agents/scanner.rs` — Graduation detection (polls `complete=true`, compares to contender cache)

The Raydium Snipe strategy detects when a token graduates from pump.fun's bonding curve to Raydium and immediately buys via Jupiter swap.

### How It Works

```
Scanner polls pump.fun API for recently graduated tokens
  → Compare against known contenders (tokens we were tracking)
  → If token just graduated (complete=true, raydium_pool set)
  → Push GraduationEvent into RaydiumSnipeStrategy buffer
  → Strategy drains buffer on next scan() call
  → Emit Signal(CurveGraduation, source=raydium_snipe)
  → StrategyEngine matches to raydium_snipe DB strategy
  → AutonomousExecutor routes to build_post_graduation_buy() (Jupiter swap)
```

### Graduation Detection

The scanner maintains a `graduated_mints` cache (TTL: 30 minutes) to prevent duplicate signals. Each scan cycle:

1. Fetch `GET /coins?complete=true&sort=last_trade_timestamp&order=DESC&limit=20`
2. For each graduated token with a `raydium_pool`:
   - Skip if already in `graduated_mints` cache
   - Skip if token was never tracked as a contender
   - Push `GraduationEvent` into the strategy's buffer
   - Add to `graduated_mints` cache

### Signal Properties

| Property | Value | Rationale |
|----------|-------|-----------|
| Signal Type | `CurveGraduation` | Reuses existing variant |
| Signal Source | `raydium_snipe` | Distinguishes from `graduation_sniper` |
| Venue Type | `DexAmm` | Token is now on Raydium |
| Confidence | 0.85 | High — graduation confirmed, pool confirmed |
| Expiry | 60 seconds | Very time-sensitive |
| Significance | Critical | Immediate action required |

### Exit Config (`ExitConfig::for_raydium_snipe()`)

Fast-flip exit strategy for post-graduation price discovery:

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| Stop Loss | 15% | Tight — cut losses fast if post-grad dump |
| Take Profit | 30% | Capture initial DEX price discovery pump |
| Trailing Stop | 10% | Lock in gains during pump |
| Time Limit | 5 minutes | Quick flip, don't hold |
| Partial TP #1 | 60% at 20% gain | Recover capital + profit early |
| Partial TP #2 | 100% at 30% gain | Exit remaining position |
| Momentum Adaptive | Disabled | Speed over optimization |

### Execution Routing

When `signal_source == "raydium_snipe"`:
- Buy: `build_post_graduation_buy()` — Jupiter swap (SOL → Token)
- Sell: `build_raydium_sell()` — Raydium Trade API (existing position exit logic)

### Controls

Raydium Snipe is always in observation mode unless execution is enabled:
- Requires `ARBFARM_SNIPER_ENTRY=1` (shared with graduation sniper)
- Signals are always generated and visible in `/scanner/signals`

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
    auto_exit_enabled: bool,    // When false, position monitor skips auto-exit checks
}
```

---

## Exit Strategies

**Detection:** `src/execution/position_monitor.rs` (checks exit conditions, sends `PositionCommand` via mpsc channel)
**Execution:** `src/execution/position_executor.rs` (receives commands, builds/signs/submits sell transactions)

### Exit Config: Defensive (DEFAULT)

**All strategies now use defensive mode.** Take profits at 15% unless momentum is strong.

```
Normal/Weak Momentum:
  → Exit at 15% (or 7.5% for weak)
  → Full exit, no partials
  → Quick protection of gains

Strong Momentum:
  → Target extends to 30% (100% extension)
  → Partial exit: 30% at first target
  → Remaining 70% rides to 50%+ extended target
  → Exit ASAP on any momentum decay
```

**Configuration:**

| Parameter | Value | Notes |
|-----------|-------|-------|
| Stop Loss | 10% | Tight capital protection |
| Take Profit (Base) | 15% | Covers ~4% fees + profit |
| Take Profit (Strong) | 30%+ | Extended with strong momentum |
| Trailing Stop | 8% | Tight protection |
| Time Limit | 5 minutes | Quick exits |

**Momentum-Adaptive Logic:**
- **Strong momentum:** targets extend by 100% (15% → 30%), exits reduced to 30%
- **Normal momentum:** exit at base 15% target
- **Weak momentum:** target reduced by 50% (15% → 7.5%), exit aggressively
- **Reversal:** immediate full exit to protect any profit

**Why Defensive Mode:**
- Fees are ~4% round-trip (entry + exit + slippage)
- 15% profit = ~11% after fees
- Strong momentum can still run (targets extend significantly)
- Quick exit on decay protects capital
- Tight 10% stop loss prevents large drawdowns

### Legacy Exit Configs

These configs are preserved and can be used via direct API calls:

**Scanner (for_scanner):** 25% TP, 20% SL, 3 min
**Sniper (for_graduation_sniper):** 50% TP, 40% SL, 15 min
**Curve Bonding (for_curve_bonding):** 100% TP, 40% SL, 15 min

### Exit Triggers

The Position Monitor checks these conditions every 2 seconds and sends matching signals as `PositionCommand::Exit` to the PositionExecutor via mpsc channel:

```rust
enum ExitReason {
    Emergency,          // CIRCUIT BREAKER: -50% catastrophic loss protection
    StopLoss,           // Price dropped below configured SL
    TakeProfit,         // Hit take profit target
    TrailingStop,       // Dropped from peak
    TimeLimit,          // Held > time limit
    PartialTakeProfit,  // Tiered exit phase
    MomentumDecay,      // Velocity declining sustained
    MomentumReversal,   // Strong reversal detected
    Salvage,            // Dead token recovery
    Manual,             // User-triggered exit
}
```

### Emergency Exit Circuit Breaker

Prevents catastrophic losses from rug pulls and crashes.

```
If unrealized_pnl_percent <= -50%:
  → IMMEDIATELY exit at Critical urgency
  → Bypasses all other exit logic
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

**Peak Drop Protection:** If position is profitable and has dropped 6%+ from peak P&L, exit to protect gains.

### Slippage Calculation

**File:** `src/execution/position_executor.rs`

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

**File:** `src/execution/position_executor.rs`

The PositionExecutor receives `PositionCommand` messages from multiple sources via mpsc channel, sorts by urgency (Critical first), deduplicates by position ID, then executes:

```
Sources:
  PositionMonitor ──────────┐
  CopyTradeExecutor ────────┤  mpsc channel  ──► PositionExecutor
  Manual API / RealtimeMonitor ──┘

Execution:
1. Build sell transaction:
   - Pre-graduation: pump.fun bonding curve
   - Post-graduation: Raydium (direct) → Jupiter (fallback)
2. Sign with DevWallet
3. Try Jito bundle (priority fee for speed)
4. If Jito fails/times out → Helius fallback
5. On timeout → Check wallet balance
   - If tokens gone → Treat as success (inferred exit)
   - If tokens remain → Retry
6. Update position status + release capital
7. Record to engrams
```

### Jupiter Rate Limit Handling

Automatic retry with exponential backoff for Jupiter API rate limits.

```
Jupiter quote/swap request
  └─► 429 or "Rate limit" error?
        └─► Retry up to 3 times with exponential backoff:
              Attempt 1: immediate
              Attempt 2: 500ms delay
              Attempt 3: 1000ms delay
              Attempt 4: 2000ms delay
        └─► Still failing? Return error
```

**Configuration:**
- `JUPITER_RATE_LIMIT_RETRIES`: 4 attempts
- `JUPITER_RATE_LIMIT_BACKOFF_MS`: 500ms base (doubles each retry)

### Error Filtering

**"Already Sold" Errors:**
Errors indicating a token was already sold are expected states for positions closed via partial TP or inferred exits. These are now filtered from engram error logging to reduce noise:
- "already sold"
- "zero on-chain balance"
- "balance" + "0"

These errors still appear in logs but don't pollute the learning engrams.

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
    max_position_sol: f64,           // 0.08 SOL default (reduced from 0.30 per LLM consensus)
    daily_loss_limit_sol: f64,       // 1.0 SOL default
    max_drawdown_percent: f64,       // 40% default (increased from 30% per LLM consensus)
    max_concurrent_positions: u32,   // 10 default
    cooldown_after_loss_ms: u64,     // 5000ms default
    auto_pause_on_drawdown: bool,    // true default
}
```

> **Note:** `max_position_sol` reduced from 0.30 to 0.08 SOL based on LLM consensus analysis of 18% win rate - smaller bets reduce drawdown on losing streaks. `max_drawdown_percent` increased from 30% to 40% to reduce premature exits on recoverable dips. Emergency circuit breaker at -50% provides catastrophic loss protection.

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

## On-Chain PnL Settlement

**Files:** `src/execution/tx_settlement.rs`, `src/helius/client.rs`

After each confirmed buy or sell transaction, the system fetches actual on-chain data via Solana's `getTransaction` RPC (Helius) to compute exact PnL:

```
1. Transaction confirmed → get signature
2. Call getTransaction (up to 3 retries, 2s delay between)
3. Find wallet pubkey in transaction accountKeys
4. Compute SOL delta = postBalances[wallet_idx] - preBalances[wallet_idx]
5. Use SOL delta as realized PnL (includes all fees, slippage, priority fees)
6. Record gas_lamports from transaction meta fee field
```

### Settlement Sources

| Source | Meaning |
|--------|---------|
| `onchain` | Used actual `getTransaction` data — PnL is exact |
| `estimated` | RPC failed or unavailable — PnL uses estimated price math |

### Graceful Fallback

If `getTransaction` fails after 3 retries (RPC down, transaction not indexed), the system falls back to the previous estimated PnL calculation (`effective_base * pnl_percent`). Trade records are never blocked on settlement resolution.

### Gas Tracking

Both buy and sell gas costs (transaction fees in lamports) are persisted:
- `entry_gas_lamports` — gas paid on the buy transaction
- `exit_gas_lamports` — gas paid on the sell transaction
- `total_gas_sol` — aggregated in PnL summary for the UI

### DB Columns (migration 015)

```sql
ALTER TABLE arb_trades ADD COLUMN entry_gas_lamports BIGINT;
ALTER TABLE arb_trades ADD COLUMN exit_gas_lamports BIGINT;
ALTER TABLE arb_trades ADD COLUMN pnl_source TEXT DEFAULT 'estimated';
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
| `/positions/{id}/auto-exit` | PATCH | Toggle auto-exit for position |
| `/positions/auto-exit-stats` | GET | Get auto-exit stats (auto vs manual count) |
| `/pnl/summary` | GET | P&L summary |
| `/scanner/start` | POST | Start scanner |
| `/scanner/stop` | POST | Stop scanner |
| `/sniper/start` | POST | Start sniper |
| `/sniper/stop` | POST | Stop sniper |
| `/execution/toggle` | POST | Enable/disable execution engine |
| `/config/risk` | GET/POST | Risk configuration |
| `/strategies` | GET | List all strategies |
| `/strategies/{id}/momentum` | POST | Toggle momentum-adaptive exits |

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `ARB_SCANNER_AUTO_START` | true | Auto-start scanner |
| `ARBFARM_ENABLE_EXECUTOR` | 0 | Enable autonomous executor |
| `ARBFARM_SNIPER_ENTRY` | 0 | Enable sniper buy entries |
| `ARBFARM_COPY_TRADING` | 0 | Enable KOL copy trading |
| `ARBFARM_DISABLE_SNIPER` | 0 | Disable sniper entirely |
| `PUMP_FUN_API_URL` | `https://frontend-api-v3.pump.fun` | pump.fun API for bulk scanning |
| `DEXSCREENER_API_URL` | `https://api.dexscreener.com/latest/dex` | DexScreener API for token info |
| `/tmp/arb-no-scan` | - | Sentinel to disable scanner |
| `/tmp/arb-no-snipe` | - | Sentinel to disable sniper |

---

## Implementation Status

### Fully Implemented

- Graduation sniper (pre/post-grad)
- Pluggable strategy factory (BehavioralStrategy trait + StrategyRegistry)
- Raydium Trade API integration (post-graduation sells)
- Position manager with P&L tracking
- Position monitor with adaptive exits
- Strategy isolation (per-strategy budgets + position tracking)
- Tiered exit strategy
- Momentum-based exits (with API toggle)
- Risk management
- Engrams integration
- Dead token salvage

### WIP

- KOL Copy Trading (infrastructure wired, strategy registration disabled)

### Stubbed (Warnings Expected)

- DEX arbitrage (mev_hunter.rs)
- Raydium venue scanner (pool discovery)
- Direct Anthropic/OpenAI providers

### Not Implemented

- Threat detection (rug pulls)
- Research/DD agent
