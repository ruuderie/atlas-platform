#![allow(dead_code, unused_imports)]
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;

/// GENERIC-10: AtlasAsset
/// The central physical/digital asset registry for the platform.
///
/// Replaces the need for per-app tables like pm_properties, pm_units, vehicle tables, equipment registers, etc.
///
/// Key design:
/// - `asset_type` acts as a discriminator (real_estate_property, real_estate_unit, vehicle, etc.)
/// - `parent_asset_id` enables hierarchy (Property → Units, Fleet → Trucks, etc.)
/// - `attributes` JSONB holds strongly-typed data per asset_type (defined in app service layers)
/// - `geo_point` links to GENERIC-01 (atlas_geo / PostGIS)
///
/// This is one of the most important new platform generics.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_assets")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub portfolio_id: Option<Uuid>,      // FK to atlas_portfolios (GENERIC-09)
    pub parent_asset_id: Option<Uuid>,   // Self-referential for hierarchy
    pub owner_user_id: Option<Uuid>,
    pub asset_type: String,
    pub name: String,
    pub serial_or_folio_number: Option<String>,
    pub status: String,                  // Backed by atlas_asset_status enum in DB
    pub address_line_1: Option<String>,
    pub address_line_2: Option<String>,
    pub city: Option<String>,
    pub state_province: Option<String>,
    pub postal_code: Option<String>,
    pub country_code: Option<String>,
    /// Stored as GEOGRAPHY(Point, 4326) in the database (requires PostGIS / GENERIC-01).
    /// For the initial POC we represent it as String on the Rust side.
    /// Full geo support (with the `geo` + `postgis` crates) can be added later without schema change.
    pub geo_point: Option<String>,
    /// Flexible attributes. Each asset_type defines its own Rust struct that serializes here.
    /// Example for real_estate_unit:
    /// { "bedrooms": 2, "bathrooms": 1.5, "sqft": 950, "floor": 4 }
    pub attributes: Option<Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
