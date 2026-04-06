use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Template::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Template::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Template::DirectoryId).uuid().not_null())
                    .col(ColumnDef::new(Template::CategoryId).uuid().not_null())
                    .col(ColumnDef::new(Template::Name).string().not_null())
                    .col(ColumnDef::new(Template::Description).string().not_null())
                    .col(ColumnDef::new(Template::TemplateType).string().not_null())
                    .col(ColumnDef::new(Template::IsActive).boolean().not_null().default(true))
                    .col(ColumnDef::new(Template::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Template::UpdatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(ForeignKey::create()
                        .name("fk-template-directory_id")
                        .from(Template::Table, Template::DirectoryId)
                        .to(Directory::Table, Directory::Id))
                    .foreign_key(ForeignKey::create()
                        .name("fk-template-category_id")
                        .from(Template::Table, Template::CategoryId)
                        .to(Category::Table, Category::Id))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Template::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Template {
    Table,
    Id,
    DirectoryId,
    CategoryId,
    Name,
    Description,
    TemplateType,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Directory {
    Table,
    Id,
}

#[derive(Iden)]
enum Category {
    Table,
    Id,
}