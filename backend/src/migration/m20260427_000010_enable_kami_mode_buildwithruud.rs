use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Merge kami_mode: true into the design_config sub-object inside app_instances.settings
        // for the buildwithruud tenant only. The jsonb_set path creates the key if missing.
        // This is fully idempotent — running again is a no-op because the value is already true.
        let sql = r#"
            UPDATE app_instances
            SET settings = jsonb_set(
                COALESCE(settings, '{}'::jsonb),
                '{design_config,kami_mode}',
                'true'::jsonb,
                true  -- create_missing
            )
            WHERE tenant_id = (
                SELECT id FROM tenant WHERE name ILIKE '%buildwithruud%' LIMIT 1
            )
            AND app_type = 'anchor';
        "#;
        db.execute_unprepared(sql).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let sql = r#"
            UPDATE app_instances
            SET settings = jsonb_set(
                COALESCE(settings, '{}'::jsonb),
                '{design_config,kami_mode}',
                'false'::jsonb,
                true
            )
            WHERE tenant_id = (
                SELECT id FROM tenant WHERE name ILIKE '%buildwithruud%' LIMIT 1
            )
            AND app_type = 'anchor';
        "#;
        db.execute_unprepared(sql).await?;

        Ok(())
    }
}
