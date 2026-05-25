use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // 1. Alter notes table to add tenant_id and is_private
        let alter_notes_sql = r#"
            ALTER TABLE notes 
            ADD COLUMN IF NOT EXISTS tenant_id UUID REFERENCES tenant(id) ON DELETE CASCADE,
            ADD COLUMN IF NOT EXISTS is_private BOOLEAN NOT NULL DEFAULT false;
        "#;
        db.execute_unprepared(alter_notes_sql).await?;

        // 2. Backfill tenant_id for existing notes using COALESCE joins on contact, lead, customer, deal, and activity
        let backfill_notes_tenant_sql = r#"
            UPDATE notes n
            SET tenant_id = COALESCE(
                (SELECT tenant_id FROM contact c WHERE c.id = n.entity_id AND n.entity_type = 'Contact'),
                (SELECT tenant_id FROM lead l WHERE l.id = n.entity_id AND n.entity_type = 'Lead'),
                (SELECT tenant_id FROM customer cust WHERE cust.id = n.entity_id AND n.entity_type = 'Customer'),
                (SELECT tenant_id FROM deal d WHERE d.id = n.entity_id AND n.entity_type = 'Deal'),
                (SELECT tenant_id FROM activity a WHERE a.id = n.entity_id AND n.entity_type = 'Activity'),
                (SELECT t.id FROM tenant t LIMIT 1) -- Fallback for orphan notes
            )
            WHERE n.tenant_id IS NULL;
        "#;
        db.execute_unprepared(backfill_notes_tenant_sql).await?;

        // 3. Create high-performance indexing on notes table
        let create_indexes_sql = r#"
            CREATE INDEX IF NOT EXISTS idx_notes_tenant_entity ON notes (tenant_id, entity_type, entity_id);
            CREATE INDEX IF NOT EXISTS idx_notes_creator_private ON notes (created_by, is_private);
        "#;
        db.execute_unprepared(create_indexes_sql).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // 1. Remove indexes and columns from notes table
        let drop_sql = r#"
            DROP INDEX IF EXISTS idx_notes_tenant_entity;
            DROP INDEX IF EXISTS idx_notes_creator_private;
            ALTER TABLE notes 
            DROP COLUMN IF EXISTS tenant_id,
            DROP COLUMN IF EXISTS is_private;
        "#;
        db.execute_unprepared(drop_sql).await?;

        Ok(())
    }
}
