use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // 1. Add company, title, and lead_status columns to the lead table
        let alter_lead_sql = r#"
            ALTER TABLE lead 
            ADD COLUMN IF NOT EXISTS company VARCHAR(255),
            ADD COLUMN IF NOT EXISTS title VARCHAR(255),
            ADD COLUMN IF NOT EXISTS lead_status VARCHAR(255) DEFAULT 'New';
        "#;
        db.execute_unprepared(alter_lead_sql).await?;

        // 2. Add account_id and associated_entities columns to the activity table
        let alter_activity_sql = r#"
            ALTER TABLE activity 
            ADD COLUMN IF NOT EXISTS account_id UUID,
            ADD COLUMN IF NOT EXISTS associated_entities JSONB DEFAULT '[]'::jsonb;
            
            ALTER TABLE activity 
            ALTER COLUMN description DROP NOT NULL;
        "#;
        db.execute_unprepared(alter_activity_sql).await?;

        // 3. Rename note table to notes to match SeaORM model
        let rename_note_sql = r#"
            ALTER TABLE IF EXISTS note RENAME TO notes;
        "#;
        db.execute_unprepared(rename_note_sql).await?;

        // 4. Load and execute the ruud_personal.sql seed script
        let seed_sql = include_str!("ruud_personal.sql");
        db.execute_unprepared(seed_sql).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Rollback lead table modifications
        let rollback_lead_sql = r#"
            ALTER TABLE lead 
            DROP COLUMN IF EXISTS company,
            DROP COLUMN IF EXISTS title,
            DROP COLUMN IF EXISTS lead_status;
        "#;
        db.execute_unprepared(rollback_lead_sql).await?;

        // Rollback activity table modifications
        let rollback_activity_sql = r#"
            ALTER TABLE activity 
            DROP COLUMN IF EXISTS account_id,
            DROP COLUMN IF EXISTS associated_entities;
            
            ALTER TABLE activity 
            ALTER COLUMN description SET NOT NULL;
        "#;
        db.execute_unprepared(rollback_activity_sql).await?;

        // Rollback note table rename
        let rollback_note_sql = r#"
            ALTER TABLE IF EXISTS notes RENAME TO note;
        "#;
        db.execute_unprepared(rollback_note_sql).await?;

        Ok(())
    }
}
