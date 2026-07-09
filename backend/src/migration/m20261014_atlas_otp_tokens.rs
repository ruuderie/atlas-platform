// backend/src/migration/m20261014_atlas_otp_tokens.rs
//
// OTP tokens for the inline wizard pre-auth flow.
//
// Used by POST /api/auth/otp/send and POST /api/auth/otp/verify.
// These are short-lived (5 min), single-use codes sent to email.
// They replace the magic-link context switch for cold QR/direct-mail flows.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AtlasOtpTokens::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(AtlasOtpTokens::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(AtlasOtpTokens::UserId).uuid().not_null())
                    .col(ColumnDef::new(AtlasOtpTokens::CodeHash).string().not_null())
                    .col(ColumnDef::new(AtlasOtpTokens::ExpiresAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(AtlasOtpTokens::IsUsed).boolean().not_null().default(false))
                    .col(ColumnDef::new(AtlasOtpTokens::CreatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_otp_tokens_user_id")
                            .from(AtlasOtpTokens::Table, AtlasOtpTokens::UserId)
                            .to(Alias::new("user"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Index for fast lookup by user_id (verify path)
        manager
            .create_index(
                Index::create()
                    .name("idx_otp_tokens_user_id")
                    .table(AtlasOtpTokens::Table)
                    .col(AtlasOtpTokens::UserId)
                    .to_owned(),
            )
            .await?;

        // Partial index: only active (unused + unexpired) tokens — keeps the
        // verify query fast without a full table scan.
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE INDEX IF NOT EXISTS idx_otp_tokens_active \
                 ON atlas_otp_tokens (user_id, expires_at) \
                 WHERE is_used = false",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AtlasOtpTokens::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AtlasOtpTokens {
    Table,
    Id,
    UserId,
    CodeHash,
    ExpiresAt,
    IsUsed,
    CreatedAt,
}
