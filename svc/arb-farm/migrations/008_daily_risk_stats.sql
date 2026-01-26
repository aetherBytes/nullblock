-- Daily risk stats persistence table
-- Ensures daily loss limits survive restarts

CREATE TABLE IF NOT EXISTS daily_risk_stats (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    date DATE NOT NULL UNIQUE,
    total_profit_lamports BIGINT NOT NULL DEFAULT 0,
    total_loss_lamports BIGINT NOT NULL DEFAULT 0,
    trade_count INTEGER NOT NULL DEFAULT 0,
    winning_trades INTEGER NOT NULL DEFAULT 0,
    losing_trades INTEGER NOT NULL DEFAULT 0,
    last_loss_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_daily_risk_stats_date ON daily_risk_stats(date);
