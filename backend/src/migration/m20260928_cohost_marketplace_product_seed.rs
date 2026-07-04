//! m20260928_cohost_marketplace_product_seed — Cohost Network Product Launch Engine Seed
//!
//! Seeds the `folio-cohost-market` platform_product so that:
//!   1. `GET /api/pub/products/folio-cohost-market` returns 200 with
//!      `launch_mode = 'waitlist'`.
//!   2. The platform-admin **Landing Pages** panel shows the Cohost Network as
//!      a trackable product with its own waitlist / lead count.
//!   3. UTM campaigns can be scoped to `folio-cohost-market` so traffic to
//!      `/cohost-market` is attributable independently of the main Folio product.
//!
//! Route served by: `CohostMarketplace` in folio at `/cohost-market`.
//! App slug: `property_management` (same Folio binary — no new k8s deployment needed).
//!
//! Without this seed:
//!   - The page still renders (it is fully SSR with static seed data).
//!   - Platform-admin has no product record to attach pixels, A/B tests, or
//!     UTM campaign tracking to.
//!   - `GET /api/pub/products/folio-cohost-market` returns 404.
//!
//! Idempotent: `ON CONFLICT (slug) DO NOTHING` + `WHERE NOT EXISTS` guard
//! make re-running safe.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // ── Step 1: Register folio-cohost-market in platform_products ──────────
        //
        // app_slug = 'property_management' — served by the folio binary.
        // launch_mode = 'waitlist' — Cohost Network is in beta; the page renders
        //   a "Join the waitlist" CTA pointing to /lp#waitlist-wrap.
        // status = 'active' so it appears in the platform-admin Products grid.
        db.execute_unprepared(
            "INSERT INTO platform_products (
                 id, name, slug, app_slug, status, launch_mode,
                 pre_order_enabled, pre_order_currency, pre_order_sold, waitlist_count,
                 apex_domain_verified, created_at, updated_at
             )
             VALUES (
                 gen_random_uuid(),
                 'Folio — Cohost Network',
                 'folio-cohost-market',
                 'property_management',
                 'active',
                 'waitlist',
                 false, 'usd', 0, 0, false, NOW(), NOW()
             )
             ON CONFLICT (slug) DO NOTHING;",
        )
        .await?;

        // ── Step 2: Seed master product_page_template ───────────────────────────
        //
        // The CohostMarketplace Leptos component renders its own hardcoded UI
        // (static seeded data for the beta), so hero_payload / blocks_payload
        // are empty objects — same pattern as folio-broker.
        //
        // meta_title / meta_description mirror the <Title> and <Meta> tags
        // already set inside CohostMarketplace so they are consistent if the
        // GTM builder ever generates a server-driven variant.
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
                 'Cohost Network — Folio',
                 'Find a verified co-host for your Airbnb, or list your property for \
co-host management. Folio''s Cohost Network connects property owners with trusted \
local experts who handle everything — and earn a share of every booking.',
                 NULL,
                 '{}'::jsonb,
                 'Join the Waitlist',
                 'waitlist',
                 NOW(),
                 NOW()
             FROM platform_products p
             WHERE p.slug = 'folio-cohost-market'
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
                 SELECT id FROM platform_products WHERE slug = 'folio-cohost-market'
             );",
        )
        .await?;

        db.execute_unprepared(
            "DELETE FROM platform_products WHERE slug = 'folio-cohost-market';",
        )
        .await?;

        Ok(())
    }
}
