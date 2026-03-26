use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(LeadCharge::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(LeadCharge::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(LeadCharge::AccountId).uuid().not_null())
                    .col(ColumnDef::new(LeadCharge::LeadId).uuid().not_null())
                    .col(ColumnDef::new(LeadCharge::AmountCents).integer().not_null())
                    .col(ColumnDef::new(LeadCharge::Status).string().not_null())
                    .col(ColumnDef::new(LeadCharge::CreatedAt).timestamp_with_time_zone().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(LeadCharge::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum LeadCharge {
    Table,
    Id,
    AccountId,
    LeadId,
    AmountCents,
    Status,
    CreatedAt,
}
