use sea_orm_migration::prelude::*;

/// Platform Products Registry — platform_products table.
///
/// Stores the set of Atlas Platform products (Folio, Anchor, NetworkInstance, Meridian)
/// as first-class entities. These exist **before any tenant** and are managed
/// from platform-admin.
///
/// Each product can reference a CMS page (`marketing_page_cms_id`) for its
/// marketing landing page and a Cloudflare Pages deploy hook URL for one-click
/// publishing from platform-admin.
///
/// Seed data: Folio, Anchor, NetworkInstance, Meridian
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE IF NOT EXISTS platform_products (
                    id                      UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
                    name                    TEXT        NOT NULL,
                    -- URL-safe identifier, e.g. 'folio', 'anchor'
                    slug                    TEXT        NOT NULL,
                    tagline                 TEXT,
                    -- 'active' | 'beta' | 'deprecated'
                    status                  TEXT        NOT NULL DEFAULT 'active',
                    -- FK to app_pages (CMS page) — the marketing landing page for this product
                    marketing_page_cms_id   UUID,
                    -- Cloudflare Pages deploy hook URL (POST triggers a new deploy)
                    deploy_hook_url         TEXT,
                    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
                    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
                    CONSTRAINT uq_platform_products_slug UNIQUE (slug)
                );

                CREATE INDEX IF NOT EXISTS idx_platform_products_status
                    ON platform_products (status);

                -- Seed the four core Atlas Platform products
                INSERT INTO platform_products (id, name, slug, tagline, status)
                VALUES
                    (gen_random_uuid(), 'Folio',           'folio',           'Cross-border property management',   'active'),
                    (gen_random_uuid(), 'Anchor',          'anchor',          'Business presence and lead engine',  'active'),
                    (gen_random_uuid(), 'NetworkInstance', 'network',         'Vertical directory and marketplace', 'active'),
                    (gen_random_uuid(), 'Meridian',        'meridian',        'Global market intelligence',         'beta')
                ON CONFLICT (slug) DO NOTHING;",
            )
            .await?;

        // updated_at trigger
        manager
            .get_connection()
            .execute_unprepared(
                "DO $$ BEGIN
                    CREATE TRIGGER set_updated_at_platform_products
                        BEFORE UPDATE ON platform_products
                        FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
                 EXCEPTION WHEN duplicate_object THEN NULL;
                 END $$;",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DROP TRIGGER IF EXISTS set_updated_at_platform_products ON platform_products;
                 DROP TABLE IF EXISTS platform_products;",
            )
            .await?;
        Ok(())
    }
}
