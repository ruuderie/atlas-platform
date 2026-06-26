use sea_orm_migration::prelude::*;

/// GTM Landing Page Engine: product_tracking_pixels
///
/// Stores per-product tracking pixel snippets (GA4, GTM container, Meta Pixel,
/// LinkedIn Insight Tag, TikTok, or any custom <script> block) that are injected
/// into landing page HTML at SSR render time.
///
/// # Design
///
/// Pixels are product-scoped (not variant-scoped) so a single GA4 measurement ID
/// or Meta Pixel ID covers all market variants of a product. Variants don't need
/// separate pixels — UTM params + gclid/fbclid on the lead record differentiate
/// traffic sources at the attribution layer.
///
/// `inject_at`: 'head' | 'body_start' | 'body_end'
/// `pixel_type`: 'gtm' | 'ga4' | 'meta' | 'linkedin' | 'tiktok' | 'custom'
///
/// # Usage
///
/// The axum SSR landing page handler fetches all active pixels for a product
/// (cached, TTL ~5 min) and injects them into the rendered HTML before responding.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TABLE IF NOT EXISTS product_tracking_pixels (
                    id           UUID        NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
                    product_id   UUID        NOT NULL
                                             REFERENCES platform_products(id) ON DELETE CASCADE,
                    name         TEXT        NOT NULL,
                    pixel_type   TEXT        NOT NULL
                                             CONSTRAINT chk_pixel_type CHECK (
                                                 pixel_type IN (
                                                     'gtm', 'ga4', 'meta',
                                                     'linkedin', 'tiktok', 'custom'
                                                 )
                                             ),
                    snippet      TEXT        NOT NULL,
                    inject_at    TEXT        NOT NULL DEFAULT 'head'
                                             CONSTRAINT chk_inject_at CHECK (
                                                 inject_at IN ('head', 'body_start', 'body_end')
                                             ),
                    is_active    BOOLEAN     NOT NULL DEFAULT true,
                    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
                    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now()
                );

                CREATE INDEX IF NOT EXISTS idx_product_tracking_pixels_product_id
                    ON product_tracking_pixels (product_id)
                    WHERE is_active = true;

                COMMENT ON TABLE product_tracking_pixels IS
                    'Per-product tracking snippets injected into landing pages at SSR render time.';
                "#,
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS product_tracking_pixels;")
            .await?;
        Ok(())
    }
}
