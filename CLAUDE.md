# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Nullblock is a cyberpunk-themed Web3 platform built on Solana featuring conversational LLM interactions, wallet analysis, and Memory Cards (NFTs). The system consists of three microservices in a monorepo structure:

- **Helios**: Python FastAPI backend with WebSocket support for real-time communication
- **Hecate**: React TypeScript frontend with SSR using @lomray/vite-ssr-boost  
- **Erebus**: Rust Solana contracts for blockchain operations

The platform includes an ECHO interface (chat-like UI) with room-based commands and plans for a browser extension called Aether.

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
just lint
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

### Frontend Architecture (Hecate)
- **SSR Framework**: @lomray/vite-ssr-boost for server-side rendering
- **State Management**: MobX with @lomray/react-mobx-manager
- **Routing**: @lomray/react-route-manager for isomorphic routing
- **Styling**: SCSS modules + Tailwind CSS
- **Build Tool**: Vite with React plugin
- **Wallet Integration**: @solana/web3.js with Phantom wallet support

### Backend Architecture (Helios)  
- **Web Framework**: FastAPI with uvicorn ASGI server
- **WebSocket Support**: Built-in for real-time communication with browser extension
- **API Structure**: RESTful endpoints + WebSocket endpoints for live data
- **Blockchain**: solana-py for Solana RPC interactions
- **Logging**: Custom logging with python-json-logger
- **Code Quality**: Ruff (linting/formatting) + MyPy (type checking)

### Key Backend Endpoints
- `/api/wallet/{public_key}` - Wallet data retrieval
- `/api/wallet/health/{public_key}` - Wallet health analysis  
- `/api/command` - Command processing for ECHO interface
- `/ws/ember-link/{client_id}` - WebSocket for frontend clients
- `/ws/aether` - WebSocket for browser extension data

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

### Command System
The ECHO interface uses a room-based command structure:

**Global Commands** (available everywhere):
- `/help`, `/status`, `/clear`, `/connect`, `/disconnect`, `/version`

**Room-Specific Commands**:
- `/logs` (default): `/trace`, `/history`, `/balance`, `/tokens`
- `/memory` (locked): `/mint`, `/upgrade`, `/features`, `/behavior`  
- `/health` (locked): `/risk`, `/audit`, `/monitor`, `/alerts`
- `/reality` (locked): `/spawn`, `/enhance`, `/interact`, `/sync`

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
- **Ember Link**: Real-time communication between frontend and backend
- **Aether Extension**: Planned browser extension WebSocket integration
- **Connection Management**: Custom EmberLinkManager class handles connections

The platform implements a cyberpunk aesthetic with neon styling and maintains immersive error messages throughout the user experience.