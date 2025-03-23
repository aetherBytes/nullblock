# Nullblock - MCP/Client Interface Platform on Solana

**Start Date**: March 22, 2025 | **Target MVP**: May 2025

## Overview
Nullblock is an MCP-driven platform on Solana, featuring:
- **Helios**: MCP server for trading and wallet health tools.
- **Hecate**: Frontend service with the **ECHO** interface (Electronic Communication Hub and Omnitool).
- **Erebus**: Solana contract server. May be used to offload some of the logic from Helios when perf is needed.

**Objective**: Launch MVP: A MCP Server and Client Interface with minimal features. Basic wallet analysis, swap mechanics, and easy to implement tooling on the MCP side. H.U.D like experience via a conversational LLM interface, with user-specific state stored in mutable **Memory Cards** (NFTs). The user only needs a Phantom wallet to use the platform. All user related data is stored in the user's wallet / blockchain or within the memory card NFTs.

## Tech Stack
- **Helios (MCP Server)**: Python, FastAPI, `solana-py`, Helius RPC
- **Hecate (Frontend)**: TypeScript, React, Vite, SCSS, Tailwind, `@solana/web3.js`, Claude API, Any other LLM provider as well as a cheaper option.
- **Erebus (Contracts)**: Rust, Solana Rust SDK, gRPC
- **Storage**: Solana blockchain (Memory Cards as mutable NFTs via Metaplex)

## Phase 1: MVP Core (6-8 weeks)
### Helios MCP Server
- **Tools**: `swapTokens`, `getWalletHealth`, `updateMemoryCard`
- **Logic**:  
  - Swap SOL/USDC on Raydium (via Erebus).  
  - Analyze wallet health (balance, risk) via Helius API.  
  - Update Memory Cards with user state (behaviors, events).  
- **On-Chain**:  
  - Blockchain data (trades, balances) fetched via Helius.  
  - Memory Cards store user-specific state (e.g., trade prefs).  
- **Agent Task**: "Code Helios with FastAPI, integrate Raydium swaps, update Memory Cards via Erebus, test 5 swaps + 5 health checks."

### Hecate Frontend Service (ECHO Interface)
- **UI**: ECHO chat box (LLM), balance/risk display  
- **Logic**:  
  - Parse commands (e.g., “Swap 0.1 SOL”) via Claude.  
  - Fetch Memory Cards from Phantom via `@solana/web3.js`.  
  - Display blockchain data from Helios.  
- **Phantom**: Connects via `window.solana`, signs TXs.  
- **Agent Task**: "Build Hecate extension with ECHO UI, wire LLM to Helios calls, read Memory Cards."

### Erebus Contract Server
- **Role**: Execute swaps, mint/update Memory Cards.  
- **Contracts**:  
  - Swap program (Raydium integration).  
  - Memory Card program (mint mutable NFTs, update metadata).  
- **Memory Cards**:  
  - Mutable NFTs bought by users (e.g., 0.1 SOL).  
  - Store: `userBehavior` (e.g., trade prefs), `eventLog` (e.g., actions), `features` (e.g., unlocked tools).  
- **Agent Task**: "Write Rust contracts for swaps and Memory Card minting/updates, deploy to Solana devnet, link to Helios via gRPC."

### Deliverables
- Helios on AWS, Hecate on GitHub, Erebus on Solana devnet, demo video.

## Phase 2: Early Growth (4-6 weeks)
- **Helios**: Add `scheduleTask`, expand health (e.g., TX history via Erebus).  
- **Hecate (ECHO)**: Add scheduler commands (e.g., “Swap tomorrow”), display TX trends.  
- **Erebus**: Deploy scheduled task contract, update Memory Cards with scheduler state.  
- **Release**: X post: "Nullblock MVP: Chat with your Phantom wallet!" | Goal: 200 users.

## Phase 3: Expansion (3-6 months)
- **Helios**: Add Skill Marketplace, Swarm Simulator, realities / missions that motivate agents involved?.  
- **Hecate (ECHO)**: Add Debate Viewer, skill UI, swarm commands.  
- **Erebus**: Optimize contracts, expand Memory Card features.  
- **Monetization**: SOL fees for premium tools (e.g., scheduled swaps).

## On-Chain Design
- **Data**: All on Solana.  
- **Memory Cards**: Mutable NFTs in Phantom wallets, storing user state (behaviors, events, features).  
- **Blockchain Data**: Trades, balances fetched by Helios via Helius RPC, not stored in Memory Cards.  
- **Access**: ECHO reads Memory Cards; Helios updates them via Erebus.

## Notes
- **Helios**: MCP backbone, fetches blockchain data, updates Memory Cards.  
- **Hecate**: Hosts ECHO, the sole user interface (Phantom + LLM).  
- **Erebus**: Performance layer, all Solana contracts.  
- **Agents**: Follow tasks, test endpoints, suggest features.