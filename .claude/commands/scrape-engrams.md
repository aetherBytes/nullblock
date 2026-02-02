# Scrape Engrams Command

Fetch ArbFarm learning engrams and generate profit optimization recommendations based on LLM consensus suggestions, trade analyses, and pattern summaries.

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
- If they are trade analyses, examine root causes and suggested fixes
- Cross-reference with the codebase to suggest implementation

---

## Full Analysis Flow (No Arguments)

### 1. Fetch Learning Data

First, fetch all learning data from the ArbFarm service:

```bash
# Fetch the combined analysis summary (config + trade analyses + pattern summary)
curl -s "http://localhost:9007/consensus/analysis-summary"

# Fetch detailed trade analyses
curl -s "http://localhost:9007/consensus/trade-analyses?limit=50"

# Fetch pattern summary (losing/winning patterns + config recommendations)
curl -s "http://localhost:9007/consensus/patterns"

# Fetch LLM consensus recommendations (key data source!)
curl -s "http://localhost:9007/consensus/recommendations?limit=50"

# Fetch pending recommendations only
curl -s "http://localhost:9007/consensus/recommendations?status=pending&limit=50"

# Fetch learning engrams (recommendations, conversations)
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

# Get consensus scheduler status (check if analysis is running)
curl -s "http://localhost:9007/consensus/scheduler"

# Get PnL summary from positions DB (authoritative trade stats)
curl -s "http://localhost:9007/positions/pnl-summary"

# Get current risk config
curl -s "http://localhost:9007/config/risk"

# Get active strategies
curl -s "http://localhost:9007/strategies"
```

### 1b. Staleness Check — Trigger Fresh LLM Analysis If Needed

After fetching the data above, check if the consensus data is stale or empty. The analysis is stale if ANY of these conditions are true:

1. **Pattern summary `created_at` is older than 1 hour** — check `consensus/patterns` response `summary.created_at`
2. **Trade analyses count is 0** — check `consensus/trade-analyses` response `total == 0`
3. **Recommendations count is 0** — check `consensus/recommendations` response `total == 0`
4. **Trade count in PnL summary exceeds pattern summary's `trades_analyzed`** — significant new data exists that hasn't been analyzed

If stale, trigger a fresh LLM consensus analysis via the MCP tool:

```bash
# Trigger fresh LLM consensus analysis with recent trade data
# This queries multiple LLMs (Claude, GPT-4, Llama) to analyze recent trades
# and generate per-trade root causes, pattern summaries, and config recommendations
curl -s -X POST "http://localhost:9007/mcp/call" \
  -H "Content-Type: application/json" \
  -d '{"name":"engram_request_analysis","arguments":{"analysis_type":"trade_review","time_period":"24h"}}'
```

**IMPORTANT**: After triggering the analysis, wait 30-60 seconds for the LLMs to respond, then re-fetch the stale endpoints:

```bash
# Wait for LLM analysis to complete
sleep 45

# Re-fetch trade analyses (should now have data)
curl -s "http://localhost:9007/consensus/trade-analyses?limit=50"

# Re-fetch pattern summary (should be updated)
curl -s "http://localhost:9007/consensus/patterns"

# Re-fetch recommendations (should have new ones)
curl -s "http://localhost:9007/consensus/recommendations?limit=50"

# Re-fetch analysis summary
curl -s "http://localhost:9007/consensus/analysis-summary"
```

If the re-fetched data is STILL empty after triggering analysis, this likely means:
- The OpenRouter API key is not configured (`OPENROUTER_API_KEY` env var)
- All LLM models failed (check arb-farm logs for errors)
- The consensus engine is disabled

In this case, proceed with the analysis using ONLY the raw trade data from engrams and PnL summary. Note in the report that LLM consensus was unavailable and the analysis is based on raw trade metrics only.

Also check if the consensus scheduler is enabled. If it's paused, mention it in the report so the user can re-enable it:

```bash
# Check scheduler status
curl -s "http://localhost:9007/consensus/scheduler"
# Response: {"scheduler_enabled": true/false, "last_queried": "..."}

# Re-enable if paused
curl -s -X POST "http://localhost:9007/consensus/scheduler" \
  -H "Content-Type: application/json" \
  -d '{"enabled": true}'
```

### 2. Analyze the Data

**From Analysis Summary (`/consensus/analysis-summary`):**
- Review the current consensus config state (enabled, models, review interval)
- Note the total trade analyses count
- Check is_dev_wallet status for context

