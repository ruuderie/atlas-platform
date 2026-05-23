use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // 1. Create crm_status_option table
        let create_table_sql = r#"
            CREATE TABLE IF NOT EXISTS crm_status_option (
                id UUID PRIMARY KEY,
                tenant_id UUID NOT NULL REFERENCES tenant(id) ON DELETE CASCADE,
                object_type VARCHAR(50) NOT NULL,
                status_key VARCHAR(50) NOT NULL,
                label VARCHAR(100) NOT NULL,
                color VARCHAR(50) NOT NULL DEFAULT 'slate',
                sort_order INT NOT NULL DEFAULT 0,
                is_system BOOLEAN NOT NULL DEFAULT false,
                created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                CONSTRAINT unique_tenant_object_key UNIQUE (tenant_id, object_type, status_key)
            );
        "#;
        db.execute_unprepared(create_table_sql).await?;

        // 2. Seed Lead default status options for all existing tenants dynamically
        let seed_leads_sql = r#"
            INSERT INTO crm_status_option (id, tenant_id, object_type, status_key, label, color, sort_order, is_system)
            SELECT 
                gen_random_uuid(), 
                t.id, 
                'Lead', 
                opt.status_key, 
                opt.label, 
                opt.color, 
                opt.sort_order, 
                opt.is_system
            FROM tenant t
            CROSS JOIN (
                VALUES 
                    ('new', 'New', 'slate', 10, false),
                    ('contacted', 'Contacted', 'blue', 20, false),
                    ('nurturing', 'Nurturing', 'purple', 30, false),
                    ('qualified', 'Qualified', 'indigo', 40, false),
                    ('unqualified', 'Unqualified', 'orange', 50, false),
                    ('converted', 'Converted', 'emerald', 60, true)
            ) AS opt(status_key, label, color, sort_order, is_system)
            ON CONFLICT (tenant_id, object_type, status_key) DO NOTHING;
        "#;
        db.execute_unprepared(seed_leads_sql).await?;

        // 3. Seed Contact default status options for all existing tenants dynamically
        let seed_contacts_sql = r#"
            INSERT INTO crm_status_option (id, tenant_id, object_type, status_key, label, color, sort_order, is_system)
            SELECT 
                gen_random_uuid(), 
                t.id, 
                'Contact', 
                opt.status_key, 
                opt.label, 
                opt.color, 
                opt.sort_order, 
                opt.is_system
            FROM tenant t
            CROSS JOIN (
                VALUES 
                    ('prospect', 'Prospect', 'slate', 10, false),
                    ('active', 'Active', 'emerald', 20, false),
                    ('inactive', 'Inactive', 'rose', 30, false),
                    ('partner', 'Partner', 'blue', 40, false)
            ) AS opt(status_key, label, color, sort_order, is_system)
            ON CONFLICT (tenant_id, object_type, status_key) DO NOTHING;
        "#;
        db.execute_unprepared(seed_contacts_sql).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let drop_table_sql = "DROP TABLE IF EXISTS crm_status_option CASCADE;";
        db.execute_unprepared(drop_table_sql).await?;

        Ok(())
    }
}
