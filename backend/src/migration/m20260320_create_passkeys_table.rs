use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Passkey::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Passkey::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Passkey::UserId).uuid().not_null())
                    .col(ColumnDef::new(Passkey::CredentialId).binary().not_null())
                    .col(ColumnDef::new(Passkey::PublicKey).binary().not_null())
                    .col(ColumnDef::new(Passkey::SignCount).integer().not_null())
                    .col(ColumnDef::new(Passkey::Name).string().not_null())
                    .col(ColumnDef::new(Passkey::LastUsedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Passkey::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Passkey::UpdatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_passkey_user_id")
                            .from(Passkey::Table, Passkey::UserId)
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
                    .name("idx_passkey_user_id")
                    .table(Passkey::Table)
                    .col(Passkey::UserId)
                    .to_owned(),
            )
            .await?;
            
        manager
            .create_index(
                Index::create()
                    .name("idx_passkey_credential_id")
                    .table(Passkey::Table)
                    .col(Passkey::CredentialId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Passkey::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Passkey {
    Table,
    Id,
    UserId,
    CredentialId,
    PublicKey,
    SignCount,
    Name,
    LastUsedAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum User {
    Table,
    Id,
}
