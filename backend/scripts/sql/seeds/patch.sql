DO $$
DECLARE
    v_bwr_tenant_id UUID;
    v_parent_investments UUID := gen_random_uuid();
BEGIN
    SELECT id INTO v_bwr_tenant_id FROM tenant WHERE name ILIKE '%buildwithruud%' LIMIT 1;
    
    -- Insert top-level menus
    INSERT INTO app_menus (id, tenant_id, menu_type, label, href, display_order, is_visible, created_at, updated_at)
    VALUES 
        (gen_random_uuid(), v_bwr_tenant_id, 'header', 'RESUME', '/resume', 10, true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
        (v_parent_investments, v_bwr_tenant_id, 'header', 'INVESTMENTS', NULL, 20, true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
        (gen_random_uuid(), v_bwr_tenant_id, 'header', 'SERVICES', '/services', 30, true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
        (gen_random_uuid(), v_bwr_tenant_id, 'header', 'PROJECTS', '/projects', 40, true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
        (gen_random_uuid(), v_bwr_tenant_id, 'header', 'BLOG', '/blog', 50, true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);
        
    -- Insert child menus for INVESTMENTS
    INSERT INTO app_menus (id, tenant_id, menu_type, label, href, parent_id, display_order, is_visible, created_at, updated_at)
    VALUES 
        (gen_random_uuid(), v_bwr_tenant_id, 'header', 'REAL ESTATE', '/investments/real-estate', v_parent_investments, 1, true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
        (gen_random_uuid(), v_bwr_tenant_id, 'header', 'BITCOIN', '/investments/bitcoin', v_parent_investments, 2, true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);
        
    -- Add default landing page for "real-estate-ventures" slug to support the redirect
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
END $$;
