-- Positions table for persistence across restarts

CREATE TABLE IF NOT EXISTS arb_positions (
    id UUID PRIMARY KEY,
    edge_id UUID NOT NULL,
    strategy_id UUID NOT NULL,
    token_mint VARCHAR(64) NOT NULL,
    token_symbol VARCHAR(32),

    -- Entry data
    entry_amount_base DECIMAL(20, 9) NOT NULL,
    entry_token_amount DECIMAL(30, 0) NOT NULL,
    entry_price DECIMAL(30, 18) NOT NULL,
    entry_time TIMESTAMPTZ NOT NULL,
    entry_tx_signature VARCHAR(128),

    -- Current state
    current_price DECIMAL(30, 18) NOT NULL,
    current_value_base DECIMAL(20, 9) NOT NULL,
    unrealized_pnl DECIMAL(20, 9) NOT NULL DEFAULT 0,
    unrealized_pnl_percent DECIMAL(10, 4) NOT NULL DEFAULT 0,
    high_water_mark DECIMAL(30, 18) NOT NULL,

    -- Exit configuration (JSON)
    exit_config JSONB NOT NULL DEFAULT '{}',

    -- Partial exits (JSON array)
    partial_exits JSONB NOT NULL DEFAULT '[]',

    -- Status
    status VARCHAR(32) NOT NULL DEFAULT 'open',

    -- Exit data (filled when closed)
    exit_price DECIMAL(30, 18),
    exit_time TIMESTAMPTZ,
    exit_tx_signature VARCHAR(128),
    realized_pnl DECIMAL(20, 9),
    exit_reason VARCHAR(64),

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_arb_positions_status ON arb_positions(status);
CREATE INDEX IF NOT EXISTS idx_arb_positions_token_mint ON arb_positions(token_mint);
CREATE INDEX IF NOT EXISTS idx_arb_positions_strategy_id ON arb_positions(strategy_id);
CREATE INDEX IF NOT EXISTS idx_arb_positions_edge_id ON arb_positions(edge_id);
CREATE INDEX IF NOT EXISTS idx_arb_positions_entry_time ON arb_positions(entry_time);

-- Trigger for updated_at
CREATE OR REPLACE FUNCTION update_arb_positions_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trigger_arb_positions_updated_at ON arb_positions;
CREATE TRIGGER trigger_arb_positions_updated_at
    BEFORE UPDATE ON arb_positions
    FOR EACH ROW
    EXECUTE FUNCTION update_arb_positions_updated_at();
