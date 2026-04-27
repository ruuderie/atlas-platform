use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let sql = r#"
            DO $migration$
            DECLARE
                v_tenant_id UUID;
                v_current_payload JSONB;
                v_new_content TEXT;
            BEGIN
                SELECT id INTO v_tenant_id FROM tenant WHERE name ILIKE '%buildwithruud%' LIMIT 1;

                IF v_tenant_id IS NOT NULL THEN
                    SELECT payload INTO v_current_payload 
                    FROM app_content 
                    WHERE tenant_id = v_tenant_id 
                      AND collection_type = 'blog_post' 
                      AND payload->>'slug' = 'exploratory-diagonalization-argument-p-vs-np'
                    LIMIT 1;

                    IF v_current_payload IS NOT NULL THEN
                        -- Replace \( and \) with $ (ensuring no extra spaces)
                        v_new_content := REPLACE(v_current_payload->>'content', '\( ', '$');
                        v_new_content := REPLACE(v_new_content, ' \)', '$');
                        -- Also catch any without spaces
                        v_new_content := REPLACE(v_new_content, '\(', '$');
                        v_new_content := REPLACE(v_new_content, '\)', '$');

                        -- Replace \[ and \] with $$
                        v_new_content := REPLACE(v_new_content, '\[', '$$');
                        v_new_content := REPLACE(v_new_content, '\]', '$$');

                        UPDATE app_content 
                        SET payload = jsonb_set(payload, '{content}', to_jsonb(v_new_content))
                        WHERE tenant_id = v_tenant_id 
                          AND collection_type = 'blog_post' 
                          AND payload->>'slug' = 'exploratory-diagonalization-argument-p-vs-np';
                          
                        RAISE NOTICE 'SUCCESS: Fixed math delimiters for P vs NP blog post';
                    END IF;
                END IF;
            END $migration$;
        "#;

        db.execute_unprepared(sql).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // No down migration needed for this patch
        Ok(())
    }
}
