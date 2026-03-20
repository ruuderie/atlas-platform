use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ListingAbTest::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(ListingAbTest::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(ListingAbTest::ListingId).uuid().not_null())
                    .col(ColumnDef::new(ListingAbTest::Status).string().not_null())
                    .col(ColumnDef::new(ListingAbTest::TrafficSplitStrategy).string().not_null())
                    .col(ColumnDef::new(ListingAbTest::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(ListingAbTest::UpdatedAt).timestamp_with_time_zone().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ListingAbVariant::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(ListingAbVariant::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(ListingAbVariant::TestId).uuid().not_null())
                    .col(ColumnDef::new(ListingAbVariant::Name).string().not_null())
                    .col(ColumnDef::new(ListingAbVariant::IsControl).boolean().not_null().default(false))
                    .col(ColumnDef::new(ListingAbVariant::Views).integer().not_null().default(0))
                    .col(ColumnDef::new(ListingAbVariant::Conversions).integer().not_null().default(0))
                    .col(ColumnDef::new(ListingAbVariant::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(ListingAbVariant::UpdatedAt).timestamp_with_time_zone().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ListingAbVariant::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(ListingAbTest::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum ListingAbTest {
    Table,
    Id,
    ListingId,
    Status,
    TrafficSplitStrategy,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
pub enum ListingAbVariant {
    Table,
    Id,
    TestId,
    Name,
    IsControl,
    Views,
    Conversions,
    CreatedAt,
    UpdatedAt,
}
