use sea_orm_migration::prelude::*;

/// Add `invite_code_id` to `platform_invite` — linking completed invites back
/// to the invite code that initiated them.
///
/// When a user completes onboarding via an invite code URL (/join/{code}),
/// the resulting `platform_invite` row links back to the originating code.
/// This enables:
///   - Audit trail: which code brought in this user
///   - Analytics: conversion rate per code
///   - Deduplication: prevent same user from using a single-use code twice
///   - Admin visibility: "12 tenants signed up via code OAK4B-2025"
///
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"ALTER TABLE platform_invite
                    ADD COLUMN IF NOT EXISTS invite_code_id UUID
                        REFERENCES atlas_invite_codes(id) ON DELETE SET NULL;

                   -- Index for "show all invites from this code"
                   CREATE INDEX IF NOT EXISTS idx_platform_invite_code
                       ON platform_invite(invite_code_id)
                       WHERE invite_code_id IS NOT NULL;
                "#,
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"DROP INDEX IF EXISTS idx_platform_invite_code;
                   ALTER TABLE platform_invite DROP COLUMN IF EXISTS invite_code_id;
                "#,
            )
            .await?;

        Ok(())
    }
}
