use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// GENERIC-09: atlas_portfolios
/// Asset Portfolio Grouping (real_estate, vehicle_fleet, equipment, insurance_book, etc.)
///
/// This is the first of the new domain generics (Round 2 from Property Management analysis).
/// It allows grouping assets for reporting, billing, and delegated access control.
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AtlasPortfolios::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasPortfolios::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(AtlasPortfolios::TenantId).uuid().not_null())
                    .col(ColumnDef::new(AtlasPortfolios::OwnerUserId).uuid().not_null())
                    .col(ColumnDef::new(AtlasPortfolios::PortfolioType).string().not_null())
                    // Examples: 'real_estate', 'vehicle_fleet', 'equipment', 'insurance_book', 'adjuster_pool'
                    .col(ColumnDef::new(AtlasPortfolios::Name).string().not_null())
                    .col(ColumnDef::new(AtlasPortfolios::Description).text())
                    .col(ColumnDef::new(AtlasPortfolios::Metadata).json_binary())
                    .col(
                        ColumnDef::new(AtlasPortfolios::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Efficient lookup by owner + type within a tenant
        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_portfolios_owner_type")
                    .table(AtlasPortfolios::Table)
                    .col(AtlasPortfolios::TenantId)
                    .col(AtlasPortfolios::OwnerUserId)
                    .col(AtlasPortfolios::PortfolioType)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AtlasPortfolios::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AtlasPortfolios {
    Table,
    Id,
    TenantId,
    OwnerUserId,
    PortfolioType,
    Name,
    Description,
    Metadata,
    CreatedAt,
}
