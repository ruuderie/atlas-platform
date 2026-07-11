//! m20261026_folio_vendor_trade_categories — Folio vendor CMS trade categories
//!
//! Adds the `trade_categories` overlay block consumed by the vendor landing page.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(
            r##"
            WITH seed(blocks) AS (
                SELECT $json$
                [
                  {
                    "type": "trade_categories",
                    "items": [
                      {"key": "cleaning", "label": "🧹 Cleaning"},
                      {"key": "handyman", "label": "🔧 Handyman"},
                      {"key": "plumbing", "label": "🚿 Plumbing"},
                      {"key": "electrical", "label": "⚡ Electrical"},
                      {"key": "hvac", "label": "❄️ HVAC"},
                      {"key": "painting", "label": "🖌️ Painting"},
                      {"key": "landscaping", "label": "🌿 Landscaping"},
                      {"key": "roofing", "label": "🏠 Roofing"},
                      {"key": "flooring", "label": "🪵 Flooring"},
                      {"key": "pest-control", "label": "🐛 Pest Control"},
                      {"key": "appliance", "label": "🛠️ Appliances"},
                      {"key": "locksmith", "label": "🔐 Locksmith"},
                      {"key": "inspection", "label": "🔍 Inspection"},
                      {"key": "movers", "label": "📦 Movers"},
                      {"key": "junk-removal", "label": "🗑️ Junk Removal"},
                      {"key": "pool-spa", "label": "🏊 Pool & Spa"},
                      {"key": "security", "label": "📷 Security"},
                      {"key": "solar", "label": "☀️ Solar"},
                      {"key": "general", "label": "🏗️ General Contractor"}
                    ]
                  }
                ]
                $json$::jsonb
            )
            UPDATE product_page_templates t
            SET blocks_payload = CASE
                    WHEN t.blocks_payload IS NULL
                      OR t.blocks_payload = '{}'::jsonb
                      OR t.blocks_payload = '[]'::jsonb
                        THEN seed.blocks
                    WHEN jsonb_typeof(t.blocks_payload) = 'array'
                        THEN t.blocks_payload || seed.blocks
                    WHEN jsonb_typeof(t.blocks_payload) = 'object'
                        THEN jsonb_set(
                            t.blocks_payload,
                            '{blocks}',
                            COALESCE(t.blocks_payload->'blocks', '[]'::jsonb) || seed.blocks,
                            true
                        )
                    ELSE t.blocks_payload
                END,
                updated_at = NOW()
            FROM platform_products p, seed
            WHERE t.product_id = p.id
              AND p.slug = 'folio-vendor'
              AND NOT EXISTS (
                  SELECT 1
                  FROM jsonb_array_elements(
                      CASE
                        WHEN jsonb_typeof(COALESCE(t.blocks_payload, '[]'::jsonb)) = 'array'
                            THEN COALESCE(t.blocks_payload, '[]'::jsonb)
                        WHEN jsonb_typeof(COALESCE(t.blocks_payload, '{}'::jsonb)) = 'object'
                            THEN COALESCE(t.blocks_payload->'blocks', '[]'::jsonb)
                        ELSE '[]'::jsonb
                      END
                  ) AS block
                  WHERE block->>'type' = 'trade_categories'
              );

            WITH seed(blocks) AS (
                SELECT $json$
                [
                  {
                    "type": "trade_categories",
                    "items": [
                      {"key": "cleaning", "label": "🧹 Cleaning"},
                      {"key": "handyman", "label": "🔧 Handyman"},
                      {"key": "plumbing", "label": "🚿 Plumbing"},
                      {"key": "electrical", "label": "⚡ Electrical"},
                      {"key": "hvac", "label": "❄️ HVAC"},
                      {"key": "painting", "label": "🖌️ Painting"},
                      {"key": "landscaping", "label": "🌿 Landscaping"},
                      {"key": "roofing", "label": "🏠 Roofing"},
                      {"key": "flooring", "label": "🪵 Flooring"},
                      {"key": "pest-control", "label": "🐛 Pest Control"},
                      {"key": "appliance", "label": "🛠️ Appliances"},
                      {"key": "locksmith", "label": "🔐 Locksmith"},
                      {"key": "inspection", "label": "🔍 Inspection"},
                      {"key": "movers", "label": "📦 Movers"},
                      {"key": "junk-removal", "label": "🗑️ Junk Removal"},
                      {"key": "pool-spa", "label": "🏊 Pool & Spa"},
                      {"key": "security", "label": "📷 Security"},
                      {"key": "solar", "label": "☀️ Solar"},
                      {"key": "general", "label": "🏗️ General Contractor"}
                    ]
                  }
                ]
                $json$::jsonb
            )
            UPDATE app_pages p
            SET blocks_payload = CASE
                    WHEN p.blocks_payload IS NULL
                      OR p.blocks_payload = '{}'::jsonb
                      OR p.blocks_payload = '[]'::jsonb
                        THEN seed.blocks
                    WHEN jsonb_typeof(p.blocks_payload) = 'array'
                        THEN p.blocks_payload || seed.blocks
                    WHEN jsonb_typeof(p.blocks_payload) = 'object'
                        THEN jsonb_set(
                            p.blocks_payload,
                            '{blocks}',
                            COALESCE(p.blocks_payload->'blocks', '[]'::jsonb) || seed.blocks,
                            true
                        )
                    ELSE p.blocks_payload
                END,
                updated_at = NOW()
            FROM seed
            WHERE p.app_id = 'folio-vendor'
              AND p.slug = 'master'
              AND NOT EXISTS (
                  SELECT 1
                  FROM jsonb_array_elements(
                      CASE
                        WHEN jsonb_typeof(COALESCE(p.blocks_payload, '[]'::jsonb)) = 'array'
                            THEN COALESCE(p.blocks_payload, '[]'::jsonb)
                        WHEN jsonb_typeof(COALESCE(p.blocks_payload, '{}'::jsonb)) = 'object'
                            THEN COALESCE(p.blocks_payload->'blocks', '[]'::jsonb)
                        ELSE '[]'::jsonb
                      END
                  ) AS block
                  WHERE block->>'type' = 'trade_categories'
              );
            "##,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
