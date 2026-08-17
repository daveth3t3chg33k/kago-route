-- KagoRoute engine — initial schema.

CREATE TABLE IF NOT EXISTS callback_logs (
    id           BIGSERIAL PRIMARY KEY,
    session_id   TEXT        NOT NULL,
    service_code TEXT        NOT NULL,
    phone_number TEXT        NOT NULL,
    ussd_text    TEXT        NOT NULL,
    reply        TEXT        NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_callback_logs_session ON callback_logs (session_id);
CREATE INDEX IF NOT EXISTS idx_callback_logs_phone  ON callback_logs (phone_number);
CREATE INDEX IF NOT EXISTS idx_callback_logs_created ON callback_logs (created_at DESC);
