use sea_orm_migration::prelude::*;

/// Adds `is_setup_token BOOLEAN NOT NULL DEFAULT false` to `magic_link_token`.
///
/// This flag distinguishes between first-time setup tokens (which should
/// wipe any existing passkeys and force a fresh registration) and standard
/// magic link logins (which should preserve existing passkeys).
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
                        ColumnDef::new(MagicLinkToken::IsSetupToken)
                            .boolean()
                            .not_null()
                            .default(false),
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
                    .drop_column(MagicLinkToken::IsSetupToken)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum MagicLinkToken {
    Table,
    IsSetupToken,
}
