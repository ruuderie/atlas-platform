use sea_orm_migration::prelude::*;
use sea_orm_migration::prelude::extension::postgres::Type;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// GENERIC-10: atlas_assets
/// Physical/Digital Asset Registry (real_estate_property, real_estate_unit, vehicle, equipment, hotel_room_type, etc.)
///
/// This is a foundational domain generic. It replaces app-specific property/unit/asset tables.
/// Supports parent-child hierarchy (e.g. property → units) and flexible attributes via JSONB.
///
/// Depends on:
/// - atlas_portfolios (GENERIC-09) for portfolio_id
/// - PostGIS (GENERIC-01) for geo_point (extension must be enabled before this runs in production)
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create the custom enum type for asset status
        manager
            .create_type(
                Type::create()
                    .as_enum(AtlasAssetStatus::Table)
                    .values([
                        AtlasAssetStatus::Active,
                        AtlasAssetStatus::Inactive,
                        AtlasAssetStatus::UnderMaintenance,
                        AtlasAssetStatus::ListedForSale,
                        AtlasAssetStatus::Decommissioned,
                        AtlasAssetStatus::PendingInspection,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AtlasAsset::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasAsset::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(AtlasAsset::TenantId).uuid().not_null())
                    .col(ColumnDef::new(AtlasAsset::PortfolioId).uuid().null()) // FK to atlas_portfolios
                    .col(ColumnDef::new(AtlasAsset::ParentAssetId).uuid().null()) // Self-referential hierarchy
                    .col(ColumnDef::new(AtlasAsset::OwnerUserId).uuid().null())
                    .col(ColumnDef::new(AtlasAsset::AssetType).string().not_null())
                    // Examples: 'real_estate_property', 'real_estate_unit', 'vehicle', 'equipment', 'hotel_room_type'
                    .col(ColumnDef::new(AtlasAsset::Name).string().not_null())
                    .col(ColumnDef::new(AtlasAsset::SerialOrFolioNumber).string())
                    .col(
                        ColumnDef::new(AtlasAsset::Status)
                            .custom(AtlasAssetStatus::Table)
                            .not_null()
                            .default(Expr::val("active")),
                    )
                    // Address / location fields
                    .col(ColumnDef::new(AtlasAsset::AddressLine1).string())
                    .col(ColumnDef::new(AtlasAsset::AddressLine2).string())
                    .col(ColumnDef::new(AtlasAsset::City).string())
                    .col(ColumnDef::new(AtlasAsset::StateProvince).string())
                    .col(ColumnDef::new(AtlasAsset::PostalCode).string())
                    .col(ColumnDef::new(AtlasAsset::CountryCode).char_len(2).default(Expr::val("US")))
                    // Geography point (requires PostGIS). Using custom type for now.
                    .col(ColumnDef::new(AtlasAsset::GeoPoint).custom(sea_orm::sea_query::Alias::new("geography(Point, 4326)")).null())
                    // Flexible type-specific attributes (JSONB is the key to avoiding dozens of app-specific tables)
                    .col(ColumnDef::new(AtlasAsset::Attributes).json_binary().null())
                    .col(
                        ColumnDef::new(AtlasAsset::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_assets_tenant_type_status")
                    .table(AtlasAsset::Table)
                    .col(AtlasAsset::TenantId)
                    .col(AtlasAsset::AssetType)
                    .col(AtlasAsset::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_assets_parent")
                    .table(AtlasAsset::Table)
                    .col(AtlasAsset::ParentAssetId)
                    .to_owned(),
            )
            .await?;

        // Note: GIST index on geography requires PostGIS. We add it here; it will only work after G-01 is applied.
        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_assets_geo")
                    .table(AtlasAsset::Table)
                    .col(AtlasAsset::GeoPoint)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AtlasAsset::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(AtlasAssetStatus::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AtlasAsset {
    Table,
    Id,
    TenantId,
    PortfolioId,
    ParentAssetId,
    OwnerUserId,
    AssetType,
    Name,
    SerialOrFolioNumber,
    Status,
    AddressLine1,
    AddressLine2,
    City,
    StateProvince,
    PostalCode,
    CountryCode,
    GeoPoint,
    Attributes,
    CreatedAt,
}

#[derive(DeriveIden)]
enum AtlasAssetStatus {
    Table,
    Active,
    Inactive,
    UnderMaintenance,
    ListedForSale,
    Decommissioned,
    PendingInspection,
}
