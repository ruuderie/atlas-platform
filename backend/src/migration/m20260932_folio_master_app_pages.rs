//! m20260932_folio_master_app_pages — Folio CMS master page seeds
//!
//! Creates published `app_pages` master rows for the public Folio marketing
//! surfaces. Payloads intentionally stay empty so the Folio frontend keeps using
//! hardcoded fallback UI until marketing authors real CMS blocks.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(
            "INSERT INTO app_pages (
                 id,
                 tenant_id,
                 app_id,
                 slug,
                 locale,
                 title,
                 description,
                 page_type,
                 hero_payload,
                 blocks_payload,
                 is_published,
                 created_at,
                 updated_at
             )
             SELECT
                 gen_random_uuid(),
                 '00000000-0000-0000-0000-000000000000'::uuid,
                 seed.app_id,
                 'master',
                 'en',
                 seed.title,
                 seed.description,
                 'landing',
                 '{}'::jsonb,
                 '{}'::jsonb,
                 true,
                 NOW(),
                 NOW()
             FROM (VALUES
                 ('folio',        'Folio — Modern Landlord OS',                      'Public path: /'),
                 ('folio-broker', 'Folio for Brokers & Real Estate Agents',          'Public path: /brokers'),
                 ('folio-pm',     'Folio for Property Managers',                     'Public path: /property-managers'),
                 ('folio-vendor', 'Folio for Vendors & Service Providers',           'Public path: /vendors')
             ) AS seed(app_id, title, description)
             WHERE NOT EXISTS (
                 SELECT 1
                 FROM app_pages p
                 WHERE p.app_id = seed.app_id
                   AND p.slug = 'master'
             );",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(
            "DELETE FROM app_pages
             WHERE tenant_id = '00000000-0000-0000-0000-000000000000'::uuid
               AND slug = 'master'
               AND app_id IN ('folio', 'folio-broker', 'folio-pm', 'folio-vendor')
               AND hero_payload = '{}'::jsonb
               AND blocks_payload = '{}'::jsonb;",
        )
        .await?;

        Ok(())
    }
}
