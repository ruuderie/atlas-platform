use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Root cause: m20260412_000003 ran an UPDATE on app_pages before the row existed,
        // so the row was never created. This migration creates the 'home' page for the
        // OplystUSA tenant idempotently, then ensures the blocks_payload is correct.
        //
        // NOTE: We do NOT use ON CONFLICT (tenant_id, slug) because the test schema
        // does not define a UNIQUE constraint on that pair (PG error 42P10).
        // Instead we use an explicit IF NOT EXISTS guard followed by a targeted UPDATE.
        let sql = r#"
            DO $$
            DECLARE
                v_tenant_id UUID;
                v_blocks_json JSONB := '[
                    {
                        "Hero": {
                            "heading": "Direct Lending for Commercial Real Estate Investors",
                            "subheading": "Fast approvals. Flexible terms. Reliable execution.",
                            "primary_cta_text": "Apply Now",
                            "primary_cta_link": "/apply",
                            "background_image": "/assets/hero-bg.webp"
                        }
                    },
                    {
                        "Callout": {
                            "text": "We provide bridge loans, fix-and-flip, and rental portfolio financing nationwide.",
                            "style": "primary"
                        }
                    },
                    {
                        "Grid": {
                            "columns": 3,
                            "items": [
                                {
                                    "title": "Bridge Loans",
                                    "description": "12-24 month terms for acquisitions or refinancing.",
                                    "icon": "account_balance"
                                },
                                {
                                    "title": "Rental Portfolios",
                                    "description": "DSCR loans tailored for landlords.",
                                    "icon": "real_estate_agent"
                                },
                                {
                                    "title": "HOA Lending",
                                    "description": "Capital improvements for condo associations.",
                                    "icon": "apartment"
                                }
                            ]
                        }
                    }
                ]'::jsonb;
            BEGIN
                SELECT id INTO v_tenant_id FROM tenant WHERE slug = 'oplystusa' LIMIT 1;

                IF v_tenant_id IS NOT NULL THEN
                    IF NOT EXISTS (
                        SELECT 1 FROM app_pages
                        WHERE tenant_id = v_tenant_id AND slug = 'home'
                    ) THEN
                        INSERT INTO app_pages (
                            id, tenant_id, slug, title, description,
                            page_type, hero_payload, blocks_payload,
                            is_published, created_at, updated_at
                        )
                        VALUES (
                            gen_random_uuid(), v_tenant_id, 'home',
                            'Commercial Capital - Direct Lending',
                            'Non-bank direct lender providing bridge loans, commercial real estate financing, and hard money lending.',
                            'landing', '{}'::jsonb, v_blocks_json,
                            true, NOW(), NOW()
                        );
                    ELSE
                        UPDATE app_pages
                        SET blocks_payload = v_blocks_json,
                            is_published   = true,
                            updated_at     = NOW()
                        WHERE tenant_id = v_tenant_id AND slug = 'home';
                    END IF;
                END IF;
            END $$;
        "#;

        db.execute_unprepared(sql).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let sql = r#"
            DELETE FROM app_pages
            WHERE tenant_id = (SELECT id FROM tenant WHERE slug = 'oplystusa' LIMIT 1)
              AND slug = 'home';
        "#;

        db.execute_unprepared(sql).await?;
        Ok(())
    }
}
