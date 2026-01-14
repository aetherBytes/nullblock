-- Crossroads Marketplace Schema
-- Phase 12: ArbFarm COW Integration

-- =============================================================================
-- Core Marketplace Tables
-- =============================================================================

-- Main listings table for all marketplace items
CREATE TABLE IF NOT EXISTS crossroads_listings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    listing_type VARCHAR(50) NOT NULL,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    author VARCHAR(255) NOT NULL,
    author_wallet VARCHAR(64),
    version VARCHAR(50) DEFAULT '1.0.0',
    tags TEXT[] DEFAULT '{}',
    status VARCHAR(50) DEFAULT 'pending',
    price_lamports BIGINT DEFAULT 0,
    is_free BOOLEAN DEFAULT true,
    rating DECIMAL(3,2),
    total_ratings INTEGER DEFAULT 0,
    download_count BIGINT DEFAULT 0,
    view_count BIGINT DEFAULT 0,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_listings_type ON crossroads_listings(listing_type);
CREATE INDEX idx_listings_status ON crossroads_listings(status);
CREATE INDEX idx_listings_author ON crossroads_listings(author_wallet);
CREATE INDEX idx_listings_created ON crossroads_listings(created_at DESC);
CREATE INDEX idx_listings_rating ON crossroads_listings(rating DESC NULLS LAST);
CREATE INDEX idx_listings_tags ON crossroads_listings USING GIN(tags);

-- Listing versions for update history
CREATE TABLE IF NOT EXISTS crossroads_listing_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    listing_id UUID NOT NULL REFERENCES crossroads_listings(id) ON DELETE CASCADE,
    version VARCHAR(50) NOT NULL,
    changelog TEXT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_listing_versions_listing ON crossroads_listing_versions(listing_id);

-- Reviews and ratings
CREATE TABLE IF NOT EXISTS crossroads_reviews (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    listing_id UUID NOT NULL REFERENCES crossroads_listings(id) ON DELETE CASCADE,
    reviewer_wallet VARCHAR(64) NOT NULL,
    rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
    review_text TEXT,
    is_verified_purchase BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(listing_id, reviewer_wallet)
);

CREATE INDEX idx_reviews_listing ON crossroads_reviews(listing_id);
CREATE INDEX idx_reviews_reviewer ON crossroads_reviews(reviewer_wallet);

-- =============================================================================
-- ArbFarm COW (Constellation of Work) Tables
-- =============================================================================

