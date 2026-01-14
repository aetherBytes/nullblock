# Research/DD Module

The Research module enables intelligent strategy discovery through URL ingestion, LLM-powered strategy extraction, backtesting, and social media monitoring.

## Overview

```
URL/Content Input → Analysis → Strategy Extraction → Backtest → Approval
        │                              │
        ▼                              ▼
   Store as Intel              Store as Engram
```

## Components

### URL Ingester (`url_ingest.rs`)

Fetches and analyzes web content, extracting trading-relevant information.

**Content Types Detected:**
- `Tweet` - Single X/Twitter post
- `Thread` - Multi-tweet thread
- `Article` - Blog post or news article
- `Documentation` - Technical docs
- `Forum` - Reddit, Discord discussions
- `Unknown` - Unclassified content

**Extraction Features:**
- **Token mentions**: `$BONK`, `$SOL`, `$PEPE` patterns
- **Solana addresses**: Base58 addresses (32-44 chars)
- **Numbers with context**: Percentages, prices, multipliers

**Usage:**
```rust
use arb_farm::research::UrlIngester;

let ingester = UrlIngester::new(reqwest_client);
let result = ingester.ingest("https://twitter.com/trader/status/123").await?;

println!("Content type: {:?}", result.content_type);
println!("Tokens: {:?}", result.tokens_mentioned);
println!("Numbers: {:?}", result.numbers_extracted);
```

**API:**
```bash
POST /research/ingest
Content-Type: application/json

{
  "url": "https://twitter.com/trader/status/123456789",
  "context": "Potential alpha from known profitable trader"
}
```

### Strategy Extractor (`strategy_extract.rs`)

Uses LLMs (via OpenRouter) to extract actionable trading strategies from content.

**Strategy Types:**
- `DexArbitrage` - Price differences across DEXs
- `BondingCurve` - pump.fun/moonshot graduation plays
- `Momentum` - Trend following
- `MeanReversion` - Buy dips, sell rallies
- `News` - Event-driven trading
- `Liquidation` - Lending protocol liquidations
- `JitLiquidity` - Just-in-time LP provision
- `CopyTrade` - Following whale wallets
- `Custom` - User-defined strategies

**Condition Types:**
- `PriceAbove` / `PriceBelow` - Price thresholds
- `PercentChange` - Percentage movement
- `VolumeSpike` - Unusual volume
- `TimeElapsed` - Duration triggers
- `GraduationProgress` - Bonding curve progress
- `HolderCount` - Number of holders
- `WhaleActivity` - Large wallet movements
- `Custom` - Custom conditions

**Confidence Levels:**
- `High` - Clear strategy with specific parameters
- `Medium` - Strategy identifiable but parameters vague
- `Low` - Possible strategy, needs verification

**Usage:**
```rust
use arb_farm::research::StrategyExtractor;

let extractor = StrategyExtractor::new(openrouter_api_key);
let strategy = extractor.extract(&ingest_result).await?;

println!("Type: {:?}", strategy.strategy_type);
println!("Confidence: {:?}", strategy.confidence);
println!("Entry conditions: {:?}", strategy.entry_conditions);
```

### Backtest Engine (`backtest.rs`)

Simulates strategy performance against historical data.

**Configuration:**
```rust
pub struct BacktestConfig {
    pub initial_capital_sol: f64,      // Starting capital
    pub start_date: String,            // YYYY-MM-DD
    pub end_date: String,              // YYYY-MM-DD
    pub max_position_percent: f64,     // Max per-trade (0.1 = 10%)
    pub slippage_bps: u32,             // Simulated slippage
    pub fee_bps: u32,                  // Trading fees
}
```

**Metrics Calculated:**
| Metric | Description |
|--------|-------------|
| `win_rate` | Profitable trades / Total trades |
| `total_return_percent` | (Final - Initial) / Initial * 100 |
| `max_drawdown_percent` | Largest peak-to-trough decline |
| `sharpe_ratio` | Risk-adjusted returns (volatility) |
| `sortino_ratio` | Risk-adjusted returns (downside only) |
| `calmar_ratio` | Annual return / Max drawdown |

**Usage:**
```rust
use arb_farm::research::{BacktestEngine, BacktestConfig};

let engine = BacktestEngine::new();
let config = BacktestConfig {
    initial_capital_sol: 10.0,
    start_date: "2024-01-01".to_string(),
    end_date: "2024-01-31".to_string(),
    max_position_percent: 0.1,
    slippage_bps: 50,
    fee_bps: 30,
};

let result = engine.run_backtest(&strategy, &config).await?;

println!("Win rate: {:.1}%", result.summary.win_rate * 100.0);
println!("Sharpe: {:.2}", result.metrics.sharpe_ratio);
```

