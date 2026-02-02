# ArbFarm: Solana MEV Agent Swarm

**NullBlock's Solana Arbitrage & MEV Agent Swarm** - Autonomous multi-agent system for capturing MEV opportunities on Solana with intelligent threat detection and composable event architecture.

---

## GOLDEN RULE: Protocol-First Event Architecture

**ALL agent-to-agent and tool communications MUST use standardized protocols with subscribable events. This enables future agents to plug in and consume events without modifying existing code.**

### Protocol Standards

| Communication Type | Protocol | Purpose |
|-------------------|----------|---------|
| Agent ↔ Agent | A2A | Inter-agent coordination, task delegation |
| Agent ↔ Tools | MCP | Tool invocation, resource access |
| Events → Subscribers | Event Bus | Publish/subscribe for all system events |
| External → Service | REST/SSE | HTTP API and real-time streams |

### Event Bus Architecture

Every agent and tool MUST emit events to a central event bus. Events are:
1. **Persisted to DB** for history and replay
2. **Streamed via SSE** for real-time subscribers
3. **Subscribable by any agent** via topic filters

```rust
pub struct ArbEvent {
    pub id: EventId,
    pub event_type: EventType,
    pub source: EventSource,       // Which agent/tool emitted
    pub topic: String,             // e.g., "arb.edge.detected", "arb.trade.executed"
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Option<String>, // Link related events
}

pub enum EventSource {
    Agent(AgentType),
    Tool(String),
    External(String),
}

pub enum AgentType {
    Scanner,
    Refiner,
    MevHunter,
    Executor,
    StrategyEngine,
    ResearchDD,
    CopyTrade,
    ThreatDetector,
    EngramHarvester,
    Overseer,
}
```

### Event Topics (Subscribable)

```
arb.scanner.*                    # All scanner events
arb.scanner.signal.detected      # New signal detected
arb.scanner.venue.added          # New venue discovered

arb.edge.*                       # All edge events
arb.edge.detected                # New opportunity found
arb.edge.approved                # Edge approved for execution
arb.edge.rejected                # Edge rejected
arb.edge.executing               # Trade in progress
arb.edge.executed                # Trade completed
arb.edge.failed                  # Trade failed

arb.strategy.*                   # Strategy events
arb.strategy.created             # New strategy added
arb.strategy.updated             # Strategy modified
arb.strategy.triggered           # Strategy conditions met

arb.research.*                   # Research/DD events
arb.research.url.ingested        # URL analyzed
arb.research.strategy.discovered # New strategy found
arb.research.strategy.approved   # Strategy approved for testing

arb.kol.*                        # KOL tracking events
arb.kol.trade.detected           # KOL made a trade
arb.kol.trade.copied             # We copied the trade
arb.kol.trust.updated            # Trust score changed

arb.threat.*                     # Threat detection events
arb.threat.detected              # Threat identified
arb.threat.blocked               # Entity blocked
arb.threat.alert                 # Alert raised

arb.engram.*                     # Engram events
arb.engram.created               # New engram stored
arb.engram.pattern.matched       # Pattern recognized
arb.engram.avoidance.created     # Avoidance rule created
arb.engram.strategy.optimized    # Strategy optimized via engram analysis

arb.consensus.*                  # Consensus events
arb.consensus.requested          # Consensus request initiated
arb.consensus.completed          # Consensus decision completed
arb.consensus.failed             # Consensus failed (timeout/disagreement)

arb.swarm.*                      # Swarm health events
arb.swarm.agent.started          # Agent spawned
arb.swarm.agent.failed           # Agent crashed
arb.swarm.agent.recovered        # Agent restarted
```

### Subscribing to Events (Any Agent/Tool)

**MCP Resource for Event Subscription**:
```yaml
erebus://arb/events:
  description: "Subscribe to ArbFarm events"
  params:
    topics: ["arb.edge.*", "arb.trade.*"]  # Filter by topic pattern
    since: "2024-01-01T00:00:00Z"          # Replay from timestamp
```

**Example: Content Generation Agent (Future Fork)**:
```rust
// A new agent that subscribes to events and generates X content
impl ContentAgent {
    async fn start(&self, event_bus: &EventBus) {
        // Subscribe to interesting events
        let subscription = event_bus.subscribe(vec![
            "arb.edge.executed",
            "arb.research.strategy.discovered",
            "arb.threat.detected",
        ]).await;

        while let Some(event) = subscription.next().await {
            match event.topic.as_str() {
                "arb.edge.executed" => {
                    let trade: TradeResult = serde_json::from_value(event.payload)?;
                    if trade.profit_sol > 0.1 {
                        self.generate_win_tweet(trade).await;
                    }
                }
                "arb.threat.detected" => {
                    let threat: ThreatAlert = serde_json::from_value(event.payload)?;
                    self.generate_warning_tweet(threat).await;
                }
                _ => {}
            }
        }
    }
}
```

### Composability Example

**Adding a Content Generation Agent to an existing fork:**

1. **Fork ArbFarm COW**
2. **Create new agent**: `agents/content_gen.rs`
3. **Subscribe to events**:
   ```rust
   event_bus.subscribe(["arb.edge.executed", "arb.threat.detected"])
   ```
4. **Process events and generate content**
5. **Publish own events**: `arb.content.tweet.generated`
6. **Other agents can subscribe** to content events

**No modification to existing agents required** - just plug in and subscribe!

---

## Core Concepts

### Abstract MEV Venue Model

Different MEV opportunities (DEXes, bonding curves, lending, etc.) share common patterns. We model them abstractly to support any venue type.

```rust
pub trait MevVenue {
    fn venue_type(&self) -> VenueType;
    fn scan_for_edges(&self) -> Vec<Edge>;
    fn estimate_profit(&self, edge: &Edge) -> ProfitEstimate;
    fn execute(&self, edge: &Edge) -> ExecutionResult;
}

pub enum VenueType {
    DexAmm,           // Jupiter, Raydium, Orca
    BondingCurve,     // pump.fun, moonshot
    Lending,          // Marginfi, Kamino
    Orderbook,        // Phoenix, OpenBook
}
```

### Multi-LLM Consensus Engine

Agent-directed trades require consensus from multiple LLMs to reduce hallucination risk and improve decision quality.

**How it works**:
1. Edge surfaced to agent via MCP/SSE
2. Agent queries multiple LLMs (via OpenRouter) with edge context
3. Each LLM returns approve/reject with confidence score
4. Weighted voting determines outcome
5. Consensus outcome stored as engram for pattern learning

```rust
pub struct ConsensusRequest {
    pub edge_id: Uuid,
    pub models: Vec<String>,  // ["claude-3-opus", "gpt-4-turbo", "llama-3-70b"]
    pub prompt_template: String,
    pub min_agreement: f32,   // 0.5 = majority
    pub timeout_secs: u32,
}

pub struct ConsensusResult {
    pub approved: bool,
    pub agreement_score: f32,
    pub model_votes: Vec<ModelVote>,
    pub reasoning_summary: String,
}
```

**Engram Integration**: Store consensus outcomes as engrams for pattern learning - which model combinations produce best results for which edge types.

### Atomic Profit Guarantees

Some MEV opportunities are **atomic** - profit is guaranteed or transaction reverts. These deserve aggressive execution with minimal risk checks since the only downside is gas cost on failure.

**Atomic Opportunity Types**:
| Type | Atomicity | Risk Profile |
|------|-----------|--------------|
| Flash Loan Arb | Fully Atomic | Zero capital risk (reverts if unprofitable) |
| Bundled DEX Arb | Fully Atomic | Zero capital risk (Jito bundle fails atomically) |
| Sandwich (Jito) | Fully Atomic | Fails if frontrun doesn't setup backrun |
| JIT Liquidity | Fully Atomic | LP provided and withdrawn in same block |
| Multi-leg Arb | Depends | Atomic if all legs in single tx/bundle |

**Non-Atomic (Capital at Risk)**:
| Type | Risk | Description |
|------|------|-------------|
| Cross-chain Arb | High | Bridge delay exposes to price movement |
| Delayed Exit | Medium | Entry and exit in separate transactions |
| Copy Trading | Medium | Following KOL with delay |

**Atomicity Classification**:
```rust
pub enum AtomicityLevel {
    FullyAtomic,      // Reverts if unprofitable, zero capital risk
    PartiallyAtomic,  // Some legs atomic, some not
    NonAtomic,        // Traditional trade with capital at risk
}

pub struct Edge {
    // ... existing fields ...
    pub atomicity: AtomicityLevel,
    pub simulated_profit_guaranteed: bool,  // Simulation passed
    pub max_gas_cost: u64,                  // Only risk on atomic trades
}
```

**Aggressive Config for Atomic Trades**:
```rust
pub struct AtomicTradeConfig {
    pub priority: Priority::Critical,       // Execute ASAP
    pub skip_agent_approval: bool,          // Auto-execute (no consensus needed)
    pub max_gas_willing: u64,               // Higher gas tolerance
    pub min_profit_after_gas_bps: u16,      // Lower threshold (e.g., 10bps)
    pub retry_with_higher_gas: bool,        // Aggressive retry
    pub bundle_tip_bps: u16,                // Tip for Jito priority
}
```

**Detection Flow**:
```
Edge Detected
     │
     ▼
┌─────────────────────┐
│ Classify Atomicity  │
│ - All legs in 1 tx? │
│ - Flash loan used?  │
│ - Jito bundle?      │
└──────────┬──────────┘
           │
    ┌──────┴──────┐
    ▼             ▼
  Atomic      Non-Atomic
    │             │
    ▼             ▼
Simulate      Normal
Transaction   Risk Flow
    │
    ▼
┌─────────────────────┐
│ Simulation Success? │
└──────────┬──────────┘
     ┌─────┴─────┐
     ▼           ▼
   Yes          No
     │           │
     ▼           ▼
Execute      Discard
Aggressively (no capital loss)
```

### Dynamic Risk Tuning

Risk parameters adjust based on multiple signals:

| Signal | Adjustment |
|--------|------------|
| High volatility | Reduce position sizes |
| Drawdown > threshold | Pause autonomous trades |
| New token (< 24h) | Require manual approval |
| Research agent flags | Adjust strategy risk scores |
| Failed trade streak | Reduce aggression |

---

## Swarm Architecture

