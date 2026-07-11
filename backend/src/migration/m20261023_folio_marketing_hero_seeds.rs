//! m20261023_folio_marketing_hero_seeds — Folio marketing CMS hero overlays
//!
//! Seeds `hero_payload` for the Folio public marketing products while preserving
//! the hardcoded section stacks until CMS blocks are intentionally authored.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(
            r#"
            WITH seed(app_id, hero, cta_label) AS (
                VALUES
                (
                    'folio',
                    $json$
                    {
                      "eyebrow": "Beta Access Open · US · Canada · Brazil",
                      "headline": "The operating system",
                      "headline_accent": " for the modern real estate investor.",
                      "subhead": "Your rental business runs on gut feel, spreadsheets, and a dozen apps that don't talk. Folio replaces them all.",
                      "proof_items": ["Beta — be one of the first", "Built by a landlord", "US · Canada · Brazil"],
                      "pricing_eyebrow": "For landlords · your own properties",
                      "pricing_heading": "Simple. Transparent. No surprises.",
                      "pricing_subtitle": "Start free. Pay as you grow. Built for landlords managing their own portfolio — no owner-client billing, no trust accounting."
                    }
                    $json$::jsonb,
                    'Join waitlist'
                ),
                (
                    'folio-broker',
                    $json$
                    {
                      "eyebrow": "Beta Access Open · Built for licensed brokers & real estate teams",
                      "headline": "Close more deals.",
                      "headline_accent": " Keep your commission straight.",
                      "subhead": "Folio is the brokerage platform that connects your listing pipeline, client portals, agent accounts, and commission ledger — under your brand, without the enterprise price tag.",
                      "proof_items": ["Multi-client portfolio", "Branded owner portals", "Agent accounts", "Commission tracking"],
                      "pricing_eyebrow": "Pricing",
                      "pricing_heading": "Priced for your team, not per listing.",
                      "pricing_subtitle": "Every seat includes the full platform. Pick the plan that fits your team size — upgrade as you grow."
                    }
                    $json$::jsonb,
                    'Get early access'
                ),
                (
                    'folio-pm',
                    $json$
                    {
                      "eyebrow": "Built for property managers & PMCs · Multi-portfolio edition",
                      "headline": "Manage every portfolio.",
                      "headline_accent": " Impress every owner.",
                      "subhead": "Professional property management runs on owner trust. Folio gives you branded portals, automated statements, trust accounting, and maintenance dispatch — so you run like a firm of 50, even when you're a team of three.",
                      "proof_items": [],
                      "pricing_eyebrow": "Pricing",
                      "pricing_heading": "Pay per portfolio, not per feature.",
                      "pricing_subtitle": "Every plan includes trust accounting, owner portals, and maintenance dispatch. No surprise add-ons."
                    }
                    $json$::jsonb,
                    'Get early access'
                ),
                (
                    'folio-vendor',
                    $json$
                    {
                      "eyebrow": "Free to join · 19 trade categories · US · Canada · Brazil",
                      "headline": "The trade network",
                      "headline_accent": " that finds you work.",
                      "subhead": "Property managers and landlords on Folio dispatch jobs directly to verified tradespeople in their area. You get the job details, accept with one tap, invoice in the app, and get paid in 24 hours. No cold calls. No chasing checks.",
                      "proof_items": [],
                      "pricing_eyebrow": "Pricing",
                      "pricing_heading": "Start free. Upgrade when you're ready.",
                      "pricing_subtitle": "Every vendor gets a marketplace profile and can accept jobs at no cost. Paid plans unlock the tools that help you win more work."
                    }
                    $json$::jsonb,
                    'Get early access'
                )
            )
            UPDATE product_page_templates t
            SET hero_payload = seed.hero,
                cta_label = CASE
                    WHEN NULLIF(TRIM(t.cta_label), '') IS NULL
                      OR t.cta_label IN (
                          'Get Started',
                          'Get started',
                          'Join the Waitlist',
                          'Get Early Access',
                          'Join the Vendor Marketplace'
                      )
                    THEN seed.cta_label
                    ELSE t.cta_label
                END,
                updated_at = NOW()
            FROM platform_products p
            JOIN seed ON seed.app_id = p.slug
            WHERE t.product_id = p.id
              AND (t.hero_payload IS NULL OR t.hero_payload = '{}'::jsonb);
            "#,
        )
        .await?;

        db.execute_unprepared(
            r#"
            WITH seed(app_id, hero) AS (
                VALUES
                (
                    'folio',
                    $json$
                    {
                      "eyebrow": "Beta Access Open · US · Canada · Brazil",
                      "headline": "The operating system",
                      "headline_accent": " for the modern real estate investor.",
                      "subhead": "Your rental business runs on gut feel, spreadsheets, and a dozen apps that don't talk. Folio replaces them all.",
                      "proof_items": ["Beta — be one of the first", "Built by a landlord", "US · Canada · Brazil"],
                      "pricing_eyebrow": "For landlords · your own properties",
                      "pricing_heading": "Simple. Transparent. No surprises.",
                      "pricing_subtitle": "Start free. Pay as you grow. Built for landlords managing their own portfolio — no owner-client billing, no trust accounting."
                    }
                    $json$::jsonb
                ),
                (
                    'folio-broker',
                    $json$
                    {
                      "eyebrow": "Beta Access Open · Built for licensed brokers & real estate teams",
                      "headline": "Close more deals.",
                      "headline_accent": " Keep your commission straight.",
                      "subhead": "Folio is the brokerage platform that connects your listing pipeline, client portals, agent accounts, and commission ledger — under your brand, without the enterprise price tag.",
                      "proof_items": ["Multi-client portfolio", "Branded owner portals", "Agent accounts", "Commission tracking"],
                      "pricing_eyebrow": "Pricing",
                      "pricing_heading": "Priced for your team, not per listing.",
                      "pricing_subtitle": "Every seat includes the full platform. Pick the plan that fits your team size — upgrade as you grow."
                    }
                    $json$::jsonb
                ),
                (
                    'folio-pm',
                    $json$
                    {
                      "eyebrow": "Built for property managers & PMCs · Multi-portfolio edition",
                      "headline": "Manage every portfolio.",
                      "headline_accent": " Impress every owner.",
                      "subhead": "Professional property management runs on owner trust. Folio gives you branded portals, automated statements, trust accounting, and maintenance dispatch — so you run like a firm of 50, even when you're a team of three.",
                      "proof_items": [],
                      "pricing_eyebrow": "Pricing",
                      "pricing_heading": "Pay per portfolio, not per feature.",
                      "pricing_subtitle": "Every plan includes trust accounting, owner portals, and maintenance dispatch. No surprise add-ons."
                    }
                    $json$::jsonb
                ),
                (
                    'folio-vendor',
                    $json$
                    {
                      "eyebrow": "Free to join · 19 trade categories · US · Canada · Brazil",
                      "headline": "The trade network",
                      "headline_accent": " that finds you work.",
                      "subhead": "Property managers and landlords on Folio dispatch jobs directly to verified tradespeople in their area. You get the job details, accept with one tap, invoice in the app, and get paid in 24 hours. No cold calls. No chasing checks.",
                      "proof_items": [],
                      "pricing_eyebrow": "Pricing",
                      "pricing_heading": "Start free. Upgrade when you're ready.",
                      "pricing_subtitle": "Every vendor gets a marketplace profile and can accept jobs at no cost. Paid plans unlock the tools that help you win more work."
                    }
                    $json$::jsonb
                )
            )
            UPDATE app_pages p
            SET hero_payload = seed.hero,
                updated_at = NOW()
            FROM seed
            WHERE p.app_id = seed.app_id
              AND (p.hero_payload IS NULL OR p.hero_payload = '{}'::jsonb);
            "#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
