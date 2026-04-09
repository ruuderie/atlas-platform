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
                    INSERT INTO app_instances (id, tenant_id, app_type)
                    VALUES (
                        v_bwr_anchor_app_id, 
                        v_bwr_tenant_id, 
                        'anchor'
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
