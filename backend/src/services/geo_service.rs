//! # GeoService — PostGIS spatial query wrappers (G01)
//!
//! All queries use raw SQL via `sea_orm::Statement` because SeaORM has no
//! native PostGIS column type. Every function is PostGIS-guarded: if the
//! extension is not installed the call returns an empty result set or a
//! descriptive error rather than panicking.
//!
//! ## Supported operations
//!
//! | Function                            | SQL primitive        | Use case                                   |
//! |-------------------------------------|----------------------|--------------------------------------------|
//! | `leads_within_radius`               | `ST_DWithin`         | Find leads near a property or office       |
//! | `nearest_leads`                     | `ST_Distance` ORDER  | Ranked nearest-lead list for a location    |
//! | `set_lead_geo_point`                | `ST_SetSRID/Point`   | Geocode a lead after address resolution    |
//! | `service_areas_containing_point`    | `ST_Contains`        | Routing: which service areas cover a point |
//! | `accounts_within_radius`            | `ST_DWithin`         | Account proximity search                   |
//! | `set_account_geo_point`             | `ST_SetSRID/Point`   | Geocode an account                         |
//! | `check_postgis`                     | `pg_extension`       | Health check / feature flag                |
//!
//! ## Coordinate convention
//!
//! All functions take `(lng, lat)` — matching GeoJSON / ST_Point argument order.
//! Distances are in **metres** (SRID 4326 geography type).

use anyhow::{anyhow, Context, Result};
use sea_orm::{ConnectionTrait, DatabaseConnection, FromQueryResult, Statement};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Result row types ──────────────────────────────────────────────────────────

/// A lead row returned from a geo radius/nearest query.
/// Only the fields needed for map rendering + routing are returned —
/// the caller can fetch the full record by `id` if needed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLead {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub status: String,
    /// Distance in metres from the query origin (only populated for distance queries).
    pub distance_m: Option<f64>,
}

impl FromQueryResult for GeoLead {
    fn from_query_result(row: &sea_orm::QueryResult, _pre: &str) -> Result<Self, sea_orm::DbErr> {
        Ok(Self {
            id: row.try_get("", "id")?,
            tenant_id: row.try_get("", "tenant_id")?,
            name: row.try_get("", "name")?,
            email: row.try_get("", "email").ok(),
            phone: row.try_get("", "phone").ok(),
            city: row.try_get("", "city").ok(),
            state: row.try_get("", "state").ok(),
            country: row.try_get("", "country").ok(),
            status: row.try_get("", "status")?,
            distance_m: row.try_get("", "distance_m").ok(),
        })
    }
}

/// An account row returned from a geo radius query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoAccount {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub city: Option<String>,
    pub country: Option<String>,
    /// Distance in metres from the query origin.
    pub distance_m: Option<f64>,
}

impl FromQueryResult for GeoAccount {
    fn from_query_result(row: &sea_orm::QueryResult, _pre: &str) -> Result<Self, sea_orm::DbErr> {
        Ok(Self {
            id: row.try_get("", "id")?,
            tenant_id: row.try_get("", "tenant_id")?,
            name: row.try_get("", "name")?,
            city: row.try_get("", "city").ok(),
            country: row.try_get("", "country").ok(),
            distance_m: row.try_get("", "distance_m").ok(),
        })
    }
}

// ── GeoService ────────────────────────────────────────────────────────────────

pub struct GeoService;

impl GeoService {
    // ── Feature detection ─────────────────────────────────────────────────────

    /// Returns `true` if the PostGIS extension is installed in the current DB.
    ///
    /// Call this to short-circuit geo operations in environments where PostGIS
    /// hasn't been provisioned (e.g. local dev with vanilla Postgres).
    pub async fn check_postgis(db: &DatabaseConnection) -> bool {
        let stmt = Statement::from_string(
            db.get_database_backend(),
            "SELECT 1 AS ok FROM pg_extension WHERE extname = 'postgis';".to_owned(),
        );
        matches!(db.query_one(stmt).await, Ok(Some(_)))
    }

    // ── Lead geo queries ──────────────────────────────────────────────────────

