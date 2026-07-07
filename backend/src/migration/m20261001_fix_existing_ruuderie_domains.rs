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
                v_ruud_id UUID;
                v_folio_app_id UUID;
                v_network_app_id UUID;
            BEGIN
                -- Find the ruuderie / buildwithruud tenant
                SELECT id INTO v_ruud_id FROM tenant WHERE name ILIKE '%buildwithruud%' OR name ILIKE '%ruud%' OR name ILIKE '%ruuderie%' LIMIT 1;
                
                IF v_ruud_id IS NOT NULL THEN
                    -- Find property_management app instance
                    SELECT id INTO v_folio_app_id FROM app_instances WHERE tenant_id = v_ruud_id AND app_type = 'property_management' LIMIT 1;
                    
                    IF v_folio_app_id IS NOT NULL THEN
                        -- Bind folio1.atlas.oply.co
                        IF NOT EXISTS (SELECT 1 FROM app_domains WHERE domain_name = 'folio1.atlas.oply.co') THEN
                            INSERT INTO app_domains (id, app_instance_id, domain_name) VALUES (gen_random_uuid(), v_folio_app_id, 'folio1.atlas.oply.co');
                        END IF;
                        -- Bind folio.ruuderie.dev.atlas.oply.co
                        IF NOT EXISTS (SELECT 1 FROM app_domains WHERE domain_name = 'folio.ruuderie.dev.atlas.oply.co') THEN
                            INSERT INTO app_domains (id, app_instance_id, domain_name) VALUES (gen_random_uuid(), v_folio_app_id, 'folio.ruuderie.dev.atlas.oply.co');
                        END IF;
                    END IF;

                    -- Find network_instance app instance
                    SELECT id INTO v_network_app_id FROM app_instances WHERE tenant_id = v_ruud_id AND app_type = 'network_instance' LIMIT 1;

                    IF v_network_app_id IS NOT NULL THEN
                        -- Bind network.ruuderie.dev.atlas.oply.co
                        IF NOT EXISTS (SELECT 1 FROM app_domains WHERE domain_name = 'network.ruuderie.dev.atlas.oply.co') THEN
                            INSERT INTO app_domains (id, app_instance_id, domain_name) VALUES (gen_random_uuid(), v_network_app_id, 'network.ruuderie.dev.atlas.oply.co');
                        END IF;
                    END IF;
                END IF;
            END $$;
        "##;

        db.execute_unprepared(sql).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let query = "DELETE FROM app_domains WHERE domain_name IN ('folio1.atlas.oply.co', 'folio.ruuderie.dev.atlas.oply.co', 'network.ruuderie.dev.atlas.oply.co');";
        db.execute_unprepared(query).await?;
        Ok(())
    }
}
