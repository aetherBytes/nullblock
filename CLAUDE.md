# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

NullBlock is an Agentic Platform that provides resources and tooling for building, deploying, and monetizing intelligent agent workflows. Built on the Model Context Protocol (MCP) architecture, NullBlock enables agents to interact with various systems and data sources through standardized interfaces. Web3 wallets, Gmail profiles, content creation, and other domain-specific integrations are use cases within the broader agentic ecosystem.

**Core Architecture**: Client ↔ NullBlock (Agentic Platform) ↔ Server (Web3 wallets, Gmail, APIs, etc.)

The platform consists of four core components in a monorepo structure:

## 🎯 **MVP STATUS: CORE AGENTIC INFRASTRUCTURE COMPLETED** ✅

### **Production-Ready Components**

- ✅ **NullBlock.mcp** (`/svc/nullblock-mcp/`): Complete MCP server providing standardized agent interfaces for authentication, context storage, security middleware, and multi-system integrations
- ✅ **NullBlock.orchestration** (`/svc/nullblock-orchestration/`): Goal-driven workflow engine for coordinating agent interactions with template system and task marketplace integration  
- ✅ **NullBlock.agents** (`/svc/nullblock-agents/`): Comprehensive agent suite including Hecate orchestrator, trading, social monitoring, information gathering, and LLM service coordination
- ✅ **Hecate Agent** (`/svc/nullblock-agents/src/agents/hecate/`): Primary interface agent and orchestration engine with full frontend integration
- 🔄 **NullBlock.platform**: Agentic workflow marketplace and management interface (pending - requires frontend development)

### **Legacy Implementation Structure** (Transitioning)

- **Helios** (`/svc/helios/`): Original FastAPI backend → **Replaced by NullBlock.mcp**
- **Hecate** (`/svc/hecate/`): React frontend with SSR → **Evolving to NullBlock.platform UI**
- **Erebus** (`/svc/erebus/`): Rust server for wallet interactions → **Foundation for multi-system integration**

### **Current Agentic Capabilities**

✅ **Multi-System Agent Integration**: Standardized interfaces for Web3 wallets, APIs, databases, and services through MCP protocol  
✅ **Intelligent Workflow Orchestration**: Goal-driven agent coordination with template-based automation and task marketplace  
✅ **Comprehensive Agent Suite**: Trading, social monitoring, data analysis, and LLM coordination agents with extensible architecture  
✅ **Advanced Security & Context Management**: Prompt injection protection, encrypted context storage, and session management across all integrations  
✅ **Flexible Authentication**: Multi-system auth support (Web3 wallets, OAuth, API keys) with challenge-response verification  
✅ **Responsive Agent Interface**: Command Lens with NullEye animations, intelligent tooltips, and gentle user interaction flows
✅ **Personalized Agent Management**: HecateHud for user-specific agent stats, system monitoring, and workflow configuration

### **🔥 Information Gathering Agent Pipeline** (August 2025)

🆕 **Data Intelligence Agent**: Specialized agent for analyzing prepared and modeled data from various data sources via MCP server
- **Multi-Source Data Access**: API oracles, on-chain data, DeFi protocols, social sentiment feeds
- **MCP Data Tools**: Standardized data source interfaces through Nullblock.mcp FastAPI server
- **Intelligent Analysis**: Pattern recognition, trend analysis, anomaly detection across data streams
- **Contextual Processing**: Agent adapts analysis based on user goals and historical performance
- **Real-time Updates**: Live data streaming with efficient caching and update mechanisms

### **🤖 LLM Service Factory** (August 2025)

