use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let oplyst_slug = "oplystusa";

        // Bind OplystUSA domains to an Anchor app instance for SSR host-resolution
        let insert_domains = format!(
            r#"
            DO $$
            DECLARE
                v_tenant_id UUID;
                v_anchor_app_id UUID;
            BEGIN
                SELECT id INTO v_tenant_id FROM tenant WHERE slug = '{}' LIMIT 1;
                IF v_tenant_id IS NOT NULL THEN
                    SELECT id INTO v_anchor_app_id FROM app_instances WHERE tenant_id = v_tenant_id AND app_type = 'anchor' LIMIT 1;
                    
                    IF v_anchor_app_id IS NULL THEN
                        v_anchor_app_id := gen_random_uuid();
                        INSERT INTO app_instances (id, tenant_id, app_type, settings)
                        VALUES (v_anchor_app_id, v_tenant_id, 'anchor', '{{}}'::jsonb);
                    END IF;
                    
                    IF NOT EXISTS (SELECT 1 FROM app_domains WHERE domain_name = 'oplystusa.com') THEN
                        INSERT INTO app_domains (id, app_instance_id, domain_name) VALUES (gen_random_uuid(), v_anchor_app_id, 'oplystusa.com');
                    END IF;
                    IF NOT EXISTS (SELECT 1 FROM app_domains WHERE domain_name = 'uat.oplystusa.com') THEN
                        INSERT INTO app_domains (id, app_instance_id, domain_name) VALUES (gen_random_uuid(), v_anchor_app_id, 'uat.oplystusa.com');
                    END IF;
                END IF;
            END $$;
            "#, oplyst_slug
        );
        
        db.execute_unprepared(&insert_domains).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let oplyst_slug = "oplystusa";
        
        let remove_domains = format!(
            r#"
            DO $$
            DECLARE
                v_tenant_id UUID;
                v_anchor_app_id UUID;
            BEGIN
                SELECT id INTO v_tenant_id FROM tenant WHERE slug = '{}' LIMIT 1;
                IF v_tenant_id IS NOT NULL THEN
                    SELECT id INTO v_anchor_app_id FROM app_instances WHERE tenant_id = v_tenant_id AND app_type = 'anchor' LIMIT 1;
                    IF v_anchor_app_id IS NOT NULL THEN
                        DELETE FROM app_domains WHERE app_instance_id = v_anchor_app_id;
                    END IF;
                END IF;
            END $$;
            "#, oplyst_slug
        );

        db.execute_unprepared(&remove_domains).await?;
        Ok(())
    }
}
