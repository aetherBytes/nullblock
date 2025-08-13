# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Nullblock is a decentralized Web3 platform for deploying and monetizing agentic workflows, built on the Model Context Protocol (MCP) architecture. The platform consists of four core components in a monorepo structure:

- **Nullblock.mcp**: MCP-based tooling layer with secure wallet interactions, Flashbots MEV protection, and prompt injection defenses
- **Nullblock.orchestration**: Goal-driven engine integrating Bittensor subnets for coordinating automated workflows  
- **Nullblock.agents**: Modular agentic army delivering niche-specific services (arbitrage bots, yield optimizers, NFT traders, DAO governance tools)
- **Nullblock.platform**: dApp and marketplace for deploying, customizing, and monetizing workflows

### Current Implementation Structure

- **Helios**: Python FastAPI backend with WebSocket support (evolving toward MCP server)
- **Hecate**: React TypeScript frontend with SSR using @lomray/vite-ssr-boost (will become Nullblock.platform UI)
- **Erebus**: Rust Solana contracts for blockchain operations (foundation for on-chain agents)

The platform currently features an ECHO interface (chat-like UI) with room-based commands and plans for a browser extension called Aether. This will evolve into a comprehensive MCP-powered agentic workflow platform.

## Common Development Commands

### Helios Backend (`svc/helios/`)
```bash
# Development server
just run
uvicorn src.helios.main:app --reload --port 8000

# Testing  
just test
pytest -vv -s src/tests

# Code quality
just lint                    # Format + check + type checking
ruff format . && ruff check . --fix && mypy .

# Full check (lint + format + type + test)
just check

# Build
just build
hatch build

# Install dependencies
just install
pip install -e .

# View logs
just logs
tail -f logs/helios.log

# Server status
just status
curl http://localhost:5000/status

# Pre-commit setup and run
just pre-commit-setup        # Install pre-commit hooks
just pre-commit-run         # Run pre-commit on all files

# Dependency management
just update-reqs            # Update requirements.txt from pyproject.toml
just sync                   # Full environment sync

# Quick aliases
just t                      # Alias for test
just l                      # Alias for lint  
just c                      # Alias for check
just i                      # Alias for install
just s                      # Alias for sync
```

### Hecate Frontend (`svc/hecate/`)
```bash
# Development server
npm run develop
ssr-boost dev

# Production builds
npm run build                # Standard build
npm run build:vercel        # Vercel deployment  
npm run build:amplify       # AWS Amplify deployment

# Server modes
npm run start:ssr           # Server-side rendering
npm run start:spa           # Single page app only
npm run preview             # Preview build

# Code quality
npm run lint:check          # Check linting
npm run lint:format         # Fix linting issues
npm run style:check         # Check SCSS/CSS
npm run style:format        # Fix style issues  
npm run ts:check            # TypeScript checking
```

### Erebus Contracts (`svc/erebus/`)
```bash
# Build and run
cargo build
cargo run
cargo test

# Release build
cargo build --release
```

## Architecture Details

### MCP Architecture Vision
Nullblock implements a Model Context Protocol-first architecture for secure, agnostic agentic interactions:

**Core MCP Features**:
- Agnostic wallet interaction (MetaMask, WalletConnect, Phantom)
- Context management on IPFS/Arweave for agent workflows
- Cross-chain support via Chainlink/Wormhole
- Flashbots integration for MEV protection
- Prompt injection protection with sanitization and allowlists
- Developer API for third-party agent development

**Bittensor Integration**:
- Nullblock subnet for crowdsourcing goal-driven tasks
- Contributors earn $NULL tokens for high-impact strategies
- Yuma Consensus for fair reward distribution
- Decentralized validation of task quality and impact

### Current Frontend Architecture (Hecate → Nullblock.platform)
- **SSR Framework**: @lomray/vite-ssr-boost for server-side rendering
- **State Management**: MobX with @lomray/react-mobx-manager
- **Routing**: @lomray/react-route-manager for isomorphic routing
- **Styling**: SCSS modules + Tailwind CSS
- **Build Tool**: Vite with React plugin
- **Wallet Integration**: @solana/web3.js with Phantom wallet support
- **Future Integration**: OnchainKit for broader Web3 functionality

