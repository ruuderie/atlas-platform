use sea_orm_migration::prelude::*;

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
                SELECT id INTO v_tenant_id FROM tenant WHERE name ILIKE '%buildwithruud%' LIMIT 1;

                IF v_tenant_id IS NULL THEN
                    RAISE EXCEPTION 'buildwithruud tenant not found — cannot restore consulting page';
                END IF;

                -- Idempotent: only insert if missing (was deleted in m20260417_000003)
                IF NOT EXISTS (SELECT 1 FROM app_pages WHERE tenant_id = v_tenant_id AND slug = 'consulting') THEN
                    INSERT INTO app_pages (
                        id, tenant_id, slug, title, description, hero_payload, blocks_payload, is_published, created_at, updated_at
                    ) VALUES (
                        gen_random_uuid(),
                        v_tenant_id,
                        'consulting',
                        'Consulting & Services',
                        'Systems design, Salesforce implementation, Rust engineering, and platform modernization.',
                        '{}'::jsonb,
                        $json$
                        [
                            {
                                "Hero": {
                                    "title": "Architecture Consulting & Advisory",
                                    "subtitle": "High-stakes platform modernization, SaaS delivery, Salesforce implementation, and rigorous Rust engineering.",
                                    "layout": "minimal"
                                }
                            },
                            {
                                "Grid": {
                                    "section_title": "Core Offerings",
                                    "columns": 2,
                                    "items": [
                                        {
                                            "id": "arch-review",
                                            "title": "Architecture Review",
                                            "description": "Comprehensive review of your SaaS or enterprise infrastructure. I identify failure points, scalability gaps, and security risks, then deliver a prioritized remediation roadmap.",
                                            "icon": "architecture",
                                            "link_url": "mailto:consulting@ruuderie.com"
                                        },
                                        {
                                            "id": "rust-audit",
                                            "title": "Rust Codebase Audit",
                                            "description": "Security, performance, and idiomatic correctness audit for server-side Rust. Covers tokio async patterns, error handling, lifetime issues, and unsafe usage.",
                                            "icon": "code_blocks",
                                            "link_url": "mailto:consulting@ruuderie.com"
                                        },
                                        {
                                            "id": "salesforce-impl",
                                            "title": "Salesforce Implementation",
                                            "description": "Enterprise CRM architecture, custom Flow automation, Apex triggers, LWC development, and cross-platform integrations (REST/SOAP). Experience across Sales Cloud, Service Cloud, and Experience Cloud.",
                                            "icon": "cloud",
                                            "link_url": "mailto:consulting@ruuderie.com"
                                        },
                                        {
                                            "id": "platform-modernization",
                                            "title": "Platform Modernization",
                                            "description": "Legacy-to-cloud migrations, Kubernetes infrastructure (K3s, Helm), CI/CD pipeline design (Woodpecker, GitHub Actions), and multi-tenant SaaS architecture.",
                                            "icon": "rocket_launch",
                                            "link_url": "mailto:consulting@ruuderie.com"
                                        }
                                    ]
                                }
                            },
                            {
                                "Callout": {
                                    "title": "Ready to get started?",
                                    "body": "I take on a limited number of engagements per quarter. Reach out early to discuss scope and availability.",
                                    "button_text": "Schedule a Discovery Call",
                                    "button_link": "/book"
                                }
                            },
                            {
                                "ContentFeed": {
                                    "source": "tenant_entries",
                                    "config": {
                                        "filter_category": "project",
                                        "layout": "cards",
                                        "show_tags": true,
                                        "show_date": true,
                                        "section_title": "Selected Case Studies"
                                    },
                                    "items": []
                                }
                            }
                        ]
                        $json$,
                        true, NOW(), NOW()
                    );
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
            DECLARE v_tenant_id UUID;
            BEGIN
                SELECT id INTO v_tenant_id FROM tenant WHERE name ILIKE '%buildwithruud%' LIMIT 1;
                IF v_tenant_id IS NOT NULL THEN
                    DELETE FROM app_pages WHERE tenant_id = v_tenant_id AND slug = 'consulting';
                END IF;
            END $$;
        "#;
        db.execute_unprepared(sql).await?;
        Ok(())
    }
}
