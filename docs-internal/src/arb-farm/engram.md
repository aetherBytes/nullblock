# Engram System

The Engram system provides pattern learning and memory capabilities for the ArbFarm MEV agent swarm. It enables agents to learn from successful trades, avoid known threats, and optimize strategies over time.

## Overview

Engrams are persistent, typed memory units that capture:
- **Edge Patterns** - Successful trade signatures and routes
- **Avoidance Rules** - Entities to avoid (rug pulls, honeypots, scam wallets)
- **Strategy Configurations** - Optimized strategy parameters
- **Threat Intelligence** - Security-related findings
- **Consensus Outcomes** - Multi-LLM decision history
- **Trade Results** - Historical trade performance
- **Market Conditions** - Environmental context snapshots

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Engram Harvester Agent                    │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │   Pattern   │  │  Avoidance  │  │    Consensus        │ │
│  │   Storage   │  │   Rules     │  │    Integration      │ │
│  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘ │
│         │                │                     │            │
│         └────────────────┼─────────────────────┘            │
│                          │                                  │
│                    ┌─────▼─────┐                           │
│                    │  Engram   │                           │
│                    │  Store    │                           │
│                    └─────┬─────┘                           │
│                          │                                  │
│                    ┌─────▼─────┐                           │
│                    │  Event    │                           │
│                    │   Bus     │                           │
│                    └───────────┘                           │
└─────────────────────────────────────────────────────────────┘
```

## Engram Types

### EdgePattern
Captures successful trade patterns for future matching.

```rust
pub struct EdgePatternContent {
    pub edge_type: String,        // "dex_arb", "curve_arb", etc.
    pub venue_type: String,       // "dex_amm", "bonding_curve"
    pub route_signature: String,  // "JUP->RAY->ORCA"
    pub success_rate: f64,        // 0.0-1.0
    pub avg_profit_bps: f64,      // Average profit in basis points
    pub sample_count: u32,        // Number of trades analyzed
    pub last_success_at: DateTime<Utc>,
}
```

### Avoidance
Marks entities that should be avoided.

```rust
pub struct AvoidanceContent {
    pub entity_type: String,      // "token", "wallet", "contract"
    pub address: String,
    pub reason: String,
    pub category: String,         // "rug_pull", "honeypot", "scam"
    pub severity: AvoidanceSeverity,
    pub evidence_url: Option<String>,
    pub reported_at: DateTime<Utc>,
}

pub enum AvoidanceSeverity {
    Low,      // Proceed with caution
    Medium,   // Require manual approval
    High,     // Block by default
    Critical, // Never interact
}
```

### Strategy
Stores optimized strategy configurations.

```rust
pub struct StrategyContent {
    pub strategy_type: String,
    pub venue_types: Vec<String>,
    pub risk_params: serde_json::Value,
    pub performance_metrics: PerformanceMetrics,
    pub optimized_at: DateTime<Utc>,
}
```

## API Endpoints

### Create Engram
```bash
POST /engram
Content-Type: application/json

{
  "key": "arb.pattern.dex_spread.jup_ray",
  "engram_type": "edge_pattern",
  "content": {
    "edge_type": "dex_arb",
    "venue_type": "dex_amm",
    "route_signature": "JUP->RAY",
    "success_rate": 0.78,
    "avg_profit_bps": 35.5,
    "sample_count": 145
  },
  "confidence": 0.85,
  "tags": ["dex", "jupiter", "raydium"]
}
```

### Search Engrams
```bash
GET /engram/search?engram_type=edge_pattern&min_confidence=0.7&limit=20
```

### Check Avoidance
```bash
GET /engram/avoidance/token/TokenMintAddress
```

Response:
```json
{
  "should_avoid": true,
  "reason": "Creator sold 80% of supply",
  "category": "rug_pull",
  "severity": "Critical"
}
```

### Create Avoidance Engram
```bash
POST /engram/avoidance
Content-Type: application/json

{
  "entity_type": "token",
  "address": "BadTokenMint123",
  "reason": "Honeypot - sell function reverts",
  "category": "honeypot",
  "severity": "critical"
}
```

### Find Matching Patterns
```bash
POST /engram/patterns
Content-Type: application/json

{
  "edge_type": "dex_arb",
  "venue_type": "dex_amm",
  "min_success_rate": 0.6
}
```

### Get Harvester Stats
```bash
GET /engram/stats
```

Response:
```json
{
  "total_engrams": 1542,
  "engrams_by_type": {
    "edge_pattern": 892,
    "avoidance": 423,
    "strategy": 127,
    "consensus_outcome": 100
  },
  "patterns_matched": 3456,
  "avoidances_created": 423,
  "last_harvest_at": "2024-01-15T10:30:00Z"
}
```

## MCP Tools

| Tool | Description |
|------|-------------|
| `engram_create` | Create a new engram |
| `engram_get` | Get engram by key |
| `engram_search` | Search with filters |
| `engram_find_patterns` | Find matching patterns |
| `engram_check_avoidance` | Check if entity should be avoided |
| `engram_create_avoidance` | Create avoidance rule |
| `engram_create_pattern` | Create pattern from trade data |
| `engram_delete` | Delete an engram |
| `engram_stats` | Get harvester statistics |

## Event Topics

| Topic | Description |
|-------|-------------|
| `arb.engram.created` | New engram stored |
| `arb.engram.pattern.matched` | Pattern recognized |
| `arb.engram.avoidance.created` | Avoidance rule created |
| `arb.engram.strategy.optimized` | Strategy optimization complete |

## Integration with Other Agents

### Threat Detector
When a threat is detected, an avoidance engram is automatically created:
```
Threat Detected → arb.threat.blocked → Engram Harvester → arb.engram.avoidance.created
```

### Strategy Engine
The strategy engine queries engrams for pattern matching before executing:
```rust
// Before execution
let patterns = harvester.find_matching_patterns(&PatternMatchRequest {
    edge_type: edge.edge_type.clone(),
    venue_type: edge.venue_type.clone(),
    min_success_rate: Some(0.6),
    ..Default::default()
}).await;

if patterns.is_empty() {
    // New pattern - require consensus
    request_consensus(edge).await;
} else {
    // Known pattern - check historical success
    let avg_success = patterns.iter().map(|p| p.success_rate).sum::<f64>() / patterns.len() as f64;
    if avg_success > 0.7 {
        execute_with_confidence(edge).await;
    }
}
```

### Consensus Engine
Consensus outcomes are stored as engrams for pattern analysis:
```
Consensus Completed → Store as consensus_outcome engram → Analyze which models perform best
```

## Key Naming Convention

Engram keys follow a hierarchical pattern:
```
arb.{type}.{subtype}.{identifier}

Examples:
- arb.pattern.dex_arb.jup_ray_orca
- arb.avoid.token.BadMint123
- arb.avoid.wallet.ScamWallet456
- arb.strategy.curve_graduation.v2
- arb.consensus.edge_abc123
```

## Expiration

Engrams can have optional expiration:
```rust
let engram = ArbEngram::new(key, engram_type, content, source)
    .with_expiry(chrono::Utc::now() + chrono::Duration::days(30));
```

Use cases for expiration:
- **Market conditions** - Expire after hours/days as conditions change
- **Threat intel** - Some threats may be time-limited
- **Strategy tests** - Expire experimental strategies after testing period
