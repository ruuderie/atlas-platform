use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let sql = r#"
            -- Create the enum for strict type scoping
            CREATE TYPE tenant_ownership_enum AS ENUM ('INTERNAL', 'CLIENT');
            
            -- Add the ownership track to the unified tenant schema
            ALTER TABLE tenant 
            ADD COLUMN ownership_type tenant_ownership_enum NOT NULL DEFAULT 'CLIENT';
            
            -- Set existing "Services" (Anchor Base) or "Admin" tenants to INTERNAL
            UPDATE tenant SET ownership_type = 'INTERNAL' 
            WHERE name = 'Oply Anchor Base' OR name ILIKE '%admin%';
        "#;
        
        db.execute_unprepared(sql).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let sql = r#"
            ALTER TABLE tenant DROP COLUMN IF EXISTS ownership_type;
            DROP TYPE IF EXISTS tenant_ownership_enum;
        "#;
        db.execute_unprepared(sql).await?;
        Ok(())
    }
}
