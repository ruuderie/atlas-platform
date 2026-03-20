use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Category::Table)
                    .add_column(
                        ColumnDef::new(Category::Icon)
                            .string()
                            .null()
                    )
                    .add_column(
                        ColumnDef::new(Category::Slug)
                            .string()
                            .null()
                            .unique_key()
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Category::Table)
                    .drop_column(Category::Icon)
                    .drop_column(Category::Slug)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum Category {
    Table,
    Icon,
    Slug,
}
