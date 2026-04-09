use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let sql = r#"
            CREATE TABLE IF NOT EXISTS case_studies (
                id SERIAL PRIMARY KEY,
                tenant_id UUID REFERENCES tenant(id) ON DELETE CASCADE,
                client_name VARCHAR(255) NOT NULL,
                problem TEXT NOT NULL,
                solution TEXT NOT NULL,
                roi_impact TEXT NOT NULL,
                is_visible BOOLEAN NOT NULL DEFAULT TRUE,
                display_order INTEGER NOT NULL DEFAULT 0,
                created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS highlights (
                id SERIAL PRIMARY KEY,
                tenant_id UUID REFERENCES tenant(id) ON DELETE CASCADE,
                title VARCHAR(255) NOT NULL,
                url VARCHAR(500) NOT NULL,
                image_url VARCHAR(500),
                description TEXT,
                is_visible BOOLEAN NOT NULL DEFAULT TRUE,
                display_order INTEGER NOT NULL DEFAULT 0,
                created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
            );
        "#;

        let db = manager.get_connection();
        db.execute_unprepared(sql).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let sql = r#"
            DROP TABLE IF EXISTS highlights;
            DROP TABLE IF EXISTS case_studies;
        "#;
        let db = manager.get_connection();
        db.execute_unprepared(sql).await?;
        Ok(())
    }
}
