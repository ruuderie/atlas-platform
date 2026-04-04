CREATE TABLE IF NOT EXISTS system_secrets (
    key VARCHAR(255) PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO system_secrets (key, value) VALUES
    ('smtp_host', 'smtp.protonmail.ch'),
    ('smtp_port', '587'),
    ('smtp_username', 'ruud@oply.co'),
    ('smtp_token', ''),
    ('smtp_from', 'ruud@oply.co')
ON CONFLICT (key) DO NOTHING;
