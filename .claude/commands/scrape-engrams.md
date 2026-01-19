# Scrape Engrams Command

Fetch ArbFarm learning engrams and generate profit optimization recommendations based on LLM consensus suggestions and trade history.

**Arguments:** `$ARGUMENTS` (optional - comma-separated list of engram UUIDs to focus on)

## Instructions

When this command is invoked, check if any arguments were provided:

### Mode 1: No Arguments (Default Full Analysis)

When invoked without arguments (`/scrape-engrams`), perform a comprehensive analysis of all learning data.

### Mode 2: Specific Engram UUIDs

When invoked with UUIDs (`/scrape-engrams uuid1,uuid2,uuid3`), focus analysis ONLY on those specific engrams:

```bash
# Fetch specific engrams by their UUIDs
curl -s -X POST "http://localhost:9007/mcp/call" \
  -H "Content-Type: application/json" \
  -d '{"name":"engram_get_by_ids","arguments":{"engram_ids":["UUID1","UUID2","UUID3"]}}'
```

Replace the UUID placeholders with the actual UUIDs provided in `$ARGUMENTS` (split by comma).

When focusing on specific engrams:
- Analyze ONLY the content of those specific engrams
- Provide detailed analysis of each engram's content
- If they are recommendations, analyze the suggested changes in depth
- If they are trade summaries, analyze the patterns in those specific trades
- Cross-reference with the codebase to suggest implementation

---

## Full Analysis Flow (No Arguments)

### 1. Fetch Learning Engrams

First, fetch all learning data from the ArbFarm service:

```bash
# Fetch learning engrams (recommendations, conversations, patterns)
curl -s -X POST "http://localhost:9007/mcp/call" \
  -H "Content-Type: application/json" \
  -d '{"name":"engram_get_arbfarm_learning","arguments":{"category":"all","limit":50}}'

# Fetch recent trade history
curl -s -X POST "http://localhost:9007/mcp/call" \
  -H "Content-Type: application/json" \
  -d '{"name":"engram_get_trade_history","arguments":{"limit":50}}'

# Fetch error history
curl -s -X POST "http://localhost:9007/mcp/call" \
  -H "Content-Type: application/json" \
  -d '{"name":"engram_get_errors","arguments":{"limit":20}}'

# Get consensus learning summary
curl -s "http://localhost:9007/consensus/learning"
```

### 2. Analyze the Data

**From Learning Engrams (arbFarm.learning tag):**
- Extract pending recommendations from LLM consensus
- Note recommendation confidence scores
- Identify patterns in suggested changes
- **IMPORTANT**: Note the `engram_id`, `engram_key`, and `tags` for each engram

**From Trade History:**
- Calculate win rate and total PnL
- Identify winning patterns (venue, token characteristics, timing)
- Identify losing patterns to avoid
- **IMPORTANT**: Note the `engram_id` for trades you want to reference

**From Error History:**
- Categorize error types (RPC timeout, slippage, insufficient funds, etc.)
- Identify systemic issues requiring code changes
- Note frequency of recoverable vs fatal errors

### 3. Cross-Reference with Codebase

Read relevant configuration files to understand current settings:

**Key Files:**
- `svc/arb-farm/src/models/strategy.rs` - Strategy parameters
- `svc/arb-farm/src/execution/risk.rs` - Risk management settings
- `svc/arb-farm/src/consensus/config.rs` - Consensus configuration
- `svc/arb-farm/src/agents/autonomous_executor.rs` - Auto-execution logic

### 4. Generate Optimization Plan

Present findings in this format:

```
## ArbFarm Learning Analysis

### Data Summary
- Trades analyzed: X
- Win rate: X%
- Total PnL: X SOL
- Errors recorded: X
- Pending recommendations: X

### Engrams Retrieved

| Engram ID | Key | Tags | Type |
|-----------|-----|------|------|
| `abc-123` | arb.learning.recommendation.xyz | arbFarm.learning | Recommendation |
| `def-456` | arb.trade.summary.xyz | arbFarm.trade | Trade Summary |

### Top LLM Consensus Recommendations

1. [HIGH/MEDIUM/LOW CONFIDENCE: X.XX] Title
   - **Engram ID**: `uuid-here`
   - Current: [current value/behavior]
   - Suggested: [recommended change]
   - Reasoning: [from consensus]
   - File: [relevant file path]

2. ...

### Patterns from Trade History

**Winning Patterns:**
- [Pattern 1] (Engram IDs: `uuid1`, `uuid2`)
- [Pattern 2]

**Losing Patterns to Avoid:**
- [Pattern 1]
- [Pattern 2]

### Error Analysis

**Systemic Issues:**
- [Issue 1]: X occurrences - [suggested fix]
- [Issue 2]: X occurrences - [suggested fix]

### Implementation Plan

Based on the above analysis, here are recommended code changes:

1. **[Change Title]**
   - File: `path/to/file.rs`
   - Change: [description]
   - Expected Impact: [profit improvement estimate]
   - Related Engrams: `uuid1`, `uuid2`

2. ...
```

### 5. Offer to Implement

After presenting the analysis, ask if the user wants to:
- Implement the top recommendation
- Implement all recommendations
- Just acknowledge recommendations (mark as reviewed)
- Take no action
- **Focus on specific engrams**: Provide UUID(s) to analyze in detail

If implementing, use the Edit tool to make changes and run tests to verify.

### 6. Acknowledge Recommendations

After implementing or reviewing, acknowledge processed recommendations:

```bash
# Mark recommendation as applied or acknowledged
curl -s -X POST "http://localhost:9007/mcp/call" \
  -H "Content-Type: application/json" \
  -d '{"name":"engram_acknowledge_recommendation","arguments":{"recommendation_id":"UUID","status":"applied"}}'
```

## Notes

- This command requires arb-farm service running on port 9007
- Recommendations come from multi-LLM consensus (Claude, GPT-4, Llama)
- All changes should maintain the profit-maximization objective
- Test thoroughly before deploying to production wallet
- **Engram IDs** are always included so you can drill down with `/scrape-engrams uuid1,uuid2`
