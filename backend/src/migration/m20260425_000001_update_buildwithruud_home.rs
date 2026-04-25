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
                            "RawHtml": {
                                "content": "<div class=\"flex flex-col md:flex-row gap-8 w-full border-b-2 border-outline-variant/30 pb-16 pt-8\"><div class=\"flex-1 space-y-6\"><h1 class=\"text-4xl md:text-5xl font-black text-on-surface uppercase tracking-tighter\">Systems Architecture<br>& Engineering.</h1><p class=\"jetbrains text-sm text-outline tracking-widest uppercase\">I build scalable systems and manage complex cross-border infrastructure.</p></div><div class=\"flex-1 font-mono text-[0.45rem] md:text-[0.55rem] text-primary leading-none opacity-80 select-none overflow-hidden\"><pre>\n   ______      __        ______   __  __\n  / __ \\ \\    / /       / __ \\ \\ / / / /\n / /_/ /\\ \\/\\/ /       / /_/ /\\ V / / /\n/ _, _/  \\    /       / ____/  | | / /\n/_/ |_|    \\/\\/       /_/       |_|/_/\n</pre></div></div>"
                            }
                        },
                        {
                            "FormBuilder": {
                                "form_id": "rev_intake",
                                "title": "",
                                "description": "",
                                "submit_button_text": "Initialize Retrieval",
                                "form_classes": "space-y-8 w-full py-8",
                                "container_classes": "w-full",
                                "button_classes": "w-full bg-secondary text-on-primary py-6 jetbrains font-bold text-sm tracking-[0.2em] uppercase hover:bg-secondary-container hover:text-on-secondary-container transition-colors shadow-lg",
                                "fields": [
                                    { "name": "email", "label": "Registry Email Address", "field_type": "email", "required": true, "placeholder": "user@organization.domain", "custom_classes": "w-full bg-transparent border-none border-b-2 border-outline-variant focus:border-primary focus:ring-0 px-0 py-4 jetbrains text-lg text-on-surface placeholder:text-outline-variant/50 transition-all", "label_classes": "jetbrains text-[0.65rem] uppercase tracking-[0.1em] text-outline text-left block mb-2" }
                                ]
                            }
                        },
                        {
                            "RawHtml": {
                                "content": "<div class=\"grid grid-cols-1 md:grid-cols-3 gap-8 pt-16 border-t-2 border-outline-variant/30\"><div class=\"space-y-3\"><h4 class=\"jetbrains text-xs text-primary font-bold uppercase tracking-widest\">// Core Runtime</h4><ul class=\"jetbrains text-[0.65rem] text-outline space-y-1\"><li>Rust (tokio, axum, leptos)</li><li>Golang (goroutines, grpc)</li><li>Python (FastAPI, pandas)</li></ul></div><div class=\"space-y-3\"><h4 class=\"jetbrains text-xs text-primary font-bold uppercase tracking-widest\">// Infrastructure</h4><ul class=\"jetbrains text-[0.65rem] text-outline space-y-1\"><li>Kubernetes (K3s, Helm)</li><li>Cloudflare (Workers, Tunnels)</li><li>AWS / GCP / Bare Metal</li></ul></div><div class=\"space-y-3\"><h4 class=\"jetbrains text-xs text-primary font-bold uppercase tracking-widest\">// Datastores</h4><ul class=\"jetbrains text-[0.65rem] text-outline space-y-1\"><li>PostgreSQL (PostGIS)</li><li>ClickHouse</li><li>Redis / NATS</li></ul></div></div>"
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
