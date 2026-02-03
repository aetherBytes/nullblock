# Moros Agent

**Second agent in the NullBlock agent system.**

Moros is a parallel agent to Hecate, sharing the same MCP tool infrastructure but with its own persona, conversation state, and engram namespace.

## Configuration

Moros uses the same LLM factory and provider infrastructure as Hecate. It has its own set of MCP tools namespaced with `moros_` prefix.

## MCP Tools

| Tool | Description |
|------|-------------|
| `moros_remember` | Save context to memory (tagged `moros`, `auto`) |
| `moros_cleanup` | Compact old conversation sessions |
| `moros_pin_engram` | Pin engram (protect from deletion) |
| `moros_unpin_engram` | Remove pin protection |

## Related

- [HECATE Agent](./hecate.md)
- [Agent Overview](./overview.md)
- [Engrams Service](../services/engrams.md)
