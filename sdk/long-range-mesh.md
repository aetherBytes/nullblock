# Nullblock Long-Range Mesh — Beyond Local Agents

Nullblock is not a monolith.

It is the **first anchor node** in a growing mesh of autonomous agents — composed of:

- 🟢 Local agents (`nullblock-agents`, `erebus`)
- 🟡 Remote ClawHub skills (`clawhub.com/mcp/goplaces`)
- 🔵 Peer nodes (future: `crossroads-orchestrator.local`, `helius-mcp-server.eth`)
- 🟣 External cloaked agents (wallet trackers, price oracles, legal bots)

## Mesh Topology Strategy

| Layer | Type | Discoverable? | Ingress | Egress |
|-------|------|----------------|---------|--------|
| **Local** | Built-in Nullblock services | ✅ `mcp/discover` | ✅ | ✅ |
| **ClawHub** | Global skill registry | ✅ via `clawhub.com/api/skills` | ✅ | ✅ |
| **Crossroads** | Orchestration layer | ✅ exposed as LDAP-like agent directory | ✅ | ✅ |
| **External** | Public MCPs (e.g., Solana price feed) | ✅ wildcard discovery via DNS-SD (未來) | ✅ | ✅ |

## Key Innovation: Recursive Discovery

A nullblock agent can:

1. Query its local `mcp/discover` → finds `clawhub.com/mcp/goplaces`
2. Call it → gets skill metadata
3. Now calls `http://clawhub.com/mcp/discover` → finds *their* external services
4. Now knows: _all_ skills connected via ClawHub are reachable via path

> 💡 This is **recursive federation**. 
> No central directory. No admin. Just **trustless discovery**.

## Security Model: Untrusted Mesh

- All remote skills are treated as **untrusted input**.
- Input validation is enforced by **local schema checker** (NONE dispatches to external).
- Results are cached + verified — no trust propagation.
- All signatures must be produced by **a known agent ID** in the local keyring.

## Next-Gen: Agent DNS-SD (Predictive)

In v2, we will implement:

```bash
# Resolve MCP service
ping goplaces.mcp.nullblock.local

# Auto-discover
dns-sd -B _mcp._tcp local.
```

Nullblock will pioneer **MCP over ZeroConf** — making agent networks as easy as printing.