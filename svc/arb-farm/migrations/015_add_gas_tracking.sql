ALTER TABLE arb_trades ADD COLUMN IF NOT EXISTS entry_gas_lamports BIGINT;
ALTER TABLE arb_trades ADD COLUMN IF NOT EXISTS exit_gas_lamports BIGINT;
ALTER TABLE arb_trades ADD COLUMN IF NOT EXISTS pnl_source TEXT DEFAULT 'estimated';

ALTER TABLE arb_positions ADD COLUMN IF NOT EXISTS entry_gas_lamports BIGINT;
ALTER TABLE arb_positions ADD COLUMN IF NOT EXISTS exit_gas_lamports BIGINT;
ALTER TABLE arb_positions ADD COLUMN IF NOT EXISTS pnl_source TEXT DEFAULT 'estimated';
