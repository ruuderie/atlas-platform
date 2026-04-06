use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Execute raw SQL for better performance
        manager.get_connection().execute_unprepared(
            r#"
            ALTER TABLE "directory" 
            ALTER COLUMN "created_at" TYPE TIMESTAMP WITH TIME ZONE,
            ALTER COLUMN "updated_at" TYPE TIMESTAMP WITH TIME ZONE;

            ALTER TABLE "listing" 
            ALTER COLUMN "created_at" TYPE TIMESTAMP WITH TIME ZONE,
            ALTER COLUMN "updated_at" TYPE TIMESTAMP WITH TIME ZONE;

            ALTER TABLE "profile" 
            ALTER COLUMN "created_at" TYPE TIMESTAMP WITH TIME ZONE,
            ALTER COLUMN "updated_at" TYPE TIMESTAMP WITH TIME ZONE;

            ALTER TABLE "user" 
            ALTER COLUMN "created_at" TYPE TIMESTAMP WITH TIME ZONE,
            ALTER COLUMN "updated_at" TYPE TIMESTAMP WITH TIME ZONE;

            ALTER TABLE "account" 
            ALTER COLUMN "created_at" TYPE TIMESTAMP WITH TIME ZONE,
            ALTER COLUMN "updated_at" TYPE TIMESTAMP WITH TIME ZONE;

            ALTER TABLE "user_account" 
            ALTER COLUMN "created_at" TYPE TIMESTAMP WITH TIME ZONE,
            ALTER COLUMN "updated_at" TYPE TIMESTAMP WITH TIME ZONE;

            ALTER TABLE "ad_purchase" 
            ALTER COLUMN "created_at" TYPE TIMESTAMP WITH TIME ZONE,
            ALTER COLUMN "updated_at" TYPE TIMESTAMP WITH TIME ZONE;

            ALTER TABLE "category" 
            ALTER COLUMN "created_at" TYPE TIMESTAMP WITH TIME ZONE,
            ALTER COLUMN "updated_at" TYPE TIMESTAMP WITH TIME ZONE;

            ALTER TABLE "template" 
            ALTER COLUMN "created_at" TYPE TIMESTAMP WITH TIME ZONE,
            ALTER COLUMN "updated_at" TYPE TIMESTAMP WITH TIME ZONE;

            ALTER TABLE "listing_attribute" 
            ALTER COLUMN "created_at" TYPE TIMESTAMP WITH TIME ZONE,
            ALTER COLUMN "updated_at" TYPE TIMESTAMP WITH TIME ZONE;

            ALTER TABLE "session" 
            ALTER COLUMN "created_at" TYPE TIMESTAMP WITH TIME ZONE,
            ALTER COLUMN "last_accessed_at" TYPE TIMESTAMP WITH TIME ZONE,
            ALTER COLUMN "token_expiration" TYPE TIMESTAMP WITH TIME ZONE,
            ALTER COLUMN "refresh_token_expiration" TYPE TIMESTAMP WITH TIME ZONE;

            ALTER TABLE "request_log" 
            ALTER COLUMN "created_at" TYPE TIMESTAMP WITH TIME ZONE;
            "#
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Execute raw SQL for better performance
        manager.get_connection().execute_unprepared(
            r#"
            ALTER TABLE "directory" 
            ALTER COLUMN "created_at" TYPE TIMESTAMP,
            ALTER COLUMN "updated_at" TYPE TIMESTAMP;

            ALTER TABLE "listing" 
            ALTER COLUMN "created_at" TYPE TIMESTAMP,
            ALTER COLUMN "updated_at" TYPE TIMESTAMP;

            ALTER TABLE "profile" 
            ALTER COLUMN "created_at" TYPE TIMESTAMP,
            ALTER COLUMN "updated_at" TYPE TIMESTAMP;

            ALTER TABLE "user" 
            ALTER COLUMN "created_at" TYPE TIMESTAMP,
            ALTER COLUMN "updated_at" TYPE TIMESTAMP;

            ALTER TABLE "account" 
            ALTER COLUMN "created_at" TYPE TIMESTAMP,
            ALTER COLUMN "updated_at" TYPE TIMESTAMP;

            ALTER TABLE "user_account" 
            ALTER COLUMN "created_at" TYPE TIMESTAMP,
            ALTER COLUMN "updated_at" TYPE TIMESTAMP;

            ALTER TABLE "ad_purchase" 
            ALTER COLUMN "created_at" TYPE TIMESTAMP,
            ALTER COLUMN "updated_at" TYPE TIMESTAMP;

            ALTER TABLE "category" 
            ALTER COLUMN "created_at" TYPE TIMESTAMP,
            ALTER COLUMN "updated_at" TYPE TIMESTAMP;

            ALTER TABLE "template" 
            ALTER COLUMN "created_at" TYPE TIMESTAMP,
            ALTER COLUMN "updated_at" TYPE TIMESTAMP;

            ALTER TABLE "listing_attribute" 
            ALTER COLUMN "created_at" TYPE TIMESTAMP,
            ALTER COLUMN "updated_at" TYPE TIMESTAMP;

            ALTER TABLE "session" 
            ALTER COLUMN "created_at" TYPE TIMESTAMP,
            ALTER COLUMN "last_accessed_at" TYPE TIMESTAMP,
            ALTER COLUMN "token_expiration" TYPE TIMESTAMP,
            ALTER COLUMN "refresh_token_expiration" TYPE TIMESTAMP;

            ALTER TABLE "request_log" 
            ALTER COLUMN "created_at" TYPE TIMESTAMP;
            
            "#
        ).await?;

        Ok(())
    }
}