**From Trade Analyses (`/consensus/trade-analyses`):**
Each trade analysis contains LLM-identified root causes for trade outcomes:
- `position_id` - Links to the original trade
- `token_symbol` / `venue` - What was traded
- `pnl_sol` / `exit_reason` - Trade outcome
- `root_cause` - **KEY**: Why the trade succeeded/failed
- `config_issue` - **KEY**: Specific config problem (e.g., "SL at 5% triggered, but token recovered to +20%")
- `pattern` - Identified recurring pattern
- `suggested_fix` - **KEY**: LLM suggestion for improvement
- `confidence` - How confident the LLM is

**From Pattern Summary (`/consensus/patterns`):**
- `losing_patterns` - Common failure modes across trades
- `winning_patterns` - Success factors to reinforce
- `config_recommendations` - **KEY**: Aggregated config change suggestions
- `trades_analyzed` - How much data informed these patterns

**From Recommendations (`/consensus/recommendations`):**
Each recommendation contains actionable suggestions from LLM consensus:
- `recommendation_id` - Unique ID for referencing and status updates
- `source` - Origin (consensus_llm, pattern_analysis, risk_engine, manual)
- `category` - Type: strategy, risk, timing, venue, position
- `title` / `description` - Human-readable summary
- `suggested_action` - **KEY**: Contains:
  - `action_type` - config_change, strategy_toggle, risk_adjustment, venue_disable, avoid_token
  - `target` - What config/setting to change
  - `current_value` / `suggested_value` - The proposed change
  - `reasoning` - Why this change is recommended
- `confidence` - LLM confidence score (0.0 - 1.0)
- `supporting_data` - Trades analyzed, time period, relevant engrams
- `status` - pending, acknowledged, applied, rejected

**From Learning Engrams (arbFarm.recommendation tag):**
- Recommendations persisted with tags: `arbFarm.recommendation`, `category:{category}`, `status:{status}`
- Note recommendation confidence scores
- Identify patterns in suggested changes
- **IMPORTANT**: Note the `engram_id`, `engram_key`, and `tags` for each engram

**From Trade History:**
- Calculate win rate and total PnL
- Cross-reference with trade analyses for deeper insight
- **IMPORTANT**: Note the `engram_id` for trades you want to reference

**From Error History:**
- Categorize error types (RPC timeout, slippage, insufficient funds, etc.)
- Identify systemic issues requiring code changes
- Note frequency of recoverable vs fatal errors

### 3. Read Strategy Code Context

Read the actual implementation code to understand current strategy logic and risk parameters:

**Required Code Reads:**
```bash
# Read the strategy engine - understand entry/exit logic
Read svc/arb-farm/src/agents/strategy_engine.rs

# Read risk management - understand SL/TP calculations
Read svc/arb-farm/src/execution/risk.rs

# Read strategies documentation
Read docs-internal/src/arb-farm/strategies.md
```

**Extract from code:**
- Current stop-loss and take-profit percentages
- Position sizing logic
- Entry criteria and filters
- Exit strategy implementation

### 4. Fetch Web Research (if available)

Check for saved web research engrams that may contain external trading insights:

```bash
# Fetch saved web research engrams
curl -s -X POST "http://localhost:9007/mcp/call" \
  -H "Content-Type: application/json" \
  -d '{"name":"web_research_list","arguments":{"limit":10}}'
```

**Optionally search for new external insights:**
```bash
# Search for recent pump.fun trading strategies
curl -s -X POST "http://localhost:9007/mcp/call" \
  -H "Content-Type: application/json" \
  -d '{"name":"web_search","arguments":{"query":"solana pump.fun trading strategy 2026","num_results":3}}'

# Fetch and analyze a relevant result
curl -s -X POST "http://localhost:9007/mcp/call" \
  -H "Content-Type: application/json" \
  -d '{"name":"web_fetch","arguments":{"url":"<url from search>","extract_mode":"article"}}'

# Summarize with trading focus
curl -s -X POST "http://localhost:9007/mcp/call" \
  -H "Content-Type: application/json" \
  -d '{"name":"web_summarize","arguments":{"content":"<fetched content>","url":"<url>","focus":"strategy","save_as_engram":true}}'
```

### 5. Cross-Reference with Codebase

Read relevant configuration files to understand current settings:

**Key Files:**
- `svc/arb-farm/src/models/strategy.rs` - Strategy parameters
- `svc/arb-farm/src/execution/risk.rs` - Risk management settings
- `svc/arb-farm/src/consensus/config.rs` - Consensus configuration
- `svc/arb-farm/src/agents/autonomous_executor.rs` - Auto-execution logic

### 6. Generate Optimization Plan

Present findings in this format:

