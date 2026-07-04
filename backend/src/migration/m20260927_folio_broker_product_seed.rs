//! m20260927_folio_broker_product_seed — Folio Broker Product Launch Engine Seed
//!
//! Seeds the `folio-broker` platform_product so that:
//!   1. `GET /api/pub/products/folio-broker` returns 200 with `launch_mode = 'waitlist'`
//!   2. `BrokerLandingPage` at `/brokers` resolves content instead of hitting error fallback.
//!   3. The "🤝 Broker Page" app selector in platform-admin has a backing product record.
//!   4. `GET /api/admin/landing-pages?app_id=folio-broker` resolves to the correct product scope.
//!
//! Without this seed row the `/api/pub/products/folio-broker` handler returns 404,
//! causing `BrokerLandingPage` to render the hardcoded fallback content (which is
//! intentional and safe), but platform-admin A/B testing + tracking pixels won't
//! attach to a known product record.
//!
//! Idempotent: both `INSERT … ON CONFLICT (slug) DO NOTHING` and the
//! `WHERE NOT EXISTS` guard in the template insert ensure re-running is safe.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // ── Step 1: Register folio-broker in platform_products ─────────────────
        //
        // app_slug = 'property_management' — same binary as Folio (folio k8s service).
        // launch_mode = 'waitlist' — renders the waitlist form and beta messaging.
        // This must not be 'draft', which renders <NotFound/>.
        db.execute_unprepared(
            "INSERT INTO platform_products (
                 id, name, slug, app_slug, status, launch_mode,
                 pre_order_enabled, pre_order_currency, pre_order_sold, waitlist_count,
                 apex_domain_verified, created_at, updated_at
             )
             VALUES (
                 gen_random_uuid(),
                 'Folio — Broker Edition',
                 'folio-broker',
                 'property_management',
                 'active',
                 'waitlist',
                 false, 'usd', 0, 0, false, NOW(), NOW()
             )
             ON CONFLICT (slug) DO NOTHING;",
        )
        .await?;

        // ── Step 2: Seed master product_page_template ──────────────────────────
        //
        // This row is the content fallback until the GTM Landing Page Builder
        // publishes a page for folio-broker via platform-admin.
        //
        // hero_payload / blocks_payload are {} — the Leptos frontend has hardcoded
        // UI in BrokerDefault. These fields exist for future CMS-driven content.
        db.execute_unprepared(
            "INSERT INTO product_page_templates (
                 id,
                 product_id,
                 hero_payload,
                 blocks_payload,
                 meta_title,
                 meta_description,
                 og_image_url,
                 structured_data,
                 cta_label,
                 cta_action,
                 created_at,
                 updated_at
             )
             SELECT
                 gen_random_uuid(),
                 p.id,
                 '{}'::jsonb,
                 '{}'::jsonb,
                 'Folio for Brokers & Property Managers — Run Your Whole Brokerage',
                 'Multi-client portfolio management, branded owner portals, commission tracking, \
and agent accounts — built for property managers and licensed brokers.',
                 NULL,
                 '{}'::jsonb,
                 'Get Early Access',
                 'waitlist',
                 NOW(),
                 NOW()
             FROM platform_products p
             WHERE p.slug = 'folio-broker'
               AND NOT EXISTS (
                   SELECT 1
                   FROM   product_page_templates t
                   WHERE  t.product_id = p.id
               );",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Remove template first (FK dependency on product row)
        db.execute_unprepared(
            "DELETE FROM product_page_templates
             WHERE product_id = (
                 SELECT id FROM platform_products WHERE slug = 'folio-broker'
             );",
        )
        .await?;

        db.execute_unprepared(
            "DELETE FROM platform_products WHERE slug = 'folio-broker';",
        )
        .await?;

        Ok(())
    }
}
