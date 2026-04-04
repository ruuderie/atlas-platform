use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let sql = r#"
            DO $$
            BEGIN
                -- Safely check if the legacy 'mailing_list' table from Anchor still exists
                IF EXISTS (SELECT FROM pg_tables WHERE schemaname = 'public' AND tablename = 'mailing_list') THEN
                    -- Port the data to the central CRM `lead` table
                    INSERT INTO lead (id, name, email, source, is_converted, converted_to_contact, created_at, updated_at, tenant_id)
                    SELECT 
                        gen_random_uuid(), 
                        SPLIT_PART(email, '@', 1), -- Simple name extraction fallback
                        email, 
                        'Services Form (' || list_type || ')', 
                        false, 
                        false, 
                        created_at, 
                        created_at, 
                        tenant_id
                    FROM mailing_list;
                    
                    -- Explicitly drop the legacy table permanently to enforce CRM architecture
                    DROP TABLE mailing_list CASCADE;
                END IF;
            END $$;
        "#;
        
        db.execute_unprepared(sql).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // This is a destructive data-port migration, down migration is a no-op 
        // to prevent reversing logical data into deprecated schema format.
        Ok(())
    }
}
