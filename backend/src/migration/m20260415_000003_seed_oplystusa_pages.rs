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
                v_programs_root_id UUID;
                v_assets_root_id UUID;
                v_partners_root_id UUID;
                v_cre_form_id UUID;
                v_hoa_form_id UUID;
            BEGIN
                SELECT id INTO v_tenant_id FROM tenant WHERE slug = 'oplystusa' LIMIT 1;

                IF v_tenant_id IS NOT NULL THEN
                    
                    -- Fetch the Form Schemas that were already seeded
                    SELECT id INTO v_cre_form_id FROM form_schemas WHERE tenant_id = v_tenant_id AND slug = 'cre-application' LIMIT 1;
                    SELECT id INTO v_hoa_form_id FROM form_schemas WHERE tenant_id = v_tenant_id AND slug = 'hoa-condo-application' LIMIT 1;

                    -- 1. Insert Apply Page (CRE)
                    IF NOT EXISTS (SELECT 1 FROM app_pages WHERE tenant_id = v_tenant_id AND slug = 'apply/cre') THEN
                        INSERT INTO app_pages (
                            id, tenant_id, slug, title, description, page_type, hero_payload, blocks_payload, is_published, created_at, updated_at
                        ) VALUES (
                            gen_random_uuid(), v_tenant_id, 'apply/cre', 'Commercial Real Estate Loan Application', 'Apply for direct CRE financing.', 'landing', '{}'::jsonb, 
                            jsonb_build_array(
                                jsonb_build_object('FormBuilder', jsonb_build_object(
                                    'title', 'Commercial Real Estate Loan Application',
                                    'description', 'Fill out the form below to apply for bridge or rental portfolio financing.',
                                    'schema_id', COALESCE(v_cre_form_id, gen_random_uuid())
                                ))
                            ), 
                            true, NOW(), NOW()
                        );
                    END IF;
                    
                    -- 2. Insert Apply Page (HOA)
                    IF NOT EXISTS (SELECT 1 FROM app_pages WHERE tenant_id = v_tenant_id AND slug = 'apply/hoa') THEN
                        INSERT INTO app_pages (
                            id, tenant_id, slug, title, description, page_type, hero_payload, blocks_payload, is_published, created_at, updated_at
                        ) VALUES (
                            gen_random_uuid(), v_tenant_id, 'apply/hoa', 'HOA Loan Application', 'Apply for HOA capital improvements.', 'landing', '{}'::jsonb, 
                            jsonb_build_array(
                                jsonb_build_object('FormBuilder', jsonb_build_object(
                                    'title', 'HOA & Condo Association Loan Application',
                                    'description', 'Unsecured lending for condo associations to fund capital improvements.',
                                    'schema_id', COALESCE(v_hoa_form_id, gen_random_uuid())
                                ))
                            ), 
                            true, NOW(), NOW()
                        );
                    END IF;

                    -- 3. Insert Programs Pages
                    IF NOT EXISTS (SELECT 1 FROM app_pages WHERE tenant_id = v_tenant_id AND slug = 'programs/bridge-loans') THEN
                        INSERT INTO app_pages (
                            id, tenant_id, slug, title, description, page_type, hero_payload, blocks_payload, is_published, created_at, updated_at
                        ) VALUES (
                            gen_random_uuid(), v_tenant_id, 'programs/bridge-loans', 'Bridge Loans', '12-24 month terms for acquisitions or refinancing.', 'landing', '{}'::jsonb, 
                            jsonb_build_array(
                                jsonb_build_object('Hero', jsonb_build_object(
                                    'heading', 'Bridge Loans', 
                                    'subheading', 'Fast capital for your acquisitions and refi.', 
                                    'primary_cta_text', 'Apply Now', 
                                    'primary_cta_link', '/p/apply/cre', 
                                    'background_image', '/assets/hero-bg.webp'
                                ))
                            ), 
                            true, NOW(), NOW()
                        );
                    END IF;

                    IF NOT EXISTS (SELECT 1 FROM app_pages WHERE tenant_id = v_tenant_id AND slug = 'programs/rental-portfolios') THEN
                        INSERT INTO app_pages (
                            id, tenant_id, slug, title, description, page_type, hero_payload, blocks_payload, is_published, created_at, updated_at
                        ) VALUES (
                            gen_random_uuid(), v_tenant_id, 'programs/rental-portfolios', 'Rental Portfolios (DSCR)', 'DSCR loans tailored for landlords.', 'landing', '{}'::jsonb, 
                            jsonb_build_array(
                                jsonb_build_object('Hero', jsonb_build_object(
                                    'heading', 'DSCR Rental Portfolios', 
                                    'subheading', 'Scale your rental property portfolio without personal DTI limits.', 
                                    'primary_cta_text', 'Apply Now', 
                                    'primary_cta_link', '/p/apply/cre', 
                                    'background_image', '/assets/hero-bg.webp'
                                ))
                            ), 
                            true, NOW(), NOW()
                        );
                    END IF;
                    
                    -- 4. Broker Partners Page
                    IF NOT EXISTS (SELECT 1 FROM app_pages WHERE tenant_id = v_tenant_id AND slug = 'partners/brokers') THEN
                        INSERT INTO app_pages (
                            id, tenant_id, slug, title, description, page_type, hero_payload, blocks_payload, is_published, created_at, updated_at
                        ) VALUES (
                            gen_random_uuid(), v_tenant_id, 'partners/brokers', 'ISO & Broker Program', 'Partner with us as an ISO or Broker.', 'landing', '{}'::jsonb, 
                            jsonb_build_array(
                                jsonb_build_object('Hero', jsonb_build_object(
                                    'heading', 'Broker Partner Program', 
                                    'subheading', 'Earn high commissions with fast underwriting direct from a private lender.', 
                                    'primary_cta_text', 'Contact Us', 
                                    'primary_cta_link', '/contact', 
                                    'background_image', '/assets/hero-bg.webp'
                                ))
                            ), 
                            true, NOW(), NOW()
                        );
                    END IF;

                    -- 5. Seed Nav Items (Hierarchical Menu)
                    -- First Check if already seeded to prevent duplication
                    IF NOT EXISTS (SELECT 1 FROM app_menus WHERE tenant_id = v_tenant_id AND label = 'Programs' AND menu_type = 'header') THEN
                        v_programs_root_id := gen_random_uuid();
                        INSERT INTO app_menus (id, tenant_id, menu_type, label, href, parent_id, display_order, is_visible, created_at, updated_at)
                        VALUES (v_programs_root_id, v_tenant_id, 'header', 'Programs', NULL, NULL, 1, true, NOW(), NOW());
                        
                        INSERT INTO app_menus (id, tenant_id, menu_type, label, href, parent_id, display_order, is_visible, created_at, updated_at)
                        VALUES 
                            (gen_random_uuid(), v_tenant_id, 'header', 'Bridge Loans', '/p/programs/bridge-loans', v_programs_root_id, 1, true, NOW(), NOW()),
                            (gen_random_uuid(), v_tenant_id, 'header', 'Rental Portfolios', '/p/programs/rental-portfolios', v_programs_root_id, 2, true, NOW(), NOW()),
                            (gen_random_uuid(), v_tenant_id, 'header', 'HOA Capital', '/p/apply/hoa', v_programs_root_id, 3, true, NOW(), NOW());
                    END IF;

                    IF NOT EXISTS (SELECT 1 FROM app_menus WHERE tenant_id = v_tenant_id AND label = 'Partner With Us' AND menu_type = 'header') THEN
                        v_partners_root_id := gen_random_uuid();
                        INSERT INTO app_menus (id, tenant_id, menu_type, label, href, parent_id, display_order, is_visible, created_at, updated_at)
                        VALUES (v_partners_root_id, v_tenant_id, 'header', 'Partner With Us', NULL, NULL, 2, true, NOW(), NOW());
                        
                        INSERT INTO app_menus (id, tenant_id, menu_type, label, href, parent_id, display_order, is_visible, created_at, updated_at)
                        VALUES 
                            (gen_random_uuid(), v_tenant_id, 'header', 'Brokers & ISOs', '/p/partners/brokers', v_partners_root_id, 1, true, NOW(), NOW());
                    END IF;

                    IF NOT EXISTS (SELECT 1 FROM app_menus WHERE tenant_id = v_tenant_id AND label = 'Apply' AND menu_type = 'header') THEN
                        INSERT INTO app_menus (id, tenant_id, menu_type, label, href, parent_id, display_order, is_visible, created_at, updated_at)
                        VALUES (gen_random_uuid(), v_tenant_id, 'header', 'Apply', '/p/apply/cre', NULL, 99, true, NOW(), NOW());
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
                SELECT id INTO v_tenant_id FROM tenant WHERE slug = 'oplystusa' LIMIT 1;
                IF v_tenant_id IS NOT NULL THEN
                    DELETE FROM app_menus WHERE tenant_id = v_tenant_id;
                    DELETE FROM app_pages WHERE tenant_id = v_tenant_id AND slug IN (
                        'apply/cre',
                        'apply/hoa',
                        'programs/bridge-loans',
                        'programs/rental-portfolios',
                        'partners/brokers'
                    );
                END IF;
            END $$;
        "#;
        db.execute_unprepared(sql).await?;
        Ok(())
    }
}