🆕 **Unified Model Selection Service**: Intelligent LLM routing for all Nullblock agents
- **Multi-Provider Support**: OpenAI, Anthropic, Groq, HuggingFace, Ollama, Local models
- **Intelligent Routing**: Automatic model selection based on task requirements and constraints
- **Performance Tiers**: Premium (GPT-4, Claude Opus), Fast (GPT-3.5, Claude Haiku), Economical (Mistral, Local)
- **Optimization Goals**: Quality, Speed, Cost, Reliability, Balanced routing strategies
- **Capability Matching**: Route to models with specific capabilities (code, math, reasoning, multimodal)
- **Cost Optimization**: Automatic cost tracking, estimation, and budget-aware routing
- **Fallback System**: Automatic failover to alternative models on errors or rate limits
- **Response Caching**: Intelligent caching to reduce costs and improve response times
- **🆕 LM Studio Primary**: LM Studio configured as primary local model server with Gemma3 270M
- **🆕 Enhanced Model Prioritization**: LM Studio models prioritized over Ollama with higher quality/reliability scores
- **🆕 Connectivity Testing**: Automatic detection and validation of local model availability
- **🆕 Improved Error Handling**: Specific guidance for LM Studio setup and troubleshooting

### **🤖 Hecate Agent - Primary Interface & Orchestrator** (August 2025)

🆕 **Standardized Agent Logging System**: Consistent logging across all Nullblock agents
- **Colored Console Output**: Cyberpunk-themed console logs with agent-specific colors
- **File Logging**: Automatic log file generation in `logs/` directory
- **Log Levels**: DEBUG, INFO, WARNING, ERROR, CRITICAL with visual indicators
- **Request Tracking**: Start/complete logging for all agent requests
- **Model Information**: LLM usage tracking with cost estimates
- **Health Monitoring**: Agent status and performance metrics
- **Tmux Integration**: Real-time log monitoring in development environment

🆕 **Production-Ready Chat Interface**: Complete frontend integration with real-time model display
- **Frontend Integration**: Full React component integration with Hecate agent backend
- **Model Awareness**: Real-time display of current LLM model in chat interface
- **Connection Status**: Visual indicators for agent connectivity and health (Live/Offline/Connecting)
- **Idle State Theming**: Red/dimmed NullEye avatars when agent is offline with slower animations
- **User Context**: Wallet-aware conversations with session persistence
- **Orchestration Preview**: Demonstrates intent analysis and multi-agent coordination
- **Error Handling**: Graceful fallbacks when agent services are unavailable

🆕 **Orchestration Architecture**: Foundation for multi-agent coordination
- **Intent Analysis**: Keyword-based analysis evolving to LLM-powered routing
- **Agent Delegation**: Framework for coordinating specialized agents
- **Response Synthesis**: LLM-powered aggregation of multi-agent responses
- **Task Queue Management**: Foundation for complex workflow orchestration
- **Context Preservation**: Maintains conversation state across agent interactions

🆕 **Development Setup**: Complete development and integration guide
- **HTTP API Server**: FastAPI server on port 8001 for frontend communication
- **Interactive CLI**: Direct agent testing and development mode
- **Frontend Chat**: React component with real-time model display
- **CSS Styling**: Cyberpunk-themed UI with connection status indicators
- **Visual Feedback**: NullEye avatars change to red "idle" theme when agent is offline
- **🆕 Standardized Logging**: Colored console output with file logging for all agents
- **🆕 Tmux Integration**: Dedicated agents tab with real-time log monitoring
- **Documentation**: Comprehensive setup and integration instructions

### **🎬 Demo & Testing Infrastructure**

**Integration Demo**: Complete end-to-end demonstration (`/demo_integration.py`)
- Multi-agent coordination showcase
- Market intelligence analysis pipeline
- Automated research report generation
- Multi-model comparison and optimization
- System monitoring and statistics
- **🆕 FAIL-FAST ERROR HANDLING**: Comprehensive service health checks with immediate failure on critical service unavailability
- **🆕 STANDARDIZED ERROR RESPONSES**: All services return consistent error formats for easy debugging and monitoring
- **🆕 ENHANCED PRICE DISPLAY**: Improved cryptocurrency price extraction and display from API responses
- **🆕 LM Studio Integration**: Demo now prioritizes LM Studio over Ollama with proper error handling
- **🆕 Better User Guidance**: Clear prerequisites and setup instructions for LM Studio configuration

**Information Gathering Agent Demo**: Focused agent testing (`/svc/nullblock-agents/src/agents/information_gathering/demo.py`)
- Price oracle analysis workflows
- DeFi opportunity detection
- Real-time data monitoring
- Custom analysis request handling
- Agent health and status monitoring

