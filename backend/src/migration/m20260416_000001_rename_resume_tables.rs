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
                -- 1. Rename tables (idempotent checks first)
                IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'resume_entries') THEN
                    ALTER TABLE resume_entries RENAME TO tenant_entries;
                END IF;

                IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'resume_profiles') THEN
                    ALTER TABLE resume_profiles RENAME TO entry_collections;
                END IF;

                IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'resume_profile_entries') THEN
                    ALTER TABLE resume_profile_entries RENAME TO collection_entries;
                END IF;

                -- 2. Rename enum type
                IF EXISTS (SELECT 1 FROM pg_type WHERE typname = 'resume_category_enum') THEN
                    ALTER TYPE resume_category_enum RENAME TO entry_category_enum;
                END IF;

                -- 3. Add generic CMS fields to tenant_entries (if not already added)
                IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'tenant_entries') THEN
                    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'tenant_entries' AND column_name = 'slug') THEN
                        ALTER TABLE tenant_entries ADD COLUMN slug VARCHAR(255);
                    END IF;
                    
                    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'tenant_entries' AND column_name = 'is_published') THEN
                        ALTER TABLE tenant_entries ADD COLUMN is_published BOOLEAN NOT NULL DEFAULT TRUE;
                    END IF;
                    
                    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'tenant_entries' AND column_name = 'published_at') THEN
                        ALTER TABLE tenant_entries ADD COLUMN published_at TIMESTAMPTZ;
                        -- Populate published_at loosely based on created_at for existing rows
                        UPDATE tenant_entries SET published_at = created_at;
                    END IF;
                END IF;
            END $$;
        "#;

        db.execute_unprepared(sql).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let sql = r#"
            DO $$
            BEGIN
                -- Reverse CMS fields
                IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'tenant_entries' AND column_name = 'slug') THEN
                    ALTER TABLE tenant_entries DROP COLUMN slug;
                END IF;
                IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'tenant_entries' AND column_name = 'is_published') THEN
                    ALTER TABLE tenant_entries DROP COLUMN is_published;
                END IF;
                IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'tenant_entries' AND column_name = 'published_at') THEN
                    ALTER TABLE tenant_entries DROP COLUMN published_at;
                END IF;

                -- Reverse enums
                IF EXISTS (SELECT 1 FROM pg_type WHERE typname = 'entry_category_enum') THEN
                    ALTER TYPE entry_category_enum RENAME TO resume_category_enum;
                END IF;

                -- Reverse table renames
                IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'collection_entries') THEN
                    ALTER TABLE collection_entries RENAME TO resume_profile_entries;
                END IF;

                IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'entry_collections') THEN
                    ALTER TABLE entry_collections RENAME TO resume_profiles;
                END IF;

                IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'tenant_entries') THEN
                    ALTER TABLE tenant_entries RENAME TO resume_entries;
                END IF;
            END $$;
        "#;
        db.execute_unprepared(sql).await?;
        Ok(())
    }
}
