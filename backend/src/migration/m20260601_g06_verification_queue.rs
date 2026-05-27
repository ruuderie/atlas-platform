use sea_orm_migration::prelude::*;
use sea_orm_migration::prelude::extension::postgres::Type;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// GENERIC-06: atlas_verification_queue — Human-in-the-Loop Trust Verification
///
/// Provides a standardized way to track verification workflows that may require
/// automated checks followed by manual human review (licenses, permits, identity,
/// GPS fraud checks, document validation, etc.).
///
/// This is intentionally generic so it can serve many verticals without each
/// app building its own verification state machine.
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Status enum
        manager
            .create_type(
                Type::create()
                    .as_enum(AtlasVerificationStatus::Table)
                    .values([
                        AtlasVerificationStatus::PendingUpload,
                        AtlasVerificationStatus::AutoChecking,
                        AtlasVerificationStatus::RequiresManualReview,
                        AtlasVerificationStatus::Approved,
                        AtlasVerificationStatus::Rejected,
                        AtlasVerificationStatus::Expired,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AtlasVerificationRequest::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasVerificationRequest::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(AtlasVerificationRequest::TenantId).uuid().not_null())
                    .col(ColumnDef::new(AtlasVerificationRequest::SubjectType).string().not_null())
                    .col(ColumnDef::new(AtlasVerificationRequest::SubjectId).uuid().not_null())
                    .col(ColumnDef::new(AtlasVerificationRequest::RequestedByUserId).uuid().not_null())
                    .col(ColumnDef::new(AtlasVerificationRequest::AttachmentId).uuid().null())
                    .col(ColumnDef::new(AtlasVerificationRequest::AutoCheckResult).json_binary().null())
                    .col(ColumnDef::new(AtlasVerificationRequest::AutoCheckPassed).boolean().null())
                    .col(
                        ColumnDef::new(AtlasVerificationRequest::Status)
                            .custom(AtlasVerificationStatus::Table)
                            .not_null()
                            .default(Expr::val("pending_upload")),
                    )
                    .col(ColumnDef::new(AtlasVerificationRequest::ReviewedByUserId).uuid().null())
                    .col(ColumnDef::new(AtlasVerificationRequest::ReviewedAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(AtlasVerificationRequest::RejectionReason).text().null())
                    .col(ColumnDef::new(AtlasVerificationRequest::VerifiedValue).string().null())
                    .col(ColumnDef::new(AtlasVerificationRequest::ExpiresAt).date().null())
                    .col(
                        ColumnDef::new(AtlasVerificationRequest::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_verification_requests_status")
                    .table(AtlasVerificationRequest::Table)
                    .col(AtlasVerificationRequest::TenantId)
                    .col(AtlasVerificationRequest::SubjectType)
                    .col(AtlasVerificationRequest::Status)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AtlasVerificationRequest::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(AtlasVerificationStatus::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AtlasVerificationRequest {
    Table,
    Id,
    TenantId,
    SubjectType,
    SubjectId,
    RequestedByUserId,
    AttachmentId,
    AutoCheckResult,
    AutoCheckPassed,
    Status,
    ReviewedByUserId,
    ReviewedAt,
    RejectionReason,
    VerifiedValue,
    ExpiresAt,
    CreatedAt,
}

#[derive(DeriveIden)]
enum AtlasVerificationStatus {
    Table,
    PendingUpload,
    AutoChecking,
    RequiresManualReview,
    Approved,
    Rejected,
    Expired,
}