**🆕 Rust Integration Test**: Comprehensive system integration testing (`/svc/erebus/tests/integration_tests.rs`)
- Complete end-to-end pipeline testing with mock services
- Wallet authentication and session management testing
- Market data and DeFi protocol integration verification
- Analysis request/response workflow testing
- LLM generation and response handling
- Error handling and edge case validation
- Performance benchmarking and load testing
- Concurrent request handling simulation

### **🔗 Quick Reference - New Infrastructure**

**Information Gathering Agent API**:
```python
# Basic usage
agent = InformationGatheringAgent("http://localhost:8000")
await agent.mcp_client.connect()

# Market analysis
result = await agent.analyze_market_trends(["bitcoin", "ethereum"], "24h")

# DeFi opportunities
opportunities = await agent.detect_defi_opportunities(["uniswap"])

# Real-time data
data = await agent.get_real_time_data("price_oracle", "coingecko", {"symbols": ["bitcoin"]})
```

**LLM Service Factory API**:
```python
# Initialize factory
factory = LLMServiceFactory()
await factory.initialize()

# Quick generation
response = await factory.quick_generate("Explain blockchain", "explanation", "speed")

# Advanced generation with requirements
request = LLMRequest(prompt="Analyze market data", max_tokens=500)
requirements = TaskRequirements(
    required_capabilities=[ModelCapability.DATA_ANALYSIS],
    optimization_goal=OptimizationGoal.QUALITY
)
response = await factory.generate(request, requirements)
```

**MCP Data Source Endpoints**:
- `GET /mcp/data-sources` - List available sources
- `GET /mcp/data/{type}/{source}?symbols=bitcoin,ethereum` - Get data
- `POST /mcp/data/{type}/{source}` - Complex queries
- `GET /mcp/data-sources/status` - System health

## ⚠️ **ORGANIZATIONAL RULES** ⚠️

**RESERVED DIRECTORIES** - Do not modify without explicit request:
- `svc/erebus/src/resources/templates/` - Reserved for MCP-specific template definitions
- `svc/erebus/src/resources/definitions/` - Reserved for MCP-specific type definitions and schemas

**EREBUS ARCHITECTURE RULES**:
- **main.rs** only contains subsystem entry points and core system routes
- **Subsystem Organization**: Each major feature gets its own directory (wallets/, mcp/)
- **Wallet Subsystem**: All wallet code in `resources/wallets/`
  - Each wallet type: own module (metamask.rs, phantom.rs)  
  - Consolidation layer: `wallet_interaction.rs`
  - HTTP routes: `wallets/routes.rs`
  - All wallets implement `WalletProvider` trait
- **MCP Subsystem**: All MCP code in `resources/mcp/`
  - Protocol handler: `handler.rs`
  - HTTP routes: `mcp/routes.rs` 
  - MCP types: `mcp/types.rs`
- **Shared types**: `resources/types.rs` for cross-subsystem types

## 🔥 **EREBUS WALLET ARCHITECTURE** (August 2025)

Erebus now serves as the main server for wallet interactions and will be the foundation for MCP integration:

### **Directory Structure**
```
svc/erebus/src/
├── main.rs                           # 🎯 Main entry point (subsystem routing only)
├── resources/
│   ├── mod.rs                        # Module organization
│   ├── types.rs                      # Shared types and traits
│   ├── wallets/                      # 👛 WALLET SUBSYSTEM
│   │   ├── mod.rs                    # Wallet module exports
│   │   ├── wallet_interaction.rs     # Generic wallet consolidation layer
│   │   ├── routes.rs                 # HTTP endpoints for wallets
│   │   ├── metamask.rs              # MetaMask-specific logic
│   │   └── phantom.rs               # Phantom-specific logic
│   ├── mcp/                          # 🔗 MCP SUBSYSTEM
│   │   ├── mod.rs                    # MCP module exports
│   │   ├── handler.rs                # MCP protocol handler
│   │   ├── routes.rs                 # HTTP endpoints for MCP
│   │   └── types.rs                  # MCP-specific types
│   ├── templates/                    # 🔒 RESERVED - MCP templates
│   └── definitions/                  # 🔒 RESERVED - MCP definitions
```

