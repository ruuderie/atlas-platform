use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Sets layout = "kami_cards" on the content_feed block in the buildwithruud
/// /p/projects page, so the projects page renders Kami parchment cards without
/// requiring every other tenant to change their layout config.
///
/// The JSONB path targeted is:
///   pages.blocks — a jsonb array of block objects where type = "content_feed"
///
/// We use jsonb_set with a subquery to locate the correct block index.
/// If the block does not exist, the query is a no-op.
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Update the content_feed block layout to "kami_cards" on the /p/projects
        // page for the buildwithruud tenant. Uses a PL/pgSQL DO block to safely
        // iterate the blocks array and patch the matching element.
        let sql = r#"
            DO $$
            DECLARE
                v_tenant_id UUID;
                v_page_id   UUID;
                v_blocks    JSONB;
                i           INT;
            BEGIN
                -- Find buildwithruud tenant
                SELECT id INTO v_tenant_id
                FROM tenant
                WHERE name ILIKE '%buildwithruud%'
                LIMIT 1;

                IF v_tenant_id IS NULL THEN RETURN; END IF;

                -- Find the /p/projects page for this tenant
                SELECT id INTO v_page_id
                FROM pages
                WHERE tenant_id = v_tenant_id
                  AND (slug = 'projects' OR slug = '/p/projects' OR path ILIKE '%projects%')
                LIMIT 1;

                IF v_page_id IS NULL THEN RETURN; END IF;

                -- Load current blocks
                SELECT COALESCE(blocks, '[]'::jsonb) INTO v_blocks
                FROM pages WHERE id = v_page_id;

                -- Walk the array and patch content_feed block config.layout
                FOR i IN 0 .. jsonb_array_length(v_blocks) - 1 LOOP
                    IF v_blocks -> i ->> 'type' = 'content_feed' THEN
                        v_blocks := jsonb_set(
                            v_blocks,
                            ARRAY[i::text, 'config', 'layout'],
                            '"kami_cards"'::jsonb,
                            true
                        );
                    END IF;
                END LOOP;

                UPDATE pages SET blocks = v_blocks WHERE id = v_page_id;
            END $$;
        "#;
        db.execute_unprepared(sql).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Revert layout back to "cards"
        let sql = r#"
            DO $$
            DECLARE
                v_tenant_id UUID;
                v_page_id   UUID;
                v_blocks    JSONB;
                i           INT;
            BEGIN
                SELECT id INTO v_tenant_id FROM tenant WHERE name ILIKE '%buildwithruud%' LIMIT 1;
                IF v_tenant_id IS NULL THEN RETURN; END IF;

                SELECT id INTO v_page_id FROM pages
                WHERE tenant_id = v_tenant_id
                  AND (slug = 'projects' OR slug = '/p/projects' OR path ILIKE '%projects%')
                LIMIT 1;
                IF v_page_id IS NULL THEN RETURN; END IF;

                SELECT COALESCE(blocks, '[]'::jsonb) INTO v_blocks FROM pages WHERE id = v_page_id;

                FOR i IN 0 .. jsonb_array_length(v_blocks) - 1 LOOP
                    IF v_blocks -> i ->> 'type' = 'content_feed' THEN
                        v_blocks := jsonb_set(v_blocks, ARRAY[i::text, 'config', 'layout'], '"cards"'::jsonb, true);
                    END IF;
                END LOOP;

                UPDATE pages SET blocks = v_blocks WHERE id = v_page_id;
            END $$;
        "#;
        db.execute_unprepared(sql).await?;

        Ok(())
    }
}
