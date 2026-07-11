use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// GENERIC-02: atlas_vault — Secure File Storage Extension
///
/// Extends the existing `attachment` table with R2/Cloudflare storage fields
/// and adds two supporting tables:
/// - attachment_share_tokens (for guest/external access)
/// - attachment_multipart_uploads (for large file uploads)
///
/// This is the highest priority infrastructure generic per the roadmap
/// because many vertical apps (PM, ClaimSwift, etc.) need secure file + sharing immediately.
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Extend existing attachment table (idempotent)
        manager
            .get_connection()
            .execute(sea_orm::Statement::from_string(
                manager.get_connection().get_database_backend(),
                r#"
                ALTER TABLE attachment
                    ADD COLUMN IF NOT EXISTS access_level VARCHAR(30) DEFAULT 'private',
                    ADD COLUMN IF NOT EXISTS r2_bucket VARCHAR(100),
                    ADD COLUMN IF NOT EXISTS r2_key VARCHAR(512),
                    ADD COLUMN IF NOT EXISTS checksum_sha256 VARCHAR(64),
                    ADD COLUMN IF NOT EXISTS upload_status VARCHAR(20) DEFAULT 'complete';
                "#
                .to_owned(),
            ))
            .await?;

        // attachment_share_tokens table
        manager
            .create_table(
                Table::create()
                    .table(AttachmentShareTokens::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AttachmentShareTokens::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(AttachmentShareTokens::TenantId).uuid().not_null())
                    .col(ColumnDef::new(AttachmentShareTokens::AttachmentId).uuid().not_null())
                    .col(
                        ColumnDef::new(AttachmentShareTokens::Token)
                            .string_len(128)
                            .not_null()
                            .unique_key()
                            .default(Expr::cust("md5(gen_random_uuid()::text) || md5(gen_random_uuid()::text) || md5(gen_random_uuid()::text)")),
                    )
                    .col(ColumnDef::new(AttachmentShareTokens::ResourceType).string().not_null())
                    .col(
                        ColumnDef::new(AttachmentShareTokens::Permissions)
                            .json_binary()
                            .not_null()
                            .default(Expr::val("[]")),
                    )
                    .col(ColumnDef::new(AttachmentShareTokens::RecipientEmail).string().null())
                    .col(ColumnDef::new(AttachmentShareTokens::ExpiresAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(AttachmentShareTokens::OneTimeUse).boolean().not_null().default(false))
                    .col(ColumnDef::new(AttachmentShareTokens::UsedAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(AttachmentShareTokens::CreatedByUserId).uuid().null())
                    .col(
                        ColumnDef::new(AttachmentShareTokens::CreatedAt)
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
                    .name("idx_attachment_share_tokens_token")
                    .table(AttachmentShareTokens::Table)
                    .col(AttachmentShareTokens::Token)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_attachment_share_tokens_attachment")
                    .table(AttachmentShareTokens::Table)
                    .col(AttachmentShareTokens::AttachmentId)
                    .to_owned(),
            )
            .await?;

        // attachment_multipart_uploads table
        manager
            .create_table(
                Table::create()
                    .table(AttachmentMultipartUploads::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AttachmentMultipartUploads::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(AttachmentMultipartUploads::TenantId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AttachmentMultipartUploads::AttachmentId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AttachmentMultipartUploads::R2UploadId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AttachmentMultipartUploads::TotalParts)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AttachmentMultipartUploads::CompletedParts)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(AttachmentMultipartUploads::Status)
                            .string()
                            .not_null()
                            .default(Expr::val("in_progress")),
                    )
                    .col(
                        ColumnDef::new(AttachmentMultipartUploads::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(AttachmentMultipartUploads::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(AttachmentShareTokens::Table).to_owned())
            .await?;

        // Note: We do not drop the added columns on attachment in down() for safety in POC.
        // A real migration would have a corresponding ALTER TABLE DROP COLUMN IF EXISTS.
        Ok(())
    }
}

#[derive(DeriveIden)]
enum AttachmentShareTokens {
    Table,
    Id,
    TenantId,
    AttachmentId,
    Token,
    ResourceType,
    Permissions,
    RecipientEmail,
    ExpiresAt,
    OneTimeUse,
    UsedAt,
    CreatedByUserId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum AttachmentMultipartUploads {
    Table,
    Id,
    TenantId,
    AttachmentId,
    R2UploadId,
    TotalParts,
    CompletedParts,
    Status,
    CreatedAt,
}
