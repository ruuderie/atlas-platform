CREATE TABLE IF NOT EXISTS services (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    deliverables JSONB NOT NULL DEFAULT '[]'::jsonb,
    price_range VARCHAR(255),
    is_visible BOOLEAN NOT NULL DEFAULT TRUE,
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS case_studies (
    id SERIAL PRIMARY KEY,
    client_name VARCHAR(255) NOT NULL,
    problem TEXT NOT NULL,
    solution TEXT NOT NULL,
    roi_impact TEXT NOT NULL,
    is_visible BOOLEAN NOT NULL DEFAULT TRUE,
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS highlights (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    url VARCHAR(500) NOT NULL,
    image_url VARCHAR(500),
    description TEXT,
    is_visible BOOLEAN NOT NULL DEFAULT TRUE,
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO site_settings (key, value)
VALUES 
    ('booking_url', 'https://cal.com/ruuderie/15min'),
    ('terms_html', '# Terms of Service\n\nPlease review our terms.'),
    ('privacy_html', '# Privacy Policy\n\nWe respect your digital privacy.'),
    ('github_url', 'https://github.com/ruuderie'),
    ('x_url', 'https://x.com/ruuderie'),
    ('linkedin_url', 'https://linkedin.com/in/ruuderie')
ON CONFLICT (key) DO NOTHING;
