#![allow(dead_code)]

//! Migration: add `invite_purpose` and `context_entity_id` to `platform_invite`.
//!
//! These two columns allow a single invite table to serve both onboarding invites
//! (the existing behaviour) and vendor-initiated G-27 review requests.
//!
//! `invite_purpose` — VARCHAR discriminator typed as `crate::types::pm::InvitePurpose`.
//!   Default: `'onboarding'` — fully backward compatible; all existing rows stay valid.
//!
//! `context_entity_id` — nullable UUID foreign key to the entity that generated the
//!   invite. For `invite_purpose = 'review_request'` this is `atlas_service_providers.id`.
//!   NULL for onboarding invites.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add invite_purpose with default 'onboarding' — backward compatible.
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE platform_invite \
                 ADD COLUMN IF NOT EXISTS invite_purpose TEXT NOT NULL DEFAULT 'onboarding' \
                 CHECK (invite_purpose IN ('onboarding', 'review_request'));",
            )
            .await?;

        // Add context_entity_id — nullable UUID; no FK constraint (polymorphic ref).
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE platform_invite \
                 ADD COLUMN IF NOT EXISTS context_entity_id UUID NULL;",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "CREATE INDEX IF NOT EXISTS idx_platform_invite_review_purpose \
                 ON platform_invite (context_entity_id) \
                 WHERE invite_purpose = 'review_request';",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP INDEX IF EXISTS idx_platform_invite_review_purpose;")
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE platform_invite \
                 DROP COLUMN IF EXISTS context_entity_id, \
                 DROP COLUMN IF EXISTS invite_purpose;",
            )
            .await?;

        Ok(())
    }
}
