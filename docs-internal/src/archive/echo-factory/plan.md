# Echo Factory Implementation Plan

> **STATUS: PAUSED** - Initiative paused in favor of [Poly Mev](../../poly-mev/plan.md). May resume after Polymarket swarm is operational.

**NullBlock's first COW** (Constellation of Work) - Autonomous X/Twitter content creation suite.

## Status

| Phase | Status | Notes |
|-------|--------|-------|
| **Phase 1: Engram Service** | ✅ Complete | Port 9004, all CRUD working |
| **Phase 1.5: MCP + Mem Cache** | 🔄 In Progress | MCP 2025-11-25 compliant, Mem Cache UI |
| **Phase 2: Crossroads COW Model** | ⏳ Next Up | COW as first-class listing |
| **Phase 3: Echo Factory Core** | ⏳ Pending | Persona, Content, Scheduler, Publisher |
| **Phase 4: X API Integration** | ⏳ Pending | OAuth 1.0a + real posting |
| **Phase 5: Frontend** | ⏳ Pending | Hecate UI components |

### Phase 1.5 Progress

| Component | Status |
|-----------|--------|
| MCP Protocol Version 2025-11-25 | ✅ Complete |
| MCP Client (Hecate) | ✅ Complete |
| MCP Proxy (Erebus) | ✅ Complete |
| Engram MCP Tools (9 total) | ✅ Complete |
| Mem Cache Tab | 🔄 In Progress |
| Crossroads MCP Display | ⏳ Pending |

## Architecture Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Engrams** | Separate service | Context for ALL workflows |
| **COWs** | Replace Workflows | First-class listing type |
| **Echo Factory** | Isolated package | Reference implementation |
| **MVP Focus** | X posting | Persona → Content → Schedule → Publish |

## Dogfooding Principle

Echo Factory MUST use only public APIs through Erebus. No privileged access.

**External developers get the same experience:**

| Access Method | Description |
|---------------|-------------|
| **REST API** | All endpoints via Erebus (3000) |
| **MCP Server** | NullBlock MCP tools |
| **NullBlock Studio** | Web GUI (future) |
| **SDK** | `nullblock-sdk` package |

## The 4 COW Tools

| Tool | Standalone | Required | Description |
|------|------------|----------|-------------|
| **Persona Creator** | Yes | Yes | Create X personas with voice, tone |
| **Content Generator** | Yes | Yes | Generate posts via Siren |
| **Scheduler** | Yes | No | Queue posts for optimal times |
| **X Publisher** | No | Yes | OAuth + X API posting |

## Echo Factory Service Structure

```
svc/echo-factory/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── models/
│   │   ├── persona.rs
│   │   ├── content.rs
│   │   └── schedule.rs
│   ├── handlers/
│   │   ├── persona.rs
│   │   ├── content.rs
│   │   └── schedule.rs
│   ├── tools/
│   │   ├── persona_creator.rs
│   │   ├── content_generator.rs
│   │   ├── scheduler.rs
│   │   └── x_publisher.rs
│   └── services/
│       ├── x_client.rs
│       └── engram_client.rs
└── migrations/
```

## API Endpoints (via Erebus)

### Personas

```bash
POST /api/echo/personas          # Create
GET  /api/echo/personas          # List
GET  /api/echo/personas/:id      # Get
PUT  /api/echo/personas/:id      # Update
POST /api/echo/personas/:id/connect-x  # OAuth
```

### Content

```bash
POST /api/echo/content/generate   # Generate
POST /api/echo/content/schedule   # Schedule
GET  /api/echo/content/scheduled  # List scheduled
POST /api/echo/content/:id/publish-now  # Publish
```

## Critical Risks

| Risk | Mitigation |
|------|------------|
| X API access delayed | Start application NOW |
| X API rate limits | Aggressive queue rate limiting |
| Content quality | Store successes in Engrams |
| OAuth expiration | Token refresh flow |

## Related

- [Engram Integration](./engrams.md)
- [Engrams Service](../../services/engrams.md)
- [Crossroads Marketplace](../../services/crossroads.md)
