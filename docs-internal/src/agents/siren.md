# Siren Marketing Agent

**Content generation specialist** for social media and marketing.

## Overview

| Property | Value |
|----------|-------|
| **Location** | `/svc/nullblock-agents/src/agents/siren_marketing.rs` |
| **Purpose** | Marketing content, social posts |

## Capabilities

- Twitter/X post creation
- Project progress analysis
- Marketing content themes
- Content style adaptation

## API Endpoints

```bash
# Chat with Siren
POST /siren/chat
{
  "message": "Write a tweet about our new feature",
  "context": {
    "project": "NullBlock",
    "tone": "professional"
  }
}
```

## Echo Factory Integration

Siren will power content generation in Echo Factory:

1. Receives persona context from Engrams
2. Generates content matching voice/style
3. Outputs ready for scheduling/publishing

## Known Issues

- Should follow user's model selection (currently uses default)
- Fix in progress

## Related

- [Agent Overview](./overview.md)
- [Echo Factory Plan](../echo-factory/plan.md)
