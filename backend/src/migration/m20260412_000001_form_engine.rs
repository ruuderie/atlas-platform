use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        
        let sql = r#"
            CREATE TABLE IF NOT EXISTS form_schemas (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                tenant_id UUID NOT NULL,
                name VARCHAR(255) NOT NULL,
                slug VARCHAR(255) NOT NULL,
                description TEXT,
                schema_json JSONB NOT NULL DEFAULT '{}'::jsonb,
                webhook_url VARCHAR(500),
                is_active BOOLEAN NOT NULL DEFAULT true,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE (tenant_id, slug)
            );

            CREATE TABLE IF NOT EXISTS form_submissions (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                form_id UUID NOT NULL REFERENCES form_schemas(id) ON DELETE CASCADE,
                tenant_id UUID NOT NULL,
                payload_json JSONB NOT NULL DEFAULT '{}'::jsonb,
                submitted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                ip_address VARCHAR(45),
                is_synced BOOLEAN NOT NULL DEFAULT false
            );
            
            -- Preseed base form_schemas if none exist for existing tenants? 
            -- Actually we will do this in the app-specific seed 000000_seed_oplystusa.
        "#;
        
        db.execute_unprepared(sql).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("DROP TABLE IF EXISTS form_submissions;").await?;
        db.execute_unprepared("DROP TABLE IF EXISTS form_schemas;").await?;
        Ok(())
    }
}
