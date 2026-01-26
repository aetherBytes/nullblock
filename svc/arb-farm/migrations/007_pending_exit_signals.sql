-- Persist pending exit signals to survive service restarts
-- Critical for ensuring positions can always exit

CREATE TABLE IF NOT EXISTS pending_exit_signals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    position_id UUID NOT NULL REFERENCES arb_positions(id) ON DELETE CASCADE,
    reason VARCHAR(64) NOT NULL,
    exit_percent DECIMAL(10, 4) NOT NULL,
    current_price DECIMAL(28, 18) NOT NULL,
    triggered_at TIMESTAMPTZ NOT NULL,
    urgency VARCHAR(16) NOT NULL DEFAULT 'medium',
    failed_attempts INT NOT NULL DEFAULT 0,
    next_retry_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_rate_limited BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_position_exit UNIQUE (position_id)
);

CREATE INDEX IF NOT EXISTS idx_pending_exit_signals_next_retry
    ON pending_exit_signals(next_retry_at)
    WHERE failed_attempts < 10;

CREATE INDEX IF NOT EXISTS idx_pending_exit_signals_position
    ON pending_exit_signals(position_id);

COMMENT ON TABLE pending_exit_signals IS 'Persisted exit signals for recovery after service restart';
