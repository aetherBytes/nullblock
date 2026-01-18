# ArbFarm Frontend Dashboard

The ArbFarm frontend is integrated into the Hecate dashboard as a submenu within the MemCache tab. It provides real-time monitoring and control of the Solana MEV agent swarm.

**Status**: ✅ Complete (Phase 11)

---

## Architecture Overview

```
svc/hecate/src/
├── types/arbfarm.ts                    # TypeScript type definitions
├── common/
│   ├── services/arbfarm-service.tsx    # API service class
│   └── hooks/useArbFarm.ts             # React data hook
└── components/
    ├── hud/VoidOverlay.tsx             # Navigation (includes ArbFarm menu item)
    └── memcache/arbfarm/
        ├── ArbFarmDashboard.tsx        # Main dashboard (7 views)
        ├── arbfarm.module.scss         # Styles (~1200 lines)
        └── components/                 # Subcomponents
            ├── EdgeCard.tsx            # Opportunity card
            ├── MetricCard.tsx          # P&L metric display
            ├── SwarmStatusCard.tsx     # Agent health
            ├── ThreatAlertCard.tsx     # Threat alerts
            └── TradeHistoryCard.tsx    # Trade history
```

---

## Quick Start

1. Navigate to `http://localhost:5173`
2. Connect your wallet (or simulate via localStorage)
3. Go to the **Mem Cache** tab
4. Click **ArbFarm** in the submenu

---

## Implementation Details

### Navigation Integration

ArbFarm is dynamically added to the MemCache submenu when the wallet has unlocked it (owns all ArbFarm tools). In development mode, it's always visible.

```typescript
// src/components/hud/VoidOverlay.tsx

// Base menu items (always visible)
const BASE_MEMCACHE_ITEMS = [
  { id: 'engrams', icon: '◈', label: 'Engrams' },
  { id: 'stash', icon: '⬡', label: 'Stash' },  // Tool inventory
  { id: 'tasks', icon: '▣', label: 'Tasks' },
  { id: 'agents', icon: '◉', label: 'Agents' },
  // ...
];

// NullBlock COW tabs are added dynamically when unlocked
const NULLBLOCK_SERVICE_COWS = [
  { id: 'arbfarm', name: 'ArbFarm', menuIcon: '⚡' },
  { id: 'polymev', name: 'PolyMev', menuIcon: '🎯' },
];
```

The tab unlock system is managed by `useWalletTools` hook and the `DEV_SHOW_ALL_COW_TABS` flag.

### Type System (`types/arbfarm.ts`)

Comprehensive TypeScript definitions (~500 lines) for all ArbFarm entities:

#### Core Types

```typescript
// Edge Status
export type EdgeStatus =
  | 'detected'
  | 'pending_approval'
  | 'executing'
  | 'executed'
  | 'expired'
  | 'failed'
  | 'rejected';

// Edge (Opportunity)
export interface Edge {
  id: string;
  strategy_id?: string;
  edge_type: EdgeType;
  venue_type: VenueType;
  execution_mode: ExecutionMode;
  atomicity: AtomicityLevel;
  simulated_profit_guaranteed: boolean;
  simulation_tx_hash?: string;
  max_gas_cost_lamports?: number;
  estimated_profit_lamports: number;
  risk_score: number;
  route_data: RouteData;
  status: EdgeStatus;
  rejection_reason?: string;
  executed_at?: string;
  actual_profit_lamports?: number;
  actual_gas_cost_lamports?: number;
  created_at: string;
  expires_at?: string;
}

// Trade
export interface Trade {
  id: string;
  edge_id: string;
  strategy_id?: string;
  tx_signature?: string;
  bundle_id?: string;
  entry_price?: number;
  exit_price?: number;
  profit_lamports: number;
  gas_cost_lamports: number;
  slippage_bps: number;
  executed_at: string;
}

// Strategy
export interface Strategy {
  id: string;
  wallet_address: string;
  name: string;
  strategy_type: StrategyType;
  venue_types: VenueType[];
  execution_mode: ExecutionMode;
  risk_params: RiskParams;
  is_active: boolean;
  stats?: StrategyStats;
  created_at: string;
  updated_at: string;
}

// Swarm Health
export interface SwarmHealth {
  total_agents: number;
  healthy_agents: number;
  degraded_agents: number;
  unhealthy_agents: number;
  dead_agents: number;
  overall_health: AgentHealth;
  is_paused: boolean;
  last_heartbeat?: string;
  agent_statuses: AgentStatus[];
}

// Threat Score
export interface ThreatScore {
  token_mint: string;
  overall_score: number;
  factors: ThreatFactors;
  confidence: number;
  external_data?: ExternalThreatData;
  recommendation: string;
  last_updated: string;
}
```

