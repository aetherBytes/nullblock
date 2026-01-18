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

### Development Mode (Private Key)

For local development, ArbFarm uses a private key stored in the environment file:

1. Open `svc/arb-farm/.env.dev`
2. Set your dev wallet credentials:
```bash
ARB_FARM_WALLET_ADDRESS=<your-wallet-public-key>
ARB_FARM_WALLET_PRIVATE_KEY=<your-wallet-private-key>
```

3. The Settings → Wallet tab will display your configured wallet address and balance

> **Security Note**: Never commit private keys to git. The `.env.dev` file is gitignored. For production, use Turnkey wallet delegation (see below).

### Production Mode (Turnkey Delegation)

For production deployments, ArbFarm supports Turnkey for secure wallet delegation - your private keys are never stored in NullBlock:

1. Configure Turnkey credentials in `.env.dev`:
```bash
TURNKEY_API_URL=https://api.turnkey.com
TURNKEY_ORGANIZATION_ID=your-org-id
TURNKEY_API_PUBLIC_KEY=your-public-key
TURNKEY_API_PRIVATE_KEY=your-private-key
```

2. Go to **Settings** → **Wallet** tab
3. Click **Connect Wallet** and approve the Phantom popup
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

## Step 9: Inject Alpha (Research Flow)

Turn URLs into active strategies using the Research view:

1. Go to **Research** view
2. Click the **URL Injection** tab
3. Paste a URL (Twitter thread, blog post, whitepaper)
4. Click **Analyze**
5. Review the extracted strategy:
   - Strategy type, entry/exit conditions
   - Risk parameters
   - Confidence score
6. Choose an action:
   - **Create Strategy** - Add directly to active strategies
   - **Backtest First** - Validate against historical data
   - **Reject** - Discard if not viable

See [Strategy Flow](./strategy-flow.md) for the complete lifecycle.

## Step 10: Configure Execution Mode

Control how aggressively ArbFarm trades:

1. Go to **Settings** → **Execution** tab
2. Configure execution mode:

| Mode | Behavior | Risk Level |
|------|----------|------------|
| **Agent Directed** | Multi-LLM consensus for every trade | Lowest |
| **Hybrid** | Auto below threshold, consensus above | Medium |
| **Autonomous** | Full auto-execution | Highest |

3. Configure auto-execution toggle:
   - Toggle **Auto-Execute Mode** ON or OFF
   - When OFF: All trades require manual approval
   - When ON: Trades execute automatically based on strategy rules

4. If enabling auto-execute:
   - Set **Min Confidence for Auto**: 80% (recommended)
   - Set **Max Position for Auto**: 0.5 SOL (start small)
   - Keep **Require Simulation** ON

**Warning**: Autonomous mode executes trades without manual approval.

### Approval Workflow

When auto-execution is OFF, opportunities require manual approval:

1. Detected opportunities appear in **Pending Approvals** panel on Dashboard
2. If Hecate agent integration is enabled, you'll see AI recommendations
3. Review and click **Approve** or **Reject**
4. Approved trades are queued for execution
5. Approvals expire after the configured timeout (default: 5 minutes)

Check the Pending Approvals panel on the Dashboard for:
- Number of pending items
- Hecate AI recommendations with reasoning
- Expiration countdowns
- Quick approve/reject buttons

## Step 11: Monitor Signals

View real-time market signals:

1. Go to **Signals** view
2. Signals appear as they're detected:
   - `price_discrepancy` - Cross-venue price gaps
   - `volume_spike` - Unusual activity
   - `curve_graduation` - Bonding curve migrations
   - `kol_signal` - Tracked wallet activity
3. Use filters to focus on relevant signals
4. Click a signal to investigate the opportunity

## Step 12: Test Consensus (Optional)

The Research view lets you test multi-LLM consensus:

1. Go to **Research** view → **Consensus** tab
2. Click **Request Test Consensus**
3. View votes from multiple LLMs (Claude, GPT-4, Llama)
4. See agreement scores and reasoning

Consensus is automatically used for agent-directed strategies.

## Step 13: View Engrams

ArbFarm automatically saves patterns and trade results as engrams:

1. Go to **MemCache** → **Engrams** in the main sidebar
2. Filter by key prefix: `arb.`
3. View saved patterns:
   - `arb.trade.*` - Trade execution results
   - `arb.pattern.*` - Winning trade patterns
   - `arb.avoid.*` - Entities to avoid
   - `arb.state.*` - Agent state snapshots

Engrams help the agent learn from past trades and improve over time.

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

# Pending approvals
curl http://localhost:9007/approvals/pending

# Execution config (auto-execution status)
curl http://localhost:9007/execution/config

# Toggle auto-execution ON
curl -X POST http://localhost:9007/execution/toggle \
  -H "Content-Type: application/json" \
  -d '{"enabled": true}'

# Toggle auto-execution OFF
curl -X POST http://localhost:9007/execution/toggle \
  -H "Content-Type: application/json" \
  -d '{"enabled": false}'

# KOL list
curl http://localhost:9007/kol

# Consensus stats
curl http://localhost:9007/consensus/stats
```
