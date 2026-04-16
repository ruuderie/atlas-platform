use sea_orm_migration::prelude::*;
use serde_json::json;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let sql = r#"
            DO $$
            DECLARE
                v_tenant_id UUID;
            BEGIN
                -- Find buildwithruud tenant
                SELECT id INTO v_tenant_id FROM tenant WHERE name = 'buildwithruud' LIMIT 1;

                IF v_tenant_id IS NOT NULL THEN
                    
                    -- 1. Resume Page
                    IF NOT EXISTS (SELECT 1 FROM app_pages WHERE tenant_id = v_tenant_id AND slug = 'resume') THEN
                        INSERT INTO app_pages (
                            id, tenant_id, slug, title, description, hero_payload, blocks_payload, is_published, created_at, updated_at
                        ) VALUES (
                            gen_random_uuid(), v_tenant_id, 'resume', 'Experience', 'My career journey and operational background.',
                            '{ "hero_title": "Professional Experience", "hero_subtitle": "My career journey and operational background." }'::jsonb,
                            $json$
                            [
                                {
                                    "Timeline": {
                                        "source": "tenant_entries",
                                        "config": {
                                            "filter_category": "work",
                                            "show_date_range": true,
                                            "show_bullets": true,
                                            "layout": "detailed",
                                            "section_title": "Work Experience"
                                        },
                                        "items": []
                                    }
                                }
                            ]
                            $json$,
                            true, NOW(), NOW()
                        );
                    END IF;

                    -- 2. Certifications Page
                    IF NOT EXISTS (SELECT 1 FROM app_pages WHERE tenant_id = v_tenant_id AND slug = 'certifications') THEN
                        INSERT INTO app_pages (
                            id, tenant_id, slug, title, description, hero_payload, blocks_payload, is_published, created_at, updated_at
                        ) VALUES (
                            gen_random_uuid(), v_tenant_id, 'certifications', 'Certifications', 'Technical qualifications and continuous learning.',
                            '{ "hero_title": "Certifications & Credentials", "hero_subtitle": "Technical qualifications and continuous learning." }'::jsonb,
                            $json$
                            [
                                {
                                    "BadgeList": {
                                        "source": "tenant_entries",
                                        "config": {
                                            "filter_category": "certification",
                                            "columns": 3,
                                            "display": "badge",
                                            "section_title": "Active Credentials"
                                        },
                                        "items": []
                                    }
                                }
                            ]
                            $json$,
                            true, NOW(), NOW()
                        );
                    END IF;

                    -- 3. Projects Page
                    IF NOT EXISTS (SELECT 1 FROM app_pages WHERE tenant_id = v_tenant_id AND slug = 'projects') THEN
                        INSERT INTO app_pages (
                            id, tenant_id, slug, title, description, hero_payload, blocks_payload, is_published, created_at, updated_at
                        ) VALUES (
                            gen_random_uuid(), v_tenant_id, 'projects', 'Projects', 'A selection of engineering and architecture projects I have delivered.',
                            '{ "hero_title": "Featured Projects", "hero_subtitle": "A selection of engineering and architecture projects I have delivered." }'::jsonb,
                            $json$
                            [
                                {
                                    "ContentFeed": {
                                        "source": "tenant_entries",
                                        "config": {
                                            "filter_category": "project",
                                            "layout": "cards",
                                            "show_tags": true,
                                            "show_date": true,
                                            "section_title": "Case Studies"
                                        },
                                        "items": []
                                    }
                                }
                            ]
                            $json$,
                            true, NOW(), NOW()
                        );
                    END IF;

                    -- 4. Uses Page (Static Block Example)
                    IF NOT EXISTS (SELECT 1 FROM app_pages WHERE tenant_id = v_tenant_id AND slug = 'uses') THEN
                        INSERT INTO app_pages (
                            id, tenant_id, slug, title, description, hero_payload, blocks_payload, is_published, created_at, updated_at
                        ) VALUES (
                            gen_random_uuid(), v_tenant_id, 'uses', 'Uses', 'My workspace, hardware, and software stack.',
                            '{ "hero_title": "What I Use", "hero_subtitle": "My workspace, hardware, and software stack." }'::jsonb,
                            $json$
                            [
                                {
                                    "Timeline": {
                                        "source": "static",
                                        "config": {
                                            "layout": "compact",
                                            "section_title": "Hardware"
                                        },
                                        "items": [
                                            {
                                                "title": "MacBook Pro 16\"",
                                                "subtitle": "M3 Max, 64GB RAM",
                                                "date_range": null,
                                                "bullets": [],
                                                "metadata": {}
                                            },
                                            {
                                                "title": "UltraFine 5K Display",
                                                "subtitle": "27-inch Dual Setup",
                                                "date_range": null,
                                                "bullets": [],
                                                "metadata": {}
                                            }
                                        ]
                                    }
                                },
                                {
                                    "BadgeList": {
                                        "source": "static",
                                        "config": {
                                            "columns": 4,
                                            "display": "list",
                                            "section_title": "Tech Stack"
                                        },
                                        "items": [
                                            { "title": "Rust", "subtitle": null, "icon_url": null, "metadata": {} },
                                            { "title": "Leptos", "subtitle": null, "icon_url": null, "metadata": {} },
                                            { "title": "PostgreSQL", "subtitle": null, "icon_url": null, "metadata": {} },
                                            { "title": "TailwindCSS", "subtitle": null, "icon_url": null, "metadata": {} }
                                        ]
                                    }
                                }
                            ]
                            $json$,
                            true, NOW(), NOW()
                        );
                    END IF;

                    -- 5. Hire Me Page (FormBuilder Example)
                    IF NOT EXISTS (SELECT 1 FROM app_pages WHERE tenant_id = v_tenant_id AND slug = 'hire-me') THEN
                        INSERT INTO app_pages (
                            id, tenant_id, slug, title, description, hero_payload, blocks_payload, is_published, created_at, updated_at
                        ) VALUES (
                            gen_random_uuid(), v_tenant_id, 'hire-me', 'Hire Me', 'I am available for select contract roles and consulting engagements.',
                            '{ "hero_title": "Work With Me", "hero_subtitle": "I am available for select contract roles and consulting engagements." }'::jsonb,
                            $json$
                            [
                                {
                                    "FormBuilder": {
                                        "form_id": "contact_form",
                                        "title": "Send me a message",
                                        "subtitle": "I will get back to you within 24 hours.",
                                        "cta_text": "Send Message"
                                    }
                                }
                            ]
                            $json$,
                            true, NOW(), NOW()
                        );
                    END IF;

                    -- 6. Consulting Services Page (Grid Example)
                    IF NOT EXISTS (SELECT 1 FROM app_pages WHERE tenant_id = v_tenant_id AND slug = 'consulting') THEN
                        INSERT INTO app_pages (
                            id, tenant_id, slug, title, description, hero_payload, blocks_payload, is_published, created_at, updated_at
                        ) VALUES (
                            gen_random_uuid(), v_tenant_id, 'consulting', 'Consulting & Services', 'Systems design, Rust mentoring, and platform scaling.',
                            '{ "hero_title": "Architecture Consulting", "hero_subtitle": "Systems design, Rust mentoring, and platform scaling." }'::jsonb,
                            $json$
                            [
                                {
                                    "Grid": {
                                        "columns": 2,
                                        "cards": [
                                            {
                                                "title": "Architecture Review",
                                                "description": "Comprehensive review of your SaaS infrastructure.",
                                                "icon": "architecture",
                                                "button_text": "Book Session",
                                                "button_link": "mailto:consulting@ruuderie.com"
                                            },
                                            {
                                                "title": "Rust Codebase Audit",
                                                "description": "Security, performance, and best practices audit for server-side Rust.",
                                                "icon": "code_blocks",
                                                "button_text": "Book Audit",
                                                "button_link": "mailto:consulting@ruuderie.com"
                                            }
                                        ]
                                    }
                                }
                            ]
                            $json$,
                            true, NOW(), NOW()
                        );
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
            DO $$
            DECLARE
                v_tenant_id UUID;
            BEGIN
                SELECT id INTO v_tenant_id FROM tenant WHERE name = 'buildwithruud' LIMIT 1;
                IF v_tenant_id IS NOT NULL THEN
                    DELETE FROM app_pages WHERE tenant_id = v_tenant_id AND slug IN ('resume', 'certifications', 'projects', 'uses', 'hire-me', 'consulting');
                END IF;
            END $$;
        "#;
        db.execute_unprepared(sql).await?;
        Ok(())
    }
}