#### Color Constants

```typescript
export const EDGE_STATUS_COLORS: Record<EdgeStatus, string> = {
  detected: '#3b82f6',      // Blue
  pending_approval: '#f59e0b', // Amber
  executing: '#8b5cf6',     // Purple
  executed: '#22c55e',      // Green
  expired: '#6b7280',       // Gray
  failed: '#ef4444',        // Red
  rejected: '#f97316',      // Orange
};

export const AGENT_HEALTH_COLORS: Record<AgentHealth, string> = {
  Healthy: '#22c55e',
  Degraded: '#f59e0b',
  Unhealthy: '#f97316',
  Dead: '#ef4444',
};

export const PRIORITY_COLORS: Record<Priority, string> = {
  critical: '#ef4444',
  high: '#f59e0b',
  medium: '#3b82f6',
  low: '#6b7280',
};
```

---

### API Service (`arbfarm-service.tsx`)

Class-based service (~500 lines) following existing patterns:

```typescript
import { arbFarmService } from '../../common/services/arbfarm-service';

// Dashboard Summary
const summary = await arbFarmService.getDashboardSummary();

// Edges
const edges = await arbFarmService.listEdges({
  status: ['detected', 'pending_approval'],
  edge_type: ['dex_arb'],
  min_profit_lamports: 100000,
});
await arbFarmService.approveEdge(edgeId);
await arbFarmService.rejectEdge(edgeId, 'Too risky');
await arbFarmService.executeEdge(edgeId, { max_slippage_bps: 50 });

// Scanner
const status = await arbFarmService.getScannerStatus();
await arbFarmService.startScanner();
await arbFarmService.stopScanner();

// Swarm
const health = await arbFarmService.getSwarmStatus();
await arbFarmService.pauseSwarm();
await arbFarmService.resumeSwarm();

// Strategies
const strategies = await arbFarmService.listStrategies();
await arbFarmService.toggleStrategy(strategyId, true);

// Threats
const score = await arbFarmService.checkTokenThreat(tokenMint);
const alerts = await arbFarmService.getThreatAlerts();
const blocked = await arbFarmService.getBlockedEntities();

// KOL Tracking
const kols = await arbFarmService.listKOLs();
await arbFarmService.enableCopyTrading(kolId, config);

// Bonding Curves
const candidates = await arbFarmService.listGraduationCandidates();

// Engrams
const engrams = await arbFarmService.searchEngrams({ key_prefix: 'arb.' });

// SSE Streams
const streamUrl = arbFarmService.getEventsStreamUrl();
```

---

### React Hook (`useArbFarm.ts`)

Comprehensive hook (~900 lines) managing all ArbFarm state:

```typescript
import { useArbFarm } from '../../common/hooks/useArbFarm';

const {
  // Dashboard
  dashboard: {
    summary,           // DashboardSummary | null
    isLoading,         // boolean
    error,             // string | null
    refresh,           // () => Promise<void>
  },

  // Edges (Opportunities)
  edges: {
    data,              // Edge[]
    filter,            // EdgeFilter
    isLoading,         // boolean
    error,             // string | null
    setFilter,         // (filter: EdgeFilter) => void
    refresh,           // () => Promise<void>
    approve,           // (id: string) => Promise<void>
    reject,            // (id: string, reason: string) => Promise<void>
    execute,           // (id: string, params?: ExecuteParams) => Promise<void>
  },

  // Trades
  trades: {
    data,              // Trade[]
    stats,             // TradeStats | null
    dailyStats,        // DailyStats[]
    isLoading,         // boolean
    refresh,           // () => Promise<void>
    refreshStats,      // (period: string) => Promise<void>
  },

  // Scanner
  scanner: {
    status,            // ScannerStatus | null
    isLoading,         // boolean
    start,             // () => Promise<void>
    stop,              // () => Promise<void>
    refresh,           // () => Promise<void>
  },

  // Swarm
  swarm: {
    health,            // SwarmHealth | null
    isLoading,         // boolean
    pause,             // () => Promise<void>
    resume,            // () => Promise<void>
    refresh,           // () => Promise<void>
  },

  // Strategies
  strategies: {
    data,              // Strategy[]
    isLoading,         // boolean
    refresh,           // () => Promise<void>
    toggle,            // (id: string, enabled: boolean) => Promise<void>
    create,            // (params: CreateStrategyParams) => Promise<void>
    update,            // (id: string, params: UpdateStrategyParams) => Promise<void>
    delete,            // (id: string) => Promise<void>
  },

  // Threats
  threats: {
    alerts,            // ThreatAlert[]
    blocked,           // BlockedEntity[]
    isLoading,         // boolean
    refresh,           // () => Promise<void>
    checkToken,        // (mint: string) => Promise<ThreatScore>
    report,            // (params: ReportThreatParams) => Promise<void>
  },

  // KOL Tracking
  kols: {
    data,              // KOLEntity[]
    isLoading,         // boolean
    refresh,           // () => Promise<void>
    add,               // (params: AddKOLParams) => Promise<void>
    remove,            // (id: string) => Promise<void>
    enableCopy,        // (id: string, config: CopyConfig) => Promise<void>
    disableCopy,       // (id: string) => Promise<void>
  },

  // Bonding Curves
  curves: {
    tokens,            // CurveToken[]
    candidates,        // GraduationCandidate[]
    isLoading,         // boolean
    refresh,           // () => Promise<void>
  },

  // Engrams
  engrams: {
    data,              // Engram[]
    isLoading,         // boolean
    refresh,           // () => Promise<void>
    search,            // (query: string, limit?: number) => Promise<void>
  },

  // SSE (Server-Sent Events)
  sse: {
    isConnected,       // boolean
    lastEvent,         // ArbEvent | null
    connect,           // (topics: string[]) => void
    disconnect,        // () => void
  },
} = useArbFarm({
  pollInterval: 10000,        // Poll every 10 seconds
  autoFetchDashboard: true,   // Auto-fetch on mount
  autoFetchScanner: true,
  autoFetchSwarm: true,
});
```

---

### Dashboard Component (`ArbFarmDashboard.tsx`)

Main component (~560 lines) with 7 views:

```typescript
export type ArbFarmView =
  | 'dashboard'      // Main overview
  | 'opportunities'  // Live edge feed
  | 'strategies'     // Strategy management
  | 'threats'        // Threat detection
  | 'kol-tracker'    // KOL tracking (placeholder)
  | 'settings'       // Configuration (placeholder)
  | 'research';      // Research module (placeholder)
```

#### View Details

| View | Features | Status |
|------|----------|--------|
| **Dashboard** | P&L metrics, swarm health, top opportunities, recent trades, threat alerts, active strategies | ✅ Complete |
| **Opportunities** | Filter by status, approve/reject/execute actions, atomicity indicators | ✅ Complete |
| **Strategies** | Strategy list, creation modal, toggle enable/disable, risk params, execution mode | ✅ Complete |
| **Threats** | Token quick check, recent alerts, blocked entities | ✅ Complete |
| **KOL Tracker** | Wallet tracking, copy trading config | ⏳ Placeholder |
| **Settings** | Wallet, risk params, venues, API config, execution settings | ✅ Complete |
| **Research** | URL injection, discoveries, sources, consensus voting | ✅ Complete |
| **Signals** | Live signal feed, filtering by type/venue/profit | ✅ Complete |

---

### Subcomponents

#### MetricCard

Displays P&L and statistics with gradient borders:

```tsx
<MetricCard
  label="Total P&L"
  value="+1.2345 SOL"
  trend="up"
  trendValue="+0.5 this week"
  color="#22c55e"
  onClick={() => navigateToDetails()}
/>
```

#### EdgeCard

Displays opportunity details with actions:

```tsx
<EdgeCard
  edge={edge}
  onApprove={() => edges.approve(edge.id)}
  onReject={(reason) => edges.reject(edge.id, reason)}
  onExecute={() => edges.execute(edge.id)}
  compact={false}
/>
```

#### SwarmStatusCard

Shows agent health status:

```tsx
<SwarmStatusCard
  health={swarm.health}
  scannerStatus={scanner.status}
  isLoading={swarm.isLoading}
/>
```

#### ThreatAlertCard