    /// Find all leads for a tenant whose `geo_point` is within `radius_m` metres
    /// of the given coordinate.
    ///
    /// Uses `geography(Point,4326)` cast for accurate great-circle distance on
    /// the spherical earth model.
    ///
    /// Returns an empty vec (not an error) when PostGIS is not available.
    pub async fn leads_within_radius(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        lng: f64,
        lat: f64,
        radius_m: f64,
    ) -> Result<Vec<GeoLead>> {
        if !Self::check_postgis(db).await {
            tracing::warn!("GeoService::leads_within_radius: PostGIS not available, returning empty");
            return Ok(vec![]);
        }

        let sql = r#"
            SELECT
                id, tenant_id, name, email, phone,
                city, state, country, status,
                ST_Distance(
                    geo_point::geography,
                    ST_SetSRID(ST_MakePoint($3, $4), 4326)::geography
                ) AS distance_m
            FROM atlas_lead
            WHERE
                tenant_id = $1
                AND geo_point IS NOT NULL
                AND ST_DWithin(
                    geo_point::geography,
                    ST_SetSRID(ST_MakePoint($3, $4), 4326)::geography,
                    $5
                )
            ORDER BY distance_m ASC
        "#;

        let rows = db
            .query_all(Statement::from_sql_and_values(
                db.get_database_backend(),
                sql,
                [
                    tenant_id.into(),
                    tenant_id.into(), // unused $2 placeholder align
                    lng.into(),
                    lat.into(),
                    radius_m.into(),
                ],
            ))
            .await
            .context("GeoService::leads_within_radius query failed")?;

        rows.iter()
            .map(|r| GeoLead::from_query_result(r, "").map_err(|e| anyhow!(e)))
            .collect()
    }

    /// Return up to `limit` leads nearest to the given coordinate, regardless of radius.
    ///
    /// Useful for "show closest leads" map widgets.
    pub async fn nearest_leads(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        lng: f64,
        lat: f64,
        limit: u32,
    ) -> Result<Vec<GeoLead>> {
        if !Self::check_postgis(db).await {
            return Ok(vec![]);
        }

        let sql = r#"
            SELECT
                id, tenant_id, name, email, phone,
                city, state, country, status,
                ST_Distance(
                    geo_point::geography,
                    ST_SetSRID(ST_MakePoint($3, $4), 4326)::geography
                ) AS distance_m
            FROM atlas_lead
            WHERE
                tenant_id = $1
                AND geo_point IS NOT NULL
            ORDER BY distance_m ASC
            LIMIT $5
        "#;

        let rows = db
            .query_all(Statement::from_sql_and_values(
                db.get_database_backend(),
                sql,
                [
                    tenant_id.into(),
                    tenant_id.into(),
                    lng.into(),
                    lat.into(),
                    (limit as i64).into(),
                ],
            ))
            .await
            .context("GeoService::nearest_leads query failed")?;

        rows.iter()
            .map(|r| GeoLead::from_query_result(r, "").map_err(|e| anyhow!(e)))
            .collect()
    }

    /// Set the `geo_point` on an `atlas_lead` row from WGS-84 coordinates.
    ///
    /// Idempotent — calling twice with the same coordinates is a no-op at the DB level.
    pub async fn set_lead_geo_point(
        db: &DatabaseConnection,
        lead_id: Uuid,
        lng: f64,
        lat: f64,
    ) -> Result<()> {
        if !Self::check_postgis(db).await {
            return Err(anyhow!("PostGIS not available — cannot set geo_point"));
        }

        let sql = r#"
            UPDATE atlas_lead
            SET geo_point = ST_SetSRID(ST_MakePoint($2, $3), 4326)
            WHERE id = $1
        "#;

        db.execute(Statement::from_sql_and_values(
            db.get_database_backend(),
            sql,
            [lead_id.into(), lng.into(), lat.into()],
        ))
        .await
        .context("GeoService::set_lead_geo_point failed")?;

        tracing::info!(lead_id = %lead_id, lng, lat, "lead geo_point updated");
        Ok(())
    }

