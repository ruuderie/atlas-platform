use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TABLE global_search_index (
                    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                    entity_type VARCHAR(255) NOT NULL,
                    entity_id UUID NOT NULL,
                    tenant_id UUID,
                    searchable_text tsvector,
                    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
                    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
                );

                CREATE INDEX idx_global_search_tenant_id ON global_search_index (tenant_id);
                CREATE UNIQUE INDEX idx_global_search_entity ON global_search_index (entity_type, entity_id);
                CREATE INDEX idx_global_search_text ON global_search_index USING GIN (searchable_text);
                "#,
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS global_search_index CASCADE;")
            .await?;

        Ok(())
    }
}
