//! m20260931_app_pages_locale — Add `locale` column to `app_pages`
//!
//! Adds a `locale` column to `app_pages` so that platform-admin operators
//! can manage locale variants of landing pages (EN / PT / ES / FR) from
//! the GTM Landing Page Builder without deploying code.
//!
//! ## Column spec
//!
//! ```sql
//! locale TEXT NOT NULL DEFAULT 'en'
//!   CHECK (locale IN ('en', 'pt', 'es', 'fr'))
//! ```
//!
//! Default `'en'` — all existing rows silently become English variants,
//! which is correct (every existing page is English content).
//!
//! ## Index
//!
//! A compound index on `(app_id, locale, is_published)` is added so the
//! admin "All Pages" list query can filter by locale with no full-table scan.
//!
//! ## Usage pattern
//!
//! When an operator creates a Portuguese variant of the vendor page:
//!   1. Select "folio-vendor" in the app pill
//!   2. Click "New Page"
//!   3. Set locale = "pt"
//!   4. Slug auto-suggests "folio-vendor-br-pt"
//!   5. Publish — `load_vendor_page("pt")` will now return this variant
//!
//! Idempotent: `ADD COLUMN IF NOT EXISTS` is safe to re-run.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Add locale column with constraint
        db.execute_unprepared(
            "ALTER TABLE app_pages
             ADD COLUMN IF NOT EXISTS locale TEXT NOT NULL DEFAULT 'en'
                 CHECK (locale IN ('en', 'pt', 'es', 'fr'));",
        )
        .await?;

        // Compound index for admin list queries: filter by app + locale + published
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_app_pages_app_locale_published
             ON app_pages (app_id, locale, is_published);",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared("DROP INDEX IF EXISTS idx_app_pages_app_locale_published;")
            .await?;

        db.execute_unprepared("ALTER TABLE app_pages DROP COLUMN IF EXISTS locale;")
            .await?;

        Ok(())
    }
}
