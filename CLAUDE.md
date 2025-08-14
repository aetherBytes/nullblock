# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Nullblock is a decentralized Web3 platform for deploying and monetizing agentic workflows, built on the Model Context Protocol (MCP) architecture. The platform consists of four core components in a monorepo structure:

## 🎯 **MVP STATUS: CORE INFRASTRUCTURE COMPLETED** ✅

### **Production-Ready Components**

- ✅ **Nullblock.mcp** (`/svc/nullblock-mcp/`): Complete MCP server with wallet authentication, IPFS context storage, Flashbots MEV protection, and ML-based security
- ✅ **Nullblock.orchestration** (`/svc/nullblock-orchestration/`): Goal-driven workflow engine with Bittensor subnet integration and template system  
- ✅ **Nullblock.agents** (`/svc/nullblock-agents/`): Full arbitrage agent suite (Price, Strategy, Execution, Reporting) with MEV protection
- 🔄 **Nullblock.platform**: dApp marketplace (pending - requires frontend development)

### **Legacy Implementation Structure** (Transitioning)

- **Helios** (`/svc/helios/`): Original FastAPI backend → **Replaced by Nullblock.mcp**
- **Hecate** (`/svc/hecate/`): React frontend with SSR → **Evolving to Nullblock.platform UI**
- **Erebus** (`/svc/erebus/`): Rust Solana contracts → **Foundation for on-chain integration**

### **Current Capabilities**

✅ **Full Arbitrage Trading Pipeline**: From price monitoring to MEV-protected execution with comprehensive reporting  
✅ **Secure Wallet Operations**: Multi-wallet support (MetaMask, WalletConnect, Phantom) with challenge-response auth  
✅ **Bittensor Task Marketplace**: Decentralized task submission with $NULL token rewards  
✅ **Advanced Security**: Prompt injection protection, encrypted context storage, anomaly detection  
✅ **Goal-Driven Automation**: Template-based workflows for arbitrage, DeFi, NFT, and DAO operations
✅ **Advanced UI/UX**: Responsive Command Lens with ball lightning NullEye animations and intelligent tooltips

## Common Development Commands

### **🆕 Social Trading Agents** (August 2025)

#### Social Trading Development (`svc/nullblock-agents/`)
```bash
# Run social trading agent
python -m agents.social_trading.main

# Debug social trading components
python -m agents.social_trading.debug --test all --token BONK

# Test specific components
python -m agents.social_trading.debug --test social --duration 60
python -m agents.social_trading.debug --test sentiment
python -m agents.social_trading.debug --test risk
python -m agents.social_trading.debug --test pipeline --token WIF

# Run comprehensive tests
pytest tests/test_social_trading.py -v
pytest svc/nullblock-mcp/tests/test_mcp_tools.py -v

# Configuration
cp config.json.example config.json
# Edit: twitter_bearer_token, dextools_api_key, monitored_tokens

# Start with custom config
python -m agents.social_trading.main --config custom_config.json --log-level DEBUG
```

#### MCP Social Tools Testing (`svc/nullblock-mcp/`)
```bash
# Test social media tools
python -c "
import asyncio
from mcp.tools.social_tools import SocialMediaTools, SocialMediaConfig
config = SocialMediaConfig()
tools = SocialMediaTools(config)
result = asyncio.run(tools.get_twitter_sentiment('$BONK'))
print(result)
"

# Test sentiment analysis
python -c "
from mcp.tools.sentiment_tools import SentimentAnalysisTools
analyzer = SentimentAnalysisTools()
signal = analyzer.analyze_text_sentiment('BONK to the moon! 🚀')
print(f'Sentiment: {signal.sentiment_score}, Confidence: {signal.confidence}')
"

# Test trading tools
python -c "
import asyncio
from mcp.tools.trading_tools import TradingTools
trader = TradingTools('https://api.mainnet-beta.solana.com')
tokens = asyncio.run(trader.get_token_list())
print(f'Loaded {len(tokens)} tokens')
"
```

### **New MCP Infrastructure** 🆕

#### Nullblock.mcp (`svc/nullblock-mcp/`)
```bash
# Install dependencies
pip install -e .

# Development server
python -m mcp.server
# OR with uvicorn
uvicorn mcp.server:create_server --host 0.0.0.0 --port 8000 --reload

# Code quality
ruff format . && ruff check . --fix && mypy .

# Testing
pytest -v src/tests/

# Environment setup
cp .env.example .env
# Edit: ETHEREUM_RPC_URL, FLASHBOTS_PRIVATE_KEY, IPFS_API
```

#### Nullblock.orchestration (`svc/nullblock-orchestration/`)
```bash
# Install dependencies  
pip install -e .

# Development
python -m orchestration.workflow.engine

# Code quality
ruff format . && ruff check . --fix && mypy .

# Testing
pytest -v src/tests/
```

