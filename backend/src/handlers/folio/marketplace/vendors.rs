//! GET  /api/folio/marketplace/vendors     — list marketplace-visible vendors by proximity + trade
//! GET  /api/folio/marketplace/vendors/{id} — vendor detail card
//!
//! Both routes require any authenticated Folio user (any role can browse the marketplace).
//! Available to Landlord, PropertyManager, and even Tenant (read-only discovery).

use axum::{
    Extension, Json, Router,
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::extractors::tenant::TenantContext;

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/folio/marketplace/vendors", get(list_vendors))
        .route(
            "/api/folio/marketplace/vendors/{id}",
            get(get_vendor_detail),
        )
}

// ── Query params ──────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct VendorSearchParams {
    /// Latitude of the search center (decimal degrees, WGS84)
    pub lat: Option<f64>,
    /// Longitude of the search center (decimal degrees, WGS84)
    pub lng: Option<f64>,
    /// Search radius in kilometers (default: 50)
    pub radius_km: Option<f64>,
    /// Filter by trade type slug (e.g. "plumber", "electrician")
    pub trade_type: Option<String>,
    /// Max results (default: 20, max: 100)
    pub limit: Option<i64>,
}

// ── Response types ────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct VendorCard {
    pub id: Uuid,
    pub business_name: String,
    pub marketplace_bio: Option<String>,
    pub trade_types: Vec<String>,
    pub rating_avg: Option<f64>,
    pub rating_count: i32,
    /// Number of cross-tenant landlord endorsements (trust signal)
    pub endorsement_count: i64,
    /// Distance from the search center in kilometers (None if no geo filter)
    pub distance_km: Option<f64>,
    pub is_insured: bool,
    pub is_bonded: bool,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// List marketplace-visible vendors, optionally filtered by proximity and trade type.
///
/// Query strategy: single raw SQL with optional WHERE clauses.
/// Endorsement count is a subquery correlated on sp.id — indexed via G-22's
/// (target_entity_type, target_entity_id) composite index.
async fn list_vendors(
    // Any authenticated user can browse — no role restriction
    _ctx: TenantContext,
    Query(params): Query<VendorSearchParams>,
    Extension(db): Extension<DatabaseConnection>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(20).min(100);
    let radius_m = params.radius_km.unwrap_or(50.0) * 1000.0;

    // Build the geo filter clause
    let geo_filter = match (params.lat, params.lng) {
        (Some(lat), Some(lng)) => format!(
            "AND ST_DWithin(sp.marketplace_location::geography, ST_MakePoint({lng}, {lat})::geography, {radius_m})"
        ),
        _ => String::new(),
    };

    // Build the distance select expression
    let distance_select = match (params.lat, params.lng) {
        (Some(lat), Some(lng)) => format!(
            "ST_Distance(sp.marketplace_location::geography, ST_MakePoint({lng}, {lat})::geography) / 1000.0 AS distance_km,"
        ),
        _ => "NULL::float AS distance_km,".to_string(),
    };

    // Trade type filter using PostgreSQL @> (array containment)
    let trade_filter = match &params.trade_type {
        Some(t) => format!("AND sp.marketplace_trade_types @> ARRAY['{t}']::text[]"),
        None => String::new(),
    };

    let sql = format!(
        r#"
        SELECT
          sp.id,
          COALESCE(sp.business_name, '') AS business_name,
          sp.marketplace_bio,
          sp.marketplace_trade_types AS trade_types,
          sp.rating_avg,
          sp.rating_count,
          sp.is_insured,
          sp.is_bonded,
          {distance_select}
          (
            SELECT COUNT(*)
            FROM   atlas_record_relationships rr
            WHERE  rr.target_entity_type = 'atlas_service_providers'
              AND  rr.target_entity_id   = sp.id
              AND  rr.relationship_type  = 'marketplace_endorsement'
          ) AS endorsement_count
        FROM   atlas_service_providers sp
        WHERE  sp.is_marketplace_visible = true
          {geo_filter}
          {trade_filter}
        ORDER  BY endorsement_count DESC, sp.rating_avg DESC NULLS LAST
        LIMIT  {limit}
        "#,
    );

    let rows: Vec<sea_orm::QueryResult> = match db
        .query_all(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            sql,
        ))
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(error = %e, "marketplace/vendors: query failed");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let vendors: Vec<VendorCard> = rows
        .into_iter()
        .filter_map(|row| {
            let id: Uuid = row.try_get("", "id").ok()?;
            Some(VendorCard {
                id,
                business_name: row.try_get("", "business_name").unwrap_or_default(),
                marketplace_bio: row.try_get("", "marketplace_bio").ok().flatten(),
                trade_types: row.try_get("", "trade_types").unwrap_or_default(),
                rating_avg: row.try_get("", "rating_avg").ok().flatten(),
                rating_count: row.try_get("", "rating_count").unwrap_or(0),
                endorsement_count: row.try_get("", "endorsement_count").unwrap_or(0),
                distance_km: row.try_get("", "distance_km").ok().flatten(),
                is_insured: row.try_get("", "is_insured").unwrap_or(false),
                is_bonded: row.try_get("", "is_bonded").unwrap_or(false),
            })
        })
        .collect();

    Json(vendors).into_response()
}

/// Get the full marketplace profile for one vendor.
async fn get_vendor_detail(
    _ctx: TenantContext,
    Path(vendor_id): Path<Uuid>,
    Extension(db): Extension<DatabaseConnection>,
) -> impl IntoResponse {
    let sql = format!(
        r#"
        SELECT
          sp.id,
          COALESCE(sp.business_name, '') AS business_name,
          sp.marketplace_bio,
          sp.marketplace_trade_types AS trade_types,
          sp.rating_avg,
          sp.rating_count,
          sp.is_insured,
          sp.is_bonded,
          NULL::float AS distance_km,
          (
            SELECT COUNT(*)
            FROM   atlas_record_relationships rr
            WHERE  rr.target_entity_type = 'atlas_service_providers'
              AND  rr.target_entity_id   = sp.id
              AND  rr.relationship_type  = 'marketplace_endorsement'
          ) AS endorsement_count
        FROM atlas_service_providers sp
        WHERE sp.id = '{id}' AND sp.is_marketplace_visible = true
        "#,
        id = vendor_id,
    );

    let rows: Vec<sea_orm::QueryResult> = match db
        .query_all(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            sql,
        ))
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(error = %e, "marketplace/vendor-detail: query failed");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    match rows.into_iter().next() {
        Some(row) => {
            let id: Uuid = match row.try_get("", "id") {
                Ok(v) => v,
                Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            };
            let card = VendorCard {
                id,
                business_name: row.try_get("", "business_name").unwrap_or_default(),
                marketplace_bio: row.try_get("", "marketplace_bio").ok().flatten(),
                trade_types: row.try_get("", "trade_types").unwrap_or_default(),
                rating_avg: row.try_get("", "rating_avg").ok().flatten(),
                rating_count: row.try_get("", "rating_count").unwrap_or(0),
                endorsement_count: row.try_get("", "endorsement_count").unwrap_or(0),
                distance_km: None,
                is_insured: row.try_get("", "is_insured").unwrap_or(false),
                is_bonded: row.try_get("", "is_bonded").unwrap_or(false),
            };
            Json(card).into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
