# Nullblock - MCP/Client Interface Platform on Solana

**Start Date**: March 22, 2025 | **Target MVP**: June 27, 2025

## Overview
Nullblock is a cutting-edge, MCP-driven platform built on Solana, designed to deliver a seamless Web3 experience through a conversational LLM interface and a cyberpunk-inspired aesthetic. The platform empowers users with wallet analysis, token swap mechanics, and personalized state management via mutable **Memory Cards** (NFTs). A new browser extension, **Aether**, enhances the experience by capturing live browser data and feeding it to the platform for contextual interactions. Key components include:

- **Helios**: Multi-service backend for MCP operations, wallet health tools, and browser data processing.
- **Hecate**: Frontend service featuring the **ECHO** interface (Electronic Communication Hub and Omnitool), a sleek H.U.D.-like UI for user interaction.
- **ECHO Chamber**: The landing screen and default experience, providing a read-only summary of system activity, agent logs, and updates. The ECHO Chamber is visually anchored by the ECHO device (bat logo) at the bottom center, which serves as the main entry point and "home" button for the interface.
- **Erebus**: Solana contract server for performance-critical logic, including token swaps and Memory Card operations.
- **Aether**: Browser extension that captures live browser data (e.g., active tab URL, page title) and syncs it with the platform for real-time display and LLM-driven insights.

**Objective**: Launch an MVP by June 27, 2025, featuring:
- A minimal MCP server and client interface with basic wallet analysis, token swap mechanics, and easy-to-implement tooling.
- A conversational LLM interface with a cyberpunk H.U.D.-like experience, accessible via a Phantom wallet.
- User-specific state stored in mutable **Memory Cards** (NFTs) on the Solana blockchain, including conversation history and browser data.
- A browser extension (**Aether**) that captures and sends live browser data to the platform, enabling contextual LLM responses and enhanced user engagement.
- A visually immersive ECHO Chamber landing screen, with the ECHO device (bat logo) as the main navigation entry point.

**First Impression (Landing Screen)**:
The Nullblock landing screen immerses users in a haunting digital void. A flickering neon campfire pulses at the center, casting vibrant embers into an infinite abyss. Subtle cosmic elements—distant stars, faint planetary silhouettes, and swirling dust clouds—drift in the background, creating a hypnotic, otherworldly atmosphere. A single, understated **ECHO device** (bat logo) glows at the bottom center, pulsing like a beacon in the darkness. Clicking it (or the Null logo) always returns the user to the ECHO Chamber, a read-only summary of system logs, agent status, and updates. The navigation is minimal: ECHO device (bat logo), Null logo, and social (X) button. This seamless entry point sets the stage for a personalized, MCP-driven Web3 journey, amplified by real-time browser context from **Aether**.

## Tech Stack
- **Helios (Backend)**:
  - **helios-mcp**: Python, FastAPI – Dedicated MCP server for agent-based operations and browser data processing.
  - **helios-api**: Python, FastAPI – REST and WebSocket endpoints for direct operations and real-time browser data ingestion.
  - **helios-core**: Python – Shared logic, utilities, and browser data parsing.
  - Dependencies: `solana-py`, Helius RPC, `websockets`, Redis (for temporary browser data caching).
- **Hecate (Frontend)**:
  - TypeScript, React, Vite, SCSS, Tailwind CSS.
  - Integrations: `@solana/web3.js` (Solana interactions), Claude API (LLM), WebSocket client (browser data).
- **Erebus (Contracts)**:
  - Rust, Solana Rust SDK, Anchor framework, gRPC.
  - Dependencies: Metaplex (NFTs), Raydium SDK (swaps).
- **Aether (Browser Extension)**:
  - JavaScript/TypeScript, WebExtensions API.
  - Integrations: `@solana/web3.js` (wallet authentication), `chrome.runtime` (browser APIs), WebSocket client.
- **Storage**:
  - Solana blockchain: Memory Cards as mutable NFTs via Metaplex for user data (conversation history, browser data, preferences).
  - Off-chain: Redis for temporary caching of browser data (5-10 minute TTL).

## Development Phases

