-- Migration: Update user_references table schema
-- This migration updates the user_references table to use generic field names
-- and enhances the metadata structure for better source tracking

-- Rename wallet_address to source_identifier for consistency
ALTER TABLE user_references RENAME COLUMN wallet_address TO source_identifier;

-- Add new metadata fields for enhanced source tracking
ALTER TABLE user_references 
ADD COLUMN source_type VARCHAR DEFAULT 'web3_wallet', -- web3_wallet, email, api, system
ADD COLUMN wallet_type VARCHAR, -- phantom, metamask, coinbase, etc.
ADD COLUMN additional_metadata JSONB DEFAULT '{}'::jsonb; -- For extra source-specific data

-- Update the unique constraint to use the new column name
DROP INDEX IF EXISTS idx_user_references_wallet_chain_unique;
CREATE UNIQUE INDEX idx_user_references_source_chain_unique ON user_references(source_identifier, chain)
    WHERE source_identifier IS NOT NULL;

-- Update the wallet_address index to use the new column name
DROP INDEX IF EXISTS idx_user_references_wallet_address;
CREATE INDEX idx_user_references_source_identifier ON user_references(source_identifier);

-- Add new indexes for the enhanced fields
CREATE INDEX idx_user_references_source_type ON user_references(source_type);
CREATE INDEX idx_user_references_wallet_type ON user_references(wallet_type);
CREATE INDEX idx_user_references_additional_metadata ON user_references USING GIN (additional_metadata);

-- Update the metadata column to include the new structure
-- This will be populated by the sync functions
COMMENT ON COLUMN user_references.source_identifier IS 'Generic identifier for the user source (wallet address, email, API key, etc.)';
COMMENT ON COLUMN user_references.source_type IS 'Type of source: web3_wallet, email, api, system';
COMMENT ON COLUMN user_references.wallet_type IS 'Specific wallet type for web3 sources: phantom, metamask, coinbase, etc.';
COMMENT ON COLUMN user_references.additional_metadata IS 'Additional source-specific metadata like session info, IP addresses, etc.';

-- Example of the enhanced metadata structure:
-- {
--   "wallet_type": "phantom",
--   "chain": "solana",
--   "session_id": "session-uuid",
--   "ip_address": "192.168.1.1",
--   "user_agent": "Mozilla/5.0...",
--   "last_login": "2024-01-01T00:00:00Z",
--   "login_count": 42
-- }

