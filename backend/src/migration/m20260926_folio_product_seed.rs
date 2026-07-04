//! m20260926_folio_product_seed — Folio Product Launch Engine Seed
//!
//! Seeds the `folio` platform_product:
//!   1. Sets `launch_mode = 'waitlist'` (was 'draft' from the schema migration)
//!   2. Inserts a minimal `product_page_templates` row, which is required by
//!      `GET /api/pub/products/folio` before any landing-page content can be served.
//!
//! Without this template row the handler returns "product template not configured"
//! (HTTP 404), causing `MarketLandingPage` to render `<NotFound/>` instead of
//! the marketing homepage.
//!
//! Idempotent: both statements are guarded by `WHERE` / `WHERE NOT EXISTS`
//! so re-running is safe.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // 1. Activate the folio product (draft → waitlist)
        db.execute_unprepared(
            "UPDATE platform_products
             SET launch_mode = 'waitlist',
                 updated_at  = NOW()
             WHERE slug = 'folio'
               AND launch_mode = 'draft';",
        )
        .await?;

        // 2. Insert master template (idempotent — skip if one already exists)
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
                 'Folio — Modern Landlord OS',
                 'The only property management platform built for independent landlords. LTR + STR + payments + compliance — one login.',
                 NULL,
                 '{}'::jsonb,
                 'Join the Waitlist',
                 'waitlist',
                 NOW(),
                 NOW()
             FROM platform_products p
             WHERE p.slug = 'folio'
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

        // Revert launch mode (back to draft)
        db.execute_unprepared(
            "UPDATE platform_products
             SET launch_mode = 'draft',
                 updated_at  = NOW()
             WHERE slug = 'folio';",
        )
        .await?;

        // Remove seeded template
        db.execute_unprepared(
            "DELETE FROM product_page_templates
             WHERE product_id = (
                 SELECT id FROM platform_products WHERE slug = 'folio'
             );",
        )
        .await?;

        Ok(())
    }
}
