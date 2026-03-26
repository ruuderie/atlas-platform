use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Account::Table)
                    .add_column(
                        ColumnDef::new(Account::StripeCustomerId)
                            .string()
                            .null(),
                    )
                    .add_column(
                        ColumnDef::new(Account::StripePaymentMethodId)
                            .string()
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
                    .table(Account::Table)
                    .drop_column(Account::StripeCustomerId)
                    .drop_column(Account::StripePaymentMethodId)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum Account {
    Table,
    StripeCustomerId,
    StripePaymentMethodId,
}