**API:**
```bash
POST /research/backtest
Content-Type: application/json

{
  "strategy_id": "uuid",
  "start_date": "2024-01-01",
  "end_date": "2024-01-31",
  "initial_capital_sol": 10.0
}
```

### Social Monitor (`social_monitor.rs`)

Tracks social media accounts for alpha signals and threat alerts.

**Source Types:**
- `Twitter` - X/Twitter accounts
- `Telegram` - Telegram channels
- `Discord` - Discord servers
- `Rss` - RSS feeds

**Track Types:**
- `Alpha` - Trading signals, alpha calls
- `Threat` - Scam warnings, rug alerts
- `Both` - Monitor for both

**Alert Types:**
- `TradingAlpha` - Actionable trade signal
- `NewToken` - New token launch
- `RugWarning` - Rug pull warning
- `ScamAlert` - Scam detection
- `WhaleMoved` - Large wallet activity
- `MarketCondition` - Market-wide alert

**Default Alpha Sources:**
- @solaboratory
- @DefiLlama
- @Jupiter_Aggregator

**Default Threat Sources:**
- @ZachXBT
- @RugDocIO
- @PeckShieldAlert

**Usage:**
```rust
use arb_farm::research::{SocialMonitor, SourceType, TrackType};

let mut monitor = SocialMonitor::new();

// Add a source
monitor.add_source(
    SourceType::Twitter,
    "@CryptoTrader",
    "Crypto Trader",
    TrackType::Alpha,
);

// Get recent alerts
let alerts = monitor.get_recent_alerts(20);
```

**API:**
```bash
# Add source
POST /research/sources
Content-Type: application/json

{
  "source_type": "twitter",
  "handle_or_url": "@ZachXBT",
  "track_type": "threat"
}

# List sources
GET /research/sources

# Get alerts
GET /research/alerts?limit=20
```

## Workflow

### 1. URL Ingestion Flow

```
User provides URL
       │
       ▼
┌──────────────────┐
│ Fetch Content    │
│ (reqwest + HTML) │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ Detect Type      │
│ (tweet/article)  │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ Extract Tokens   │
│ Addresses        │
│ Numbers          │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ Return Result    │
└──────────────────┘
```

### 2. Strategy Discovery Flow

```
Ingestion Result
       │
       ▼
┌──────────────────┐
│ LLM Analysis     │
│ (OpenRouter)     │
└────────┬─────────┘
         │
    ┌────┴────┐
    ▼         ▼
 Found     Not Found
    │         │
    ▼         ▼
┌────────┐  Store as
│Backtest│  Intel
│Strategy│  Engram
└───┬────┘
    │
    ▼
┌──────────────────┐
│ Calculate Metrics│
│ Sharpe, Sortino  │
│ Drawdown, etc.   │
└────────┬─────────┘
         │
    ┌────┴────┐
    ▼         ▼
Profitable  Unprofitable
    │           │
    ▼           ▼
Surface for  Archive
Approval
```

### 3. Social Monitor Flow

```
Check Sources (periodic)
       │
       ▼
┌──────────────────┐
│ Fetch New Posts  │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ Analyze Content  │
└────────┬─────────┘
         │
    ┌────┴────┐
    ▼         ▼
  Alpha    Threat
    │         │
    ▼         ▼
Create    Create
Alert     Alert
    │         │
    └────┬────┘
         │
         ▼
┌──────────────────┐
│ Emit Event       │
│ arb.research.*   │
└──────────────────┘
```

## Events Emitted

| Topic | Description |
|-------|-------------|
| `arb.research.url.ingested` | URL successfully analyzed |
| `arb.research.strategy.discovered` | New strategy extracted |
| `arb.research.strategy.approved` | Strategy approved by user |
| `arb.research.strategy.rejected` | Strategy rejected |
| `arb.research.backtest.started` | Backtest in progress |
| `arb.research.backtest.completed` | Backtest finished |
| `arb.research.alert.created` | New social alert |

## MCP Tools

| Tool | Description |
|------|-------------|
| `research_ingest_url` | Analyze a URL |
| `research_list_discoveries` | List discovered strategies |
| `research_approve_discovery` | Approve a strategy |
| `research_reject_discovery` | Reject a strategy |
| `research_backtest_strategy` | Run backtest |
| `research_list_sources` | List monitored sources |
| `research_alerts` | Get recent alerts |
| `research_stats` | Monitor statistics |

## Related

- [Development Guide](./development.md)
- [API Reference](./api.md)
- [Event Bus](./events.md)
