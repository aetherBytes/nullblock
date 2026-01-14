# Bonding Curves Module

The Curves module handles bonding curve tokens from platforms like pump.fun and moonshot, enabling graduation tracking, cross-venue arbitrage, and holder analysis.

## Overview

Bonding curves are token launch mechanisms where price increases algorithmically as supply is bought. When enough SOL is deposited, the token "graduates" to a DEX like Raydium.

```
Token Creation → Bonding Curve Trading → Graduation → DEX Trading
                        │
                        ├── Track Progress
                        ├── Analyze Holders
                        └── Detect Arb Opportunities
```

## Supported Venues

| Venue | Description | API |
|-------|-------------|-----|
| **pump.fun** | Original bonding curve platform | pumpportal.fun |
| **moonshot** | Alternative curve platform | moonshot.cc |

## Key Concepts

### Graduation

When a bonding curve accumulates enough SOL (typically ~$69k worth), it "graduates":
1. Liquidity moves to Raydium
2. Token becomes tradeable on DEXs
3. Price discovery shifts to open market

### Virtual Reserves

Bonding curves use virtual reserves to calculate price:
- `virtual_sol_reserves` - SOL in the curve
- `virtual_token_reserves` - Tokens remaining
- Price = `sol_reserves / token_reserves`

### Graduation Progress

```
progress_percent = (current_sol / graduation_threshold) * 100
```

## API Endpoints

### List Tracked Tokens

```bash
GET /curves/tokens
```

Returns all tokens being tracked across bonding curve venues.

### Venue Health

```bash
GET /curves/health
```

Returns connectivity status for each venue API.

### Graduation Candidates

```bash
GET /curves/graduation-candidates
```

Returns tokens near graduation threshold (e.g., >70% progress).

### Cross-Venue Arbitrage

```bash
GET /curves/cross-venue-arb
```

Detects price differences between venues for the same token.

### Token-Specific Endpoints

```bash
# Graduation progress
GET /curves/:mint/progress

# Holder statistics
GET /curves/:mint/holders

# Buy/sell quote
POST /curves/:mint/quote
Content-Type: application/json
{"amount_sol": 0.5, "is_buy": true}

# Curve parameters
GET /curves/:mint/parameters
```

## Response Formats

### Graduation Progress

```json
{
  "token_mint": "ABC123...",
  "token_name": "MoonCoin",
  "token_symbol": "MOON",
  "progress_percent": 72.5,
  "bonding_curve_address": "DEF456...",
  "current_supply": 800000000,
  "virtual_sol_reserves": 30.5,
  "virtual_token_reserves": 200000000,
  "graduation_threshold_sol": 85.0,
  "estimated_graduation_at": "2024-01-15T14:00:00Z",
  "venue": "pump_fun"
}
```

### Holder Stats

```json
{
  "token_mint": "ABC123...",
  "total_holders": 234,
  "top_10_concentration": 0.45,
  "creator_holdings_percent": 0.05,
  "holders": [
    {
      "address": "7Vk3...",
      "balance": 50000000,
      "percent": 0.125,
      "is_creator": false
    }
  ]
}
```

### Curve Parameters

```json
{
  "token_mint": "ABC123...",
  "bonding_curve_address": "DEF456...",
  "fee_bps": 100,
  "graduation_threshold_sol": 85.0,
  "initial_virtual_sol": 30.0,
  "initial_virtual_tokens": 1000000000,
  "current_price_sol": 0.000015,
  "market_cap_sol": 12.5
}
```

## Trading Strategies

### Pre-Graduation Entry

Buy tokens early on the curve, sell after graduation when DEX liquidity is added.

**Entry Conditions:**
- Progress 20-40%
- Low creator holdings (<10%)
- Growing holder count
- No threat signals

**Exit Conditions:**
- Progress >90% (pre-graduation)
- Or post-graduation on DEX

### Graduation Arb

Exploit price differences between curve and DEX immediately after graduation.

**Detection:**
- Monitor tokens at >95% progress
- Watch for graduation event
- Compare curve price vs Raydium price

### Cross-Venue Arb

Same token on multiple curves (pump.fun vs moonshot).

**Detection:**
- Track same token across venues
- Alert when price diff > threshold
- Account for fees on both sides

## Holder Analysis

### Concentration Risk

High concentration indicates whale manipulation risk:

| Concentration | Risk Level |
|---------------|------------|
| Top 10 < 30% | Low |
| Top 10 30-50% | Medium |
| Top 10 > 50% | High |

### Creator Holdings

Creator selling is a rug indicator:

| Creator % | Risk Level |
|-----------|------------|
| < 5% | Low |
| 5-20% | Medium |
| > 20% | High |

## Events Emitted

| Topic | Description |
|-------|-------------|
| `arb.curve.token.detected` | New token discovered |
| `arb.curve.progress.updated` | Progress changed significantly |
| `arb.curve.graduation.imminent` | Token >90% progress |
| `arb.curve.graduated` | Token graduated to DEX |
| `arb.curve.arb.detected` | Arbitrage opportunity found |

## MCP Tools

| Tool | Description |
|------|-------------|
| `curve_tokens` | List tracked curve tokens |
| `curve_progress` | Get graduation progress |
| `curve_holders` | Get holder statistics |
| `curve_quote` | Get buy/sell quote |
| `curve_graduation_candidates` | Find near-graduation tokens |
| `curve_cross_venue_arb` | Detect cross-venue opportunities |

## Threat Integration

Bonding curves are high-risk for rugs. The threat module monitors:

- Creator wallet activity (large sells)
- Holder concentration spikes
- Social media warnings
- Contract analysis (mint authority, etc.)

See [Threat Detection](./threat.md) for details.

## Example: Finding Graduation Plays

```bash
# 1. Find candidates near graduation
curl http://localhost:9007/curves/graduation-candidates

# 2. Check holder concentration
curl http://localhost:9007/curves/ABC123.../holders

# 3. If safe, get a quote
curl -X POST http://localhost:9007/curves/ABC123.../quote \
  -H "Content-Type: application/json" \
  -d '{"amount_sol": 0.5, "is_buy": true}'
```

## Related

- [API Reference](./api.md)
- [Development Guide](./development.md)
- [Event Bus](./events.md)
