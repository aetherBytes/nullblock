-- Engram Service Database Schema
-- Universal memory/context persistence layer for NullBlock

-- Enable UUID extension if not already enabled
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Main engrams table
CREATE TABLE IF NOT EXISTS engrams (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    -- Wallet-centric ownership (primary scope)
    wallet_address VARCHAR NOT NULL,

    -- Classification
    engram_type VARCHAR NOT NULL CHECK (engram_type IN ('persona', 'preference', 'strategy', 'knowledge', 'compliance')),
    key VARCHAR NOT NULL,
    tags TEXT[] DEFAULT '{}',

    -- Content
    content JSONB NOT NULL,
    summary TEXT,

    -- Versioning & Lineage
    version INTEGER DEFAULT 1,
    parent_id UUID REFERENCES engrams(id) ON DELETE SET NULL,
    lineage_root_id UUID,

    -- Visibility & Monetization (Monad chain)
    is_public BOOLEAN DEFAULT false,
    is_mintable BOOLEAN DEFAULT false,
    nft_token_id VARCHAR,
    price_mon DECIMAL(18, 8),
    royalty_percent INTEGER DEFAULT 5 CHECK (royalty_percent >= 0 AND royalty_percent <= 100),

    -- Metadata
    priority INTEGER DEFAULT 0,
    ttl_seconds INTEGER,
    created_by VARCHAR,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    accessed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Unique constraint: one key per wallet per version
    UNIQUE (wallet_address, key, version)
);

-- Engram history for audit trail
CREATE TABLE IF NOT EXISTS engram_history (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    engram_id UUID NOT NULL REFERENCES engrams(id) ON DELETE CASCADE,
    version INTEGER NOT NULL,
    content JSONB NOT NULL,
    changed_by VARCHAR,
    change_reason VARCHAR,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for common queries (IF NOT EXISTS)
CREATE INDEX IF NOT EXISTS idx_engrams_wallet ON engrams(wallet_address);
CREATE INDEX IF NOT EXISTS idx_engrams_type ON engrams(engram_type);
CREATE INDEX IF NOT EXISTS idx_engrams_key ON engrams(key);
CREATE INDEX IF NOT EXISTS idx_engrams_tags ON engrams USING GIN(tags);
CREATE INDEX IF NOT EXISTS idx_engrams_public ON engrams(is_public) WHERE is_public = true;
CREATE INDEX IF NOT EXISTS idx_engrams_lineage ON engrams(lineage_root_id);
CREATE INDEX IF NOT EXISTS idx_engrams_updated ON engrams(updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_engrams_priority ON engrams(priority DESC);

-- History indexes
CREATE INDEX IF NOT EXISTS idx_engram_history_engram_id ON engram_history(engram_id);
CREATE INDEX IF NOT EXISTS idx_engram_history_version ON engram_history(engram_id, version DESC);
