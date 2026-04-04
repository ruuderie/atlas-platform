use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let sql = r##"
            -- Insert a base tenant for Anchor (assuming one doesn't exist to bind the app to)
            -- Wait, Anchor is an app_instance. It requires a tenant_id. Let's create a generic "Oply Base" tenant if not exists, 
            -- or just seed the Anchor app_instance into the first available tenant, OR create a dedicated Anchor tenant.
            
            DO $$
            DECLARE
                v_tenant_id UUID := gen_random_uuid();
                v_app_instance_id UUID := gen_random_uuid();
            BEGIN
                -- 1. Create a Primary Tenant for Anchor if needed
                INSERT INTO tenant (id, name, description, created_at, updated_at) 
                VALUES (v_tenant_id, 'Oply Anchor Base', 'Base tenant for Anchor application', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);
                
                -- 2. Create the Services App Instance for personal/agency tools
                INSERT INTO app_instances (id, tenant_id, app_type, settings)
                VALUES (
                    v_app_instance_id, 
                    v_tenant_id, 
                    'Services', 
                    '{"site_title": "Ruud Salym Erie", "booking_url": "https://cal.com/ruuderie/15min", "terms_html": "# Terms of Service\n\nPlease review our terms.", "privacy_html": "# Privacy Policy\n\nWe respect your digital privacy.", "github_url": "https://github.com/ruuderie", "x_url": "https://x.com/ruuderie", "linkedin_url": "https://linkedin.com/in/ruuderie"}'::jsonb
                );
                
                -- 3. Create Default Site Settings (Anchor menus and pages)
                -- Add some initial app menus
                INSERT INTO app_menus (id, tenant_id, menu_type, label, href, display_order, is_visible, created_at, updated_at)
                VALUES 
                    (gen_random_uuid(), v_tenant_id, 'header', 'Blog', '/blog', 10, true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                    (gen_random_uuid(), v_tenant_id, 'header', 'About', '/about', 20, true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                    (gen_random_uuid(), v_tenant_id, 'header', 'Contact', '/contact', 30, true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);
                
                -- Add default landing page (like the old landing_pages) for "home" slug
                INSERT INTO app_pages (id, tenant_id, slug, title, description, page_type, hero_payload, blocks_payload, is_published, created_at, updated_at)
                VALUES (
                    gen_random_uuid(), 
                    v_tenant_id, 
                    'home', 
                    'Home', 
                    'Welcome to my site', 
                    'landing',
                    '{"hero_title": "Welcome to Anchor", "hero_subtitle": "This is a dynamic CMS powered page."}'::jsonb,
                    '{"lead_capture_title": "Join Us", "lead_capture_desc": "Sign up for our newsletter", "lead_capture_btn": "Subscribe", "options_json": "{}"}'::jsonb,
                    true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
                );
            END $$;
        "##;
        
        db.execute_unprepared(sql).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let sql = r#"
            DELETE FROM tenant WHERE name = 'Oply Anchor Base';
        "#;
        db.execute_unprepared(sql).await?;
        Ok(())
    }
}
