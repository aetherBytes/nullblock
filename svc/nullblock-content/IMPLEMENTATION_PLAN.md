# NullBlock Content Service - Implementation Plan

## Status: READY FOR IMPLEMENTATION

**Created:** 2026-02-02  
**By:** Moros (Mo)

---

## Overview

Build a Rust-based content generation and social media management service integrated with the Nullblock ecosystem.

---

## Service Architecture

```
                    Erebus Router (Port 3000)
                            ↓
                  NullBlock Content Service
                        (Port 8002)
                            ↓
        ┌───────────────────┼───────────────────┐
        ↓                   ↓                   ↓
   PostgreSQL           Kafka              External APIs
   (content DB)      (events)         (X/Twitter, etc.)
```

---

## Database Schema

### `content_queue`
```sql
CREATE TABLE content_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    theme VARCHAR(50) NOT NULL,
    text TEXT NOT NULL,
    tags TEXT[] DEFAULT '{}',
    image_prompt TEXT,
    image_path TEXT,
    status VARCHAR(20) DEFAULT 'pending',
    scheduled_at TIMESTAMPTZ,
    posted_at TIMESTAMPTZ,
    tweet_url TEXT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_content_queue_status ON content_queue(status);
CREATE INDEX idx_content_queue_theme ON content_queue(theme);
CREATE INDEX idx_content_queue_scheduled ON content_queue(scheduled_at);
```

### `content_metrics`
```sql
CREATE TABLE content_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    content_id UUID REFERENCES content_queue(id) ON DELETE CASCADE,
    likes INT DEFAULT 0,
    retweets INT DEFAULT 0,
    replies INT DEFAULT 0,
    impressions BIGINT DEFAULT 0,
    engagement_rate FLOAT,
    fetched_at TIMESTAMPTZ DEFAULT NOW(),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_content_metrics_content ON content_metrics(content_id);
```

### `content_templates`
```sql
CREATE TABLE content_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    theme VARCHAR(50) UNIQUE NOT NULL,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    templates JSONB NOT NULL,
    insights JSONB DEFAULT '[]',
    metadata JSONB DEFAULT '{}',
    active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_content_templates_theme ON content_templates(theme);
CREATE INDEX idx_content_templates_active ON content_templates(active);
```

---

## API Routes (via Erebus)

### Content Generation
- `POST /api/content/generate` - Generate content from theme
- `POST /api/content/generate/batch` - Generate multiple pieces

### Queue Management
- `GET /api/content/queue` - List pending content
- `GET /api/content/queue/:id` - Get specific content
- `PUT /api/content/queue/:id` - Update content
- `DELETE /api/content/queue/:id` - Delete content
- `POST /api/content/queue/:id/approve` - Approve for posting

### Posting
- `POST /api/content/queue/:id/post` - Post to social platform
- `POST /api/content/queue/batch-post` - Post multiple items

### Analytics
- `GET /api/content/posted` - List posted content
- `GET /api/content/metrics/:id` - Get metrics for content
- `GET /api/content/stats` - Aggregate statistics

### Templates
- `GET /api/content/templates` - List templates
- `GET /api/content/templates/:theme` - Get theme template
- `POST /api/content/templates` - Create template
- `PUT /api/content/templates/:theme` - Update template

### Scheduling
- `POST /api/content/schedule` - Schedule content generation
- `GET /api/content/schedules` - List schedules
- `DELETE /api/content/schedules/:id` - Remove schedule

---

## MCP Integration

### Tools Exposed via nullblock-protocols

1. **generate_content**
   - Parameters: `theme` (string), `include_image` (bool)
   - Returns: Generated content object

2. **post_content**
   - Parameters: `content_id` (uuid), `platform` (string)
   - Returns: Posted URL and metrics

3. **schedule_content**
   - Parameters: `theme` (string), `cron_expression` (string)
   - Returns: Schedule ID

4. **get_content_queue**
   - Parameters: `status` (optional), `limit` (optional)
   - Returns: Array of content items

### Resources

- `content://queue` - Access pending content
- `content://templates/{theme}` - Access theme templates
- `content://metrics/{id}` - Access content metrics

### Prompts

- `content_generation` - Generate themed content
- `content_review` - Review content before posting

---

## Kafka Events

### Published Events
- `content.generated` - New content created
- `content.posted` - Content published to platform
- `content.failed` - Posting failed
- `content.scheduled` - New schedule created

### Event Schema
```json
{
  "event_type": "content.generated",
  "content_id": "uuid",
  "theme": "MORNING_INSIGHT",
  "status": "pending",
  "metadata": {},
  "timestamp": "ISO8601"
}
```

---

## Content Generation Engine

### Themes
1. **MORNING_INSIGHT** - Daily motivation (9 AM)
2. **PROGRESS_UPDATE** - Dev updates (3 PM)
3. **EDUCATIONAL** - Deep dives (Wed noon)
4. **EERIE_FUN** - Vault-Tec PSAs (Sun 6 PM)
5. **COMMUNITY** - Engagement (6 PM daily)

