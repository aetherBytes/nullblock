-- KOL Enhancements: Additional columns and indexes for copy trading

-- Performance index for copy-enabled KOLs
CREATE INDEX IF NOT EXISTS idx_kol_entities_copy_enabled
ON arb_kol_entities(copy_trading_enabled) WHERE copy_trading_enabled = true;

-- Discovery tracking columns
ALTER TABLE arb_kol_entities ADD COLUMN IF NOT EXISTS discovery_source TEXT;
ALTER TABLE arb_kol_entities ADD COLUMN IF NOT EXISTS last_trade_at TIMESTAMPTZ;
ALTER TABLE arb_kol_entities ADD COLUMN IF NOT EXISTS total_volume_sol NUMERIC(20, 9) DEFAULT 0;
ALTER TABLE arb_kol_entities ADD COLUMN IF NOT EXISTS win_streak INTEGER DEFAULT 0;

-- Token tracking for KOL trades
ALTER TABLE arb_kol_trades ADD COLUMN IF NOT EXISTS token_symbol TEXT;
ALTER TABLE arb_kol_trades ADD COLUMN IF NOT EXISTS token_amount NUMERIC(20, 9);

-- Additional indexes for trade queries
CREATE INDEX IF NOT EXISTS idx_kol_trades_detected ON arb_kol_trades(detected_at DESC);
CREATE INDEX IF NOT EXISTS idx_kol_trades_token ON arb_kol_trades(token_mint);

-- Copy trade performance indexes
CREATE INDEX IF NOT EXISTS idx_copy_trades_executed ON arb_copy_trades(executed_at DESC) WHERE status = 'executed';
CREATE INDEX IF NOT EXISTS idx_copy_trades_kol_trade ON arb_copy_trades(kol_trade_id);

-- Pending copy trades for processing
ALTER TABLE arb_copy_trades ADD COLUMN IF NOT EXISTS is_copied BOOLEAN DEFAULT false;
CREATE INDEX IF NOT EXISTS idx_copy_trades_pending
ON arb_kol_trades(entity_id, detected_at DESC)
WHERE id NOT IN (SELECT kol_trade_id FROM arb_copy_trades WHERE kol_trade_id IS NOT NULL);