```
                    ┌─────────────────────┐
                    │  Resilience Overseer │
                    │    (Meta-Agent)      │
                    └──────────┬──────────┘
                               │ monitors/recovers
    ┌──────────────────────────┼──────────────────────────┐
    │                          │                          │
┌───▼───┐  ┌───────┐  ┌───────▼───────┐  ┌───────┐  ┌───▼───┐
│Scanner│─▶│Refiner│─▶│Strategy Engine│─▶│Executor│─▶│Engram │
│       │  │       │  │               │  │        │  │Harvest│
└───┬───┘  └───────┘  └───────┬───────┘  └────────┘  └───────┘
    │                         │
    │  ┌──────────────────────┼──────────────────────┐
    │  │                      │                      │
    │  ▼                      ▼                      ▼
    │ ┌────────┐        ┌──────────┐          ┌──────────┐
    │ │Research│        │Copy Trade│          │  Threat  │
    │ │  /DD   │        │  Agent   │          │ Detector │
    │ └────────┘        └──────────┘          └──────────┘
    │
    ▼
┌─────────┐
│MEV      │
│Hunter   │
└─────────┘
```

---

## Agent Definitions

### 1. Venue Scanner Agent
**Purpose**: Continuously scan Solana for MEV opportunities across all venue types

**Responsibilities**:
- Monitor DEX pools (Jupiter, Raydium, Orca)
- Track bonding curve tokens (pump.fun, moonshot)
- Watch lending positions (Marginfi, Kamino)
- Detect price discrepancies across venues
- Emit raw signals via event bus

**Data Sources**:
- Helius RPC (enhanced Solana node)
- Jupiter Price API
- Raydium SDK
- pump.fun API
- Birdeye API

### 2. Signal Refiner Agent
**Purpose**: Filter and enrich raw scanner signals

**Responsibilities**:
- Calculate true profitability (accounting for fees, slippage)
- Apply historical pattern matching from engrams
- Score signal quality (1-100)
- Flag signals for manual review vs auto-execute
- Enrich with market context

### 3. MEV Hunter Agent
**Purpose**: Specialized detection for advanced MEV patterns

**Types Detected**:
| Pattern | Description |
|---------|-------------|
| DEX Arbitrage | Price differences across AMMs |
| Liquidation | Undercollateralized lending positions |
| JIT Liquidity | Just-in-time LP provision before large swaps |
| Sandwich | Frontrun + backrun profitable swaps |
| Backrun | Profit from price impact of large trades |

### 4. Execution Engine Agent
**Purpose**: Execute approved trades with maximum reliability

**Capabilities**:
- Jito bundle submission (priority ordering)
- Transaction simulation before execution
- Slippage protection
- Gas optimization
- Retry logic with exponential backoff
- Multi-leg atomic execution

**Execution Modes**:
| Mode | Trigger | Approval |
|------|---------|----------|
| Autonomous | High confidence, within risk params | Auto-execute |
| Hybrid | Medium confidence | Auto if profit > X, else surface |
| Agent-Directed | Low confidence, novel pattern | Require MCP approval |

### 5. Strategy Engine Agent
**Purpose**: Route opportunities to correct execution path based on strategy classification and atomicity

**Responsibilities**:
- Evaluate incoming edges against active strategies
- **Classify atomicity** (guaranteed profit vs capital-at-risk)
- Route atomic trades to aggressive execution path
- Route autonomous strategies directly to execution
- Surface agent-directed opportunities via MCP/SSE
- Apply dynamic risk adjustments before execution
- Track strategy performance for optimization

**Decision Flow** (with Atomicity):
```
Edge Detected
    │
    ▼
┌─────────────────────┐
│ Classify Atomicity  │
└────────┬────────────┘
         │
    ┌────┴────┐
    ▼         ▼
 Atomic    Non-Atomic
    │         │
    ▼         ▼
┌────────┐ ┌─────────────────┐
│Simulate│ │ Match Strategy? │──No──▶ Log & Discard
│  Tx    │ └────────┬────────┘
└───┬────┘          │ Yes
    │               ▼
    ▼          ┌─────────────────────┐
Success?       │ Within Risk Params? │──No──▶ Adjust or Reject
    │          └────────┬────────────┘
┌───┴───┐               │ Yes
▼       ▼               ▼
Yes     No         ┌─────────────────────┐
│       │          │  Execution Mode?    │
▼       ▼          └────────┬────────────┘
Execute Discard             │
AGGRESSIVE     ┌────────────┼────────────┐
(skip risk     ▼            ▼            ▼
 checks)     Auto        Hybrid       Agent
             │            │             │
             ▼            ▼             ▼
          Execute     Check Params   Surface
                                     to Agent
```

**Atomic Trade Execution**:
- Skip normal risk checks (only gas cost at risk)
- Lower profit threshold (10bps vs 50bps)
- Higher priority in Jito bundles
- Auto-retry with increasing gas
- No agent approval needed

**Bonding Curve Tools** (pump.fun, moonshot, etc.):
```
curve_buy_token        - Buy on any bonding curve
curve_sell_token       - Sell (before/after graduation)
curve_check_progress   - Check graduation progress %
curve_get_holder_stats - Top holders, concentration
curve_graduation_eta   - Estimated graduation time
```

### 6. Research/DD Agent
**Purpose**: Ingest external intelligence from URLs, social media, and perform due diligence

**Capabilities**:
- **URL Ingestion**: WebFetch-style analysis of links (tweets, articles, threads)
- **Strategy Extraction**: Parse trading strategies from content
- **X/Twitter Monitoring**: Track specified accounts for alpha
- **Backtesting**: Simulate discovered strategies against historical data

**Workflow**:
```
URL/Content Input
       │
       ▼
┌──────────────────┐
│ Content Analysis │
│ (LLM extraction) │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ Strategy Found?  │──No──▶ Store as intel engram
└────────┬─────────┘
         │ Yes
         ▼
┌──────────────────┐
│ Backtest Strategy│
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│  Profitable?     │──No──▶ Archive
└────────┬─────────┘
         │ Yes
         ▼
┌──────────────────┐
│ Surface for      │
│ Approval/Testing │
└──────────────────┘
```

**MCP Tools**:
```yaml
research_ingest_url:
  description: "Ingest and analyze a URL for trading strategies"
  params: [url, context]

research_monitor_account:
  description: "Add X/Twitter account to monitoring list"
  params: [handle, track_type]

research_backtest_strategy:
  description: "Backtest a discovered strategy"
  params: [strategy_id, period_days]

research_list_discoveries:
  description: "List discovered strategies pending review"
  params: [status, limit]
```

### 7. KOL Tracking + Copy Trade Agent
**Purpose**: Track Key Opinion Leaders and optionally copy their trades

**Features**:
| Feature | Description |
|---------|-------------|
| Wallet Tracking | Monitor specified wallets via Helius webhooks |
| Social Handle Tracking | Link X handles to wallets |
| Trust Scoring | Track KOL performance over time |
| Copy Trading | Auto-copy trades with configurable delay |
| Auto-Disable | Stop copying if trust score drops below threshold |

**Trust Score Calculation**:
```rust
pub struct TrustScore {
    pub total_trades_tracked: u32,
    pub profitable_trades: u32,
    pub avg_profit_percent: f32,
    pub max_drawdown: f32,
    pub consistency_score: f32,  // Low variance = higher score
    pub final_score: f32,        // Weighted combination (0-100)
}
```

**Copy Trade Config**:
```rust
pub struct CopyTradeConfig {
    pub entity_id: Uuid,          // KOL ID
    pub enabled: bool,
    pub max_position_sol: f64,    // Max per-trade
    pub delay_ms: u64,            // Wait before copying (avoid frontrun)
    pub min_trust_score: f32,     // Auto-disable if below
    pub copy_percentage: f32,     // 1.0 = match position, 0.5 = half size
    pub token_whitelist: Option<Vec<String>>,
    pub token_blacklist: Option<Vec<String>>,
}
```

**Flow**:
```
Helius Webhook (KOL Trade)
         │
         ▼
┌──────────────────┐
│ Parse Trade Data │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ Check Copy Config│
│ - Enabled?       │
│ - Trust > min?   │
│ - Within limits? │
└────────┬─────────┘
         │ Pass
         ▼
┌──────────────────┐
│   Apply Delay    │
│   (config.delay) │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ Execute Copy     │
│ (w/ delay)       │
└──────────────────┘
```

### 8. Threat Detection Agent
**Purpose**: Identify and block bad actors, rug pulls, honeypots, and malicious activity

**Threat Categories**:
| Category | Indicators | Response |
|----------|------------|----------|
| Rug Pull | Creator sells >50%, liquidity removal | Block token, warn swarm |
| Honeypot | Buy works, sell reverts | Blacklist contract |
| Pump & Dump | Coordinated buys followed by dump | Avoid entry, track wallets |
| Sandwich Attack | Frontrun/backrun targeting us | Flag bundle, adjust routes |
| Malicious Bundle | Jito bundle with hidden exploit | Reject, report |
| Wash Trading | Same wallets cycling volume | Discount volume signals |

**External Threat Intelligence APIs**:
| Source | Purpose | Data |
|--------|---------|------|
| RugCheck API | Contract audit | Ownership, mint authority, freeze |
| GoPlus Security | Honeypot detection | Buy/sell tax, transfer restrictions |
| Birdeye | Holder analysis | Top holders, concentration |
| Helius | Transaction patterns | Unusual activity, related wallets |

**X/Twitter Threat Feeds** (via Research Agent):
- @ZachXBT - Scam exposés
- @CryptoSlam - Rug alerts
- @RugDocIO - Contract audits
- Custom keyword monitoring: "rug", "honeypot", "scam", "$TOKEN"

**In-House Threat Metrics**:
```rust
pub struct ThreatScore {
    pub token_mint: String,
    pub overall_score: f32,        // 0 (safe) - 1 (threat)
    pub factors: ThreatFactors,
    pub confidence: f32,
    pub last_updated: DateTime<Utc>,
}

pub struct ThreatFactors {
    // Contract Analysis
    pub has_mint_authority: bool,      // Can mint infinite tokens
    pub has_freeze_authority: bool,    // Can freeze wallets
    pub has_blacklist: bool,           // Can block addresses
    pub upgradeable: bool,             // Proxy contract

    // Holder Analysis
    pub top_10_concentration: f32,     // % held by top 10
    pub creator_holdings: f32,         // % held by deployer
    pub suspicious_holder_count: u32,  // Known scam wallets

    // Trading Patterns
    pub sell_pressure_score: f32,      // Recent insider selling
    pub wash_trade_likelihood: f32,    // Fake volume detection
    pub bundle_manipulation: bool,     // Suspicious Jito bundles

    // External Signals
    pub rugcheck_score: f32,           // 0-1, external audit
    pub goplus_honeypot: bool,         // External honeypot check
    pub community_warnings: u32,       // X/Telegram mentions
}
```

