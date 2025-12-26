# Hecate Frontend

**React-based interface for the NullBlock agent orchestration platform**

## Overview

Hecate is the primary user interface for interacting with NullBlock agents, managing tasks, and exploring the Crossroads marketplace. Built with React, TypeScript, Three.js, and Vite for an immersive, responsive experience.

## Quick Start

### Prerequisites

- Node.js 18+ and npm
- Running Erebus router (port 3000)
- Running Agents service (port 9003)

### Development Setup

```bash
cd svc/hecate
npm install
npm run develop
```

The application will be available at `http://localhost:5173`

## Environment Configuration

Create a `.env` file in the `svc/hecate` directory:

```bash
VITE_EREBUS_API_URL=http://localhost:3000
VITE_PROTOCOLS_API_URL=http://localhost:8001
VITE_HECATE_API_URL=http://localhost:9003
```

**IMPORTANT**: All API calls must route through Erebus (port 3000).

## Core Features

### Void Experience (Home Screen)

The post-login home screen is an immersive 3D environment built with React Three Fiber:

- **CrossroadsOrb**: Central marketplace hub with gyroscope rings and animated sun surface
- **AgentClusters**: Orbiting AI agents (HECATE, Siren, Erebus) with click-to-focus interaction
- **HESSI-RHESSI**: Biometric-to-digital interface module for chat communication
- **ChatTendril**: GLSL shader-based energy beams connecting HESSI to HECATE
- **NeuralLines**: Constellation network of tools and services
- **ParticleField**: Ambient starfield background

### Chat Interface

The VoidChatHUD provides real-time communication with HECATE:

- Steam/energy effects during message transmission
- Tendril animations showing message flow
- Markdown rendering for agent responses
- History popup for conversation review

### Crossroads Marketplace

Browse and discover AI services:

- Agent listings with health status
- Workflow templates
- MCP server catalog
- Tool integrations

## Project Structure

```
src/
├── components/
│   ├── void-experience/     # 3D home screen
│   │   ├── scene/           # Three.js components
│   │   │   ├── CrossroadsOrb.tsx
│   │   │   ├── AgentCluster.tsx
│   │   │   ├── HessiRhessi.tsx
│   │   │   ├── ChatTendril.tsx
│   │   │   └── ...
│   │   └── chat/
│   │       └── VoidChatHUD.tsx
│   ├── crossroads/          # Marketplace UI
│   └── hud/                 # Panel overlays
├── common/
│   └── services/            # API clients
├── pages/                   # Route pages
└── types/                   # TypeScript types
```

## Key Technologies

- **React** + **TypeScript**: Core framework
- **React Three Fiber**: Three.js React renderer
- **@react-three/drei**: 3D utilities and helpers
- **GLSL**: Custom shaders for tendril effects
- **SCSS Modules**: Scoped styling
- **Vite**: Build tooling

## 3D Assets

Located in `public/models/`:

- `hecate-orb.glb`: HECATE vessel (MK1 hull)
- `HESSI-RHESSI.glb`: Biometric interface module

## Scripts

```bash
npm run develop   # Start dev server
npm run build     # Production build
npm run preview   # Preview production build
```

## License

NullBlock is open-sourced under the MIT License.
