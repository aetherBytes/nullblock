-- Add auto_exit_enabled flag to positions table
-- When false, position monitor will skip automatic exit signal generation for this position
ALTER TABLE arb_positions ADD COLUMN auto_exit_enabled BOOLEAN NOT NULL DEFAULT true;

-- Index for filtering positions by auto_exit status
CREATE INDEX idx_arb_positions_auto_exit ON arb_positions(auto_exit_enabled) WHERE status IN ('open', 'pending_exit', 'partially_exited');