    // ── Account geo queries ───────────────────────────────────────────────────

    /// Find all accounts for a tenant within `radius_m` metres of the given point.
    pub async fn accounts_within_radius(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        lng: f64,
        lat: f64,
        radius_m: f64,
    ) -> Result<Vec<GeoAccount>> {
        if !Self::check_postgis(db).await {
            return Ok(vec![]);
        }

        let sql = r#"
            SELECT
                id, tenant_id, name, city, country,
                ST_Distance(
                    geo_point::geography,
                    ST_SetSRID(ST_MakePoint($3, $4), 4326)::geography
                ) AS distance_m
            FROM atlas_accounts
            WHERE
                tenant_id = $1
                AND geo_point IS NOT NULL
                AND ST_DWithin(
                    geo_point::geography,
                    ST_SetSRID(ST_MakePoint($3, $4), 4326)::geography,
                    $5
                )
            ORDER BY distance_m ASC
        "#;

        let rows = db
            .query_all(Statement::from_sql_and_values(
                db.get_database_backend(),
                sql,
                [
                    tenant_id.into(),
                    tenant_id.into(),
                    lng.into(),
                    lat.into(),
                    radius_m.into(),
                ],
            ))
            .await
            .context("GeoService::accounts_within_radius query failed")?;

        rows.iter()
            .map(|r| GeoAccount::from_query_result(r, "").map_err(|e| anyhow!(e)))
            .collect()
    }

    /// Geocode an account — set its `geo_point` from WGS-84 coordinates.
    pub async fn set_account_geo_point(
        db: &DatabaseConnection,
        account_id: Uuid,
        lng: f64,
        lat: f64,
    ) -> Result<()> {
        if !Self::check_postgis(db).await {
            return Err(anyhow!("PostGIS not available — cannot set geo_point"));
        }

        let sql = r#"
            UPDATE atlas_accounts
            SET geo_point = ST_SetSRID(ST_MakePoint($2, $3), 4326)
            WHERE id = $1
        "#;

        db.execute(Statement::from_sql_and_values(
            db.get_database_backend(),
            sql,
            [account_id.into(), lng.into(), lat.into()],
        ))
        .await
        .context("GeoService::set_account_geo_point failed")?;

        tracing::info!(account_id = %account_id, lng, lat, "account geo_point updated");
        Ok(())
    }

    // ── Service area queries ──────────────────────────────────────────────────

    /// Retrieve service areas. If owner_entity_type and owner_entity_id are provided,
    /// filter by them. Otherwise, retrieve all service areas for the tenant.
    pub async fn get_service_areas(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        owner_entity_type: Option<String>,
        owner_entity_id: Option<Uuid>,
    ) -> Result<Vec<crate::entities::geo_service_area::Model>> {
        use sea_orm::QueryFilter;
        use sea_orm::ColumnTrait;
        use sea_orm::EntityTrait;

        let mut query = crate::entities::geo_service_area::Entity::find()
            .filter(crate::entities::geo_service_area::Column::TenantId.eq(tenant_id));

        if let Some(entity_type) = owner_entity_type {
            query = query.filter(crate::entities::geo_service_area::Column::OwnerEntityType.eq(entity_type));
        }

        if let Some(entity_id) = owner_entity_id {
            query = query.filter(crate::entities::geo_service_area::Column::OwnerEntityId.eq(entity_id));
        }

        let results = query
            .all(db)
            .await
            .context("GeoService::get_service_areas failed")?;

        Ok(results)
    }

