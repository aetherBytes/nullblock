-- ArbFarm: Core Tables Migration
-- Events, Venues, Strategies, Edges, Trades

-- Events (GOLDEN RULE - all agents emit here)
CREATE TABLE IF NOT EXISTS arb_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type TEXT NOT NULL,
    source_type TEXT NOT NULL,
    source_id TEXT NOT NULL,
    topic TEXT NOT NULL,
    payload JSONB NOT NULL,
    correlation_id UUID,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_events_topic ON arb_events(topic);
CREATE INDEX IF NOT EXISTS idx_events_source ON arb_events(source_type, source_id);
CREATE INDEX IF NOT EXISTS idx_events_created ON arb_events(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_events_correlation ON arb_events(correlation_id) WHERE correlation_id IS NOT NULL;

-- Topic subscriptions for agents
CREATE TABLE IF NOT EXISTS arb_event_subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subscriber_id TEXT NOT NULL,
    topics TEXT[] NOT NULL,
    is_active BOOLEAN DEFAULT true,
    last_event_id UUID,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Tracked venues
CREATE TABLE IF NOT EXISTS arb_venues (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    venue_type TEXT NOT NULL,
    name TEXT NOT NULL,
    address TEXT,
    config JSONB DEFAULT '{}',
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Trading strategies
CREATE TABLE IF NOT EXISTS arb_strategies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_address VARCHAR NOT NULL,
    name TEXT NOT NULL,
    strategy_type TEXT NOT NULL,
    venue_types TEXT[] NOT NULL,
    execution_mode TEXT NOT NULL,
    risk_params JSONB NOT NULL,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Detected edges (opportunities)
CREATE TABLE IF NOT EXISTS arb_edges (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    strategy_id UUID REFERENCES arb_strategies(id),
    edge_type TEXT NOT NULL,
    execution_mode TEXT NOT NULL,
    atomicity TEXT NOT NULL DEFAULT 'non_atomic',
    simulated_profit_guaranteed BOOLEAN DEFAULT false,
    simulation_tx_hash TEXT,
    max_gas_cost_lamports BIGINT,
    estimated_profit_lamports BIGINT,
    risk_score INTEGER,
    route_data JSONB NOT NULL,
    status TEXT DEFAULT 'detected',
    rejection_reason TEXT,
    executed_at TIMESTAMPTZ,
    actual_profit_lamports BIGINT,
    actual_gas_cost_lamports BIGINT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_edges_status ON arb_edges(status);
CREATE INDEX IF NOT EXISTS idx_edges_strategy ON arb_edges(strategy_id);
CREATE INDEX IF NOT EXISTS idx_edges_mode ON arb_edges(execution_mode);
CREATE INDEX IF NOT EXISTS idx_edges_atomicity ON arb_edges(atomicity);
CREATE INDEX IF NOT EXISTS idx_edges_guaranteed ON arb_edges(simulated_profit_guaranteed) WHERE simulated_profit_guaranteed = true;

-- Trade history
CREATE TABLE IF NOT EXISTS arb_trades (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    edge_id UUID REFERENCES arb_edges(id),
    strategy_id UUID REFERENCES arb_strategies(id),
    tx_signature TEXT,
    bundle_id TEXT,
    entry_price NUMERIC(20, 9),
    exit_price NUMERIC(20, 9),
    profit_lamports BIGINT,
    gas_cost_lamports BIGINT,
    slippage_bps INTEGER,
    executed_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_trades_edge ON arb_trades(edge_id);
CREATE INDEX IF NOT EXISTS idx_trades_strategy ON arb_trades(strategy_id);
CREATE INDEX IF NOT EXISTS idx_trades_executed ON arb_trades(executed_at DESC);

-- Consensus requests/results
CREATE TABLE IF NOT EXISTS arb_consensus (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    edge_id UUID REFERENCES arb_edges(id),
    models TEXT[] NOT NULL,
    model_votes JSONB NOT NULL,
    approved BOOLEAN,
    agreement_score NUMERIC(3, 2),
    reasoning_summary TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_consensus_edge ON arb_consensus(edge_id);
