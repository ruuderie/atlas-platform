use sea_orm_migration::prelude::*;

/// Product Launch Engine — Phase 2 additions:
///
/// 1. `platform_products.apex_domain` — the product's own marketing domain
///    (e.g. "folio.app"). Used by the domain resolver to identify which product
///    a request belongs to without any path prefix.
///
/// 2. `product_page_variants` additions:
///    - `copy_strategy`        — "manual" | "city_inject" | "ai_localize"
///    - `localization_status`  — tracks G-08 AI task lifecycle
///    - `localization_task_id` — FK to atlas_ai_task for the active job
///    - `subdomain_override`   — if set, variant is served at
///                               {subdomain}.{product.apex_domain}
///                               e.g. "miami" → miami.folio.app
///
/// 3. New `product_domain_aliases` table for custom variant domains.
///    Enables: listings.oakwoodpm.com → folio product / miami-fl variant.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // ── platform_products: apex domain ────────────────────────────────────
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE platform_products
                     ADD COLUMN IF NOT EXISTS apex_domain          TEXT UNIQUE,
                     ADD COLUMN IF NOT EXISTS apex_domain_verified BOOLEAN NOT NULL DEFAULT false;

                 CREATE INDEX IF NOT EXISTS idx_platform_products_apex_domain
                     ON platform_products (apex_domain)
                     WHERE apex_domain IS NOT NULL;",
            )
            .await?;

        // ── product_page_variants: localization + subdomain ───────────────────
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE product_page_variants
                     ADD COLUMN IF NOT EXISTS copy_strategy TEXT NOT NULL DEFAULT 'manual'
                         CONSTRAINT chk_variant_copy_strategy
                         CHECK (copy_strategy IN ('manual', 'city_inject', 'ai_localize')),

                     ADD COLUMN IF NOT EXISTS localization_status TEXT NOT NULL DEFAULT 'not_started'
                         CONSTRAINT chk_variant_localization_status
                         CHECK (localization_status IN (
                             'not_started', 'pending', 'complete', 'failed'
                         )),

                     -- FK to atlas_ai_task — the active or last localization job
                     ADD COLUMN IF NOT EXISTS localization_task_id UUID,

                     -- If set, serve this variant at {subdomain_override}.{apex_domain}
                     -- e.g. subdomain_override='miami' → miami.folio.app
                     ADD COLUMN IF NOT EXISTS subdomain_override TEXT;

                 CREATE INDEX IF NOT EXISTS idx_product_page_variants_localization
                     ON product_page_variants (localization_status)
                     WHERE localization_status IN ('pending', 'failed');",
            )
            .await?;

        // ── product_domain_aliases ────────────────────────────────────────────
        // Maps fully-qualified domain names to a product + optional variant.
        // Enables custom domains for variants:
        //   miami.folio.app        → product=folio, variant=miami-fl
        //   listings.oakwoodpm.com → product=folio, variant=miami-fl (white-label)
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE IF NOT EXISTS product_domain_aliases (
                    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
                    product_id      UUID        NOT NULL
                                    REFERENCES platform_products(id) ON DELETE CASCADE,
                    variant_id      UUID        -- null = resolves to master product page
                                    REFERENCES product_page_variants(id) ON DELETE SET NULL,

                    -- Fully-qualified domain name, e.g. 'miami.folio.app'
                    domain          TEXT        NOT NULL,
                    -- Optional path prefix match, e.g. '/miami' on folio.app
                    path_prefix     TEXT,

                    -- DNS verification
                    is_verified     BOOLEAN     NOT NULL DEFAULT false,
                    verified_at     TIMESTAMPTZ,
                    -- Expected TXT record value for ownership verification
                    verification_token TEXT     NOT NULL DEFAULT gen_random_uuid()::TEXT,

                    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),

                    CONSTRAINT uq_product_domain_alias UNIQUE (domain, path_prefix)
                );

                CREATE INDEX IF NOT EXISTS idx_product_domain_aliases_domain
                    ON product_domain_aliases (domain);
                CREATE INDEX IF NOT EXISTS idx_product_domain_aliases_product
                    ON product_domain_aliases (product_id);",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DROP TABLE IF EXISTS product_domain_aliases;
                 DROP INDEX IF EXISTS idx_product_page_variants_localization;
                 ALTER TABLE product_page_variants
                     DROP COLUMN IF EXISTS copy_strategy,
                     DROP COLUMN IF EXISTS localization_status,
                     DROP COLUMN IF EXISTS localization_task_id,
                     DROP COLUMN IF EXISTS subdomain_override;
                 DROP INDEX IF EXISTS idx_platform_products_apex_domain;
                 ALTER TABLE platform_products
                     DROP COLUMN IF EXISTS apex_domain,
                     DROP COLUMN IF EXISTS apex_domain_verified;",
            )
            .await?;
        Ok(())
    }
}
