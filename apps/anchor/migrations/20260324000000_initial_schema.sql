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

CREATE TABLE IF NOT EXISTS landing_pages (
    id SERIAL PRIMARY KEY,
    slug VARCHAR(255) UNIQUE NOT NULL,
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    hero_title VARCHAR(255) NOT NULL,
    hero_subtitle VARCHAR(255) NOT NULL,
    lead_capture_title VARCHAR(255) NOT NULL,
    lead_capture_desc TEXT NOT NULL,
    lead_capture_btn VARCHAR(50) NOT NULL,
    options_json TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS nav_items (
    id SERIAL PRIMARY KEY,
    label VARCHAR(255) NOT NULL,
    href VARCHAR(255),
    parent_id INTEGER REFERENCES nav_items(id) ON DELETE CASCADE,
    display_order INTEGER NOT NULL DEFAULT 0,
    is_visible BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS footer_items (
    id SERIAL PRIMARY KEY,
    label VARCHAR(255) NOT NULL,
    href VARCHAR(255),
    display_order INTEGER NOT NULL DEFAULT 0,
    is_visible BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
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

CREATE TABLE IF NOT EXISTS page_headers (
    route_path VARCHAR(255) PRIMARY KEY,
    badge_text VARCHAR(255),
    title VARCHAR(255) NOT NULL,
    subtitle TEXT
);
