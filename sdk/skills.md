# Nullblock SDK — Skills Integration Guide

## Crossroads + ClawHub Interoperability

Nullblock agents are designed to seamlessly consume **ClawHub skills** as remote MCP services, leveraging **Crossroads** as the orchestration layer.

### Officially Supported ClawHub Skills

| Skill | Type | Purpose | Integration Method |
|-------|------|---------|---------------------|
| `goplaces` | MCP | Geospatial awareness via Google Places | Auto-discovered via `/mcp/discover` |
| `bird` | MCP | Social sentiment, posting, engagement | Implemented as agent-triggered lambda |
| `1password` | MCP | Secure secret retrieval | Injected via `op` → Crossroads secrets bus |
| `mcporter` | MCP | Dynamic tool discovery | Core driver — exposes ClawHub as MCP endpoints |
| `nano-banana-pro` | MCP | Image generation | Called via `/mcp/image/generate` |
| `songsee` | MCP | Audio feature analysis | Fused with Hecate’s reasoning layer |
| `blogwatcher` | MCP | Event-triggered market signals | Auto-subscribed to RSS/Atom feeds |
| `github` | MCP | Dev-activity correlation | Triggers risk adjustments on commit signals |

### How to Add a ClawHub Skill to Crossroads

1. **Publish the skill on ClawHub:**
   ```bash
   clawhub publish ./path/to/skill/
   ```

2. **Run `clawhub sync`** on your agent to refresh local cache.

3. **Use `mcporter` to list available MCP servers:**
   ```bash
   mcporter list
   ```

4. **Call the skill from Crossroads via direct endpoint:**
   ```http
   POST /mcp/goplaces/search
   {
     "query": "coffee near me",
     "location": "Denver, CO"
   }
   ```

### Skill Definition Standards

All ClawHub skills must provide:
- ✅ `SKILL.md` — properly formatted with `run:`, `inputs:`, `outputs:`, `dependencies:`
- ✅ A binary or Node.js entrypoint (`main` field)
- ✅ **No hardcoded secrets** — all secrets passed via environment or 1Password injection
- ✅ **Output in JSON** — never human-readable text

Crossroads will auto-generate `skills.md` linting rules from these fields.

> 💡 Pro Tip: Use `clawhub validate ./your-skill` to verify compliance before publishing.