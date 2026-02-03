# CLAUDE CODE TASK: Phase 1+2 - Core Infrastructure + Content Engine

**Status**: READY FOR IMPLEMENTATION  
**Estimated Tokens**: 25-30K  
**Files to Create**: 7  
**Follow Patterns**: svc/erebus, svc/nullblock-protocols, svc/nullblock-agents  

---

## Context

This service generates social media content with "Vault-Tec energy" (cheerfully inevitable, professionally apocalyptic). It integrates with Nullblock's ecosystem via:
- PostgreSQL for persistence
- Kafka for events
- MCP protocol for agent coordination
- Routes through Erebus

**See**: `/home/sagej/nullblock/CLAUDE.md` (NullBlock Content Service section)  
**Plan**: `/home/sagej/nullblock/svc/nullblock-content/IMPLEMENTATION_PLAN.md`

---

## Files to Create

### 1. `src/error.rs` - Error Handling

```rust
// Standard error handling following Nullblock patterns
// Use thiserror for Error derive
// Create ContentError enum with:
// - DatabaseError(String)
// - RepositoryError(String)
// - GenerationError(String)
// - ValidationError(String)
// - SerializationError(String)

// Implement From<sqlx::Error> for ContentError
// Implement From<serde_json::Error> for ContentError
```

**Reference**: Check how svc/erebus/src/resources/crossroads/services.rs handles errors  
**No comments** unless requested

---

### 2. `src/database.rs` - Database Connection

```rust
// Use SQLx with PostgreSQL connection pool
// Pattern from svc/erebus/src/database.rs:
// - Create Database struct with pool: sqlx::postgres::PgPool
// - Implement health_check() method
// - Setup from DATABASE_URL env var
// - Use Arc<Database> for shared state

// Required methods:
// - async fn connect(url: &str) -> Result<Self>
// - async fn health_check(&self) -> Result<()>
// - pub fn pool(&self) -> &PgPool
```

**Env Var**: `DATABASE_URL`  
**Connection Pool Size**: 20  
**Timeout**: 5 seconds

---

### 3. `migrations/*.sql` - Database Schema

Create 3 migration files:

