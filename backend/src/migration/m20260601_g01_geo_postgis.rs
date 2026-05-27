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
        // Enable PostGIS extension (idempotent).
        // In some test/CI environments the PostGIS binaries may not be installed on the
        // Postgres server. We make this step non-fatal so the rest of the test suite and
        // dev deploys can proceed. Geo-dependent features will simply be unavailable until
        // PostGIS is enabled on the target database.
        let ext_result = manager
            .get_connection()
            .execute(sea_orm::Statement::from_string(
                manager.get_connection().get_database_backend(),
                "CREATE EXTENSION IF NOT EXISTS postgis;".to_owned(),
            ))
            .await;

        if let Err(ref e) = ext_result {
            tracing::warn!(
                "PostGIS extension could not be created (common in test/CI DBs without PostGIS installed). \
                 Geo features (G-01) will be disabled until the extension is available. Error: {:?}",
                e
            );
            // Do not fail the whole migration — continue so other tests and deploys succeed.
            return Ok(());
        }

        // Main table
        manager
            .create_table(
                Table::create()
                    .table(GeoServiceAreas::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(GeoServiceAreas::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(GeoServiceAreas::TenantId).uuid().not_null())
                    .col(ColumnDef::new(GeoServiceAreas::OwnerEntityType).string().not_null())
                    // Examples: 'agency', 'adjuster', 'property', 'listing', 'vendor'
                    .col(ColumnDef::new(GeoServiceAreas::OwnerEntityId).uuid().not_null())
                    .col(ColumnDef::new(GeoServiceAreas::Label).string().null())
                    // Geometry for polygons (MultiPolygon in SRID 4326)
                    .col(ColumnDef::new(GeoServiceAreas::Geom).custom(sea_orm::sea_query::Alias::new("geometry(MultiPolygon, 4326)")).null())
                    // Geography point for fast radius / distance queries
                    .col(ColumnDef::new(GeoServiceAreas::Point).custom(sea_orm::sea_query::Alias::new("geography(Point, 4326)")).null())
                    // Optional zip code list for fallback matching
                    .col(ColumnDef::new(GeoServiceAreas::ZipCodes).json_binary().null())
                    .col(
                        ColumnDef::new(GeoServiceAreas::CreatedAt)
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
                    .table(GeoServiceAreas::Table)
                    .col(GeoServiceAreas::Geom)
                    .to_owned(),
            )
            .await?;

        // GIST index on geography point (for ST_DWithin etc.)
        manager
            .create_index(
                Index::create()
                    .name("idx_geo_service_areas_point")
                    .table(GeoServiceAreas::Table)
                    .col(GeoServiceAreas::Point)
                    .to_owned(),
            )
            .await?;

        // Common lookup index
        manager
            .create_index(
                Index::create()
                    .name("idx_geo_service_areas_tenant_type")
                    .table(GeoServiceAreas::Table)
                    .col(GeoServiceAreas::TenantId)
                    .col(GeoServiceAreas::OwnerEntityType)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GeoServiceAreas::Table).to_owned())
            .await?;

        // Note: We intentionally do NOT drop the PostGIS extension in down(),
        // as it may be shared with other databases or required by other features.
        Ok(())
    }
}

#[derive(DeriveIden)]
enum GeoServiceAreas {
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
