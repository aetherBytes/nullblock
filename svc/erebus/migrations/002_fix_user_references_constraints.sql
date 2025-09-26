-- Migration: Fix user_references constraints in Erebus database
-- This migration fixes the constraint issues from the previous migration

-- Drop the old constraint first
ALTER TABLE user_references DROP CONSTRAINT user_references_wallet_address_chain_key;

-- Create the new unique constraint with the updated column name
CREATE UNIQUE INDEX user_references_source_chain_unique ON user_references(source_identifier, chain)
    WHERE source_identifier IS NOT NULL;