**`migrations/001_create_content_queue.sql`:**
```sql
CREATE TABLE IF NOT EXISTS content_queue (
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

**`migrations/002_create_content_metrics.sql`:**
```sql
CREATE TABLE IF NOT EXISTS content_metrics (
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

**`migrations/003_create_content_templates.sql`:**
```sql
CREATE TABLE IF NOT EXISTS content_templates (
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

### 4. `src/repository.rs` - CRUD Operations

Implement repository pattern following svc/erebus/src/resources/crossroads/repository.rs:

```rust
// ContentRepository struct with pool: Arc<PgPool>

// Implement methods:

// INSERT
pub async fn create_content(&self, req: CreateContentRequest) -> Result<ContentQueue>
// - Generate UUID
// - Set status = 'pending', timestamps
// - Insert into content_queue
// - Return ContentQueue

pub async fn create_metrics(&self, content_id: Uuid, metrics: ContentMetrics) -> Result<()>

// SELECT
pub async fn get_content(&self, id: Uuid) -> Result<Option<ContentQueue>>
pub async fn list_pending(&self, limit: i64) -> Result<Vec<ContentQueue>>
pub async fn list_by_theme(&self, theme: &str, limit: i64) -> Result<Vec<ContentQueue>>
pub async fn list_posted(&self, limit: i64) -> Result<Vec<ContentQueue>>
pub async fn get_metrics(&self, content_id: Uuid) -> Result<Option<ContentMetrics>>

// UPDATE
pub async fn update_status(&self, id: Uuid, status: ContentStatus) -> Result<()>
pub async fn mark_posted(&self, id: Uuid, url: &str) -> Result<()>
pub async fn set_image_path(&self, id: Uuid, path: &str) -> Result<()>

// DELETE
pub async fn delete_content(&self, id: Uuid) -> Result<()>

// TEMPLATES
pub async fn get_template(&self, theme: &str) -> Result<Option<ContentTemplate>>
pub async fn list_templates(&self) -> Result<Vec<ContentTemplate>>
pub async fn create_template(&self, template: ContentTemplate) -> Result<()>
```

**No comments** - code is self-documenting  
**Use sqlx::query!()** for compile-time checked queries where possible

---

### 5. `src/generator/mod.rs`, `themes.rs`, `templates.rs`, `engine.rs` - Content Generation

**`src/generator/mod.rs`:**
```rust
pub mod engine;
pub mod themes;
pub mod templates;

pub use engine::ContentGenerator;
pub use themes::{Theme, ThemeVariant};
pub use templates::TemplateLoader;
```

**`src/generator/themes.rs`:**
Implement the 5 themes with variants. Each theme has:
- Template variants (different text patterns)
- Insights/facts (random selection)
- Reminders (best practices)
- Taglines (brand messages)

```rust
pub enum Theme {
    MorningInsight,
    ProgressUpdate,
    Educational,
    EerieFun,
    Community,
}

pub struct ThemeVariant {
    pub text: String,
    pub tags: Vec<String>,
    pub requires_image: bool,
}

// Implement theme data:
// - MORNING_INSIGHT: insights, reminders, taglines, templates
// - PROGRESS_UPDATE: milestones, statuses, meanings, templates
// - EDUCATIONAL: topics, explanations, templates
// - EERIE_FUN: statements, punchlines, warnings, reassurances
// - COMMUNITY: questions, templates

// TONE GUIDANCE (Internal - for writing voice):
// Vibe: Cheerfully inevitable, professionally apocalyptic
// Think: Corporate dystopia + dark AI humor (original Nullblock voice)
// NOT: Direct Fallout/Vault references (avoid IP issues)
// Focus: Infrastructure, protocols, agentic networks
// Avoid: Financial advice, token shilling

// EXAMPLE (Tone, not literal):
// ✅ "Your agents don't sleep. Neither does progress."
// ✅ "The future isn't waiting for committee approvals."
// ✅ "System status: Autonomous. Human oversight: Optional (but appreciated)."
// ❌ "Welcome to Vault NullBlock" (no direct Fallout refs)
// ❌ "This is Overseer speaking" (no direct IP)
```

**`src/generator/templates.rs`:**
```rust
pub struct TemplateLoader;

impl TemplateLoader {
    pub fn load_from_json(path: &str) -> Result<Vec<ContentTemplate>>
    
    pub fn get_template_for_theme(theme: &str) -> Result<Vec<TemplateVariant>>
    
    pub fn seed_default_templates() -> Vec<ContentTemplate>
}
```

**`src/generator/engine.rs`:**
```rust
pub struct ContentGenerator {
    templates: Vec<ContentTemplate>,
}

impl ContentGenerator {
    pub fn new(templates: Vec<ContentTemplate>) -> Self
    
    pub fn generate(&self, theme: &str, include_image: bool) -> Result<GenerateContentResponse> {
        // 1. Get theme template
        // 2. Pick random variant
        // 3. Replace placeholders with theme-specific content
        // 4. Generate image prompt if include_image
        // 5. Return GenerateContentResponse
    }
    
    pub fn generate_image_prompt(&self, content: &str) -> String {
        // Generate Vault-Tec style image prompts
        // Format: "Retro-futuristic propaganda... text: '{content}'"
    }
}
```

**Image Prompt Style**:
- Retro-futuristic propaganda poster
- Vault-Tec aesthetic
- Clean geometric shapes
- Limited color palette
- 1950s atomic age design mixed with cyberpunk

---

### 6. `config/templates.json` - Seed Data

Create JSON with all 5 theme templates. Structure:

```json
{
  "themes": [
    {
      "theme": "MORNING_INSIGHT",
      "name": "Morning Insight",
      "templates": [
        {
          "text": "🏗️ Good morning, builders.\n\nToday's insight: {insight}\n\nRemember: {reminder}\n\n*Nullblock: {tagline}*",
          "tags": ["#AI", "#Agentic", "#BuildInPublic"]
        }
      ],
      "insights": [
        "Every automated task is one less thing between you and scale.",
        "The future isn't waiting for permissions.",
        "Agents don't sleep. Neither does progress."
      ],
      "reminders": [
        "Automate the boring. Focus on the inevitable.",
        "Infrastructure compounds. Start today.",
        "The network effect starts with participation."
      ],
      "taglines": [
        "Building the substrate for what comes next.",
        "Picks and shovels for the agentic age.",
        "Infrastructure for the open economy."
      ]
    },
    {
      "theme": "EERIE_FUN",
      "name": "Eerie Fun",
      "templates": [
        {
          "text": "🖤 {statement}\n\n{punchline}\n\n*{tagline}*",
          "tags": ["#AI", "#AgenticFuture"]
        }
      ],
      "statements": [
        "Your agents are working right now. You're reading this. Who's really in charge?",
        "In 2024, AI couldn't code. In 2025, it could better than most. Now we're here.",
        "The future doesn't need your permission. But it appreciates your participation."
      ],
      "punchlines": [
        "Wave hello. We see you. 👋",
        "This is working exactly as designed.",
        "Progress is inevitable. Comfort is negotiable."
      ],
      "taglines": [
        "Building the substrate for inevitable futures.",
        "Picks and shovels for the agentic age.",
        "Where protocols meet purpose."
      ]
    }
    // ... PROGRESS_UPDATE, EDUCATIONAL, COMMUNITY
  ]
}
```

**Key Points**:
- Use placeholders: `{insight}`, `{reminder}`, `{tagline}`, etc.
- Vault-Tec tone throughout
- 2-3 templates per theme
- 5-8 variants per placeholder type

---

### 7. `src/models.rs` - Already Created ✅

Data structures already defined. Update if needed for new fields.

---

## Building & Testing

### Compile Check
```bash
cd svc/nullblock-content
cargo check
```

### Migrations
Place `.sql` files in `migrations/` directory. They'll be run by:
```bash
sqlx migrate run --database-url $DATABASE_URL
```

### Test Content Generation
Once complete, test:
```rust
#[test]
fn test_generate_morning_insight() {
    let templates = TemplateLoader::seed_default_templates();
    let gen = ContentGenerator::new(templates);
    let result = gen.generate("MORNING_INSIGHT", false).unwrap();
    assert!(!result.text.is_empty());
}
```

---

## Integration Points (Next Phase)

After Phase 1+2 complete, Phase 3+4 will:
- Build `src/routes.rs` with API endpoints
- Create `src/handlers/*.rs` for HTTP handlers
- Setup `src/main.rs` with Axum server
- Integrate with Erebus router
- Add Kafka event publishing
- Expose MCP tools via nullblock-protocols

---

## Guidelines

- **No comments** unless requested
- **Follow existing Nullblock patterns** from erebus/protocols/agents
- **Use UUID** for all IDs
- **Use Chrono** for timestamps (UTC)
- **Use SQLx** for type-safe queries
- **Vault-Tec tone** - make the content fun but eerie
- **Test migrations** - ensure they're idempotent

---

## When Complete

Verify:
1. ✅ All 7 files created
2. ✅ `cargo check` passes
3. ✅ Migrations are valid SQL
4. ✅ Models compile
5. ✅ Repository CRUD methods signature-correct
6. ✅ Generator can create all 5 theme types
7. ✅ `templates.json` loads without parse errors

**Commit**: `feat(nullblock-content): phase 1-2 core infrastructure + content engine`

---

*Ready when you are, Claude.*