### **Erebus API Endpoints** (Port 3000)
**Core System:**
- `GET /health` - Server health check with subsystem status

**👛 Wallet Subsystem:**
- `GET /api/wallets` - List supported wallets
- `POST /api/wallets/challenge` - Create wallet authentication challenge
- `POST /api/wallets/verify` - Verify wallet signature and create session
- `GET /api/wallets/{type}/networks` - Get supported networks for wallet type
- `POST /api/wallets/sessions/validate` - Validate session token

**🔗 MCP Subsystem:**
- `POST /mcp` - Main MCP protocol endpoint
- `POST /mcp/initialize` - Initialize MCP server capabilities
- `POST /mcp/resources` - List available MCP resources
- `POST /mcp/tools` - List available MCP tools
- `POST /mcp/prompts` - List available MCP prompts

### **Wallet Provider Architecture**
All wallet implementations must conform to the `WalletProvider` trait:
```rust
pub trait WalletProvider {
    fn get_wallet_info() -> WalletInfo;
    fn create_challenge_message(wallet_address: &str, challenge_id: &str) -> String;
    fn verify_signature(message: &str, signature: &str, wallet_address: &str) -> Result<bool, String>;
}
```

### **Supported Wallets**
- **Phantom**: Solana wallet with Ed25519 signature verification
- **MetaMask**: Ethereum wallet with ECDSA signature verification  
- **Extensible**: New wallets can be added by implementing `WalletProvider`

### **Session Management**
- 24-hour session tokens with automatic expiration
- In-memory storage (production should use Redis)
- Session validation for authenticated endpoints
- Automatic cleanup of expired sessions

## Common Development Commands

### **🤖 LLM Development Environment** (August 2025)

#### LM Studio with Gemma3 270M (`llm` tab in tmux)
```bash
# Development environment includes dedicated LLM tab
./scripts/dev-tmux  # Starts complete environment with LLM tab

# LLM tab automatically:
# - Loads Gemma3 270M model (gemma-3-270m-it-mlx)  
# - Starts LM Studio API server on localhost:1234
# - Streams logs for real-time monitoring
# - Tests API connectivity with verification

# Manual LM Studio commands
lms status                    # Check LM Studio status
lms load gemma-3-270m-it-mlx -y  # Load Gemma3 with GPU acceleration
lms server start             # Start API server
lms server stop              # Stop API server
lms ps                       # List loaded models
lms ls                       # List downloaded models
lms log stream               # Stream live logs

# API testing
curl http://localhost:1234/v1/models  # List available models
curl -X POST http://localhost:1234/v1/completions \
  -H "Content-Type: application/json" \
  -d '{"prompt":"Hello","max_tokens":20}'  # Test completion

# Integration with Nullblock agents
# Gemma3 is available as local model option in LLM Service Factory
# Use for cost-effective local inference in development

# 🆕 NEW: LM Studio is now the primary local model server
# The system will automatically prioritize LM Studio over Ollama
# Enhanced error handling provides clear setup guidance
```

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

#### Information Gathering Agent Development (`svc/nullblock-agents/`)
```bash
# Run Information Gathering Agent demo
python -m agents.information_gathering.demo

# Run specific demo scenarios
python -c "
import asyncio
from agents.information_gathering.demo import demo_price_analysis
asyncio.run(demo_price_analysis())
"

# Test agent with custom parameters
python -c "
import asyncio
from agents.information_gathering.main import InformationGatheringAgent
async def test():
    agent = InformationGatheringAgent('http://localhost:8000')
    await agent.mcp_client.connect()
    result = await agent.analyze_market_trends(['bitcoin'], '24h')
    print(f'Analysis: {result.insights}')
    await agent.mcp_client.disconnect()
asyncio.run(test())
"

# LLM Service Factory testing
python -c "
import asyncio
from agents.llm_service.factory import LLMServiceFactory, LLMRequest
from agents.llm_service.router import TaskRequirements, OptimizationGoal
async def test():
    factory = LLMServiceFactory()
    await factory.initialize()
    response = await factory.quick_generate('Explain DeFi in one sentence', 'explanation', 'speed')
    print(f'Response: {response}')
    await factory.cleanup()
asyncio.run(test())
"
```

