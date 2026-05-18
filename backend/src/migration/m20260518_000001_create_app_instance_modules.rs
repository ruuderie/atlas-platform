use sea_orm_migration::prelude::*;

/// Creates the `app_instance_module` table and seeds the existing `buildwithruud`
/// tenant with its current module set, preserving backward compatibility.
///
/// ## Purpose
/// Transitions the Atlas Platform admin dashboard from a hardcoded tab array
/// to a database-driven, multi-tenant configurable module registry.
///
/// ## Schema
/// - `app_instance_id` + `module_type` → unique pair (one row per module per tenant)
/// - `sort_order` → tenant-configurable tab ordering without redeploy
/// - `is_fixed` → platform-enforced flag preventing disable of core modules
/// - `config` JSONB → optional per-module tenant configuration (future use)
///
/// ## Seed strategy for buildwithruud
/// Preserves the existing hardcoded tab state exactly. The previous "MAILING LIST"
/// tab maps to the new `CONTACTS` module_type (semantically correct rename).
/// The new `LEADS` module is seeded enabled — buildwithruud has an active lead
/// ingestion pipeline via `POST /api/v1/leads/ingest`.
///
/// ## Idempotency
/// All INSERTs use `ON CONFLICT DO NOTHING`. Safe to run multiple times.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // ── Create table ────────────────────────────────────────────────────
        db.execute_unprepared(r#"
            CREATE TABLE IF NOT EXISTS app_instance_module (
                id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
                app_instance_id UUID        NOT NULL
                    REFERENCES app_instances(id) ON DELETE CASCADE,
                module_type     TEXT        NOT NULL,
                display_name    TEXT        NOT NULL,
                icon            TEXT,
                sort_order      INTEGER     NOT NULL DEFAULT 0,
                is_enabled      BOOLEAN     NOT NULL DEFAULT true,
                is_fixed        BOOLEAN     NOT NULL DEFAULT false,
                config          JSONB,
                created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
                UNIQUE (app_instance_id, module_type)
            );
        "#).await?;

        // ── Indexes ──────────────────────────────────────────────────────────
        // Hot-path: fetch enabled modules for a tenant ordered by sort_order.
        db.execute_unprepared(r#"
            CREATE INDEX IF NOT EXISTS idx_app_instance_module_tenant
                ON app_instance_module (app_instance_id, is_enabled, sort_order);
        "#).await?;

        // ── Seed buildwithruud ───────────────────────────────────────────────
        // Locate the buildwithruud app_instance via its known domains.
        // ON CONFLICT DO NOTHING makes this idempotent.
        db.execute_unprepared(r#"
            INSERT INTO app_instance_module
                (app_instance_id, module_type, display_name, sort_order, is_fixed, is_enabled)
            SELECT
                ai.id,
                m.module_type,
                m.display_name,
                m.sort_order,
                m.is_fixed,
                true
            FROM app_instances ai
            JOIN app_domains ad ON ad.app_instance_id = ai.id
            CROSS JOIN (VALUES
                ('DASHBOARD',       'Dashboard',       0,   true),
                ('BLOG',            'Blog',            10,  false),
                ('SERVICES',        'Services',        20,  false),
                ('CASE_STUDIES',    'Case Studies',    30,  false),
                ('HIGHLIGHTS',      'Highlights',      40,  false),
                ('CONTACTS',        'Contacts',        50,  false),
                ('SETTINGS',        'Settings',        60,  true),
                ('LEAD_OPTIONS',    'Lead Options',    70,  false),
                ('NAVIGATION',      'Navigation',      80,  false),
                ('FOOTER',          'Footer',          90,  false),
                ('PAGE_HEADERS',    'Page Headers',    100, false),
                ('LANDING_PAGES',   'Landing Pages',   110, false),
                ('RESUME_PROFILES', 'Resume Profiles', 120, false),
                ('RESUME_ENTRIES',  'Resume Entries',  130, false),
                ('WEBFORMS',        'Webforms',        140, false),
                ('SECURITY',        'Security',        150, true),
                ('LEADS',           'Leads',           160, false)
            ) AS m(module_type, display_name, sort_order, is_fixed)
            WHERE ad.domain_name IN (
                'buildwithruud.com',
                'www.buildwithruud.com',
                'dev.buildwithruud.com'
            )
            ON CONFLICT (app_instance_id, module_type) DO NOTHING;
        "#).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS app_instance_module CASCADE;")
            .await?;
        Ok(())
    }
}
