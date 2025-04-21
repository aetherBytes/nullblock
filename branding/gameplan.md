# Nullblock - MCP/Client Interface Platform on Solana

**Start Date**: March 22, 2025 | **Target MVP**: May 2025

## Overview
Nullblock is an MCP-driven platform on Solana, featuring:
- **Helios**: MCP server for trading and wallet health tools.
- **Hecate**: Frontend service with the **ECHO** interface (Electronic Communication Hub and Omnitool).
- **Erebus**: Solana contract server. May be used to offload some of the logic from Helios when perf is needed.

**Objective**: Launch MVP: A MCP Server and Client Interface with minimal features. Basic wallet analysis, swap mechanics, and easy to implement tooling on the MCP side. H.U.D like experience via a conversational LLM interface, with user-specific state stored in mutable **Memory Cards** (NFTs). The user only needs a Phantom wallet to use the platform. All user related data is stored in the user's wallet / blockchain or within the memory card NFTs.

**First Impression** (Landing Screen):
The Nullblock landing screen immerses users in a eerie void a flickering digital campfire pulses at the center, casting neon embers into an endless abyss. Subtle cosmic elements (distant stars, faint planetary silhouettes, or swirling dust) drift in the background, creating an intoxicating, almost hypnotic atmosphere. A single, understated blip—the E.C.H.O menu button—glows in the bottom left, pulsing faintly like a beacon. Clicking it reveals a minimalistic H.U.D. overlay, sleek and futuristic, hinting at the platform’s depth. Users are prompted to connect their Web3 wallet (Phantom) or Solana ID, initiating their journey into Nullblock’s Matrix—a seamless entry to a personalized, MCP-driven Web3 experience.


## Tech Stack
- **Helios**: Multi-service backend system:
  - **helios-mcp**: Python, FastAPI - Dedicated MCP server for agent-based operations
  - **helios-api**: Python, FastAPI - Standard API server for direct operations
  - **helios-core**: Python - Shared core logic and utilities
  - Dependencies: `solana-py`, Helius RPC
- **Hecate (Frontend)**: TypeScript, React, Vite, SCSS, Tailwind, `@solana/web3.js`, Claude API, Any other LLM provider as well as a cheaper option.
- **Erebus (Contracts)**: Rust, Solana Rust SDK, gRPC
- **Storage**: Solana blockchain (Memory Cards as mutable NFTs via Metaplex)

## Development Phases

### Phase 1: Core Infrastructure (2-3 weeks)
- **Helios**:  
  - Basic command processing (/help, /status, /clear) ✓
  - Random response system for invalid commands ✓
  - Wallet data endpoints (mock data for now) ✓
  - Memory Card data structure ✓
  - Service Architecture:
    - Split into MCP, API, and Core packages
    - Shared utilities in helios-core
    - API endpoints for system analysis
    - MCP server for complex operations
- **Hecate**:  
  - ECHO chat interface with cyberpunk styling ✓
  - Locked features (all except /logs) ✓
  - "Translation matrix invalid" state ✓
  - Phantom wallet connection
- **Erebus**:  
  - Memory Card program structure ✓
  - Basic instruction handling ✓

### Phase 2: LLM Integration (2-3 weeks)
- **Helios**:  
  - Claude API integration
  - Command parsing and routing
  - Context management
- **Hecate**:  
  - Unlock translation matrix
  - Enable memory and health views
  - Add LLM response formatting
  - ECHO Interface Updates:
    - Base Camp Page:
      - User Profile Banner:
        - ID: Wallet address/name display (e.g. sage.sol)
        - Ascent: User level system (Solo Leveling inspired)
        - Nectar: Total resource value (SOL, tokens, NFTs)
        - Memories: Active Memory Card count
        - Matrix: User's translation matrix level/rarity
      - Matrix System:
        - Base Model: Basic LLM access
        - Rarity Tiers: Enhanced features and benefits
        - NFT-based progression system
- **Memory Cards**:  
  - Implement minting flow
  - Store conversation history
  - Track user preferences

### Phase 3: Wallet Features (2-3 weeks)
- **Helios**:  
  - Helius API integration
  - Real wallet health analysis
  - Token swap implementation
- **Erebus**:  
  - Raydium integration
  - Swap program
  - Transaction handling
- **Hecate**:  
  - Wallet status display
  - Transaction history
  - Risk analysis visualization

### Phase 4: Reality System (3-4 weeks)
- **Concept**: Virtual environment where users can interact with their Memory Cards
- **Features**:  
  - Reality interface unlocking
  - Memory Card upgrades
  - User achievements
  - Social features

## Command System
### Global Commands (Available in all rooms)
- `/help` - Display available commands ✓
- `/status` - Check system status ✓
- `/clear` - Clear chat log ✓
- `/connect` - Connect Phantom wallet
- `/disconnect` - Disconnect wallet
- `/version` - Display system version

### Room: /logs (Default Room)
- `/trace <tx>` - Analyze transaction
- `/history` - Show recent transactions
- `/balance` - Show wallet balance
- `/tokens` - List owned tokens

### Room: /memory (Locked)
- `/mint` - Create new Memory Card
- `/upgrade` - Enhance Memory Card
- `/features` - List available features
- `/behavior` - View behavior analysis

### Room: /health (Locked)
- `/risk` - Calculate wallet risk score
- `/audit` - Deep wallet analysis
- `/monitor` - Set up monitoring
- `/alerts` - Configure health alerts

### Room: /reality (Locked)
- `/spawn` - Enter reality interface
- `/enhance` - Upgrade environment
- `/interact` - Engage with Memory Card
- `/sync` - Synchronize state

## Development Progress
### Phase 1: Core Infrastructure
- [x] Basic command processing (/help, /status, /clear)
- [x] Random response system for invalid commands
- [x] Pretty-printed command output
- [x] Room-based command structure
- [x] Global vs room-specific commands
- [x] Wallet connection commands
- [ ] Transaction analysis tools

## Notes
- All features except basic logging are locked until LLM integration
- Memory Cards will be central to feature unlocking
- Focus on cyberpunk aesthetic and mysterious UI elements
- Keep error messages cryptic but helpful