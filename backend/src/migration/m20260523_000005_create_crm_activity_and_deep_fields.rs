use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // 1. Extend the contact table with middle_name (address is already cleanly modeled as JSONB)
        let alter_contact_sql = r#"
            ALTER TABLE contact 
            ADD COLUMN IF NOT EXISTS middle_name VARCHAR(100);
        "#;
        db.execute_unprepared(alter_contact_sql).await?;

        // 2. Extend the existing activity table with a tenant_id column for clean tenant isolation
        let alter_activity_sql = r#"
            ALTER TABLE activity 
            ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenant(id) ON DELETE CASCADE;
        "#;
        db.execute_unprepared(alter_activity_sql).await?;

        // 3. Backfill tenant_id for existing activities dynamically using COALESCE joins on contact, lead, and customer
        let backfill_activity_tenant_sql = r#"
            UPDATE activity a
            SET tenant_id = COALESCE(
                (SELECT tenant_id FROM contact c WHERE c.id = a.contact_id),
                (SELECT tenant_id FROM lead l WHERE l.id = a.lead_id),
                (SELECT tenant_id FROM customer cust WHERE cust.id = a.customer_id),
                (SELECT t.id FROM tenant t LIMIT 1) -- Fallback for orphan activities
            )
            WHERE a.tenant_id IS NULL;
        "#;
        db.execute_unprepared(backfill_activity_tenant_sql).await?;

        // 4. Create high-performance indexing on the existing activity table for global 360-degree timeline feeds
        let create_indexes_sql = r#"
            CREATE INDEX IF NOT EXISTS idx_activity_tenant_created ON activity (tenant_id, created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_activity_contact_created ON activity (contact_id, created_at DESC);
        "#;
        db.execute_unprepared(create_indexes_sql).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // 1. Remove indexes and columns from activity table
        let drop_indexes_sql = r#"
            DROP INDEX IF EXISTS idx_activity_tenant_created;
            DROP INDEX IF EXISTS idx_activity_contact_created;
            ALTER TABLE activity DROP COLUMN IF EXISTS tenant_id;
        "#;
        db.execute_unprepared(drop_indexes_sql).await?;

        // 2. Remove added columns from contact table
        let drop_columns_sql = r#"
            ALTER TABLE contact 
            DROP COLUMN IF EXISTS middle_name;
        "#;
        db.execute_unprepared(drop_columns_sql).await?;

        Ok(())
    }
}
