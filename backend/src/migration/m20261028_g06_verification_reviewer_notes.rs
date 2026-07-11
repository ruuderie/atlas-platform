//! G-06: add `request_type` + `reviewer_notes` to `atlas_verification_requests`.
//!
//! `request_type` separates the verification category (business/identity/document)
//! from polymorphic `subject_type` (tenant/user/asset).
//! `reviewer_notes` persists admin review notes and request-more-info messages.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE atlas_verification_requests \
                 ADD COLUMN IF NOT EXISTS request_type VARCHAR(50) NULL, \
                 ADD COLUMN IF NOT EXISTS reviewer_notes TEXT NULL;",
            )
            .await?;

        // Backfill request_type from legacy subject_type values used as categories.
        manager
            .get_connection()
            .execute_unprepared(
                "UPDATE atlas_verification_requests \
                 SET request_type = lower(subject_type) \
                 WHERE request_type IS NULL \
                   AND lower(subject_type) IN ('business', 'identity', 'document', 'kyc');",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "UPDATE atlas_verification_requests \
                 SET request_type = 'identity' \
                 WHERE request_type IS NULL \
                   AND lower(subject_type) = 'kyc';",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE atlas_verification_requests \
                 DROP COLUMN IF EXISTS reviewer_notes, \
                 DROP COLUMN IF EXISTS request_type;",
            )
            .await?;
        Ok(())
    }
}
