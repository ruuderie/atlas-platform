use sea_orm_migration::prelude::*;

/// Canonicalize tenant_setting table from app_instances.settings JSONB.
///
/// Root cause of UAT data issue (2026-04-30):
///   - buildwithruud: all settings were in app_instances.settings JSONB, zero rows in tenant_setting.
///     The anchor get_site_settings() server fn reads ONLY tenant_setting → everything defaulted.
///   - OplystUSA: settings were in tenant_setting but with lc_* keys; the server fn reads lead_capture_*.
///
/// This migration:
///   1. Extracts flat settings from app_instances.settings and upserts them into tenant_setting
///      (with lc_* → lead_capture_* remapping).
///   2. For any tenant with lc_* keys, aliases them to lead_capture_* keys.
///   3. Seeds the design_config key from app_instances.settings (needed for onboarding step check).
///
/// Safe to re-run on any environment — all INSERTs use ON CONFLICT DO UPDATE.
/// Will no-op on fresh deployments where buildwithruud is seeded correctly already.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // ── 1. For all anchor app_instances with settings, extract flat KV pairs ──────
        // Map lc_* → lead_capture_* (the form the server fn actually reads).
        let sql = r#"
            WITH flat AS (
                SELECT
                    ai.tenant_id,
                    kv.key,
                    kv.value
                FROM app_instances ai,
                LATERAL (VALUES
                    ('current_focus',             ai.settings->>'current_focus'),
                    ('status',                     ai.settings->>'status'),
                    ('hero_quote',                 ai.settings->>'hero_quote'),
                    ('hero_subtitle',              ai.settings->>'hero_subtitle'),
                    ('site_title',                 ai.settings->>'site_title'),
                    ('lead_capture_title',         ai.settings->>'lc_title'),
                    ('lead_capture_desc',          ai.settings->>'lc_desc'),
                    ('lead_capture_label',         ai.settings->>'lc_label'),
                    ('lead_capture_placeholder',   ai.settings->>'lc_placeholder'),
                    ('lead_capture_btn',           ai.settings->>'lc_btn'),
                    ('lead_capture_footer',        ai.settings->>'lc_footer'),
                    ('lead_capture_endpoint',      ai.settings->>'lc_endpoint'),
                    ('status_color',               ai.settings->>'status_color'),
                    ('webhook_url',                ai.settings->>'webhook_url'),
                    ('admin_email',                ai.settings->>'admin_email'),
                    ('google_analytics_id',        ai.settings->>'google_analytics_id'),
                    ('booking_url',                ai.settings->>'booking_url'),
                    ('terms_html',                 ai.settings->>'terms_html'),
                    ('privacy_html',               ai.settings->>'privacy_html'),
                    ('github_url',                 ai.settings->>'github_url'),
                    ('x_url',                      ai.settings->>'x_url'),
                    ('linkedin_url',               ai.settings->>'linkedin_url'),
                    ('b2b_enabled',                CASE WHEN (ai.settings->>'b2b_enabled')::boolean THEN 'true' ELSE 'false' END),
                    ('meta_title',                 ai.settings->>'meta_title'),
                    ('meta_description',           ai.settings->>'meta_description'),
                    ('og_image',                   ai.settings->>'og_image'),
                    ('design_config',              (ai.settings->'design_config')::text)
                ) AS kv(key, value)
                WHERE ai.app_type = 'anchor'
                  AND ai.settings IS NOT NULL
                  AND kv.value IS NOT NULL
                  AND kv.value != 'null'
                  AND kv.value != ''
            )
            INSERT INTO tenant_setting (id, tenant_id, key, value, created_at, updated_at)
            SELECT gen_random_uuid(), tenant_id, key, value, NOW(), NOW()
            FROM flat
            ON CONFLICT (tenant_id, key) DO UPDATE
                SET value = EXCLUDED.value, updated_at = NOW();
        "#;

        db.execute_unprepared(sql).await?;

        // ── 2. Alias any legacy lc_* keys → lead_capture_* ─────────────────────────
        // Handles OplystUSA (and any future tenant) that was seeded with lc_* keys directly.
        let alias_sql = r#"
            WITH lc_map(old_key, new_key) AS (VALUES
                ('lc_title',       'lead_capture_title'),
                ('lc_desc',        'lead_capture_desc'),
                ('lc_label',       'lead_capture_label'),
                ('lc_placeholder', 'lead_capture_placeholder'),
                ('lc_btn',         'lead_capture_btn'),
                ('lc_footer',      'lead_capture_footer'),
                ('lc_endpoint',    'lead_capture_endpoint')
            )
            INSERT INTO tenant_setting (id, tenant_id, key, value, created_at, updated_at)
            SELECT gen_random_uuid(), ts.tenant_id, m.new_key, ts.value, NOW(), NOW()
            FROM lc_map m
            JOIN tenant_setting ts ON ts.key = m.old_key
            ON CONFLICT (tenant_id, key) DO UPDATE
                SET value = EXCLUDED.value, updated_at = NOW();
        "#;

        db.execute_unprepared(alias_sql).await?;

        tracing::info!("m20260501_000001: tenant_setting canonicalized from app_instances.settings");

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // This migration is additive only — no destructive rollback.
        // Rows can be left in tenant_setting; they are idempotent KV pairs.
        Ok(())
    }
}
