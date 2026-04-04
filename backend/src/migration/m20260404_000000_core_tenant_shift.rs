use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        
        let sql = r#"
            ALTER TABLE IF EXISTS directory RENAME TO tenant;
            
            ALTER TABLE IF EXISTS template RENAME COLUMN tenant_id TO tenant_id;
            ALTER TABLE IF EXISTS lead RENAME COLUMN tenant_id TO tenant_id;
            ALTER TABLE IF EXISTS account RENAME COLUMN tenant_id TO tenant_id;
            ALTER TABLE IF EXISTS deal RENAME COLUMN tenant_id TO tenant_id;
            ALTER TABLE IF EXISTS contact RENAME COLUMN tenant_id TO tenant_id;
            ALTER TABLE IF EXISTS customer RENAME COLUMN tenant_id TO tenant_id;
            ALTER TABLE IF EXISTS category RENAME COLUMN tenant_id TO tenant_id;
            ALTER TABLE IF EXISTS feed RENAME COLUMN tenant_id TO tenant_id;
            ALTER TABLE IF EXISTS profile RENAME COLUMN tenant_id TO tenant_id;
            ALTER TABLE IF EXISTS listing RENAME COLUMN tenant_id TO tenant_id;
            
            DROP TABLE IF EXISTS directory_type CASCADE;
            
            CREATE TABLE IF NOT EXISTS app_instances (
                id UUID PRIMARY KEY,
                tenant_id UUID NOT NULL REFERENCES tenant(id) ON DELETE CASCADE,
                app_type VARCHAR NOT NULL,
                database_url VARCHAR,
                data_seed_name VARCHAR,
                settings JSONB,
                created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            
            CREATE TABLE IF NOT EXISTS app_domains (
                id UUID PRIMARY KEY,
                app_instance_id UUID NOT NULL REFERENCES app_instances(id) ON DELETE CASCADE,
                domain_name VARCHAR NOT NULL UNIQUE,
                created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            
            -- Insert an app_instance for existing tenants
            INSERT INTO app_instances (id, tenant_id, app_type, settings)
            SELECT gen_random_uuid(), id, 'Directory', custom_settings 
            FROM tenant;
            
            -- Insert app_domains for their explicit domains
            INSERT INTO app_domains (id, app_instance_id, domain_name)
            SELECT gen_random_uuid(), a.id, t.domain
            FROM tenant t
            JOIN app_instances a ON a.tenant_id = t.id AND a.app_type = 'Directory'
            WHERE t.domain IS NOT NULL;
            
            -- Insert app_domains for subdomains too if they exist
            INSERT INTO app_domains (id, app_instance_id, domain_name)
            SELECT gen_random_uuid(), a.id, t.subdomain || '.oply.co'
            FROM tenant t
            JOIN app_instances a ON a.tenant_id = t.id AND a.app_type = 'Directory'
            WHERE t.subdomain IS NOT NULL AND t.subdomain != '';
            
            -- Insert app_domains for custom_domains too if they exist
            INSERT INTO app_domains (id, app_instance_id, domain_name)
            SELECT gen_random_uuid(), a.id, t.custom_domain
            FROM tenant t
            JOIN app_instances a ON a.tenant_id = t.id AND a.app_type = 'Directory'
            WHERE t.custom_domain IS NOT NULL AND t.custom_domain != '';
            
            -- Now, clean up the tenant table properties
            ALTER TABLE tenant DROP COLUMN IF EXISTS custom_settings;
            ALTER TABLE tenant DROP COLUMN IF EXISTS domain;
            ALTER TABLE tenant DROP COLUMN IF EXISTS subdomain;
            ALTER TABLE tenant DROP COLUMN IF EXISTS custom_domain;
            ALTER TABLE tenant DROP COLUMN IF EXISTS enabled_modules;
            ALTER TABLE tenant DROP COLUMN IF EXISTS theme;
            ALTER TABLE tenant DROP COLUMN IF EXISTS directory_type_id;
        "#;
        
        db.execute_unprepared(sql).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let sql = r#"
            -- Reverse operations for down migration (lossy)
            ALTER TABLE IF EXISTS tenant RENAME TO directory;
            
            ALTER TABLE IF EXISTS template RENAME COLUMN tenant_id TO tenant_id;
            ALTER TABLE IF EXISTS lead RENAME COLUMN tenant_id TO tenant_id;
            ALTER TABLE IF EXISTS account RENAME COLUMN tenant_id TO tenant_id;
            ALTER TABLE IF EXISTS deal RENAME COLUMN tenant_id TO tenant_id;
            ALTER TABLE IF EXISTS contact RENAME COLUMN tenant_id TO tenant_id;
            ALTER TABLE IF EXISTS customer RENAME COLUMN tenant_id TO tenant_id;
            ALTER TABLE IF EXISTS category RENAME COLUMN tenant_id TO tenant_id;
            ALTER TABLE IF EXISTS feed RENAME COLUMN tenant_id TO tenant_id;
            ALTER TABLE IF EXISTS profile RENAME COLUMN tenant_id TO tenant_id;
            ALTER TABLE IF EXISTS listing RENAME COLUMN tenant_id TO tenant_id;
            
            DROP TABLE IF EXISTS app_domains CASCADE;
            DROP TABLE IF EXISTS app_instances CASCADE;
        "#;
        db.execute_unprepared(sql).await?;
        Ok(())
    }
}
