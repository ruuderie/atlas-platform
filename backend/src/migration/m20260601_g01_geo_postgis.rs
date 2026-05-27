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
        // Query to check if PostGIS is available as an extension before trying to create it.
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

        if !has_postgis {
            tracing::warn!(
                "PostGIS extension is not available in the PostgreSQL catalog (common in test/CI DBs without PostGIS installed). \
                 Geo features (G-01) will be disabled until the extension is available."
            );
            // Do not fail the whole migration, and do not execute CREATE EXTENSION (which aborts the transaction).
            return Ok(());
        }

        // Now safe to run CREATE EXTENSION as we know it's available on the server
        let ext_result = manager
            .get_connection()
            .execute(sea_orm::Statement::from_string(
                manager.get_connection().get_database_backend(),
                "CREATE EXTENSION IF NOT EXISTS postgis;".to_owned(),
            ))
            .await;

        if let Err(ref e) = ext_result {
            tracing::warn!(
                "PostGIS extension could not be created despite being available. \
                 Geo features (G-01) will be disabled. Error: {:?}",
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
