use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // The buildwithruud home page was created with the old flat hero_payload format:
        //   hero_payload: {"hero_title": "...", "hero_subtitle": "..."}
        //   blocks_payload: {"lead_capture_title": "...", ...}
        //
        // The new DynamicHomeLanding reads blocks_payload as a block array.
        // This migration upgrades the home page to block format so the homepage renders.
        // It also ensures the row exists if it was somehow missing.
        let sql = r#"
            DO $$
            DECLARE
                v_tenant_id UUID;
                v_blocks_json JSONB := '[
                    {
                        "Hero": {
                            "heading": "Systems Architect. Engineer. Builder.",
                            "subheading": "Rust backends, Salesforce platforms, and distributed infrastructure for enterprises and startups.",
                            "primary_cta_text": "View Work",
                            "primary_cta_link": "/resume",
                            "background_image": ""
                        }
                    },
                    {
                        "Grid": {
                            "columns": 3,
                            "items": [
                                {
                                    "title": "Salesforce Architecture",
                                    "description": "Multi-org deployments, platform events, Apex engineering, and complex data models.",
                                    "icon": "cloud"
                                },
                                {
                                    "title": "Rust Systems",
                                    "description": "High-performance backends, API platforms, and blockchain integrations built in Rust.",
                                    "icon": "memory"
                                },
                                {
                                    "title": "Infrastructure",
                                    "description": "K8s clusters, NixOS hosts, CI/CD pipelines, and multi-tenant SaaS platforms.",
                                    "icon": "dns"
                                }
                            ]
                        }
                    }
                ]'::jsonb;
            BEGIN
                -- Locate buildwithruud tenant
                SELECT id INTO v_tenant_id
                FROM tenant
                WHERE name ILIKE '%buildwithruud%'
                   OR name ILIKE '%ruud%'
                   OR name ILIKE '%ruuderie%'
                LIMIT 1;

                IF v_tenant_id IS NULL THEN
                    -- Fallback: find by domain
                    SELECT t.id INTO v_tenant_id
                    FROM tenant t
                    JOIN app_instances ai ON ai.tenant_id = t.id
                    JOIN app_domains ad ON ad.app_instance_id = ai.id
                    WHERE ad.domain_name ILIKE '%buildwithruud%'
                    LIMIT 1;
                END IF;

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
                            'Ruud Salym Erie — Technical Architect',
                            'Systems architect and software engineer specializing in Rust, Salesforce, and high-performance enterprise applications.',
                            'landing', '{}'::jsonb, v_blocks_json,
                            true, NOW(), NOW()
                        );
                    ELSE
                        -- Upgrade existing home page to block format
                        UPDATE app_pages
                        SET blocks_payload = v_blocks_json,
                            hero_payload   = '{}'::jsonb,
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

        // Restore the old flat format for buildwithruud home page on rollback
        let sql = r#"
            UPDATE app_pages
            SET blocks_payload = '{"lead_capture_title": "Join Us", "lead_capture_desc": "Sign up for our newsletter", "lead_capture_btn": "Subscribe", "options_json": "{}"}'::jsonb,
                hero_payload   = '{"hero_title": "Systems Architect. Engineer. Builder.", "hero_subtitle": "Rust backends, Salesforce platforms, and distributed infrastructure."}'::jsonb,
                updated_at     = NOW()
            WHERE slug = 'home'
              AND tenant_id IN (
                SELECT t.id FROM tenant t
                WHERE t.name ILIKE '%buildwithruud%' OR t.name ILIKE '%ruud%'
              );
        "#;

        db.execute_unprepared(sql).await?;
        Ok(())
    }
}