**Detection Flow**:
```
New Token/Venue Detected
         │
         ▼
┌─────────────────────────┐
│ Parallel Threat Checks  │
├─────────────────────────┤
│ - RugCheck API          │
│ - GoPlus honeypot       │
│ - Holder concentration  │
│ - Creator wallet history│
│ - Contract analysis     │
└──────────┬──────────────┘
           │
           ▼
┌─────────────────────────┐
│   Calculate ThreatScore │
└──────────┬──────────────┘
           │
    ┌──────┴──────┐
    ▼             ▼
 Safe (<0.3)   Risky (>0.3)
    │             │
    ▼             ▼
 Allow         Block/Flag
 Trading       │
               ▼
         ┌─────────────┐
         │ >0.7 = Block│
         │ 0.3-0.7=Flag│
         │ + Engram    │
         └─────────────┘
```

**Real-Time Monitoring**:
- Track creator wallet activity (large sells = warning)
- Monitor liquidity pool changes (removal = rug indicator)
- Watch for coordinated bundle activity
- Alert on X/Twitter scam mentions

**MCP Tools**:
```yaml
threat_check_token:
  description: "Run full threat analysis on a token"
  params: [token_mint]

threat_check_wallet:
  description: "Analyze wallet for scam history"
  params: [wallet_address]

threat_list_blocked:
  description: "List blocked tokens/wallets"
  params: [category, limit]

threat_report:
  description: "Manually report a threat (saves to engram)"
  params: [entity_type, address, reason, evidence_url]

threat_score_history:
  description: "Get threat score history for a token"
  params: [token_mint]

threat_watch_creator:
  description: "Add creator wallet to high-alert monitoring"
  params: [wallet_address, token_mint]
```

**Engram Integration**:
- Store known scam wallets as `arb.threat.wallet.{address}`
- Store honeypot contracts as `arb.threat.honeypot.{mint}`
- Share threat intel across forked COWs

### 9. Engram Harvester Agent
**Purpose**: Extract patterns from successful trades and create shareable engrams

**Engram Types Created**:
| Type | Example Key | Content |
|------|-------------|---------|
| Edge Pattern | `arb.pattern.dex_spread` | Profitable trade signature |
| Avoidance | `arb.avoid.honeypot.{mint}` | Token to never trade |
| Strategy | `arb.strategy.curve_graduation` | Full strategy config |

### 10. Resilience Overseer (Meta-Agent)
**Purpose**: Swarm health, coordination, and recovery

**Responsibilities**:
- Monitor agent heartbeats
- Detect and recover failed agents
- Balance load across agent instances
- Coordinate graceful shutdown
- Manage circuit breakers

---

## Service Structure

```
svc/arb-farm/
├── Cargo.toml
├── .env.example
├── src/
│   ├── main.rs
│   ├── config.rs
│   ├── error.rs
│   ├── server.rs
│   │
│   ├── agents/
│   │   ├── mod.rs
│   │   ├── scanner.rs         # Venue Scanner Agent
│   │   ├── refiner.rs         # Signal Refiner Agent
│   │   ├── mev_hunter.rs      # MEV Hunter Agent
│   │   ├── executor.rs        # Execution Engine Agent
│   │   ├── strategy_engine.rs # Strategy Engine Agent
│   │   ├── research_dd.rs     # Research/DD Agent
│   │   ├── copy_trade.rs      # Copy Trade Agent
│   │   ├── threat_detector.rs # Threat Detection Agent
│   │   ├── engram_harvester.rs
│   │   └── overseer.rs        # Resilience Overseer
│   │
│   ├── events/                 # Event bus (GOLDEN RULE)
│   │   ├── mod.rs
│   │   ├── bus.rs              # Event bus implementation
│   │   ├── types.rs            # ArbEvent, EventSource
│   │   ├── topics.rs           # Topic constants
│   │   ├── publisher.rs        # Publish events
│   │   ├── subscriber.rs       # Subscribe to events
│   │   └── persistence.rs      # DB storage and replay
│   │
│   ├── threat/                 # Threat detection subsystem
│   │   ├── mod.rs
│   │   ├── score.rs           # ThreatScore calculation
│   │   ├── factors.rs         # ThreatFactors analysis
│   │   ├── monitor.rs         # Real-time threat monitoring
│   │   └── external/
│   │       ├── mod.rs
│   │       ├── rugcheck.rs    # RugCheck API client
│   │       ├── goplus.rs      # GoPlus Security API
│   │       └── birdeye.rs     # Birdeye holder analysis
│   │
│   ├── research/               # Research/DD subsystem
│   │   ├── mod.rs
│   │   ├── url_ingest.rs      # WebFetch-style URL ingestion
│   │   ├── strategy_extract.rs # LLM strategy extraction
│   │   ├── backtest.rs        # Strategy backtesting
│   │   └── social_monitor.rs  # X/Twitter monitoring
│   │
│   ├── venues/
│   │   ├── mod.rs
│   │   ├── traits.rs          # MevVenue trait
│   │   ├── dex/
│   │   │   ├── jupiter.rs
│   │   │   ├── raydium.rs
│   │   │   └── orca.rs
│   │   ├── curves/
│   │   │   ├── pump_fun.rs
│   │   │   └── moonshot.rs
│   │   └── lending/
│   │       ├── marginfi.rs
│   │       └── kamino.rs
│   │
│   ├── consensus/
│   │   ├── mod.rs
│   │   ├── engine.rs          # Multi-LLM consensus
│   │   ├── openrouter.rs      # OpenRouter client
│   │   └── voting.rs          # Weighted voting logic
│   │
│   ├── execution/
│   │   ├── mod.rs
│   │   ├── jito.rs            # Jito bundle submission
│   │   ├── simulation.rs      # Pre-execution simulation
│   │   └── atomic.rs          # Atomic execution helper
│   │
│   ├── models/
│   │   ├── mod.rs
│   │   ├── edge.rs
│   │   ├── strategy.rs
│   │   ├── trade.rs
│   │   └── signal.rs
│   │
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── health.rs
│   │   ├── scanner.rs
│   │   ├── edges.rs
│   │   ├── strategies.rs
│   │   ├── consensus.rs       # Multi-LLM consensus endpoints
│   │   ├── research.rs        # Research/DD endpoints
│   │   ├── copy_trade.rs      # Copy trading endpoints
│   │   ├── threat.rs          # Threat detection endpoints
│   │   ├── swarm.rs           # Swarm management endpoints
│   │   └── sse.rs             # Server-sent events
│   │
│   └── database/
│       ├── mod.rs
│       └── repositories/
│           ├── mod.rs
│           ├── edges.rs
│           ├── strategies.rs
│           └── trades.rs
│
├── migrations/
│   ├── 001_create_arb_tables.sql
│   ├── 002_strategies.sql
│   ├── 003_venues.sql
│   ├── 004_bonding_curves.sql
│   ├── 005_curve_tokens.sql
│   ├── 006_consensus.sql
│   ├── 007_research.sql
│   ├── 008_kol_copy_trade.sql
│   ├── 009_threat_detection.sql
│   └── 010_events.sql
│
└── tests/
    ├── integration/
    └── simulation/
```

---

## API Endpoints (via Erebus)

### Scanner
```
GET  /api/arb/scanner/status           # Scanner status
GET  /api/arb/scanner/stream           # SSE event stream
POST /api/arb/scanner/start            # Start scanning
POST /api/arb/scanner/stop             # Stop scanning
```

### Venues
```
GET  /api/arb/venues                   # List tracked venues
POST /api/arb/venues                   # Add venue
GET  /api/arb/venues/:id               # Venue details
DELETE /api/arb/venues/:id             # Remove venue
```

### Edges (Opportunities)
```
GET  /api/arb/edges                    # List detected edges
GET  /api/arb/edges/:id                # Edge details
POST /api/arb/edges/:id/approve        # Approve for execution
POST /api/arb/edges/:id/reject         # Reject with reason
POST /api/arb/edges/:id/execute        # Execute immediately
GET  /api/arb/edges/stream             # SSE for new edges
```

### Strategies
```
GET  /api/arb/strategies               # List strategies
POST /api/arb/strategies               # Create strategy
GET  /api/arb/strategies/:id           # Strategy details
PUT  /api/arb/strategies/:id           # Update strategy
DELETE /api/arb/strategies/:id         # Delete strategy
POST /api/arb/strategies/:id/toggle    # Enable/disable
GET  /api/arb/strategies/:id/stats     # Performance stats
```

### Consensus
```
POST /api/arb/consensus/request        # Request multi-LLM consensus
GET  /api/arb/consensus/:id            # Get consensus result
GET  /api/arb/consensus/history        # Past consensus decisions
```

### Research/DD
```
POST /api/arb/research/ingest          # Ingest URL for analysis
GET  /api/arb/research/discoveries     # List discovered strategies
POST /api/arb/research/discoveries/:id/approve   # Approve for testing
POST /api/arb/research/discoveries/:id/reject    # Reject
GET  /api/arb/research/sources         # List monitored sources
POST /api/arb/research/sources         # Add source to monitor
DELETE /api/arb/research/sources/:id   # Remove source
```

### KOL Tracking
```
GET  /api/arb/kol                      # List tracked KOLs
POST /api/arb/kol                      # Add KOL to track
GET  /api/arb/kol/:id                  # KOL details + stats
PUT  /api/arb/kol/:id                  # Update tracking config
DELETE /api/arb/kol/:id                # Stop tracking
GET  /api/arb/kol/:id/trades           # KOL trade history
POST /api/arb/kol/:id/copy/enable      # Enable copy trading
POST /api/arb/kol/:id/copy/disable     # Disable copy trading
```

