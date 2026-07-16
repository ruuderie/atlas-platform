//! G-19 Direct Mail companions: PG enum fix, mail_drops, offer_codes.
//!
//! - Adds `direct_mail` to `atlas_campaign_type` (Rust already had CampaignType::DirectMail)
//! - `atlas_campaign_mail_drops` — per-creative drop under a campaign
//! - `atlas_campaign_offer_codes` — offline→online attribution backup

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // PG enum may already include the value — catch duplicate_object.
        let _ = db
            .execute_unprepared(
                r#"
                DO $$ BEGIN
                    ALTER TYPE atlas_campaign_type ADD VALUE 'direct_mail';
                EXCEPTION
                    WHEN duplicate_object THEN NULL;
                END $$;
                "#,
            )
            .await;

        db.execute_unprepared(
            r#"
            CREATE TABLE IF NOT EXISTS atlas_campaign_mail_drops (
                id UUID PRIMARY KEY,
                tenant_id UUID NOT NULL,
                campaign_id UUID NOT NULL REFERENCES atlas_campaigns(id) ON DELETE CASCADE,
                drop_name VARCHAR(200) NOT NULL,
                creative_variant VARCHAR(120),
                utm_content VARCHAR(200),
                piece_count INTEGER NOT NULL DEFAULT 0,
                unit_cost_cents BIGINT,
                provider_job_id VARCHAR(200),
                status VARCHAR(40) NOT NULL DEFAULT 'draft',
                mailed_at TIMESTAMPTZ,
                metadata JSONB,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            CREATE INDEX IF NOT EXISTS idx_mail_drops_campaign
                ON atlas_campaign_mail_drops (tenant_id, campaign_id);
            "#,
        )
        .await?;

        db.execute_unprepared(
            r#"
            CREATE TABLE IF NOT EXISTS atlas_campaign_offer_codes (
                id UUID PRIMARY KEY,
                tenant_id UUID NOT NULL,
                campaign_id UUID NOT NULL REFERENCES atlas_campaigns(id) ON DELETE CASCADE,
                mail_drop_id UUID REFERENCES atlas_campaign_mail_drops(id) ON DELETE SET NULL,
                code VARCHAR(64) NOT NULL,
                is_active BOOLEAN NOT NULL DEFAULT TRUE,
                redemption_count INTEGER NOT NULL DEFAULT 0,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE (tenant_id, code)
            );
            CREATE INDEX IF NOT EXISTS idx_offer_codes_campaign
                ON atlas_campaign_offer_codes (campaign_id);
            CREATE UNIQUE INDEX IF NOT EXISTS idx_offer_codes_code_lower
                ON atlas_campaign_offer_codes (lower(code));
            "#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("DROP TABLE IF EXISTS atlas_campaign_offer_codes;")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS atlas_campaign_mail_drops;")
            .await?;
        // Cannot remove enum value from PG easily — leave direct_mail in place.
        Ok(())
    }
}
