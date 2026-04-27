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
                v_kami_profile_id INTEGER;
            BEGIN
                -- Find buildwithruud tenant
                SELECT id INTO v_bwr_tenant_id FROM tenant WHERE name ILIKE '%buildwithruud%' LIMIT 1;
                
                IF v_bwr_tenant_id IS NOT NULL THEN
                    
                    -- Check if Kami Resume profile already exists
                    IF NOT EXISTS (SELECT 1 FROM entry_collections WHERE tenant_id = v_bwr_tenant_id AND name = 'Kami Resume' LIMIT 1) THEN

                        -- Insert the new Kami Resume profile
                        INSERT INTO entry_collections (tenant_id, name, full_name, objective, is_public, target_role, category_visibility, category_order) 
                        VALUES (
                            v_bwr_tenant_id, 
                            'Kami Resume', 
                            'Ruud Salym Erie', 
                            'A comprehensive log of active systems architecture, smart contract deployments, and client configurations.', 
                            true, 
                            'Systems Architect', 
                            '{"work":true,"education":true,"certification":true,"project":true,"skill":true,"language":true}'::jsonb, 
                            '["work", "education", "certification", "project", "skill", "volunteer", "extracurricular", "language", "hobby"]'::jsonb
                        ) RETURNING id INTO v_kami_profile_id;

                        -- Map entries to the new profile with Kami formatting overrides for work entries
                        -- Enterprise Cloud Solutions
                        INSERT INTO collection_entries (profile_id, entry_id, overrides)
                        SELECT v_kami_profile_id, id, '{"bullets": [
                            "Role: Technical Architect leading two major ERP implementations.",
                            "Actions: Directed developer team, architected Salesforce-to-ERP data migrations and custom MailChimp/DocuSign integration framework.",
                            "Impact: Delivered both projects on schedule; custom integration measurably improved client operational efficiency and forecasting accuracy."
                        ]}'::jsonb
                        FROM tenant_entries WHERE tenant_id = v_bwr_tenant_id AND title = 'Salesforce Architect/Technical Lead' AND category = 'work';

                        -- Swan Bitcoin
                        INSERT INTO collection_entries (profile_id, entry_id, overrides)
                        SELECT v_kami_profile_id, id, '{"bullets": [
                            "Role: Lead Salesforce architect rebuilding application security and data integrity for a fintech platform.",
                            "Actions: Redesigned application architecture, introduced Salesforce Shield, implemented version control, and established structured release process.",
                            "Impact: Reduced deployment errors and improved data compliance; custom Apex/Rust integration tests eliminated manual QA gaps."
                        ]}'::jsonb
                        FROM tenant_entries WHERE tenant_id = v_bwr_tenant_id AND title = 'Salesforce Architect/Engineer' AND category = 'work';

                        -- Zing.It
                        INSERT INTO collection_entries (profile_id, entry_id, overrides)
                        SELECT v_kami_profile_id, id, '{"bullets": [
                            "Role: Frontend engineer and blockchain integration contributor on a Solidity-based platform.",
                            "Actions: Authored TypeScript/React unit tests, implemented SendGrid automation for key platform events, developed Solidity smart contract interactions.",
                            "Impact: Improved UI reliability and user communication; automated email system increased platform engagement measurably."
                        ]}'::jsonb
                        FROM tenant_entries WHERE tenant_id = v_bwr_tenant_id AND title = 'Blockchain Developer' AND category = 'work';

                        -- Grayscale Investments
                        INSERT INTO collection_entries (profile_id, entry_id, overrides)
                        SELECT v_kami_profile_id, id, '{"bullets": [
                            "Role: External Salesforce auditor and implementation consultant.",
                            "Actions: Audited Salesforce systems, consolidated data structure, implemented lead capture integrations, and trained internal team.",
                            "Impact: Streamlined sales process for wealth management team; reduced manual data entry and improved pipeline visibility."
                        ]}'::jsonb
                        FROM tenant_entries WHERE tenant_id = v_bwr_tenant_id AND title = 'Salesforce Consultant' AND category = 'work';

                        -- Principle Studios
                        INSERT INTO collection_entries (profile_id, entry_id, overrides)
                        SELECT v_kami_profile_id, id, '{"bullets": [
                            "Role: Lead architect building a Transportation Management System on Salesforce from scratch.",
                            "Actions: Developed Trigger Framework for end-to-end load management, implemented per-office security model, and built Automatic/Manual Credit Approval flows.",
                            "Impact: Reduced agent credit approval time materially; zero-downtime deployments via SFDX across environments."
                        ]}'::jsonb
                        FROM tenant_entries WHERE tenant_id = v_bwr_tenant_id AND title = 'Salesforce Architect and Senior Developer' AND category = 'work';

                        -- Mercury Healthcare
                        INSERT INTO collection_entries (profile_id, entry_id, overrides)
                        SELECT v_kami_profile_id, id, '{"bullets": [
                            "Role: Implementation architect for ISV product rollouts across Sales, Service, and Marketing Cloud.",
                            "Actions: Implemented SSO for 50+ customer projects (Microsoft, Okta, Salesforce, Appian); configured Tableau/Snowflake for Salesforce analytics; developed SFDX CI process.",
                            "Impact: Standardized feature delivery across 50+ enterprise healthcare accounts including Prisma, Ascension, and Advocate Aurora."
                        ]}'::jsonb
                        FROM tenant_entries WHERE tenant_id = v_bwr_tenant_id AND title = 'Salesforce Lead Engineer / Implementation Architect' AND category = 'work';

                        -- Oplyst International
                        INSERT INTO collection_entries (profile_id, entry_id, overrides)
                        SELECT v_kami_profile_id, id, '{"bullets": [
                            "Role: Founder-level technical architect providing enterprise Salesforce consultation and product engineering.",
                            "Actions: Delivered Einstein Analytics implementations, custom Lightning development across Health/Nonprofit clouds, REST and Platform Event integrations, and built product APIs in Rust, Python, and Deno.",
                            "Impact: 12+ year track record supporting medium-to-large enterprises; current platform stack serves multi-tenant SaaS workloads."
                        ]}'::jsonb
                        FROM tenant_entries WHERE tenant_id = v_bwr_tenant_id AND title = 'Salesforce Technical Architect / Lead Software Engineer' AND category = 'work';

                        -- ZIPARI Inc
                        INSERT INTO collection_entries (profile_id, entry_id, overrides)
                        SELECT v_kami_profile_id, id, '{"bullets": [
                            "Role: Principal Salesforce Architect and engineering manager for 5-person senior team.",
                            "Actions: Provided technical design and API specs for new features, implemented CI/CD via SFDX, designed Lightning Service Cloud call center application, and raised code coverage from 0% to 85% in under 2 months.",
                            "Impact: Team shipped on-time features for all Salesforce product lines; code coverage transformation reduced regression risk across the product suite."
                        ]}'::jsonb
                        FROM tenant_entries WHERE tenant_id = v_bwr_tenant_id AND title = 'CRM Technical Manager / Salesforce Architect' AND category = 'work';

                        -- Evariant Inc
                        INSERT INTO collection_entries (profile_id, entry_id, overrides)
                        SELECT v_kami_profile_id, id, '{"bullets": [
                            "Role: Lead engineer modernizing a healthcare marketing platform used by 50+ US health networks.",
                            "Actions: Migrated 30+ companies to Lightning, rebuilt lead list builder as Lightning components, wrote custom REST API for healthcare case data ingestion.",
                            "Impact: Platform used daily by 50+ health networks; Lightning migration eliminated Classic dependencies and unblocked future feature delivery."
                        ]}'::jsonb
                        FROM tenant_entries WHERE tenant_id = v_bwr_tenant_id AND title = 'Salesforce Architect / Lead Software Engineer' AND category = 'work';

                        -- Add all other entries (projects, certifications, skills) without overrides
                        INSERT INTO collection_entries (profile_id, entry_id)
                        SELECT v_kami_profile_id, id
                        FROM tenant_entries 
                        WHERE tenant_id = v_bwr_tenant_id 
                        AND category != 'work';

                    END IF;
                END IF;
            END $$;
        "##;
        
        db.execute_unprepared(sql).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let sql = r#"
            DO $$
            DECLARE
                v_bwr_tenant_id UUID;
            BEGIN
                SELECT id INTO v_bwr_tenant_id FROM tenant WHERE name ILIKE '%buildwithruud%' LIMIT 1;
                IF v_bwr_tenant_id IS NOT NULL THEN
                    DELETE FROM entry_collections WHERE tenant_id = v_bwr_tenant_id AND name = 'Kami Resume';
                END IF;
            END $$;
        "#;
        db.execute_unprepared(sql).await?;
        Ok(())
    }
}