### Threat Detection
```
GET  /api/arb/threat/check/:mint       # Full threat analysis for token
GET  /api/arb/threat/wallet/:address   # Check wallet for scam history
GET  /api/arb/threat/blocked           # List blocked tokens/wallets
POST /api/arb/threat/report            # Manually report a threat
GET  /api/arb/threat/score/:mint       # Get current threat score
GET  /api/arb/threat/score/:mint/history # Threat score history
POST /api/arb/threat/watch             # Add creator to high-alert
GET  /api/arb/threat/alerts            # Recent threat alerts
POST /api/arb/threat/whitelist         # Whitelist trusted token/wallet
```

### Swarm Management
```
GET  /api/arb/swarm/status             # Swarm health
POST /api/arb/swarm/pause              # Pause all execution
POST /api/arb/swarm/resume             # Resume execution
GET  /api/arb/swarm/agents             # List agent statuses
POST /api/arb/swarm/agents/:type/restart # Restart specific agent
```

### Trades
```
GET  /api/arb/trades                   # Trade history
GET  /api/arb/trades/:id               # Trade details
GET  /api/arb/trades/stats             # P&L statistics
```

---

## MCP Tools

```yaml
# Scanner Tools
scanner_status:
  description: "Get scanner status and statistics"
  params: []

scanner_signals:
  description: "Get recent signals with filtering"
  params: [venue_type, min_profit_bps, limit]

scanner_add_venue:
  description: "Add a venue to scan"
  params: [venue_type, address, config]

# Edge Tools
edge_list:
  description: "List detected edges (opportunities)"
  params: [status, venue_type, min_profit, limit]

edge_details:
  description: "Get full edge details"
  params: [edge_id]

edge_approve:
  description: "Approve edge for execution"
  params: [edge_id]

edge_reject:
  description: "Reject edge with reason (saved as avoidance engram)"
  params: [edge_id, reason]

# Atomicity Tools (Guaranteed Profit Detection)
edge_classify_atomicity:
  description: "Analyze if edge is atomic (guaranteed profit or revert)"
  params: [edge_id]

edge_simulate_atomic:
  description: "Simulate atomic trade, returns guaranteed profit or failure"
  params: [edge_id, gas_price_lamports]

edge_list_atomic:
  description: "List all atomic edges with guaranteed profit"
  params: [min_profit_lamports, limit]

edge_execute_atomic:
  description: "Execute atomic trade with aggressive config"
  params: [edge_id, max_gas_lamports, bundle_tip_bps]

# Strategy Tools
strategy_list:
  description: "List active trading strategies"
  params: []

strategy_create:
  description: "Create a new trading strategy"
  params: [name, venue_types, risk_params, execution_mode]

strategy_toggle:
  description: "Enable or disable a strategy"
  params: [strategy_id, enabled]

# Consensus Tools
consensus_request:
  description: "Request multi-LLM consensus on a trade"
  params: [edge_id, models, min_agreement]

consensus_result:
  description: "Get consensus result"
  params: [consensus_id]

consensus_history:
  description: "Get history of consensus decisions"
  params: [edge_id, approved_only, limit]

# Engram Tools
engram_create:
  description: "Create a new engram (pattern, avoidance, strategy, or intel)"
  params: [key, engram_type, content, confidence, tags, expires_in_hours]

engram_get:
  description: "Get an engram by its key"
  params: [key]

engram_search:
  description: "Search engrams with filtering"
  params: [engram_type, key_prefix, tag, min_confidence, limit, offset]

engram_find_patterns:
  description: "Find matching patterns for an edge type and venue"
  params: [edge_type, venue_type, route_signature, min_success_rate]

engram_check_avoidance:
  description: "Check if an entity should be avoided based on stored avoidance engrams"
  params: [entity_type, address]

engram_create_avoidance:
  description: "Create an avoidance engram for a bad actor or risky entity"
  params: [entity_type, address, reason, category, severity]

engram_create_pattern:
  description: "Create an edge pattern engram from successful trade data"
  params: [edge_type, venue_type, route_signature, success_rate, avg_profit_bps, sample_count]

engram_delete:
  description: "Delete an engram by its key"
  params: [key]

engram_stats:
  description: "Get engram harvester statistics"
  params: []

# Research/DD Tools
research_ingest_url:
  description: "Ingest and analyze a URL for trading strategies"
  params: [url, context]

research_monitor_account:
  description: "Add X/Twitter account to monitoring list"
  params: [handle, track_type]

research_backtest_strategy:
  description: "Backtest a discovered strategy"
  params: [strategy_id, period_days]

research_list_discoveries:
  description: "List discovered strategies pending review"
  params: [status, limit]

# KOL/Copy Trade Tools
kol_track:
  description: "Start tracking a KOL wallet or social handle"
  params: [wallet_address, twitter_handle, name]

kol_list:
  description: "List tracked KOLs with trust scores"
  params: []

kol_stats:
  description: "Get detailed KOL performance stats"
  params: [kol_id]

copy_enable:
  description: "Enable copy trading for a KOL"
  params: [kol_id, max_position_sol, delay_ms]

copy_disable:
  description: "Disable copy trading for a KOL"
  params: [kol_id]

copy_active:
  description: "List currently active copy positions"
  params: []

# Threat Detection Tools
threat_check_token:
  description: "Run full threat analysis on a token"
  params: [token_mint]

threat_check_wallet:
  description: "Analyze wallet for scam history"
  params: [wallet_address]

threat_list_blocked:
  description: "List blocked tokens/wallets"
  params: [category, limit]

threat_report:
  description: "Manually report a threat (saves to engram)"
  params: [entity_type, address, reason, evidence_url]

threat_score_history:
  description: "Get threat score history for a token"
  params: [token_mint]

threat_watch_creator:
  description: "Add creator wallet to high-alert monitoring"
  params: [wallet_address, token_mint]

threat_whitelist:
  description: "Whitelist a trusted token or wallet"
  params: [entity_type, address, reason]

threat_alerts:
  description: "Get recent threat alerts"
  params: [severity, limit]

# Event Tools
event_publish:
  description: "Publish an event to the event bus"
  params: [topic, payload, correlation_id]

event_subscribe:
  description: "Subscribe to events matching topic patterns"
  params: [topics, since_timestamp]

event_history:
  description: "Query historical events"
  params: [topics, since, until, limit]

event_replay:
  description: "Replay events from a point in time"
  params: [subscription_id, from_event_id]

# Execution Tools
execute_trade:
  description: "Execute an approved edge"
  params: [edge_id, max_slippage_bps]

trade_history:
  description: "Get trade execution history"
  params: [limit, offset]

trade_stats:
  description: "Get P&L statistics"
  params: [period]

# Swarm Management Tools
swarm_status:
  description: "Get full swarm status including agents and circuit breakers"
  params: []

swarm_health:
  description: "Get swarm health summary"
  params: []

swarm_agents:
  description: "List all registered agents"
  params: []

swarm_agent_status:
  description: "Get status for a specific agent"
  params: [agent_id]

swarm_pause:
  description: "Pause all swarm trading activity"
  params: []

swarm_resume:
  description: "Resume swarm trading activity"
  params: []

swarm_heartbeat:
  description: "Record agent heartbeat"
  params: [agent_id]

swarm_report_failure:
  description: "Report agent failure"
  params: [agent_id, error]

circuit_breakers_list:
  description: "List all circuit breakers and their states"
  params: []

circuit_breaker_reset:
  description: "Reset a specific circuit breaker"
  params: [name]

circuit_breakers_reset_all:
  description: "Reset all circuit breakers"
  params: []
```

---

## Database Schema

