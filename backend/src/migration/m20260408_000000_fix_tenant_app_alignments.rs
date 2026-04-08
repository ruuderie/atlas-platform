use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let sql = r##"
            DO $$
            DECLARE
                v_bwr_tenant_id UUID;
                v_bwr_anchor_app_id UUID := gen_random_uuid();
                
                v_ct_tenant_id UUID;
                v_ct_anchor_app_id UUID := gen_random_uuid();
                
                v_old_anchor_base UUID;
            BEGIN
                -- 1. Locate or create 'buildwithruud' tenant
                SELECT id INTO v_bwr_tenant_id FROM tenant WHERE name ILIKE '%buildwithruud%' LIMIT 1;
                IF v_bwr_tenant_id IS NULL THEN
                    v_bwr_tenant_id := gen_random_uuid();
                    INSERT INTO tenant (id, name, description, created_at, updated_at) 
                    VALUES (v_bwr_tenant_id, 'buildwithruud', 'Software Portfolio', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);
                END IF;

                -- Check if buildwithruud tenant already has an anchor app
                IF NOT EXISTS (SELECT 1 FROM app_instances WHERE tenant_id = v_bwr_tenant_id AND app_type = 'anchor') THEN
                    INSERT INTO app_instances (id, tenant_id, app_type, settings)
                    VALUES (
                        v_bwr_anchor_app_id, 
                        v_bwr_tenant_id, 
                        'anchor', 
                        '{"current_focus": "AI Agent Swarms (Agentforce / CrewAI)", "status": "Available for Critical Ops", "hero_quote": "Vires in Numeris. Systems architecture is not defined by lines, but by cryptographic proofs and immutable data flows.", "hero_subtitle": "SALESFORCE TECHNICAL ARCHITECT // SPECIALIZING IN ENTERPRISE CLOUD SOLUTIONS, LWC, APEX, AND RUST EXTERNAL MICROSERVICES.", "site_title": "ANCHOR", "lc_title": "Request Tailored CV", "lc_desc": "Input your protocol for a mission-specific credentials package.", "lc_label": "Registry Email Address", "lc_placeholder": "user@organization.domain", "lc_btn": "Initialize Retrieval", "lc_footer": "* Check your email to confirm the request parameters.", "lc_endpoint": "/api/DownloadResume", "status_color": "#ff5449", "webhook_url": "", "admin_email": "", "google_analytics_id": "", "booking_url": "https://cal.com/ruuderie/15min", "terms_html": "# Terms of Service\\n\\nPlease review our terms.", "privacy_html": "# Privacy Policy\\n\\nWe respect your digital privacy.", "github_url": "https://github.com/ruuderie", "x_url": "https://x.com/ruuderie", "linkedin_url": "https://linkedin.com/in/ruuderie", "b2b_enabled": true, "meta_title": "Ruud Salym Erie - Technical Architect", "meta_description": "Technical Architect and Software Engineer specializing in Rust, Salesforce, and high-performance enterprise applications.", "og_image": ""}'::jsonb
                    );
                ELSE
                    SELECT id INTO v_bwr_anchor_app_id FROM app_instances WHERE tenant_id = v_bwr_tenant_id AND app_type = 'anchor' LIMIT 1;
                END IF;
                
                -- Move domains to buildwithruud anchor app
                UPDATE app_domains 
                SET app_instance_id = v_bwr_anchor_app_id
                WHERE domain_name LIKE '%buildwithruud.com';

                -- Add standard navigation menus if they don't exist
                IF NOT EXISTS (SELECT 1 FROM app_menus WHERE tenant_id = v_bwr_tenant_id LIMIT 1) THEN
                    DECLARE
                        v_parent_investments UUID := gen_random_uuid();
                    BEGIN
                        INSERT INTO app_menus (id, tenant_id, menu_type, label, href, display_order, is_visible, created_at, updated_at)
                        VALUES 
                            (gen_random_uuid(), v_bwr_tenant_id, 'header', 'RESUME', '/resume', 10, true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                            (v_parent_investments, v_bwr_tenant_id, 'header', 'INVESTMENTS', NULL, 20, true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                            (gen_random_uuid(), v_bwr_tenant_id, 'header', 'SERVICES', '/services', 30, true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                            (gen_random_uuid(), v_bwr_tenant_id, 'header', 'PROJECTS', '/projects', 40, true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                            (gen_random_uuid(), v_bwr_tenant_id, 'header', 'BLOG', '/blog', 50, true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);
                            
                        INSERT INTO app_menus (id, tenant_id, menu_type, label, href, parent_id, display_order, is_visible, created_at, updated_at)
                        VALUES 
                            (gen_random_uuid(), v_bwr_tenant_id, 'header', 'REAL ESTATE', '/investments/real-estate', v_parent_investments, 1, true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                            (gen_random_uuid(), v_bwr_tenant_id, 'header', 'BITCOIN', '/investments/bitcoin', v_parent_investments, 2, true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);
                    END;
                END IF;

                -- Add default real-estate-ventures page if missing
                IF NOT EXISTS (SELECT 1 FROM app_pages WHERE tenant_id = v_bwr_tenant_id AND slug = 'real-estate-ventures') THEN
                    INSERT INTO app_pages (id, tenant_id, slug, title, description, page_type, hero_payload, blocks_payload, is_published, created_at, updated_at)
                    VALUES (
                        gen_random_uuid(), 
                        v_bwr_tenant_id, 
                        'real-estate-ventures', 
                        'Real Estate Ventures', 
                        'Commercial property and residential portfolios.', 
                        'landing',
                        '{"hero_title": "Real Estate Ventures", "hero_subtitle": "Acquisition, management, and financing of physical assets."}'::jsonb,
                        '{"lead_capture_title": "Invest with Us", "lead_capture_desc": "Contact us for passive opportunities.", "lead_capture_btn": "Submit", "options_json": "{}"}'::jsonb,
                        true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
                    );
                END IF;

                -- 2. Locate or create 'ctbuildpros' tenant (the one that owns directory.localhost or is named CT Build Pros)
                SELECT id INTO v_ct_tenant_id FROM tenant WHERE name ILIKE '%CT Build Pros%' OR name ILIKE 'ctbuildpros' LIMIT 1;
                IF v_ct_tenant_id IS NULL THEN
                    v_ct_tenant_id := gen_random_uuid();
                    INSERT INTO tenant (id, name, description, created_at, updated_at) 
                    VALUES (v_ct_tenant_id, 'ctbuildpros', 'General Contractor Hub', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);
                END IF;

                -- Inject the Secondary Anchor app for ctbuildpros
                IF NOT EXISTS (SELECT 1 FROM app_instances WHERE tenant_id = v_ct_tenant_id AND app_type = 'anchor') THEN
                    INSERT INTO app_instances (id, tenant_id, app_type, settings)
                    VALUES (
                        v_ct_anchor_app_id,
                        v_ct_tenant_id,
                        'anchor',
                        '{"site_title": "CT Build Pros", "booking_url": "https://cal.com/ctbuildpros/consultation", "terms_html": "# Terms of Service\n\nStandard GC terms apply.", "privacy_html": "# Privacy Policy\n\nWe protect your data."}'::jsonb
                    );

                    -- Insert Custom Menus for General Contractor
                    INSERT INTO app_menus (id, tenant_id, menu_type, label, href, display_order, is_visible, created_at, updated_at)
                    VALUES 
                        (gen_random_uuid(), v_ct_tenant_id, 'header', 'Services', '/services', 10, true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                        (gen_random_uuid(), v_ct_tenant_id, 'header', 'Portfolio', '/portfolio', 20, true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                        (gen_random_uuid(), v_ct_tenant_id, 'header', 'Contact Us', '/contact', 30, true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);

                    -- Insert General Contractor Landing Page
                    INSERT INTO app_pages (id, tenant_id, slug, title, description, page_type, hero_payload, blocks_payload, is_published, created_at, updated_at)
                    VALUES (
                        gen_random_uuid(), 
                        v_ct_tenant_id, 
                        'home', 
                        'CT Build Pros Home', 
                        'Top rated remodeling contractors in Connecticut.', 
                        'landing',
                        '{"hero_title": "Building Connecticut''s Future", "hero_subtitle": "Premium renovations and general contracting services with a verified 4.9 star network rating."}'::jsonb,
                        '{"lead_capture_title": "Get a Free Estimate", "lead_capture_desc": "Schedule an on-site consultation with our specialists.", "lead_capture_btn": "Request Quote", "options_json": "{}"}'::jsonb,
                        true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
                    );
                END IF;
                
                -- Optional cleanup: if "Oply Anchor Base" exists and is empty, delete it
                SELECT id INTO v_old_anchor_base FROM tenant WHERE name = 'Oply Anchor Base' LIMIT 1;
                IF v_old_anchor_base IS NOT NULL THEN
                    DELETE FROM tenant WHERE id = v_old_anchor_base AND id != v_bwr_tenant_id AND id != v_ct_tenant_id;
                END IF;

            END $$;
        "##;
        
        db.execute_unprepared(sql).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let sql = r#"
            -- Downgrade logic if needed
        "#;
        db.execute_unprepared(sql).await?;
        Ok(())
    }
}
