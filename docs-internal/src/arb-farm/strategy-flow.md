# ArbFarm Strategy Flow

Complete guide to the strategy lifecycle from alpha discovery to profit realization.

## Overview

The strategy flow consists of 5 phases:
1. **Discovery** - Extract strategies from research URLs
2. **Detection** - Match signals against active strategies
3. **Decision** - Determine execution mode (auto/consensus)
4. **Execution** - Submit transactions via Jito bundles
5. **Learning** - Store patterns as engrams

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          STRATEGY LIFECYCLE                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  DISCOVERY                 DETECTION                EXECUTION               │
│  ─────────                 ─────────                ─────────               │
│                                                                             │
│  ┌─────────┐             ┌───────────┐            ┌───────────┐           │
│  │Research │──────────▶  │  Scanner  │─────────▶  │ Executor  │           │
│  │  Agent  │             │   Agent   │            │           │           │
│  └────┬────┘             └─────┬─────┘            └─────┬─────┘           │
│       │                        │                        │                  │
│       ▼                        ▼                        ▼                  │
│  ┌─────────┐             ┌───────────┐            ┌───────────┐           │
│  │Strategy │             │   Edge    │            │   Trade   │           │
│  │Extracted│             │ (Opport.) │            │  Result   │           │
│  └────┬────┘             └─────┬─────┘            └─────┬─────┘           │
│       │                        │                        │                  │
│       ▼                        ▼                        ▼                  │
│  ┌─────────┐             ┌───────────┐            ┌───────────┐           │
│  │Backtest │             │ Consensus │            │  Engram   │           │
│  │Validate │             │  (if req) │            │  Storage  │           │
│  └─────────┘             └───────────┘            └───────────┘           │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Phase 1: Discovery (Research Agent)

The discovery phase turns raw alpha into actionable strategies.

### URL Injection

1. User submits a URL (Twitter thread, blog post, whitepaper)
2. Content is fetched and preprocessed
3. LLM extracts strategy parameters

### Extracted Parameters

| Parameter | Description |
|-----------|-------------|
| Strategy Type | dex_arb, curve_graduation, liquidation, kol_copy, momentum |
| Entry Conditions | Rules that trigger trade entry |
| Exit Conditions | Rules that trigger trade exit |
| Risk Parameters | Position size, slippage, loss limits |
| Confidence Score | LLM's confidence in the extraction (0-1) |

### Workflow

```
URL Input → Fetch Content → LLM Analysis → Strategy Extraction → Validation
```

1. Navigate to Research view
2. Paste URL in the injection form
3. Click "Analyze"
4. Review extracted strategy
5. Either:
   - **Create Strategy** - Add directly to active strategies
   - **Backtest First** - Validate against historical data

## Phase 2: Detection (Scanner Agent)

The scanner continuously monitors venues for opportunities.

### Signal Types

| Signal Type | Source | Description |
|-------------|--------|-------------|
| `price_discrepancy` | DEX AMMs | Price difference between venues |
| `volume_spike` | All venues | Unusual volume activity |
| `dex_arb` | Jupiter/Raydium | Cross-DEX arbitrage |
| `liquidation` | Lending protocols | Liquidatable positions |
| `curve_graduation` | pump.fun/moonshot | Bonding curve graduation imminent |
| `kol_signal` | Wallet tracking | KOL made a trade |

### Signal → Edge Flow

```
Signal Detected → Strategy Match → Edge Created → Approval Queue
```

1. Scanner detects a signal
2. Signal is matched against active strategies
3. If match found, an Edge (opportunity) is created
4. Edge enters the approval queue

### Signal Filtering

The Signals view supports filtering by:
- Signal type
- Venue type
- Minimum profit (bps)
- Minimum confidence

## Phase 3: Decision (Consensus Engine)

How the system decides whether to execute.

### Execution Modes

| Mode | Behavior | Use Case |
|------|----------|----------|
| **Agent Directed** | Multi-LLM consensus for every trade | Most conservative, highest confidence |
| **Hybrid** | Auto below threshold, consensus above | Balanced approach |
| **Autonomous** | Full auto-execution | Speed-critical opportunities |

