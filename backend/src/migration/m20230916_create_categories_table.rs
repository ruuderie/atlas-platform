use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Category::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Category::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Category::DirectoryTypeId).uuid().not_null())
                    .col(ColumnDef::new(Category::ParentCategoryId).uuid())
                    .col(ColumnDef::new(Category::Name).string().not_null())
                    .col(ColumnDef::new(Category::Description).string().not_null())
                    .col(ColumnDef::new(Category::IsCustom).boolean().not_null())
                    .col(ColumnDef::new(Category::IsActive).boolean().not_null().default(true))
                    .col(ColumnDef::new(Category::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Category::UpdatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(ForeignKey::create()
                        .name("fk-category-directory_type_id")
                        .from(Category::Table, Category::DirectoryTypeId)
                        .to(DirectoryType::Table, DirectoryType::Id))
                    .foreign_key(ForeignKey::create()
                        .name("fk-category-parent_category_id")
                        .from(Category::Table, Category::ParentCategoryId)
                        .to(Category::Table, Category::Id))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Category::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Category {
    Table,
    Id,
    DirectoryTypeId,
    ParentCategoryId,
    Name,
    Description,
    IsCustom,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum DirectoryType {
    Table,
    Id,
}