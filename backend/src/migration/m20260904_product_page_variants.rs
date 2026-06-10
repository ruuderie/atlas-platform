use sea_orm_migration::prelude::*;

/// Product Page Templates + Variants — programmatic SEO landing page engine.
///
/// ## Template (`product_page_templates`)
/// One master CMS template per product. All variants inherit from this.
/// Fields: hero, blocks, default SEO, default CTA action.
///
/// ## Variant (`product_page_variants`)
/// N rows per product — one per city / market / locale.
/// Each variant stores field-level overrides over the template.
/// Public URL: /products/{product_slug}/{variant_slug}
///
/// ## Lead deduplication rule
/// 1 atlas_lead per (email + product_id). The `source_metadata` column is a JSONB
/// ARRAY — additional market joins append to it rather than creating a new lead row.
/// The dedup + append logic lives in the waitlist service.
///
/// ## Pre-order caps
/// Variant-level: `pre_order_cap` + `pre_order_sold` on the variant.
/// If null, falls back to product-level cap. null on both = unlimited.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                // ── product_page_templates ─────────────────────────────────────
                "CREATE TABLE IF NOT EXISTS product_page_templates (
                    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
                    product_id      UUID        NOT NULL
                                    REFERENCES platform_products(id) ON DELETE CASCADE,

                    -- Block content (same shape as app_pages)
                    hero_payload    JSONB       NOT NULL DEFAULT '{}',
                    blocks_payload  JSONB       NOT NULL DEFAULT '[]',

                    -- Default SEO (inherited by all variants unless overridden)
                    meta_title          TEXT,
                    meta_description    TEXT,
                    og_image_url        TEXT,
                    -- JSON-LD schema.org/SoftwareApplication
                    structured_data     JSONB   NOT NULL DEFAULT '{}',

                    -- Default CTA
                    cta_label       TEXT        NOT NULL DEFAULT 'Join the Waitlist',
                    -- 'waitlist' | 'pre_order' | 'signup' | 'contact' | 'ai_localize'
                    cta_action      TEXT        NOT NULL DEFAULT 'waitlist',

                    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),

                    CONSTRAINT uq_product_page_template_product UNIQUE (product_id)
                );",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                // ── product_page_variants ──────────────────────────────────────
                "CREATE TABLE IF NOT EXISTS product_page_variants (
                    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
                    product_id      UUID        NOT NULL
                                    REFERENCES platform_products(id) ON DELETE CASCADE,
                    template_id     UUID        NOT NULL
                                    REFERENCES product_page_templates(id) ON DELETE CASCADE,

                    -- Identity / URL
                    variant_slug    TEXT        NOT NULL,
                    -- Full public path: /products/{product.slug}/{variant_slug}

                    -- Geo / locale
                    locale          TEXT        NOT NULL DEFAULT 'en',
                    country_code    TEXT,       -- ISO 3166-1 alpha-2
                    region          TEXT,       -- state / province
                    city            TEXT,
                    geo_lat         DOUBLE PRECISION,
                    geo_lng         DOUBLE PRECISION,

                    -- Content overrides (field-level diff over template)
                    hero_overrides  JSONB       NOT NULL DEFAULT '{}',
                    -- block_overrides: { [block_id]: { field: value } }
                    block_overrides JSONB       NOT NULL DEFAULT '{}',

                    -- SEO overrides (null = inherit from template)
                    meta_title          TEXT,
                    meta_description    TEXT,
                    og_image_url        TEXT,
                    canonical_url       TEXT,
                    -- LocalBusiness JSON-LD for this specific market
                    structured_data     JSONB,

                    -- Launch state (per-variant — can differ from product default)
                    launch_mode     TEXT        NOT NULL DEFAULT 'draft'
                                    CONSTRAINT chk_variant_launch_mode
                                    CHECK (launch_mode IN (
                                        'draft', 'pre_launch', 'waitlist',
                                        'active', 'invite_only', 'deprecated'
                                    )),
                    is_published    BOOLEAN     NOT NULL DEFAULT false,

                    -- CTA override (null = inherit from template)
                    cta_label       TEXT,
                    cta_action      TEXT,

                    -- Per-variant pre-order cap (null = use product-level cap)
                    pre_order_cap   INTEGER,
                    pre_order_sold  INTEGER     NOT NULL DEFAULT 0,

                    -- Denormalized metrics (incremented via service)
                    lead_count      INTEGER     NOT NULL DEFAULT 0,
                    view_count      INTEGER     NOT NULL DEFAULT 0,

                    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),

                    CONSTRAINT uq_product_page_variant_slug UNIQUE (product_id, variant_slug)
                );

                CREATE INDEX IF NOT EXISTS idx_product_page_variants_product
                    ON product_page_variants (product_id);
                CREATE INDEX IF NOT EXISTS idx_product_page_variants_country
                    ON product_page_variants (country_code);
                CREATE INDEX IF NOT EXISTS idx_product_page_variants_locale
                    ON product_page_variants (locale);
                CREATE INDEX IF NOT EXISTS idx_product_page_variants_published
                    ON product_page_variants (is_published, launch_mode);",
            )
            .await?;

        // updated_at triggers
        manager
            .get_connection()
            .execute_unprepared(
                "DO $$ BEGIN
                    CREATE TRIGGER set_updated_at_product_page_templates
                        BEFORE UPDATE ON product_page_templates
                        FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
                 EXCEPTION WHEN duplicate_object THEN NULL; END $$;

                 DO $$ BEGIN
                    CREATE TRIGGER set_updated_at_product_page_variants
                        BEFORE UPDATE ON product_page_variants
                        FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
                 EXCEPTION WHEN duplicate_object THEN NULL; END $$;",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DROP TRIGGER IF EXISTS set_updated_at_product_page_variants ON product_page_variants;
                 DROP TRIGGER IF EXISTS set_updated_at_product_page_templates ON product_page_templates;
                 DROP TABLE IF EXISTS product_page_variants;
                 DROP TABLE IF EXISTS product_page_templates;",
            )
            .await?;
        Ok(())
    }
}
