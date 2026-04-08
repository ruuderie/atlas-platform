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
                v_bwr_anchor_app_id UUID;
                v_ct_tenant_id UUID;
                v_ct_network_app_id UUID;
            BEGIN
                -- 1. Identify 'buildwithruud' tenant and its anchor app
                SELECT id INTO v_bwr_tenant_id FROM tenant WHERE name = 'buildwithruud' LIMIT 1;
                IF v_bwr_tenant_id IS NOT NULL THEN
                    SELECT id INTO v_bwr_anchor_app_id FROM app_instances WHERE tenant_id = v_bwr_tenant_id AND app_type = 'anchor' LIMIT 1;
                    
                    -- Inject missing uat and prod domains if they don't exist anywhere
                    IF v_bwr_anchor_app_id IS NOT NULL THEN
                        IF NOT EXISTS (SELECT 1 FROM app_domains WHERE domain_name = 'buildwithruud.com') THEN
                            INSERT INTO app_domains (id, app_instance_id, domain_name) VALUES (gen_random_uuid(), v_bwr_anchor_app_id, 'buildwithruud.com');
                        END IF;
                        IF NOT EXISTS (SELECT 1 FROM app_domains WHERE domain_name = 'uat.buildwithruud.com') THEN
                            INSERT INTO app_domains (id, app_instance_id, domain_name) VALUES (gen_random_uuid(), v_bwr_anchor_app_id, 'uat.buildwithruud.com');
                        END IF;
                    END IF;
                END IF;

                -- 2. Identify 'ctbuildpros' tenant
                SELECT id INTO v_ct_tenant_id FROM tenant WHERE name = 'ctbuildpros' LIMIT 1;
                IF v_ct_tenant_id IS NOT NULL THEN
                    -- Check if 'network' app exists for this tenant
                    SELECT id INTO v_ct_network_app_id FROM app_instances WHERE tenant_id = v_ct_tenant_id AND app_type = 'Network' LIMIT 1;
                    
                    -- If the legacy network app was completely lost or bypassed, generate it now
                    IF v_ct_network_app_id IS NULL THEN
                        v_ct_network_app_id := gen_random_uuid();
                        INSERT INTO app_instances (id, tenant_id, app_type, settings)
                        VALUES (
                            v_ct_network_app_id,
                            v_ct_tenant_id,
                            'Network',
                            '{"site_title": "CT Build Pros", "contact_email": "admin@ctbuildpros.com"}'::jsonb
                        );
                    END IF;
                    
                    -- Bind missing subdomains to the network app
                    IF NOT EXISTS (SELECT 1 FROM app_domains WHERE domain_name = 'directory.localhost') THEN
                        INSERT INTO app_domains (id, app_instance_id, domain_name) VALUES (gen_random_uuid(), v_ct_network_app_id, 'directory.localhost');
                    END IF;
                    IF NOT EXISTS (SELECT 1 FROM app_domains WHERE domain_name = 'ct-build-pros.oply.co') THEN
                        INSERT INTO app_domains (id, app_instance_id, domain_name) VALUES (gen_random_uuid(), v_ct_network_app_id, 'ct-build-pros.oply.co');
                    END IF;
                END IF;
            END $$;
        "##;

        db.execute_unprepared(sql).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Rollback can simply remove the injected domains, though it is usually safer to leave them intact.
        let db = manager.get_connection();
        let query = "DELETE FROM app_domains WHERE domain_name IN ('buildwithruud.com', 'uat.buildwithruud.com', 'directory.localhost', 'ct-build-pros.oply.co');";
        db.execute_unprepared(query).await?;
        Ok(())
    }
}