### Template Structure
```rust
pub struct ContentTemplate {
    pub theme: String,
    pub templates: Vec<TemplateVariant>,
    pub insights: Vec<String>,
    pub reminders: Vec<String>,
    pub taglines: Vec<String>,
}

pub struct TemplateVariant {
    pub text: String,
    pub tags: Vec<String>,
    pub requires_image: bool,
}
```

### Generation Logic
1. Select random template variant
2. Replace placeholders with theme-specific content
3. Add appropriate tags
4. Generate image prompt if needed
5. Store in queue with status='pending'
6. Publish Kafka event

---

## Crossroads Integration

### Listing Metadata
```json
{
  "listing_type": "Tool",
  "title": "NullBlock Content Generator",
  "description": "Automated social media content generation with Vault-Tec energy. Generates themed content for X/Twitter with MCP protocol support.",
  "author": "NullBlock Core",
  "version": "0.1.0",
  "tags": ["content", "social-media", "automation", "mcp"],
  "is_free": true,
  "metadata": {
    "mcp_tools": ["generate_content", "post_content", "schedule_content"],
    "platforms": ["X/Twitter"],
    "themes": ["MORNING_INSIGHT", "PROGRESS_UPDATE", "EDUCATIONAL", "EERIE_FUN", "COMMUNITY"]
  }
}
```

---

## SKILL.md (ClawHub Compatibility)

The service will include a SKILL.md file at the root for ClawHub to import:

```markdown
---
name: nullblock-content
description: Social media content generation and posting service with MCP protocol support
homepage: https://github.com/aetherBytes/nullblock
metadata:
  openclaw:
    requires:
      services: ["nullblock-content"]
      database: ["postgres"]
---

# NullBlock Content Service

Generate and post social media content with Vault-Tec energy...
```

---

## File Structure

```
svc/nullblock-content/
├── Cargo.toml
├── README.md
├── SKILL.md
├── .env.example
├── migrations/
│   ├── 001_create_content_queue.sql
│   ├── 002_create_content_metrics.sql
│   └── 003_create_content_templates.sql
├── config/
│   └── templates.json
└── src/
    ├── main.rs
    ├── routes.rs
    ├── models.rs
    ├── repository.rs
    ├── services.rs
    ├── error.rs
    ├── database.rs
    ├── kafka.rs
    ├── handlers/
    │   ├── mod.rs
    │   ├── generate.rs
    │   ├── queue.rs
    │   ├── post.rs
    │   ├── templates.rs
    │   └── metrics.rs
    └── generator/
        ├── mod.rs
        ├── engine.rs
        ├── themes.rs
        └── templates.rs
```

---

## Implementation Steps

### Phase 1: Core Service (Day 1)
- [ ] Create Cargo.toml
- [ ] Implement models.rs
- [ ] Set up database.rs with SQLx
- [ ] Create migrations
- [ ] Build repository.rs (CRUD operations)

### Phase 2: Content Engine (Day 1-2)
- [ ] Implement generator/themes.rs
- [ ] Build generator/templates.rs
- [ ] Create generator/engine.rs
- [ ] Seed default templates

### Phase 3: API Layer (Day 2)
- [ ] Build routes.rs
- [ ] Implement handlers (generate, queue, post)
- [ ] Add error handling
- [ ] Set up main.rs with Axum

### Phase 4: Integration (Day 2-3)
- [ ] Add Kafka event publishing
- [ ] Create MCP tool definitions
- [ ] Register with Crossroads
- [ ] Add to Erebus routing

### Phase 5: Testing & Docs (Day 3)
- [ ] Integration tests
- [ ] API documentation
- [ ] Create SKILL.md
- [ ] Update main README

---

## Environment Variables

```bash
# Service
PORT=8002
SERVICE_NAME=nullblock-content
RUST_LOG=info

# Database
DATABASE_URL=postgresql://postgres:pass@localhost:5442/content

# Kafka
KAFKA_BOOTSTRAP_SERVERS=localhost:9092
KAFKA_TOPIC_PREFIX=nullblock

# External Services
EREBUS_URL=http://localhost:3000
PROTOCOLS_URL=http://localhost:8001
AGENTS_URL=http://localhost:9003

# Social Platforms (Future)
TWITTER_API_KEY=
TWITTER_API_SECRET=
```

---

## Next Steps

**Option A: Manual Implementation**
- I can build this file-by-file following the patterns
- Estimated: 2-3 hours of focused work
- Full control, proper review at each step

**Option B: Batch Implementation**
- Create all files at once using templates
- Faster initial scaffold
- Requires more iteration/debugging

**Option C: Hybrid**
- Core infrastructure first (Phase 1-2)
- Test locally
- Then add integration (Phase 3-4)

**Your call, architect.** How should we proceed? 🖤

---

*Status: AWAITING APPROVAL*  
*Ready to implement on your signal*