### Phase 1: Core Infrastructure (3 weeks, March 22 - April 11, 2025)
**Goal**: Establish the foundational backend, frontend, contract, and browser extension infrastructure to support MVP functionality.
- **Helios**:
  - Implement basic command processing (/help, /status, /clear). ✓
  - Develop a random response system for invalid commands. ✓
  - Create mock wallet data endpoints for testing. ✓
  - Define Memory Card data structure (e.g., conversation history, browser data fields). ✓
  - Service Architecture:
    - Split into **helios-mcp**, **helios-api**, and **helios-core** packages.
    - Add WebSocket server in **helios-api** for **Aether** data ingestion.
    - Implement shared utilities in **helios-core** for browser data parsing and validation.
    - Set up Redis for temporary browser data caching.
- **Hecate**:
  - Build ECHO chat interface with cyberpunk styling (neon glow, glitch effects). ✓
  - Lock all features except /logs room. ✓
  - Implement "Translation matrix invalid" state for locked features. ✓
  - Enable Phantom wallet connection via `@solana/web3.js`.
  - Integrate WebSocket client to receive and display browser data from **Aether**.
- **Erebus**:
  - Scaffold Memory Card program using Anchor framework. ✓
  - Implement basic instruction handling for NFT minting. ✓
- **Aether**:
  - Create browser extension scaffold using WebExtensions API (targeting Chrome, Firefox).
  - Capture active tab URL and page title using `chrome.tabs` or equivalent.
  - Establish secure WebSocket connection to **helios-api** for data transmission.
  - Add optional Phantom wallet authentication for user identification.
  - Design minimal UI with a toggle to enable/disable data sharing.
- **Deliverables**:
  - Functional ECHO interface with wallet login and basic commands.
  - **Aether** prototype sending mock browser data (e.g., URLs) to **Hecate**.
  - Backend architecture with WebSocket support and mock wallet endpoints.

### Phase 2: LLM & Browser Integration (3 weeks, April 12 - May 2, 2025)
**Goal**: Integrate the LLM for conversational interactions and enable browser data to enhance contextual responses and Memory Card functionality.
- **Helios**:
  - Integrate Claude API for command parsing, response generation, and browser data contextualization.
  - Develop logic to process browser data (e.g., categorize URLs as DeFi, NFT, or other).
  - Store browser data in **Memory Cards** (e.g., top 5 recent URLs).
  - Implement caching for LLM responses to reduce API costs.
- **Hecate**:
  - Unlock translation matrix, enabling /memory and /health rooms.
  - Display browser data in ECHO interface (e.g., "Active Tab: raydium.io – DeFi").
  - Format LLM responses to incorporate browser context (e.g., "You're on a DeFi site, want to check your wallet?").
  - Update Base Camp Page:
    - **User Profile Banner**:
      - ID: Wallet address or Solana name (e.g., sage.sol).
      - Ascent: User level system (Solo Leveling-inspired).
      - Nectar: Total resource value (SOL, tokens, NFTs).
      - Memories: Active Memory Card count.
      - Matrix: Translation matrix level/rarity.
      - Active Context: Current browser tab info (URL, title).
    - **Matrix System**:
      - Base Model: Basic LLM access.
      - Rarity Tiers: Enhanced features based on browser activity (e.g., frequent DeFi visits).
      - NFT-based progression tied to Memory Cards.
- **Aether**:
  - Refine data capture to include page metadata (e.g., title, favicon).
  - Add user toggle for data sharing with clear privacy messaging.
  - Secure data transmission with TLS-encrypted WebSockets.
  - Test reliability across Chrome and Firefox.
- **Erebus**:
  - Finalize Memory Card minting flow using Metaplex.
  - Enable storage of conversation history and browser data in NFTs.
  - Implement user preference tracking (e.g., preferred DeFi platforms).
- **Deliverables**:
  - Conversational LLM interface with browser-contextual responses.
  - **Aether** sending live browser data to **Hecate**.
  - Memory Cards storing conversation and browser data.

### Phase 3: Wallet & Extension Features (3 weeks, May 3 - May 23, 2025)
**Goal**: Add core wallet functionality and enhance browser extension features to support real-time wallet and site interactions.
- **Helios**:
  - Integrate Helius API for real-time wallet health analysis (e.g., token balances, risk scores).
  - Implement token swap logic using Raydium SDK.
  - Correlate browser data with wallet actions (e.g., suggest swaps for DeFi sites).
  - Add endpoints for transaction history and risk analysis.
