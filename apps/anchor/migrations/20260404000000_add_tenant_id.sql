-- Add tenant_id to support Application-Level Multi-Tenancy from Atlas Router
-- Standard Modules
ALTER TABLE users ADD COLUMN tenant_id UUID;
ALTER TABLE auth_challenges ADD COLUMN tenant_id UUID;
ALTER TABLE blog_posts ADD COLUMN tenant_id UUID;
ALTER TABLE mailing_list ADD COLUMN tenant_id UUID;
ALTER TABLE page_views ADD COLUMN tenant_id UUID;
ALTER TABLE api_requests_log ADD COLUMN tenant_id UUID;
ALTER TABLE landing_pages ADD COLUMN tenant_id UUID;
ALTER TABLE nav_items ADD COLUMN tenant_id UUID;
ALTER TABLE footer_items ADD COLUMN tenant_id UUID;
ALTER TABLE resume_profiles ADD COLUMN tenant_id UUID;
ALTER TABLE resume_entries ADD COLUMN tenant_id UUID;
ALTER TABLE page_headers ADD COLUMN tenant_id UUID;

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

-- Landing Pages can have the same slug across different tenants
ALTER TABLE landing_pages DROP CONSTRAINT IF EXISTS landing_pages_slug_key;
ALTER TABLE landing_pages ADD CONSTRAINT landing_pages_tenant_slug_key UNIQUE (tenant_id, slug);

-- Lead Capture Options value_keys can be duplicated across different tenants
ALTER TABLE lead_capture_options DROP CONSTRAINT IF EXISTS lead_capture_options_value_key_key;
ALTER TABLE lead_capture_options ADD CONSTRAINT lead_capture_options_tenant_value_key_key UNIQUE (tenant_id, value_key);

-- 3. Create indexes to speed up tenant-specific queries
CREATE INDEX IF NOT EXISTS idx_users_tenant ON users(tenant_id);
CREATE INDEX IF NOT EXISTS idx_blog_posts_tenant ON blog_posts(tenant_id);
CREATE INDEX IF NOT EXISTS idx_landing_pages_tenant ON landing_pages(tenant_id);
CREATE INDEX IF NOT EXISTS idx_services_tenant ON services(tenant_id);
CREATE INDEX IF NOT EXISTS idx_case_studies_tenant ON case_studies(tenant_id);
CREATE INDEX IF NOT EXISTS idx_highlights_tenant ON highlights(tenant_id);
