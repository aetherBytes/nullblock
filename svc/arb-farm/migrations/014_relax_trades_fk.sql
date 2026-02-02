ALTER TABLE arb_trades DROP CONSTRAINT IF EXISTS arb_trades_edge_id_fkey;
ALTER TABLE arb_trades DROP CONSTRAINT IF EXISTS arb_trades_strategy_id_fkey;
ALTER TABLE arb_trades ALTER COLUMN edge_id DROP NOT NULL;
ALTER TABLE arb_trades ALTER COLUMN strategy_id DROP NOT NULL;
