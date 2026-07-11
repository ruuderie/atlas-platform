//! m20260929_folio_pm_product_seed — Folio Property Manager Edition Seed
//!
//! Seeds the `folio-pm` platform_product so that:
//!   1. `GET /api/pub/products/folio-pm` returns 200 with `launch_mode = 'waitlist'`
//!   2. `PropertyManagerLandingPage` at `/property-managers` resolves content
//!      instead of hitting an error fallback.
//!   3. The "🏗️ Property Manager" app pill in platform-admin has a backing product record,
//!      enabling A/B testing, tracking pixels, and funnel analytics for the PM audience.
//!   4. `GET /api/admin/landing-pages?app_id=folio-pm` resolves to the correct product scope.
//!
//! # Pricing model
//! Per-portfolio / per-unit — distinct from:
//!   - Landlord (per-door)
//!   - Broker (per-seat)
//!   - Vendor (freemium + marketplace)
//!
//! Tiers: Starter PM ($79/mo) / Growth PM ($199/mo) / Scale PM ($399/mo) / Enterprise (custom)
//!
//! Idempotent: `ON CONFLICT (slug) DO NOTHING` + `WHERE NOT EXISTS` guard.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // ── Step 1: Register folio-pm in platform_products ────────────────────
        //
        // app_slug = 'property_management' — same binary as Folio.
        // launch_mode = 'waitlist' — renders the beta waitlist form.
        db.execute_unprepared(
            "INSERT INTO platform_products (
                 id, name, slug, app_slug, status, launch_mode,
                 pre_order_enabled, pre_order_currency, pre_order_sold, waitlist_count,
                 apex_domain_verified, created_at, updated_at
             )
             VALUES (
                 gen_random_uuid(),
                 'Folio — Property Manager Edition',
                 'folio-pm',
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
        // hero_payload / blocks_payload are {} — the Leptos frontend has hardcoded
        // UI in PropertyManagerLandingPage. These fields exist for future CMS-driven
        // content (PT/ES locale variants, operator-edited copy).
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
                 'Folio for Property Managers — Run Every Portfolio, Bill Every Owner',
                 'Owner portals, trust accounting, maintenance dispatch, and \
multi-portfolio billing in one platform. Start free, scale to hundreds of units.',
                 NULL,
                 '{}'::jsonb,
                 'Get Early Access',
                 'waitlist',
                 NOW(),
                 NOW()
             FROM platform_products p
             WHERE p.slug = 'folio-pm'
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
                 SELECT id FROM platform_products WHERE slug = 'folio-pm'
             );",
        )
        .await?;

        db.execute_unprepared("DELETE FROM platform_products WHERE slug = 'folio-pm';")
            .await?;

        Ok(())
    }
}
