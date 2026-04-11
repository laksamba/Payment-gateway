CREATE TABLE payments (
  id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  merchant_id       UUID NOT NULL REFERENCES merchants(id),
  amount            NUMERIC(20,6) NOT NULL,
  currency          TEXT NOT NULL DEFAULT 'USDT',
  deposit_address   TEXT NOT NULL UNIQUE,
  status            TEXT NOT NULL DEFAULT 'pending'
                    CHECK (status IN ('pending','detected',
                    'confirming','confirmed','expired','failed')),
  tx_hash           TEXT,
  confirmations     INTEGER NOT NULL DEFAULT 0,
  required_confirmations INTEGER NOT NULL DEFAULT 20,
  idempotency_key   TEXT UNIQUE,
  metadata          JSONB NOT NULL DEFAULT '{}',
  expires_at        TIMESTAMPTZ NOT NULL,
  confirmed_at      TIMESTAMPTZ,
  created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_payments_merchant ON payments(merchant_id);
CREATE INDEX idx_payments_status   ON payments(status);
CREATE INDEX idx_payments_address  ON payments(deposit_address);
CREATE INDEX idx_payments_idem     ON payments(idempotency_key)
  WHERE idempotency_key IS NOT NULL;