#### Complete Integration Demo
```bash
# Run complete integration demo (requires MCP server running)
python demo_integration.py

# Prerequisites setup
cd svc/nullblock-mcp && python -m mcp.server &  # Start MCP server
export OPENAI_API_KEY="your-key-here"          # Optional for LLM testing
export ANTHROPIC_API_KEY="your-key-here"       # Optional for LLM testing
cd ../.. && python demo_integration.py         # Run integration demo

# 🆕 NEW: Fail-fast demo with comprehensive error handling
# Demo will now fail immediately if:
# - MCP server is not accessible
# - No LLM models are available
# - Network connectivity issues
# - Invalid authentication tokens
# - Data source failures

# 🆕 NEW: LM Studio Integration
# Demo now prioritizes LM Studio over Ollama
# Enhanced price display shows actual cryptocurrency prices
# Clear setup instructions for LM Studio configuration
# Better error messages and user guidance
```

# 🆕 NEW: Rust integration testing
cd svc/erebus/
cargo test --test integration_tests test_full_integration_pipeline -- --nocapture
cargo test --test integration_tests test_performance_benchmarks -- --nocapture
cargo test --test integration_tests test_load_simulation -- --nocapture

# Run all tests
cargo test

# Run benchmarks
cargo bench

# Run specific test categories
cargo test --test unit_tests
cargo test --test integration_tests
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

# 🆕 Run Hecate Agent (Primary Interface & Orchestrator)
python -m agents.hecate.server  # HTTP API server for frontend integration (Port 8001)
python -m agents.hecate.main    # Interactive CLI mode

# 🤖 Complete agent development environment (via tmux):
./scripts/dev-tmux  # Includes new "agents" tab with:
# - General agents server (port 8003)
# - Hecate agent server (port 8001) 
# - Real-time log monitoring for all agents
# - Standardized logging with cyberpunk color themes

# 📊 Agent Log System:
# Console: Colored output with agent-specific themes
# Files: logs/hecate.log, logs/hecate-server.log, logs/hecate-startup.log
# Format: [timestamp] [AGENT] LEVEL message with emojis and colors
# Monitoring: Real-time log viewer in tmux agents tab

# Manual Hecate agent setup:
# 1. Start Hecate agent server for frontend chat integration
cd svc/nullblock-agents && python -m agents.hecate.server

# 2. Frontend will connect automatically on localhost:8001
# 3. Model selection handled by LLM Service Factory
# 4. Orchestration features demonstrate multi-agent coordination
# 5. Logs written to logs/hecate.log and logs/hecate-server.log

# 🔍 Reading Agent Logs:
# View real-time logs: tail -f logs/hecate-server.log
# Check startup logs: cat logs/hecate-startup.log  
# Monitor all agents: ./scripts/dev-tmux (agents tab, pane 3)
# Test logging: python test_hecate_logging.py

# Run specialized agents
python -m agents.arbitrage.price_agent
python -m agents.arbitrage.strategy_agent
python -m agents.information_gathering.main

# Code quality
ruff format . && ruff check . --fix && mypy .

# Testing
pytest -v src/tests/
```

### **🔥 Erebus Wallet Server** (`svc/erebus/`)
```bash
# Install dependencies
cargo build

# Development server
cargo run

# Development with auto-reload (install cargo-watch first)
cargo install cargo-watch
cargo watch -x run

# Code quality
cargo fmt
cargo clippy

# Testing
cargo test

# 🆕 NEW: Integration testing
cargo test test_full_integration_pipeline -- --nocapture
cargo test test_performance_benchmarks -- --nocapture
cargo test test_load_simulation -- --nocapture

# Release build
cargo build --release

# Check for compilation errors without running
cargo check

# Environment setup
# Server runs on localhost:3000 by default
# Wallet API endpoints ready for MCP integration
# 🆕 NEW: Comprehensive integration test suite with mock services
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
# Development server (requires Hecate agent for chat functionality)
npm run develop
ssr-boost dev

