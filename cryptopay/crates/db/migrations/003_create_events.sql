CREATE TABLE events (
  id          BIGSERIAL PRIMARY KEY,
  payment_id  UUID NOT NULL REFERENCES payments(id),
  event_type  TEXT NOT NULL,
  payload     JSONB NOT NULL DEFAULT '{}',
  created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- This table is append-only. Never UPDATE or DELETE from it.
CREATE INDEX idx_events_payment ON events(payment_id);
CREATE INDEX idx_events_type    ON events(event_type);