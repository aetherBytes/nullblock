# Hecate Frontend TODO: Profile, Memory & Settings UI

Backend is complete. These are the frontend tasks to wire up the new Hecate capabilities.

## Endpoints Available

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/mcp/jsonrpc` `tools/call` | Execute any MCP tool |
| GET | `/hecate/tools` | List all available MCP tools |
| POST | `/hecate/chat` | Chat (now with function calling) |
| POST | `/hecate/clear` | Clear conversation (now accepts optional `wallet_address` body to save session before clearing) |
| GET | `/api/users/:user_id/api-keys` | List user API keys (masked) |
| POST | `/api/users/:user_id/api-keys` | Add user API key |
| DELETE | `/api/users/:user_id/api-keys/:key_id` | Delete user API key |

## Tasks

### 1. Profile Settings Page
- [ ] New route: `/settings/profile`
- [ ] Display current profile from `user_profile_get` tool
- [ ] Editable fields: display_name, bio, title, experience level, interests
- [ ] Save via `user_profile_update` tool (field + content JSON)
- [ ] Show avatar placeholder (avatar_url field exists but no upload yet)

### 2. API Key Management
- [ ] New route: `/settings/api-keys`
- [ ] List current keys via `GET /api/users/:user_id/api-keys` (shows masked keys)
- [ ] Add key form: provider dropdown (openrouter, anthropic, openai, groq) + key input
- [ ] Delete key button with confirmation
- [ ] Show which providers are active (green dot / badge)
- [ ] Info text: "Add your own API key to unlock all models and bypass free-tier limits"

### 3. Memory/Engrams Viewer
- [ ] New route: `/settings/memory`
- [ ] List all engrams for wallet via `engram_search` tool
- [ ] Filter by type: persona, preference, knowledge, conversation
- [ ] Show pinned status (lock icon)
- [ ] Pin/unpin toggle via `hecate_pin_engram` / `hecate_unpin_engram`
- [ ] Delete button (respects pin protection - backend returns error if pinned)
- [ ] "Cleanup old sessions" button via `hecate_cleanup` tool
- [ ] Preview engram content in expandable card

### 4. Chat UI Enhancements
- [ ] Pass `wallet_address` in body when calling `/hecate/clear` (enables session save before wipe)
- [ ] Show tool call indicator when Hecate uses function calling (e.g., "Saving to memory..." toast)
- [ ] Display `supports_function_calling` badge on model selector (from model metadata)
- [ ] "Previous sessions" indicator when `load_recent_chat_context` found history

### 5. Model Selector Enhancements
- [ ] Show `supports_function_calling` as a filter/badge in model picker
- [ ] When user has own API keys, show "unlocked" models that require paid access
- [ ] Indicate current provider being used (openrouter vs anthropic vs user key)

## MCP Tool Reference (for frontend devs)

```bash
# Get profile
curl -X POST http://localhost:9003/mcp/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"user_profile_get","arguments":{"wallet_address":"WALLET"}}}'

# Update profile field
curl -X POST http://localhost:9003/mcp/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"user_profile_update","arguments":{"wallet_address":"WALLET","field":"base","content":"{\"display_name\":\"Name\",\"title\":\"Title\",\"bio\":\"Bio\"}"}}}'

# Search engrams
curl -X POST http://localhost:9003/mcp/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"engram_search","arguments":{"wallet_address":"WALLET","limit":20}}}'

# Pin engram
curl -X POST http://localhost:9003/mcp/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"hecate_pin_engram","arguments":{"id":"ENGRAM_ID"}}}'

# Cleanup old sessions
curl -X POST http://localhost:9003/mcp/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"hecate_cleanup","arguments":{"wallet_address":"WALLET"}}}'
```

## Priority Order

1. **API Key Management** - unblocks paid model access for users
2. **Chat UI Enhancements** - shows function calling is working
3. **Profile Settings** - users can edit what Hecate knows about them
4. **Memory Viewer** - power-user feature, pin/unpin/cleanup
5. **Model Selector** - nice-to-have badges and filters
