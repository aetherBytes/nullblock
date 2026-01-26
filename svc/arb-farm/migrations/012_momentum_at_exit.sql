-- Add momentum_at_exit column to track momentum score when position was closed
ALTER TABLE arb_positions ADD COLUMN IF NOT EXISTS momentum_at_exit DECIMAL(10, 2);
