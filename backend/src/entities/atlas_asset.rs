#![allow(dead_code, unused_imports)]
use chrono::{DateTime, NaiveDate, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// GENERIC-10: AtlasAsset
/// The central physical/digital asset registry for the platform.
///
/// Replaces the need for per-app tables like pm_properties, pm_units, vehicle tables, equipment registers, etc.
///
/// Key design:
/// - `asset_type` acts as a discriminator (real_estate_property, real_estate_unit, appliance, vehicle, etc.)
/// - `parent_asset_id` enables hierarchy (Property → Units → Appliances, Fleet → Trucks, etc.)
/// - `attributes` JSONB holds strongly-typed spatial/financial data per asset_type
/// - `lifecycle_metadata` JSONB holds app-owned typed maintenance/identity data (see asset_metadata_shapes.md)
/// - `scheduled_service_date` + `expiry_date` are indexed first-class columns for alert queries
/// - `geo_point` links to GENERIC-01 (atlas_geo / PostGIS)
///
/// G-10 Lifecycle Extension (m20260900):
/// Added `scheduled_service_date`, `expiry_date`, `condition`, `lifecycle_metadata`
/// to support universal asset lifecycle tracking without per-vertical extension tables.
/// See: docs/architecture/platform_generics_v3.md §4 and §8 Risk #8
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_assets")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub portfolio_id: Option<Uuid>, // FK to atlas_portfolios (GENERIC-09)
    pub parent_asset_id: Option<Uuid>, // Self-referential for hierarchy
    pub owner_user_id: Option<Uuid>,
    pub asset_type: String,
    pub name: String,
    pub serial_or_folio_number: Option<String>,
    pub status: String, // Backed by atlas_asset_status enum in DB
    pub address_line_1: Option<String>,
    pub address_line_2: Option<String>,
    pub city: Option<String>,
    pub state_province: Option<String>,
    pub postal_code: Option<String>,
    pub country_code: Option<String>,
    /// Stored as GEOGRAPHY(Point, 4326) in the database (requires PostGIS / GENERIC-01).
    pub geo_point: Option<String>,
    /// Flexible spatial/financial attributes per asset_type.
    pub attributes: Option<Value>,

    // ── G-10 Lifecycle Extension (m20260900) ─────────────────────────────────
    /// Next scheduled maintenance / calibration / inspection due date.
    /// Indexed. Written by each app; queried platform-wide for alert generation.
    pub scheduled_service_date: Option<NaiveDate>,

    /// Warranty / cert / license / registration expiry date.
    /// Indexed. Written by each app; queried platform-wide for expiry alerts.
    pub expiry_date: Option<NaiveDate>,

    /// Current operational state. Validated at the service layer per app.
    /// Standard values: "excellent" | "good" | "fair" | "poor" | "retired"
    pub condition: Option<String>,

    /// App-owned typed metadata sidecar. Shape varies per `asset_type`.
    /// Keys treated as stable public API — rename via versioned backfill migration.
    /// See: docs/architecture/asset_metadata_shapes.md for registered shapes.
    pub lifecycle_metadata: Option<Value>,

    // ── STR (Short-Term Rental) Traits (m20261007) ────────────────────────────
    //
    // STR capability is a PROPERTY trait, not a user persona trait.
    // A Landlord with str_eligible assets gets the STR nav sections shown
    // dynamically via `has_str_assets` in SessionInfo. No role change needed.
    /// Landlord has opted this property into STR and accepted the STR terms.
    /// Gate for all STR functionality on this asset. Default: false.
    pub str_eligible: bool,

    /// Listing is live on the Folio platform marketplace (visible to cohosts
    /// and guests searching). Requires str_eligible = true.
    pub str_listing_active: bool,

    /// Property is syndicated to the Cohost Network (internal STR marketplace).
    /// Requires str_listing_active = true.
    /// External OTA channels (Airbnb, VRBO) tracked separately in a future
    /// `str_channels TEXT[]` column.
    pub str_syndicated: bool,

    /// Audit timestamp of when str_eligible was first set true.
    #[sea_orm(column_type = "TimestampWithTimeZone", nullable)]
    pub str_terms_accepted_at: Option<DateTime<Utc>>,

    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
