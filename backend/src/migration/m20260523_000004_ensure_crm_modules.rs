use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Ensure the CRM modules are seeded and enabled for all existing app_instances.
        // This guarantees that even if m20260518_000001 was already applied,
        // any app_instance (on dev, UAT, or prod) has these modules active.
        db.execute_unprepared(
            r#"
            INSERT INTO app_instance_module
                (app_instance_id, module_type, display_name, sort_order, is_fixed, is_enabled)
            SELECT
                ai.id,
                m.module_type,
                m.display_name,
                m.sort_order,
                m.is_fixed,
                true
            FROM app_instances ai
            CROSS JOIN (VALUES
                ('CONTACTS',        'Contacts',        50,  false),
                ('LEAD_OPTIONS',    'Lead Options',    70,  false),
                ('LEADS',           'Leads',           160, false)
            ) AS m(module_type, display_name, sort_order, is_fixed)
            ON CONFLICT (app_instance_id, module_type) DO UPDATE
            SET is_enabled = true;
        "#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
