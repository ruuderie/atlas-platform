use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        // Register dev.buildwithruud.com in app_domains so the tenant-resolution
        // middleware in anchor/main.rs can resolve the correct TenantContext from
        // the Host header on fresh (incognito) sessions.
        let sql = r##"
            DO $$
            DECLARE
                v_bwr_tenant_id UUID;
                v_bwr_anchor_app_id UUID;
            BEGIN
                SELECT id INTO v_bwr_tenant_id FROM tenant WHERE name = 'buildwithruud' LIMIT 1;
                IF v_bwr_tenant_id IS NOT NULL THEN
                    SELECT id INTO v_bwr_anchor_app_id FROM app_instances WHERE tenant_id = v_bwr_tenant_id AND app_type = 'anchor' LIMIT 1;
                    IF v_bwr_anchor_app_id IS NOT NULL THEN
                        IF NOT EXISTS (SELECT 1 FROM app_domains WHERE domain_name = 'dev.buildwithruud.com') THEN
                            INSERT INTO app_domains (id, app_instance_id, domain_name)
                            VALUES (gen_random_uuid(), v_bwr_anchor_app_id, 'dev.buildwithruud.com');
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
            "DELETE FROM app_domains WHERE domain_name = 'dev.buildwithruud.com';",
        )
        .await?;
        Ok(())
    }
}
