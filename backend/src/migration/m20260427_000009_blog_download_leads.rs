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
                CREATE TABLE IF NOT EXISTS blog_download_leads (
                    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                    tenant_id         UUID NOT NULL REFERENCES tenant(id) ON DELETE CASCADE,
                    post_id           UUID NOT NULL REFERENCES app_content(id) ON DELETE CASCADE,
                    email             TEXT NOT NULL,
                    name              TEXT,
                    ip_address        TEXT,
                    notification_sent BOOLEAN NOT NULL DEFAULT FALSE,
                    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
                );

                CREATE INDEX IF NOT EXISTS idx_blog_download_leads_tenant_post
                    ON blog_download_leads(tenant_id, post_id);
                CREATE INDEX IF NOT EXISTS idx_blog_download_leads_email
                    ON blog_download_leads(tenant_id, email);
            END $$;
        "##;
        db.execute_unprepared(sql).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let sql = r#"DROP TABLE IF EXISTS blog_download_leads;"#;
        db.execute_unprepared(sql).await?;
        Ok(())
    }
}
