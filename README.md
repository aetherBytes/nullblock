# 🚀 Nullblock MVP - Decentralized Web3 Agentic Platform

**Status**: 🟢 **MVP Core Infrastructure Complete** - Ready for Phase 2 Development

Nullblock is a decentralized Web3 platform for deploying and monetizing agentic workflows, powered by the Model Context Protocol (MCP) architecture with MEV protection, Bittensor integration, and comprehensive security.

## 🎯 **MVP Status: Core Systems Delivered** ✅

### **Production-Ready Components**
- ✅ **Nullblock.mcp** - Complete MCP server with wallet authentication, IPFS context storage, Flashbots MEV protection
- ✅ **Nullblock.orchestration** - Goal-driven workflow engine with Bittensor subnet integration  
- ✅ **Nullblock.agents** - Full arbitrage agent suite (Price, Strategy, Execution, Reporting)
- 🔄 **Nullblock.platform** - dApp marketplace (pending frontend development)

### **Current Capabilities**
- ✅ **Full Arbitrage Trading Pipeline**: From price monitoring to MEV-protected execution
- ✅ **Secure Wallet Operations**: Multi-wallet support with challenge-response auth
- ✅ **Bittensor Task Marketplace**: Decentralized task submission with $NULL token rewards
- ✅ **Advanced Security**: Prompt injection protection, encrypted context storage
- ✅ **Goal-Driven Automation**: Template-based workflows for arbitrage, DeFi, NFT, and DAO operations

## 🏗️ **Architecture Overview**

```
┌─── NULLBLOCK.MCP (Security & Context Layer) ───┐
│  ✅ Wallet Authentication & Session Management  │
│  ✅ IPFS Context Storage with AES Encryption    │
│  ✅ Flashbots MEV Protection Client             │
│  ✅ ML-Based Prompt Injection Security          │
│  ✅ FastAPI Server with Security Middleware     │
└─────────────────────────────────────────────────┘
                     │
┌─── NULLBLOCK.ORCHESTRATION (Coordination) ───┐
│  ✅ Goal-Driven Workflow Orchestration         │
│  ✅ Bittensor Subnet Integration & Validation  │
│  ✅ Agent Task Coordination & Distribution     │
│  ✅ Template-Based Workflow Generation         │
│  ✅ $NULL Token Reward Distribution            │
└───────────────────────────────────────────────┘
                     │
┌─── NULLBLOCK.AGENTS (Execution Layer) ───┐
│  ✅ Price Monitoring & Opportunity Detection │
│  ✅ Risk Assessment & Strategy Analysis      │
│  ✅ MEV-Protected Trade Execution            │
│  ✅ Performance Analytics & Reporting        │
└─────────────────────────────────────────────┘
```

## 🚀 **Quick Start**

### **Option 1: Docker (Recommended)**

1. **Prerequisites**
   ```bash
   # Install Docker Desktop for Mac
   # Download from: https://www.docker.com/products/docker-desktop
   ```

2. **Clone and Start**
   ```bash
   git clone https://github.com/your-org/nullblock.git
   cd nullblock
   
   # Start all services
   ./start-nullblock.sh start
   ```

3. **Access Services**
   - Frontend: http://localhost:5173
   - MCP API: http://localhost:8000
   - Orchestration API: http://localhost:8001
   - Agents API: http://localhost:8002
   - IPFS Gateway: http://localhost:8080

### **Option 2: Local Development**

1. **Prerequisites**
   ```bash
   # Install Python 3.12, Node.js 18+, Rust
   brew install python@3.12 node rust
   
   # Install IPFS
   brew install ipfs
   ```

2. **Setup Development Environment**
   ```bash
   ./dev-setup.sh setup
   ```

3. **Start Services**
   ```bash
   # Manual setup (multiple terminals)
   # Terminal 1: Start infrastructure
   brew services start postgresql@15
   brew services start redis
   ipfs daemon --enable-gc
   
   # Terminal 2: MCP Server
   ./start-mcp.sh
   
   # Terminal 3: Orchestration
   ./start-orchestration.sh
   
   # Terminal 4: Agents
   ./start-agents.sh
   
   # Terminal 5: Frontend
   ./start-frontend.sh
   ```

### **Option 3: Tmuxinator (Advanced)**

For developers who prefer a comprehensive multi-pane tmux setup:

1. **Install Tmuxinator**
   ```bash
   gem install tmuxinator
   ```

2. **Setup Tmuxinator Config**
   ```bash
   # Copy the example config
   cp tmuxinator-config-example.yml ~/.config/tmuxinator/nullblock-dev.yml
   
   # Edit the root path to match your setup
   vim ~/.config/tmuxinator/nullblock-dev.yml
   ```

3. **Start Development Environment**
   ```bash
   # Start all services in organized tmux panes
   tmuxinator start nullblock-dev
   
   # Or use the shortcut script
   just dev-tmux
   ```

This will create a comprehensive tmux session with:
- **Infrastructure**: PostgreSQL, Redis, IPFS
- **Backend Services**: MCP, Orchestration, Agents  
- **Rust**: Erebus server and build
- **Frontend**: Hecate development server
- **Monitoring**: Health checks and service status
- **Tools**: Git status and development commands

## 📋 **Docker Commands**

