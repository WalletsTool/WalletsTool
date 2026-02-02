ALTER TABLE chains ADD COLUMN ecosystem TEXT NOT NULL DEFAULT 'evm';
UPDATE chains SET ecosystem = 'solana' WHERE chain_key IN ('sol', 'solana') AND ecosystem != 'solana';
UPDATE chains SET ecosystem = 'evm' WHERE (ecosystem IS NULL OR ecosystem = '');
