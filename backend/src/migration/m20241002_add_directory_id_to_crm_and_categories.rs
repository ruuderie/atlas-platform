use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
enum Category {
    Table,
    DirectoryId,
}

#[derive(DeriveIden)]
enum Lead {
    Table,
    DirectoryId,
}

#[derive(DeriveIden)]
enum Contact {
    Table,
    DirectoryId,
}

#[derive(DeriveIden)]
enum Customer {
    Table,
    DirectoryId,
}

#[derive(DeriveIden)]
enum Deal {
    Table,
    DirectoryId,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add directory_id to Category
        manager
            .alter_table(
                Table::alter()
                    .table(Category::Table)
                    .add_column(ColumnDef::new(Category::DirectoryId).uuid().null())
                    .to_owned(),
            )
            .await?;

        // Add directory_id to Lead
        manager
            .alter_table(
                Table::alter()
                    .table(Lead::Table)
                    .add_column(ColumnDef::new(Lead::DirectoryId).uuid().null())
                    .to_owned(),
            )
            .await?;

        // Add directory_id to Contact
        manager
            .alter_table(
                Table::alter()
                    .table(Contact::Table)
                    .add_column(ColumnDef::new(Contact::DirectoryId).uuid().null())
                    .to_owned(),
            )
            .await?;

        // Add directory_id to Customer
        manager
            .alter_table(
                Table::alter()
                    .table(Customer::Table)
                    .add_column(ColumnDef::new(Customer::DirectoryId).uuid().null())
                    .to_owned(),
            )
            .await?;

        // Add directory_id to Deal
        manager
            .alter_table(
                Table::alter()
                    .table(Deal::Table)
                    .add_column(ColumnDef::new(Deal::DirectoryId).uuid().null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Category::Table)
                    .drop_column(Category::DirectoryId)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Lead::Table)
                    .drop_column(Lead::DirectoryId)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Contact::Table)
                    .drop_column(Contact::DirectoryId)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Customer::Table)
                    .drop_column(Customer::DirectoryId)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Deal::Table)
                    .drop_column(Deal::DirectoryId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
