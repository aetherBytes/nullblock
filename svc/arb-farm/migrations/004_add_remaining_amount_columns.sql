-- Add remaining_amount columns to track position size after partial exits
-- This fixes the P&L calculation bug where unrealized P&L was calculated
-- on the original entry amount instead of the remaining position size

ALTER TABLE arb_positions
ADD COLUMN IF NOT EXISTS remaining_amount_base DECIMAL(20, 9);

ALTER TABLE arb_positions
ADD COLUMN IF NOT EXISTS remaining_token_amount DECIMAL(30, 0);

-- Initialize existing positions with entry values (for open positions)
UPDATE arb_positions
SET remaining_amount_base = entry_amount_base,
    remaining_token_amount = entry_token_amount
WHERE remaining_amount_base IS NULL;

-- Set NOT NULL with default for future inserts
ALTER TABLE arb_positions
ALTER COLUMN remaining_amount_base SET DEFAULT 0;

ALTER TABLE arb_positions
ALTER COLUMN remaining_token_amount SET DEFAULT 0;
