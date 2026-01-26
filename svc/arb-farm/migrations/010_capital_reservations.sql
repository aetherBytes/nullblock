-- Capital reservations persistence table
-- Ensures capital allocations survive restarts

CREATE TABLE IF NOT EXISTS capital_reservations (
    position_id UUID PRIMARY KEY,
    strategy_id UUID NOT NULL,
    amount_lamports BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_capital_reservations_strategy ON capital_reservations(strategy_id);