Displays threat alerts with severity indicators:

```tsx
<ThreatAlertCard
  alert={alert}
  compact={true}
/>
```

#### TradeHistoryCard

Shows trade execution details:

```tsx
<TradeHistoryCard
  trade={trade}
  compact={true}
/>
```

---

### Research View

The Research view enables alpha discovery through URL injection and strategy extraction.

#### Tabs

| Tab | Description |
|-----|-------------|
| **URL Injection** | Submit URLs (Twitter, blogs, whitepapers) for LLM analysis |
| **Discoveries** | List of extracted strategies ready for review/activation |
| **Sources** | Tracked alpha sources and their success rates |
| **Consensus** | Multi-LLM consensus voting on strategy validity |

#### URL Injection Flow

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  Paste URL  │ ──▶ │ LLM Analyze │ ──▶ │  Extracted  │
│  + Analyze  │     │   Content   │     │  Strategy   │
└─────────────┘     └─────────────┘     └─────────────┘
                                              │
                    ┌─────────────────────────┼─────────────────────────┐
                    │                         │                         │
                    ▼                         ▼                         ▼
             ┌─────────────┐           ┌─────────────┐           ┌─────────────┐
             │   Create    │           │  Backtest   │           │   Reject    │
             │  Strategy   │           │    First    │           │             │
             └─────────────┘           └─────────────┘           └─────────────┘
```

#### Extracted Strategy Display

When a URL is analyzed, the following parameters are extracted:

| Parameter | Description |
|-----------|-------------|
| Strategy Type | `dex_arb`, `curve_graduation`, `liquidation`, `kol_copy`, `momentum` |
| Entry Conditions | Rules that trigger trade entry |
| Exit Conditions | Rules that trigger trade exit |
| Risk Parameters | Position size, slippage limits, loss limits |
| Confidence Score | LLM confidence in the extraction (0-1) |

---

### Signals View

Real-time signal monitoring with filtering capabilities.

#### Signal Card

```tsx
<div className={styles.signalCard}>
  <div className={styles.signalHeader}>
    <span className={styles.signalType}>{signal.signal_type}</span>
    <span className={styles.signalVenue}>{signal.venue_type}</span>
    <span className={styles.signalTime}>{formatTime(signal.detected_at)}</span>
  </div>
  <div className={styles.signalMetrics}>
    <div>Est. Profit: {signal.estimated_profit_bps} bps</div>
    <div>Confidence: {signal.confidence * 100}%</div>
  </div>
</div>
```

#### Signal Types

| Type | Source | Description |
|------|--------|-------------|
| `price_discrepancy` | DEX AMMs | Price difference between venues |
| `volume_spike` | All venues | Unusual volume activity |
| `dex_arb` | Jupiter/Raydium | Cross-DEX arbitrage opportunity |
| `liquidation` | Lending protocols | Liquidatable positions |
| `curve_graduation` | pump.fun/moonshot | Bonding curve graduation imminent |
| `kol_signal` | Wallet tracking | KOL wallet activity |

#### Filtering Options

- **Signal Type**: Filter by specific signal types
- **Venue Type**: Filter by DEX, lending protocol, etc.
- **Min Profit (bps)**: Minimum profit threshold
- **Min Confidence**: Minimum confidence threshold

---

### Strategy Creation Modal

Full strategy configuration form accessible from the Strategies view.

#### Form Fields

| Field | Type | Description |
|-------|------|-------------|
| Name | text | Strategy display name |
| Strategy Type | select | `dex_arb`, `curve_graduation`, `liquidation`, `kol_copy` |
| Venue Types | multiselect | `dex_amm`, `bonding_curve`, `lending` |
| Execution Mode | select | `autonomous`, `hybrid`, `agent_directed` |

#### Risk Parameters

| Parameter | Description | Default |
|-----------|-------------|---------|
| Max Position (SOL) | Maximum position size | 0.5 |
| Daily Loss Limit (SOL) | Max daily loss before pausing | 2.0 |
| Min Profit (bps) | Minimum profit to execute | 50 |
| Max Slippage (bps) | Maximum allowed slippage | 100 |

#### Execution Modes

| Mode | Behavior |
|------|----------|
| **Autonomous** | Auto-execute all qualifying opportunities |
| **Hybrid** | Auto below threshold, consensus above |
| **Agent Directed** | Multi-LLM consensus required for every trade |

---

### Settings View

Configuration tabs for all ArbFarm settings.

#### Tabs

| Tab | Settings |
|-----|----------|
| **Wallet** | Connected wallet address, SOL balance display |
| **Risk** | Global risk parameters |
| **Venues** | Enable/disable specific venues |
| **API** | API keys, RPC endpoints |
| **Execution** | Auto-execute toggle, thresholds |

#### Execution Settings

The execution tab provides fine-grained control over autonomous trading:

```tsx
// Master toggle
<div className={styles.masterToggle}>
  <span>Auto-Execute Mode</span>
  <input
    type="checkbox"
    checked={autoExecuteEnabled}
    onChange={handleToggle}
  />
