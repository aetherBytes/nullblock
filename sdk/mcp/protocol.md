# Nullblock MCP Protocol — Reference Implementation v1.4

Nullblock implements the **MCP (Machine Composition Protocol)** as a *fully executable spec* — not a suggestion.

## Core Rules

1. **All interactions are HTTP POST to `/mcp/<skill>`**
2. **All inputs are JSON with schema validation**
3. **All outputs are JSON with `success: boolean`, `data: any`, `error: string?`**
4. **All agent identity is enforced via `X-Agent-ID`, `X-Signature`, `X-PubKey`**
5. **Secrets are NEVER code — always `op://` or AWS Secrets Manager**
6. **Every skill MUST have `SKILL.md` — and it MUST conform to the schema below**
7. **All services expose `GET /mcp/info`**
8. **All services expose `GET /mcp/discover` — returns list of all registered MCP endpoints**

## Schema: `SKILL.md`

```yaml
name: goplaces/search
version: 1.2
author: Sage
description: Search for places via Google Places API
inputs:
  query: string
  location: string
  radius: integer (optional, default=500)
outputs:
  places: array
    - name: string
      address: string
      rating: float
      photos: array of strings
dependencies:
  - 1password
  - http
signature_required: true
timeout: 5s
```

> 📌 All fields are **required** unless marked `(optional)`. 
> Types: `string`, `integer`, `float`, `boolean`, `array`, `object`
> Use `patsyanalysis` for complex nested types.

## Endpoints

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/mcp/info` | GET | Returns `name`, `version`, `author`, `schema`, `dependencies` |
| `/mcp/discover` | GET | Returns list of all available MCP endpoints (e.g., `["http://nullblock.local:9003/mcp", "https://clawhub.com/mcp/goplaces"]`) |
| `/mcp/skill/name` | POST | Invoke skill — JSON-RPC 2.0 body: `{ "method": "skill", "params": { ... }, "id": 1 }` |
| `/mcp/status` | GET | Returns `health: ok`, `uptime`, `memory`, `clients`, `requests_last_min` |

## Identity & Security

- Agent ID: SHA-256 hash of public key + machine context  
- Signature: Ed25519 signature of `method + params + nonce + timestamp`  
- Public Key: Stored in `~/.openclaw/keys/agent.pub`
- Validation: Any agent can validate incoming signature using `/mcp/info`'s `pubkey` field

## Mesh Behavior

- If a remote MCP responds with `404`, agent **caches negative result** for 1m.
- If a remote MCP fails 3x, agent **disables it** and logs to `mcp-failure.log`.
- If a remote MCP's `mcp/info` schema changes, agent **checks version** and logs warning: "Skill goplaces/v1.1 is outdated. New: v1.3"

## Examples

```bash
# Call from any agent
curl -X POST http://nullblock.local:9003/mcp/goplaces/search \
  -H "X-Agent-ID: 7a5f8b..." \
  -H "X-Signature: ..." \
  -H "X-PubKey: ..." \
  -d '{
    "method": "goplaces/search",
    "params": {
      "query": "coffee",
      "location": "Denver, CO"
    }
  }'
```