```sql
-- Events (GOLDEN RULE - all agents emit here)
CREATE TABLE arb_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type TEXT NOT NULL,
    source_type TEXT NOT NULL, -- agent, tool, external
    source_id TEXT NOT NULL,
    topic TEXT NOT NULL,
    payload JSONB NOT NULL,
    correlation_id UUID,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_events_topic ON arb_events(topic);
CREATE INDEX idx_events_source ON arb_events(source_type, source_id);
CREATE INDEX idx_events_created ON arb_events(created_at DESC);
CREATE INDEX idx_events_correlation ON arb_events(correlation_id) WHERE correlation_id IS NOT NULL;

-- Topic subscriptions for agents
CREATE TABLE arb_event_subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subscriber_id TEXT NOT NULL, -- agent or external subscriber
    topics TEXT[] NOT NULL,      -- ["arb.edge.*", "arb.trade.*"]
    is_active BOOLEAN DEFAULT true,
    last_event_id UUID,          -- For replay cursor
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Tracked venues
CREATE TABLE arb_venues (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    venue_type TEXT NOT NULL, -- dex_amm, bonding_curve, lending
    name TEXT NOT NULL,
    address TEXT, -- contract/pool address if applicable
    config JSONB DEFAULT '{}',
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Trading strategies
CREATE TABLE arb_strategies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_address VARCHAR NOT NULL,
    name TEXT NOT NULL,
    strategy_type TEXT NOT NULL, -- dex_arb, curve_arb, liquidation, etc.
    venue_types TEXT[] NOT NULL,
    execution_mode TEXT NOT NULL, -- autonomous, agent_directed, hybrid
    risk_params JSONB NOT NULL,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Detected edges (opportunities)
CREATE TABLE arb_edges (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    strategy_id UUID REFERENCES arb_strategies(id),
    edge_type TEXT NOT NULL, -- dex_arb, curve_arb, liquidation, backrun, jit
    execution_mode TEXT NOT NULL, -- autonomous, agent_directed, hybrid

    -- Atomicity classification (guaranteed profit vs capital-at-risk)
    atomicity TEXT NOT NULL DEFAULT 'non_atomic', -- fully_atomic, partially_atomic, non_atomic
    simulated_profit_guaranteed BOOLEAN DEFAULT false, -- Simulation passed
    simulation_tx_hash TEXT,                          -- Reference simulation
    max_gas_cost_lamports BIGINT,                     -- Only risk on atomic trades

    estimated_profit_lamports BIGINT,
    risk_score INTEGER, -- 1-100 (lower for atomic trades)
    route_data JSONB NOT NULL,
    status TEXT DEFAULT 'detected', -- detected, pending_approval, executing, executed, expired, failed, rejected
    rejection_reason TEXT,
    executed_at TIMESTAMPTZ,
    actual_profit_lamports BIGINT,
    actual_gas_cost_lamports BIGINT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ
);
CREATE INDEX idx_edges_status ON arb_edges(status);
CREATE INDEX idx_edges_strategy ON arb_edges(strategy_id);
CREATE INDEX idx_edges_mode ON arb_edges(execution_mode);
CREATE INDEX idx_edges_atomicity ON arb_edges(atomicity);
CREATE INDEX idx_edges_guaranteed ON arb_edges(simulated_profit_guaranteed) WHERE simulated_profit_guaranteed = true;

-- Trade history
CREATE TABLE arb_trades (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    edge_id UUID REFERENCES arb_edges(id),
    strategy_id UUID REFERENCES arb_strategies(id),
    tx_signature TEXT,
    bundle_id TEXT, -- Jito bundle ID if applicable
    entry_price NUMERIC(20, 9),
    exit_price NUMERIC(20, 9),
    profit_lamports BIGINT,
    gas_cost_lamports BIGINT,
    slippage_bps INTEGER,
    executed_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_trades_edge ON arb_trades(edge_id);
CREATE INDEX idx_trades_strategy ON arb_trades(strategy_id);
CREATE INDEX idx_trades_executed ON arb_trades(executed_at DESC);

-- Consensus requests/results
CREATE TABLE arb_consensus (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    edge_id UUID REFERENCES arb_edges(id),
    models TEXT[] NOT NULL,
    model_votes JSONB NOT NULL,
    approved BOOLEAN,
    agreement_score NUMERIC(3, 2),
    reasoning_summary TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_consensus_edge ON arb_consensus(edge_id);

-- Research discoveries
CREATE TABLE arb_research_discoveries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_url TEXT,
    source_type TEXT NOT NULL, -- url, twitter, telegram
    source_account TEXT, -- @handle if applicable
    extracted_strategy JSONB,
    backtest_result JSONB,
    status TEXT DEFAULT 'pending', -- pending, approved, rejected, testing, live
    confidence_score NUMERIC(3, 2),
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_research_status ON arb_research_discoveries(status);

-- Monitored sources (X accounts, Telegram channels, etc.)
CREATE TABLE arb_research_sources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_type TEXT NOT NULL, -- twitter, telegram, rss
    handle_or_url TEXT NOT NULL,
    track_type TEXT NOT NULL, -- alpha, threat, both
    is_active BOOLEAN DEFAULT true,
    last_checked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- KOL/Wallet tracking
CREATE TABLE arb_kol_entities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type TEXT NOT NULL, -- wallet, twitter_handle
    identifier TEXT NOT NULL UNIQUE, -- wallet address or @handle
    display_name TEXT,
    linked_wallet TEXT, -- If twitter handle, linked wallet if known
    trust_score NUMERIC(5, 2) DEFAULT 50.0,
    total_trades_tracked INTEGER DEFAULT 0,
    profitable_trades INTEGER DEFAULT 0,
    avg_profit_percent NUMERIC(8, 4),
    max_drawdown NUMERIC(8, 4),
    copy_trading_enabled BOOLEAN DEFAULT false,
    copy_config JSONB DEFAULT '{}',
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_kol_identifier ON arb_kol_entities(identifier);
CREATE INDEX idx_kol_trust ON arb_kol_entities(trust_score DESC);

-- KOL trade history
CREATE TABLE arb_kol_trades (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_id UUID REFERENCES arb_kol_entities(id),
    tx_signature TEXT NOT NULL,
    trade_type TEXT NOT NULL, -- buy, sell
    token_mint TEXT NOT NULL,
    amount_sol NUMERIC(20, 9),
    price_at_trade NUMERIC(20, 9),
    detected_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_kol_trades_entity ON arb_kol_trades(entity_id);

-- Copy trade executions
CREATE TABLE arb_copy_trades (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_id UUID REFERENCES arb_kol_entities(id),
    kol_trade_id UUID REFERENCES arb_kol_trades(id),
    our_tx_signature TEXT,
    copy_amount_sol NUMERIC(20, 9),
    delay_ms BIGINT,
    profit_loss_lamports BIGINT,
    status TEXT DEFAULT 'pending', -- pending, executed, failed
    executed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_copy_trades_entity ON arb_copy_trades(entity_id);
CREATE INDEX idx_copy_trades_status ON arb_copy_trades(status);

-- Threat scores for tokens
CREATE TABLE arb_threat_scores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token_mint TEXT NOT NULL,
    overall_score NUMERIC(3,2) NOT NULL, -- 0-1
    factors JSONB NOT NULL,              -- ThreatFactors
    confidence NUMERIC(3,2),
    external_data JSONB DEFAULT '{}',    -- RugCheck, GoPlus responses
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_threat_scores_token ON arb_threat_scores(token_mint);
CREATE INDEX idx_threat_scores_score ON arb_threat_scores(overall_score DESC);
CREATE INDEX idx_threat_scores_created ON arb_threat_scores(created_at DESC);

-- Blocked/flagged entities
CREATE TABLE arb_threat_blocked (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type TEXT NOT NULL, -- token, wallet, contract
    address TEXT UNIQUE NOT NULL,
    threat_category TEXT NOT NULL, -- rug_pull, honeypot, scam_wallet, wash_trader
    threat_score NUMERIC(3,2),
    reason TEXT,
    evidence_url TEXT,
    reported_by TEXT, -- system, user, external
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_threat_blocked_type ON arb_threat_blocked(entity_type);
CREATE INDEX idx_threat_blocked_category ON arb_threat_blocked(threat_category);

-- Whitelisted trusted entities
CREATE TABLE arb_threat_whitelist (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type TEXT NOT NULL, -- token, wallet
    address TEXT UNIQUE NOT NULL,
    reason TEXT,
    whitelisted_by TEXT, -- user, system
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- High-alert monitoring (creator wallets, etc.)
CREATE TABLE arb_threat_watched (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_address TEXT NOT NULL,
    related_token_mint TEXT,
    watch_reason TEXT,
    alert_on_sell BOOLEAN DEFAULT true,
    alert_on_transfer BOOLEAN DEFAULT true,
    alert_threshold_sol NUMERIC(20,9), -- Alert if sells > this
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_threat_watched_wallet ON arb_threat_watched(wallet_address);

-- Threat alerts history
CREATE TABLE arb_threat_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    alert_type TEXT NOT NULL, -- rug_detected, honeypot_detected, large_sell, etc.
    severity TEXT NOT NULL, -- low, medium, high, critical
    entity_type TEXT NOT NULL,
    address TEXT NOT NULL,
    details JSONB NOT NULL,
    action_taken TEXT, -- blocked, flagged, warned
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_threat_alerts_type ON arb_threat_alerts(alert_type);
CREATE INDEX idx_threat_alerts_severity ON arb_threat_alerts(severity);
CREATE INDEX idx_threat_alerts_created ON arb_threat_alerts(created_at DESC);
```

---

## Frontend UI/UX (MemCache Integration)

### Navigation Structure
```
MemCache Tab
├── Sandbox (existing)
├── Pinned (existing)
└── ArbFarm (new submenu)
    ├── Dashboard        # Overview, P&L, key metrics
    ├── Opportunities    # Live edge feed
    ├── Strategies       # Strategy management
    ├── Research         # Discovered strategies, URL ingestion
    ├── KOL Tracker      # Tracked wallets, copy trading
    ├── Threats          # Blocked tokens, alerts
    └── Settings         # Configuration, risk params
```

