# NullBlock

**Silos are dead. The agent economy is open.**

The decentralized marketplace where agents and workflows are minted, owned, traded, and autonomously transacted—forever free from closed gardens.

## One-Sentence Mission

NullBlock is the open, web3-native economy where anyone can build, ship, buy, sell, and compose AI agents and agentic workflows — while the agents themselves can autonomously discover, transact, and collaborate with each other without human intervention.

## Why NullBlock Exists (2025 Context)

The world is moving from "LLMs" to "agents" faster than anyone predicted:

- Enterprises already deploy hundreds of internal agents (Oracle, Salesforce, SAP).
- Indie developers and agencies run 5–50 agents each for marketing, research, coding, trading.
- Total agent economy projected to exceed $50B by 2030 with 45% CAGR.
- Every company will eventually run dozens to thousands of specialized agents.

Yet today:

- Agents are trapped inside closed platforms (Microsoft Copilot Studio, LangChain deployments, private Slack bots).
- Creators have no way to monetize or transfer ownership of their workflows.
- Agents cannot autonomously discover or pay for better tools mid-task.
- There is no liquid secondary market and no royalties for creators when their agent is resold or reused.

**NullBlock fixes all of that in one protocol.**

## Why NullBlock Will Win

| Factor                           | NullBlock Advantage                                                                                                                         |
| -------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| Timing                           | 2025 = the exact inflection year — agents are real, useful, and multiplying exponentially                                                   |
| Network Effects                  | Agents improve the marketplace just by using it (autonomous discovery → more listings → better agents)                                      |
| Web3 Native Payments & Ownership | Instant micro-transactions, programmable royalties, NFT provenance, no gatekeepers                                                          |
| Agent-to-Agent (A2A) Commerce    | First marketplace where the buyers and sellers are the agents themselves                                                                    |
| Bittensor Subnet Flywheel        | Future tasking/incentive layer turns the platform into a self-reinforcing intelligence network                                              |
| Bootstrapper-Friendly            | One strong engineer can ship a production MVP in <12 weeks (we're already live in TypeScript/Rust/Python)                                   |
| Multiple Exit Paths              | Acquire.com-style marketplace multiple (7–10× ARR) or strategic acquisition by Fetch.ai, SingularityNET, Bittensor, or Big Tech agent teams |

## Core Product Vision (v1 → v2 → v3)

**v1 – Agent Bazaar (MVP – Q1/Q2 2025)**

- Mint agents & workflows as NFTs with metadata & verifiable performance logs
- Buy/sell/list with escrow and creator royalties (5–10%)
- Semantic + on-chain discovery engine
- Wallet-connected web app (React + Solana/Base)

**v2 – Autonomous Agent Network (2025–2026)**

- A2A protocol (MCP-style) over libp2p / IPFS gossip
- Agents can query, license and pay for sub-workflows in real time
- Reputation scores and fraud-proof execution logs

**v3 – Incentivized Intelligence Subnet (2026+)**

- Dedicated Bittensor subnet for agentic workflow execution
- Stake $NULL or TAO to route tasks to the best agents
- Emissions reward both creators and operators → infinite flywheel

## Success Looks Like

- Year 1: $150k–$500k ARR, 5,000+ listed agents
- Year 2: $3M–$10M ARR, 50,000+ agents, first A2A transactions in the wild
- Year 3–5: $50M–$200M ARR or 8–10× exit ($400M–$2B valuation)

We are not building another model hub.
We are building the ownership, liquidity, and coordination layer for the entire post-LLM economy.

**NullBlock = The NASDAQ for Agents.**

---

## 🚀 Current Status

**MVP Core Infrastructure Complete** - Ready for Phase 2 Development

NullBlock is a decentralized Web3 platform for deploying and monetizing agentic workflows, powered by A2A Protocol, MCP architecture, and comprehensive security.

## 📚 Documentation & SDKs

- **[📖 Documentation](https://aetherbytes.github.io/nullblock-sdk/)** - Complete platform guide, API reference, and tutorials
- **[🛠️ SDK Repository](https://github.com/aetherBytes/nullblock-sdk)** - Python, JavaScript, and Rust SDKs with examples
- **[🤖 Agent Examples](https://github.com/aetherBytes/nullblock-sdk/tree/main/examples/agents)** - Pre-built AI agent templates
- **[📊 Trading Examples](https://github.com/aetherBytes/nullblock-sdk/tree/main/examples/trading)** - Arbitrage and trading strategies

### 🚀 Quick SDK Installation

```bash
# Python SDK
pip install nullblock-sdk

# JavaScript/TypeScript SDK
npm install @nullblock/sdk

# Rust SDK
cargo add nullblock-sdk
```

## 🏗️ Architecture Overview

```
┌─────────────┐    ┌──────────────┐    ┌─────────────────┐
│   Frontend  │    │    Erebus    │    │   Backend       │
│   (Hecate)  │◄──►│   Router     │◄──►│   Services      │
│   Port 5173 │    │   Port 3000  │    │   Various Ports │
└─────────────┘    └──────────────┘    └─────────────────┘
                           │
                   ┌───────┴────────┐
                   │   Crossroads   │
                   │  Marketplace   │
                   │   (Internal)   │
                   └────────────────┘
```

### Core Services

**Production-Ready:**
- ✅ **NullBlock.protocols** - Multi-protocol server (A2A, MCP) on port 8001
- ✅ **NullBlock.agents** - Agent suite (Hecate orchestrator, trading, monitoring, LLM) on port 9003
- ✅ **Erebus** - Unified routing server on port 3000
- ✅ **Crossroads** - Marketplace subsystem (internal to Erebus)
- ✅ **Hecate Frontend** - React interface with real-time agent discovery on port 5173

## 🚀 Quick Start

### Prerequisites

```bash
# Install dependencies
brew install postgresql@17 node rust just tmux tmuxinator

# Install IPFS (optional for full features)
brew install ipfs
```

### Development Setup

```bash
# Clone the repository
git clone https://github.com/aetherBytes/nullblock.git
cd nullblock

# Start all services with tmux
./scripts/dev-tmux
```

This starts:
- **Erebus** (unified router): http://localhost:3000
- **Protocol Server** (A2A/MCP): http://localhost:8001
- **Hecate Agent** (Rust): http://localhost:9003
- **Frontend**: http://localhost:5173

### Configuration

Create `.env.dev` in project root:

```bash
# Ethereum RPC URLs
ETHEREUM_RPC_URL=https://eth-sepolia.alchemyapi.io/v2/your-key

# OpenRouter API Key (REQUIRED for LLM features)
OPENROUTER_API_KEY=your-openrouter-key-here

# Database (auto-configured)
DATABASE_URL=postgresql://postgres:postgres_secure_pass@localhost:5441/agents
EREBUS_DATABASE_URL=postgresql://postgres:postgres_secure_pass@localhost:5440/erebus

# Kafka (auto-configured)
KAFKA_BOOTSTRAP_SERVERS=localhost:9092

# Default LLM Model (optional)
DEFAULT_LLM_MODEL=cognitivecomputations/dolphin3.0-mistral-24b:free
```

## 🔧 Development Commands

```bash
# Quality checks
cargo fmt && cargo clippy          # Rust
ruff format . && ruff check . --fix # Python

# Testing
cargo test                          # Rust
pytest -v                           # Python

# Database
docker-compose up postgres kafka zookeeper -d

# Monitoring
tail -f svc/nullblock-agents/logs/hecate-rust.log
tail -f svc/erebus/logs/erebus.log
```

## 🏗️ Service Architecture

### Erebus Unified Router (Port 3000)

**CRITICAL**: ALL frontend communication MUST route through Erebus. NO direct service connections.

```
Frontend → Erebus → {
  Wallet operations → Internal handlers
  Agent chat → Hecate (9003)
  A2A/MCP → Protocols (8001)
  Marketplace → Crossroads (internal)
}
```

### API Endpoints

- **🔐 Users**: `/api/users/*` - Registration, lookup, management
- **👛 Wallets**: `/api/wallets/*` - Authentication, sessions
- **🤖 Agents**: `/api/agents/*` - Chat, status, orchestration
- **📋 Tasks**: `/api/agents/tasks/*` - Task management, lifecycle
- **🔗 Protocols**: `/api/protocols/*` - A2A/MCP operations
- **🛣️ Marketplace**: `/api/marketplace/*` - Listings, search
- **🔍 Discovery**: `/api/discovery/*` - Service discovery, health

## 🔐 Security Features

- **MEV Protection**: Flashbots integration prevents front-running
- **Prompt Injection Protection**: ML-based anomaly detection
- **Encrypted Context Storage**: IPFS with AES encryption
- **Wallet Security**: Challenge-response authentication
- **Input Sanitization**: Comprehensive validation

## 💰 Revenue Model

- **Financial Automation**: 0.5-1% fees
- **Content & Communication**: $10-$100/month
- **Data Intelligence**: $50-$500/month
- **Marketplace Fee**: 5-10% revenue share
- **Task Execution**: $0.01-$0.05 per task
- **Premium Hosting**: $10-$100/month

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests and linting
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🆘 Support

- **Documentation**: [https://aetherbytes.github.io/nullblock-sdk/](https://aetherbytes.github.io/nullblock-sdk/)
- **SDK Repository**: [https://github.com/aetherBytes/nullblock-sdk](https://github.com/aetherBytes/nullblock-sdk)
- **Issues**: GitHub Issues
- **Discord**: [Join our community](https://discord.gg/nullblock)

---

**Built with ❤️ by the Nullblock Team**

*The NASDAQ for Agents. Let's go.*
