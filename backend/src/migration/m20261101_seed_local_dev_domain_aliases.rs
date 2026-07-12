use sea_orm_migration::prelude::*;

/// Local Compose / atlas-local host aliases so migration-seeded tenants are
/// reachable on `*.localhost` via Caddy (without editing Caddy for each tenant).
/// Safe on server DBs: inserts only if the domain_name does not already exist.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let sql = r##"
            DO $$
            DECLARE
                v_tenant_id UUID;
                v_instance_id UUID;
            BEGIN
                -- buildwithruud → buildwithruud.localhost + anchor.localhost
                SELECT id INTO v_tenant_id FROM tenant WHERE name = 'buildwithruud' LIMIT 1;
                IF v_tenant_id IS NOT NULL THEN
                    SELECT id INTO v_instance_id FROM app_instances
                      WHERE tenant_id = v_tenant_id AND app_type = 'anchor' LIMIT 1;
                    IF v_instance_id IS NOT NULL THEN
                        IF NOT EXISTS (SELECT 1 FROM app_domains WHERE domain_name = 'buildwithruud.localhost') THEN
                            INSERT INTO app_domains (id, app_instance_id, domain_name)
                            VALUES (gen_random_uuid(), v_instance_id, 'buildwithruud.localhost');
                        END IF;
                        IF NOT EXISTS (SELECT 1 FROM app_domains WHERE domain_name = 'anchor.localhost') THEN
                            INSERT INTO app_domains (id, app_instance_id, domain_name)
                            VALUES (gen_random_uuid(), v_instance_id, 'anchor.localhost');
                        END IF;
                    END IF;
                END IF;

                -- oplystusa → oplystusa.localhost
                SELECT id INTO v_tenant_id FROM tenant WHERE name = 'oplystusa' LIMIT 1;
                IF v_tenant_id IS NOT NULL THEN
                    SELECT id INTO v_instance_id FROM app_instances
                      WHERE tenant_id = v_tenant_id AND app_type = 'anchor' LIMIT 1;
                    IF v_instance_id IS NOT NULL THEN
                        IF NOT EXISTS (SELECT 1 FROM app_domains WHERE domain_name = 'oplystusa.localhost') THEN
                            INSERT INTO app_domains (id, app_instance_id, domain_name)
                            VALUES (gen_random_uuid(), v_instance_id, 'oplystusa.localhost');
                        END IF;
                    END IF;
                END IF;

                -- ctbuildpros → directory.network.localhost (if not already present)
                SELECT id INTO v_tenant_id FROM tenant WHERE name = 'ctbuildpros' LIMIT 1;
                IF v_tenant_id IS NOT NULL THEN
                    SELECT id INTO v_instance_id FROM app_instances
                      WHERE tenant_id = v_tenant_id
                        AND app_type IN ('network', 'network_instance', 'Network')
                      LIMIT 1;
                    IF v_instance_id IS NOT NULL THEN
                        IF NOT EXISTS (SELECT 1 FROM app_domains WHERE domain_name = 'directory.network.localhost') THEN
                            INSERT INTO app_domains (id, app_instance_id, domain_name)
                            VALUES (gen_random_uuid(), v_instance_id, 'directory.network.localhost');
                        END IF;
                    END IF;
                END IF;

                -- ruuderie → ruuderie.localhost + folio.localhost
                SELECT id INTO v_tenant_id FROM tenant WHERE name = 'ruuderie' LIMIT 1;
                IF v_tenant_id IS NOT NULL THEN
                    SELECT id INTO v_instance_id FROM app_instances
                      WHERE tenant_id = v_tenant_id AND app_type = 'property_management' LIMIT 1;
                    IF v_instance_id IS NOT NULL THEN
                        IF NOT EXISTS (SELECT 1 FROM app_domains WHERE domain_name = 'ruuderie.localhost') THEN
                            INSERT INTO app_domains (id, app_instance_id, domain_name)
                            VALUES (gen_random_uuid(), v_instance_id, 'ruuderie.localhost');
                        END IF;
                        IF NOT EXISTS (SELECT 1 FROM app_domains WHERE domain_name = 'folio.localhost') THEN
                            INSERT INTO app_domains (id, app_instance_id, domain_name)
                            VALUES (gen_random_uuid(), v_instance_id, 'folio.localhost');
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
        db.execute_unprepared(
            "DELETE FROM app_domains WHERE domain_name IN (
                'buildwithruud.localhost',
                'anchor.localhost',
                'oplystusa.localhost',
                'directory.network.localhost',
                'ruuderie.localhost',
                'folio.localhost'
            );",
        )
        .await?;
        Ok(())
    }
}
