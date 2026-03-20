use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Listing::Table)
                    .add_column(
                        ColumnDef::new(Listing::Slug)
                            .text()
                            .null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Listing::Table)
                    .drop_column(Listing::Slug)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Listing {
    Table,
    Slug,
}