### Current Backend Architecture (Helios → Nullblock.mcp)  
- **Web Framework**: FastAPI with uvicorn ASGI server
- **WebSocket Support**: Built-in for real-time communication with browser extension
- **API Structure**: RESTful endpoints + WebSocket endpoints for live data
- **Blockchain**: solana-py for Solana RPC interactions
- **Logging**: Custom logging with python-json-logger
- **Code Quality**: Ruff (linting/formatting) + MyPy (type checking)
- **Evolving Toward**: MCP server implementation with secure agentic workflows

### Key Backend Endpoints (Current)
- `/api/wallet/{public_key}` - Wallet data retrieval
- `/api/wallet/health/{public_key}` - Wallet health analysis  
- `/api/memory-card/{public_key}` - Memory Card NFT data (mutable)
- `/api/swap` - Token swap execution via Raydium
- `/api/command` - Command processing for ECHO interface
- `/api/missions/{public_key}` - Active mission data
- `/ws/ember-link/{client_id}` - WebSocket for frontend clients
- `/ws/aether` - WebSocket for browser extension data
- `/status/helios` - Server status and ASCII art

### Frontend Component Structure
```
src/
├── app.tsx                    # Root app with providers
├── client.ts & server.ts      # SSR entry points
├── common/
│   ├── components/            # Shared components
│   │   ├── echo/             # Main chat interface
│   │   ├── layouts/          # Page layouts
│   │   └── modal/            # Modal components
│   └── services/
│       └── api.tsx           # API client with axios
├── pages/                    # Route-based pages
└── routes/                   # Route definitions
```

### Command System (Current ECHO Interface)
The ECHO interface uses a room-based command structure that will evolve into MCP-powered agentic workflows:

**Global Commands** (available everywhere):
- `/help`, `/status`, `/clear`, `/connect`, `/disconnect`, `/version`

**Room-Specific Commands**:
- `/logs` (default): `/trace`, `/history`, `/balance`, `/tokens`
- `/memory` (locked): `/mint`, `/upgrade`, `/features`, `/behavior`  
- `/health` (locked): `/risk`, `/audit`, `/monitor`, `/alerts`
- `/reality` (locked): `/spawn`, `/enhance`, `/interact`, `/sync`

**Future MCP Commands** (planned):
- Arbitrage workflows: `/arbitrage/start`, `/arbitrage/strategy`, `/arbitrage/monitor`
- DeFi automation: `/defi/yield`, `/defi/rebalance`, `/defi/risk`
- NFT operations: `/nft/trade`, `/nft/bid`, `/nft/analyze`
- DAO governance: `/dao/proposals`, `/dao/vote`, `/dao/delegate`

### Environment Configuration
- **Hecate**: Uses Vite env vars (`VITE_FAST_API_BACKEND_URL`)
- **Helios**: Python-dotenv for environment management
- **Development**: CORS configured for localhost:5173 (Vite dev server)

### Build & Deployment
- **Hecate**: Supports multiple deployment targets (Vercel, Amplify)
- **Docker**: Dockerfile present in Hecate service
- **TypeScript**: Strict mode enabled with path aliases configured
- **Build Output**: `svc/hecate/build/` for production builds

### WebSocket Architecture
- **Ember Link**: Real-time communication between frontend and backend (foundation for MCP connections)
- **Aether Extension**: Browser extension WebSocket integration for cross-platform agent data
- **Connection Management**: Custom EmberLinkManager class handles connections
- **Future MCP Integration**: WebSocket layer will support MCP protocol for agent-to-agent communication

### Monetization Strategy (From Gameplan)
**Target Niches**:
- **Arbitrage Trading**: 0.5-1% trade fees on automated DEX/cross-chain arbitrage
- **DeFi Yield Farming**: 0.5% asset management fees on automated portfolio rebalancing  
- **NFT Trading Automation**: 1% trading fees on automated buying/selling/bidding
- **DAO Governance Automation**: $100-$1000/month subscriptions for proposal analysis/voting

**MCP SDK Monetization**:
- Freemium model: Free basic MCP, premium features $50-$500/month
- Transaction fees: 0.1% per MCP-mediated transaction
- White-label licensing for protocols (Uniswap, Aave)

**Platform Revenue**:
- Marketplace fee: 5-10% of user-created workflow revenue
- Task fees: $0.01-$0.05 per automated task
- Premium features: $10-$100/month for advanced analytics and priority support

The platform implements a cyberpunk aesthetic with neon styling and maintains immersive error messages throughout the user experience while building toward a comprehensive MCP-powered agentic ecosystem.