# Prerequisites for full chat functionality:
# 1. Start Hecate agent server: cd svc/nullblock-agents && python -m agents.hecate.server
# 2. Configure LLM models (LM Studio or API keys)
# 3. Chat interface will display current model and connection status

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
NullBlock implements a Model Context Protocol-first architecture for secure, agnostic agent interactions across multiple systems and data sources:

**Core MCP Features**:
- Agnostic system integration (Web3 wallets, Gmail accounts, APIs, databases, services)
- Universal context management with encrypted storage for agent workflows and user preferences
- Multi-protocol support for various authentication methods (challenge-response, OAuth, API keys)
- Advanced security middleware with prompt injection protection and input sanitization
- Standardized agent-to-system interfaces enabling seamless integration with new platforms
- Developer API and SDK for building custom agents and system integrations

**🆕 Hecate Agent Architecture** (August 2025):
- **Primary Interface Agent**: Hecate serves as the main user-facing conversational interface
- **Orchestration Engine**: Coordinates and delegates tasks across specialized agents
- **Decision-Making Hub**: Analyzes gathered data vs. task requirements for intelligent routing
- **Context Management**: Maintains conversation state and user preferences across sessions
- **LLM Integration**: Unified access to multiple language models with intelligent selection

**🆕 Domain-Specific Agent Extensions** (August 2025):
- Social media monitoring and sentiment analysis (Twitter/X, GMGN, DEXTools)
- Content creation and management (Gmail, Google Docs, social platforms)
- Financial data processing (trading APIs, DeFi protocols, market data)
- Communication automation (email, messaging, notifications)
- Comprehensive testing framework with debug utilities for all agent types

**Task Marketplace Integration**:
- Decentralized task submission and validation system
- Contributors earn rewards for high-impact agent workflows and strategies
- Community-driven consensus for task quality and impact assessment
- Extensible reward mechanisms supporting various token and point systems

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
- **HecateHud Interface**: Renamed from context-dashboard, now displays personalized user stats instead of generic system metrics
- **Gentle Wallet Messaging**: Non-aggressive error handling with info/error message types and soft blue styling for notifications
- **Wallet Name Resolution**: Click-to-edit wallet naming with SNS/ENS integration and localStorage persistence

### **Production MCP Architecture** ✅ (Nullblock.mcp)
- **Web Framework**: FastAPI with uvicorn ASGI server
- **Authentication**: Challenge-response wallet verification with session management
- **Context Storage**: IPFS-based encrypted storage with local caching
- **MEV Protection**: Flashbots client with bundle simulation and submission
- **Security**: ML-based prompt injection detection with anomaly detection
- **Multi-Wallet**: MetaMask, WalletConnect, Phantom support
- **API Structure**: RESTful MCP endpoints with comprehensive security middleware
- **🆕 Error Handling**: Fail-fast architecture with standardized error responses and comprehensive health checks
- **🆕 Service Validation**: Proper connection testing and service availability verification before processing requests

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
├── components/
│   ├── hecateHud/            # 🆕 Main interface dashboard (renamed from context-dashboard)
│   │   ├── hecateHud.tsx     # User-personalized stats and interface
│   │   └── hecateHud.module.scss
│   └── hud/                  # Core HUD system with wallet integration
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

### Monetization Strategy (Agentic Platform)
**Target Use Cases**:
- **Financial Automation**: 0.5-1% fees on automated trading, portfolio management, and DeFi operations
- **Content & Communication**: $10-$100/month subscriptions for automated content creation, email management, and social media coordination  
- **Data Intelligence**: $50-$500/month for automated data analysis, reporting, and insights across various data sources
- **Workflow Automation**: $25-$250/month for complex multi-step agent workflows and business process automation

**MCP Platform Monetization**:
- Freemium model: Free basic agent development tools, premium features $50-$500/month
- Transaction fees: 0.1% per agent-mediated operation across all systems
- White-label licensing for enterprises and service providers

**Platform Revenue**:
- Marketplace fee: 5-10% of user-created agent workflow revenue
- Task execution fees: $0.01-$0.05 per automated agent task
- Premium agent hosting: $10-$100/month for advanced analytics, priority execution, and enhanced security

