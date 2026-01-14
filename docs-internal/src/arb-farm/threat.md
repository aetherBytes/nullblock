# Threat Detection System

Phase 8 of ArbFarm implements a comprehensive threat detection system to identify and block rug pulls, honeypots, scam wallets, and other malicious activity.

## Architecture

```
                  ┌─────────────────────┐
                  │   ThreatDetector    │
                  └──────────┬──────────┘
                             │
        ┌────────────────────┼────────────────────┐
        │                    │                    │
   ┌────▼────┐         ┌─────▼─────┐        ┌────▼────┐
   │ RugCheck │         │  GoPlus   │        │ Birdeye │
   │  Client  │         │  Client   │        │ Client  │
   └──────────┘         └───────────┘        └─────────┘
        │                    │                    │
        └────────────────────┴────────────────────┘
                             │
                    ┌────────▼────────┐
                    │  ThreatScore    │
                    │  Calculation    │
                    └─────────────────┘
```

## External APIs

### RugCheck API
- Contract audit and risk analysis
- Checks mint authority, freeze authority, LP status
- Top holder concentration analysis
- Insider wallet detection

### GoPlus Security API
- Honeypot detection (buy works, sell reverts)
- Blacklist function detection
- Tax analysis (buy/sell taxes)
- Creator wallet tracking
- Proxy contract detection

### Birdeye API
- Holder concentration analysis
- Wash trading detection
- Trade pattern analysis
- Volume verification

## Threat Categories

| Category | Indicators | Action |
|----------|------------|--------|
| `rug_pull` | Creator sells >50%, LP removal | Auto-block |
| `honeypot` | Sell reverts, high sell tax | Auto-block |
| `scam_wallet` | History of rugs, linked to scams | Flag for review |
| `wash_trader` | Circular trading patterns | Discount volume |
| `fake_token` | Impersonating known token | Auto-block |
| `blacklist_function` | Can blacklist addresses | High risk flag |
| `bundle_manipulation` | Suspicious Jito bundles | Alert |

## Threat Factors

The `ThreatFactors` struct captures all risk indicators:

```rust
pub struct ThreatFactors {
    // Contract Analysis
    pub has_mint_authority: bool,      // Can mint infinite tokens
    pub has_freeze_authority: bool,    // Can freeze wallets
    pub has_blacklist: bool,           // Can block addresses
    pub is_proxy: bool,                // Upgradeable proxy

    // Holder Analysis
    pub top_10_concentration: f64,     // % held by top 10
    pub creator_holdings_percent: f64, // % held by deployer

    // Trading Patterns
    pub wash_trade_likelihood: f64,    // Fake volume detection

    // External Signals
    pub rugcheck_score: Option<f64>,   // 0-1, external audit
    pub goplus_honeypot: Option<bool>, // External honeypot check
    pub goplus_is_blacklisted: Option<bool>,
}
```

## Score Calculation

Threat scores range from 0 (safe) to 1 (dangerous):

```
Score = weighted_sum(factors)

Weights:
- Honeypot detected:       +0.3
- Mint authority:          +0.15
- Freeze authority:        +0.1
- High concentration:      +0.1-0.2
- Creator >20%:            +0.1
- Wash trading:            +factor_value
- RugCheck score:          +factor_value * 0.15
```

### Risk Levels

| Score Range | Risk Level | Recommendation |
|-------------|------------|----------------|
| 0.0 - 0.3   | Low        | Safe to trade |
| 0.3 - 0.5   | Medium     | Monitor closely |
| 0.5 - 0.7   | High       | Caution advised |
| 0.7 - 1.0   | Critical   | Avoid - likely scam |

## API Endpoints

### Token Analysis

```http
GET /threat/check/:mint
```
Run full threat analysis on a token. Returns `ThreatScore` with all factors.

**Response:**
```json
{
  "success": true,
  "score": {
    "token_mint": "TokenMintAddress...",
    "overall_score": 0.65,
    "factors": {
      "has_mint_authority": true,
      "has_freeze_authority": false,
      "top_10_concentration": 0.78,
      "rugcheck_score": 0.45,
      "goplus_honeypot": false
    },
    "risk_level": "high",
    "recommendation": "CAUTION - Significant risk factors detected",
    "external_data": { ... }
  }
}
```

### Wallet Analysis

```http
GET /threat/wallet/:address
```
Check wallet for scam history and associations.

### Blocklist Management