#### Nullblock.agents (`svc/nullblock-agents/`)
```bash
# Install dependencies
pip install -e .

# Run arbitrage agents
python -m agents.arbitrage.price_agent
python -m agents.arbitrage.strategy_agent

# Code quality
ruff format . && ruff check . --fix && mypy .

# Testing
pytest -v src/tests/
```

### **Legacy Backend** (Helios - `svc/helios/`)
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

**🆕 Social Trading MCP Extensions** (August 2025):
- Real-time social media monitoring tools (Twitter/X, GMGN, DEXTools)
- Advanced sentiment analysis with ML-powered keyword detection
- Jupiter DEX integration for automated Solana trading
- Dynamic risk management and position sizing algorithms
- Comprehensive testing framework with debug utilities

**Bittensor Integration**:
- Nullblock subnet for crowdsourcing goal-driven tasks
- Contributors earn $NULL tokens for high-impact strategies
- Yuma Consensus for fair reward distribution
- Decentralized validation of task quality and impact

### Current Frontend Architecture (Hecate → Nullblock.platform)
- **SSR Framework**: @lomray/vite-ssr-boost for server-side rendering
- **State Management**: MobX with @lomray/react-mobx-manager
- **Routing**: @lomray/react-route-manager for isomorphic routing
- **Styling**: SCSS modules + Tailwind CSS with responsive grid systems
- **Build Tool**: Vite with React plugin
- **Wallet Integration**: @solana/web3.js with Phantom wallet support
- **Future Integration**: OnchainKit for broader Web3 functionality

### **🎨 Advanced UI/UX Features** (August 2025)
- **Command Lens Interface**: Redesigned compact grid with instant access button styling
- **NullEye Ball Lightning**: Realistic electrical effects with silver-gold lightning arcs
- **Intelligent Tooltips**: Hover-based help system replacing static descriptions
- **Responsive Design**: Optimized for small screens (13-inch MacBooks) with 4-column layouts
- **Interactive Feedback**: Clickable NullEyes with state-responsive animations and navigation
- **Universal Navigation**: All NullEye instances default to Hecate tab with enhanced visual cues

### **Production MCP Architecture** ✅ (Nullblock.mcp)
- **Web Framework**: FastAPI with uvicorn ASGI server
- **Authentication**: Challenge-response wallet verification with session management
- **Context Storage**: IPFS-based encrypted storage with local caching
- **MEV Protection**: Flashbots client with bundle simulation and submission
- **Security**: ML-based prompt injection detection with anomaly detection
- **Multi-Wallet**: MetaMask, WalletConnect, Phantom support
- **API Structure**: RESTful MCP endpoints with comprehensive security middleware

### **MCP Server Endpoints** (Production)
- `/health` - System health check with service status
- `/auth/challenge` - Create wallet authentication challenge
- `/auth/verify` - Verify signed challenge and create session
- `/context` - Get user context and preferences (authenticated)
- `/context/update` - Update user context (authenticated)
- `/trading/command` - Execute trading commands with security validation
- `/wallet/balance` - Get wallet balance (authenticated)

### **Legacy Backend Architecture** (Helios - Transitioning)  
- **Web Framework**: FastAPI with uvicorn ASGI server
- **WebSocket Support**: Built-in for real-time communication with browser extension
- **API Structure**: RESTful endpoints + WebSocket endpoints for live data
- **Blockchain**: solana-py for Solana RPC interactions
- **Logging**: Custom logging with python-json-logger
- **Code Quality**: Ruff (linting/formatting) + MyPy (type checking)
- **Status**: Legacy system being replaced by Nullblock.mcp

### **Legacy Endpoints** (Helios - For Reference)
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

**✅ MCP Commands** (Implemented):
- Arbitrage workflows: `/arbitrage` - Execute arbitrage with MEV protection and risk assessment
- Trading commands: `/swap`, `/trade` - Execute trades with security validation  
- Portfolio management: `/rebalance` - Rebalance portfolio based on user preferences
- Settings: `/set`, `/update` - Update user context and trading preferences

**🔄 Future MCP Commands** (Planned):
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

The platform implements a cyberpunk aesthetic with neon styling, ball lightning visual effects, and maintains immersive error messages throughout the user experience while building toward a comprehensive MCP-powered agentic ecosystem.

### **🎨 Visual Design System**
- **NullEye Animations**: Each NullEye features 8 randomized lightning arcs with varied sizes, orientations, and timing
- **Silver-Gold Lightning**: Consistent electrical effects using brilliant silver (#e8e8e8) with gold accents (#e6c200)
- **State-Responsive Design**: Core colors change based on agent state while maintaining consistent lightning
- **Universal Clickability**: All NullEyes navigate to agent interfaces with enhanced hover feedback
- **Compact Layouts**: Command Lens uses 4-column responsive grids optimized for smaller screens
- **Smart Information Architecture**: Hover tooltips replace static text for cleaner, more interactive interfaces