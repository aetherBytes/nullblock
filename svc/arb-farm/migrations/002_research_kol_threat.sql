-- ArbFarm: Research, KOL Tracking, and Threat Detection Tables

-- Research discoveries
CREATE TABLE IF NOT EXISTS arb_research_discoveries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_url TEXT,
    source_type TEXT NOT NULL,
    source_account TEXT,
    extracted_strategy JSONB,
    backtest_result JSONB,
    status TEXT DEFAULT 'pending',
    confidence_score NUMERIC(3, 2),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_research_status ON arb_research_discoveries(status);

-- Monitored sources (X accounts, Telegram channels, etc.)
CREATE TABLE IF NOT EXISTS arb_research_sources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_type TEXT NOT NULL,
    handle_or_url TEXT NOT NULL,
    track_type TEXT NOT NULL,
    is_active BOOLEAN DEFAULT true,
    last_checked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- KOL/Wallet tracking
CREATE TABLE IF NOT EXISTS arb_kol_entities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type TEXT NOT NULL,
    identifier TEXT NOT NULL UNIQUE,
    display_name TEXT,
    linked_wallet TEXT,
    trust_score NUMERIC(5, 2) DEFAULT 50.0,
    total_trades_tracked INTEGER DEFAULT 0,
    profitable_trades INTEGER DEFAULT 0,
    avg_profit_percent NUMERIC(8, 4),
    max_drawdown NUMERIC(8, 4),
    copy_trading_enabled BOOLEAN DEFAULT false,
    copy_config JSONB DEFAULT '{}',
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_kol_identifier ON arb_kol_entities(identifier);
CREATE INDEX IF NOT EXISTS idx_kol_trust ON arb_kol_entities(trust_score DESC);

-- KOL trade history
CREATE TABLE IF NOT EXISTS arb_kol_trades (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_id UUID REFERENCES arb_kol_entities(id),
    tx_signature TEXT NOT NULL,
    trade_type TEXT NOT NULL,
    token_mint TEXT NOT NULL,
    amount_sol NUMERIC(20, 9),
    price_at_trade NUMERIC(20, 9),
    detected_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_kol_trades_entity ON arb_kol_trades(entity_id);

-- Copy trade executions
CREATE TABLE IF NOT EXISTS arb_copy_trades (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_id UUID REFERENCES arb_kol_entities(id),
    kol_trade_id UUID REFERENCES arb_kol_trades(id),
    our_tx_signature TEXT,
    copy_amount_sol NUMERIC(20, 9),
    delay_ms BIGINT,
    profit_loss_lamports BIGINT,
    status TEXT DEFAULT 'pending',
    executed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_copy_trades_entity ON arb_copy_trades(entity_id);
CREATE INDEX IF NOT EXISTS idx_copy_trades_status ON arb_copy_trades(status);

-- Threat scores for tokens
CREATE TABLE IF NOT EXISTS arb_threat_scores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token_mint TEXT NOT NULL,
    overall_score NUMERIC(3, 2) NOT NULL,
    factors JSONB NOT NULL,
    confidence NUMERIC(3, 2),
    external_data JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_threat_scores_token ON arb_threat_scores(token_mint);
CREATE INDEX IF NOT EXISTS idx_threat_scores_score ON arb_threat_scores(overall_score DESC);
CREATE INDEX IF NOT EXISTS idx_threat_scores_created ON arb_threat_scores(created_at DESC);

-- Blocked/flagged entities
CREATE TABLE IF NOT EXISTS arb_threat_blocked (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type TEXT NOT NULL,
    address TEXT UNIQUE NOT NULL,
    threat_category TEXT NOT NULL,
    threat_score NUMERIC(3, 2),
    reason TEXT,
    evidence_url TEXT,
    reported_by TEXT,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_threat_blocked_type ON arb_threat_blocked(entity_type);
CREATE INDEX IF NOT EXISTS idx_threat_blocked_category ON arb_threat_blocked(threat_category);

-- Whitelisted trusted entities
CREATE TABLE IF NOT EXISTS arb_threat_whitelist (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type TEXT NOT NULL,
    address TEXT UNIQUE NOT NULL,
    reason TEXT,
    whitelisted_by TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- High-alert monitoring (creator wallets, etc.)
CREATE TABLE IF NOT EXISTS arb_threat_watched (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_address TEXT NOT NULL,
    related_token_mint TEXT,
    watch_reason TEXT,
    alert_on_sell BOOLEAN DEFAULT true,
    alert_on_transfer BOOLEAN DEFAULT true,
    alert_threshold_sol NUMERIC(20, 9),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_threat_watched_wallet ON arb_threat_watched(wallet_address);

-- Threat alerts history
CREATE TABLE IF NOT EXISTS arb_threat_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    alert_type TEXT NOT NULL,
    severity TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    address TEXT NOT NULL,
    details JSONB NOT NULL,
    action_taken TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_threat_alerts_type ON arb_threat_alerts(alert_type);
CREATE INDEX IF NOT EXISTS idx_threat_alerts_severity ON arb_threat_alerts(severity);
CREATE INDEX IF NOT EXISTS idx_threat_alerts_created ON arb_threat_alerts(created_at DESC);