    /// Find all `geo_service_areas` polygons that **contain** the given point.
    ///
    /// Used for:
    /// - Routing: which PM service area should handle this lead?
    /// - Compliance: is this address within a regulated zone?
    /// - Coverage: does the tenant serve this location?
    pub async fn service_areas_containing_point(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        lng: f64,
        lat: f64,
    ) -> Result<Vec<crate::entities::geo_service_area::Model>> {
        if !Self::check_postgis(db).await {
            return Ok(vec![]);
        }

        let sql = r#"
            SELECT id, tenant_id, owner_entity_type, owner_entity_id, label, geom, point, zip_codes, created_at
            FROM geo_service_areas
            WHERE
                tenant_id = $1
                AND geom IS NOT NULL
                AND ST_Contains(
                    geom,
                    ST_SetSRID(ST_MakePoint($2, $3), 4326)
                )
            ORDER BY label ASC
        "#;

        let rows = db
            .query_all(Statement::from_sql_and_values(
                db.get_database_backend(),
                sql,
                [tenant_id.into(), lng.into(), lat.into()],
            ))
            .await
            .context("GeoService::service_areas_containing_point query failed")?;

        rows.iter()
            .map(|r| crate::entities::geo_service_area::Model::from_query_result(r, "").map_err(|e| anyhow!(e)))
            .collect()
    }

    /// Find service areas whose **centroid point** is within `radius_m` metres.
    ///
    /// Alternative to polygon containment for coarse proximity matching.
    pub async fn service_areas_within_radius(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        lng: f64,
        lat: f64,
        radius_m: f64,
    ) -> Result<Vec<crate::entities::geo_service_area::Model>> {
        if !Self::check_postgis(db).await {
            return Ok(vec![]);
        }

        let sql = r#"
            SELECT id, tenant_id, owner_entity_type, owner_entity_id, label, geom, point, zip_codes, created_at
            FROM geo_service_areas
            WHERE
                tenant_id = $1
                AND point IS NOT NULL
                AND ST_DWithin(
                    point,
                    ST_SetSRID(ST_MakePoint($2, $3), 4326)::geography,
                    $4
                )
            ORDER BY label ASC
        "#;

        let rows = db
            .query_all(Statement::from_sql_and_values(
                db.get_database_backend(),
                sql,
                [tenant_id.into(), lng.into(), lat.into(), radius_m.into()],
            ))
            .await
            .context("GeoService::service_areas_within_radius query failed")?;

        rows.iter()
            .map(|r| crate::entities::geo_service_area::Model::from_query_result(r, "").map_err(|e| anyhow!(e)))
            .collect()
    }

    // ── Batch geocode helper ──────────────────────────────────────────────────

    /// Update `geo_point` for multiple leads in a single query using
    /// a `VALUES` batch — used by geocoding background jobs.
    ///
    /// `points` is `Vec<(lead_id, lng, lat)>`.
    pub async fn batch_set_lead_geo_points(
        db: &DatabaseConnection,
        points: &[(Uuid, f64, f64)],
    ) -> Result<u64> {
        if points.is_empty() {
            return Ok(0);
        }
        if !Self::check_postgis(db).await {
            return Err(anyhow!("PostGIS not available — cannot batch geocode leads"));
        }

        // Build: UPDATE atlas_lead SET geo_point = v.pt
        // FROM (VALUES (...), ...) AS v(id, pt)
        // WHERE atlas_lead.id = v.id
        let mut values_clauses: Vec<String> = Vec::with_capacity(points.len());
        let mut params: Vec<sea_orm::Value> = Vec::with_capacity(points.len() * 3);

        for (i, (lead_id, lng, lat)) in points.iter().enumerate() {
            let base = i * 3 + 1;
            values_clauses.push(format!(
                "(${base}::uuid, ST_SetSRID(ST_MakePoint(${lng_p}, ${lat_p}), 4326))",
                base = base,
                lng_p = base + 1,
                lat_p = base + 2,
            ));
            params.push((*lead_id).into());
            params.push((*lng).into());
            params.push((*lat).into());
        }

        let sql = format!(
            r#"
            UPDATE atlas_lead AS al
            SET geo_point = v.pt
            FROM (VALUES {values}) AS v(id, pt)
            WHERE al.id = v.id
            "#,
            values = values_clauses.join(", "),
        );

        let result = db
            .execute(Statement::from_sql_and_values(
                db.get_database_backend(),
                &sql,
                params,
            ))
            .await
            .context("GeoService::batch_set_lead_geo_points failed")?;

        let rows_affected = result.rows_affected();
        tracing::info!(rows_affected, "GeoService::batch_set_lead_geo_points complete");
        Ok(rows_affected)
    }
}
