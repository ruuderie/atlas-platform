use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let sql = r#"
            DO $$
            DECLARE
                v_tenant_id UUID;
            BEGIN
                SELECT id INTO v_tenant_id FROM tenant WHERE name ILIKE '%buildwithruud%' LIMIT 1;

                IF v_tenant_id IS NOT NULL THEN
                    
                    -- Fix 'consulting' page: Change "cards" to "items" inside the Grid block so it deserializes properly
                    UPDATE app_pages 
                    SET blocks_payload = REPLACE(blocks_payload::text, '"cards":', '"items":')::jsonb 
                    WHERE slug = 'consulting' AND tenant_id = v_tenant_id;

                    -- Fix 'real-estate-ventures' page: Change legacy flat object into proper DynamicBlock array
                    UPDATE app_pages
                    SET blocks_payload = '[
                        {
                            "Hero": {
                                "title": "Real Estate Ventures",
                                "subtitle": "Acquisition, management, and financing of physical assets.",
                                "layout": "standard"
                            }
                        },
                        {
                            "FormBuilder": {
                                "form_id": "contact_form",
                                "title": "Invest with Us",
                                "description": "Contact us for passive opportunities.",
                                "cta_text": "Submit"
                            }
                        }
                    ]'::jsonb
                    WHERE slug = 'real-estate-ventures' AND tenant_id = v_tenant_id;
                    
                END IF;
            END $$;
        "#;

        db.execute_unprepared(sql).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Safe down: just leave it fixed.
        Ok(())
    }
}