- **Hecate**:
  - Display wallet status (e.g., SOL balance, token holdings).
  - Show transaction history with browser context (e.g., "Swap initiated from raydium.io").
  - Visualize risk analysis (e.g., "High-risk token detected").
  - Add browser-driven alerts (e.g., "Warning: Unverified DeFi site").
- **Erebus**:
  - Deploy Raydium integration for token swaps.
  - Implement swap program and transaction handling with retry mechanisms.
  - Optimize Memory Card updates for gas efficiency.
- **Aether**:
  - Add interaction button in extension UI (e.g., "Trace Site" to trigger /trace command).
  - Apply cyberpunk styling to extension popup (neon borders, glitch effects).
  - Test cross-browser compatibility (Chrome, Firefox) and submit to Chrome Web Store/Firefox Add-ons.
- **Deliverables**:
  - MVP with wallet health analysis, token swaps, Memory Card functionality, and browser data integration.
  - Polished **Aether** extension with basic user interaction.

### Phase 4: Reality System & Extension Polish (3 weeks, May 24 - June 13, 2025)
**Goal**: Introduce the Reality System as a virtual environment for Memory Card interaction and finalize the browser extension for broader compatibility.
- **Helios**:
  - Enhance browser data processing for Reality System (e.g., virtual environment reflects browsing habits, like DeFi-themed rooms).
  - Add social feature endpoints (e.g., user achievements, leaderboards).
- **Hecate**:
  - Unlock Reality interface for Memory Card visualization and upgrades.
  - Display browser-driven achievements (e.g., "DeFi Explorer" for frequent DeFi visits).
  - Add social features (e.g., share Memory Card upgrades).
- **Erebus**:
  - Optimize Memory Card programs for scalability.
  - Add upgrade instructions for Memory Card enhancements.
- **Aether**:
  - Add support for Edge browser.
  - Optional: Capture additional metadata (e.g., page keywords) for Reality System.
  - Address feedback from Chrome/Firefox store reviews.
- **Deliverables**:
  - Polished **Aether** extension with cross-browser support.
  - Reality System as a stretch goal (deprioritized if timeline is tight).

### Phase 5: Testing & Launch (2 weeks, June 14 - June 27, 2025)
**Goal**: Conduct end-to-end testing, polish the platform, and launch the MVP to beta users.
- End-to-end testing of **Helios**, **Hecate**, **Erebus**, and **Aether**.
- Security audit for **Erebus** contracts (via OtterSec or Sec3) and **Aether** data handling.
- UI/UX refinements based on internal testing feedback.
- Beta launch with 500 users, including **Aether** installation guide.
- Collect user feedback via in-app form and **X** community.
- **Deliverables**:
  - Stable MVP with wallet analysis, swaps, Memory Cards, and browser data integration.
  - Beta-ready **Aether** extension with documented setup process.

## Command System
### Global Commands (Available in all rooms)
- `/help` – Display available commands. ✓
- `/status` – Check system status (including **Aether** connection). ✓
- `/clear` – Clear chat log. ✓
- `/connect` – Connect Phantom wallet.
- `/disconnect` – Disconnect wallet.
- `/version` – Display system version.
- `/browser` – Show current browser tab info (URL, title).
- `/context` – Display recent browser data stored in Memory Card.

### Room: /logs (Default Room)
- `/trace <tx>` – Analyze transaction with browser context (e.g., "Transaction from raydium.io").
- `/history` – Show recent transactions with site data.
- `/balance` – Display wallet balance.
- `/tokens` – List owned tokens.
- `/site` – Analyze current site's relevance to wallet (e.g., DeFi, NFT marketplace).

### Room: /memory (Locked)
- `/mint` – Create new Memory Card.
- `/upgrade` – Enhance Memory Card features.
- `/features` – List available Memory Card features.
- `/behavior` – View user behavior analysis (e.g., frequent DeFi visits).
- `/store` – Save current browser tab data to Memory Card.
- `/recall` – View stored browser history from Memory Card.

