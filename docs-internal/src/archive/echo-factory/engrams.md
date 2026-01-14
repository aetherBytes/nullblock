# Echo Factory Engram Integration

How Echo Factory uses the Engram service for persistent context.

## Engram Types Used

| Type | Key Pattern | Purpose |
|------|-------------|---------|
| `persona` | `echo.personas.{name}` | Twitter personas |
| `preference` | `echo.settings.{setting}` | User preferences |
| `knowledge` | `echo.content.successful` | High-engagement posts |
| `strategy` | `echo.strategies.{name}` | Content strategies |

## Persona Storage

When creating a persona, Echo Factory stores:

```json
{
  "wallet_address": "0x742d35Cc...",
  "engram_type": "persona",
  "key": "echo.personas.crypto_sage",
  "content": {
    "name": "Crypto Sage",
    "voice_style": "casual",
    "tone": {
      "professional": 0.7,
      "humor": 0.3
    },
    "topics": ["DeFi", "Monad", "AI Agents"],
    "hashtags": ["#DeFi", "#Monad"],
    "x_account_id": "123456789",
    "x_username": "crypto_sage"
  },
  "tags": ["echo", "persona", "twitter"]
}
```

## Content Learning

After successful posts, store as knowledge:

```json
{
  "engram_type": "knowledge",
  "key": "echo.content.successful",
  "content": {
    "posts": [
      {
        "text": "Just shipped a major update...",
        "engagement": {
          "likes": 150,
          "retweets": 42,
          "replies": 23
        },
        "timestamp": "2025-01-10T..."
      }
    ]
  },
  "tags": ["echo", "content", "high-engagement"]
}
```

## Cross-Session Context

Engrams enable:
- Persona persistence across sessions
- Learning from engagement patterns
- Consistent voice/tone
- Content strategy evolution

## Integration Flow

```
1. User creates persona → Store as engram
2. Content generation → Load persona engram
3. Siren generates → Apply voice/tone
4. Post succeeds → Store in knowledge engram
5. Future generation → Use knowledge for context
```

## Related

- [Echo Factory Plan](./plan.md)
- [Engrams Service](../services/engrams.md)