### Consensus Process

For Agent Directed and Hybrid modes:

1. Edge is sent to multiple LLMs (via OpenRouter)
2. Each LLM votes: Approve, Reject, or Abstain
3. Consensus reached based on voting rules
4. If approved, edge moves to execution

### Threat Checks

Parallel to consensus:
- Token threat score calculation
- Wallet risk assessment
- Rug pull detection
- Honeypot analysis

## Phase 4: Execution (Executor Agent)

Turning opportunities into trades.

### Execution Pipeline

```
Edge Approved → Simulation → Bundle Creation → Jito Submission → Confirmation
```

1. **Simulation** - Verify trade will succeed
2. **Bundle Creation** - Create Jito bundle for MEV protection
3. **Submission** - Send to Jito block engine
4. **Confirmation** - Wait for on-chain confirmation

### Atomicity Levels

| Level | Risk | Strategy |
|-------|------|----------|
| **FullyAtomic** | Zero capital risk | Flash loans, Jito bundles |
| **PartiallyAtomic** | Mixed | Some legs protected |
| **NonAtomic** | Capital at risk | Cross-venue, delayed exit |

## Phase 5: Learning (Engram Harvester)

Converting experience into persistent memory.

### Engram Types

| Type | Pattern | Purpose |
|------|---------|---------|
| Trade Result | `arb.trade.{tx_sig}` | Record of executed trade |
| Pattern | `arb.pattern.{type}.{venue}` | Winning trade patterns |
| Avoidance | `arb.avoid.{entity_type}.{address}` | Entities to avoid |
| Agent State | `arb.state.{timestamp}` | Periodic state snapshots |

### Auto-Save Triggers

- **On Trade Success** → TradeResultEngram + PatternEngram
- **On Trade Failure** → AvoidanceEngram (if systemic failure)
- **Periodic** → AgentStateEngram (every 5 minutes)

## Risk Management

### Recommended Settings

| Parameter | Conservative | Moderate | Aggressive |
|-----------|--------------|----------|------------|
| Max Position | 0.1 SOL | 0.5 SOL | 2.0 SOL |
| Min Profit | 50 bps | 25 bps | 10 bps |
| Max Slippage | 25 bps | 50 bps | 100 bps |
| Daily Loss Limit | 0.5 SOL | 2.0 SOL | 5.0 SOL |
| Execution Mode | Agent-Directed | Hybrid | Autonomous |

### Position Sizing by Atomicity

| Atomicity | Sizing Strategy |
|-----------|-----------------|
| Fully Atomic | Can use larger positions (lower risk) |
| Partially Atomic | Standard position sizing |
| Non-Atomic | Reduce position size (higher risk) |

## Monitoring Dashboards

| View | Purpose |
|------|---------|
| Dashboard | Overall P&L, swarm health, active strategies |
| Opportunities | Live edge feed, approval queue |
| Signals | Real-time signal detection |
| Strategies | Strategy management, performance |
| Research | URL injection, discoveries |
| Threats | Blocked tokens, risk alerts |
| Settings | Risk config, wallet, execution mode |

## Best Practices

### Getting Started

1. Start with Agent-Directed mode
2. Use conservative risk settings
3. Begin with DEX arbitrage (most reliable)
4. Monitor for a few days before increasing exposure

### Profit Optimization

1. **DEX Arb** - Most reliable, lowest risk when atomic
2. **Curve Graduation** - Higher variance, requires timing
3. **KOL Copy** - Follow proven wallets with delay mitigation
4. **Liquidation** - Capital efficient but competitive

### Common Failure Modes

| Issue | Cause | Mitigation |
|-------|-------|------------|
| Slippage > Expected | Market moved | Use tighter slippage limits |
| Rug Pull | Malicious token | Enable threat detection |
| Front-run | MEV extraction | Use Jito bundles |
| Stale Data | Signal expired | Reduce scan interval |
