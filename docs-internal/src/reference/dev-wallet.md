# Dev Wallet Overrides

The dev wallet (`5wrmi85pTPmB4NDv7rUYncEMi1KqVo93bZn3XtXSbjYT`) bypasses free-tier limitations and receives premium features.

**Config Location**: `svc/nullblock-agents/src/config/dev_wallet.rs`

## Model Selection

| Setting | Free Tier | Dev Wallet |
|---------|-----------|------------|
| Default Model | `cognitivecomputations/dolphin3.0-mistral-24b:free` | `anthropic/claude-sonnet-4` |
| Model Validation | Must use `:free` suffix models | Any model allowed |
| Model Selection | Restricted to free models | Unrestricted |

**Available Premium Models** (dev wallet):
- `anthropic/claude-sonnet-4` (default)
- `anthropic/claude-3.5-sonnet`
- `openai/gpt-4-turbo`
- `meta-llama/llama-3.1-405b-instruct`

## Token Limits

| Limit | Free Tier | Dev Wallet |
|-------|-----------|------------|
| Max Input | 8,000 chars (~2,000 tokens) | Unlimited |
| Max Output | 1,500 tokens | 4,096 tokens |
| Image Responses | 16,384 tokens | 16,384 tokens |

## Bypassed Checks

The dev wallet bypasses:

1. **Input length validation** - No character limit on messages
2. **Output token limits** - Higher response length allowed
3. **Model tier validation** - Can use paid models without API keys
4. **Free-tier model enforcement** - No `:free` suffix requirement

## Seeded Engrams

On service startup, the dev wallet receives persona engrams:

| Engram Key | Agent | Purpose |
|------------|-------|---------|
| `hecate.persona.architect` | HECATE | Recognizes user as "The Architect" |
| `moros.persona.architect` | MOROS | Recognizes user as "The Architect" |

These engrams establish the dev wallet user's identity as "Sage the Architect" in agent memory.

## Code References

| Override | File | Line |
|----------|------|------|
| Model override (Hecate) | `agents/hecate.rs` | 604-609 |
| Model override (Moros) | `agents/moros.rs` | 597-604 |
| Max tokens (Hecate) | `agents/hecate.rs` | 618-620 |
| Max tokens (Moros) | `agents/moros.rs` | 608-609 |
| Input length bypass | `handlers/hecate.rs` | 115 |
| Output limit bypass | `handlers/hecate.rs` | 122-126 |
| Model validation bypass | `handlers/hecate.rs` | 152, 980 |
| Persona engram seeding | `server.rs` | 196-255 |

## Adding Dev Wallets

To add additional dev wallets, edit `DEV_WALLETS` in `config/dev_wallet.rs`:

```rust
pub const DEV_WALLETS: &[&str] = &[
    "5wrmi85pTPmB4NDv7rUYncEMi1KqVo93bZn3XtXSbjYT",
    // Add new wallets here
];
```

Requires service restart to take effect.
