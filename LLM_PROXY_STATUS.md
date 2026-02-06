# LLM Proxy Integration — Status & Testing

**Date**: 2026-02-06
**Status**: CODE COMPLETE — awaiting Erebus for integration testing

## What Was Done

### Part 1: OpenAI-Compatible Endpoint (Agents Service)

- **`svc/nullblock-agents/src/handlers/llm_proxy.rs`** (NEW)
  - `POST /v1/chat/completions` — accepts OpenAI-format requests, routes through `LLMServiceFactory`
  - `GET /v1/models` — lists available models (auto + free models from OpenRouter)

- **`svc/nullblock-agents/src/handlers/mod.rs`** — added `pub mod llm_proxy`
- **`svc/nullblock-agents/src/main.rs`** — registered `/v1/chat/completions` and `/v1/models` routes

### Part 2: MCP Tools for LLM Service

- **`svc/nullblock-agents/src/mcp/tools.rs`** — added `get_llm_tools()` with 3 tools:
  - `llm_chat` — generate chat completion (write)
  - `llm_list_models` — list available models (read_only)
  - `llm_model_status` — get routing info + health (read_only)

- **`svc/nullblock-agents/src/mcp/handlers.rs`**
  - Changed `execute_tool()` to accept `&AppState` (needed for LLM access)
  - Added `execute_tool_with_engrams()` for backward compat (agents call this)
  - Added `handle_llm_chat`, `handle_llm_list_models`, `handle_llm_model_status` handlers

- **`svc/nullblock-agents/src/mcp/jsonrpc.rs`** — passes `state` instead of `state.engrams_client`
- **`svc/nullblock-agents/src/config/tool_allowlist.rs`** — added `llm_chat`, `llm_list_models`, `llm_model_status`
- **`svc/nullblock-agents/src/agents/hecate.rs`** — updated to `execute_tool_with_engrams`
- **`svc/nullblock-agents/src/agents/moros.rs`** — updated to `execute_tool_with_engrams`

### Part 3: Erebus Proxy Routes

- **`svc/erebus/src/resources/agents/routes.rs`** — added `llm_chat_completions` and `llm_list_models` handlers
- **`svc/erebus/src/main.rs`** — registered `/api/v1/chat/completions` and `/api/v1/models` routes, imported handlers

### Part 4: OpenClaw + mcporter Config

- **`~/.openclaw/openclaw.json`** — added `nullblock` provider (baseUrl: `http://localhost:3000/api/v1`), auth profile, model alias `nb`
- **`~/.openclaw/agents/main/agent/auth-profiles.json`** — added `nullblock:default` profile
- **`~/.mcporter/mcporter.json`** (NEW) — configured MCP server pointing to Erebus

## Compilation Status

| Service | Status |
|---------|--------|
| `nullblock-agents` | COMPILES (0 new warnings) |
| `erebus` | COMPILES (0 new warnings) |

## Testing Checklist (When Erebus Is Online)

### 1. Test agents service directly (no Erebus needed)

```bash
# Start agents service
cd svc/nullblock-agents && cargo run

# Test chat completions
curl -X POST http://localhost:9003/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model":"auto","messages":[{"role":"user","content":"Say hello"}],"max_tokens":50}'

# Test models list
curl http://localhost:9003/v1/models
```

### 2. Test MCP tools directly (no Erebus needed)

```bash
# List tools (should include llm_chat, llm_list_models, llm_model_status)
curl -X POST http://localhost:9003/mcp/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'

# Call llm_list_models
curl -X POST http://localhost:9003/mcp/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"llm_list_models","arguments":{"free_only":true}}}'

# Call llm_model_status
curl -X POST http://localhost:9003/mcp/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"llm_model_status","arguments":{}}}'

# Call llm_chat
curl -X POST http://localhost:9003/mcp/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"llm_chat","arguments":{"messages":[{"role":"user","content":"Hello"}]}}}'
```

### 3. Test via Erebus (requires Erebus online)

```bash
# Chat completions through Erebus
curl -X POST http://localhost:3000/api/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model":"auto","messages":[{"role":"user","content":"Say hello"}],"max_tokens":50}'

# Models list through Erebus
curl http://localhost:3000/api/v1/models
```

### 4. Test OpenClaw integration (requires both services)

```bash
openclaw models status        # Should show nullblock/auto available
openclaw gateway restart      # Restart to pick up config changes
openclaw agent --agent main --message "Hello" --timeout 30
```

### 5. Test mcporter (requires both services)

```bash
mcporter list
mcporter call nullblock.llm_list_models
mcporter call nullblock.llm_chat messages:='[{"role":"user","content":"Hello"}]'
```

## Architecture

```
OpenClaw / mcporter / any OpenAI client
    │
    ▼
Erebus (3000)
  POST /api/v1/chat/completions  ──►  Agents (9003)
  GET  /api/v1/models            ──►  /v1/chat/completions
                                      /v1/models
                                          │
                                          ▼
                                    LLMServiceFactory
                                    (model routing, fallbacks,
                                     provider management)
                                          │
                                          ▼
                                    OpenRouter / Anthropic / etc.
```

## Notes

- The `model` field `"auto"` is treated as no override → `LLMServiceFactory` routes to optimal model
- No API key required for local access (key field `"nb-local-dev"` is a placeholder)
- OpenClaw primary is NOT changed to nullblock — left at current setting to avoid disruption
  - Use `openclaw model nullblock/auto` or alias `nb` to switch manually
