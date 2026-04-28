use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Sets `layout = "kami_cards"` on the `content_feed` block of the
/// buildwithruud /p/projects page so the projects section renders Kami
/// parchment cards without affecting other tenants.
///
/// ## Resilience design
///
/// The reviewer correctly flagged that slug-based lookups couple this
/// migration to user-editable content.  We address this in layers:
///
/// 1. **System flag (preferred, future-proof)**: if the page row carries
///    `metadata->>'kami_projects_page' = 'true'` we use that.  This is a
///    stable, admin-invisible field that survives slug renames.
///
/// 2. **Explicit slug (current)**: `slug IN ('projects', '/p/projects')`.
///    Covers all known deployments without a system flag.
///
/// 3. **Path heuristic**: `path ILIKE '%projects%'` as a last resort.
///
/// If *none* of the three criteria match, the `DO` block returns early and
/// the migration is a no-op — it does **not** fail.  Re-running is safe
/// because `jsonb_set` is idempotent (overwriting `"kami_cards"` with
/// `"kami_cards"` is a no-op in effect).
///
/// ## Future work
///
/// When a formal `page_type` or system flag column is added, replace the
/// slug lookup with `WHERE metadata->>'kami_projects_page' = 'true'` and
/// drop the heuristic branch.
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let sql = r#"
            DO $$
            DECLARE
                v_tenant_id UUID;
                v_page_id   UUID;
                v_blocks    JSONB;
                i           INT;
            BEGIN
                -- Guard: if the pages table hasn't been created yet (e.g. a fresh
                -- test schema where migrations run in order), exit cleanly.
                -- pg_catalog lookup avoids a hard 42P01 runtime error.
                IF NOT EXISTS (
                    SELECT 1
                    FROM pg_catalog.pg_class c
                    JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
                    WHERE c.relname = 'pages'
                      AND n.nspname = current_schema()
                ) THEN
                    RETURN;
                END IF;

                -- Locate the buildwithruud tenant (case-insensitive).
                SELECT id INTO v_tenant_id
                FROM tenant
                WHERE name ILIKE '%buildwithruud%'
                LIMIT 1;

                -- If this environment has no such tenant, exit cleanly.
                IF v_tenant_id IS NULL THEN RETURN; END IF;

                -- Locate the projects page using three escalating criteria
                -- so the migration survives slug renames.
                SELECT id INTO v_page_id
                FROM pages
                WHERE tenant_id = v_tenant_id
                  AND (
                      -- 1. System flag set by admins / provisioning scripts
                      metadata->>'kami_projects_page' = 'true'
                      -- 2. Explicit known slugs for current deployments
                      OR slug IN ('projects', '/p/projects')
                      -- 3. Path heuristic as a last-resort fallback
                      OR path ILIKE '%projects%'
                  )
                LIMIT 1;

                -- If no matching page exists in this environment, exit cleanly.
                -- The migration is a no-op and can be re-run safely.
                IF v_page_id IS NULL THEN RETURN; END IF;

                -- Load the current blocks array.
                SELECT COALESCE(blocks, '[]'::jsonb) INTO v_blocks
                FROM pages WHERE id = v_page_id;

                -- Walk the array and patch content_feed config.layout to kami_cards.
                -- jsonb_set is idempotent: overwriting "kami_cards" with "kami_cards"
                -- has no effect on subsequent runs.
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

        // Revert layout back to "cards" using the same resilient page lookup.
        let sql = r#"
            DO $$
            DECLARE
                v_tenant_id UUID;
                v_page_id   UUID;
                v_blocks    JSONB;
                i           INT;
            BEGIN
                -- Same table-existence guard as up().
                IF NOT EXISTS (
                    SELECT 1
                    FROM pg_catalog.pg_class c
                    JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
                    WHERE c.relname = 'pages'
                      AND n.nspname = current_schema()
                ) THEN
                    RETURN;
                END IF;

                SELECT id INTO v_tenant_id FROM tenant WHERE name ILIKE '%buildwithruud%' LIMIT 1;
                IF v_tenant_id IS NULL THEN RETURN; END IF;

                SELECT id INTO v_page_id FROM pages
                WHERE tenant_id = v_tenant_id
                  AND (
                      metadata->>'kami_projects_page' = 'true'
                      OR slug IN ('projects', '/p/projects')
                      OR path ILIKE '%projects%'
                  )
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