The platform implements a cyberpunk aesthetic with neon styling, ball lightning visual effects, and maintains immersive error messages throughout the user experience while building toward a comprehensive MCP-powered agentic ecosystem.

## **📋 TODO LIST** (August 2025)

### **🆕 Immediate Tasks**
- [ ] **Ollama Integration**: Complete Ollama integration for secondary local model support
- [ ] **Model Performance Testing**: Benchmark LM Studio vs Ollama performance and reliability
- [ ] **Demo Price Display**: Further improve cryptocurrency price extraction from different API response formats
- [ ] **Error Recovery**: Implement automatic fallback between LM Studio and Ollama when one is unavailable
- [ ] **Documentation**: Create comprehensive setup guide for LM Studio configuration

### **🔄 Medium Priority**
- [ ] **Model Routing Optimization**: Fine-tune model selection algorithms for better performance
- [ ] **Cost Analysis**: Implement detailed cost tracking and comparison between local and cloud models
- [ ] **User Experience**: Improve error messages and user guidance for model setup
- [ ] **Testing**: Expand test coverage for local model integration scenarios

### **🎯 Long Term**
- [ ] **Multi-Model Support**: Support for additional local model servers (vLLM, Text Generation WebUI)
- [ ] **Model Management**: UI for managing and switching between different local models
- [ ] **Performance Monitoring**: Real-time monitoring of model performance and health
- [ ] **Community Integration**: Allow users to contribute and share model configurations

## **🏥 Health Monitoring & Service Standards** (August 2025)

### **Standardized Health Check System**
All Nullblock services implement consistent health monitoring with 5-minute intervals:

**📊 Health Endpoints (All Services)**:
- `GET /health` - Standardized health check endpoint
- **MCP Server**: `http://localhost:8001/health` - System components status
- **Orchestration**: `http://localhost:8002/health` - Workflow engine status  
- **General Agents**: `http://localhost:9001/health` - Agent service status
- **Hecate Agent**: `http://localhost:9002/health` - Agent running status & model info
- **Erebus Server**: `http://localhost:3000/health` - Wallet subsystems status

**🔄 Monitoring Schedule**:
- **Initial Checks**: 10-second intervals for first 30 seconds (fast startup detection)
- **Standard Monitoring**: 5-minute intervals for all services (reduced frequency)
- **Automated Rotation**: Log files with archival for long-term health tracking

**📝 Health Response Format**:
```json
{
  "status": "healthy|unhealthy",
  "service": "service-name", 
  "version": "0.1.0",
  "timestamp": "2025-08-19T05:59:25.724Z",
  "components": {
    "component1": "healthy|unhealthy",
    "component2": "healthy|unhealthy"
  }
}
```

**🎯 Health Monitor Integration**:
- **Tmux Logs Tab**: 55/45 split (logs/monitoring+commands)
- **Visual Status**: ✅/❌ indicators for all services
- **Service Count**: Real-time online/offline tracking
- **Log Integration**: Health status logged to persistent files

### **🎨 Visual Design System**
- **NullEye Animations**: Each NullEye features 8 randomized lightning arcs with varied sizes, orientations, and timing
- **Silver-Gold Lightning**: Consistent electrical effects using brilliant silver (#e8e8e8) with gold accents (#e6c200)
- **State-Responsive Design**: Core colors change based on agent state while maintaining consistent lightning
- **🆕 Idle State Theme**: Red-tinted, dimmed NullEye avatars with slower animations when Hecate agent is offline
- **Universal Clickability**: All NullEyes navigate to agent interfaces with enhanced hover feedback
- **Compact Layouts**: Command Lens uses 4-column responsive grids optimized for smaller screens
- **Smart Information Architecture**: Hover tooltips replace static text for cleaner, more interactive interfaces
- **Gentle Message System**: Blue info messages (#4a90e2) for notifications, red error messages (#ff3333) for failures, no aggressive animations for user guidance
- **Personalized User Interface**: HecateHud displays wallet-specific information (address, type, session time, connection status) instead of generic system stats