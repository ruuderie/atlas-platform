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
        // Query to check if PostGIS is available as an extension before trying to create tables using geography types.
        // This is read-only and will never abort the current transaction if the extension is not available.
        let check_res = manager
            .get_connection()
            .query_one(sea_orm::Statement::from_string(
                manager.get_connection().get_database_backend(),
                "SELECT 1 FROM pg_available_extensions WHERE name = 'postgis';".to_owned(),
            ))
            .await;

        let has_postgis = match check_res {
            Ok(Some(_)) => true,
            _ => false,
        };

        let geo_point_type = if has_postgis {
            sea_orm::sea_query::Alias::new("geography(Point, 4326)")
        } else {
            sea_orm::sea_query::Alias::new("text")
        };

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
                    .table(AtlasAssets::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasAssets::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(AtlasAssets::TenantId).uuid().not_null())
                    .col(ColumnDef::new(AtlasAssets::PortfolioId).uuid().null()) // FK to atlas_portfolios
                    .col(ColumnDef::new(AtlasAssets::ParentAssetId).uuid().null()) // Self-referential hierarchy
                    .col(ColumnDef::new(AtlasAssets::OwnerUserId).uuid().null())
                    .col(ColumnDef::new(AtlasAssets::AssetType).string().not_null())
                    // Examples: 'real_estate_property', 'real_estate_unit', 'vehicle', 'equipment', 'hotel_room_type'
                    .col(ColumnDef::new(AtlasAssets::Name).string().not_null())
                    .col(ColumnDef::new(AtlasAssets::SerialOrFolioNumber).string())
                    .col(
                        ColumnDef::new(AtlasAssets::Status)
                            .string_len(30)
                            .not_null()
                            .default(Expr::val("active")),
                    )
                    // Address / location fields
                    .col(ColumnDef::new(AtlasAssets::AddressLine1).string())
                    .col(ColumnDef::new(AtlasAssets::AddressLine2).string())
                    .col(ColumnDef::new(AtlasAssets::City).string())
                    .col(ColumnDef::new(AtlasAssets::StateProvince).string())
                    .col(ColumnDef::new(AtlasAssets::PostalCode).string())
                    .col(ColumnDef::new(AtlasAssets::CountryCode).char_len(2).default(Expr::val("US")))
                    // Geography point (requires PostGIS). Using dynamic/fallback type.
                    .col(ColumnDef::new(AtlasAssets::GeoPoint).custom(geo_point_type).null())
                    // Flexible type-specific attributes (JSONB is the key to avoiding dozens of app-specific tables)
                    .col(ColumnDef::new(AtlasAssets::Attributes).json_binary().null())
                    .col(
                        ColumnDef::new(AtlasAssets::CreatedAt)
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
                    .table(AtlasAssets::Table)
                    .col(AtlasAssets::TenantId)
                    .col(AtlasAssets::AssetType)
                    .col(AtlasAssets::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_assets_parent")
                    .table(AtlasAssets::Table)
                    .col(AtlasAssets::ParentAssetId)
                    .to_owned(),
            )
            .await?;

        // Note: GIST index on geography requires PostGIS. We add it here conditionally.
        if has_postgis {
            manager
                .create_index(
                    Index::create()
                        .name("idx_atlas_assets_geo")
                        .table(AtlasAssets::Table)
                        .col(AtlasAssets::GeoPoint)
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AtlasAssets::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(AtlasAssetStatus::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AtlasAssets {
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
    #[sea_orm(iden = "address_line_1")]
    AddressLine1,
    #[sea_orm(iden = "address_line_2")]
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
