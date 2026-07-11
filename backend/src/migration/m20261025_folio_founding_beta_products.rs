//! m20261025_folio_founding_beta_products - Folio founding + beta campaign seeds
//!
//! Registers the Folio founding member and beta application campaign products so
//! `/founding` and `/beta` can fetch CMS-managed hero copy and campaign metadata
//! from the public product API while preserving their bespoke Leptos form UX.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(
            r#"
            INSERT INTO platform_products (
                id, name, slug, app_slug, status, launch_mode,
                pre_order_enabled, pre_order_currency, pre_order_sold, waitlist_count,
                apex_domain_verified, created_at, updated_at
            )
            VALUES
                (
                    gen_random_uuid(),
                    'Folio - Founding Member Program',
                    'folio-founding',
                    'property_management',
                    'active',
                    'waitlist',
                    false, 'usd', 0, 0, false, NOW(), NOW()
                ),
                (
                    gen_random_uuid(),
                    'Folio - Beta Program',
                    'folio-beta',
                    'property_management',
                    'active',
                    'waitlist',
                    false, 'usd', 0, 0, false, NOW(), NOW()
                )
            ON CONFLICT (slug) DO NOTHING;
            "#,
        )
        .await?;

        db.execute_unprepared(
            r##"
            WITH seed(slug, hero, blocks, meta_title, meta_description, cta_label) AS (
                VALUES
                (
                    'folio-founding',
                    $json$
                    {
                      "eyebrow": "Founding Member Program · Limited Spots",
                      "headline": "Pay once.",
                      "headline_accent": " Use Folio forever.",
                      "subhead": "Lock in lifetime access at a price that will never go up. Pick the license that matches what you do — landlord, broker, property manager, or vendor. No subscription. No renewal. No surprises.",
                      "cta_label": "See founding tiers",
                      "cta_href": "#founding-landlord",
                      "spot_inventory": {
                        "ll-grow": {"total": 500, "taken": 47},
                        "ll-pro": {"total": 250, "taken": 31},
                        "ll-investor": {"total": 100, "taken": 12},
                        "br-solo": {"total": 200, "taken": 8},
                        "br-team": {"total": 100, "taken": 4},
                        "br-firm": {"total": 50, "taken": 1},
                        "pm-starter": {"total": 150, "taken": 7},
                        "pm-growth": {"total": 75, "taken": 3},
                        "vd-pro": {"total": 300, "taken": 19}
                      }
                    }
                    $json$::jsonb,
                    '{}'::jsonb,
                    'Folio Founding Member — Lifetime Access, No Monthly Fees',
                    'Lock in lifetime access to Folio for a one-time payment. Choose the license for your role — landlord, broker, property manager, or vendor. Limited spots. No monthly fees, ever.',
                    'See founding tiers'
                ),
                (
                    'folio-beta',
                    $json$
                    {
                      "eyebrow": "Beta Program · Application Required · Limited Spots",
                      "headline": "Discounted access.",
                      "headline_accent": " Real feedback.",
                      "subhead": "We're opening a curated beta program for active landlords, brokers, property managers, and vendors. If accepted, you get full access to Folio at a discounted rate during the beta period — in exchange for real usage and honest feedback.",
                      "cta_label": "Apply for beta",
                      "cta_href": "#beta-apply"
                    }
                    $json$::jsonb,
                    $json$
                    [
                      {
                        "type": "stats",
                        "items": [
                          {"value": "Discounted", "label": "rate during beta"},
                          {"value": "Curated", "label": "application required"},
                          {"value": "48h", "label": "decision turnaround"}
                        ]
                      }
                    ]
                    $json$::jsonb,
                    'Folio Beta Program — Apply for Discounted Early Access',
                    'Apply to join the Folio beta program. Get discounted access during beta in exchange for real usage and feedback. Limited spots. We review every application.',
                    'Apply now'
                )
            )
            INSERT INTO product_page_templates (
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
                seed.hero,
                seed.blocks,
                seed.meta_title,
                seed.meta_description,
                NULL,
                '{}'::jsonb,
                seed.cta_label,
                'waitlist',
                NOW(),
                NOW()
            FROM platform_products p
            JOIN seed ON seed.slug = p.slug
            WHERE NOT EXISTS (
                SELECT 1
                FROM product_page_templates t
                WHERE t.product_id = p.id
            );
            "##,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(
            "DELETE FROM product_page_templates
             WHERE product_id IN (
                 SELECT id
                 FROM platform_products
                 WHERE slug IN ('folio-founding', 'folio-beta')
             );",
        )
        .await?;

        db.execute_unprepared(
            "DELETE FROM platform_products
             WHERE slug IN ('folio-founding', 'folio-beta');",
        )
        .await?;

        Ok(())
    }
}
