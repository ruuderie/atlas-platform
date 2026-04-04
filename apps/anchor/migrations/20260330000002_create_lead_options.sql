CREATE TABLE IF NOT EXISTS lead_capture_options (
    id SERIAL PRIMARY KEY,
    value_key VARCHAR(255) UNIQUE NOT NULL,
    label VARCHAR(255) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed with default initial options based on previous state
INSERT INTO lead_capture_options (value_key, label, is_active, display_order) VALUES
('mailing_list', 'Join Mailing List', true, 10),
('resume', 'Request Tailored CV', true, 20)
ON CONFLICT (value_key) DO NOTHING;

-- Cleanup the old JSON settings
DELETE FROM site_settings WHERE key = 'landing_options_json';
