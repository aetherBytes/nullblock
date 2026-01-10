# Quick Start

## Prerequisites

- Rust (latest stable)
- Node.js 18+
- Docker Desktop
- tmux + tmuxinator

## One-Command Start

```bash
cd ~/nullblock
just dev-mac    # macOS
just dev-linux  # Linux
```

This starts:
- Docker infrastructure (PostgreSQL, Redis, Kafka)
- All Rust services (Erebus, Agents, Protocols, Engrams)
- Frontend dev server
- Chrome with DevTools debugging

## Manual Start

### 1. Infrastructure

```bash
cd ~/nullblock
just start
```

### 2. Individual Services

```bash
# Terminal 1 - Erebus Router
cd ~/nullblock/svc/erebus && cargo run

# Terminal 2 - Agents Service
cd ~/nullblock/svc/nullblock-agents && cargo run

# Terminal 3 - Protocols Service
cd ~/nullblock/svc/nullblock-protocols && cargo run

# Terminal 4 - Engrams Service
cd ~/nullblock/svc/nullblock-engrams && cargo run

# Terminal 5 - Frontend
cd ~/nullblock/svc/hecate && npm run develop
```

## Verify Services

```bash
# Health checks
curl http://localhost:3000/health   # Erebus
curl http://localhost:9003/health   # Agents
curl http://localhost:8001/health   # Protocols
curl http://localhost:9004/health   # Engrams
```

## Tmux Navigation

After `just dev-mac`, attach to the session:

```bash
tmux attach -t nullblock-dev
```

**Windows:**
- `Ctrl+B, 0` - Monitoring dashboard
- `Ctrl+B, 1` - Erebus
- `Ctrl+B, 2` - Agents
- `Ctrl+B, 3` - Protocols
- `Ctrl+B, 4` - Engrams
- `Ctrl+B, 5` - Task management
- `Ctrl+B, 6` - Frontend

## Access Points

| Service | URL |
|---------|-----|
| Frontend | http://localhost:5173 |
| Erebus API | http://localhost:3000 |
| Agents API | http://localhost:9003 |
| Protocols API | http://localhost:8001 |
| Engrams API | http://localhost:9004 |
| Chrome DevTools | localhost:9222 |
