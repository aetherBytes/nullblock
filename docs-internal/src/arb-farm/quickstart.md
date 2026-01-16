# ArbFarm Quickstart Guide

Get ArbFarm running and ready for mainnet testing in 10 minutes.

## Overview

ArbFarm is an autonomous MEV agent swarm for Solana. It:
- Scans for arbitrage, liquidation, and copy trading opportunities
- Uses multi-LLM consensus for agent-directed decisions
- Executes via Jito bundles for MEV protection
- Manages risk with configurable limits and policies

## Prerequisites

Before starting, ensure you have:

1. **Solana Wallet** - Phantom or any Solana wallet
2. **API Keys** (store in Erebus, not .env files):
   - `HELIUS_API_KEY` - For RPC and webhooks ([helius.dev](https://helius.dev))
   - `OPENROUTER_API_KEY` - For LLM consensus ([openrouter.ai](https://openrouter.ai))
   - `BIRDEYE_API_KEY` - For token data (optional)

3. **Services Running**:
```bash
just dev-mac  # or just dev-linux
```

## Step 1: Access the Dashboard

1. Open the Hecate frontend: `http://localhost:5173`
2. Navigate to **MemCache** → **ArbFarm** in the sidebar
3. You'll see the ArbFarm dashboard

## Step 2: Configure Wallet

ArbFarm uses Turnkey for secure wallet delegation - your private keys are never stored in NullBlock.

1. Go to **Settings** view (gear icon)
2. In the **Wallet Setup** section, click **Connect Wallet**
3. Approve the Phantom popup
4. Turnkey creates a delegated wallet with policy controls

### Default Policy Limits (Dev Testing)

| Setting | Value |
|---------|-------|
| Max per transaction | 5 SOL |
| Daily volume limit | 25 SOL |
| Max concurrent positions | 10 |
| Allowed programs | Jupiter, Raydium, pump.fun, System, Token |

You can adjust these in Settings → Risk Parameters.

## Step 3: Configure API Keys

API keys are stored in the Erebus database, not in environment files.

1. In Settings → **API Keys**, verify status:
   - Helius: Required for RPC
   - OpenRouter: Required for consensus
   - Birdeye: Optional for token data

2. If missing, seed them:
```bash
cd svc/erebus && cargo run --bin seed_agent_keys
```

Or add via the API:
```bash
curl -X POST http://localhost:3000/api/agents/keys \
  -H "Content-Type: application/json" \
  -d '{"service": "helius", "api_key": "your-key-here"}'
```

## Step 4: Configure Risk Parameters

Go to Settings → **Risk Parameters**:

### Presets

| Preset | Max Position | Daily Limit | Use Case |
|--------|--------------|-------------|----------|
| Conservative | 0.5 SOL | 5 SOL | Initial testing |
| **Dev Testing** | 5 SOL | 25 SOL | Active development |
| Aggressive | 10 SOL | 100 SOL | Production |

Click a preset button to apply, or customize individual values.

### Recommended Starting Config

```
Max Position: 0.5 SOL (start small)
Daily Loss Limit: 2 SOL
Max Slippage: 100 bps (1%)
Require Simulation: ON
```

## Step 5: Enable Venues

In Settings → **Venues**, toggle which markets to scan:

| Venue | Type | Recommended |
|-------|------|-------------|
| Jupiter | DEX Aggregator | ✅ Enable |
| Raydium | AMM | ✅ Enable |
| pump.fun | Bonding Curve | ✅ Enable |
| Moonshot | Bonding Curve | ✅ Enable |
| Marginfi | Lending | Optional |
| Kamino | Lending | Optional |

## Step 6: Start the Scanner

1. Go to **Dashboard** view
2. Click **Start Scanner** in the header
3. The scanner begins detecting opportunities

### Scanner Status Indicators

| Status | Meaning |
|--------|---------|
| 🟢 Active | Scanning all venues |
| 🟡 Paused | Manually paused |
| 🔴 Error | Check logs |

## Step 7: Review Opportunities

As the scanner finds opportunities, they appear in **Opportunities** view:

### Edge Types

| Type | Description |
|------|-------------|
| Arbitrage | Cross-venue price differences |
| Liquidation | Undercollateralized positions |
| GraduationArb | Bonding curve → DEX migration |
| CopyTrade | KOL wallet mirroring |

### Edge Actions

- **Simulate** - Test without executing
- **Approve** - Queue for execution
- **Execute** - Execute immediately
- **Reject** - Skip this opportunity

## Step 8: Set Up KOL Tracking (Optional)

Track and copy-trade successful wallets:

1. Go to **KOL Tracker** view
2. Click **Add KOL**
3. Enter wallet address or Twitter handle
4. Helius webhook is automatically registered

### Enable Copy Trading

1. Select a KOL from the list
2. Configure copy parameters:
   - Max position: 0.5 SOL
   - Delay: 500ms
   - Min trust score: 60
3. Toggle **Enable Copy Trading**

## Step 9: Test Consensus (Optional)

The Research view lets you test multi-LLM consensus:

1. Go to **Research** view
2. Click **Request Test Consensus**
3. View votes from multiple LLMs (Claude, GPT-4, Llama)
4. See agreement scores and reasoning

Consensus is automatically used for agent-directed strategies.

## Monitoring

### Real-time Events

Events stream in real-time via SSE:
```bash
curl -N http://localhost:9007/events/stream
```

### Key Metrics

| Metric | Location |
|--------|----------|
| P&L | Dashboard header |
| Win Rate | Dashboard stats |
| Active Positions | Dashboard |
| Threat Alerts | Threats view |

## Troubleshooting

### Scanner Not Starting

1. Check Helius API key is configured
2. Verify RPC endpoint is responding:
```bash
curl -X POST https://mainnet.helius-rpc.com/?api-key=YOUR_KEY \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}'
```

### Consensus Failing

1. Verify OpenRouter API key
2. Check credit balance on openrouter.ai
3. Test manually:
```bash
curl -X POST http://localhost:9007/consensus/request \
  -H "Content-Type: application/json" \
  -d '{
    "edge_type": "Arbitrage",
    "venue": "jupiter",
    "token_pair": ["SOL", "USDC"],
    "estimated_profit_lamports": 50000000,
    "risk_score": 25,
    "route_data": {}
  }'
```

### Wallet Connection Failed

1. Ensure Phantom extension is installed
2. Check browser console for errors
3. Try disconnecting and reconnecting

### Execution Failing

1. Check wallet has sufficient SOL for gas
2. Verify transaction simulation passes
3. Check Jito bundle status in logs

## Mainnet Testing Progression

Follow this progression to safely ramp up:

| Phase | Max Position | Duration | Goal |
|-------|--------------|----------|------|
| 1 | 0.1 SOL | 48 hours | Verify signal accuracy |
| 2 | 0.5 SOL | 1 week | Test execution flow |
| 3 | 1 SOL | 1 week | Validate risk limits |
| 4 | 5 SOL | Ongoing | Full operation |

## Next Steps

- [API Reference](./api.md) - All endpoints documented
- [Development Guide](./development.md) - Extend the system
- [Research Module](./research.md) - Strategy discovery
- [Threat Detection](./threat.md) - Safety systems
- [KOL Tracking](./kol.md) - Copy trading details

## Quick Reference

### Ports

| Service | Port |
|---------|------|
| Hecate (Frontend) | 5173 |
| Erebus (Router) | 3000 |
| ArbFarm | 9007 |

### Key Endpoints

```bash
# Health check
curl http://localhost:9007/health

# List opportunities
curl http://localhost:9007/edges

# Scanner status
curl http://localhost:9007/scanner/status

# KOL list
curl http://localhost:9007/kol

# Consensus stats
curl http://localhost:9007/consensus/stats
```
