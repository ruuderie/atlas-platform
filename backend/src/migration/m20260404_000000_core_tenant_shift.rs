use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        
        // 1. Rename directory to tenant if "directory" exists
        let row = db.query_one(sea_orm::Statement::from_string(manager.get_database_backend(), "SELECT table_name FROM information_schema.tables WHERE table_name='directory'".to_owned())).await?;
        if row.is_some() {
            db.execute_unprepared("ALTER TABLE directory RENAME TO tenant;").await?;
        }
        
        let tables_to_rename = vec!["template", "lead", "account", "deal", "contact", "customer", "category", "feed", "profile", "listing"];
        for table in tables_to_rename {
            let q = format!("SELECT column_name FROM information_schema.columns WHERE table_name='{}' AND column_name='directory_id'", table);
            let has_col = db.query_one(sea_orm::Statement::from_string(manager.get_database_backend(), q)).await?;
            if has_col.is_some() {
                let alter = format!("ALTER TABLE {} RENAME COLUMN directory_id TO tenant_id;", table);
                db.execute_unprepared(&alter).await?;
            }
        }
        
        let q = "SELECT column_name FROM information_schema.columns WHERE table_name='category' AND column_name='directory_type_id'";
        if db.query_one(sea_orm::Statement::from_string(manager.get_database_backend(), q.to_owned())).await?.is_some() {
            db.execute_unprepared("ALTER TABLE category DROP COLUMN directory_type_id;").await?;
        }
        
        db.execute_unprepared("DROP TABLE IF EXISTS directory_type CASCADE;").await?;
        
        let sql = r#"
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
            
            INSERT INTO app_instances (id, tenant_id, app_type, settings)
            SELECT gen_random_uuid(), id, 'Directory', custom_settings 
            FROM tenant ON CONFLICT DO NOTHING;
            
            INSERT INTO app_domains (id, app_instance_id, domain_name)
            SELECT gen_random_uuid(), a.id, t.domain
            FROM tenant t
            JOIN app_instances a ON a.tenant_id = t.id AND a.app_type = 'Directory'
            WHERE t.domain IS NOT NULL ON CONFLICT DO NOTHING;
            
            INSERT INTO app_domains (id, app_instance_id, domain_name)
            SELECT gen_random_uuid(), a.id, t.subdomain || '.oply.co'
            FROM tenant t
            JOIN app_instances a ON a.tenant_id = t.id AND a.app_type = 'Directory'
            WHERE t.subdomain IS NOT NULL AND t.subdomain != '' ON CONFLICT DO NOTHING;
            
            INSERT INTO app_domains (id, app_instance_id, domain_name)
            SELECT gen_random_uuid(), a.id, t.custom_domain
            FROM tenant t
            JOIN app_instances a ON a.tenant_id = t.id AND a.app_type = 'Directory'
            WHERE t.custom_domain IS NOT NULL AND t.custom_domain != '' ON CONFLICT DO NOTHING;
        "#;
        
        let _ = db.execute_unprepared(sql).await;
        
        let cols_to_drop = vec!["custom_settings", "domain", "subdomain", "custom_domain", "enabled_modules", "theme", "directory_type_id"];
        for col in cols_to_drop {
            let q = format!("SELECT column_name FROM information_schema.columns WHERE table_name='tenant' AND column_name='{}'", col);
            if db.query_one(sea_orm::Statement::from_string(manager.get_database_backend(), q)).await?.is_some() {
                let alter = format!("ALTER TABLE tenant DROP COLUMN {};", col);
                db.execute_unprepared(&alter).await?;
            }
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let row = db.query_one(sea_orm::Statement::from_string(manager.get_database_backend(), "SELECT table_name FROM information_schema.tables WHERE table_name='tenant'".to_owned())).await?;
        if row.is_some() {
            db.execute_unprepared("ALTER TABLE tenant RENAME TO directory;").await?;
        }
        
        let tables_to_rename = vec!["template", "lead", "account", "deal", "contact", "customer", "category", "feed", "profile", "listing"];
        for table in tables_to_rename {
            let q = format!("SELECT column_name FROM information_schema.columns WHERE table_name='{}' AND column_name='tenant_id'", table);
            let has_col = db.query_one(sea_orm::Statement::from_string(manager.get_database_backend(), q)).await?;
            if has_col.is_some() {
                let alter = format!("ALTER TABLE {} RENAME COLUMN tenant_id TO directory_id;", table);
                db.execute_unprepared(&alter).await?;
            }
        }
        
        db.execute_unprepared("DROP TABLE IF EXISTS app_domains CASCADE;").await?;
        db.execute_unprepared("DROP TABLE IF EXISTS app_instances CASCADE;").await?;
        Ok(())
    }
}
