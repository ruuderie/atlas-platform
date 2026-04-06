use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Profile::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Profile::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Profile::AccountId).uuid().not_null())
                    .col(ColumnDef::new(Profile::DirectoryId).uuid().not_null())
                    .col(ColumnDef::new(Profile::ProfileType).string().not_null())
                    .col(ColumnDef::new(Profile::DisplayName).string().not_null())
                    .col(ColumnDef::new(Profile::ContactInfo).string().not_null())
                    .col(ColumnDef::new(Profile::BusinessName).string())
                    .col(ColumnDef::new(Profile::BusinessAddress).string())
                    .col(ColumnDef::new(Profile::BusinessPhone).string())
                    .col(ColumnDef::new(Profile::BusinessWebsite).string())
                    .col(ColumnDef::new(Profile::AdditionalInfo).json())
                    .col(ColumnDef::new(Profile::IsActive).boolean().not_null().default(true))
                    .col(ColumnDef::new(Profile::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Profile::UpdatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(ForeignKey::create()
                        .name("fk-profile-account_id")
                        .from(Profile::Table, Profile::AccountId)
                        .to(Account::Table, Account::Id))
                    .foreign_key(ForeignKey::create()
                        .name("fk-profile-directory_id")
                        .from(Profile::Table, Profile::DirectoryId)
                        .to(Directory::Table, Directory::Id))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Profile::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Profile {
    Table,
    Id,
    AccountId,
    DirectoryId,
    ProfileType,
    DisplayName,
    ContactInfo,
    BusinessName,
    BusinessAddress,
    BusinessPhone,
    BusinessWebsite,
    AdditionalInfo,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Account {
    Table,
    Id,
}

#[derive(Iden)]
enum Directory {
    Table,
    Id,
}