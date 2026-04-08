use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let sql = r##"
            DO $$
            DECLARE
                v_bwr_tenant_id UUID;
            BEGIN
                -- 1. Identify 'buildwithruud' tenant
                SELECT id INTO v_bwr_tenant_id FROM tenant WHERE name = 'buildwithruud' LIMIT 1;
                
                IF v_bwr_tenant_id IS NOT NULL THEN
                    -- Insert the BitcoinSync background job
                    IF NOT EXISTS (SELECT 1 FROM tenant_background_jobs WHERE tenant_id = v_bwr_tenant_id AND job_type = 'BitcoinSync') THEN
                        INSERT INTO tenant_background_jobs (id, tenant_id, job_type, config, interval_seconds, last_run, is_active) 
                        VALUES (
                            gen_random_uuid(),
                            v_bwr_tenant_id,
                            'BitcoinSync',
                            '{"api_url": "https://mempool.space/api/blocks"}'::jsonb,
                            600, -- 10 minutes
                            NULL,
                            true
                        );
                    END IF;
                END IF;
            END $$;
        "##;

        db.execute_unprepared(sql).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let sql = r#"
            DELETE FROM tenant_background_jobs WHERE job_type = 'BitcoinSync';
        "#;
        db.execute_unprepared(sql).await?;
        Ok(())
    }
}
