use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AdPurchase::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(AdPurchase::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(AdPurchase::ListingId).uuid().not_null())
                    .col(ColumnDef::new(AdPurchase::ProfileId).uuid().not_null())
                    .col(ColumnDef::new(AdPurchase::StartDate).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(AdPurchase::EndDate).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(AdPurchase::Price).float().not_null())
                    .col(ColumnDef::new(AdPurchase::Status).string().not_null())
                    .col(ColumnDef::new(AdPurchase::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(AdPurchase::UpdatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(ForeignKey::create()
                        .name("fk-ad_purchase-listing_id")
                        .from(AdPurchase::Table, AdPurchase::ListingId)
                        .to(Listing::Table, Listing::Id))
                    .foreign_key(ForeignKey::create()
                        .name("fk-ad_purchase-profile_id")
                        .from(AdPurchase::Table, AdPurchase::ProfileId)
                        .to(Profile::Table, Profile::Id))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AdPurchase::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum AdPurchase {
    Table,
    Id,
    ListingId,
    ProfileId,
    StartDate,
    EndDate,
    Price,
    Status,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Listing {
    Table,
    Id,
}

#[derive(Iden)]
enum Profile {
    Table,
    Id,
}