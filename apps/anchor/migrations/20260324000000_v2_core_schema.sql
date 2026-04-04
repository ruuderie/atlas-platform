CREATE TABLE IF NOT EXISTS users (
  id SERIAL PRIMARY KEY,
  username TEXT UNIQUE NOT NULL,
  passkey JSONB NOT NULL,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  session_token TEXT UNIQUE
);

CREATE TABLE IF NOT EXISTS auth_challenges (
  id UUID PRIMARY KEY,
  challenge_data JSONB NOT NULL,
  created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS blog_posts (
  id SERIAL PRIMARY KEY,
  slug TEXT UNIQUE NOT NULL,
  title TEXT NOT NULL,
  content TEXT NOT NULL,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  tags TEXT[] NOT NULL DEFAULT '{}'
);

CREATE TABLE IF NOT EXISTS bitcoin_blocks (
    id TEXT PRIMARY KEY,
    height BIGINT NOT NULL UNIQUE,
    version BIGINT,
    timestamp BIGINT NOT NULL,
    tx_count INT,
    size INT,
    weight INT,
    merkle_root TEXT,
    previousblockhash TEXT,
    mediantime BIGINT,
    nonce BIGINT,
    bits BIGINT,
    difficulty DOUBLE PRECISION,
    fetched_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS site_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS mailing_list (
    id SERIAL PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    list_type TEXT NOT NULL,
    preferences JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS page_views (
    id SERIAL PRIMARY KEY,
    path TEXT NOT NULL,
    user_agent TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS api_requests_log (
    id SERIAL PRIMARY KEY,
    endpoint TEXT NOT NULL,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TYPE resume_category_enum AS ENUM (
    'work', 
    'education', 
    'certification',
    'skill', 
    'project', 
    'language', 
    'volunteer', 
    'extracurricular', 
    'hobby'
);

CREATE TABLE IF NOT EXISTS resume_profiles (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    full_name VARCHAR(255) NOT NULL DEFAULT 'Document Header',
    objective TEXT NOT NULL,
    is_public BOOLEAN NOT NULL DEFAULT FALSE,
    target_role VARCHAR(255),
    contact_email VARCHAR(255),
    contact_phone VARCHAR(255),
    contact_location VARCHAR(255),
    contact_link VARCHAR(255),
    category_visibility JSONB DEFAULT '{}'::jsonb,
    category_order JSONB NOT NULL DEFAULT '["work", "education", "certification", "project", "skill", "volunteer", "extracurricular", "language", "hobby"]'::jsonb,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS resume_entries (
    id SERIAL PRIMARY KEY,
    category resume_category_enum NOT NULL,
    title VARCHAR(500) NOT NULL,
    subtitle VARCHAR(500),
    date_range VARCHAR(255),
    bullets JSONB NOT NULL DEFAULT '[]'::jsonb,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS resume_profile_entries (
    profile_id INTEGER NOT NULL REFERENCES resume_profiles(id) ON DELETE CASCADE,
    entry_id INTEGER NOT NULL REFERENCES resume_entries(id) ON DELETE CASCADE,
    display_order INTEGER NOT NULL DEFAULT 0,
    is_visible BOOLEAN NOT NULL DEFAULT TRUE,
    overrides JSONB DEFAULT '{ }'::jsonb,
    PRIMARY KEY (profile_id, entry_id)
);


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


INSERT INTO site_settings (key, value) VALUES 
    ('meta_title', 'Ruud Salym Erie - Technical Architect'),
    ('meta_description', 'Technical Architect and Software Engineer specializing in Rust, Salesforce, and high-performance enterprise applications.'),
    ('og_image', '')
ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value;


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


-- Add tenant_id to support Application-Level Multi-Tenancy from Atlas Router
-- Standard Modules
ALTER TABLE users ADD COLUMN tenant_id UUID;
ALTER TABLE auth_challenges ADD COLUMN tenant_id UUID;
ALTER TABLE blog_posts ADD COLUMN tenant_id UUID;
ALTER TABLE mailing_list ADD COLUMN tenant_id UUID;
ALTER TABLE page_views ADD COLUMN tenant_id UUID;
ALTER TABLE api_requests_log ADD COLUMN tenant_id UUID;
ALTER TABLE resume_profiles ADD COLUMN tenant_id UUID;
ALTER TABLE resume_entries ADD COLUMN tenant_id UUID;

-- B2B / SaaS Modules
ALTER TABLE services ADD COLUMN tenant_id UUID;
ALTER TABLE case_studies ADD COLUMN tenant_id UUID;
ALTER TABLE highlights ADD COLUMN tenant_id UUID;
ALTER TABLE lead_capture_options ADD COLUMN tenant_id UUID;

-- 1. Update site_settings composite Primary Key
ALTER TABLE site_settings DROP CONSTRAINT IF EXISTS site_settings_pkey;
ALTER TABLE site_settings ADD COLUMN tenant_id UUID;
ALTER TABLE site_settings ADD PRIMARY KEY (tenant_id, key);

-- 2. Refactor UNIQUE constraints for Multi-Tenancy
-- Users can have the same username across different tenants
ALTER TABLE users DROP CONSTRAINT IF EXISTS users_username_key;
ALTER TABLE users ADD CONSTRAINT users_tenant_username_key UNIQUE (tenant_id, username);

-- Blog Posts can have the same slug across different tenants
ALTER TABLE blog_posts DROP CONSTRAINT IF EXISTS blog_posts_slug_key;
ALTER TABLE blog_posts ADD CONSTRAINT blog_posts_tenant_slug_key UNIQUE (tenant_id, slug);

-- Mailing List emails can be duplicated across different tenants
ALTER TABLE mailing_list DROP CONSTRAINT IF EXISTS mailing_list_email_key;
ALTER TABLE mailing_list ADD CONSTRAINT mailing_list_tenant_email_key UNIQUE (tenant_id, email);

-- Lead Capture Options value_keys can be duplicated across different tenants
ALTER TABLE lead_capture_options DROP CONSTRAINT IF EXISTS lead_capture_options_value_key_key;
ALTER TABLE lead_capture_options ADD CONSTRAINT lead_capture_options_tenant_value_key_key UNIQUE (tenant_id, value_key);

-- 3. Create indexes to speed up tenant-specific queries
CREATE INDEX IF NOT EXISTS idx_users_tenant ON users(tenant_id);
CREATE INDEX IF NOT EXISTS idx_blog_posts_tenant ON blog_posts(tenant_id);
CREATE INDEX IF NOT EXISTS idx_services_tenant ON services(tenant_id);
CREATE INDEX IF NOT EXISTS idx_case_studies_tenant ON case_studies(tenant_id);
CREATE INDEX IF NOT EXISTS idx_highlights_tenant ON highlights(tenant_id);


