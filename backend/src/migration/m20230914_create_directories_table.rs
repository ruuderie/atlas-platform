use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Directory::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Directory::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Directory::DirectoryTypeId).uuid().not_null())
                    .col(ColumnDef::new(Directory::Name).string().not_null())
                    .col(ColumnDef::new(Directory::Domain).string().not_null())
                    .col(ColumnDef::new(Directory::Description).string().not_null())
                    .col(ColumnDef::new(Directory::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Directory::UpdatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(ForeignKey::create()
                        .name("fk-directory-directory_type_id")
                        .from(Directory::Table, Directory::DirectoryTypeId)
                        .to(DirectoryType::Table, DirectoryType::Id))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Directory::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Directory {
    Table,
    Id,
    DirectoryTypeId,
    Name,
    Domain,
    Description,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum DirectoryType {
    Table,
    Id,
}