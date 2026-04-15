ALTER TABLE payments
  ADD COLUMN sweep_status TEXT NOT NULL DEFAULT 'not_swept'
    CHECK (sweep_status IN ('not_swept', 'swept', 'sweep_failed')),
  ADD COLUMN sweep_tx_hash TEXT,
  ADD COLUMN swept_at TIMESTAMPTZ,
  ADD COLUMN sweep_error TEXT;

CREATE INDEX idx_payments_sweep_status
  ON payments(sweep_status, status);
