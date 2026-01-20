-- Add additional columns to arb_consensus table for enhanced tracking

-- Add weighted_confidence column
ALTER TABLE arb_consensus
ADD COLUMN IF NOT EXISTS weighted_confidence NUMERIC(5, 4);

-- Add edge_context column (stores the context string sent to LLMs)
ALTER TABLE arb_consensus
ADD COLUMN IF NOT EXISTS edge_context TEXT;

-- Add total_latency_ms column (total time for all model queries)
ALTER TABLE arb_consensus
ADD COLUMN IF NOT EXISTS total_latency_ms BIGINT;

-- Create index on created_at for faster time-based queries
CREATE INDEX IF NOT EXISTS idx_consensus_created ON arb_consensus(created_at DESC);

-- Create index on approved for filtering by decision
CREATE INDEX IF NOT EXISTS idx_consensus_approved ON arb_consensus(approved);
