#![allow(dead_code, unused_imports)]
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// GENERIC-01: GeoServiceArea
///
/// Represents a geographic service area (polygon) or point of interest for a tenant.
/// Used for geofencing, proximity search, coverage mapping, etc.
///
/// Note on geometry storage:
/// - `geom` and `point` are stored using PostGIS types (GEOMETRY / GEOGRAPHY).
/// - For the initial POC implementation we represent them as String (WKT) in Rust.
/// - Full integration with the `geo` + `postgis` crates (with proper type mapping)
///   can be added later without a breaking schema change.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "geo_service_areas")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub owner_entity_type: String,
    pub owner_entity_id: Uuid,
    pub label: Option<String>,
    /// PostGIS GEOMETRY(MultiPolygon, 4326) — stored as WKT string in this POC.
    pub geom: Option<String>,
    /// PostGIS GEOGRAPHY(Point, 4326) — stored as WKT string in this POC.
    pub point: Option<String>,
    pub zip_codes: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