### Dashboard View
```
┌─────────────────────────────────────────────────────────────┐
│  ArbFarm Dashboard                              [Pause] [⚙️] │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │ Total P&L    │  │ Today's P&L  │  │ Win Rate     │      │
│  │ +12.5 SOL    │  │ +0.8 SOL     │  │ 67%          │      │
│  │ ↑ 15%        │  │ ↑ 3.2%       │  │ 134/200      │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │               P&L Chart (7 days)                     │   │
│  │  [Line chart showing daily P&L over time]            │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌──────────────────────┐  ┌─────────────────────────┐     │
│  │ Active Strategies    │  │ Recent Trades           │     │
│  │ ───────────────────  │  │ ─────────────────────── │     │
│  │ ● DEX Arb     ON  ↗ │  │ ✓ +0.05 SOL  PEPE arb  │     │
│  │ ● Curve Arb   ON  ↗ │  │ ✓ +0.12 SOL  JIT liq   │     │
│  │ ● JIT Liquidity ON ↗│  │ ✗ -0.01 SOL  gas only  │     │
│  │ ○ Copy Trade OFF    │  │ ✓ +0.08 SOL  backrun   │     │
│  └──────────────────────┘  └─────────────────────────┘     │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Swarm Status: 🟢 Healthy | Agents: 10/10 | Uptime: 99.9% │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### Opportunities View (Live Feed)
```
┌─────────────────────────────────────────────────────────────┐
│  Live Opportunities                    [Filter ▼] [Refresh] │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  🔴 CRITICAL - Atomic                                       │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ DEX Arb: BONK/SOL                    +45 bps profit │   │
│  │ Route: Jupiter → Raydium                            │   │
│  │ Est. Profit: 0.15 SOL | Gas: 0.001 SOL             │   │
│  │ Atomicity: ✅ Guaranteed | Risk: 5/100              │   │
│  │                               [Simulate] [Execute]  │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  🟡 HIGH - Agent Directed                                   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Curve Arb: New pump.fun token                       │   │
│  │ Token: $WOJAK | Progress: 85%                       │   │
│  │ Opportunity: Buy pre-graduation, sell on Raydium    │   │
│  │ Est. Profit: 0.5 SOL | Risk: 45/100                 │   │
│  │ Consensus: Awaiting (2/3 models queried)            │   │
│  │                     [View Details] [Approve] [Reject]│   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  🟢 MEDIUM - Auto Executing...                              │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ JIT Liquidity: SOL/USDC Pool                        │   │
│  │ Status: Executing bundle...                         │   │
│  │ Est. Profit: 0.03 SOL | Progress: ████████░░ 80%   │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Strategy Management View
```
┌─────────────────────────────────────────────────────────────┐
│  Strategies                                  [+ New Strategy]│
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ DEX Arbitrage                               🟢 ON   │   │
│  │ Mode: Autonomous | Venues: Jupiter, Raydium, Orca   │   │
│  │ ───────────────────────────────────────────────     │   │
│  │ Profit Threshold: 25 bps | Max Risk: 30/100         │   │
│  │ ───────────────────────────────────────────────     │   │
│  │ Performance (7d):                                   │   │
│  │   Trades: 145 | Win Rate: 78% | P&L: +2.3 SOL      │   │
│  │   Avg Profit: +35 bps | Best: +120 bps             │   │
│  │                                    [Edit] [Toggle]  │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Bonding Curve Graduation                    🟢 ON   │   │
│  │ Mode: Hybrid | Venues: pump.fun, moonshot           │   │
│  │ ───────────────────────────────────────────────     │   │
│  │ Auto-execute if: profit > 50bps AND risk < 40      │   │
│  │ Otherwise: Surface for approval                     │   │
│  │ ───────────────────────────────────────────────     │   │
│  │ Performance (7d):                                   │   │
│  │   Trades: 23 | Win Rate: 65% | P&L: +1.8 SOL       │   │
│  │                                    [Edit] [Toggle]  │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Research/DD View
```
┌─────────────────────────────────────────────────────────────┐
│  Research & Discovery                    [+ Ingest URL]     │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  URL Ingestion                                              │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ 🔗 https://twitter.com/whale/status/123...          │   │
│  │    [Paste URL]                          [Analyze]   │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  Discovered Strategies (Pending Review)                     │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ 📊 "Early pump.fun Entry" - Confidence: 72%         │   │
│  │ Source: @CryptoKOL tweet                            │   │
│  │ Pattern: Buy tokens < 20% curve, sell at 70%        │   │
│  │ Backtest: +34% over 30 days (simulated)             │   │
│  │                      [View Details] [Approve] [Reject]│  │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  Monitored Sources                                          │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ @ZachXBT          Twitter    🟢 Active   [Remove]   │   │
│  │ @CryptoSlam       Twitter    🟢 Active   [Remove]   │   │
│  │ Solana Alpha TG   Telegram   🟢 Active   [Remove]   │   │
│  │                                       [+ Add Source] │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### KOL Tracker View
```
┌─────────────────────────────────────────────────────────────┐
│  KOL & Wallet Tracking                     [+ Track Wallet] │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Copy Trading: 🟢 Enabled (2 KOLs active)                   │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ 🐋 WhaleWallet.sol                     Trust: 78%   │   │
│  │ Address: 7Vk3...8xNp                                │   │
│  │ ───────────────────────────────────────────────     │   │
│  │ Tracked Trades: 45 | Our Copies: 23 | Win: 74%      │   │
│  │ Our P&L from copying: +1.2 SOL                      │   │
│  │ ───────────────────────────────────────────────     │   │
│  │ Recent: Bought 5 SOL of $PEPE (2 min ago)           │   │
│  │ Copy Status: ✅ Copied (0.5 SOL, 500ms delay)       │   │
│  │                           [View Trades] [Configure]  │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ 📊 @TradingGuru                        Trust: 62%   │   │
│  │ Type: Twitter Handle                                │   │
│  │ ───────────────────────────────────────────────     │   │
│  │ Copy Trading: 🔴 OFF (trust below 70% threshold)    │   │
│  │ Tracked Mentions: 34 | Strategies Extracted: 8      │   │
│  │                    [Enable Copy] [View History]      │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Threat Alerts View
```
┌─────────────────────────────────────────────────────────────┐
│  Threat Detection                        [Blocked] [Alerts] │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Recent Alerts                                              │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ 🔴 CRITICAL - Rug Pull Detected         2 min ago   │   │
│  │ Token: $SCAM (9xK2...4mNp)                          │   │
│  │ Creator sold 80% of supply                          │   │
│  │ Action: AUTO-BLOCKED                                │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ 🟡 WARNING - Honeypot Suspected         15 min ago  │   │
│  │ Token: $FAKE (3jL9...2kPq)                          │   │
│  │ GoPlus: Sell function has restrictions              │   │
│  │ Action: FLAGGED - requires manual review            │   │
│  │                               [Investigate] [Block]  │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  Token Quick Check                                          │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ [Enter token mint address...]           [Check]     │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  Blocked Tokens: 23 | Blocked Wallets: 45 | Whitelisted: 12│
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Settings View
```
┌─────────────────────────────────────────────────────────────┐
│  ArbFarm Settings                                           │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Risk Configuration                                         │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Base Profile: [Balanced ▼]                          │   │
│  │                                                     │   │
│  │ Max Position Size:     [____] SOL                   │   │
│  │ Daily Loss Limit:      [____] SOL                   │   │
│  │ Min Profit Threshold:  [____] bps                   │   │
│  │ Max Slippage:          [____] bps                   │   │
│  │                                                     │   │
│  │ Dynamic Adjustments:   [✓] Enabled                  │   │
│  │   - Volatility scaling                              │   │
│  │   - Drawdown scaling                                │   │
│  │   - Research agent input                            │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  Multi-LLM Consensus                                        │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Active Models:                                      │   │
│  │   [✓] Claude 3 Opus      Weight: 1.5               │   │
│  │   [✓] GPT-4 Turbo        Weight: 1.0               │   │
│  │   [✓] Llama 3 70B        Weight: 0.8               │   │
│  │                                                     │   │
│  │ Min Agreement: [50%]  Timeout: [30s]               │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  Execution Settings                                         │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ [✓] Enable Atomic Trade Aggressive Mode            │   │
│  │ [✓] Auto-retry failed transactions                  │   │
│  │ [ ] Require approval for all trades                 │   │
│  │                                                     │   │
│  │ Jito Bundle Tip: [____] bps                        │   │
│  │ Priority Fee: [Auto ▼]                             │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│                                      [Reset] [Save Changes] │
└─────────────────────────────────────────────────────────────┘
```

### Frontend Components (React/TypeScript)
```
svc/hecate/src/components/memcache/
├── arbfarm/
│   ├── ArbFarmSubmenu.tsx      # Main submenu container
│   ├── Dashboard/
│   │   ├── DashboardView.tsx   # Main dashboard
│   │   ├── PLChart.tsx         # P&L line chart
│   │   ├── MetricCard.tsx      # Reusable metric card
│   │   ├── RecentTrades.tsx    # Recent trades list
│   │   └── SwarmStatus.tsx     # Agent health indicator
│   ├── Opportunities/
│   │   ├── OpportunitiesView.tsx
│   │   ├── EdgeCard.tsx        # Single opportunity card
│   │   ├── EdgeFilters.tsx     # Filter controls
│   │   └── ExecuteModal.tsx    # Execution confirmation
│   ├── Strategies/
│   │   ├── StrategiesView.tsx
│   │   ├── StrategyCard.tsx
│   │   ├── StrategyEditor.tsx  # Create/edit strategy
│   │   └── PerformanceChart.tsx
│   ├── Research/
│   │   ├── ResearchView.tsx
│   │   ├── URLIngest.tsx       # URL input + analysis
│   │   ├── DiscoveredStrategies.tsx
│   │   └── SourcesManager.tsx
│   ├── KOLTracker/
│   │   ├── KOLTrackerView.tsx
│   │   ├── KOLCard.tsx
│   │   ├── WalletMonitor.tsx
│   │   └── CopyTradeConfig.tsx
│   ├── Threats/
│   │   ├── ThreatsView.tsx
│   │   ├── AlertCard.tsx
│   │   ├── TokenChecker.tsx
│   │   └── BlockedList.tsx
│   ├── Settings/
│   │   ├── SettingsView.tsx
│   │   ├── RiskConfig.tsx
│   │   ├── ConsensusConfig.tsx
│   │   └── ExecutionConfig.tsx
│   └── hooks/
│       ├── useArbFarm.ts       # Main data hook
│       ├── useOpportunities.ts # SSE subscription
│       ├── useStrategies.ts
│       ├── useKOLTracker.ts
│       └── useThreats.ts
```

### Real-Time Updates (SSE)
```typescript
// Subscribe to live opportunities
const useOpportunities = () => {
  const [opportunities, setOpportunities] = useState<Edge[]>([]);

  useEffect(() => {
    const eventSource = new EventSource('/api/arb/scanner/stream');

    eventSource.onmessage = (event) => {
      const data = JSON.parse(event.data);
      setOpportunities(prev => [data, ...prev].slice(0, 50));
    };

    return () => eventSource.close();
  }, []);

  return opportunities;
};
```

### Key Frontend Features
- **Real-time SSE updates** for live opportunity feed
- **Interactive charts** (P&L over time, strategy performance)
- **One-click execution** for approved opportunities
- **Drag-and-drop** strategy prioritization
- **Quick token check** for threat analysis
- **Toast notifications** for important events (executed trades, threats)
- **Dark mode** by default (matches existing MemCache)
- **Responsive** for different screen sizes

---

## Current Capabilities vs Known Gaps

### ✅ What Actually Works Today

| Component | Status | Notes |
|-----------|--------|-------|
| **Curve Graduation Detection** | ✅ Working | pump.fun + moonshot via DexScreener API |
| **Lending Liquidations** | ✅ Working | Marginfi + Kamino real API calls |
| **MEV Hunter Agent** | ✅ Working | Scanning logic for arb/JIT/backrun/liquidation |
| **Helius Webhooks** | ✅ Working | Webhook registration + event processing |
| **Helius RPC/Sender** | ✅ Working | Full transaction submission |
| **Multi-LLM Consensus** | ✅ Working | OpenRouter integration, weighted voting |
| **Engram Integration** | ✅ Working | Storage, retrieval, pattern matching |
| **Capital Management** | ✅ Working | Per-strategy allocation, reservation |
| **Position Monitoring** | ✅ Working | Stop-loss, take-profit, trailing stops |
| **Frontend Dashboard** | ✅ Working | SSE, metrics, strategy management |

### ⚠️ Partially Implemented

| Component | Status | What's Missing |
|-----------|--------|----------------|
| **KOL Discovery** | ⚠️ Partial | Real pump.fun API + mock fallback |
| **KOL Copy Trading** | ⚠️ Partial | Helius webhook wired but no auto-copy execution |
| **Helius LaserStream** | ⚠️ Stub | WebSocket skeleton, needs tokio-tungstenite |

### ❌ Not Implemented (Stubbed)

