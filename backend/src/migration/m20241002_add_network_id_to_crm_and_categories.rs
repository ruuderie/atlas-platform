use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
enum Category {
    Table,
    NetworkId,
}

#[derive(DeriveIden)]
enum Lead {
    Table,
    NetworkId,
}

#[derive(DeriveIden)]
enum Contact {
    Table,
    NetworkId,
}

#[derive(DeriveIden)]
enum Customer {
    Table,
    NetworkId,
}

#[derive(DeriveIden)]
enum Deal {
    Table,
    NetworkId,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add network_id to Category
        manager
            .alter_table(
                Table::alter()
                    .table(Category::Table)
                    .add_column(ColumnDef::new(Category::NetworkId).uuid().null())
                    .to_owned(),
            )
            .await?;

        // Add network_id to Lead
        manager
            .alter_table(
                Table::alter()
                    .table(Lead::Table)
                    .add_column(ColumnDef::new(Lead::NetworkId).uuid().null())
                    .to_owned(),
            )
            .await?;

        // Add network_id to Contact
        manager
            .alter_table(
                Table::alter()
                    .table(Contact::Table)
                    .add_column(ColumnDef::new(Contact::NetworkId).uuid().null())
                    .to_owned(),
            )
            .await?;

        // Add network_id to Customer
        manager
            .alter_table(
                Table::alter()
                    .table(Customer::Table)
                    .add_column(ColumnDef::new(Customer::NetworkId).uuid().null())
                    .to_owned(),
            )
            .await?;

        // Add network_id to Deal
        manager
            .alter_table(
                Table::alter()
                    .table(Deal::Table)
                    .add_column(ColumnDef::new(Deal::NetworkId).uuid().null())
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
                    .drop_column(Category::NetworkId)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Lead::Table)
                    .drop_column(Lead::NetworkId)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Contact::Table)
                    .drop_column(Contact::NetworkId)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Customer::Table)
                    .drop_column(Customer::NetworkId)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Deal::Table)
                    .drop_column(Deal::NetworkId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
