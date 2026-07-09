#![allow(dead_code)]

//! Migration: add review moderation fields to `atlas_rating_sessions`.
//!
//! Extends the generic G-27 session table to support vendor review testimonials
//! and platform moderation — without a separate vendor-review table.
//!
//! Columns added:
//!   `testimonial`    TEXT NULL  — free-text review body written by the rater.
//!   `is_flagged`     BOOL       — auto-flag trigger (fraud signals) or manual flag.
//!   `flag_reason`    TEXT NULL  — machine or moderator reason for flag.
//!   `published_at`   TIMESTAMPTZ NULL — NULL = held for moderation; non-null = live.
//!
//! Moderation flow:
//!   1. Rater submits → `published_at = NULL` (held).
//!   2. Platform admin reviews (or auto-clears if no fraud signals) → sets `published_at`.
//!   3. `GET /api/pub/vendors/:sp_id` only returns rows WHERE published_at IS NOT NULL.
//!   4. Vendor cannot set `published_at` — write is restricted to platform-admin role.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE atlas_rating_sessions \
                 ADD COLUMN IF NOT EXISTS testimonial   TEXT NULL, \
                 ADD COLUMN IF NOT EXISTS is_flagged    BOOLEAN NOT NULL DEFAULT FALSE, \
                 ADD COLUMN IF NOT EXISTS flag_reason   TEXT NULL, \
                 ADD COLUMN IF NOT EXISTS published_at  TIMESTAMPTZ NULL;",
            )
            .await?;

        // Partial index: fast lookup of published reviews for a given scorecard.
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE INDEX IF NOT EXISTS idx_rating_sessions_published \
                 ON atlas_rating_sessions (scorecard_id, published_at DESC) \
                 WHERE published_at IS NOT NULL;",
            )
            .await?;

        // Partial index: moderation queue — sessions with testimonial awaiting publish.
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE INDEX IF NOT EXISTS idx_rating_sessions_moderation_queue \
                 ON atlas_rating_sessions (tenant_id, is_flagged, created_at DESC) \
                 WHERE testimonial IS NOT NULL AND published_at IS NULL;",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DROP INDEX IF EXISTS idx_rating_sessions_moderation_queue; \
                 DROP INDEX IF EXISTS idx_rating_sessions_published;",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE atlas_rating_sessions \
                 DROP COLUMN IF EXISTS testimonial, \
                 DROP COLUMN IF EXISTS is_flagged, \
                 DROP COLUMN IF EXISTS flag_reason, \
                 DROP COLUMN IF EXISTS published_at;",
            )
            .await?;

        Ok(())
    }
}