### Room: /health (Locked)
- `/risk` – Calculate wallet risk score (enhanced with browser context).
- `/audit` – Perform deep wallet analysis.
- `/monitor` – Set up wallet monitoring.
- `/alerts` – Configure health alerts (e.g., risky site warnings).

### Room: /reality (Locked)
- `/spawn` – Enter Reality interface.
- `/enhance` – Upgrade virtual environment.
- `/interact` – Engage with Memory Card in Reality.
- `/sync` – Synchronize Reality state with browser data.

## Development Progress
### Phase 1: Core Infrastructure
- [x] Basic command processing (/help, /status, /clear).
- [x] Random response system for invalid commands.
- [x] Pretty-printed command output.
- [x] Room-based command structure.
- [x] Global vs room-specific commands.
- [x] Wallet connection commands.
- [ ] Transaction analysis tools.
- [ ] **Aether** extension scaffold and WebSocket integration.
- [ ] WebSocket server in **helios-api**.
- [ ] Redis setup for browser data caching.

### Phase 2: LLM & Browser Integration
- [ ] Claude API integration.
- [ ] Browser data processing and storage in Memory Cards.
- [ ] Unlock translation matrix and /memory room.
- [ ] Display browser data in ECHO interface.
- [ ] **Aether** data capture refinement (metadata, privacy toggle).

### Phase 3: Wallet & Extension Features
- [ ] Helius API integration.
- [ ] Raydium swap implementation.
- [ ] Wallet status and risk visualization.
- [ ] **Aether** interaction button and store submission.

### Phase 4: Reality System & Extension Polish
- [ ] Reality interface implementation.
- [ ] Browser-driven achievements.
- [ ] **Aether** Edge support and polish.

### Phase 5: Testing & Launch
- [ ] End-to-end testing.
- [ ] Security audits.
- [ ] Beta launch with 500 users.

## Notes
- **Privacy & Security**:
  - **Aether** collects only active tab URL and page title for MVP, with opt-in consent and clear privacy messaging.
  - Browser data is stored on-chain in **Memory Cards** (encrypted if needed) to minimize server-side risks.
  - TLS-encrypted WebSockets ensure secure data transmission.
- **Performance**:
  - Cache LLM responses and browser data in Redis to reduce latency and API costs.
  - Implement WebSocket heartbeat to maintain **Aether**-**Helios** connection.
  - Optimize **Erebus** contracts for gas efficiency using Anchor.
- **User Experience**:
  - The ECHO Chamber is the default landing screen, providing a read-only summary of system activity, agent logs, and updates.
  - The ECHO device (bat logo) is the main entry point and "home" button, always returning the user to the ECHO Chamber.
  - Navigation is minimal and consistent: ECHO device (bat logo), Null logo, and social (X) button.
  - All previous main buttons have been removed for a cleaner, more focused interface.
  - Maintain cryptic, cyberpunk error messages (e.g., "Matrix misalignment detected") while ensuring they guide users.
  - Provide a setup guide for **Aether** installation in the landing screen.
  - Add tooltips in **Hecate** to explain locked features and browser data usage.
- **Community**:
  - Share teasers of the landing screen and **Aether** UI on **X** to build hype.
  - Create a **Discord** or **X** community for beta testers to provide feedback.
- **MVP Scope**:
  - Focus on core wallet features, Memory Card minting, and **Aether** URL/title capture.
  - Defer Reality System and advanced **Aether** features (e.g., page content analysis) to post-MVP.

## Metrics for Success
- **Development**:
  - Complete Phase 1 by April 11, 2025.
  - Achieve < 500ms latency for ECHO responses and < 1s for browser data display.
  - Ensure 99% transaction success rate for swaps and Memory Card minting.
- **User Engagement**:
  - Onboard 500 beta users by June 2025.
  - Achieve 50% **Aether** installation rate among beta users.
  - Collect 100+ feedback responses during beta.
- **Technical Stability**:
  - Maintain < 5% downtime for **Helios**, **Erebus**, and **Aether**.
  - Ensure **Aether** crash rate < 1% on Chrome/Firefox.
  - Achieve > 99% WebSocket uptime for **Aether**-**Helios** communication.

## DD on similar products / features
- ****