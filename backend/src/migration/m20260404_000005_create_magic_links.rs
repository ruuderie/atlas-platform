use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(MagicLinkToken::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(MagicLinkToken::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(MagicLinkToken::UserId).uuid().not_null())
                    .col(ColumnDef::new(MagicLinkToken::Token).string().not_null())
                    .col(ColumnDef::new(MagicLinkToken::ExpiresAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(MagicLinkToken::IsUsed).boolean().not_null().default(false))
                    .col(ColumnDef::new(MagicLinkToken::CreatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_magic_link_user_id")
                            .from(MagicLinkToken::Table, MagicLinkToken::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_magic_link_token")
                    .table(MagicLinkToken::Table)
                    .col(MagicLinkToken::Token)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(MagicLinkToken::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum MagicLinkToken {
    Table,
    Id,
    UserId,
    Token,
    ExpiresAt,
    IsUsed,
    CreatedAt,
}

#[derive(Iden)]
enum User {
    Table,
    Id,
}
