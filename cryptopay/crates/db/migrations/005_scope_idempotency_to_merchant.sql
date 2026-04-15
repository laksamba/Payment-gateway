DROP INDEX IF EXISTS idx_payments_idem;
ALTER TABLE payments DROP CONSTRAINT IF EXISTS payments_idempotency_key_key;

CREATE UNIQUE INDEX idx_payments_idem_per_merchant
  ON payments(merchant_id, idempotency_key)
  WHERE idempotency_key IS NOT NULL;