```bash
# Start all services
./start-nullblock.sh start

# Stop all services
./start-nullblock.sh stop

# Restart all services
./start-nullblock.sh restart

# Build all services
./start-nullblock.sh build

# View logs
./start-nullblock.sh logs
./start-nullblock.sh logs nullblock-mcp

# Check status
./start-nullblock.sh status

# Health check
./start-nullblock.sh health

# Clean up everything
./start-nullblock.sh cleanup
```

## 🔧 **Configuration**

### **Environment Variables**

Create `.env` file (Docker) or `.env.dev` file (local development):

```bash
# Ethereum RPC URLs
ETHEREUM_RPC_URL=https://eth-mainnet.alchemyapi.io/v2/your-key
POLYGON_RPC_URL=https://polygon-mainnet.alchemyapi.io/v2/your-key
WEB3_RPC_URL=https://eth-mainnet.alchemyapi.io/v2/your-key

# Flashbots Configuration
FLASHBOTS_RPC_URL=https://relay.flashbots.net
FLASHBOTS_PRIVATE_KEY=your-flashbots-private-key
ENABLE_MEV_PROTECTION=true

# Bittensor Configuration
BITTENSOR_NETWORK=test
BITTENSOR_WALLET_PATH=your-bittensor-wallet-path

# Database Configuration
DATABASE_URL=postgresql://nullblock:REDACTED_DB_PASS@postgres:5432/nullblock
REDIS_URL=redis://redis:6379

# IPFS Configuration
IPFS_API_URL=http://ipfs:5001

# API Keys
DEX_API_KEYS=your-dex-api-keys

# Solana Configuration
SOLANA_RPC_URL=https://api.devnet.solana.com

# Frontend Configuration
VITE_MCP_API_URL=http://localhost:8000
VITE_ORCHESTRATION_API_URL=http://localhost:8001
VITE_AGENTS_API_URL=http://localhost:8002
```

## 🏗️ **Service Architecture**

### **Nullblock.mcp** (`svc/nullblock-mcp/`)
- **Purpose**: Secure tooling layer with MCP implementation
- **Features**: Wallet authentication, context storage, MEV protection, security
- **Port**: 8000
- **Health**: `/health`

### **Nullblock.orchestration** (`svc/nullblock-orchestration/`)
- **Purpose**: Goal-driven workflow coordination
- **Features**: Bittensor integration, task management, workflow templates
- **Port**: 8001
- **Health**: `/health`

### **Nullblock.agents** (`svc/nullblock-agents/`)
- **Purpose**: Modular agentic army for Web3 automation
- **Features**: Arbitrage agents, price monitoring, execution
- **Port**: 8002
- **Health**: `/health`

### **Hecate Frontend** (`svc/hecate/`)
- **Purpose**: React dApp with SSR
- **Features**: HUD interface, wallet integration, real-time updates
- **Port**: 5173
- **Build**: `npm run develop`

### **Erebus Contracts** (`svc/erebus/`)
- **Purpose**: Solana smart contracts
- **Features**: On-chain integration, Rust implementation
- **Port**: 8003
- **Build**: `cargo build`

## 🔒 **Security Features**

- **MEV Protection**: Flashbots integration prevents front-running
- **Prompt Injection Protection**: ML-based anomaly detection
- **Encrypted Context Storage**: IPFS with AES encryption
- **Wallet Security**: Challenge-response authentication
- **Input Sanitization**: Comprehensive validation and sanitization

## 💰 **Revenue Model**

- **Arbitrage Trading**: 0.5% trade fees on automated DEX arbitrage
- **MCP Transactions**: 0.1% fees on MCP-mediated transactions
- **Premium Subscriptions**: $50-$500/month for advanced features
- **$NULL Token Rewards**: Bittensor task contributions
- **Marketplace Fees**: 5% on user-created workflows (pending)

## 🚧 **Remaining Tasks for Full MVP**

1. **Nullblock.platform** - React dApp on Polygon (Frontend development)
2. **Marketplace Integration** - Workflow marketplace with 5% revenue sharing
3. **Polygon Testnet Deployment** - Infrastructure deployment and testing
4. **Beta User Onboarding** - Target 100 users in 30 days

## 🛠️ **Development**

### **Code Quality**
```bash
# Python services
cd svc/nullblock-mcp
ruff format . && ruff check . --fix && mypy .

# Frontend
cd svc/hecate
npm run lint:check && npm run ts:check
```

### **Testing**
```bash
# Python services
cd svc/nullblock-mcp
pytest -v src/tests/

# Frontend
cd svc/hecate
npm test
```

### **Building for Production**
```bash
# Frontend production build
cd svc/hecate
npm run build

# Docker production build
docker-compose -f docker-compose.yml --profile production up -d
```

## 📊 **Monitoring & Logs**

### **Service Logs**
```bash
# All services
docker-compose logs -f

# Specific service
docker-compose logs -f nullblock-mcp
```

### **Health Checks**
```bash
# Check all services
./start-nullblock.sh health

# Individual service health
curl http://localhost:8000/health  # MCP
curl http://localhost:8001/health  # Orchestration
curl http://localhost:8002/health  # Agents
```

## 🤝 **Contributing**

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests and linting
5. Submit a pull request

## 📄 **License**

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🆘 **Support**

- **Documentation**: [CLAUDE.md](CLAUDE.md)
- **Issues**: GitHub Issues
- **Discord**: [Join our community](https://discord.gg/nullblock)

---

**Built with ❤️ by the Nullblock Team**

*Empowering the future of decentralized agentic workflows*
