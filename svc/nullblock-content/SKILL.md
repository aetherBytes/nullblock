---
name: nullblock-content
description: Social media content generation service with N.E.X.U.S. framework
homepage: https://github.com/aetherBytes/nullblock
version: 0.1.0
metadata:
  openclaw:
    requires:
      services: ["erebus", "nullblock-content"]
      database: ["postgres"]
    port: 8002
    routes:
      - "/api/content/*"
---

# NullBlock Content Service

Autonomous social media content generation and posting service for the NullBlock brand.

## Overview

The NullBlock Content Service generates themed social media content using the N.E.X.U.S. (Network, Execution, Expansion, Utility, Security) agent framework. It supports 5 content themes aligned with the NullBlock brand voice: cheerfully inevitable, professionally apocalyptic infrastructure content.

## Features

- **5 Content Themes:**
  - `MORNING_INSIGHT` - Daily infrastructure wisdom (9 AM)
  - `PROGRESS_UPDATE` - Development milestones (3 PM)
  - `EDUCATIONAL` - Agentic workflow deep dives (Wed noon)
  - `EERIE_FUN` - Dark AI humor (Sun 6 PM)
  - `COMMUNITY` - Engagement polls/questions (6 PM daily)

- **Template System:**
  - JSON-based template configuration
  - Placeholder replacement engine
  - 13 template variants with 40+ unique content pieces
  - Fallback to hardcoded defaults

- **Image Generation:**
  - Retro-futuristic propaganda style prompts
  - Optional image inclusion per theme
  - Constructivist + cyberpunk aesthetic

- **Event Publishing:**
  - `content.generated` - New content created
  - `content.posted` - Content published
  - `content.failed` - Error events
  - HTTP-based (Kafka-ready trait design)

- **Queue Management:**
  - Approval workflow (pending → approved → posted)
  - Status tracking and filtering
  - Metrics collection (likes, retweets, impressions)

## API Endpoints

All endpoints accessible via Erebus at `http://localhost:3000/api/content/*`

### Content Generation

```bash
POST /api/content/generate
```

**Request:**
```json
{
  "theme": "morning_insight",
  "include_image": false
}
```

**Response:**
```json
{
  "id": "uuid",
  "theme": "morning_insight",
  "text": "Protocols don't sleep. Neither should your infrastructure.\n\nShip protocols, not promises.",
  "tags": ["infrastructure", "morning", "protocols"],
  "image_prompt": null,
  "status": "pending",
  "created_at": "2026-02-02T12:00:00Z"
}
```

### Queue Management

```bash
GET /api/content/queue?status=pending&limit=10
GET /api/content/queue/:id
PUT /api/content/queue/:id
DELETE /api/content/queue/:id
```

### Analytics

```bash
GET /api/content/metrics/:id
GET /api/content/templates
```

## Configuration

### Environment Variables

```bash
# Service
PORT=8002
SERVICE_NAME=nullblock-content
RUST_LOG=info

# Database
DATABASE_URL=postgresql://postgres:password@localhost:5432/nullblock_content

# Templates
TEMPLATES_PATH=config/templates.json

# Events (optional)
EVENT_ENDPOINT=http://localhost:9000/events
```

### Database Setup

```sql
-- Run migrations in order
psql -d nullblock_content -f migrations/001_create_content_queue.sql
psql -d nullblock_content -f migrations/002_create_content_metrics.sql
psql -d nullblock_content -f migrations/003_create_content_templates.sql
```

## Usage

### Start Service

```bash
cd svc/nullblock-content
cargo run --release
```

### Generate Content

```bash
curl -X POST http://localhost:3000/api/content/generate \
  -H "Content-Type: application/json" \
  -d '{"theme":"morning_insight","include_image":false}'
```

### List Queue

```bash
curl http://localhost:3000/api/content/queue?status=pending
```

### Approve Content

```bash
curl -X PUT http://localhost:3000/api/content/queue/{id} \
  -H "Content-Type: application/json" \
  -d '{"status":"approved"}'
```

## Content Themes

### MORNING_INSIGHT
Daily infrastructure wisdom. Focus: protocols, autonomous systems, infrastructure.

**Example:**
> Protocols don't sleep. Neither should your infrastructure.
>
> Ship protocols, not promises.

**Tags:** infrastructure, morning, protocols

### PROGRESS_UPDATE
Development milestone announcements. Focus: transparency, shipping, building.

**Example:**
> Another module shipped. The system grows.
>
> Milestone: Agent coordination protocol — locked in.

**Tags:** progress, dev, building

### EDUCATIONAL
Deep dives into agentic concepts. Focus: education, workflows, infrastructure.

**Example:**
> Let's talk about agent-to-agent protocols.
>
> Agents aren't smart because of their models. They're smart because of their tooling.

**Tags:** educational, deepdive, agents

### EERIE_FUN
Dark AI humor with brand voice. Focus: autonomous systems, dark humor, inevitability.

**Example:**
> Your agents don't sleep. Neither does the inevitable.
>
> But hey, at least the uptime is great.

**Tags:** darkhumor, ai, autonomous

### COMMUNITY
Engagement questions and polls. Focus: community input, discussion.

**Example:**
> What's the most underrated piece of agent infrastructure?
>
> Drop your take below.

**Tags:** community, engagement, poll

## Brand Voice Guidelines

- **DO:** Cheerfully inevitable, professionally apocalyptic, infrastructure-focused
- **DO:** Dark humor about autonomous systems, corporate dystopia vibes
- **DO:** Focus on protocols, infrastructure, agentic networks
- **DON'T:** Fallout/Vault-Tec direct references (original voice only)
- **DON'T:** Financial advice, token shilling, day trading
- **DON'T:** Generic corporate buzzwords without substance

## N.E.X.U.S. Framework

Content generation follows the N.E.X.U.S. agent framework:

- **Network:** Coordination ability (themes align with community engagement)
- **Execution:** Task completion speed (automated generation pipeline)
- **Expansion:** Learning & scaling (template system adapts)
- **Utility:** Practical ROI (measurable engagement metrics)
- **Security:** Verifiability & trust (approval workflow before posting)

## Architecture

```
Frontend/CLI
     ↓
Erebus:3000 (/api/content/*)
     ↓
Content Service:8002
     ↓
  ┌────┴────┐
  ↓         ↓
PostgreSQL  Events → HTTP Endpoint
```

## Development

### Build

```bash
cargo build --release
```

### Test

```bash
cargo test
```

### Check

```bash
cargo check
```

## Integration

### With Erebus

All routes automatically proxied through Erebus router at `/api/content/*`. No direct access needed.

### Event Publishing

Events published to `EVENT_ENDPOINT` if configured. Future: Kafka integration.

### Database

Separate PostgreSQL database. Not shared with Erebus/Agents services.

## Troubleshooting

**Service won't start:**
- Check `DATABASE_URL` is set and accessible
- Verify migrations have been run
- Check `templates.json` exists or fallback is used

**Content generation fails:**
- Verify theme is valid: `morning_insight`, `progress_update`, `educational`, `eerie_fun`, `community`
- Check database connection
- Review logs for template loading errors

**No events published:**
- `EVENT_ENDPOINT` is optional - service works without it
- Check endpoint URL is accessible
- Review logs for publish failures (non-blocking)

## License

MIT

## Contributing

See main NullBlock repository for contribution guidelines.

---

*Built with Rust + Axum. The void where agentic content connects.*