```http
GET /threat/blocked                # List blocked entities
POST /threat/report                # Report and block a threat
DELETE /threat/blocked/:address    # Unblock an address
GET /threat/blocked/:address/status # Check if blocked
```

### Whitelist Management

```http
GET /threat/whitelist              # List whitelisted entities
POST /threat/whitelist             # Add to whitelist
DELETE /threat/whitelist/:address  # Remove from whitelist
GET /threat/whitelist/:address/status # Check if whitelisted
```

### Wallet Watching

```http
GET /threat/watch                  # List watched wallets
POST /threat/watch                 # Add wallet to watch list
```

Watch creator wallets for suspicious activity:
```json
{
  "wallet_address": "CreatorWallet...",
  "related_token_mint": "TokenMint...",
  "watch_reason": "Token creator - monitoring for dumps",
  "alert_on_sell": true,
  "alert_threshold_sol": 10.0
}
```

### Alerts

```http
GET /threat/alerts?severity=high&limit=50
```

Alert types:
- `rug_detected` - Creator sold significant holdings
- `honeypot_detected` - Sell function restricted
- `large_sell` - Major holder dumping
- `creator_dumping` - Creator reducing position
- `concentration_spike` - Unusual accumulation
- `wash_trading_detected` - Fake volume patterns

### Statistics

```http
GET /threat/stats
```

Returns:
```json
{
  "total_tokens_checked": 1234,
  "threats_detected": 89,
  "rugs_prevented": 23,
  "blocked_tokens": 45,
  "blocked_wallets": 67,
  "whitelisted_count": 12,
  "watched_wallets": 34,
  "alerts_last_24h": 15
}
```

## MCP Tools

### Analysis Tools

| Tool | Description |
|------|-------------|
| `threat_check_token` | Full threat analysis on a token |
| `threat_check_wallet` | Analyze wallet for scam history |
| `threat_score_history` | Get historical threat scores |
| `threat_stats` | Get detection statistics |

### Management Tools

| Tool | Description |
|------|-------------|
| `threat_report` | Report a threat and add to blocklist |
| `threat_whitelist` | Whitelist a trusted entity |
| `threat_watch_wallet` | Add wallet to monitoring |
| `threat_list_blocked` | List blocked entities |
| `threat_alerts` | Get recent threat alerts |
| `threat_is_blocked` | Check if address is blocked |
| `threat_is_whitelisted` | Check if address is whitelisted |

## Usage Examples

### Check Token Before Trading

```bash
# Check if a token is safe to trade
curl http://localhost:9007/threat/check/TokenMintAddress

# Response with high risk score would indicate caution
```

### Report a Scam

```bash
curl -X POST http://localhost:9007/threat/report \
  -H "Content-Type: application/json" \
  -d '{
    "entity_type": "token",
    "address": "ScamTokenMint...",
    "category": "rug_pull",
    "reason": "Creator dumped 80% of supply",
    "evidence_url": "https://solscan.io/tx/..."
  }'
```

### Watch a Creator Wallet

```bash
curl -X POST http://localhost:9007/threat/watch \
  -H "Content-Type: application/json" \
  -d '{
    "wallet_address": "CreatorWallet...",
    "related_token_mint": "TokenMint...",
    "alert_on_sell": true,
    "alert_threshold_sol": 5.0
  }'
```

## Integration with Trading

The threat detection system integrates with the strategy engine:

1. **Pre-Trade Check**: Before executing any trade, check token threat score
2. **Auto-Block**: Tokens with score >= 0.7 are automatically blocked
3. **Alert Generation**: Suspicious patterns generate real-time alerts
4. **Event Publishing**: All threat events published to event bus (`arb.threat.*`)

### Event Topics

- `arb.threat.detected` - New threat identified
- `arb.threat.blocked` - Entity added to blocklist
- `arb.threat.alert` - Alert generated
- `arb.threat.whitelist.added` - Entity whitelisted

## Caching

Threat scores are cached for 5 minutes to reduce API calls:
- Cache key: token mint address
- TTL: 5 minutes
- Force refresh available via `force_refresh=true` query param

## Files

| File | Description |
|------|-------------|
| `src/models/threat.rs` | Threat data models |
| `src/threat/mod.rs` | Main ThreatDetector |
| `src/threat/external/rugcheck.rs` | RugCheck API client |
| `src/threat/external/goplus.rs` | GoPlus API client |
| `src/threat/external/birdeye.rs` | Birdeye API client |
| `src/handlers/threat.rs` | HTTP handlers |
| `src/mcp/tools.rs` | MCP tools (get_threat_tools) |
