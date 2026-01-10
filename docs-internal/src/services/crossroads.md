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
