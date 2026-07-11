use sea_orm_migration::prelude::*;

/// Adds `redirect_url TEXT NULL` to `magic_link_token`.
///
/// Storing the redirect URL in the token row means the email link builder has
/// access to it without requiring the caller to re-supply it on verify, and
/// allows future audit queries to see where each token was directed.
///
/// Nullable because:
///  - Existing tokens predating this migration have no redirect URL.
///  - Platform-admin-originated tokens (no app context) fall back to ADMIN_URL.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(MagicLinkToken::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(MagicLinkToken::RedirectUrl).text().null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(MagicLinkToken::Table)
                    .drop_column(MagicLinkToken::RedirectUrl)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum MagicLinkToken {
    Table,
    RedirectUrl,
}
