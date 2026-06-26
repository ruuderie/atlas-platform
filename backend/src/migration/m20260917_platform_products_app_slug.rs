//! Migration: add `app_slug` to `platform_products`
//!
//! Binds each product record to the Atlas app binary that owns it,
//! replacing the implicit slug-convention match with an explicit,
//! DB-enforced column. The valid values derive from `AppId::all_db_values()`
//! in `types/gtm.rs` — keep the CHECK constraint in sync when adding apps.
//!
//! The `pub_resolve` handler reads this column and includes `app_slug` in
//! every resolve response so CDN workers can route to the correct app binary.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Add the column with a default so existing rows are non-null immediately.
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                ALTER TABLE platform_products
                  ADD COLUMN IF NOT EXISTS app_slug TEXT
                    NOT NULL
                    DEFAULT 'property_management'
                    CHECK (app_slug IN (
                      'property_management',
                      'anchor',
                      'network_instance',
                      'meridian',
                      'core_platform'
                    ));
                "#,
            )
            .await?;

        // 2. Backfill known product rows from their marketing slug.
        //    New rows set this explicitly via the admin API — no future backfill needed.
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                UPDATE platform_products SET app_slug = 'property_management' WHERE slug = 'folio';
                UPDATE platform_products SET app_slug = 'anchor'              WHERE slug = 'anchor';
                UPDATE platform_products SET app_slug = 'network_instance'    WHERE slug = 'network_instance';
                UPDATE platform_products SET app_slug = 'meridian'            WHERE slug = 'meridian';
                "#,
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("ALTER TABLE platform_products DROP COLUMN IF EXISTS app_slug;")
            .await?;
        Ok(())
    }
}
