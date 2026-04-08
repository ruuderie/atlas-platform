use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserAccount::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(UserAccount::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(UserAccount::UserId).uuid().not_null())
                    .col(ColumnDef::new(UserAccount::AccountId).uuid().not_null())
                    .col(ColumnDef::new(UserAccount::Role).string().not_null())
                    .col(ColumnDef::new(UserAccount::IsActive).boolean().not_null().default(true))
                    .col(ColumnDef::new(UserAccount::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(UserAccount::UpdatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(ForeignKey::create()
                        .name("fk-user_account-user_id")
                        .from(UserAccount::Table, UserAccount::UserId)
                        .to(User::Table, User::Id))
                    .foreign_key(ForeignKey::create()
                        .name("fk-user_account-account_id")
                        .from(UserAccount::Table, UserAccount::AccountId)
                        .to(Account::Table, Account::Id))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserAccount::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum UserAccount {
    Table,
    Id,
    UserId,
    AccountId,
    Role,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum User {
    Table,
    Id,
}

#[derive(Iden)]
enum Account {
    Table,
    Id,
}