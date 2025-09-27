-- Migration: Update source_type to JSONB in Agents database
-- This migration changes source_type from VARCHAR to JSONB to match Erebus database

-- Add a temporary column for the new JSONB source_type
ALTER TABLE user_references ADD COLUMN source_type_new JSONB;

-- Migrate existing data from VARCHAR to JSONB format
UPDATE user_references 
SET source_type_new = jsonb_build_object(
    'type', COALESCE(source_type, 'web3_wallet'),
    'provider', COALESCE(wallet_type, 'unknown'),
    'metadata', additional_metadata
)
WHERE source_type_new IS NULL;

-- Drop the old VARCHAR column
ALTER TABLE user_references DROP COLUMN source_type;

-- Rename the new column to source_type
ALTER TABLE user_references RENAME COLUMN source_type_new TO source_type;

-- Update the default value for new records
ALTER TABLE user_references ALTER COLUMN source_type SET DEFAULT '{"type": "web3_wallet", "provider": "unknown", "metadata": {}}'::jsonb;

-- Create new indexes for the JSONB field
CREATE INDEX idx_user_references_source_type_gin ON user_references USING GIN (source_type);
CREATE INDEX idx_user_references_source_type_type ON user_references USING BTREE ((source_type->>'type'));

-- Update comments
COMMENT ON COLUMN user_references.source_type IS 'JSONB field containing source type information: {"type": "web3_wallet", "provider": "phantom", "metadata": {...}}';



