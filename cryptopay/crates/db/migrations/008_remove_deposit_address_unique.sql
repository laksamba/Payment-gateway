-- In non-custodial mode, deposit_address = merchant's own wallet address.
-- The same merchant can have multiple payments, all pointing to the same wallet.
-- The old UNIQUE constraint on deposit_address prevented this.

ALTER TABLE payments DROP CONSTRAINT IF EXISTS payments_deposit_address_key;
