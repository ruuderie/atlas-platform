use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// GENERIC-01: atlas_geo — Spatial / PostGIS Foundation
///
/// Enables geographic queries across the platform (service areas, geofencing, proximity search).
/// This is a foundational infrastructure generic.
///
/// Note: `CREATE EXTENSION postgis` requires appropriate database privileges.
/// In managed environments this is often done once at the cluster level.
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Enable PostGIS extension (idempotent)
        manager
            .get_connection()
            .execute(sea_orm::Statement::from_string(
                manager.get_connection().get_database_backend(),
                "CREATE EXTENSION IF NOT EXISTS postgis;".to_owned(),
            ))
            .await?;

        // Main table
        manager
            .create_table(
                Table::create()
                    .table(GeoServiceArea::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(GeoServiceArea::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(GeoServiceArea::TenantId).uuid().not_null())
                    .col(ColumnDef::new(GeoServiceArea::OwnerEntityType).string().not_null())
                    // Examples: 'agency', 'adjuster', 'property', 'listing', 'vendor'
                    .col(ColumnDef::new(GeoServiceArea::OwnerEntityId).uuid().not_null())
                    .col(ColumnDef::new(GeoServiceArea::Label).string().null())
                    // Geometry for polygons (MultiPolygon in SRID 4326)
                    .col(ColumnDef::new(GeoServiceArea::Geom).custom(sea_orm::sea_query::Alias::new("geometry(MultiPolygon, 4326)")).null())
                    // Geography point for fast radius / distance queries
                    .col(ColumnDef::new(GeoServiceArea::Point).custom(sea_orm::sea_query::Alias::new("geography(Point, 4326)")).null())
                    // Optional zip code list for fallback matching
                    .col(ColumnDef::new(GeoServiceArea::ZipCodes).json_binary().null())
                    .col(
                        ColumnDef::new(GeoServiceArea::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // GIST index on geometry (polygons)
        manager
            .create_index(
                Index::create()
                    .name("idx_geo_service_areas_geom")
                    .table(GeoServiceArea::Table)
                    .col(GeoServiceArea::Geom)
                    .to_owned(),
            )
            .await?;

        // GIST index on geography point (for ST_DWithin etc.)
        manager
            .create_index(
                Index::create()
                    .name("idx_geo_service_areas_point")
                    .table(GeoServiceArea::Table)
                    .col(GeoServiceArea::Point)
                    .to_owned(),
            )
            .await?;

        // Common lookup index
        manager
            .create_index(
                Index::create()
                    .name("idx_geo_service_areas_tenant_type")
                    .table(GeoServiceArea::Table)
                    .col(GeoServiceArea::TenantId)
                    .col(GeoServiceArea::OwnerEntityType)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GeoServiceArea::Table).to_owned())
            .await?;

        // Note: We intentionally do NOT drop the PostGIS extension in down(),
        // as it may be shared with other databases or required by other features.
        Ok(())
    }
}

#[derive(DeriveIden)]
enum GeoServiceArea {
    Table,
    Id,
    TenantId,
    OwnerEntityType,
    OwnerEntityId,
    Label,
    Geom,
    Point,
    ZipCodes,
    CreatedAt,
}
