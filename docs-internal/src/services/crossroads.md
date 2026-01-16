# Crossroads Marketplace

**"App Store for AI Services"** - Web3-native marketplace for AI agents, workflows, tools, and MCP servers.

## Overview

| Property | Value |
|----------|-------|
| **Location** | `/svc/erebus/src/resources/crossroads/` |
| **Frontend** | `/svc/hecate/src/components/crossroads/` |
| **Integration** | Native Erebus subsystem |

## Features

### Discovery & Marketplace

- Browse agents, workflows, tools, MCP servers, datasets, models
- Full-text search with PostgreSQL indexing
- Advanced filtering (category, price, rating, tags)
- Featured listings and trending services
- Real-time health monitoring

### Publishing

- Multi-step wizard for service submission
- Configuration schemas and validation
- Pricing models (see below)
- Automatic discovery integration

### Web3 Integration

- OnchainKit Identity (ENS/Basename)
- Wallet-gated features
- On-chain payments (Monad)
- Service ownership verification

## Listing Categories

| Category | Description |
|----------|-------------|
| `Cow` | Constellation of Work - curated tool suites |
| `Agent` | AI agents |
| `Tool` | Individual tools |
| `McpServer` | MCP protocol servers |
| `Dataset` | Training data |
| `Model` | AI models |

## Pricing Models

| Model | Description |
|-------|-------------|
| `Free` | No cost |
| `Subscription` | Monthly/yearly |
| `OneTime` | Single purchase |
| `PayPerUse` | Per-task pricing |
| `TokenStaking` | Stake tokens for access |

## API Endpoints

### Marketplace

```bash
GET  /api/marketplace/listings      # Browse all
POST /api/marketplace/listings      # Create listing
GET  /api/marketplace/listings/:id  # Get details
POST /api/marketplace/search        # Search
GET  /api/marketplace/featured      # Featured
GET  /api/marketplace/stats         # Statistics
```

### Discovery

```bash
GET  /api/discovery/agents          # Auto-discover
GET  /api/discovery/workflows       # Discover workflows
GET  /api/discovery/tools           # Discover tools
POST /api/discovery/scan            # Trigger scan
GET  /api/discovery/health/:endpoint  # Health check
```

### Deployments

```bash
POST /api/marketplace/listings/:id/deploy  # Deploy
GET  /api/marketplace/deployments          # User deployments
POST /api/marketplace/deployments/:id/start  # Start
POST /api/marketplace/deployments/:id/stop   # Stop
```

### Reviews

```bash
GET  /api/marketplace/listings/:id/reviews  # Get reviews
POST /api/marketplace/listings/:id/reviews  # Add review
POST /api/marketplace/listings/:id/favorite # Favorite
```

### Admin

```bash
POST /api/admin/listings/approve/:id  # Approve
POST /api/admin/listings/reject/:id   # Reject
POST /api/admin/listings/feature/:id  # Feature
```

### Wallet Stash

```bash
GET /api/marketplace/wallet/:address/stash    # Get wallet's owned COWs, tools, unlock progress
GET /api/marketplace/wallet/:address/unlocks  # Get which COW tabs are unlocked
```

## Stash: Tool Inventory & Unlock System

The **Stash** tab in MemCache shows a wallet's personal inventory of tools, COWs, and access rights from Crossroads.

### What Stash Displays

- **Owned COWs** - Full COWs the wallet created or forked
- **Individual Tools** - MCP tools, Agents, strategies with active/inactive status
- **NullBlock Branded Items** - Special branding for official NullBlock tools
- **Unlock Progress** - Progress toward unlocking full COW tabs

### NullBlock Branding

A COW or tool is "NullBlock Branded" if it meets one of these conditions:

**Condition 1: Official Wallet**

Created by one of the NullBlock official wallet addresses:
- `0x7D05f6Be03D54cB2Ea05DD4b885A6f6da1Aafe8E` (EVM)
- `6xu5aRG6z7ej3hKmQkv23cENWyxzMiFA49Ww1FRzmEaU` (Solana)
- `0xcEcEe0C5f8d0d08F42727402b7081bf7Bc895D44` (EVM)
- `5wrmi85pTPmB4NDv7rUYncEMi1KqVo93bZn3XtXSbjYT` (Solana)
- `0x3be88EDa9E12ac15bBfD16Bb13eEFDd9871Bb6B7` (EVM)
- `AncqdtRrVVbGjWCCv6z2gwL8SwrWTomyUbswJeCbe4vJ` (Solana)

**Condition 2: Dedicated Service**

Has a corresponding service implementation in the NullBlock codebase:
- `svc/arb-farm/` → ArbFarm COW
- `svc/poly-mev/` → PolyMev COW (planned)

NullBlock branded items get:
- NullBlock logo indicator (⬢) in Stash
- Special UI tab in MemCache when fully owned

### COW Tab Unlock System

When a wallet owns all tools of a NullBlock-branded COW, it unlocks a dedicated submenu tab in MemCache (like the ArbFarm tab).

**Unlock Flow:**
1. User acquires tools individually from Crossroads
2. Progress tracked in Stash → Unlocks view
3. When all tools owned, tab appears in MemCache submenu

## Database Tables

- `crossroads_listings` - Service listings
- `crossroads_reviews` - User reviews
- `crossroads_deployments` - Active instances
- `crossroads_favorites` - User bookmarks
- `crossroads_discovery_scans` - Discovery tracking
- `crossroads_analytics_events` - Analytics

## Related

- [Architecture Overview](../architecture.md)
- [Echo Factory Plan](../echo-factory/plan.md)
