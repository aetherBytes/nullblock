# Nullblock MCP - Model Context Protocol

Secure Web3 agentic tooling layer with wallet authentication, context storage, MEV protection, and prompt injection defenses.

## Features

### 🔐 Wallet Authentication
- **MetaMask Support**: Web3 signature-based authentication
- **WalletConnect**: Multi-wallet compatibility
- **Phantom**: Solana wallet support (MVP implementation)
- **Session Management**: Secure session handling with expiration
- **Challenge-Response**: Cryptographic challenge verification

### 💾 Context Storage
- **IPFS Integration**: Decentralized context storage
- **Encryption**: AES encryption for sensitive data
- **Local Caching**: Performance optimization with local cache
- **User Preferences**: Trading settings, risk profiles, agent configurations
- **Versioning**: Context migration support

### ⚡ MEV Protection
- **Flashbots Integration**: Bundle submission for MEV protection
- **Bundle Simulation**: Pre-execution validation
- **Arbitrage Bundles**: Multi-transaction arbitrage protection
- **Gas Optimization**: Dynamic gas estimation
- **Fallback Support**: Graceful degradation when Flashbots unavailable

### 🛡️ Security Features
- **Prompt Injection Protection**: ML-based threat detection
- **Input Sanitization**: Comprehensive input cleaning
- **Rate Limiting**: Per-user request throttling
- **Command Validation**: Trading command allowlists
- **Anomaly Detection**: Behavioral analysis for suspicious inputs

## Quick Start

### Installation

```bash
cd svc/nullblock-mcp
pip install -e .
```

### Environment Setup

```bash
# Copy environment template
cp .env.example .env

# Edit with your configuration
ETHEREUM_RPC_URL=https://eth-mainnet.alchemyapi.io/v2/your-key
IPFS_API=/ip4/127.0.0.1/tcp/5001
FLASHBOTS_PRIVATE_KEY=your-private-key
ENABLE_MEV_PROTECTION=true
```

### Running the Server

```bash
# Development mode
python -m mcp.server

# Production mode
uvicorn mcp.server:create_server --host 0.0.0.0 --port 8000
```

## API Documentation

### Authentication Flow

1. **Create Challenge**
```bash
curl -X POST http://localhost:8000/auth/challenge \
  -H "Content-Type: application/json" \
  -d '{"wallet_address": "0x742d35Cc6D585Ec93BFB0..."}'
```

2. **Sign Message** (in your wallet)

3. **Verify Signature**
```bash
curl -X POST http://localhost:8000/auth/verify \
  -H "Content-Type: application/json" \
  -d '{
    "wallet_address": "0x742d35Cc6D585Ec93BFB0...",
    "signature": "0x8d5c3...",
    "provider": "metamask"
  }'
```

### Trading Commands

```bash
# Execute arbitrage (with session token)
curl -X POST http://localhost:8000/trading/command \
  -H "Authorization: Bearer your-session-token" \
  -H "Content-Type: application/json" \
  -d '{
    "command": "arbitrage",
    "parameters": {
      "from_token": "ETH",
      "to_token": "USDC",
      "min_profit": 0.01,
      "max_amount": 1000
    }
  }'
```

### Context Management

```bash
# Get user context
curl -X GET http://localhost:8000/context \
  -H "Authorization: Bearer your-session-token"

# Update preferences
curl -X POST http://localhost:8000/context/update \
  -H "Authorization: Bearer your-session-token" \
  -H "Content-Type: application/json" \
  -d '{
    "updates": {
      "risk_tolerance": "medium",
      "max_trade_amount": 5000,
      "preferred_dexes": ["uniswap", "sushiswap"]
    }
  }'
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Nullblock MCP Server                     │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │   Wallet    │  │   Context   │  │  Security   │         │
│  │    Auth     │  │   Storage   │  │ Protection  │         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
│                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │ Flashbots   │  │   Trading   │  │   Agent     │         │
│  │   Client    │  │  Commands   │  │ Integration │         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
└─────────────────────────────────────────────────────────────┘
```

## Security Considerations

### Prompt Injection Protection
- **Pattern Detection**: Regex-based malicious pattern detection
- **ML Anomaly Detection**: TF-IDF + Isolation Forest for anomaly detection
- **Input Sanitization**: HTML/script tag removal, character filtering
- **Command Validation**: Allowlist-based command validation

### MEV Protection
- **Bundle Transactions**: Private mempool submission via Flashbots
- **Simulation First**: Pre-execution bundle validation
- **Gas Optimization**: Dynamic gas pricing for better inclusion
- **Fallback Handling**: Graceful degradation when protection unavailable

### Data Security
- **Encrypted Storage**: AES encryption for sensitive context data
- **Session Management**: Time-limited sessions with proper cleanup
- **Rate Limiting**: Protection against DoS and abuse
- **Audit Logging**: Security event logging for monitoring

## Development

### Running Tests

```bash
pytest -v src/tests/
```

### Code Quality

```bash
# Formatting
ruff format .

# Linting
ruff check . --fix

# Type checking
mypy .
```

### Adding New Features

1. **New Wallet Provider**:
   - Extend `BaseWalletProvider` in `wallet/auth.py`
   - Implement signature verification and balance checking
   - Add to `WalletAuthenticator.providers`

2. **New Security Rule**:
   - Add patterns to `PromptInjectionDetector.malicious_patterns`
   - Update validation logic in `PromptProtectionManager`

3. **New Trading Command**:
   - Add command to `InputSanitizer.allowed_commands`
   - Implement handler in `MCPServer._execute_command`

## Integration Examples

### With Nullblock Agents

```python
from mcp.server import MCPServer
from agents.arbitrage import ArbitrageAgent

# Initialize MCP server
mcp_server = MCPServer()

# Create agent with MCP integration
arbitrage_agent = ArbitrageAgent(mcp_client=mcp_server)

# Agent automatically uses MCP for:
# - Wallet authentication
# - Context retrieval (user preferences)
# - MEV protection (Flashbots)
# - Security validation
```

### With Nullblock Platform

```typescript
// Frontend integration
import { MCPClient } from '@nullblock/mcp-client';

const mcp = new MCPClient('http://localhost:8000');

// Authenticate wallet
const challenge = await mcp.createChallenge(walletAddress);
const signature = await wallet.signMessage(challenge.message);
const session = await mcp.verifyChallenge(walletAddress, signature);

// Execute trading commands
const result = await mcp.executeCommand('arbitrage', {
  fromToken: 'ETH',
  toToken: 'USDC',
  minProfit: 0.01
});
```

## Roadmap

- [ ] **Multi-chain Support**: Polygon, Arbitrum, Optimism
- [ ] **Advanced MEV Strategies**: Custom bundle strategies
- [ ] **zk-SNARK Integration**: Zero-knowledge proof validation
- [ ] **Governance Integration**: DAO proposal automation
- [ ] **Cross-chain Context**: IPFS/Arweave integration
- [ ] **Agent Marketplace**: Third-party agent development SDK

## License

MIT License - see LICENSE file for details.