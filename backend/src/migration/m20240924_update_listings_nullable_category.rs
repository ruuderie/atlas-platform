use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Make category_id nullable in the listings table
        manager
            .alter_table(
                Table::alter()
                    .table(Listing::Table)
                    .modify_column(
                        ColumnDef::new(Listing::CategoryId)
                            .uuid()
                            .null()
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Revert category_id back to not null
        // Note: This might fail if there are null values in the column
        manager
            .alter_table(
                Table::alter()
                    .table(Listing::Table)
                    .modify_column(
                        ColumnDef::new(Listing::CategoryId)
                            .uuid()
                            .not_null()
                    )
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum Listing {
    Table,
    CategoryId,
} 