use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(DirectoryType::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(DirectoryType::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(DirectoryType::Name).string().not_null())
                    .col(ColumnDef::new(DirectoryType::Description).string().not_null())
                    .col(ColumnDef::new(DirectoryType::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(DirectoryType::UpdatedAt).timestamp_with_time_zone().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(DirectoryType::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum DirectoryType {
    Table,
    Id,
    Name,
    Description,
    CreatedAt,
    UpdatedAt,
}