-- KagoRoute engine — schema-driven logging columns.

ALTER TABLE callback_logs
    ADD COLUMN IF NOT EXISTS flow_id TEXT,
    ADD COLUMN IF NOT EXISTS flow_version INTEGER,
    ADD COLUMN IF NOT EXISTS variables TEXT;

CREATE INDEX IF NOT EXISTS idx_callback_logs_flow ON callback_logs (flow_id, flow_version);
