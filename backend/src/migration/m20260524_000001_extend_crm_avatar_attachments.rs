use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // 1. Alter contact table to add avatar_url
        let alter_contact_sql = r#"
            ALTER TABLE contact ADD COLUMN IF NOT EXISTS avatar_url VARCHAR(512);
        "#;
        db.execute_unprepared(alter_contact_sql).await?;

        // 2. Alter lead table to add avatar_url
        let alter_lead_sql = r#"
            ALTER TABLE lead ADD COLUMN IF NOT EXISTS avatar_url VARCHAR(512);
        "#;
        db.execute_unprepared(alter_lead_sql).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("ALTER TABLE contact DROP COLUMN IF EXISTS avatar_url;")
            .await?;
        db.execute_unprepared("ALTER TABLE lead DROP COLUMN IF EXISTS avatar_url;")
            .await?;
        Ok(())
    }
}
