//! m20261030_campaign_global_name_ff_vendors_g37_ambassadors
//!
//! - Add `atlas_campaigns.global_name` (UNIQUE snake_case human id)
//! - Backfill existing campaigns; rename F&F seed global_name
//! - Seed Friends & Family Vendors child campaign + UTM preset
//! - G-37: `atlas_ambassadors` + `atlas_ambassador_campaigns`

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const FF_LANDLORD_ID: &str = "00000000-0000-0000-0002-000000000001";
const FF_VENDOR_ID: &str = "00000000-0000-0000-0002-000000000003";
const FF_VENDOR_UTM_PRESET_ID: &str = "00000000-0000-0000-0002-000000000004";
const SENTINEL_TENANT: &str = "00000000-0000-0000-0000-000000000000";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // ── global_name column ─────────────────────────────────────────────
        db.execute_unprepared(
            r#"
            ALTER TABLE atlas_campaigns
                ADD COLUMN IF NOT EXISTS global_name VARCHAR(200);
            "#,
        )
        .await?;

        // Backfill: prefer folio_ + slug(name); F&F known id gets canonical value
        db.execute_unprepared(
            r#"
            UPDATE atlas_campaigns
            SET global_name = 'folio_' || lower(regexp_replace(
                    regexp_replace(trim(name), '[^a-zA-Z0-9]+', '_', 'g'),
                    '_+', '_', 'g'
                ))
            WHERE global_name IS NULL OR global_name = '';
            "#,
        )
        .await?;

        db.execute_unprepared(&format!(
            r#"
            UPDATE atlas_campaigns
            SET global_name = 'folio_friends_family'
            WHERE id = '{FF_LANDLORD_ID}'::uuid;
            "#
        ))
        .await?;

        // Deduplicate any collisions by appending short id suffix
        db.execute_unprepared(
            r#"
            WITH dups AS (
                SELECT id, global_name,
                       ROW_NUMBER() OVER (PARTITION BY global_name ORDER BY created_at, id) AS rn
                FROM atlas_campaigns
            )
            UPDATE atlas_campaigns c
            SET global_name = c.global_name || '_' || substr(replace(c.id::text, '-', ''), 1, 8)
            FROM dups d
            WHERE c.id = d.id AND d.rn > 1;
            "#,
        )
        .await?;

        db.execute_unprepared(
            r#"
            ALTER TABLE atlas_campaigns
                ALTER COLUMN global_name SET NOT NULL;
            "#,
        )
        .await?;

        db.execute_unprepared(
            r#"
            CREATE UNIQUE INDEX IF NOT EXISTS idx_atlas_campaigns_global_name
                ON atlas_campaigns (global_name);
            "#,
        )
        .await?;

        // ── Friends & Family Vendors campaign ──────────────────────────────
        db.execute_unprepared(&format!(
            r#"
            INSERT INTO atlas_campaigns (
                id, tenant_id, parent_campaign_id, name, global_name, campaign_type, status,
                audience_segment_id, audience_filter,
                goal_type, goal_entity_type, target_conversion_count,
                budget_cents, currency, spent_cents, attribution_window_days,
                external_campaign_id, integration_id,
                subject_entity_type, subject_entity_id,
                starts_at, ends_at,
                utm_source, utm_medium, utm_campaign,
                total_contacts, total_opens, total_clicks, total_replies, total_conversions,
                created_by_user_id, created_at, updated_at
            )
            VALUES (
                '{FF_VENDOR_ID}'::uuid,
                '{SENTINEL_TENANT}'::uuid,
                '{FF_LANDLORD_ID}'::uuid,
                'Friends & Family Vendors',
                'folio_friends_family_vendors',
                'referral',
                'active',
                NULL, NULL,
                'lead_capture', NULL, NULL,
                NULL, 'USD', 0, 30,
                NULL, NULL,
                NULL, NULL,
                NOW(), NULL,
                'referral', 'friends_family', 'friends_family_vendors',
                0, 0, 0, 0, 0,
                NULL, NOW(), NOW()
            )
            ON CONFLICT (id) DO NOTHING;
            "#
        ))
        .await?;

        db.execute_unprepared(&format!(
            r#"
            INSERT INTO app_utm_presets (
                id, app_id, name,
                utm_source, utm_medium, utm_campaign, utm_content, utm_term,
                click_count, created_at, updated_at
            )
            VALUES (
                '{FF_VENDOR_UTM_PRESET_ID}'::uuid,
                'folio',
                'Friends & Family Vendors Referral',
                'referral',
                'friends_family',
                'friends_family_vendors',
                NULL,
                NULL,
                0,
                NOW(),
                NOW()
            )
            ON CONFLICT (id) DO NOTHING;
            "#
        ))
        .await?;

        // ── G-37 ambassadors ───────────────────────────────────────────────
        db.execute_unprepared(
            r#"
            CREATE TABLE IF NOT EXISTS atlas_ambassadors (
                id UUID PRIMARY KEY,
                tenant_id UUID NOT NULL,
                code VARCHAR(64) NOT NULL,
                display_name VARCHAR(200) NOT NULL,
                partner_type VARCHAR(32) NOT NULL
                    CHECK (partner_type IN ('referral', 'influencer', 'affiliate', 'recruiter')),
                status VARCHAR(32) NOT NULL DEFAULT 'active'
                    CHECK (status IN ('active', 'disabled')),
                account_id UUID,
                contact_id UUID,
                notes TEXT,
                channels JSONB,
                fulfillment_requests JSONB NOT NULL DEFAULT '[]'::jsonb,
                created_by_user_id UUID,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE (tenant_id, code)
            );
            "#,
        )
        .await?;

        db.execute_unprepared(
            r#"
            CREATE TABLE IF NOT EXISTS atlas_ambassador_campaigns (
                ambassador_id UUID NOT NULL REFERENCES atlas_ambassadors(id) ON DELETE CASCADE,
                campaign_id UUID NOT NULL REFERENCES atlas_campaigns(id) ON DELETE CASCADE,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                PRIMARY KEY (ambassador_id, campaign_id)
            );
            "#,
        )
        .await?;

        db.execute_unprepared(
            r#"
            CREATE INDEX IF NOT EXISTS idx_atlas_ambassadors_tenant_status
                ON atlas_ambassadors (tenant_id, status);
            CREATE INDEX IF NOT EXISTS idx_atlas_ambassador_campaigns_campaign
                ON atlas_ambassador_campaigns (campaign_id);
            "#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("DROP TABLE IF EXISTS atlas_ambassador_campaigns;")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS atlas_ambassadors;")
            .await?;
        db.execute_unprepared(&format!(
            "DELETE FROM app_utm_presets WHERE id = '{FF_VENDOR_UTM_PRESET_ID}'::uuid;"
        ))
        .await?;
        db.execute_unprepared(&format!(
            "DELETE FROM atlas_campaigns WHERE id = '{FF_VENDOR_ID}'::uuid;"
        ))
        .await?;
        db.execute_unprepared(
            "DROP INDEX IF EXISTS idx_atlas_campaigns_global_name;",
        )
        .await?;
        db.execute_unprepared(
            "ALTER TABLE atlas_campaigns DROP COLUMN IF EXISTS global_name;",
        )
        .await?;
        Ok(())
    }
}
