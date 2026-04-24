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
                v_ruud_id UUID;
                v_oplyst_id UUID;
            BEGIN
                SELECT id INTO v_ruud_id FROM tenant WHERE name ILIKE '%buildwithruud%' LIMIT 1;
                SELECT id INTO v_oplyst_id FROM tenant WHERE name ILIKE '%oplystusa%' LIMIT 1;

                IF v_ruud_id IS NOT NULL THEN
                    
                    -- 1. DELETE consulting page
                    DELETE FROM app_pages WHERE slug = 'consulting' AND tenant_id = v_ruud_id;

                    -- 2. Update real-estate-ventures to use full FormBuilder schema
                    UPDATE app_pages
                    SET blocks_payload = '[
                        {
                            "Hero": {
                                "title": "Real Estate Ventures",
                                "subtitle": "Acquisition, management, and financing of physical assets.",
                                "layout": "standard"
                            }
                        },
                        {
                            "FormBuilder": {
                                "form_id": "rev_intake",
                                "title": "Invest with Us",
                                "description": "Contact us for passive opportunities.",
                                "submit_button_text": "Submit Details",
                                "fields": [
                                    { "name": "first_name", "label": "First Name", "field_type": "text", "required": true, "placeholder": "Jane" },
                                    { "name": "last_name", "label": "Last Name", "field_type": "text", "required": true, "placeholder": "Doe" },
                                    { "name": "email", "label": "Email Address", "field_type": "email", "required": true, "placeholder": "jane@example.com" },
                                    { "name": "interest", "label": "Primary Interest", "field_type": "select", "required": true, "options": ["Passive Investment", "Selling Property", "Joint Venture", "Other"] },
                                    { "name": "details", "label": "Additional Details", "field_type": "textarea", "required": false, "placeholder": "Tell us more..." }
                                ]
                            }
                        }
                    ]'::jsonb
                    WHERE slug = 'real-estate-ventures' AND tenant_id = v_ruud_id;
                    
                    -- 3. Rewrite home page payload to match landing_lead_capture design brief
                    -- Ensure home page exists
                    IF NOT EXISTS (SELECT 1 FROM app_pages WHERE tenant_id = v_ruud_id AND slug = 'home') THEN
                        INSERT INTO app_pages (
                            id, tenant_id, slug, title, description, hero_payload, blocks_payload, is_published, created_at, updated_at
                        ) VALUES (
                            gen_random_uuid(), v_ruud_id, 'home', 'BuildWithRuud', 'Engineering and Architecture.',
                            '{}'::jsonb, '[]'::jsonb, true, NOW(), NOW()
                        );
                    END IF;
                    
                    UPDATE app_pages
                    SET blocks_payload = '[
                        {
                            "Hero": {
                                "title": "Systems Architecture & Engineering",
                                "subtitle": "I build scalable systems and manage complex cross-border infrastructure.",
                                "layout": "centered"
                            }
                        },
                        {
                            "Callout": {
                                "title": "Ready to scale your platform?",
                                "text": "Get in touch for architecture reviews, system scaling, and engineering leadership.",
                                "cta_text": "Contact Me",
                                "cta_link": "/p/resume"
                            }
                        },
                        {
                            "Grid": {
                                "columns": 3,
                                "items": [
                                    { "title": "Backend Systems", "description": "High-performance APIs in Rust & Go.", "icon": "dns" },
                                    { "title": "Infrastructure", "description": "Kubernetes, Cloudflare, AWS.", "icon": "cloud" },
                                    { "title": "Data Engineering", "description": "PostgreSQL & Clickhouse pipelines.", "icon": "database" }
                                ]
                            }
                        }
                    ]'::jsonb
                    WHERE slug = 'home' AND tenant_id = v_ruud_id;

                END IF;

                IF v_oplyst_id IS NOT NULL THEN
                    
                    -- 4. Update apply/cre for OplystUSA
                    UPDATE app_pages
                    SET blocks_payload = '[
                        {
                            "FormBuilder": {
                                "form_id": "cre_application",
                                "title": "Commercial Real Estate Loan Application",
                                "description": "Apply for direct CRE financing. Please provide initial high-level details.",
                                "submit_button_text": "Start Application",
                                "fields": [
                                    { "name": "full_name", "label": "Full Name", "field_type": "text", "required": true },
                                    { "name": "email", "label": "Email Address", "field_type": "email", "required": true },
                                    { "name": "property_type", "label": "Property Type", "field_type": "select", "required": true, "options": ["Multifamily", "Retail", "Office", "Industrial", "Mixed Use", "Other"] },
                                    { "name": "loan_amount", "label": "Requested Loan Amount", "field_type": "text", "required": true, "placeholder": "$500,000" },
                                    { "name": "property_address", "label": "Property Address", "field_type": "text", "required": true },
                                    { "name": "summary", "label": "Deal Summary", "field_type": "textarea", "required": false }
                                ]
                            }
                        }
                    ]'::jsonb
                    WHERE slug = 'apply/cre' AND tenant_id = v_oplyst_id;

                END IF;
            END $$;
        "#;

        db.execute_unprepared(sql).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