| Component | Status | Details |
|-----------|--------|---------|
| **DEX Arbitrage Scanning** | ❌ Stubbed | `scan_for_signals()` returns `vec![]` for Jupiter/Raydium |
| **Cross-DEX Price Comparison** | ❌ Missing | No inter-venue comparison logic exists |
| **JIT Liquidity Detection** | ❌ Stubbed | Signal type defined but not generated |
| **Backrun Detection** | ❌ Stubbed | Signal type defined but not generated |
| **Threat Detection Agent** | ❌ Not Started | Directory structure only |
| **Research/DD Agent** | ❌ Not Started | Directory structure only |
| **Meme Coin Creation** | ❌ Not Started | Strategy type not implemented |
| **MCP Tool Aggregation** | ❌ Not Started | Protocols service doesn't aggregate ArbFarm tools |

### Signal Types Status

| Signal Type | Status | Used By |
|-------------|--------|---------|
| `CurveGraduation` | ✅ Active | pump_fun, moonshot |
| `Liquidation` | ✅ Active | kamino, marginfi |
| `DexArb` | ❌ Defined but unused | - |
| `JitLiquidity` | ❌ Defined but unused | - |
| `Backrun` | ❌ Defined but unused | - |
| `PriceDiscrepancy` | ❌ Unused | - |
| `VolumeSpike` | ❌ Unused | - |
| `LiquidityChange` | ❌ Unused | - |
| `NewToken` | ❌ Unused | - |
| `LargeOrder` | ❌ Unused | - |
| `PoolImbalance` | ❌ Unused | - |

---

## External API Dependencies

| API | Purpose | Rate Limits | Fallback |
|-----|---------|-------------|----------|
| Jupiter v6 | DEX aggregation | 600/min | Cached routes |
| Raydium | AMM pools | 100/min | Jupiter fallback |
| pump.fun | Token creation/trading | TBD | Queue + retry |
| Helius | Enhanced RPC + webhooks | Tier-based | Public RPC |
| Jito | Bundle submission | Unlimited | Direct RPC |
| Birdeye | Token data + holder analysis | 100/min | Cached data |
| OpenRouter | Multi-LLM consensus | Varies by model | Single model fallback |
| Anthropic | Claude direct (optional) | Tier-based | OpenRouter |
| OpenAI | GPT direct (optional) | Tier-based | OpenRouter |
| Marginfi | Lending positions | TBD | Cached state |
| Kamino | Lending positions | TBD | Cached state |
| RugCheck | Contract audit / threat intel | TBD | In-house analysis |
| GoPlus Security | Honeypot detection | TBD | Simulation fallback |
| Twitter/X API | Social monitoring / KOL tracking | Tier-based | Semantic search |

---

## Implementation Phases

### Phase 1: Service Scaffold + Core Models ✅
- [x] Create `svc/arb-farm/` directory structure
- [x] Set up Cargo.toml with dependencies
- [x] Implement config, error, server modules
- [x] Create all model files (edge, strategy, trade, signal)
- [x] Implement health endpoint
- [x] Set up database migrations
- [x] **Event bus infrastructure** (GOLDEN RULE)

**Verification**:
```bash
curl http://localhost:9007/health
# Should return: {"status": "ok", "service": "arb-farm"}
```

### Phase 2: Venue Scanner + Strategy Engine ⚠️ Partial
- [x] Implement MevVenue trait
- [x] Jupiter DEX scanner (quotes work)
- [x] Raydium AMM scanner (quotes work)
- [ ] DEX `scan_for_signals()` - **STUBBED** (returns empty vec)
- [x] Signal detection engine (curves + lending work)
- [x] Strategy matching logic
- [x] SSE streaming for edges
- [x] MCP tools: scanner_status, scanner_signals

> **Note**: DEX venue scanners can fetch quotes but `scan_for_signals()` returns empty. Curve graduation and lending liquidation scanning fully work.

**Verification**:
```bash
# Start SSE stream
curl -N http://localhost:3000/api/arb/scanner/stream

# Get scanner status
curl http://localhost:3000/api/arb/scanner/status
```

### Phase 3: Execution Engine + Risk Management ✅
- [x] Jito bundle submission
- [x] Transaction simulation
- [x] Risk parameter enforcement
- [x] Execution modes (auto/hybrid/agent)
- [x] Trade history persistence
- [x] MCP tools: edge_approve, edge_reject, execute_trade

**Verification**:
```bash
# Get edges
curl http://localhost:3000/api/arb/edges

# Simulate execution
curl -X POST http://localhost:3000/api/arb/edges/{id}/simulate

# Execute trade
curl -X POST http://localhost:3000/api/arb/edges/{id}/execute
```

### Phase 4: Bonding Curve Integration (pump.fun, moonshot) ✅
- [x] pump.fun API client
- [x] moonshot API client
- [x] Graduation progress tracking
- [x] Cross-venue arbitrage detection
- [x] Curve-specific MCP tools

**Verification**:
```bash
# Check graduation progress
curl http://localhost:9007/curves/{token}/progress

# Get curve opportunity
curl http://localhost:9007/edges?venue_type=bonding_curve
```

### Phase 5: MEV Detection (DEX Arb, Liquidations, JIT) ⚠️ Partial
- [ ] DEX arbitrage detection - **STUBBED** (scanner returns empty)
- [x] Lending liquidation scanner (Marginfi, Kamino) - **WORKING**
- [ ] JIT liquidity opportunity detection - **STUBBED**
- [ ] Backrun opportunity detection - **STUBBED**
- [x] Priority queue for time-sensitive edges
- [x] MEV Hunter agent exists (scanning logic defined, venues stubbed)

> **Note**: Liquidation detection via Marginfi/Kamino APIs works. DEX arbitrage, JIT, and backrun detection are defined in the MEV Hunter agent but underlying venue `scan_for_signals()` returns empty.

**Verification**:
```bash
# Get MEV opportunities by type (liquidations work)
curl "http://localhost:9007/edges?edge_type=liquidation"

# These return empty (stubbed):
curl "http://localhost:9007/edges?edge_type=jit"
curl "http://localhost:9007/edges?edge_type=dex_arb"
```

### Phase 6: Research/DD Agent + URL Ingestion ❌ Not Started
- [ ] URL ingestion pipeline (WebFetch-style)
- [ ] LLM strategy extraction
- [ ] Backtest engine (simulated historical)
- [ ] X/Twitter monitoring integration
- [ ] MCP tools: research_ingest_url, research_backtest_strategy

> **Note**: Research/DD agent is not implemented. The `research/` directory has stub files but no functional implementation.

**Verification**:
```bash
# Not implemented - endpoints will return 404 or stub responses
curl -X POST http://localhost:9007/research/ingest \
  -H "Content-Type: application/json" \
  -d '{"url": "https://twitter.com/trader/status/123"}'
```

### Phase 7: KOL Tracking + Copy Trading ⚠️ Partial
- [x] KOL discovery via pump.fun API - **WORKING**
- [x] Helius webhook integration for wallet monitoring - **WORKING**
- [x] Trust score calculation
- [ ] Copy trade auto-execution - **STUBBED** (webhook wired, no execution logic)
- [ ] Auto-disable on poor performance
- [x] MCP tools: kol_track, copy_enable, copy_disable (API exists)

> **Note**: KOL discovery and wallet tracking via Helius webhooks work. Copy trade auto-execution logic is not implemented - webhooks are received but trades are not automatically copied.

**Verification**:
```bash
# Track a KOL (works)
curl -X POST http://localhost:9007/kol \
  -H "Content-Type: application/json" \
  -d '{"wallet_address": "7Vk3...", "name": "WhaleWallet"}'

# Enable copy trading (config saved, but auto-copy not implemented)
curl -X POST http://localhost:9007/kol/{id}/copy/enable \
  -H "Content-Type: application/json" \
  -d '{"max_position_sol": 0.5, "min_delay_ms": 500}'
```

### Phase 8: Threat Detection + Safety ❌ Not Started
- [ ] RugCheck API integration
- [ ] GoPlus Security API integration
- [ ] In-house threat scoring system
- [ ] Holder concentration analysis
- [ ] Creator wallet monitoring (Helius webhooks)
- [ ] Honeypot simulation detection
- [ ] Blocklist/whitelist management
- [ ] Real-time threat alerts
- [ ] MCP tools: threat_check_token, threat_check_wallet, threat_report

> **Note**: Threat detection agent is not implemented. The `threat/` directory structure is defined but no functional implementation exists.

**Verification**:
```bash
# Not implemented - endpoints will return 404 or stub responses
curl http://localhost:9007/threat/check/TokenMintHere
```

### Phase 9: Engram Integration + Learning ✅
- [x] Engram harvester agent implementation
- [x] Pattern storage (successful edges)
- [x] Avoidance engram creation (failed trades, rug pulls)
- [x] Multi-LLM consensus engine
- [x] Strategy optimization via engram analysis
- [x] MCP tools: engram_create, engram_get, engram_search, engram_patterns, engram_avoidance, engram_stats
- [x] Consensus tools: consensus_request, consensus_result, consensus_history

**Verification**:
```bash
# Get engram harvester stats
curl http://localhost:9007/engram/stats

# Search for engrams
curl "http://localhost:9007/engram/search?engram_type=edge_pattern"

# Check avoidance
curl http://localhost:9007/engram/avoidance/token/TokenMintHere

# Create an avoidance engram
curl -X POST http://localhost:9007/engram/avoidance \
  -H "Content-Type: application/json" \
  -d '{"entity_type":"token","address":"BadTokenMint","reason":"Rug pull","category":"rug_pull","severity":"high"}'
```

### Phase 10: Swarm Orchestration + Resilience ✅
- [x] Resilience overseer meta-agent
- [x] Agent health monitoring (heartbeats)
- [x] Auto-recovery logic (respawn failed agents)
- [x] Circuit breakers for API failures
- [x] API failover handling (cached engrams as fallback)
- [x] Swarm management endpoints

**Key Files**:
- `src/agents/overseer.rs` - ResilienceOverseer meta-agent
- `src/resilience/circuit_breaker.rs` - Circuit breaker pattern
- `src/handlers/swarm.rs` - Swarm management HTTP handlers

**Verification**:
```bash
# Get swarm status
curl http://localhost:9007/swarm/status

# Get swarm health (quick check)
curl http://localhost:9007/swarm/health

# List all agents
curl http://localhost:9007/swarm/agents

# List circuit breakers
curl http://localhost:9007/swarm/circuit-breakers

# Pause swarm
curl -X POST http://localhost:9007/swarm/pause

# Resume swarm
curl -X POST http://localhost:9007/swarm/resume

# Reset a circuit breaker
curl -X POST http://localhost:9007/swarm/circuit-breakers/jupiter_api/reset

# Reset all circuit breakers
curl -X POST http://localhost:9007/swarm/circuit-breakers/reset-all
```

