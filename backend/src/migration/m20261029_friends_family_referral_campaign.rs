//! m20261029_friends_family_referral_campaign — Friends & Family referral seed
//!
//! Seeds:
//! - Active `atlas_campaigns` row (type=referral, utm_campaign=friends_family)
//! - Matching `app_utm_presets` row for Folio LP builder
//!
//! Waitlist signups from `/refer` send these UTMs and auto-enroll into this campaign.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const CAMPAIGN_ID: &str = "00000000-0000-0000-0002-000000000001";
const UTM_PRESET_ID: &str = "00000000-0000-0000-0002-000000000002";
const SENTINEL_TENANT: &str = "00000000-0000-0000-0000-000000000000";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(&format!(
            r#"
            INSERT INTO atlas_campaigns (
                id, tenant_id, parent_campaign_id, name, campaign_type, status,
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
                '{CAMPAIGN_ID}'::uuid,
                '{SENTINEL_TENANT}'::uuid,
                NULL,
                'Friends & Family',
                'referral',
                'active',
                NULL, NULL,
                'lead_capture', NULL, NULL,
                NULL, 'USD', 0, 30,
                NULL, NULL,
                NULL, NULL,
                NOW(), NULL,
                'referral', 'friends_family', 'friends_family',
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
                '{UTM_PRESET_ID}'::uuid,
                'folio',
                'Friends & Family Referral',
                'referral',
                'friends_family',
                'friends_family',
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

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(&format!(
            "DELETE FROM app_utm_presets WHERE id = '{UTM_PRESET_ID}'::uuid;"
        ))
        .await?;
        db.execute_unprepared(&format!(
            "DELETE FROM atlas_campaigns WHERE id = '{CAMPAIGN_ID}'::uuid;"
        ))
        .await?;
        Ok(())
    }
}
