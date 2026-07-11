//! m20260930_folio_vendor_product_seed — Folio Vendor Marketplace Seed
//!
//! Seeds the `folio-vendor` platform_product so that:
//!   1. `GET /api/pub/products/folio-vendor` returns 200 with `launch_mode = 'waitlist'`
//!   2. `VendorLandingPage` at `/vendors` resolves content instead of error fallback.
//!   3. The "🔧 Vendor Page" app pill in platform-admin has a backing product record,
//!      enabling A/B testing, tracking pixels, and funnel analytics for the vendor audience.
//!   4. `GET /api/admin/landing-pages?app_id=folio-vendor` resolves to the correct scope.
//!
//! # Pricing model
//! Freemium + marketplace — distinct from other Folio products:
//!   - Basic (Free): marketplace listing, accept jobs, platform invoicing
//!   - Pro Vendor ($29/mo): priority placement, automated invoicing, job analytics
//!   - Business ($79/mo): multi-tech accounts, branded profile, 0% platform fee
//!
//! # Trade categories
//! Vendors sign up with a specific trade (19 categories) and service area.
//! The `trade` and `service_area` fields are collected at signup and stored
//! with the atlas_lead record (source = 'vendor-page').
//!
//! # Locale variants (future)
//! When PT/ES translations are ready, seed locale variants here:
//!   folio-vendor-br-pt  (Brazilian Portuguese)
//!   folio-vendor-latam-es (LATAM Spanish)
//!
//! Idempotent: `ON CONFLICT (slug) DO NOTHING` + `WHERE NOT EXISTS` guard.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // ── Step 1: Register folio-vendor in platform_products ────────────────
        //
        // app_slug = 'property_management' — same binary as Folio.
        // launch_mode = 'waitlist' — vendor signup form is live but marketplace
        // dispatch is not yet open; vendors join the founding vendor waitlist.
        db.execute_unprepared(
            "INSERT INTO platform_products (
                 id, name, slug, app_slug, status, launch_mode,
                 pre_order_enabled, pre_order_currency, pre_order_sold, waitlist_count,
                 apex_domain_verified, created_at, updated_at
             )
             VALUES (
                 gen_random_uuid(),
                 'Folio — Vendor Marketplace',
                 'folio-vendor',
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
        // UI in VendorLandingPage (trade category signup, G-27 preview, etc.).
        // These fields exist for future CMS-driven content and locale variants.
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
                 'Folio for Vendors — Get Dispatched, Get Paid, No Chasing',
                 'Folio connects tradespeople and contractors to landlords and property \
managers. Get dispatched jobs, send invoices, collect payment, and grow your \
service business — all on one platform.',
                 NULL,
                 '{}'::jsonb,
                 'Join the Vendor Marketplace',
                 'waitlist',
                 NOW(),
                 NOW()
             FROM platform_products p
             WHERE p.slug = 'folio-vendor'
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
                 SELECT id FROM platform_products WHERE slug = 'folio-vendor'
             );",
        )
        .await?;

        db.execute_unprepared("DELETE FROM platform_products WHERE slug = 'folio-vendor';")
            .await?;

        Ok(())
    }
}