</div>

// Thresholds
<div className={styles.thresholds}>
  <div>Min Confidence for Auto: {autoMinConfidence}%</div>
  <div>Max Position for Auto: {autoMaxPosition} SOL</div>
  <div>Require Simulation: {requireSimulation ? 'Yes' : 'No'}</div>
</div>
```

**Warning**: When auto-execute is enabled, the agent will execute trades without manual approval for opportunities meeting the threshold criteria.

---

### Styling (`arbfarm.module.scss`)

SCSS modules (~1200 lines) with consistent color scheme:

```scss
// Color palette
$color-success: #22c55e;
$color-warning: #f59e0b;
$color-danger: #ef4444;
$color-info: #3b82f6;
$color-purple: #8b5cf6;

// Gradient borders (MetricCard)
.metricCard {
  background: linear-gradient(135deg, rgba(255,255,255,0.05), transparent);
  border: 1px solid transparent;
  border-image: linear-gradient(135deg, rgba(255,255,255,0.2), transparent) 1;
}

// Status indicators
.priorityCritical { border-left: 3px solid $color-danger; }
.priorityHigh { border-left: 3px solid $color-warning; }
.priorityMedium { border-left: 3px solid $color-info; }
.priorityLow { border-left: 3px solid #6b7280; }

// Health badges
.healthBadge {
  &.healthy { background: rgba(34, 197, 94, 0.2); color: $color-success; }
  &.degraded { background: rgba(245, 158, 11, 0.2); color: $color-warning; }
  &.unhealthy { background: rgba(249, 115, 22, 0.2); color: #f97316; }
  &.dead { background: rgba(239, 68, 68, 0.2); color: $color-danger; }
}
```

---

## SSE Integration

Real-time updates via Server-Sent Events:

```typescript
// In ArbFarmDashboard.tsx
useEffect(() => {
  sse.connect(['arb.edge.*', 'arb.trade.*', 'arb.threat.*', 'arb.swarm.*']);
  return () => sse.disconnect();
}, []);

// Live indicator
{sse.isConnected && (
  <div className={styles.sseStatus}>
    <span className={styles.sseIndicator} />
    Live updates active
  </div>
)}
```

---

## Known Issues Fixed

### React Hooks Violation

**Problem**: `useState` was called inside render functions (`renderOpportunitiesView`, `renderThreatsView`) which violates React's rules of hooks.

**Solution**: Moved state to component top level:

```typescript
// Before (broken)
const renderOpportunitiesView = () => {
  const [filter, setFilter] = useState('all'); // WRONG: hook inside conditional
  // ...
};

// After (fixed)
const ArbFarmDashboard = () => {
  const [opportunitiesFilter, setOpportunitiesFilter] = useState('all'); // Correct
  const [threatTokenInput, setThreatTokenInput] = useState('');
  const [threatCheckResult, setThreatCheckResult] = useState(null);
  const [threatChecking, setThreatChecking] = useState(false);

  const renderOpportunitiesView = () => {
    // Use top-level state
  };
};
```

---

## Environment Variables

```bash
# .env
VITE_API_BASE_URL=http://localhost:3000  # Erebus router
```

Note: All ArbFarm API calls go through Erebus (port 3000), not directly to the service (port 9007).

---

## Development

```bash
# Start frontend
cd svc/hecate
npm run dev

# Build
npm run build

# Lint
npm run lint:format
```

---

## Related Documentation

- [ArbFarm Service](./service.md) - Backend service architecture
- [API Reference](./api.md) - REST API endpoints
- [Event Bus](./events.md) - SSE event topics
- [Implementation Plan](./plan.md) - Full roadmap