```
## ArbFarm Learning Analysis

### Data Summary
- Trades analyzed: X
- Trade analyses stored: X (from LLM consensus)
- Win rate: X% (from /positions/pnl-summary)
- Total PnL: X SOL
- Today PnL: X SOL
- Errors recorded: X
- Pending recommendations: X
- Web research engrams: X
- Pattern summary age: X minutes (stale if >1h)
- Consensus scheduler: enabled/paused
- LLM analysis triggered: yes/no (if data was stale)

### Engrams Retrieved

| Engram ID | Key | Tags | Type |
|-----------|-----|------|------|
| `abc-123` | arb.learning.recommendation.xyz | arbFarm.learning | Recommendation |
| `def-456` | arb.learning.trade_analysis.xyz | arbFarm.tradeAnalysis | Trade Analysis |
| `ghi-789` | arb.learning.pattern_summary.xyz | arbFarm.patternSummary | Pattern Summary |
| `jkl-012` | arb.research.web.xyz | arbFarm.webResearch | Web Research |

### Code Implementation Context

**Current Strategy Configuration (from code):**
```rust
// From svc/arb-farm/src/execution/risk.rs
stop_loss_percent: X%
take_profit_percent: X%
max_position_sol: X
```

**Exit Strategy Logic:**
[Summarize key logic from strategy_engine.rs]

### Trade Analysis Insights

**Per-Trade Root Causes (from LLM analysis):**

| Token | Venue | PnL | Exit | Root Cause | Config Issue |
|-------|-------|-----|------|------------|--------------|
| PUMP1 | pump.fun | -0.02 | StopLoss | SL too tight | SL 5% → suggest 8% |
| TOKEN2 | pump.fun | +0.05 | TakeProfit | Good entry timing | - |

**Suggested Fixes from Trade Analyses:**
1. [Token/Pattern]: [suggested_fix from analysis]
2. ...

### Pattern Summary (from `/consensus/patterns`)

**Losing Patterns:**
- [Pattern 1 from losing_patterns]
- [Pattern 2]

**Winning Patterns:**
- [Pattern 1 from winning_patterns]
- [Pattern 2]

**Config Recommendations (from pattern analysis):**
- [Recommendation 1 from config_recommendations]
- [Recommendation 2]

### External Research Insights

**From Saved Web Research:**
| Source | Focus | Key Insight | Confidence |
|--------|-------|-------------|------------|
| [URL] | Strategy | [insight] | X% |

**Relevant External Strategies:**
- [Strategy from web research with source]

### Top LLM Consensus Recommendations

1. [HIGH/MEDIUM/LOW CONFIDENCE: X.XX] Title
   - **Engram ID**: `uuid-here`
   - Current: [current value/behavior]
   - Suggested: [recommended change]
   - Reasoning: [from consensus]
   - File: [relevant file path]

2. ...

### Error Analysis

**Systemic Issues:**
- [Issue 1]: X occurrences - [suggested fix]
- [Issue 2]: X occurrences - [suggested fix]

### Implementation Plan

Based on the above analysis (trade data + patterns + web research + code context), here are recommended code changes:

1. **[Change Title]** (from trade analysis + pattern summary)
   - File: `path/to/file.rs`
   - Change: [description]
   - Code snippet showing current vs proposed:
     ```rust
     // Current:
     stop_loss_percent: 0.05

     // Proposed:
     stop_loss_percent: 0.08
     ```
   - Evidence: [cite specific trade analyses and patterns]
   - External support: [cite web research if applicable]
   - Expected Impact: [profit improvement estimate]
   - Related Engrams: `uuid1`, `uuid2`

2. ...
```

### 7. Offer to Implement

After presenting the analysis, ask if the user wants to:
- Implement the top recommendation
- Implement all recommendations
- Just acknowledge recommendations (mark as reviewed)
- Take no action
- **Focus on specific engrams**: Provide UUID(s) to analyze in detail
- **Search for more external insights**: Run web searches on specific topics

If implementing, use the Edit tool to make changes and run tests to verify.

### 8. Acknowledge Recommendations

After implementing or reviewing, acknowledge processed recommendations:

```bash
# Mark recommendation as applied or acknowledged
curl -s -X POST "http://localhost:9007/mcp/call" \
  -H "Content-Type: application/json" \
  -d '{"name":"engram_acknowledge_recommendation","arguments":{"recommendation_id":"UUID","status":"applied"}}'
```

## API Endpoints Reference

The following endpoints are available for learning analysis:

| Endpoint | Description |
|----------|-------------|
| `GET /consensus/trade-analyses?limit=N` | Per-trade LLM root cause analyses |
| `GET /consensus/patterns` | Pattern summary (losing/winning/config recs) — check `created_at` for staleness |
| `GET /consensus/analysis-summary` | Combined view with config + recent analyses |
| `GET /consensus/recommendations?status=&limit=` | LLM consensus recommendations (filter by status) |
| `PUT /consensus/recommendations/:id/status` | Update recommendation status |
| `GET /consensus/learning` | Learning summary stats |
| `GET /consensus/scheduler` | Scheduler status (enabled/paused, last_queried timestamp) |
| `POST /consensus/scheduler` | Toggle scheduler `{"enabled": true/false}` |
| `GET /positions/pnl-summary` | Authoritative PnL stats from DB (total_sol, wins, losses, recent_trades) |
| `GET /config/risk` | Current global risk config (SL, TP, position size, etc.) |
| `GET /strategies` | Active DB strategies with risk params |

**MCP Tools for Analysis:**

| Tool | Description |
|------|-------------|
| `engram_request_analysis` | Trigger fresh LLM consensus analysis — args: `{"analysis_type":"trade_review","time_period":"24h"}` |
| `engram_get_arbfarm_learning` | Fetch learning engrams (recommendations, conversations, patterns) |
| `engram_get_trade_history` | Get trade history with PnL from engrams |
| `engram_get_errors` | Get execution error history |
| `engram_acknowledge_recommendation` | Update recommendation status |

**Web Research MCP Tools:**

| Tool | Description |
|------|-------------|
| `web_search` | Search the web (requires SERPER_API_KEY) |
| `web_fetch` | Fetch and extract content from a URL |
| `web_summarize` | Summarize content with LLM, optionally save as engram |
| `web_research_list` | List saved web research engrams |

## Data Flow

```
/scrape-engrams invoked
        │
        ▼
┌─────────────────────────────────────────────┐
│ Step 1: Fetch all learning data             │
│ - /consensus/analysis-summary               │
│ - /consensus/trade-analyses                 │
│ - /consensus/patterns                       │
│ - /consensus/recommendations                │
│ - /positions/pnl-summary (authoritative)    │
│ - engram_get_trade_history (engrams)        │
│ - engram_get_errors                         │
└─────────────────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────────────────┐
│ Step 1b: Staleness Check                    │
│ Is pattern_summary > 1h old?                │
│ Are trade_analyses empty?                   │
│ Are recommendations empty?                  │
│ Are there more trades than analyzed?         │
│                                             │
│ IF STALE → trigger engram_request_analysis  │
│   via MCP tool (queries Claude/GPT-4/Llama) │
│   Wait ~45s, re-fetch updated data          │
└─────────────────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────────────────┐
│ Step 2-5: Analyze + cross-reference         │
│ - Trade analyses with root causes           │
│ - Pattern summaries                         │
│ - Code context (risk.rs, strategies.md)     │
│ - Recommendations with action items         │
└─────────────────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────────────────┐
│ Step 6: Generate optimization plan          │
│ - Implementation steps with file references │
│ - Evidence from trade data + LLM consensus  │
│ - Expected impact estimates                 │
└─────────────────────────────────────────────┘
```

**Background: Scheduled Analysis**
The consensus scheduler runs every 5 minutes (configurable). Each cycle it fetches the last 15 closed trades, builds an analysis prompt, queries multiple LLMs, and saves results as engrams. If the scheduler is paused or the OpenRouter API key is missing, no new analyses are generated — `/scrape-engrams` will detect this and trigger analysis on-demand.

## Notes

- This command requires arb-farm service running on port 9007
- Recommendations come from multi-LLM consensus (Claude, GPT-4, Llama via OpenRouter)
- Trade analyses now include **per-trade root causes** from LLM analysis
- Pattern summaries aggregate issues across trades for systemic improvements
- All changes should maintain the profit-maximization objective
- Test thoroughly before deploying to production wallet
- **Engram IDs** are always included so you can drill down with `/scrape-engrams uuid1,uuid2`
- **Staleness detection**: The skill automatically detects when consensus data is stale (>1h or empty) and triggers fresh LLM analysis via `engram_request_analysis`. This ensures you always get up-to-date insights even if the scheduler was paused or crashed.
- **Authoritative PnL source**: Use `/positions/pnl-summary` for accurate trade stats (total_sol, wins, losses, win_rate). The engrams trade history may lag behind the DB.
- **Consensus scheduler**: Runs every 5 minutes by default. Check `GET /consensus/scheduler` to see if it's enabled. If paused, the skill triggers analysis on-demand instead.
