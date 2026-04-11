CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE merchants (
  id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name              TEXT NOT NULL,
  email             TEXT UNIQUE NOT NULL,
  api_key_hash      TEXT NOT NULL,
  webhook_url       TEXT,
  webhook_secret    TEXT NOT NULL,
  fee_percent       NUMERIC(5,2) NOT NULL DEFAULT 1.00,
  is_active         BOOLEAN NOT NULL DEFAULT true,
  created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_merchants_email ON merchants(email);