-- Main ArbFarm COW table
CREATE TABLE IF NOT EXISTS arbfarm_cows (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    listing_id UUID UNIQUE NOT NULL REFERENCES crossroads_listings(id) ON DELETE CASCADE,
    creator_wallet VARCHAR(64) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,

    -- Fork tracking
    parent_cow_id UUID REFERENCES arbfarm_cows(id) ON DELETE SET NULL,
    fork_count INTEGER DEFAULT 0,
    fork_depth INTEGER DEFAULT 0,

    -- Performance metrics
    total_profit_generated_lamports BIGINT DEFAULT 0,
    total_trades INTEGER DEFAULT 0,
    successful_trades INTEGER DEFAULT 0,
    win_rate DECIMAL(5,4) DEFAULT 0,

    -- Revenue configuration
    creator_revenue_share_bps INTEGER DEFAULT 500,
    fork_revenue_share_bps INTEGER DEFAULT 200,

    -- Visibility
    is_public BOOLEAN DEFAULT true,
    is_forkable BOOLEAN DEFAULT true,

    -- Risk profile (stored as JSONB)
    risk_profile JSONB NOT NULL DEFAULT '{}',

    -- Inherited engrams from parent
    inherited_engrams TEXT[] DEFAULT '{}',

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_arbfarm_cows_listing ON arbfarm_cows(listing_id);
CREATE INDEX idx_arbfarm_cows_creator ON arbfarm_cows(creator_wallet);
CREATE INDEX idx_arbfarm_cows_parent ON arbfarm_cows(parent_cow_id);
CREATE INDEX idx_arbfarm_cows_public ON arbfarm_cows(is_public) WHERE is_public = true;
CREATE INDEX idx_arbfarm_cows_forkable ON arbfarm_cows(is_forkable) WHERE is_forkable = true;
CREATE INDEX idx_arbfarm_cows_profit ON arbfarm_cows(total_profit_generated_lamports DESC);

-- ArbFarm strategies within a COW
CREATE TABLE IF NOT EXISTS arbfarm_cow_strategies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    cow_id UUID NOT NULL REFERENCES arbfarm_cows(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    strategy_type VARCHAR(50) NOT NULL,
    venue_types TEXT[] DEFAULT '{}',
    execution_mode VARCHAR(50) DEFAULT 'hybrid',
    risk_params JSONB NOT NULL DEFAULT '{}',
    is_active BOOLEAN DEFAULT true,

    -- Performance
    total_trades INTEGER DEFAULT 0,
    successful_trades INTEGER DEFAULT 0,
    win_rate DECIMAL(5,4) DEFAULT 0,
    total_profit_lamports BIGINT DEFAULT 0,
    avg_profit_per_trade_lamports BIGINT DEFAULT 0,
    max_drawdown_lamports BIGINT DEFAULT 0,
    sharpe_ratio DECIMAL(6,4),
    last_trade_at TIMESTAMPTZ,

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_cow_strategies_cow ON arbfarm_cow_strategies(cow_id);
CREATE INDEX idx_cow_strategies_type ON arbfarm_cow_strategies(strategy_type);
CREATE INDEX idx_cow_strategies_active ON arbfarm_cow_strategies(is_active) WHERE is_active = true;

-- Fork history
CREATE TABLE IF NOT EXISTS arbfarm_cow_forks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_cow_id UUID NOT NULL REFERENCES arbfarm_cows(id) ON DELETE CASCADE,
    forked_cow_id UUID NOT NULL REFERENCES arbfarm_cows(id) ON DELETE CASCADE,
    forker_wallet VARCHAR(64) NOT NULL,

    -- What was inherited
    inherited_strategies UUID[] DEFAULT '{}',
    inherited_engrams TEXT[] DEFAULT '{}',

    -- Payment
    fork_price_paid_lamports BIGINT DEFAULT 0,
    tx_signature VARCHAR(128),

    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_cow_forks_parent ON arbfarm_cow_forks(parent_cow_id);
CREATE INDEX idx_cow_forks_forked ON arbfarm_cow_forks(forked_cow_id);
CREATE INDEX idx_cow_forks_forker ON arbfarm_cow_forks(forker_wallet);

-- =============================================================================
-- Revenue Tracking Tables
-- =============================================================================

-- Revenue entries for COW creators
CREATE TABLE IF NOT EXISTS arbfarm_revenue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    cow_id UUID NOT NULL REFERENCES arbfarm_cows(id) ON DELETE CASCADE,
    wallet_address VARCHAR(64) NOT NULL,
    revenue_type VARCHAR(50) NOT NULL,
    amount_lamports BIGINT NOT NULL,

    -- Source tracking
    source_trade_id UUID,
    source_fork_id UUID REFERENCES arbfarm_cow_forks(id),

    -- Period
    period_start TIMESTAMPTZ NOT NULL,
    period_end TIMESTAMPTZ NOT NULL,

    -- Distribution status
    is_distributed BOOLEAN DEFAULT false,
    distributed_at TIMESTAMPTZ,
    tx_signature VARCHAR(128),

    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_revenue_cow ON arbfarm_revenue(cow_id);
CREATE INDEX idx_revenue_wallet ON arbfarm_revenue(wallet_address);
CREATE INDEX idx_revenue_type ON arbfarm_revenue(revenue_type);
CREATE INDEX idx_revenue_distributed ON arbfarm_revenue(is_distributed) WHERE is_distributed = false;
CREATE INDEX idx_revenue_period ON arbfarm_revenue(period_start, period_end);

-- Revenue distribution batches
CREATE TABLE IF NOT EXISTS arbfarm_revenue_distributions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    batch_id VARCHAR(64) UNIQUE NOT NULL,
    total_amount_lamports BIGINT NOT NULL,
    recipient_count INTEGER NOT NULL,
    status VARCHAR(50) DEFAULT 'pending',
    tx_signature VARCHAR(128),
    error_message TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    processed_at TIMESTAMPTZ
);

CREATE INDEX idx_distributions_status ON arbfarm_revenue_distributions(status);
CREATE INDEX idx_distributions_created ON arbfarm_revenue_distributions(created_at DESC);

-- =============================================================================
-- User Interactions
-- =============================================================================

-- User favorites
CREATE TABLE IF NOT EXISTS crossroads_favorites (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    listing_id UUID NOT NULL REFERENCES crossroads_listings(id) ON DELETE CASCADE,
    wallet_address VARCHAR(64) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(listing_id, wallet_address)
);

CREATE INDEX idx_favorites_listing ON crossroads_favorites(listing_id);
CREATE INDEX idx_favorites_wallet ON crossroads_favorites(wallet_address);

-- Purchase/deployment history
CREATE TABLE IF NOT EXISTS crossroads_purchases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    listing_id UUID NOT NULL REFERENCES crossroads_listings(id) ON DELETE CASCADE,
    buyer_wallet VARCHAR(64) NOT NULL,
    price_paid_lamports BIGINT NOT NULL,
    tx_signature VARCHAR(128),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_purchases_listing ON crossroads_purchases(listing_id);
CREATE INDEX idx_purchases_buyer ON crossroads_purchases(buyer_wallet);
CREATE INDEX idx_purchases_created ON crossroads_purchases(created_at DESC);

-- =============================================================================
-- Helper Functions
-- =============================================================================

-- Function to update listing rating after review
CREATE OR REPLACE FUNCTION update_listing_rating()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE crossroads_listings
    SET rating = (
        SELECT AVG(rating)::DECIMAL(3,2)
        FROM crossroads_reviews
        WHERE listing_id = COALESCE(NEW.listing_id, OLD.listing_id)
    ),
    total_ratings = (
        SELECT COUNT(*)
        FROM crossroads_reviews
        WHERE listing_id = COALESCE(NEW.listing_id, OLD.listing_id)
    ),
    updated_at = NOW()
    WHERE id = COALESCE(NEW.listing_id, OLD.listing_id);
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_listing_rating
AFTER INSERT OR UPDATE OR DELETE ON crossroads_reviews
FOR EACH ROW EXECUTE FUNCTION update_listing_rating();

-- Function to increment fork count on parent COW
CREATE OR REPLACE FUNCTION increment_fork_count()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE arbfarm_cows
    SET fork_count = fork_count + 1,
        updated_at = NOW()
    WHERE id = NEW.parent_cow_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_increment_fork_count
AFTER INSERT ON arbfarm_cow_forks
FOR EACH ROW EXECUTE FUNCTION increment_fork_count();

-- Function to update COW performance metrics
CREATE OR REPLACE FUNCTION update_cow_performance()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE arbfarm_cows
    SET total_trades = (
            SELECT COALESCE(SUM(total_trades), 0)
            FROM arbfarm_cow_strategies
            WHERE cow_id = COALESCE(NEW.cow_id, OLD.cow_id)
        ),
        successful_trades = (
            SELECT COALESCE(SUM(successful_trades), 0)
            FROM arbfarm_cow_strategies
            WHERE cow_id = COALESCE(NEW.cow_id, OLD.cow_id)
        ),
        total_profit_generated_lamports = (
            SELECT COALESCE(SUM(total_profit_lamports), 0)
            FROM arbfarm_cow_strategies
            WHERE cow_id = COALESCE(NEW.cow_id, OLD.cow_id)
        ),
        win_rate = CASE
            WHEN (SELECT COALESCE(SUM(total_trades), 0) FROM arbfarm_cow_strategies WHERE cow_id = COALESCE(NEW.cow_id, OLD.cow_id)) > 0
            THEN (SELECT COALESCE(SUM(successful_trades), 0)::DECIMAL / COALESCE(SUM(total_trades), 1) FROM arbfarm_cow_strategies WHERE cow_id = COALESCE(NEW.cow_id, OLD.cow_id))
            ELSE 0
        END,
        updated_at = NOW()
    WHERE id = COALESCE(NEW.cow_id, OLD.cow_id);
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_cow_performance
AFTER INSERT OR UPDATE OR DELETE ON arbfarm_cow_strategies
FOR EACH ROW EXECUTE FUNCTION update_cow_performance();

-- =============================================================================
-- Initial Data
-- =============================================================================

-- No initial data needed - populated via API
