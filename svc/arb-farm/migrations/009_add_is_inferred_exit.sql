-- Add is_inferred_exit column to track exits that don't have real on-chain signatures
-- These are exits where we detected tokens left wallet but didn't capture the actual tx

ALTER TABLE arb_positions ADD COLUMN IF NOT EXISTS is_inferred_exit BOOLEAN NOT NULL DEFAULT FALSE;
