use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Migrates all app_instances to the new data-driven WidgetInstance config model.
/// - buildwithruud gets the bitcoin_block_clock widget configured for nav + landing placement
/// - All other tenants get an explicit empty widgets array (prevents fallback to defaults)
///
/// The WidgetInstance schema is fully defined in:
///   apps/anchor/src/components/widget_registry.rs
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let sql = r#"
            DO $$
            DECLARE
                v_ruud_tenant_id UUID;
                v_ruud_instance_id UUID;
            BEGIN
                SELECT id INTO v_ruud_tenant_id FROM tenant WHERE name ILIKE '%buildwithruud%' LIMIT 1;

                IF v_ruud_tenant_id IS NULL THEN
                    RAISE EXCEPTION 'buildwithruud tenant not found — cannot configure widget instances';
                END IF;

                SELECT id INTO v_ruud_instance_id
                FROM app_instances
                WHERE tenant_id = v_ruud_tenant_id AND app_type = 'anchor'
                LIMIT 1;

                IF v_ruud_instance_id IS NULL THEN
                    RAISE EXCEPTION 'buildwithruud anchor app_instance not found — cannot configure widget instances';
                END IF;

                -- Enable the bitcoin block clock widget for buildwithruud (nav + landing placement)
                UPDATE app_instances
                SET settings = jsonb_set(
                    COALESCE(settings, '{}'),
                    '{widgets}',
                    '[
                        {
                            "id": "bitcoin_block_clock",
                            "name": "Bitcoin Block Clock",
                            "renderer": {"renderer": "block_clock"},
                            "data_source": {
                                "source_type": "platform_table",
                                "table": "bitcoin_blocks",
                                "column": "height",
                                "filter": null
                            },
                            "placement": ["nav", "landing"],
                            "refresh_seconds": 600,
                            "enabled": true
                        }
                    ]'::jsonb
                )
                WHERE id = v_ruud_instance_id;

                -- All other anchor instances get an explicit empty widgets array
                -- (prevents any future fallback rendering of platform-level defaults)
                UPDATE app_instances
                SET settings = jsonb_set(
                    COALESCE(settings, '{}'),
                    '{widgets}',
                    '[]'::jsonb
                )
                WHERE app_type = 'anchor'
                  AND id != v_ruud_instance_id
                  AND (settings IS NULL OR settings->>'widgets' IS NULL);

            END $$;
        "#;

        db.execute_unprepared(sql).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        // Remove widget config from all anchor instances (reverts to pre-widget-system state)
        let sql = r#"
            UPDATE app_instances
            SET settings = settings - 'widgets'
            WHERE app_type = 'anchor'
              AND settings ? 'widgets';
        "#;
        db.execute_unprepared(sql).await?;
        Ok(())
    }
}
