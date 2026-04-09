use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let sql = r##"
            DO $$
            BEGIN
                CREATE TABLE IF NOT EXISTS app_content (
                    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                    tenant_id UUID NOT NULL REFERENCES tenant(id) ON DELETE CASCADE,
                    collection_type VARCHAR(255) NOT NULL,
                    title VARCHAR(500) NOT NULL,
                    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
                    status VARCHAR(50) NOT NULL DEFAULT 'published',
                    display_order INTEGER NOT NULL DEFAULT 0,
                    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
                );

                CREATE INDEX IF NOT EXISTS idx_app_content_tenant_collection ON app_content(tenant_id, collection_type);
                CREATE INDEX IF NOT EXISTS idx_app_content_status ON app_content(status);
            END $$;
        "##;
        
        db.execute_unprepared(sql).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let sql = r#"
            DROP TABLE IF EXISTS app_content;
        "#;
        db.execute_unprepared(sql).await?;
        Ok(())
    }
}