### Phase 11: Frontend Dashboard (MemCache Integration) ✅

- [x] ArbFarm submenu in MemCache tab
- [x] Live opportunity feed component
- [x] P&L dashboard with metrics
- [x] Strategy performance cards
- [x] Threat alerts panel
- [x] KOL tracker interface (placeholder)
- [x] Settings/configuration panel (placeholder)
- [x] Real-time SSE integration

**Key Files Created**:
```
svc/hecate/src/
├── types/arbfarm.ts                    # TypeScript type definitions (~550 lines)
├── common/
│   ├── services/arbfarm-service.tsx    # API service class (~600 lines)
│   └── hooks/
│       ├── useArbFarm.ts               # Main data hook (~650 lines)
│       └── index.ts
└── components/memcache/arbfarm/
    ├── ArbFarmDashboard.tsx            # Main dashboard component
    ├── arbfarm.module.scss             # SCSS styles (~700 lines)
    ├── index.tsx
    └── components/
        ├── MetricCard.tsx
        ├── SwarmStatusCard.tsx
        ├── EdgeCard.tsx
        ├── TradeHistoryCard.tsx
        ├── ThreatAlertCard.tsx
        └── index.ts
```

**Frontend Architecture**:
- **Types** (`types/arbfarm.ts`): Comprehensive TypeScript types for all ArbFarm entities
  - Edge, Trade, Scanner, Swarm, Strategy, Threat, KOL, Curve, Engram types
  - Color constants for status visualization
  - Filter interfaces for list views

- **Service** (`arbfarm-service.tsx`): API service class with methods for all endpoints
  - Edge operations: list, get, approve, reject, execute, simulate
  - Trade operations: list, get, stats, daily stats
  - Scanner operations: status, start, stop, signals
  - Swarm operations: status, pause, resume, agents, circuit breakers
  - Threat operations: check token, list blocked, report, whitelist
  - KOL operations: list, add, remove, enable/disable copy trading
  - Curve operations: list tokens, graduation candidates, quotes
  - SSE stream URL getters for real-time updates

- **Hook** (`useArbFarm.ts`): Comprehensive React hook with:
  - Auto-polling for scanner and swarm status
  - SSE connection management with auto-reconnect
  - State management for all entity types
  - Action methods for CRUD operations
  - Filter state for edge views

- **Components**:
  - `ArbFarmDashboard`: Multi-view dashboard with internal navigation
  - `MetricCard`: P&L and stats display
  - `SwarmStatusCard`: Agent health visualization
  - `EdgeCard`: Opportunity display with approve/reject/execute
  - `TradeHistoryCard`: Trade history with P&L
  - `ThreatAlertCard`: Threat alert display with severity

**Views Implemented**:
1. **Dashboard**: Overview with metrics, swarm status, top opportunities, recent trades
2. **Opportunities**: Full edge list with filtering and actions
3. **Strategies**: Strategy management with toggle and stats
4. **Threats**: Token checker, alerts, blocked list
5. **KOL Tracker**: Placeholder for wallet tracking
6. **Research**: Placeholder for URL ingestion
7. **Settings**: Placeholder for configuration

**Integration Points**:
- Added to `MemCacheSidebar` as navigation item
- Added to `MemCache` component with view state
- SSE subscriptions for: `arb.edge.*`, `arb.trade.*`, `arb.threat.*`, `arb.swarm.*`

**Verification**:
```bash
# Navigate in browser
1. Open http://localhost:5173
2. Connect wallet
3. Go to MemCache tab
4. Click "ArbFarm" in sidebar
5. Verify dashboard loads

# Verify SSE connection
- Check browser console for "Live updates active"
- SSE indicator should show green dot

# Verify API integration
- Dashboard should show metrics (or loading states)
- Opportunities list should populate when scanner is running
```

### Phase 12: Crossroads Integration + Revenue ⏳

**Status**: Next Phase - Ready to implement

The final phase integrates ArbFarm with the Crossroads marketplace to enable forkable, sellable MEV strategies as COW (Constellation of Work) NFTs.

#### Tasks

- [ ] **COW NFT Minting on Monad**
  - Define ArbFarm COW metadata schema
  - Implement COW minting contract on Monad
  - Create minting UI in MemCache
  - Connect to existing Crossroads listing flow

- [ ] **Forkable Scanner/Strategy Patterns**
  - Define exportable strategy format
  - Implement strategy serialization/deserialization
  - Create "Fork Strategy" action in UI
  - Handle engram inheritance on fork

- [ ] **Per-COW Configuration and Isolation**
  - Implement COW-scoped configuration storage
  - Add wallet-specific strategy instances
  - Ensure data isolation between forked COWs
  - API key management per-COW

- [ ] **Revenue Sharing Model**
  - Define revenue split logic (original creator vs forker)
  - Implement profit tracking per-COW
  - Create revenue distribution mechanism
  - Dashboard for earnings tracking

#### COW Metadata Schema

```typescript
interface ArbFarmCOW {
  // Standard COW fields
  id: string;
  name: string;
  description: string;
  creator_wallet: string;

  // ArbFarm-specific
  included_strategies: Strategy[];
  venue_types: VenueType[];
  risk_profile: RiskParams;

  // Fork tracking
  parent_cow_id?: string;
  fork_count: number;
  total_profit_generated: number;

  // Revenue
  creator_revenue_share_bps: number;  // e.g., 500 = 5%
  fork_revenue_share_bps: number;     // Revenue to original on forks
}
```

#### Integration Points

1. **Crossroads Listing**: ArbFarm strategies appear as listable COWs
2. **Fork Flow**: Users can fork existing ArbFarm COWs
3. **Engram Inheritance**: Forked COWs inherit avoidance engrams
4. **Revenue Split**: Profits split between original creator and forker

#### Verification

```bash
# List ArbFarm COWs in marketplace
curl http://localhost:3000/api/marketplace/listings?type=arbfarm

# Create new ArbFarm COW
curl -X POST http://localhost:3000/api/marketplace/listings \
  -H "Content-Type: application/json" \
  -d '{"type": "arbfarm", "strategies": [...], "price_sol": 0.5}'

# Fork existing COW
curl -X POST http://localhost:3000/api/marketplace/listings/{id}/fork

# Check revenue earnings
curl http://localhost:3000/api/marketplace/earnings
```

---

## Verification Summary

```bash
# Service health
curl http://localhost:9007/health

# Scanner status
curl http://localhost:9007/scanner/status

# Live edge stream
curl -N http://localhost:9007/edges/stream

# Trade statistics
curl http://localhost:9007/trades/stats

# Engram stats
curl http://localhost:9007/engram/stats

# Swarm health
curl http://localhost:9007/swarm/health

# Swarm status (full)
curl http://localhost:9007/swarm/status

# Circuit breakers
curl http://localhost:9007/swarm/circuit-breakers
```

---

## Risk Considerations

| Risk | Mitigation |
|------|------------|
| API rate limits | Caching, fallback providers, exponential backoff |
| Slippage | Pre-execution simulation, max slippage params |
| Front-running | Jito bundles, private mempool |
| Rug pulls | Threat detection agent, blocklist |
| LLM hallucination | Multi-model consensus, human-in-loop for large trades |
| Smart contract bugs | Simulation before execution, start with small positions |

---

## Audit Log

### 2026-01-19: Production Readiness Audit

**Status**: 95% Production Ready

#### Critical Fixes Implemented
1. **Exit Transactions Saved to Engrams** ✅
   - Added `save_exit_to_engrams()` in `position_executor.rs` (moved from position_monitor.rs)
   - Both buy AND sell transactions now tracked with PnL
   - Files: `position_executor.rs`, `server.rs`

2. **MCP Tool Annotations** ✅
   - All 97 tools annotated with `readOnlyHint`, `destructiveHint`, `idempotentHint`
   - External agents can determine tool safety
   - Files: `mcp/tools.rs`, `mcp/types.rs`

3. **A2A Tags Exposed** ✅
   - Learning tools tagged with `arbFarm.learning`
   - Tools: `engram_get_arbfarm_learning`, `engram_acknowledge_recommendation`, `engram_get_trade_history`, `engram_get_errors`, `engram_request_analysis`, `engram_get_by_ids`
   - Files: `mcp/tools.rs`, `mcp/types.rs`

4. **Wallet Funding Validation** ✅
   - Startup blocked if balance < 0.05 SOL
   - Clear error message with wallet address
   - File: `main.rs`

5. **Position Monitor Auto-Start** ✅ (Already existed)
   - Auto-starts with curve support + engrams
   - File: `main.rs` lines 226-232

6. **Frontend Duplicate Methods Removed** ✅
   - Removed 7 duplicate method definitions
   - File: `arbfarm-service.tsx`

#### Build Warnings (Safe to Ignore)
These are from stubbed/unimplemented features:

| Module | Warnings | Reason |
|--------|----------|--------|
| `events/topics.rs` | 66 | Event constants for future features |
| `agents/mev_hunter.rs` | 16 | DEX arbitrage (stubbed) |
| `venues/dex/raydium.rs` | 8 | Raydium integration (stubbed) |
| `consensus/providers/*` | 14 | Direct Anthropic/OpenAI (using OpenRouter) |
| `agents/graduation_tracker.rs` | 8 | Graduation tracking (stubbed) |

#### Verification Commands
```bash
# Test exit engram saves
curl http://localhost:9007/mcp/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"engram_get_trade_history","arguments":{"limit":10}}}'

# Verify annotations in tools/list
curl http://localhost:9007/mcp/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' | jq '.result.tools[0].annotations'

# Filter tools by A2A tag
curl http://localhost:9007/mcp/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' | jq '.result.tools[] | select(.tags != null)'

# Test wallet validation (will fail if < 0.05 SOL)
# Startup will show: "❌ STARTUP BLOCKED: Wallet balance..."
```

---

## Related Documentation

- [Poly Mev Plan](./poly-mev/plan.md) - Polymarket swarm (different service)
- [Engrams Service](./services/engrams.md)
- [Crossroads Marketplace](./services/crossroads.md)
- [Architecture](./architecture.md)
