-- Migration: Create user_references table
-- READ-ONLY sync cache from Erebus users table
-- Populated via Kafka events, used for task foreign key validation

CREATE TABLE user_references (
    id UUID PRIMARY KEY, -- mirrors Erebus users.id exactly
    wallet_address VARCHAR,
    chain VARCHAR,
    user_type VARCHAR, -- web3, gmail, etc (agnostic for future expansion)
    email VARCHAR,     -- for gmail users, null for web3-only

    -- User metadata and preferences
    metadata JSONB DEFAULT '{}'::jsonb,
    preferences JSONB DEFAULT '{}'::jsonb,

    -- Sync tracking
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_active BOOLEAN DEFAULT true,

    -- Original timestamps from Erebus (for reference)
    erebus_created_at TIMESTAMPTZ,
    erebus_updated_at TIMESTAMPTZ
);

-- Indexes for performance and lookups
CREATE INDEX idx_user_references_wallet_address ON user_references(wallet_address);
CREATE INDEX idx_user_references_chain ON user_references(chain);
CREATE INDEX idx_user_references_user_type ON user_references(user_type);
CREATE INDEX idx_user_references_email ON user_references(email);
CREATE INDEX idx_user_references_is_active ON user_references(is_active);
CREATE INDEX idx_user_references_synced_at ON user_references(synced_at);

-- Unique constraints
CREATE UNIQUE INDEX idx_user_references_wallet_chain_unique ON user_references(wallet_address, chain)
    WHERE wallet_address IS NOT NULL;
CREATE UNIQUE INDEX idx_user_references_email_unique ON user_references(email)
    WHERE email IS NOT NULL;